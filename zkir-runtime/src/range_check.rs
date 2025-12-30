//! Range Checking System for ZKIR v3.4
//!
//! This module implements deferred range checking with chunk decomposition:
//! - 20-bit limbs decompose into two 10-bit chunks
//! - Lookup table contains 1024 entries (0..1023)
//! - Values are tracked with bounds and checked at checkpoints
//! - 8-bit headroom allows ~256 deferred additions before overflow

use crate::error::{RuntimeError, Result};
use zkir_spec::{Config, Value, Value40, ValueBound};

/// Lookup table for range checking
///
/// For default 20-bit limbs:
/// - chunk_bits = limb_bits / 2 = 10 bits
/// - table_size = 2^10 = 1024 entries
/// - Each entry is a valid 10-bit value (0..1023)
#[derive(Debug, Clone)]
pub struct RangeLookupTable {
    /// Valid chunk values (e.g., 0..1024 for 10-bit chunks)
    table: Vec<u16>,
    /// Bits per chunk
    chunk_bits: u32,
}

impl RangeLookupTable {
    /// Create a new lookup table for the given configuration
    pub fn new(config: &Config) -> Self {
        let chunk_bits = config.limb_bits / 2;
        let table_size = 1 << chunk_bits;

        // Populate table with all valid chunk values
        let table: Vec<u16> = (0..table_size).map(|i| i as u16).collect();

        Self {
            table,
            chunk_bits: chunk_bits as u32
        }
    }

    /// Check if a chunk value is in the valid range
    pub fn is_valid_chunk(&self, chunk: u16) -> bool {
        (chunk as usize) < self.table.len()
    }

    /// Get the chunk size in bits
    pub fn chunk_bits(&self) -> u32 {
        self.chunk_bits
    }

    /// Get the number of chunks needed for a limb
    pub fn chunks_per_limb(&self) -> usize {
        2  // limb_bits = chunk_bits * 2
    }
}

/// A value pending range check
#[derive(Debug, Clone)]
pub struct PendingCheck {
    /// The value to check
    pub value: Value40,
    /// The bound on this value
    pub bound: ValueBound,
    /// Program counter where this value was created (for debugging)
    pub pc: u64,
}

/// Range check tracker with deferred checking
///
/// Accumulates values with bounds and performs range checks at checkpoints.
/// Uses headroom to determine when checks are required.
#[derive(Debug)]
pub struct RangeCheckTracker {
    /// Configuration (determines chunk size, headroom, etc.)
    config: Config,

    /// Lookup table for valid chunks
    table: RangeLookupTable,

    /// Pending checks (values waiting for checkpoint)
    pending: Vec<PendingCheck>,

    /// Total checkpoints performed
    checkpoint_count: u64,
}

impl RangeCheckTracker {
    /// Create a new range check tracker
    pub fn new(config: Config) -> Self {
        let table = RangeLookupTable::new(&config);

        Self {
            config,
            table,
            pending: Vec::new(),
            checkpoint_count: 0,
        }
    }

    /// Check if a value needs range checking
    ///
    /// A value needs checking if its bound exceeds the data width.
    /// For 40-bit values: needs check if bound > 40 bits
    pub fn needs_check(&self, bound: &ValueBound) -> bool {
        bound.max_bits > self.config.data_bits()
    }

    /// Defer a range check for later
    ///
    /// Adds the value to the pending list. Will be checked at next checkpoint.
    pub fn defer(&mut self, value: Value40, bound: ValueBound, pc: u64) {
        if self.needs_check(&bound) {
            self.pending.push(PendingCheck { value, bound, pc });
        }
    }

    /// Check if we should insert a checkpoint
    ///
    /// Returns true if:
    /// - We have pending checks, AND
    /// - Either we have many pending checks OR headroom is getting low
    pub fn should_checkpoint(&self) -> bool {
        if self.pending.is_empty() {
            return false;
        }

        // Checkpoint if we have many pending checks (amortize overhead)
        if self.pending.len() >= 16 {
            return true;
        }

        // Checkpoint if any value is close to overflow
        // (This is conservative; in practice we have 8-bit headroom)
        self.pending.iter().any(|p| p.bound.max_bits >= self.config.data_bits() + 4)
    }

