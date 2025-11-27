//! # ZK IR Specification
//!
//! This crate defines the core types and encoding for the ZK IR bytecode format.
//!
//! ## Overview
//!
//! ZK IR is a bytecode format designed for efficient zero-knowledge proof generation.
//! It features:
//! - Simple instruction set (~40 instructions)
//! - Fixed-width 64-bit instruction encoding
//! - ZK-friendly operations (native field arithmetic, hashing)
//! - Deterministic execution
//!
//! ## Example
//!
//! ```rust
//! use zkir_spec::{Instruction, Opcode, Program, Register};
//!
//! // Create a simple program: r1 = r2 + r3
//! let instructions = vec![
//!     Instruction::new_r(Opcode::Add, Register::R1, Register::R2, Register::R3),
//!     Instruction::new_halt(),
//! ];
//!
//! let program = Program::new(instructions);
//! let bytes = program.to_bytes();
//! ```

pub mod opcode;
pub mod instruction;
pub mod register;
pub mod program;
pub mod encoding;
pub mod field;
pub mod error;

pub use opcode::Opcode;
pub use instruction::Instruction;
pub use register::Register;
pub use program::Program;
pub use field::FieldElement;
pub use error::ZkIrError;

/// Magic number for ZKBC files: "ZKBC" in ASCII
pub const MAGIC: u32 = 0x5A4B4243;

/// Current format version: 1.0.0
pub const VERSION: u32 = 0x00010000;

/// Number of general-purpose registers
pub const NUM_REGISTERS: usize = 32;

/// Number of field registers
pub const NUM_FIELD_REGISTERS: usize = 16;

/// Default stack size in words
pub const DEFAULT_STACK_SIZE: u32 = 0x100000; // 1MB worth of field elements

/// Default heap size in words  
pub const DEFAULT_HEAP_SIZE: u32 = 0x1000000; // 16MB worth of field elements
