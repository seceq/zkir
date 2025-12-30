//! # Error Types for ZKIR v3.4

use crate::config::ConfigError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZkIrError {
    // Configuration errors
    #[error("Invalid configuration: {0}")]
    InvalidConfig(#[from] ConfigError),

    // Program format errors
    #[error("Invalid program magic: expected 0x5A4B4952, got {0:#010x}")]
    InvalidMagic(u32),

    #[error("Invalid program version: expected {expected:#010x}, found {found:#010x}")]
    InvalidVersion { expected: u32, found: u32 },

    #[error("Invalid header size: expected {expected} bytes, found {found} bytes")]
    InvalidHeaderSize { expected: usize, found: usize },

    #[error("Invalid program size: expected {expected} bytes, found {found} bytes")]
    InvalidProgramSize { expected: usize, found: usize },

    #[error("Invalid code size: expected {expected} bytes, found {found} bytes")]
    InvalidCodeSize { expected: usize, found: usize },

    #[error("Invalid data size: expected {expected} bytes, found {found} bytes")]
    InvalidDataSize { expected: usize, found: usize },

    // Instruction errors
    #[error("Invalid instruction encoding: {0:#010x}")]
    InvalidEncoding(u32),

    #[error("Invalid opcode: {0:#04x}")]
    InvalidOpcode(u8),

    #[error("Invalid register index: {0} (valid range: 0-15)")]
    InvalidRegister(u8),

    #[error("Invalid immediate value: {0}")]
    InvalidImmediate(i32),

    // Runtime errors
    #[error("Memory alignment error: address {address:#010x} is not aligned to {alignment} bytes")]
    MisalignedAccess { address: u64, alignment: usize },

    #[error("Memory access out of bounds: address {address:#010x}")]
    OutOfBounds { address: u64 },

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Invalid syscall number: {0}")]
    InvalidSyscall(u32),

    #[error("Range check failed: value {value} exceeds {max_bits} bits")]
    RangeCheckFailed { value: u64, max_bits: u32 },

    // I/O errors
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    // General errors
    #[error("{0}")]
    Other(String),
}

impl ZkIrError {
    /// Check if this is a fatal error that should halt execution
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            ZkIrError::DivisionByZero
                | ZkIrError::InvalidEncoding(_)
                | ZkIrError::MisalignedAccess { .. }
                | ZkIrError::OutOfBounds { .. }
                | ZkIrError::RangeCheckFailed { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ZkIrError::InvalidRegister(16);
        assert_eq!(
            err.to_string(),
            "Invalid register index: 16 (valid range: 0-15)"
        );

        let err = ZkIrError::DivisionByZero;
        assert_eq!(err.to_string(), "Division by zero");
    }

    #[test]
    fn test_is_fatal() {
        assert!(ZkIrError::DivisionByZero.is_fatal());
        assert!(ZkIrError::InvalidEncoding(0).is_fatal());
        assert!(!ZkIrError::InvalidMagic(0).is_fatal());
    }
}