    /// Perform a checkpoint: verify all pending range checks
    ///
    /// Returns a witness of all chunk decompositions performed.
    pub fn checkpoint(&mut self) -> Result<RangeCheckWitness> {
        let mut witness = RangeCheckWitness::new();

        // Collect pending checks (to avoid borrowing issues)
        let pending_checks: Vec<_> = self.pending.drain(..).collect();

        for pending in pending_checks {
            // Decompose the value into chunks
            let chunks = self.decompose_value(&pending.value)?;

            // Verify each chunk is in the lookup table
            for chunk in &chunks {
                if !self.table.is_valid_chunk(*chunk) {
                    return Err(RuntimeError::Other(format!(
                        "Range check failed at PC {:#x}: chunk {} out of range (max {})",
                        pending.pc,
                        chunk,
                        (1 << self.table.chunk_bits()) - 1
                    )));
                }
            }

            // Add to witness
            witness.add_check(pending.value, chunks, pending.pc);
        }

        self.checkpoint_count += 1;
        Ok(witness)
    }

    /// Decompose a Value40 into chunks
    ///
    /// For default 20-bit × 2 limbs with 10-bit chunks:
    /// - Limb 0 (bits 0-19) → chunks[0] (bits 0-9), chunks[1] (bits 10-19)
    /// - Limb 1 (bits 20-39) → chunks[2] (bits 20-29), chunks[3] (bits 30-39)
    fn decompose_value(&self, value: &Value40) -> Result<Vec<u16>> {
        let limbs = value.limbs();
        let mut chunks = Vec::new();

        let chunk_mask = (1u32 << self.table.chunk_bits()) - 1;

        for limb in limbs {
            // Low chunk: bits 0..chunk_bits
            let low = (limb & chunk_mask) as u16;
            chunks.push(low);

            // High chunk: bits chunk_bits..(2*chunk_bits)
            let high = ((limb >> self.table.chunk_bits()) & chunk_mask) as u16;
            chunks.push(high);
        }

        Ok(chunks)
    }

    /// Get the number of pending checks
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get the total number of checkpoints performed
    pub fn checkpoint_count(&self) -> u64 {
        self.checkpoint_count
    }
}

/// Witness data for a range check checkpoint
///
/// Contains all chunk decompositions that were verified.
/// In the prover, this becomes part of the execution trace.
#[derive(Debug, Clone)]
pub struct RangeCheckWitness {
    /// Each entry is (value, chunks, pc)
    checks: Vec<(Value40, Vec<u16>, u64)>,
}

impl RangeCheckWitness {
    fn new() -> Self {
        Self { checks: Vec::new() }
    }

    fn add_check(&mut self, value: Value40, chunks: Vec<u16>, pc: u64) {
        self.checks.push((value, chunks, pc));
    }

    /// Get the number of range checks in this witness
    pub fn len(&self) -> usize {
        self.checks.len()
    }

    /// Check if witness is empty
    pub fn is_empty(&self) -> bool {
        self.checks.is_empty()
    }

