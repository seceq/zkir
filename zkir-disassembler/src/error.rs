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
