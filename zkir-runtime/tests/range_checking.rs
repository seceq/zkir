//! Range checking integration tests for ZKIR v3.4
//!
//! Tests deferred range checking with chunk decomposition.

use zkir_runtime::range_check::{RangeLookupTable, RangeCheckTracker};
use zkir_spec::{Config, Value, Value40, ValueBound};

// ============================================================================
// Lookup Table Tests
// ============================================================================

#[test]
fn test_lookup_table_default_config() {
    let config = Config::default();
    let table = RangeLookupTable::new(&config);

    // Default: 20-bit limbs → 10-bit chunks → 1024 entries
    assert_eq!(table.chunk_bits(), 10);
    assert_eq!(table.chunks_per_limb(), 2);
}

#[test]
fn test_lookup_table_validity() {
    let config = Config::default();
    let table = RangeLookupTable::new(&config);

    // All values 0..1023 should be valid
    for i in 0u16..1024 {
        assert!(table.is_valid_chunk(i), "Chunk {} should be valid", i);
    }
}

#[test]
fn test_lookup_table_invalid_chunks() {
    let config = Config::default();
    let table = RangeLookupTable::new(&config);

    // Values >= 1024 should be invalid
    assert!(!table.is_valid_chunk(1024));
    assert!(!table.is_valid_chunk(1025));
    assert!(!table.is_valid_chunk(2000));
    assert!(!table.is_valid_chunk(u16::MAX));
}

#[test]
fn test_lookup_table_boundary() {
    let config = Config::default();
    let table = RangeLookupTable::new(&config);

    // Test boundary
    assert!(table.is_valid_chunk(1023));
    assert!(!table.is_valid_chunk(1024));
}

// ============================================================================
// Range Check Tracker Tests
// ============================================================================

#[test]
fn test_tracker_creation() {
    let config = Config::default();
    let tracker = RangeCheckTracker::new(config);

    assert_eq!(tracker.pending_count(), 0);
    assert_eq!(tracker.checkpoint_count(), 0);
}

#[test]
fn test_needs_check() {
    let config = Config::default();
    let tracker = RangeCheckTracker::new(config);

    // Values within data width (40 bits) don't need check
    assert!(!tracker.needs_check(&ValueBound::from_type_width(32)));
    assert!(!tracker.needs_check(&ValueBound::from_type_width(40)));

    // Values exceeding data width need check
    assert!(tracker.needs_check(&ValueBound::from_type_width(41)));
    assert!(tracker.needs_check(&ValueBound::from_type_width(48)));
    assert!(tracker.needs_check(&ValueBound::from_type_width(64)));
}

#[test]
fn test_defer_value() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    // Value within bounds - shouldn't be deferred
    let small_bound = ValueBound::from_type_width(32);
    tracker.defer(Value40::from_u64(100), small_bound, 0x1000);
    assert_eq!(tracker.pending_count(), 0);

    // Value exceeding bounds - should be deferred
    let large_bound = ValueBound::from_type_width(48);
    tracker.defer(Value40::from_u64(100), large_bound, 0x1004);
    assert_eq!(tracker.pending_count(), 1);
}

#[test]
fn test_defer_multiple_values() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    let bound = ValueBound::from_type_width(48);

    for i in 0..10 {
        tracker.defer(Value40::from_u64(i * 100), bound.clone(), 0x1000 + i * 4);
    }

    assert_eq!(tracker.pending_count(), 10);
}

// ============================================================================
// Checkpoint Tests
// ============================================================================

#[test]
fn test_checkpoint_empty() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    // Checkpoint with no pending checks
    let witness = tracker.checkpoint().unwrap();
    assert!(witness.is_empty());
    assert_eq!(tracker.checkpoint_count(), 1);
}

#[test]
fn test_checkpoint_clears_pending() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    let bound = ValueBound::from_type_width(48);
    tracker.defer(Value40::from_u64(100), bound.clone(), 0x1000);
    tracker.defer(Value40::from_u64(200), bound.clone(), 0x1004);
    assert_eq!(tracker.pending_count(), 2);

    let witness = tracker.checkpoint().unwrap();
    assert_eq!(witness.len(), 2);
    assert_eq!(tracker.pending_count(), 0);
    assert_eq!(tracker.checkpoint_count(), 1);
}