    /// Get all checks
    pub fn checks(&self) -> &[(Value40, Vec<u16>, u64)] {
        &self.checks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_table_creation() {
        let config = Config::default();
        let table = RangeLookupTable::new(&config);

        // Default: 20-bit limbs → 10-bit chunks → 1024 entries
        assert_eq!(table.chunk_bits(), 10);
        assert_eq!(table.table.len(), 1024);
        assert_eq!(table.chunks_per_limb(), 2);
    }

    #[test]
    fn test_lookup_table_validation() {
        let config = Config::default();
        let table = RangeLookupTable::new(&config);

        // Valid chunks: 0..1023
        assert!(table.is_valid_chunk(0));
        assert!(table.is_valid_chunk(512));
        assert!(table.is_valid_chunk(1023));

        // Invalid chunks: >= 1024
        assert!(!table.is_valid_chunk(1024));
        assert!(!table.is_valid_chunk(2048));
    }

    #[test]
    fn test_chunk_decomposition() {
        let config = Config::default();
        let tracker = RangeCheckTracker::new(config);

        // Test value: 0x12345 in limb 0, 0xABCDE in limb 1
        // Limb 0: 0x12345 (already fits in 20 bits)
        //   Low 10 bits:  0x345 = 837
        //   High 10 bits: 0x048 = 72
        // Limb 1: 0xABCDE (masked to 20 bits = 0xABCDE & 0xFFFFF = 0xABCDE, already fits)
        //   Low 10 bits:  0x0DE = 222
        //   High 10 bits: 0x2AF = 687
        let value = Value40::from_limbs(&[0x12345, 0xABCDE]);
        let chunks = tracker.decompose_value(&value).unwrap();

        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0], 0x345);  // Limb 0 low
        assert_eq!(chunks[1], 0x048);  // Limb 0 high
        assert_eq!(chunks[2], 0x0DE);  // Limb 1 low
        assert_eq!(chunks[3], 0x2AF);  // Limb 1 high
    }

    #[test]
    fn test_needs_check() {
        let config = Config::default();
        let tracker = RangeCheckTracker::new(config);

        // 40-bit value: doesn't need check
        let bound_40 = ValueBound::from_type_width(40);
        assert!(!tracker.needs_check(&bound_40));

        // 41-bit value: needs check
        let bound_41 = ValueBound::from_type_width(41);
        assert!(tracker.needs_check(&bound_41));

        // 48-bit value: needs check (headroom exceeded)
        let bound_48 = ValueBound::from_type_width(48);
        assert!(tracker.needs_check(&bound_48));
    }

    #[test]
    fn test_defer_and_checkpoint() {
        let config = Config::default();
        let mut tracker = RangeCheckTracker::new(config);

        // Create a value that needs checking (48-bit bound)
        let value = Value40::from_u64(12345);
        let bound = ValueBound::from_type_width(48);

        // Defer the check
        tracker.defer(value, bound, 0x1000);
        assert_eq!(tracker.pending_count(), 1);

        // Perform checkpoint
        let witness = tracker.checkpoint().unwrap();
        assert_eq!(witness.len(), 1);
        assert_eq!(tracker.pending_count(), 0);
        assert_eq!(tracker.checkpoint_count(), 1);
    }

    #[test]
    fn test_checkpoint_multiple_values() {
        let config = Config::default();
        let mut tracker = RangeCheckTracker::new(config);

        // Add multiple values
        for i in 0..5 {
            let value = Value40::from_u64(i * 1000);
            let bound = ValueBound::from_type_width(48);
            tracker.defer(value, bound, 0x1000 + i * 4);
        }

        assert_eq!(tracker.pending_count(), 5);

        // Checkpoint clears all
        let witness = tracker.checkpoint().unwrap();
        assert_eq!(witness.len(), 5);
        assert_eq!(tracker.pending_count(), 0);
    }

    #[test]
    fn test_valid_value_checkpoint() {
        let config = Config::default();
        let mut tracker = RangeCheckTracker::new(config);

        // Valid 40-bit value: all chunks should be in range
        let value = Value40::from_u64((1u64 << 40) - 1);  // Max 40-bit value
        let bound = ValueBound::from_type_width(48);

        tracker.defer(value, bound, 0x2000);
        let witness = tracker.checkpoint().unwrap();

        // Should succeed - all chunks are valid
        assert_eq!(witness.len(), 1);

        // Verify chunks are all < 1024
        let chunks = &witness.checks()[0].1;
        for chunk in chunks {
            assert!(*chunk < 1024);
        }
    }

    #[test]
    fn test_should_checkpoint() {
        let config = Config::default();
        let mut tracker = RangeCheckTracker::new(config);

        // No pending checks: no checkpoint needed
        assert!(!tracker.should_checkpoint());

        // Add a few checks: not yet
        for i in 0..10 {
            tracker.defer(Value40::from_u64(i), ValueBound::from_type_width(42), 0x1000);
        }
        assert!(!tracker.should_checkpoint());

        // Add more checks: should checkpoint (>= 16)
        for i in 10..20 {
            tracker.defer(Value40::from_u64(i), ValueBound::from_type_width(42), 0x1000);
        }
        assert!(tracker.should_checkpoint());
    }
}
