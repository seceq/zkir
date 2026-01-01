//! # Memory and Execution Trace Types
//!
//! Types for collecting execution traces for proof generation.

use crate::bound::{ValueBound, CryptoType};
use std::cmp::Ordering;

/// Register storage state for deferred carry model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RegisterState {
    /// Register contains a normalized value (20-bit limbs packed with normalized_bits)
    #[default]
    Normalized,
    /// Register contains an accumulated value (30-bit limbs packed with limb_bits)
    Accumulated,
}

/// A single execution trace row
///
/// Records the complete VM state at a single cycle for proof generation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraceRow {
    /// Cycle number
    pub cycle: u64,

    /// Program counter
    pub pc: u64,

    /// Encoded instruction (32-bit)
    pub instruction: u32,

    /// Register state (16 registers)
    pub registers: [u64; 16],

    /// Bounds for each register value
    pub bounds: [ValueBound; 16],

    /// Register storage states (Normalized vs Accumulated)
    /// Used by the converter to determine correct unpacking method
    pub register_states: [RegisterState; 16],

    /// Memory operations performed during this cycle
    ///
    /// - Empty for regular ALU instructions
    /// - Single element for LW/SW instructions
    /// - Multiple elements for ECALL (syscalls that perform multiple memory accesses)
    pub memory_ops: Vec<MemoryOp>,
}

impl TraceRow {
    /// Create a new trace row with no memory operations
    pub fn new(
        cycle: u64,
        pc: u64,
        instruction: u32,
        registers: [u64; 16],
        bounds: [ValueBound; 16],
    ) -> Self {
        Self {
            cycle,
            pc,
            instruction,
            registers,
            bounds,
            register_states: [RegisterState::Normalized; 16],
            memory_ops: Vec::new(),
        }
    }

    /// Create a new trace row with register states
    pub fn with_register_states(
        cycle: u64,
        pc: u64,
        instruction: u32,
        registers: [u64; 16],
        bounds: [ValueBound; 16],
        register_states: [RegisterState; 16],
    ) -> Self {
        Self {
            cycle,
            pc,
            instruction,
            registers,
            bounds,
            register_states,
            memory_ops: Vec::new(),
        }
    }

    /// Create a new trace row with a single memory operation
    pub fn with_memory_op(
        cycle: u64,
        pc: u64,
        instruction: u32,
        registers: [u64; 16],
        bounds: [ValueBound; 16],
        memory_op: MemoryOp,
    ) -> Self {
        Self {
            cycle,
            pc,
            instruction,
            registers,
            bounds,
            register_states: [RegisterState::Normalized; 16],
            memory_ops: vec![memory_op],
        }
    }

    /// Create a new trace row with multiple memory operations (for syscalls)
    pub fn with_memory_ops(
        cycle: u64,
        pc: u64,
        instruction: u32,
        registers: [u64; 16],
        bounds: [ValueBound; 16],
        memory_ops: Vec<MemoryOp>,
    ) -> Self {
        Self {
            cycle,
            pc,
            instruction,
            registers,
            bounds,
            register_states: [RegisterState::Normalized; 16],
            memory_ops,
        }
    }
}

/// Memory operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MemOpType {
    /// Memory read operation
    Read,
    /// Memory write operation
    Write,
}

/// Memory operation trace entry
///
/// Records a single memory access for proof generation.
/// Operations are sorted by timestamp for memory consistency verification.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemoryOp {
    /// Memory address accessed
    pub address: u64,

    /// Value read or written (40-bit)
    pub value: u64,

    /// Timestamp (cycle number) of the operation
    pub timestamp: u64,

    /// Operation type (read or write)
    pub op_type: MemOpType,

    /// Value bound at time of operation
    pub bound: ValueBound,

    /// Access width in bytes (1, 2, 4, or 8)
    pub width: u8,
}

impl MemoryOp {
    /// Create a new memory read operation
    #[inline]
    pub fn read(address: u64, value: u64, timestamp: u64, bound: ValueBound, width: u8) -> Self {
        Self {
            address,
            value,
            timestamp,
            op_type: MemOpType::Read,
            bound,
            width,
        }
    }

    /// Create a new memory write operation
    #[inline]
    pub fn write(address: u64, value: u64, timestamp: u64, bound: ValueBound, width: u8) -> Self {
        Self {
            address,
            value,
            timestamp,
            op_type: MemOpType::Write,
            bound,
            width,
        }
    }

    /// Check if this is a read operation
    #[inline]
    pub fn is_read(&self) -> bool {
        self.op_type == MemOpType::Read
    }

    /// Check if this is a write operation
    #[inline]
    pub fn is_write(&self) -> bool {
        self.op_type == MemOpType::Write
    }
}

