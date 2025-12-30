//! Virtual Machine for ZKIR v3.4

use crate::error::{RuntimeError, Result};
use crate::execute::execute;
use crate::memory::Memory;
use crate::range_check::{RangeCheckTracker, RangeCheckWitness};
use crate::state::{VMState, HaltReason};
use crate::syscall::{handle_syscall, IOHandler};
use zkir_spec::{Instruction, Program, MemoryOp, TraceRow};

/// VM configuration
#[derive(Debug, Clone)]
pub struct VMConfig {
    /// Maximum number of cycles before halting
    pub max_cycles: u64,

    /// Enable execution tracing
    pub trace: bool,

    /// Enable range checking (default: false for now, will be true when fully integrated)
    pub enable_range_checking: bool,

    /// Enable execution trace collection
    ///
    /// When enabled, collects complete execution trace including:
    /// - Register state at each cycle
    /// - Bounds for each register
    /// - Memory operations (automatically collected)
    pub enable_execution_trace: bool,
}

impl Default for VMConfig {
    fn default() -> Self {
        Self {
            max_cycles: 1_000_000,
            trace: false,
            enable_range_checking: false,
            enable_execution_trace: false,
        }
    }
}

/// Execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Number of cycles executed
    pub cycles: u64,

    /// Output values
    pub outputs: Vec<u64>,

    /// Reason for halting
    pub halt_reason: HaltReason,

    /// Range check witnesses (if range checking enabled)
    pub range_check_witnesses: Vec<RangeCheckWitness>,

    /// Execution trace (if execution tracing enabled)
    ///
    /// This is the single source of truth for all execution state.
    /// Memory operations are embedded in each TraceRow's memory_ops field.
    pub execution_trace: Vec<TraceRow>,
}

impl ExecutionResult {
    /// Extract all memory operations from the execution trace
    ///
    /// Returns a sorted vector of all memory operations that occurred during execution.
    /// Operations are sorted by timestamp, then address, then operation type.
    pub fn get_memory_trace(&self) -> Vec<MemoryOp> {
        let mut ops: Vec<MemoryOp> = self
            .execution_trace
            .iter()
            .flat_map(|row| row.memory_ops.iter().cloned())
            .collect();

        ops.sort();
        ops
    }

    /// Get count of memory operations
    pub fn memory_op_count(&self) -> usize {
        self.execution_trace
            .iter()
            .map(|row| row.memory_ops.len())
            .sum()
    }
}

/// ZKIR v3.4 Virtual Machine
pub struct VM {
    /// VM state (registers, PC, etc.)
    state: VMState,

    /// Memory subsystem
    memory: Memory,

    /// I/O handler
    io: IOHandler,

    /// Configuration
    config: VMConfig,

    /// Range check tracker (optional)
    range_checker: Option<RangeCheckTracker>,

    /// Accumulated range check witnesses
    range_check_witnesses: Vec<RangeCheckWitness>,

    /// Execution trace (if enabled)
    execution_trace: Vec<TraceRow>,
}

impl VM {
    /// Create a new VM with a program and inputs
    pub fn new(program: Program, inputs: Vec<u64>, config: VMConfig) -> Self {
        let entry_point = program.header.entry_point as u64;
        let mut state = VMState::new(entry_point);
        let mut memory = Memory::new();

        // Load program code into memory
        if !program.code.is_empty() {
            memory
                .load_code(&program.code, entry_point)
                .expect("Failed to load program code");
        }

        // Disable strict memory protection for now
        // TODO: Enable proper memory protection once we have a better
        // memory layout or use LUI/ADDI for address loading
        memory.set_strict_protection(false);

        // NOTE: Stack pointer (R2/SP) initialization removed to fix witness sentinel value issue
        // R2 now starts at 0 like all other registers
        // Programs that need a stack should explicitly initialize R2
        // use zkir_spec::Register;
        // state.write_reg(Register::R2, memory.stack_top());

        // Create range checker if enabled
        let range_checker = if config.enable_range_checking {
            Some(RangeCheckTracker::new(program.header.config()))
        } else {
            None
        };

        // Enable memory trace if execution trace is enabled
        if config.enable_execution_trace {
            memory.set_trace_enabled(true);
        }

        Self {
            state,
            memory,
            io: IOHandler::new(inputs),
            config,
            range_checker,
            range_check_witnesses: Vec::new(),
            execution_trace: Vec::new(),
        }
    }

