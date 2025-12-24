//! ZK IR Assembler
//!
//! Assemble ZK IR assembly language into executable bytecode.
//!
//! ## Example
//!
//! ```rust
//! use zkir_assembler::assemble;
//!
//! let source = r#"
//!     ecall
//!     halt
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
pub use parser::{parse_instruction, parse_register};
pub use encoder::encode;
