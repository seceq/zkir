//! # ZK IR Specification
//!
//! 32-bit register-based instruction set for zero-knowledge proof generation.
//!
//! ## Key Features
//! - 32-bit architecture (RISC-V inspired)
//! - Baby Bear field (p = 2^31 - 2^27 + 1)
//! - No field registers (use syscalls for crypto)
//! - Register pairs for 64-bit values
//! - Syscalls for cryptographic operations

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

/// Magic number for ZKIR files: "ZKIR" = 0x5A4B4952
pub const MAGIC: u32 = 0x5A4B4952;

/// Version
pub const VERSION: u32 = 0x00020001;

/// Memory layout constants
pub const CODE_BASE: u32 = 0x0000_1000;
pub const DATA_BASE: u32 = 0x1000_0000;
pub const HEAP_BASE: u32 = 0x8000_0000;
pub const STACK_TOP: u32 = 0xFFFF_0000;

/// Default sizes
pub const DEFAULT_STACK_SIZE: u32 = 1 << 20;  // 1 MB
pub const DEFAULT_HEAP_SIZE: u32 = 1 << 20;   // 1 MB
pub const DEFAULT_CODE_SIZE: u32 = 1 << 18;   // 256 KB

/// Word size (32-bit)
pub type Word = u32;

/// Address type (32-bit)
pub type Address = u32;

/// Signed word
pub type SWord = i32;
