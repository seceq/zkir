//! Error types for ZK IR

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZkIrError {
    #[error("Invalid instruction encoding: 0x{0:08X}")]
    InvalidEncoding(u32),

    #[error("Invalid register index: {0}")]
    InvalidRegister(u8),

    #[error("Invalid program: {0}")]
    InvalidProgram(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),
}
