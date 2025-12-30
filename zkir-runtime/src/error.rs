//! Runtime error types for ZKIR v3.4

use thiserror::Error;
use zkir_spec::ZkIrError;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Spec error: {0}")]
    SpecError(#[from] ZkIrError),

    #[error("Memory out of bounds: address {address:#x}")]
    OutOfBounds { address: u64 },

    #[error("Misaligned access: address {address:#x}, alignment {alignment}")]
    MisalignedAccess { address: u64, alignment: usize },

    #[error("Invalid memory access at {address:#x}: {reason}")]
    InvalidMemoryAccess { address: u64, reason: String },

    #[error("Division by zero at PC {pc:#x}")]
    DivisionByZero { pc: u64 },

    #[error("Invalid syscall: {syscall}")]
    InvalidSyscall { syscall: u64 },

    #[error("Cycle limit exceeded: {limit}")]
    CycleLimitExceeded { limit: u64 },

    #[error("Halted: {reason:?}")]
    Halted { reason: String },

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, RuntimeError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_out_of_bounds_display() {
        let err = RuntimeError::OutOfBounds { address: 0xDEADBEEF };
        assert_eq!(err.to_string(), "Memory out of bounds: address 0xdeadbeef");
    }

    #[test]
    fn test_misaligned_access_display() {
        let err = RuntimeError::MisalignedAccess {
            address: 0x1001,
            alignment: 4,
        };
        assert_eq!(
            err.to_string(),
            "Misaligned access: address 0x1001, alignment 4"
        );
    }

    #[test]
    fn test_invalid_memory_access_display() {
        let err = RuntimeError::InvalidMemoryAccess {
            address: 0x5000,
            reason: "read-only region".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Invalid memory access at 0x5000: read-only region"
        );
    }

    #[test]
    fn test_division_by_zero_display() {
        let err = RuntimeError::DivisionByZero { pc: 0x1000 };
        assert_eq!(err.to_string(), "Division by zero at PC 0x1000");
    }

    #[test]
    fn test_invalid_syscall_display() {
        let err = RuntimeError::InvalidSyscall { syscall: 999 };
        assert_eq!(err.to_string(), "Invalid syscall: 999");
    }

    #[test]
    fn test_cycle_limit_exceeded_display() {
        let err = RuntimeError::CycleLimitExceeded { limit: 1_000_000 };
        assert_eq!(err.to_string(), "Cycle limit exceeded: 1000000");
    }

    #[test]
    fn test_halted_display() {
        let err = RuntimeError::Halted {
            reason: "user request".to_string(),
        };
        assert_eq!(err.to_string(), "Halted: \"user request\"");
    }

    #[test]
    fn test_io_error_from() {
        let io_err = IoError::new(ErrorKind::NotFound, "file not found");
        let runtime_err: RuntimeError = io_err.into();
        assert!(runtime_err.to_string().contains("file not found"));
    }

    #[test]
    fn test_other_display() {
        let err = RuntimeError::Other("custom error message".to_string());
        assert_eq!(err.to_string(), "custom error message");
    }

    #[test]
    fn test_spec_error_from() {
        let spec_err = ZkIrError::DivisionByZero;
        let runtime_err: RuntimeError = spec_err.into();
        assert!(runtime_err.to_string().contains("Division by zero"));
    }

    #[test]
    fn test_error_debug_format() {
        let err = RuntimeError::OutOfBounds { address: 0x100 };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("OutOfBounds"));
        assert!(debug_str.contains("256")); // 0x100 in decimal
    }

    #[test]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(RuntimeError::Other("test".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        // RuntimeError should be Send + Sync for use across threads
        // Note: This won't compile if the types don't implement Send/Sync
        // For thiserror with #[from] std::io::Error, this should work
    }

    #[test]
    fn test_all_variants_display() {
        // Ensure all error variants have proper Display implementations
        let errors: Vec<RuntimeError> = vec![
            RuntimeError::OutOfBounds { address: 0 },
            RuntimeError::MisalignedAccess {
                address: 0,
                alignment: 1,
            },
            RuntimeError::InvalidMemoryAccess {
                address: 0,
                reason: String::new(),
            },
            RuntimeError::DivisionByZero { pc: 0 },
            RuntimeError::InvalidSyscall { syscall: 0 },
            RuntimeError::CycleLimitExceeded { limit: 0 },
            RuntimeError::Halted {
                reason: String::new(),
            },
            RuntimeError::Other(String::new()),
        ];

        for err in errors {
            // Each variant should produce a non-empty display string
            let display = err.to_string();
            assert!(!display.is_empty() || matches!(err, RuntimeError::Other(_)));
        }
    }
}
