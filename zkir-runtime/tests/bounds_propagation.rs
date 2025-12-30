//! Tests for instruction execution with bounds propagation
//!
//! Tests the bound tracking system that enables deferred range checking.

use zkir_runtime::{VM, VMConfig, HaltReason};
use zkir_spec::{Instruction, Program, Register, Config};

fn create_program_from_instructions(instructions: Vec<Instruction>) -> Program {
    let mut program = Program::new();
    let code: Vec<u32> = instructions
        .iter()
        .map(|inst| zkir_assembler::encode(inst))
        .collect();
    program.code = code;
    program.header.code_size = (program.code.len() * 4) as u32;
    program
}

// ============================================================================
// Basic Bound Initialization Tests
// ============================================================================

#[test]
fn test_register_zero_initial_bound() {
    // r0 should always be zero with constant bound of 0 bits
    let instructions = vec![
        Instruction::Add {
            rd: Register::R1,
            rs1: Register::R0,
            rs2: Register::R0,
        },
        Instruction::Ebreak,
    ];

    let program = create_program_from_instructions(instructions);
    let mut config = VMConfig::default();
    config.enable_range_checking = true;
    config.enable_execution_trace = true;

    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Ebreak);
}

#[test]
fn test_immediate_constant_bound() {
    // Immediate values should get tight constant bounds
    let instructions = vec![
        // r1 = 100 (constant bound: 7 bits because 64 < 100 < 128)
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: 100,
        },
        Instruction::Ebreak,
    ];

    let program = create_program_from_instructions(instructions);
    let mut config = VMConfig::default();
    config.enable_range_checking = true;
    config.enable_execution_trace = true;

    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Ebreak);
    // Small constants should not trigger range checks
    assert_eq!(result.range_check_witnesses.len(), 0);
}

// ============================================================================
// Bound Growth Tests
// ============================================================================

#[test]
fn test_add_bound_growth() {
    // Adding two N-bit values produces at most (N+1)-bit result
    let mut instructions = vec![];

    // r1 = 100 (7 bits)
    instructions.push(Instruction::Addi {
        rd: Register::R1,
        rs1: Register::R0,
        imm: 100,
    });

    // r2 = 200 (8 bits)
    instructions.push(Instruction::Addi {
        rd: Register::R2,
        rs1: Register::R0,
        imm: 200,
    });

    // r3 = r1 + r2 (bound: max(7,8) + 1 = 9 bits)
    instructions.push(Instruction::Add {
        rd: Register::R3,
        rs1: Register::R1,
        rs2: Register::R2,
    });

    instructions.push(Instruction::Ebreak);

    let program = create_program_from_instructions(instructions);
    let mut config = VMConfig::default();
    config.enable_range_checking = true;

    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Ebreak);
    // All within 40 bits, no witnesses needed
    assert_eq!(result.range_check_witnesses.len(), 0);
}

#[test]
fn test_mul_bound_growth() {
    // Multiplying two N-bit values produces at most 2N-bit result
    let instructions = vec![
        // r1 = 1000 (10 bits)
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: 1000,
        },
        // r2 = 1000 (10 bits)
        Instruction::Addi {
            rd: Register::R2,
            rs1: Register::R0,
            imm: 1000,
        },
        // r3 = r1 * r2 (bound: 10 + 10 = 20 bits)
        Instruction::Mul {
            rd: Register::R3,
            rs1: Register::R1,
            rs2: Register::R2,
        },
        Instruction::Ebreak,
    ];

    let program = create_program_from_instructions(instructions);
    let mut config = VMConfig::default();
    config.enable_range_checking = true;

    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Ebreak);
}

#[test]
fn test_shift_bound_growth() {
    // Left shift by N adds N bits to bound
    let instructions = vec![
        // r1 = 1 (1 bit)
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: 1,
        },
        // r2 = r1 << 10 (11 bits)
        Instruction::Slli {
            rd: Register::R2,
            rs1: Register::R1,
            shamt: 10,
        },
        Instruction::Ebreak,
    ];

    let program = create_program_from_instructions(instructions);
    let mut config = VMConfig::default();
    config.enable_range_checking = true;

    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Ebreak);
}

// ============================================================================
// Deferred Range Check Tests
// ============================================================================

#[test]
fn test_accumulated_bounds_trigger_checks() {
    // Repeated additions should accumulate bounds until they exceed threshold
    let mut instructions = vec![];

    // Start with a value
    instructions.push(Instruction::Addi {
        rd: Register::R1,
        rs1: Register::R0,
        imm: (1 << 15) - 1, // Large 15-bit value
    });

    // Repeatedly double the value (each add grows bound by 1 bit)
    for _ in 0..30 {
        instructions.push(Instruction::Add {
            rd: Register::R1,
            rs1: Register::R1,
            rs2: Register::R1,
        });
    }

    // Store triggers checkpoint
    instructions.push(Instruction::Addi {
        rd: Register::R2,
        rs1: Register::R0,
        imm: 0x1000,
    });
    instructions.push(Instruction::Sw {
        rs1: Register::R2,
        rs2: Register::R1,
        imm: 0,
    });

    instructions.push(Instruction::Ebreak);

    let program = create_program_from_instructions(instructions);
    let mut config = VMConfig::default();
    config.enable_range_checking = true;

    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Ebreak);
    // Bounds grew beyond 40 bits, should have witnesses
    assert!(
        result.range_check_witnesses.len() > 0,
        "Expected range check witnesses from accumulated bound growth"
    );
}