#[test]
fn test_checkpoint_witness() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    let value = Value40::from_u64(12345);
    let bound = ValueBound::from_type_width(48);
    tracker.defer(value, bound, 0x1000);

    let witness = tracker.checkpoint().unwrap();
    assert_eq!(witness.len(), 1);

    // Verify witness contents
    let checks = witness.checks();
    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].2, 0x1000); // PC
}

#[test]
fn test_multiple_checkpoints() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    let bound = ValueBound::from_type_width(48);

    // First batch
    tracker.defer(Value40::from_u64(100), bound.clone(), 0x1000);
    let witness1 = tracker.checkpoint().unwrap();
    assert_eq!(witness1.len(), 1);

    // Second batch
    tracker.defer(Value40::from_u64(200), bound.clone(), 0x1004);
    tracker.defer(Value40::from_u64(300), bound.clone(), 0x1008);
    let witness2 = tracker.checkpoint().unwrap();
    assert_eq!(witness2.len(), 2);

    assert_eq!(tracker.checkpoint_count(), 2);
}

// ============================================================================
// Should Checkpoint Tests
// ============================================================================

#[test]
fn test_should_checkpoint_empty() {
    let config = Config::default();
    let tracker = RangeCheckTracker::new(config);

    // No pending checks - no checkpoint needed
    assert!(!tracker.should_checkpoint());
}

#[test]
fn test_should_checkpoint_few_checks() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    let bound = ValueBound::from_type_width(42);

    // Add a few checks - not enough to trigger checkpoint
    for i in 0..5 {
        tracker.defer(Value40::from_u64(i), bound.clone(), 0x1000);
    }

    // Less than 16 and bound is not too high
    assert!(!tracker.should_checkpoint());
}

#[test]
fn test_should_checkpoint_many_checks() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    let bound = ValueBound::from_type_width(42);

    // Add many checks - should trigger checkpoint
    for i in 0..20 {
        tracker.defer(Value40::from_u64(i), bound.clone(), 0x1000);
    }

    // 20 >= 16, should checkpoint
    assert!(tracker.should_checkpoint());
}

#[test]
fn test_should_checkpoint_high_bound() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    // High bound (close to overflow)
    let high_bound = ValueBound::from_type_width(45); // 40 + 5 >= 40 + 4

    tracker.defer(Value40::from_u64(100), high_bound, 0x1000);

    // Should checkpoint due to high bound
    assert!(tracker.should_checkpoint());
}

// ============================================================================
// Chunk Decomposition Tests
// ============================================================================

#[test]
fn test_decomposition_simple_value() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    // Simple value that fits in first limb
    let value = Value40::from_u64(0x12345);
    let bound = ValueBound::from_type_width(48);
    tracker.defer(value, bound, 0x1000);

    let witness = tracker.checkpoint().unwrap();
    let chunks = &witness.checks()[0].1;

    // 4 chunks total (2 per limb, 2 limbs)
    assert_eq!(chunks.len(), 4);

    // All chunks should be valid (< 1024)
    for &chunk in chunks {
        assert!(chunk < 1024);
    }
}

#[test]
fn test_decomposition_max_value() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    // Maximum 40-bit value
    let value = Value40::from_u64((1u64 << 40) - 1);
    let bound = ValueBound::from_type_width(48);
    tracker.defer(value, bound, 0x1000);

    let witness = tracker.checkpoint().unwrap();
    let chunks = &witness.checks()[0].1;

    // All chunks should be 0x3FF (1023) for max value
    for &chunk in chunks {
        assert_eq!(chunk, 1023);
    }
}

#[test]
fn test_decomposition_zero() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    // Zero value
    let value = Value40::from_u64(0);
    let bound = ValueBound::from_type_width(48);
    tracker.defer(value, bound, 0x1000);

    let witness = tracker.checkpoint().unwrap();
    let chunks = &witness.checks()[0].1;

    // All chunks should be 0
    for &chunk in chunks {
        assert_eq!(chunk, 0);
    }
}

