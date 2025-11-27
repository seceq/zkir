//! Assembler errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssemblerError {
    #[error("Syntax error at line {line}, column {column}: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Unknown instruction: {0}")]
    UnknownInstruction(String),

    #[error("Invalid register: {0}")]
    InvalidRegister(String),

    #[error("Invalid immediate value: {0}")]
    InvalidImmediate(String),

    #[error("Undefined label: {0}")]
    UndefinedLabel(String),

    #[error("Duplicate label: {0}")]
    DuplicateLabel(String),

    #[error("Invalid directive: {0}")]
    InvalidDirective(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AssemblerError>;