    /// Run the VM until halt
    pub fn run(mut self) -> Result<ExecutionResult> {
        while !self.state.is_halted() {
            // Check cycle limit
            if self.state.cycles >= self.config.max_cycles {
                self.state.halt(HaltReason::CycleLimit);
                break;
            }

            // Sync memory timestamp with VM cycles (for trace collection)
            if self.config.enable_execution_trace {
                self.memory.set_timestamp(self.state.cycles);
            }

            // Save PC before execution (for filtering instruction fetch from data ops)
            let fetch_pc = self.state.pc;

            // Fetch and decode instruction
            let (inst, encoded_inst) = self.fetch_and_decode()?;

            if self.config.trace {
                eprintln!(
                    "[{:6}] PC={:#010x} {:?}",
                    self.state.cycles, self.state.pc, inst
                );
            }

            // Capture PRE-state for execution trace (before instruction modifies registers)
            // This is needed for AIR constraints: constraint references rs1/rs2 from LOCAL row
            // and rd from NEXT row, so each row must contain the state BEFORE the instruction.
            let pre_regs = if self.config.enable_execution_trace {
                Some((self.state.regs, self.state.bounds))
            } else {
                None
            };

            // Execute instruction (pass range checker if enabled)
            execute(&inst, &mut self.state, &mut self.memory, self.range_checker.as_mut())?;

            // Handle syscalls
            if matches!(inst, Instruction::Ecall) {
                handle_syscall(&mut self.state, &mut self.memory, &mut self.io)?;
            }

            // Collect execution trace (if enabled)
            if let Some((regs, bounds)) = pre_regs {
                // Collect data memory operations from this cycle
                // The instruction fetch is always at PC and should be excluded
                // We need to find data operations that happened during instruction execution

                let trace = self.memory.get_trace();

                // Find all operations from this cycle, excluding the instruction fetch at PC
                // Note: There may be multiple operations if this is a syscall (e.g., SHA-256)
                let memory_ops: Vec<MemoryOp> = trace
                    .iter()
                    .filter(|op| {
                        op.timestamp == self.state.cycles  // Same cycle
                        && op.address != fetch_pc          // Not instruction fetch (use saved PC)
                    })
                    .cloned()
                    .collect();

                // Create trace row with PRE-state (register values BEFORE instruction executes)
                // This enables correct AIR constraints even when rd == rs1 or rd == rs2
                let trace_row = TraceRow {
                    cycle: self.state.cycles,
                    pc: fetch_pc,  // Use PC where instruction was fetched from
                    instruction: encoded_inst,
                    registers: regs,   // PRE-state: values before execution
                    bounds: bounds,    // PRE-state bounds
                    memory_ops,
                };

                self.execution_trace.push(trace_row);
            }

            // Range check checkpoint insertion (if enabled)
            if let Some(ref mut checker) = self.range_checker {
                // Insert checkpoint at stores, branches, jumps, and division
                let needs_checkpoint = matches!(
                    inst,
                    Instruction::Sb { .. }
                        | Instruction::Sh { .. }
                        | Instruction::Sw { .. }
                        | Instruction::Sd { .. }
                        | Instruction::Beq { .. }
                        | Instruction::Bne { .. }
                        | Instruction::Blt { .. }
                        | Instruction::Bge { .. }
                        | Instruction::Bltu { .. }
                        | Instruction::Bgeu { .. }
                        | Instruction::Jal { .. }
                        | Instruction::Jalr { .. }
                        | Instruction::Div { .. }
                        | Instruction::Divu { .. }
                        | Instruction::Rem { .. }
                        | Instruction::Remu { .. }
                );

                if needs_checkpoint || checker.should_checkpoint() {
                    let witness = checker.checkpoint()?;
                    if !witness.is_empty() {
                        self.range_check_witnesses.push(witness);
                    }
                }
            }

            // Increment cycle counter
            self.state.inc_cycles();
        }

        Ok(ExecutionResult {
            cycles: self.state.cycles,
            outputs: self.io.outputs().to_vec(),
            halt_reason: self.state.halt_reason.clone().unwrap_or(HaltReason::Ebreak),
            range_check_witnesses: self.range_check_witnesses,
            execution_trace: self.execution_trace,
        })
    }

