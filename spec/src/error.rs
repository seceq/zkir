//! Error types for ZK IR.

use thiserror::Error;

/// Errors that can occur in ZK IR processing
#[derive(Debug, Error)]
pub enum ZkIrError {
    /// Invalid opcode byte
    #[error("Invalid opcode: 0x{0:02x}")]
    InvalidOpcode(u8),

    /// Invalid register index
    #[error("Invalid register index: {0}")]
    InvalidRegister(u8),

    /// Invalid instruction encoding
    #[error("Invalid instruction encoding: {0}")]
    InvalidInstruction(String),

    /// Invalid file format
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),

    /// Parse error (for assembler)
    #[error("Parse error at line {line}: {message}")]
    ParseError {
        line: usize,
        message: String,
    },

    /// Undefined symbol
    #[error("Undefined symbol: {0}")]
    UndefinedSymbol(String),

    /// Duplicate symbol
    #[error("Duplicate symbol: {0}")]
    DuplicateSymbol(String),

    /// Execution error
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Stack overflow
    #[error("Stack overflow")]
    StackOverflow,

    /// Stack underflow
    #[error("Stack underflow")]
    StackUnderflow,

    /// Out of bounds memory access
    #[error("Memory access out of bounds: address 0x{0:08x}")]
    MemoryOutOfBounds(u32),

    /// Assertion failed
    #[error("Assertion failed: {0}")]
    AssertionFailed(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl From<std::io::Error> for ZkIrError {
    fn from(err: std::io::Error) -> Self {
        ZkIrError::IoError(err.to_string())
    }
}

/// Result type for ZK IR operations
pub type ZkIrResult<T> = Result<T, ZkIrError>;
