//! # ZKIR Disassembler v3.4
//!
//! Disassemble ZKIR v3.4 bytecode into human-readable assembly.
//!
//! ## Format Modes
//!
//! ZKIR bytecode files can be in two formats:
//!
//! - **Release mode**: `[header][code][data]` - standard format for execution
//! - **Debug mode**: `[header][globals][functions][code]` - includes symbol tables
//!
//! This disassembler works with `Program` objects (release format only).
//! For debug format files produced by `zkir-llvm --debug`, use the built-in
//! `zkir-llvm --emit disasm` command which understands the function table format.
//!
//! Use [`zkir_spec::FormatMode::detect()`] to check if a bytecode file is
//! in release or debug format before attempting to load it.
//!
//! ## Example
//!
//! ```rust
//! use zkir_spec::{Program, FormatMode};
//! use zkir_disassembler::disassemble;
//!
//! // Check format before loading
//! let bytes: &[u8] = &[/* ... bytecode ... */];
//! # let mut program = Program::new();
//! # program.code = vec![0x50, 0x51]; // ecall, ebreak
//! # program.header.code_size = 8;
//! # let bytes = program.to_bytes();
//!
//! match FormatMode::detect(&bytes) {
//!     Some(FormatMode::Release) => {
//!         let program = Program::from_bytes(&bytes).unwrap();
//!         let asm = disassemble(&program).unwrap();
//!         println!("{}", asm);
//!     }
//!     Some(FormatMode::Debug) => {
//!         eprintln!("Debug format - use 'zkir-llvm --emit disasm' instead");
//!     }
//!     None => {
//!         eprintln!("Invalid ZKIR file");
//!     }
//! }
//! ```

pub mod error;
pub mod decoder;
pub mod formatter;
pub mod disassembler;

pub use error::{DisassemblerError, Result};
pub use disassembler::disassemble;
pub use decoder::decode;
pub use formatter::format;

#[cfg(test)]
mod tests {
    use super::*;
    use zkir_spec::{Instruction, Program, Opcode, Register};

    #[test]
    fn test_public_exports() {
        // Verify all public types/functions are accessible
        let _ = DisassemblerError::UnknownOpcode(0xFF);
    }

    #[test]
    fn test_decode_function() {
        let word = Opcode::Ecall.to_u8() as u32;
        let instr = decode(word).unwrap();
        assert_eq!(instr, Instruction::Ecall);
    }

    #[test]
    fn test_disassemble_function() {
        let program = Program::new();
        let output = disassemble(&program).unwrap();
        assert!(output.contains("ZKIR v3.4"));
    }

    #[test]
    fn test_format_function() {
        let instr = Instruction::Add {
            rd: Register::R1,
            rs1: Register::R2,
            rs2: Register::R3,
        };
        let formatted = format(&instr);
        assert!(formatted.contains("add"));
    }

    #[test]
    fn test_disassembler_error_variants() {
        let errors: Vec<DisassemblerError> = vec![
            DisassemblerError::InvalidEncoding(0xDEADBEEF),
            DisassemblerError::UnknownOpcode(0xFF),
        ];

        for err in errors {
            let _ = err.to_string();
        }
    }

    #[test]
    fn test_result_type() {
        let ok: Result<i32> = Ok(42);
        assert!(ok.is_ok());

        let err: Result<i32> = Err(DisassemblerError::UnknownOpcode(0xFF));
        assert!(err.is_err());
    }

    #[test]
    fn test_decode_all_system_instructions() {
        assert_eq!(
            decode(Opcode::Ecall.to_u8() as u32).unwrap(),
            Instruction::Ecall
        );
        assert_eq!(
            decode(Opcode::Ebreak.to_u8() as u32).unwrap(),
            Instruction::Ebreak
        );
    }

    #[test]
    fn test_decode_invalid_opcode() {
        // 0x7F is not a valid opcode
        let result = decode(0x7F);
        assert!(result.is_err());
    }

    #[test]
    fn test_disassemble_with_code() {
        let mut program = Program::new();
        program.code = vec![Opcode::Ecall.to_u8() as u32];
        program.header.code_size = 4;

        let output = disassemble(&program).unwrap();
        assert!(output.contains("ecall"));
        assert!(output.contains("1 instructions"));
    }

    #[test]
    fn test_format_all_instruction_types() {
        // Test R-type
        let r_type = Instruction::Add {
            rd: Register::R1,
            rs1: Register::R2,
            rs2: Register::R3,
        };
        assert!(format(&r_type).contains("add"));

        // Test I-type
        let i_type = Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R2,
            imm: 100,
        };
        assert!(format(&i_type).contains("addi"));

        // Test Load
        let load = Instruction::Lw {
            rd: Register::R1,
            rs1: Register::R2,
            imm: 0,
        };
        assert!(format(&load).contains("lw"));

        // Test Store
        let store = Instruction::Sw {
            rs1: Register::R2,
            rs2: Register::R1,
            imm: 0,
        };
        assert!(format(&store).contains("sw"));

        // Test Branch
        let branch = Instruction::Beq {
            rs1: Register::R1,
            rs2: Register::R2,
            offset: 8,
        };
        assert!(format(&branch).contains("beq"));

        // Test Jump
        let jump = Instruction::Jal {
            rd: Register::R1,
            offset: 100,
        };
        assert!(format(&jump).contains("jal"));
    }

    // Note: Cross-crate roundtrip tests are in the workspace-level tests/cross_module.rs
}