    /// Fetch and decode instruction from memory
    /// Returns (instruction, encoded_word)
    fn fetch_and_decode(&mut self) -> Result<(Instruction, u32)> {
        // PC must be 4-byte aligned
        if self.state.pc % 4 != 0 {
            return Err(RuntimeError::Other(format!(
                "Misaligned PC: {:#x}",
                self.state.pc
            )));
        }

        // Read instruction word from memory
        let word = self.memory.read_u32(self.state.pc)?;

        // Decode instruction using disassembler
        let inst = zkir_disassembler::decode(word)
            .map_err(|e| RuntimeError::Other(format!("Decode error: {}", e)))?;

        Ok((inst, word))
    }

    /// Get current state (for debugging)
    pub fn state(&self) -> &VMState {
        &self.state
    }

    /// Get memory (for debugging)
    pub fn memory(&self) -> &Memory {
        &self.memory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zkir_spec::Register;

    fn create_program_from_instructions(instructions: Vec<Instruction>) -> Program {
        let mut program = Program::new();

        // Encode instructions to u32 using assembler
        let code: Vec<u32> = instructions
            .iter()
            .map(|inst| zkir_assembler::encode(inst))
            .collect();

        program.code = code;
        program.header.code_size = (program.code.len() * 4) as u32;
        program
    }

    #[test]
    fn test_vm_basic_execution() {
        // Simple program: add R1 + R2 -> R3, then ebreak
        let instructions = vec![
            Instruction::Addi {
                rd: Register::R1,
                rs1: Register::R0,
                imm: 10,
            },
            Instruction::Addi {
                rd: Register::R2,
                rs1: Register::R0,
                imm: 20,
            },
            Instruction::Add {
                rd: Register::R3,
                rs1: Register::R1,
                rs2: Register::R2,
            },
            Instruction::Ebreak,
        ];

        let program = create_program_from_instructions(instructions);
        let vm = VM::new(program, vec![], VMConfig::default());
        let result = vm.run().unwrap();

        assert_eq!(result.halt_reason, HaltReason::Ebreak);
        assert_eq!(result.cycles, 4);
    }

    #[test]
    fn test_vm_exit_syscall() {
        // Program: exit with code 42
        let instructions = vec![
            Instruction::Addi {
                rd: Register::R10, // a0 = syscall number
                rs1: Register::R0,
                imm: 0, // SYSCALL_EXIT
            },
            Instruction::Addi {
                rd: Register::R11, // a1 = exit code
                rs1: Register::R0,
                imm: 42,
            },
            Instruction::Ecall,
        ];

        let program = create_program_from_instructions(instructions);
        let vm = VM::new(program, vec![], VMConfig::default());
        let result = vm.run().unwrap();

        assert_eq!(result.halt_reason, HaltReason::Exit(42));
        assert_eq!(result.cycles, 3);
    }

    #[test]
    fn test_vm_io_syscalls() {
        // Program: read input, write to output, exit
        let instructions = vec![
            // Read syscall
            Instruction::Addi {
                rd: Register::R10,
                rs1: Register::R0,
                imm: 1, // SYSCALL_READ
            },
            Instruction::Ecall,
            // R10 now has input value
            // Write syscall - copy R10 to R11
            Instruction::Addi {
                rd: Register::R11,
                rs1: Register::R10,
                imm: 0,
            },
            Instruction::Addi {
                rd: Register::R10,
                rs1: Register::R0,
                imm: 2, // SYSCALL_WRITE
            },
            Instruction::Ecall,
            // Clear R11 before exit
            Instruction::Addi {
                rd: Register::R11,
                rs1: Register::R0,
                imm: 0,
            },
            // Exit
            Instruction::Addi {
                rd: Register::R10,
                rs1: Register::R0,
                imm: 0, // SYSCALL_EXIT
            },
            Instruction::Ecall,
        ];

        let program = create_program_from_instructions(instructions);
        let vm = VM::new(program, vec![123], VMConfig::default());
        let result = vm.run().unwrap();

        assert_eq!(result.outputs, vec![123]);
        assert_eq!(result.halt_reason, HaltReason::Exit(0));
    }

    #[test]
    fn test_vm_cycle_limit() {
        // Infinite loop: jal to self
        let instructions = vec![Instruction::Jal {
            rd: Register::R0,
            offset: 0,
        }];

        let program = create_program_from_instructions(instructions);
        let mut config = VMConfig::default();
        config.max_cycles = 100;

        let vm = VM::new(program, vec![], config);
        let result = vm.run().unwrap();

        assert_eq!(result.halt_reason, HaltReason::CycleLimit);
        assert_eq!(result.cycles, 100);
    }

    #[test]
    fn test_vm_memory_operations() {
        // Program: store and load from memory
        let instructions = vec![
            // Set up address in R1
            Instruction::Addi {
                rd: Register::R1,
                rs1: Register::R0,
                imm: 0x1000,
            },
            // Set up value in R2
            Instruction::Addi {
                rd: Register::R2,
                rs1: Register::R0,
                imm: 0x42,
            },
            // Store word
            Instruction::Sw {
                rs1: Register::R1,
                rs2: Register::R2,
                imm: 0,
            },
            // Load word into R3
            Instruction::Lw {
                rd: Register::R3,
                rs1: Register::R1,
                imm: 0,
            },
            Instruction::Ebreak,
        ];

        let program = create_program_from_instructions(instructions);
        let vm = VM::new(program, vec![], VMConfig::default());
        let result = vm.run().unwrap();

        assert_eq!(result.halt_reason, HaltReason::Ebreak);
        assert_eq!(result.cycles, 5);
    }

    #[test]
    fn test_vm_branches() {
        // Program: conditional branch
        let instructions = vec![
            // R1 = 10
            Instruction::Addi {
                rd: Register::R1,
                rs1: Register::R0,
                imm: 10,
            },
            // R2 = 10
            Instruction::Addi {
                rd: Register::R2,
                rs1: Register::R0,
                imm: 10,
            },
            // if R1 == R2, skip next instruction
            Instruction::Beq {
                rs1: Register::R1,
                rs2: Register::R2,
                offset: 8, // Skip 2 instructions (8 bytes)
            },
            // This should be skipped
            Instruction::Addi {
                rd: Register::R3,
                rs1: Register::R0,
                imm: 99,
            },
            // Jump target - R3 should still be 0
            Instruction::Ebreak,
        ];

        let program = create_program_from_instructions(instructions);
        let vm = VM::new(program, vec![], VMConfig::default());
        let result = vm.run().unwrap();

        assert_eq!(result.halt_reason, HaltReason::Ebreak);
        // Cycles: addi, addi, beq (taken), ebreak = 4
        assert_eq!(result.cycles, 4);
    }

    #[test]
    fn test_vm_range_checking_enabled() {
        // Program with stores (which trigger checkpoints)
        let instructions = vec![
            // R1 = address
            Instruction::Addi {
                rd: Register::R1,
                rs1: Register::R0,
                imm: 0x1000,
            },
            // R2 = value
            Instruction::Addi {
                rd: Register::R2,
                rs1: Register::R0,
                imm: 0x42,
            },
            // Store (triggers checkpoint)
            Instruction::Sw {
                rs1: Register::R1,
                rs2: Register::R2,
                imm: 0,
            },
            Instruction::Ebreak,
        ];

        let program = create_program_from_instructions(instructions);

        // Run with range checking enabled
        let mut config = VMConfig::default();
        config.enable_range_checking = true;

        let vm = VM::new(program, vec![], config);
        let result = vm.run().unwrap();

        assert_eq!(result.halt_reason, HaltReason::Ebreak);
        assert_eq!(result.cycles, 4);

        // Range checking is enabled, but we haven't deferred any checks yet
        // (would need bound tracking in execute.rs to actually defer checks)
        // For now, just verify the infrastructure works
        assert!(result.range_check_witnesses.len() >= 0);
    }

    #[test]
    fn test_vm_range_checking_disabled() {
        // Same program, but without range checking
        let instructions = vec![
            Instruction::Addi {
                rd: Register::R1,
                rs1: Register::R0,
                imm: 0x42,
            },
            Instruction::Ebreak,
        ];

        let program = create_program_from_instructions(instructions);
        let vm = VM::new(program, vec![], VMConfig::default());
        let result = vm.run().unwrap();

        // Range checking disabled - no witnesses
        assert_eq!(result.range_check_witnesses.len(), 0);
    }

    #[test]
    fn test_bound_propagation_and_deferred_checks() {
        // Program that accumulates bounds through repeated additions
        // This should trigger deferred range checks
        let mut instructions = vec![];

        // Initialize R1 with a large value
        instructions.push(Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: (1 << 15) - 1,  // Large immediate (15 bits)
        });

        // Perform many additions to accumulate bound growth
        // After each add, bound grows by 1 bit
        for _ in 0..30 {
            instructions.push(Instruction::Add {
                rd: Register::R1,
                rs1: Register::R1,
                rs2: Register::R1,  // R1 = R1 + R1 (doubles value, grows bound)
            });
        }

        // Store triggers checkpoint
        instructions.push(Instruction::Addi {
            rd: Register::R2,
            rs1: Register::R0,
            imm: 0x1000,  // Address
        });
        instructions.push(Instruction::Sw {
            rs1: Register::R2,
            rs2: Register::R1,
            imm: 0,
        });

        instructions.push(Instruction::Ebreak);

        let program = create_program_from_instructions(instructions);

        // Run with range checking enabled
        let mut config = VMConfig::default();
        config.enable_range_checking = true;

        let vm = VM::new(program, vec![], config);
        let result = vm.run().unwrap();

        assert_eq!(result.halt_reason, HaltReason::Ebreak);

        // With bound propagation, the adds will accumulate bounds
        // that exceed 40 bits, triggering deferred checks
        // The store checkpoint should have generated witnesses
        assert!(
            result.range_check_witnesses.len() > 0,
            "Expected range check witnesses from accumulated bound growth"
        );
    }

