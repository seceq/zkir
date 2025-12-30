//! # Error Types for ZKIR Assembler v3.4

use thiserror::Error;
use zkir_spec::{ConfigError, ZkIrError};

/// Assembler error types
#[derive(Debug, Error)]
pub enum AssemblerError {
    /// Invalid syntax
    #[error("Syntax error at line {line}: {message}")]
    SyntaxError { line: usize, message: String },

    /// Invalid instruction
    #[error("Invalid instruction at line {line}: {instruction}")]
    InvalidInstruction { line: usize, instruction: String },

    /// Invalid register
    #[error("Invalid register at line {line}: {register}")]
    InvalidRegister { line: usize, register: String },

    /// Invalid immediate value
    #[error("Invalid immediate value at line {line}: {value}")]
    InvalidImmediate { line: usize, value: String },

    /// Invalid label
    #[error("Undefined label at line {line}: {label}")]
    UndefinedLabel { line: usize, label: String },

    /// Duplicate label
    #[error("Duplicate label at line {line}: {label}")]
    DuplicateLabel { line: usize, label: String },

    /// Invalid directive
    #[error("Invalid directive at line {line}: {directive}")]
    InvalidDirective { line: usize, directive: String },

    /// Configuration error
    #[error("Configuration error at line {line}: {source}")]
    ConfigError {
        line: usize,
        source: ConfigError,
    },

    /// Invalid config value
    #[error("Invalid config value at line {line}: {key}={value}")]
    InvalidConfigValue {
        line: usize,
        key: String,
        value: String,
    },

    /// ZkIr spec error
    #[error("Spec error: {0}")]
    SpecError(#[from] ZkIrError),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// General error
    #[error("{0}")]
    Other(String),
}

/// Result type for assembler operations
pub type Result<T> = std::result::Result<T, AssemblerError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_syntax_error_display() {
        let err = AssemblerError::SyntaxError {
            line: 10,
            message: "unexpected token".to_string(),
        };
        assert_eq!(err.to_string(), "Syntax error at line 10: unexpected token");
    }

    #[test]
    fn test_invalid_instruction_display() {
        let err = AssemblerError::InvalidInstruction {
            line: 5,
            instruction: "foobar".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid instruction at line 5: foobar");
    }

    #[test]
    fn test_invalid_register_display() {
        let err = AssemblerError::InvalidRegister {
            line: 3,
            register: "r99".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid register at line 3: r99");
    }

    #[test]
    fn test_invalid_immediate_display() {
        let err = AssemblerError::InvalidImmediate {
            line: 7,
            value: "abc".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid immediate value at line 7: abc");
    }

    #[test]
    fn test_undefined_label_display() {
        let err = AssemblerError::UndefinedLabel {
            line: 15,
            label: "missing_label".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Undefined label at line 15: missing_label"
        );
    }

    #[test]
    fn test_duplicate_label_display() {
        let err = AssemblerError::DuplicateLabel {
            line: 20,
            label: "main".to_string(),
        };
        assert_eq!(err.to_string(), "Duplicate label at line 20: main");
    }

    #[test]
    fn test_invalid_directive_display() {
        let err = AssemblerError::InvalidDirective {
            line: 2,
            directive: ".unknown".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid directive at line 2: .unknown");
    }

    #[test]
    fn test_invalid_config_value_display() {
        let err = AssemblerError::InvalidConfigValue {
            line: 1,
            key: "limb_bits".to_string(),
            value: "invalid".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Invalid config value at line 1: limb_bits=invalid"
        );
    }

    #[test]
    fn test_io_error_from() {
        let io_err = IoError::new(ErrorKind::NotFound, "file not found");
        let asm_err: AssemblerError = io_err.into();
        assert!(asm_err.to_string().contains("file not found"));
    }

    #[test]
    fn test_other_display() {
        let err = AssemblerError::Other("custom error".to_string());
        assert_eq!(err.to_string(), "custom error");
    }

    #[test]
    fn test_spec_error_from() {
        let spec_err = ZkIrError::InvalidRegister(99);
        let asm_err: AssemblerError = spec_err.into();
        assert!(asm_err.to_string().contains("Invalid register"));
    }

    #[test]
    fn test_config_error_display() {
        let config_err = ConfigError::InvalidLimbBits;
        let err = AssemblerError::ConfigError {
            line: 1,
            source: config_err,
        };
        let display = err.to_string();
        assert!(display.contains("Configuration error at line 1"));
    }

    #[test]
    fn test_error_debug_format() {
        let err = AssemblerError::SyntaxError {
            line: 1,
            message: "test".to_string(),
        };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("SyntaxError"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(AssemblerError::Other("test".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_all_variants_display() {
        // Ensure all error variants have proper Display implementations
        let errors: Vec<AssemblerError> = vec![
            AssemblerError::SyntaxError {
                line: 1,
                message: String::new(),
            },
            AssemblerError::InvalidInstruction {
                line: 1,
                instruction: String::new(),
            },
            AssemblerError::InvalidRegister {
                line: 1,
                register: String::new(),
            },
            AssemblerError::InvalidImmediate {
                line: 1,
                value: String::new(),
            },
            AssemblerError::UndefinedLabel {
                line: 1,
                label: String::new(),
            },
            AssemblerError::DuplicateLabel {
                line: 1,
                label: String::new(),
            },
            AssemblerError::InvalidDirective {
                line: 1,
                directive: String::new(),
            },
            AssemblerError::InvalidConfigValue {
                line: 1,
                key: String::new(),
                value: String::new(),
            },
            AssemblerError::Other(String::new()),
        ];

        for err in errors {
            // Each variant should produce a display string
            let display = err.to_string();
            assert!(!display.is_empty() || matches!(err, AssemblerError::Other(_)));
        }
    }

    #[test]
    fn test_error_line_numbers() {
        // Verify line numbers are correctly included in error messages
        let errors = vec![
            (
                AssemblerError::SyntaxError {
                    line: 42,
                    message: "msg".to_string(),
                },
                "42",
            ),
            (
                AssemblerError::InvalidInstruction {
                    line: 100,
                    instruction: "x".to_string(),
                },
                "100",
            ),
            (
                AssemblerError::InvalidRegister {
                    line: 1,
                    register: "x".to_string(),
                },
                "1",
            ),
        ];

        for (err, expected_line) in errors {
            assert!(
                err.to_string().contains(expected_line),
                "Error should contain line number {}",
                expected_line
            );
        }
    }
}