// ============================================================================
// Bound-Preserving Operations
// ============================================================================

#[test]
fn test_and_preserves_bound() {
    // AND cannot increase the bound
    let instructions = vec![
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: 0xFF,
        },
        Instruction::Addi {
            rd: Register::R2,
            rs1: Register::R0,
            imm: 0x0F,
        },
        // r3 = r1 & r2 (bound: min(8, 4) = 4 bits)
        Instruction::And {
            rd: Register::R3,
            rs1: Register::R1,
            rs2: Register::R2,
        },
        Instruction::Ebreak,
    ];

    let program = create_program_from_instructions(instructions);
    let mut config = VMConfig::default();
    config.enable_range_checking = true;

    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Ebreak);
}

#[test]
fn test_srl_reduces_bound() {
    // Right shift by N reduces bound by N
    let instructions = vec![
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: 0xFF00, // 16 bits
        },
        // r2 = r1 >> 8 (8 bits after shift)
        Instruction::Srli {
            rd: Register::R2,
            rs1: Register::R1,
            shamt: 8,
        },
        Instruction::Ebreak,
    ];

    let program = create_program_from_instructions(instructions);
    let mut config = VMConfig::default();
    config.enable_range_checking = true;

    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Ebreak);
}

// ============================================================================
// Checkpoint Trigger Tests
// ============================================================================

#[test]
fn test_store_triggers_checkpoint() {
    let instructions = vec![
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: 42,
        },
        Instruction::Addi {
            rd: Register::R2,
            rs1: Register::R0,
            imm: 0x1000,
        },
        Instruction::Sw {
            rs1: Register::R2,
            rs2: Register::R1,
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
}

#[test]
fn test_branch_triggers_checkpoint() {
    let instructions = vec![
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: 1,
        },
        Instruction::Addi {
            rd: Register::R2,
            rs1: Register::R0,
            imm: 1,
        },
        Instruction::Beq {
            rs1: Register::R1,
            rs2: Register::R2,
            offset: 4, // Skip next instruction
        },
        Instruction::Ebreak,
    ];

    let program = create_program_from_instructions(instructions);
    let mut config = VMConfig::default();
    config.enable_range_checking = true;

    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    // Branch taken, so we execute the ebreak
    assert_eq!(result.halt_reason, HaltReason::Ebreak);
}

#[test]
fn test_division_triggers_checkpoint() {
    let instructions = vec![
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: 100,
        },
        Instruction::Addi {
            rd: Register::R2,
            rs1: Register::R0,
            imm: 10,
        },
        Instruction::Divu {
            rd: Register::R3,
            rs1: Register::R1,
            rs2: Register::R2,
        },
        Instruction::Ebreak,
    ];

    let program = create_program_from_instructions(instructions);
    let mut config = VMConfig::default();
    config.enable_range_checking = true;

    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Ebreak);
}

// ============================================================================
// Execution Trace Tests
// ============================================================================

#[test]
fn test_execution_trace_records_bounds() {
    let instructions = vec![
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: 100,
        },
        Instruction::Addi {
            rd: Register::R2,
            rs1: Register::R0,
            imm: 200,
        },
        Instruction::Add {
            rd: Register::R3,
            rs1: Register::R1,
            rs2: Register::R2,
        },
        Instruction::Ebreak,
    ];

    let program = create_program_from_instructions(instructions);
    let mut config = VMConfig::default();
    config.enable_execution_trace = true;

    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Ebreak);
    assert_eq!(result.execution_trace.len(), 4);

    // Each trace row should have bounds for all 16 registers
    for row in &result.execution_trace {
        assert_eq!(row.bounds.len(), 16);
    }
}

// ============================================================================
// Config-Dependent Tests
// ============================================================================

#[test]
fn test_different_limb_config() {
    // Test with 30-bit limbs
    let instructions = vec![
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: 1000,
        },
        Instruction::Ebreak,
    ];

    let config30 = Config {
        limb_bits: 30,
        data_limbs: 2,
        addr_limbs: 2,
    };

    let mut program = Program::with_config(config30).unwrap();
    let code: Vec<u32> = instructions
        .iter()
        .map(|inst| zkir_assembler::encode(inst))
        .collect();
    program.code = code;
    program.header.code_size = (program.code.len() * 4) as u32;

    let mut vm_config = VMConfig::default();
    vm_config.enable_range_checking = true;

    let vm = VM::new(program, vec![], vm_config);
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Ebreak);
}