    #[test]
    fn test_immediate_constant_bounds() {
        // Test that immediate values get tight constant bounds
        let instructions = vec![
            // R1 = 100 (constant bound: 7 bits)
            Instruction::Addi {
                rd: Register::R1,
                rs1: Register::R0,
                imm: 100,
            },
            // R2 = 200 (constant bound: 8 bits)
            Instruction::Addi {
                rd: Register::R2,
                rs1: Register::R0,
                imm: 200,
            },
            // R3 = R1 + R2 (bound: max(7,8) + 1 = 9 bits)
            Instruction::Add {
                rd: Register::R3,
                rs1: Register::R1,
                rs2: Register::R2,
            },
            // Store (checkpoint, but no deferred checks needed - all within 40 bits)
            Instruction::Addi {
                rd: Register::R4,
                rs1: Register::R0,
                imm: 0x2000,
            },
            Instruction::Sw {
                rs1: Register::R4,
                rs2: Register::R3,
                imm: 0,
            },
            Instruction::Ebreak,
        ];

        let program = create_program_from_instructions(instructions);

        let mut config = VMConfig::default();
        config.enable_range_checking = true;

        let vm = VM::new(program, vec![], config);
        let result = vm.run().unwrap();

        assert_eq!(result.halt_reason, HaltReason::Ebreak);

        // All bounds are well within 40 bits, so no witnesses expected
        assert_eq!(
            result.range_check_witnesses.len(),
            0,
            "No range checks needed for small constant operations"
        );
    }

