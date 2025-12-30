//! # ZKIR Assembler v3.4
//!
//! Assemble ZKIR assembly language into executable bytecode with variable limb configuration.
//!
//! ## Example
//!
//! ```rust
//! use zkir_assembler::assemble;
//!
//! let source = r#"
//!     .config limb_bits 20
//!     .config data_limbs 2
//!     .config addr_limbs 2
//!
//!     add r1, r2, r3
//!     ecall
//! "#;
//!
//! let program = assemble(source).unwrap();
//! ```

pub mod error;
pub mod lexer;
pub mod parser;
pub mod encoder;
pub mod assembler;

pub use error::{AssemblerError, Result};
pub use assembler::assemble;
pub use parser::parse_register;
pub use encoder::encode;

#[cfg(test)]
mod tests {
    use super::*;
    use zkir_spec::{Instruction, Register};

    #[test]
    fn test_public_exports() {
        // Verify all public types/functions are accessible
        let _ = AssemblerError::Other("test".to_string());
    }

    #[test]
    fn test_assemble_function() {
        let source = "ecall";
        let program = assemble(source).unwrap();
        assert_eq!(program.code.len(), 1);
    }

    #[test]
    fn test_encode_function() {
        let instr = Instruction::Add {
            rd: Register::R1,
            rs1: Register::R2,
            rs2: Register::R3,
        };
        let encoded = encode(&instr);
        assert!(encoded > 0);
    }

    #[test]
    fn test_parse_register_function() {
        let reg = parse_register("r0").unwrap();
        assert_eq!(reg, Register::R0);

        let reg = parse_register("zero").unwrap();
        assert_eq!(reg, Register::R0);
    }

    #[test]
    fn test_assembler_error_variants() {
        let errors: Vec<AssemblerError> = vec![
            AssemblerError::SyntaxError {
                line: 1,
                message: "test".to_string(),
            },
            AssemblerError::InvalidInstruction {
                line: 1,
                instruction: "test".to_string(),
            },
            AssemblerError::InvalidRegister {
                line: 1,
                register: "test".to_string(),
            },
            AssemblerError::Other("test".to_string()),
        ];

        for err in errors {
            let _ = err.to_string();
        }
    }

    #[test]
    fn test_result_type() {
        let ok: Result<i32> = Ok(42);
        assert!(ok.is_ok());

        let err: Result<i32> = Err(AssemblerError::Other("test".to_string()));
        assert!(err.is_err());
    }

    #[test]
    fn test_assemble_returns_valid_program() {
        let source = r#"
            add r1, r2, r3
            sub r4, r5, r6
            ecall
        "#;
        let program = assemble(source).unwrap();

        // Program should have valid header
        assert!(program.header.code_size > 0);
        assert_eq!(program.code.len(), 3);
    }

    // Note: Cross-crate roundtrip tests are in the workspace-level tests/cross_module.rs

    #[test]
    fn test_parse_register_all_numeric() {
        for i in 0..16u8 {
            let name = format!("r{}", i);
            let reg = parse_register(&name).unwrap();
            assert_eq!(reg.index(), i);
        }
    }

    #[test]
    fn test_parse_register_abi_names() {
        let abi_names = vec![
            ("zero", Register::R0),
            ("ra", Register::R1),
            ("sp", Register::R2),
            ("a0", Register::R11),  // a0-a4 map to R11-R15 in ZKIR
            ("a1", Register::R12),
        ];

        for (name, expected) in abi_names {
            let reg = parse_register(name).unwrap();
            assert_eq!(reg, expected);
        }
    }
}
