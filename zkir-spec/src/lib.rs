//! # ZK IR Specification v2.2
//!
//! 30-bit register-based instruction set for zero-knowledge proof generation.
//!
//! ## Key Features
//! - 30-bit data width (fits in Baby Bear field)
//! - 30-bit instructions stored in 32-bit slots
//! - Baby Bear field (p = 2^31 - 2^27 + 1)
//! - 32 general-purpose registers
//! - Harvard architecture (separate code/data memory)
//! - Field arithmetic instructions (FADD, FSUB, FMUL, FNEG, FINV)
//! - Syscalls for cryptographic operations (Poseidon2, SHA-256)

pub mod field;
pub mod register;
pub mod instruction;
pub mod error;
pub mod program;

pub use field::{BabyBear, BABYBEAR_PRIME};
pub use register::{Register, NUM_REGISTERS};
pub use instruction::Instruction;
pub use error::ZkIrError;
pub use program::{Program, ProgramHeader};

/// Magic number for ZKIR files: "ZK22" = 0x5A4B3232
pub const MAGIC: u32 = 0x5A4B3232;

/// Version: v2.2 = 0x00020002
pub const VERSION: u32 = 0x00020002;

/// Memory layout constants (30-bit address space)
pub const CODE_BASE: u32 = 0x0000_1000;
pub const DATA_BASE: u32 = 0x1000_0000;
pub const HEAP_BASE: u32 = 0x2000_0000;
pub const STACK_TOP: u32 = 0x3FFF_F000;

/// Maximum 30-bit value
pub const MAX_30BIT: u32 = (1 << 30) - 1;

/// Default sizes
pub const DEFAULT_STACK_SIZE: u32 = 1 << 20;  // 1 MB
pub const DEFAULT_HEAP_SIZE: u32 = 1 << 20;   // 1 MB
pub const DEFAULT_CODE_SIZE: u32 = 1 << 18;   // 256 KB

/// Word size (30-bit data, stored in 32-bit)
pub type Word = u32;

/// Address type (30-bit)
pub type Address = u32;

/// Signed word (30-bit represented as i32)
pub type SWord = i32;