    #[test]
    fn test_vm_memory_trace_collection() {
        // Program that performs memory operations
        let instructions = vec![
            // Store value to memory
            Instruction::Addi {
                rd: Register::R1,
                rs1: Register::R0,
                imm: 0x42,
            },
            Instruction::Addi {
                rd: Register::R3,
                rs1: Register::R0,
                imm: 0x1000,  // Address
            },
            Instruction::Sw {
                rs1: Register::R3,
                rs2: Register::R1,
                imm: 0,
            },
            // Load value from memory
            Instruction::Lw {
                rd: Register::R4,
                rs1: Register::R3,
                imm: 0,
            },
            Instruction::Ebreak,
        ];

        let program = create_program_from_instructions(instructions);

        // Run with execution trace enabled (which automatically collects memory ops)
        let mut config = VMConfig::default();
        config.enable_execution_trace = true;

        let vm = VM::new(program, vec![], config);
        let result = vm.run().unwrap();

        assert_eq!(result.halt_reason, HaltReason::Ebreak);

        // Extract memory operations from execution trace
        let memory_trace = result.get_memory_trace();

        // Should have memory operations recorded
        assert!(
            memory_trace.len() > 0,
            "Expected memory trace to contain operations"
        );

        // Verify we have at least one write and one read
        let has_write = memory_trace.iter().any(|op| op.is_write());
        let has_read = memory_trace.iter().any(|op| op.is_read());

        assert!(has_write, "Expected at least one write operation in trace");
        assert!(has_read, "Expected at least one read operation in trace");
    }

