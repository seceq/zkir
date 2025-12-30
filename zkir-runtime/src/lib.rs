//! # ZKIR Runtime v3.4
//!
//! Execute ZKIR v3.4 programs with variable limb architecture.
//!
//! This runtime provides a virtual machine for executing ZKIR programs using
//! configurable limb sizes (default: 20-bit × 2 limbs = 40-bit values).
//!
//! ## Features
//!
//! - **Variable limb sizes**: Configurable 16-30 bit limbs
//! - **47 instructions**: Complete v3.4 instruction set
//! - **16 registers**: R0-R15
//! - **Memory operations**: Byte, halfword, word, and doubleword loads/stores
//! - **Syscalls**: Exit, read, write
//!
//! ## Example
//!
//! ```rust,no_run
//! use zkir_runtime::{VM, VMConfig};
//! use zkir_spec::Program;
//!
//! let program = Program::new();
//! let inputs = vec![];
//! let vm = VM::new(program, inputs, VMConfig::default());
//! let result = vm.run().unwrap();
//! println!("Cycles: {}", result.cycles);
//! ```

pub mod error;
pub mod state;
pub mod memory;
pub mod execute;
pub mod syscall;
pub mod vm;
pub mod range_check;
pub mod crypto;

pub use state::{VMState, HaltReason};
pub use memory::{Memory, MemoryRegion};
pub use syscall::{IOHandler, handle_syscall};
pub use vm::{VM, VMConfig, ExecutionResult};
pub use error::RuntimeError;
pub use range_check::{RangeCheckTracker, RangeCheckWitness, RangeLookupTable};

/// Simple execution helper
///
/// Runs a program with the given inputs and returns the outputs.
pub fn run(program: zkir_spec::Program, inputs: Vec<u64>) -> Result<Vec<u64>, RuntimeError> {
    let vm = VM::new(program, inputs, VMConfig::default());
    Ok(vm.run()?.outputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zkir_spec::{Instruction, Register, Program};

    fn create_test_program(instructions: Vec<Instruction>) -> Program {
        let mut program = Program::new();
        let code: Vec<u32> = instructions
            .iter()
            .map(|inst| zkir_assembler::encode(inst))
            .collect();
        program.code = code;
        program.header.code_size = (program.code.len() * 4) as u32;
        program
    }

    #[test]
    fn test_public_exports() {
        // Verify all public types are accessible
        let _ = VMConfig::default();
        let _ = HaltReason::Ebreak;
        let _ = MemoryRegion::Code;
    }

    #[test]
    fn test_vm_new() {
        let program = Program::new();
        let _vm = VM::new(program, vec![], VMConfig::default());
    }

    #[test]
    fn test_vmconfig_default() {
        let config = VMConfig::default();
        assert_eq!(config.max_cycles, 1_000_000);
        assert!(!config.trace);
        assert!(!config.enable_range_checking);
        assert!(!config.enable_execution_trace);
    }

    #[test]
    fn test_memory_new() {
        let mem = Memory::new();
        assert_eq!(mem.get_region(0x0), MemoryRegion::Reserved);
    }

    #[test]
    fn test_halt_reason_variants() {
        let reasons = vec![
            HaltReason::Exit(0),
            HaltReason::Ebreak,
            HaltReason::CycleLimit,
        ];

        for reason in reasons {
            let _ = format!("{:?}", reason);
        }
    }

    #[test]
    fn test_run_helper() {
        let instructions = vec![
            Instruction::Addi {
                rd: Register::R10,
                rs1: Register::R0,
                imm: 0,  // SYSCALL_EXIT
            },
            Instruction::Addi {
                rd: Register::R11,
                rs1: Register::R0,
                imm: 0,  // exit code
            },
            Instruction::Ecall,
        ];

        let program = create_test_program(instructions);
        let result = run(program, vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_runtime_error_reexport() {
        // Verify RuntimeError is accessible
        let err = RuntimeError::Other("test".to_string());
        assert_eq!(err.to_string(), "test");
    }

    #[test]
    fn test_range_check_tracker_new() {
        let config = zkir_spec::Config::DEFAULT;
        let _tracker = RangeCheckTracker::new(config);
    }

    #[test]
    fn test_execution_result_methods() {
        let result = ExecutionResult {
            cycles: 100,
            outputs: vec![1, 2, 3],
            halt_reason: HaltReason::Ebreak,
            range_check_witnesses: vec![],
            execution_trace: vec![],
        };

        assert_eq!(result.cycles, 100);
        assert_eq!(result.outputs, vec![1, 2, 3]);
        assert_eq!(result.memory_op_count(), 0);
        assert!(result.get_memory_trace().is_empty());
    }

    #[test]
    fn test_io_handler_new() {
        let handler = IOHandler::new(vec![1, 2, 3]);
        assert_eq!(handler.outputs().len(), 0);
    }

    #[test]
    fn test_memory_region_properties() {
        assert!(!MemoryRegion::Reserved.is_writable());
        assert!(!MemoryRegion::Code.is_writable());
        assert!(MemoryRegion::Data.is_writable());
        assert!(MemoryRegion::Heap.is_writable());
        assert!(MemoryRegion::Stack.is_writable());

        assert!(MemoryRegion::Reserved.is_readable());
        assert!(MemoryRegion::Code.is_readable());
        assert!(MemoryRegion::Data.is_readable());
    }
}
