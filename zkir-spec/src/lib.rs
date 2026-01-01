//! # ZKIR Specification v3.4
//!
//! Variable limb architecture for zero-knowledge proof generation.
//!
//! ## Key Features
//! - Variable limb sizes (16-30 bits, default 20)
//! - Variable data widths (16-120 bits, default 40)
//! - Mersenne 31 field (p = 2^31 - 1)
//! - 16 general-purpose registers (r0-r15)
//! - 32-bit instructions
//! - Crypto-aware bound tracking
//! - Deferred range checking with headroom

pub mod config;
pub mod value;
pub mod bound;
pub mod field;
pub mod register;
pub mod instruction;
pub mod opcode;
pub mod encoding;
pub mod error;
pub mod program;
pub mod trace;
pub mod validation;

// Re-export commonly used types
pub use config::{Config, ConfigError};
pub use value::{Value, Value40, GenericValue, Value40Generic, Value60, Value80, Value30, Value64};
pub use bound::{
    BoundSource, BoundedValue, CryptoType, ValueBound,
};
pub use field::{Mersenne31, MERSENNE31_PRIME};
pub use register::{Register, NUM_REGISTERS};
pub use instruction::Instruction;
pub use opcode::{Opcode, InstructionFamily};
pub use error::ZkIrError;
pub use program::{Program, ProgramHeader, FormatMode, MAGIC, VERSION};
pub use trace::{
    TraceRow, MemoryOp, MemOpType, CryptoWitness, Sha256Witness,
    Poseidon2Witness, Keccak256Witness, RegisterState,
};
pub use validation::{
    validate, validate_program, ValidationError, ValidationResult, ValidationWarning,
};

/// Memory layout constants for default 40-bit address space
pub mod memory {
    /// Reserved region
    pub const RESERVED_BASE: u64 = 0x00_0000_0000;
    pub const RESERVED_SIZE: u64 = 0x1000; // 4 KB

    /// Code section (256 MB)
    pub const CODE_BASE: u64 = 0x00_0000_1000;
    pub const CODE_SIZE: u64 = 0x10_0000_000;

    /// Static data section (256 MB)
    pub const DATA_BASE: u64 = 0x10_0000_000;
    pub const DATA_SIZE: u64 = 0x10_0000_000;

    /// Heap (starts after data, ~1 TB)
    pub const HEAP_BASE: u64 = 0x20_0000_000;

    /// Stack (top of address space, ~768 GB)
    pub const STACK_TOP: u64 = 0xFF_FFFF_FFFF;

    /// Default sizes
    pub const DEFAULT_STACK_SIZE: usize = 1 << 20; // 1 MB
    pub const DEFAULT_HEAP_SIZE: usize = 1 << 20; // 1 MB
}

/// ABI constants for ZKIR v3.4 calling convention
///
/// These constants define the Application Binary Interface (ABI) parameters
/// for the ZKIR calling convention documented in the Register module.
pub mod abi {
    /// Register size in bytes (32-bit registers).
    ///
    /// Each ZKIR register is encoded as a 32-bit word in stack frames and memory,
    /// regardless of the number of internal limbs. This is the fundamental unit
    /// for stack frame calculations and parameter passing.
    pub const REGISTER_SIZE_BYTES: usize = 4;

    /// Parameter alignment in bytes (word-aligned).
    ///
    /// Stack-passed arguments (those that don't fit in the 6 argument registers)
    /// are aligned to register size boundaries (4-byte alignment). This ensures
    /// proper memory access alignment and compatibility across implementations.
    pub const PARAM_ALIGNMENT: usize = 4;

    /// Stack frame alignment in bytes.
    ///
    /// All stack frames must be 16-byte aligned per the ZKIR calling convention.
    /// This alignment requirement ensures:
    /// - Performance on modern architectures
    /// - Compatibility with SIMD operations
    /// - Consistency across different ZKIR implementations
    pub const FRAME_ALIGNMENT: usize = 16;
}

/// Instruction size in bytes
pub const INSTRUCTION_SIZE: usize = 4;

/// Address type (configurable based on addr_limbs)
pub type Address = u64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::DEFAULT;
        assert_eq!(config.limb_bits, 20);
        assert_eq!(config.data_limbs, 2);
        assert_eq!(config.addr_limbs, 2);
        assert_eq!(config.data_bits(), 40);
        assert_eq!(config.addr_bits(), 40);
    }

    #[test]
    fn test_value40() {
        let v = Value40::from_u64(0x123456789);
        assert_eq!(v.to_u64(), 0x123456789);
    }

    #[test]
    fn test_bound_tracking() {
        // Test crypto output bound (algorithm bits, not internal bits)
        let bound = ValueBound::from_crypto(CryptoType::Sha256);
        assert_eq!(bound.max_bits, 32); // Algorithm width
        assert!(!bound.needs_range_check(40)); // 32 <= 40: no range check

        // Test adaptive internal representation
        let sha = CryptoType::Sha256;
        assert_eq!(sha.internal_bits(40), 44); // Uses min (44-bit)
        assert_eq!(sha.internal_bits(60), 60); // Uses program (60-bit)
        assert_eq!(sha.post_crypto_headroom(40), 8); // 40 - 32
    }

    #[test]
    fn test_program_header() {
        let header = ProgramHeader::new();
        assert_eq!(header.magic, MAGIC);
        assert_eq!(header.version, VERSION);
    }

    #[test]
    fn test_abi_constants() {
        // Verify ABI constants match ZKIR v3.4 specification
        assert_eq!(abi::REGISTER_SIZE_BYTES, 4);
        assert_eq!(abi::PARAM_ALIGNMENT, 4);
        assert_eq!(abi::FRAME_ALIGNMENT, 16);

        // Verify FRAME_ALIGNMENT is a power of 2
        assert_eq!(abi::FRAME_ALIGNMENT & (abi::FRAME_ALIGNMENT - 1), 0);

        // Verify PARAM_ALIGNMENT equals REGISTER_SIZE_BYTES
        assert_eq!(abi::PARAM_ALIGNMENT, abi::REGISTER_SIZE_BYTES);
    }
}