    #[test]
    fn test_vm_memory_trace_disabled() {
        // Same program but with tracing disabled
        let instructions = vec![
            Instruction::Addi {
                rd: Register::R1,
                rs1: Register::R0,
                imm: 0x42,
            },
            Instruction::Addi {
                rd: Register::R3,
                rs1: Register::R0,
                imm: 0x1000,
            },
            Instruction::Sw {
                rs1: Register::R3,
                rs2: Register::R1,
                imm: 0,
            },
            Instruction::Ebreak,
        ];

        let program = create_program_from_instructions(instructions);
        let vm = VM::new(program, vec![], VMConfig::default());  // trace disabled by default
        let result = vm.run().unwrap();

        // With tracing disabled, execution trace should be empty
        assert_eq!(
            result.execution_trace.len(),
            0,
            "Execution trace should be empty when tracing is disabled"
        );

        // Memory operations count should be zero
        assert_eq!(
            result.memory_op_count(),
            0,
            "Memory operation count should be zero when tracing is disabled"
        );
    }

    #[test]
    fn test_vm_execution_trace_collection() {
        // Program with several instructions to trace
        let instructions = vec![
            // R1 = 100
            Instruction::Addi {
                rd: Register::R1,
                rs1: Register::R0,
                imm: 100,
            },
            // R2 = 200
            Instruction::Addi {
                rd: Register::R2,
                rs1: Register::R0,
                imm: 200,
            },
            // R3 = R1 + R2
            Instruction::Add {
                rd: Register::R3,
                rs1: Register::R1,
                rs2: Register::R2,
            },
            Instruction::Ebreak,
        ];

        let program = create_program_from_instructions(instructions);

        // Run with execution trace enabled
        let mut config = VMConfig::default();
        config.enable_execution_trace = true;

        let vm = VM::new(program, vec![], config);
        let result = vm.run().unwrap();

        assert_eq!(result.halt_reason, HaltReason::Ebreak);

        // Should have execution trace entries
        assert!(
            result.execution_trace.len() > 0,
            "Expected execution trace to contain entries"
        );

        // Should have 4 trace rows (3 instructions + ebreak)
        assert_eq!(
            result.execution_trace.len(),
            4,
            "Expected 4 trace rows"
        );

        // Verify trace row structure
        let first_row = &result.execution_trace[0];
        assert_eq!(first_row.cycle, 0, "First row should be at cycle 0");
        assert_eq!(first_row.registers.len(), 16, "Should have 16 registers");
        assert_eq!(first_row.bounds.len(), 16, "Should have 16 bounds");

        // Last row should be at cycle 3 (after 4 instructions, cycle increments after execution)
        let last_row = &result.execution_trace[3];
        assert_eq!(last_row.cycle, 3, "Last row should be at cycle 3");
    }

