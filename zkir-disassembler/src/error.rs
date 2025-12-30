//! Disassembler errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DisassemblerError {
    #[error("Invalid instruction encoding: 0x{0:08X}")]
    InvalidEncoding(u32),

    #[error("Unknown opcode: 0x{0:02X}")]
    UnknownOpcode(u8),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, DisassemblerError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_invalid_encoding_display() {
        let err = DisassemblerError::InvalidEncoding(0xDEADBEEF);
        assert_eq!(err.to_string(), "Invalid instruction encoding: 0xDEADBEEF");
    }

    #[test]
    fn test_invalid_encoding_display_zero() {
        let err = DisassemblerError::InvalidEncoding(0);
        assert_eq!(err.to_string(), "Invalid instruction encoding: 0x00000000");
    }

    #[test]
    fn test_unknown_opcode_display() {
        let err = DisassemblerError::UnknownOpcode(0xFF);
        assert_eq!(err.to_string(), "Unknown opcode: 0xFF");
    }

    #[test]
    fn test_unknown_opcode_display_zero() {
        // Note: 0x00 is actually a valid opcode (ADD), but test the display format
        let err = DisassemblerError::UnknownOpcode(0x00);
        assert_eq!(err.to_string(), "Unknown opcode: 0x00");
    }

    #[test]
    fn test_unknown_opcode_display_various() {
        // Test various opcode values for proper formatting
        let test_cases = vec![
            (0x52, "Unknown opcode: 0x52"),
            (0x7F, "Unknown opcode: 0x7F"),
            (0x80, "Unknown opcode: 0x80"),
        ];

        for (opcode, expected) in test_cases {
            let err = DisassemblerError::UnknownOpcode(opcode);
            assert_eq!(err.to_string(), expected);
        }
    }

    #[test]
    fn test_io_error_from() {
        let io_err = IoError::new(ErrorKind::NotFound, "file not found");
        let disasm_err: DisassemblerError = io_err.into();
        assert!(disasm_err.to_string().contains("file not found"));
    }

    #[test]
    fn test_io_error_various_kinds() {
        let error_kinds = vec![
            (ErrorKind::PermissionDenied, "permission denied"),
            (ErrorKind::UnexpectedEof, "unexpected eof"),
            (ErrorKind::InvalidData, "invalid data"),
        ];

        for (kind, msg) in error_kinds {
            let io_err = IoError::new(kind, msg);
            let disasm_err: DisassemblerError = io_err.into();
            assert!(disasm_err.to_string().contains(msg));
        }
    }

    #[test]
    fn test_error_debug_format() {
        let err = DisassemblerError::InvalidEncoding(0x12345678);
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("InvalidEncoding"));
        assert!(debug_str.contains("305419896")); // 0x12345678 in decimal
    }

    #[test]
    fn test_unknown_opcode_debug_format() {
        let err = DisassemblerError::UnknownOpcode(0xAB);
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("UnknownOpcode"));
        assert!(debug_str.contains("171")); // 0xAB in decimal
    }

    #[test]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(DisassemblerError::UnknownOpcode(0xFF));
        assert!(result.is_err());
    }

    #[test]
    fn test_all_variants_display() {
        // Ensure all error variants have proper Display implementations
        let errors: Vec<DisassemblerError> = vec![
            DisassemblerError::InvalidEncoding(0),
            DisassemblerError::UnknownOpcode(0),
        ];

        for err in errors {
            let display = err.to_string();
            assert!(!display.is_empty());
        }
    }

    #[test]
    fn test_encoding_hex_format() {
        // Verify that encoding is always displayed with 8 hex digits (zero-padded)
        let test_cases = vec![
            (0x00000001, "0x00000001"),
            (0x0000FFFF, "0x0000FFFF"),
            (0xFFFFFFFF, "0xFFFFFFFF"),
        ];

        for (encoding, expected_substr) in test_cases {
            let err = DisassemblerError::InvalidEncoding(encoding);
            assert!(
                err.to_string().contains(expected_substr),
                "Expected {} in error message for encoding {:#x}",
                expected_substr,
                encoding
            );
        }
    }

    #[test]
    fn test_opcode_hex_format() {
        // Verify that opcode is always displayed with 2 hex digits (zero-padded)
        let test_cases = vec![
            (0x01, "0x01"),
            (0x0F, "0x0F"),
            (0x10, "0x10"),
        ];

        for (opcode, expected_substr) in test_cases {
            let err = DisassemblerError::UnknownOpcode(opcode);
            assert!(
                err.to_string().contains(expected_substr),
                "Expected {} in error message for opcode {:#x}",
                expected_substr,
                opcode
            );
        }
    }
}