/// Ordering for MemoryOp: sort by timestamp, then address, then operation type
impl Ord for MemoryOp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
            .then_with(|| self.address.cmp(&other.address))
            .then_with(|| {
                // Reads before writes at same timestamp/address
                match (self.op_type, other.op_type) {
                    (MemOpType::Read, MemOpType::Write) => Ordering::Less,
                    (MemOpType::Write, MemOpType::Read) => Ordering::Greater,
                    _ => Ordering::Equal,
                }
            })
    }
}

impl PartialOrd for MemoryOp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// SHA-256 round witness for proof generation
///
/// Captures intermediate states for all 64 compression rounds.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sha256Witness {
    /// Input message block (16 words × 32 bits)
    pub message_block: [u32; 16],

    /// Initial hash state (8 words × 32 bits)
    pub initial_state: [u32; 8],

    /// Message schedule (64 words × 32 bits)
    /// W[0..16] = message_block, W[16..64] computed
    pub message_schedule: [u32; 64],

    /// Intermediate states for each round (64 rounds × 8 words)
    /// round_states[i] = state after round i
    pub round_states: Vec<[u32; 8]>,

    /// Final hash output (8 words × 32 bits)
    pub final_state: [u32; 8],

    /// Cycle/timestamp when this operation occurred
    pub timestamp: u64,
}

impl Sha256Witness {
    /// Create a new SHA-256 witness with the given capacity for round states
    pub fn new(timestamp: u64) -> Self {
        Self {
            message_block: [0; 16],
            initial_state: [0; 8],
            message_schedule: [0; 64],
            round_states: Vec::with_capacity(64),
            final_state: [0; 8],
            timestamp,
        }
    }

    /// Record a round state
    pub fn record_round(&mut self, round: usize, state: [u32; 8]) {
        if round < 64 {
            if self.round_states.len() <= round {
                self.round_states.resize(round + 1, [0; 8]);
            }
            self.round_states[round] = state;
        }
    }

    /// Get the number of recorded rounds
    pub fn num_rounds(&self) -> usize {
        self.round_states.len()
    }
}

/// Poseidon2 witness for proof generation (placeholder)
///
/// Poseidon2 operates on field elements in Mersenne-31 field.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Poseidon2Witness {
    /// Input state (field elements)
    pub input_state: Vec<u32>,

    /// Round states (one per round)
    pub round_states: Vec<Vec<u32>>,

    /// Output state (field elements)
    pub output_state: Vec<u32>,

    /// Cycle/timestamp when this operation occurred
    pub timestamp: u64,
}

/// Keccak-256 witness for proof generation (placeholder)
///
/// Keccak-256 operates on 64-bit lanes in a 5×5 state array.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Keccak256Witness {
    /// Input state (5×5 array of 64-bit lanes)
    pub input_state: [[u64; 5]; 5],

    /// Round states (24 rounds, each with 5×5 lanes)
    pub round_states: Vec<[[u64; 5]; 5]>,

    /// Output state (5×5 array of 64-bit lanes)
    pub output_state: [[u64; 5]; 5],

    /// Cycle/timestamp when this operation occurred
    pub timestamp: u64,
}

/// Cryptographic operation witness
///
/// Tagged union for different crypto operation types.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CryptoWitness {
    /// SHA-256 hash operation
    Sha256(Sha256Witness),

    /// Poseidon2 hash operation
    Poseidon2(Poseidon2Witness),

    /// Keccak-256 hash operation
    Keccak256(Keccak256Witness),
}

impl CryptoWitness {
    /// Get the timestamp of this crypto operation
    pub fn timestamp(&self) -> u64 {
        match self {
            CryptoWitness::Sha256(w) => w.timestamp,
            CryptoWitness::Poseidon2(w) => w.timestamp,
            CryptoWitness::Keccak256(w) => w.timestamp,
        }
    }