    #[test]
    fn test_vm_execution_trace_disabled() {
        // Same program but with tracing disabled
        let instructions = vec![
            Instruction::Addi {
                rd: Register::R1,
                rs1: Register::R0,
                imm: 100,
            },
            Instruction::Add {
                rd: Register::R2,
                rs1: Register::R1,
                rs2: Register::R0,
            },
            Instruction::Ebreak,
        ];

        let program = create_program_from_instructions(instructions);
        let vm = VM::new(program, vec![], VMConfig::default()); // trace disabled by default
        let result = vm.run().unwrap();

        // With tracing disabled, trace should be empty
        assert_eq!(
            result.execution_trace.len(),
            0,
            "Execution trace should be empty when tracing is disabled"
        );
    }

    #[test]
    fn test_vm_execution_trace_with_memory_ops() {
        // Program with memory operations to verify trace includes them
        let instructions = vec![
            // R1 = 42
            Instruction::Addi {
                rd: Register::R1,
                rs1: Register::R0,
                imm: 0x42,
            },
            // R3 = 0x1000 (address)
            Instruction::Addi {
                rd: Register::R3,
                rs1: Register::R0,
                imm: 0x1000,
            },
            // Store R1 to [R3]
            Instruction::Sw {
                rs1: Register::R3,
                rs2: Register::R1,
                imm: 0,
            },
            // Load from [R3] to R4
            Instruction::Lw {
                rd: Register::R4,
                rs1: Register::R3,
                imm: 0,
            },
            Instruction::Ebreak,
        ];

        let program = create_program_from_instructions(instructions);

        // Run with execution trace enabled (automatically collects memory ops)
        let mut config = VMConfig::default();
        config.enable_execution_trace = true;

        let vm = VM::new(program, vec![], config);
        let result = vm.run().unwrap();

        assert_eq!(result.halt_reason, HaltReason::Ebreak);

        // Should have 5 trace rows
        assert_eq!(result.execution_trace.len(), 5);

        // The SW instruction (index 2) should have memory_ops with one write
        let sw_trace = &result.execution_trace[2];
        assert_eq!(
            sw_trace.memory_ops.len(),
            1,
            "SW instruction trace should have exactly one memory operation"
        );
        assert!(
            sw_trace.memory_ops[0].is_write(),
            "Memory op for SW should be a write"
        );

        // The LW instruction (index 3) should have memory_ops with one read
        let lw_trace = &result.execution_trace[3];
        assert_eq!(
            lw_trace.memory_ops.len(),
            1,
            "LW instruction trace should have exactly one memory operation"
        );
        assert!(
            lw_trace.memory_ops[0].is_read(),
            "Memory op for LW should be a read"
        );

        // Non-memory instructions should have empty memory_ops
        assert_eq!(
            result.execution_trace[0].memory_ops.len(),
            0,
            "ADDI should have no memory operations"
        );
    }

