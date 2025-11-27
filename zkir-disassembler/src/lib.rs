//! ZK IR Disassembler
//!
//! Disassemble ZK IR bytecode into human-readable assembly.
//!
//! ## Example
//!
//! ```rust
//! use zkir_spec::Program;
//! use zkir_disassembler::disassemble;
//!
//! let code = vec![0x00000073]; // ecall
//! let program = Program::new(code);
//! let asm = disassemble(&program).unwrap();
//! println!("{}", asm);
//! ```

pub mod error;
pub mod decoder;
pub mod formatter;
pub mod disassembler;

pub use error::{DisassemblerError, Result};
pub use disassembler::disassemble;
pub use decoder::decode;
pub use formatter::format;