#[test]
fn test_decomposition_preserves_value() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    // Test that decomposition can reconstruct original value
    let original = 0xABCDE12345u64;
    let value = Value40::from_u64(original);
    let bound = ValueBound::from_type_width(48);
    tracker.defer(value, bound, 0x1000);

    let witness = tracker.checkpoint().unwrap();
    let chunks = &witness.checks()[0].1;

    // Reconstruct from chunks
    // chunks[0] = limb0 bits 0-9
    // chunks[1] = limb0 bits 10-19
    // chunks[2] = limb1 bits 0-9 (overall bits 20-29)
    // chunks[3] = limb1 bits 10-19 (overall bits 30-39)
    let limb0 = (chunks[0] as u32) | ((chunks[1] as u32) << 10);
    let limb1 = (chunks[2] as u32) | ((chunks[3] as u32) << 10);
    let reconstructed = (limb0 as u64) | ((limb1 as u64) << 20);

    // Mask to 40 bits
    let original_masked = original & ((1u64 << 40) - 1);
    assert_eq!(reconstructed, original_masked);
}

// ============================================================================
// Witness Tests
// ============================================================================

#[test]
fn test_witness_empty() {
    // Create an empty witness through a tracker
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);
    let witness = tracker.checkpoint().unwrap();

    assert!(witness.is_empty());
    assert_eq!(witness.len(), 0);
}

#[test]
fn test_witness_contents() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    let values_and_pcs: [(Value40, u64); 3] = [
        (Value40::from_u64(100), 0x1000u64),
        (Value40::from_u64(200), 0x1004u64),
        (Value40::from_u64(300), 0x1008u64),
    ];

    let bound = ValueBound::from_type_width(48);
    for (value, pc) in values_and_pcs.iter() {
        tracker.defer(*value, bound.clone(), *pc);
    }

    let witness = tracker.checkpoint().unwrap();
    assert_eq!(witness.len(), 3);

    let checks = witness.checks();
    for (i, (_, pc)) in values_and_pcs.iter().enumerate() {
        assert_eq!(checks[i].2, *pc);
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_boundary_values() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    // Values at chunk boundaries
    let boundary_values = [
        0u64,           // All zeros
        0x3FF,          // Max single chunk
        0x3FF_3FF,      // Max two chunks (limb0)
        (1u64 << 20) - 1,  // Max limb0
        1u64 << 20,     // Min limb1
        (1u64 << 40) - 1,  // Max 40-bit
    ];

    let bound = ValueBound::from_type_width(48);
    for value in boundary_values {
        tracker.defer(Value40::from_u64(value), bound.clone(), 0x1000);
    }

    let witness = tracker.checkpoint().unwrap();
    assert_eq!(witness.len(), boundary_values.len());
}

#[test]
fn test_power_of_two_values() {
    let config = Config::default();
    let mut tracker = RangeCheckTracker::new(config);

    let bound = ValueBound::from_type_width(48);

    // Test powers of 2 that fit in 40 bits
    for shift in 0..40 {
        let value = 1u64 << shift;
        tracker.defer(Value40::from_u64(value), bound.clone(), 0x1000 + shift * 4);
    }

    let witness = tracker.checkpoint().unwrap();
    assert_eq!(witness.len(), 40);
}

// ============================================================================
// ValueBound Tests
// ============================================================================

#[test]
fn test_value_bound_from_type_width() {
    let bound8 = ValueBound::from_type_width(8);
    assert_eq!(bound8.max_bits, 8);

    let bound16 = ValueBound::from_type_width(16);
    assert_eq!(bound16.max_bits, 16);

    let bound32 = ValueBound::from_type_width(32);
    assert_eq!(bound32.max_bits, 32);

    let bound64 = ValueBound::from_type_width(64);
    assert_eq!(bound64.max_bits, 64);
}

#[test]
fn test_value_bound_clone() {
    let bound = ValueBound::from_type_width(42);
    let cloned = bound.clone();
    assert_eq!(bound.max_bits, cloned.max_bits);
}