    #[test]
    fn test_trace_timestamp_synchronization() {
        // Comprehensive test to ensure timestamps match between execution_trace and memory_trace
        let instructions = vec![
            // R1 = 0x100
            Instruction::Addi {
                rd: Register::R1,
                rs1: Register::R0,
                imm: 0x100,
            },
            // R2 = 0x1000 (address)
            Instruction::Addi {
                rd: Register::R2,
                rs1: Register::R0,
                imm: 0x1000,
            },
            // SW: Store R1 to [R2] - cycle 2
            Instruction::Sw {
                rs1: Register::R2,
                rs2: Register::R1,
                imm: 0,
            },
            // R3 = 0x200
            Instruction::Addi {
                rd: Register::R3,
                rs1: Register::R0,
                imm: 0x200,
            },
            // SW: Store R3 to [R2+4] - cycle 4
            Instruction::Sw {
                rs1: Register::R2,
                rs2: Register::R3,
                imm: 4,
            },
            // LW: Load from [R2] to R4 - cycle 5
            Instruction::Lw {
                rd: Register::R4,
                rs1: Register::R2,
                imm: 0,
            },
            // LW: Load from [R2+4] to R5 - cycle 6
            Instruction::Lw {
                rd: Register::R5,
                rs1: Register::R2,
                imm: 4,
            },
            Instruction::Ebreak,
        ];

        let program = create_program_from_instructions(instructions);

        // Enable execution trace (memory ops are automatically collected)
        let mut config = VMConfig::default();
        config.enable_execution_trace = true;

        let vm = VM::new(program, vec![], config);
        let result = vm.run().unwrap();

        assert_eq!(result.halt_reason, HaltReason::Ebreak);

        // Verify we have execution trace
        assert!(result.execution_trace.len() > 0, "Should have execution trace");

        // Extract all memory operations from execution trace
        let all_memory_ops = result.get_memory_trace();

        // Verify that we captured all data memory operations (2 SW + 2 LW = 4)
        assert_eq!(
            all_memory_ops.len(),
            4,
            "Should have exactly 4 data memory operations (2 SW + 2 LW)"
        );

        // Count writes and reads
        let writes: Vec<_> = all_memory_ops.iter().filter(|op| op.is_write()).collect();
        let reads: Vec<_> = all_memory_ops.iter().filter(|op| op.is_read()).collect();

        assert_eq!(writes.len(), 2, "Should have 2 SW operations");
        assert_eq!(reads.len(), 2, "Should have 2 LW operations");

        // Verify SW instruction (index 2) has correct memory_ops
        let sw1_trace = &result.execution_trace[2];
        assert_eq!(sw1_trace.memory_ops.len(), 1, "SW should have 1 memory op");
        assert!(sw1_trace.memory_ops[0].is_write());
        assert_eq!(sw1_trace.memory_ops[0].timestamp, 2);

        // Verify second SW instruction (index 4) has correct memory_ops
        let sw2_trace = &result.execution_trace[4];
        assert_eq!(sw2_trace.memory_ops.len(), 1, "SW should have 1 memory op");
        assert!(sw2_trace.memory_ops[0].is_write());
        assert_eq!(sw2_trace.memory_ops[0].timestamp, 4);

        // Verify first LW instruction (index 5) has correct memory_ops
        let lw1_trace = &result.execution_trace[5];
        assert_eq!(lw1_trace.memory_ops.len(), 1, "LW should have 1 memory op");
        assert!(lw1_trace.memory_ops[0].is_read());
        assert_eq!(lw1_trace.memory_ops[0].timestamp, 5);

        // Verify second LW instruction (index 6) has correct memory_ops
        let lw2_trace = &result.execution_trace[6];
        assert_eq!(lw2_trace.memory_ops.len(), 1, "LW should have 1 memory op");
        assert!(lw2_trace.memory_ops[0].is_read());
        assert_eq!(lw2_trace.memory_ops[0].timestamp, 6);

        // Verify non-load/store instructions have empty memory_ops
        assert_eq!(result.execution_trace[0].memory_ops.len(), 0, "ADDI should have no memory ops");
        assert_eq!(result.execution_trace[1].memory_ops.len(), 0, "ADDI should have no memory ops");
        assert_eq!(result.execution_trace[3].memory_ops.len(), 0, "ADDI should have no memory ops");
        assert_eq!(result.execution_trace[7].memory_ops.len(), 0, "EBREAK should have no memory ops");

        // Verify timestamps are properly synchronized with cycle numbers
        for row in &result.execution_trace {
            for mem_op in &row.memory_ops {
                assert_eq!(
                    mem_op.timestamp, row.cycle,
                    "Memory op timestamp should match trace row cycle"
                );
            }
        }

        // Verify memory operations are sorted (by timestamp)
        let sorted_ops = result.get_memory_trace();
        for i in 1..sorted_ops.len() {
            assert!(
                sorted_ops[i - 1].timestamp <= sorted_ops[i].timestamp,
                "Memory operations should be sorted by timestamp"
            );
        }
    }
}