    /// Get the crypto type
    pub fn crypto_type(&self) -> CryptoType {
        match self {
            CryptoWitness::Sha256(_) => CryptoType::Sha256,
            CryptoWitness::Poseidon2(_) => CryptoType::Poseidon2,
            CryptoWitness::Keccak256(_) => CryptoType::Keccak256,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bound::ValueBound;

    #[test]
    fn test_memory_op_creation() {
        let bound = ValueBound::from_constant(0x1234);

        let read_op = MemoryOp::read(0x1000, 0x1234, 100, bound, 4);
        assert!(read_op.is_read());
        assert!(!read_op.is_write());
        assert_eq!(read_op.address, 0x1000);
        assert_eq!(read_op.value, 0x1234);
        assert_eq!(read_op.timestamp, 100);
        assert_eq!(read_op.width, 4);

        let write_op = MemoryOp::write(0x2000, 0x5678, 200, bound, 4);
        assert!(write_op.is_write());
        assert!(!write_op.is_read());
        assert_eq!(write_op.address, 0x2000);
        assert_eq!(write_op.value, 0x5678);
        assert_eq!(write_op.timestamp, 200);
        assert_eq!(write_op.width, 4);
    }

    #[test]
    fn test_memory_op_ordering() {
        let bound = ValueBound::from_constant(0);

        let op1 = MemoryOp::read(0x1000, 0, 100, bound, 4);
        let op2 = MemoryOp::write(0x1000, 0, 100, bound, 4);
        let op3 = MemoryOp::read(0x1000, 0, 101, bound, 4);

        // Same timestamp, same address: reads before writes
        assert!(op1 < op2);

        // Different timestamp
        assert!(op2 < op3);

        // Transitivity
        assert!(op1 < op3);
    }

    #[test]
    fn test_memory_op_sorting() {
        let bound = ValueBound::from_constant(0);

        let mut ops = vec![
            MemoryOp::write(0x2000, 0, 200, bound, 4),
            MemoryOp::read(0x1000, 0, 100, bound, 4),
            MemoryOp::write(0x1000, 0, 100, bound, 4),
            MemoryOp::read(0x3000, 0, 150, bound, 4),
        ];

        ops.sort();

        // Expected order: timestamp 100 (read before write), 150, 200
        assert_eq!(ops[0].timestamp, 100);
        assert!(ops[0].is_read());
        assert_eq!(ops[1].timestamp, 100);
        assert!(ops[1].is_write());
        assert_eq!(ops[2].timestamp, 150);
        assert_eq!(ops[3].timestamp, 200);
    }

    #[test]
    fn test_memory_op_width() {
        let bound = ValueBound::from_constant(0);

        let byte_op = MemoryOp::read(0x1000, 0xFF, 100, bound, 1);
        assert_eq!(byte_op.width, 1);

        let halfword_op = MemoryOp::read(0x1000, 0xFFFF, 100, bound, 2);
        assert_eq!(halfword_op.width, 2);

        let word_op = MemoryOp::read(0x1000, 0xFFFFFFFF, 100, bound, 4);
        assert_eq!(word_op.width, 4);

        let dword_op = MemoryOp::read(0x1000, 0xFFFFFFFFFFFF, 100, bound, 8);
        assert_eq!(dword_op.width, 8);
    }

    #[test]
    fn test_sha256_witness_creation() {
        let witness = Sha256Witness::new(100);

        assert_eq!(witness.timestamp, 100);
        assert_eq!(witness.num_rounds(), 0);
        assert_eq!(witness.message_block, [0; 16]);
        assert_eq!(witness.initial_state, [0; 8]);
        assert_eq!(witness.final_state, [0; 8]);
    }

    #[test]
    fn test_sha256_witness_record_round() {
        let mut witness = Sha256Witness::new(0);

        let state = [1, 2, 3, 4, 5, 6, 7, 8];
        witness.record_round(0, state);

        assert_eq!(witness.num_rounds(), 1);
        assert_eq!(witness.round_states[0], state);

        // Record another round
        let state2 = [9, 10, 11, 12, 13, 14, 15, 16];
        witness.record_round(1, state2);

        assert_eq!(witness.num_rounds(), 2);
        assert_eq!(witness.round_states[1], state2);
    }

    #[test]
    fn test_crypto_witness_timestamp() {
        let sha_witness = Sha256Witness::new(42);
        let crypto_witness = CryptoWitness::Sha256(sha_witness);

        assert_eq!(crypto_witness.timestamp(), 42);
        assert_eq!(crypto_witness.crypto_type(), CryptoType::Sha256);
    }

    #[test]
    fn test_poseidon2_witness_creation() {
        let witness = Poseidon2Witness {
            input_state: vec![1, 2, 3],
            round_states: vec![],
            output_state: vec![4, 5, 6],
            timestamp: 50,
        };

        assert_eq!(witness.timestamp, 50);
        assert_eq!(witness.input_state.len(), 3);
        assert_eq!(witness.output_state.len(), 3);
    }

    #[test]
    fn test_keccak256_witness_creation() {
        let witness = Keccak256Witness {
            input_state: [[0; 5]; 5],
            round_states: vec![],
            output_state: [[0; 5]; 5],
            timestamp: 75,
        };

        assert_eq!(witness.timestamp, 75);
    }

    #[test]
    fn test_crypto_witness_variants() {
        let sha = CryptoWitness::Sha256(Sha256Witness::new(10));
        assert_eq!(sha.timestamp(), 10);
        assert_eq!(sha.crypto_type(), CryptoType::Sha256);

        let poseidon = CryptoWitness::Poseidon2(Poseidon2Witness {
            input_state: vec![],
            round_states: vec![],
            output_state: vec![],
            timestamp: 20,
        });
        assert_eq!(poseidon.timestamp(), 20);
        assert_eq!(poseidon.crypto_type(), CryptoType::Poseidon2);

        let keccak = CryptoWitness::Keccak256(Keccak256Witness {
            input_state: [[0; 5]; 5],
            round_states: vec![],
            output_state: [[0; 5]; 5],
            timestamp: 30,
        });
        assert_eq!(keccak.timestamp(), 30);
        assert_eq!(keccak.crypto_type(), CryptoType::Keccak256);
    }
}
