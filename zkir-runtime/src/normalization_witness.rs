//! Normalization witness for deferred carry model
//!
//! Tracks normalization events that occur during execution for proof generation.
//! These witnesses allow the prover to verify that deferred arithmetic was correctly
//! normalized at observation points.

use zkir_spec::Register;
use crate::normalize::NormalizationResult;

/// Normalization event witness
///
/// Records when a register was normalized during execution, including:
/// - The cycle at which normalization occurred
/// - Which register was normalized
/// - The accumulated limbs before normalization
/// - The normalized limbs after normalization
/// - The carries extracted during normalization
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationWitness {
    /// Cycle at which normalization occurred
    pub cycle: u64,

    /// Program counter at normalization
    pub pc: u64,

    /// Register that was normalized
    pub register: Register,

    /// Accumulated limbs before normalization [limb0, limb1]
    pub accumulated_limbs: [u64; 2],

    /// Normalized limbs after normalization [limb0, limb1]
    pub normalized_limbs: [u32; 2],

    /// Carries extracted during normalization [carry0, carry1]
    pub carries: [u32; 2],

    /// Configuration: bits per normalized limb
    pub normalized_bits: u8,

    /// Configuration: bits per accumulated limb
    pub limb_bits: u8,
}

impl NormalizationWitness {
    /// Create normalization witness from a normalization result
    pub fn new(
        cycle: u64,
        pc: u64,
        register: Register,
        result: &NormalizationResult,
        normalized_bits: u8,
        limb_bits: u8,
    ) -> Self {
        Self {
            cycle,
            pc,
            register,
            accumulated_limbs: result.accumulated,
            normalized_limbs: result.normalized,
            carries: result.carries,
            normalized_bits,
            limb_bits,
        }
    }

    /// Check if normalization extracted any carries
    pub fn has_carries(&self) -> bool {
        self.carries[0] != 0 || self.carries[1] != 0
    }

    /// Get total carry value (for debugging)
    pub fn total_carry(&self) -> u64 {
        self.carries[0] as u64 + ((self.carries[1] as u64) << self.normalized_bits)
    }

    /// Verify the normalization was performed correctly
    ///
    /// Checks that the normalization algorithm was applied correctly:
    /// 1. Carry extraction: carry[i] = accumulated[i] >> normalized_bits
    /// 2. Normalization: norm[i] = accumulated[i] & ((1 << normalized_bits) - 1)
    /// 3. Carry propagation: accumulated[i+1] += carry[i]
    pub fn verify(&self) -> bool {
        let normalized_mask = (1u64 << self.normalized_bits) - 1;

        // Step 1: Extract carry from limb 0
        let expected_carry_0 = (self.accumulated_limbs[0] >> self.normalized_bits) as u32;
        let expected_norm_0 = (self.accumulated_limbs[0] & normalized_mask) as u32;

        if self.carries[0] != expected_carry_0 || self.normalized_limbs[0] != expected_norm_0 {
            return false;
        }

        // Step 2: Propagate carry to limb 1
        let limb1_with_carry = self.accumulated_limbs[1] + self.carries[0] as u64;
        let expected_carry_1 = (limb1_with_carry >> self.normalized_bits) as u32;
        let expected_norm_1 = (limb1_with_carry & normalized_mask) as u32;

        if self.carries[1] != expected_carry_1 || self.normalized_limbs[1] != expected_norm_1 {
            return false;
        }

        true
    }

    /// Get the register index (0-15)
    pub fn register_index(&self) -> u8 {
        self.register as u8
    }
}

/// Normalization event type
///
/// Describes why a normalization occurred
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationCause {
    /// Normalization before an observation point instruction
    ObservationPoint,

    /// Normalization due to overflow (accumulated limbs approaching limit)
    Overflow,

    /// Explicit normalization (forced by implementation)
    Explicit,
}

/// Extended normalization witness with cause
#[derive(Debug, Clone)]
pub struct NormalizationEvent {
    /// The normalization witness
    pub witness: NormalizationWitness,

    /// Why this normalization occurred
    pub cause: NormalizationCause,

    /// Opcode of the instruction that triggered normalization (if observation point)
    pub triggering_opcode: Option<zkir_spec::Opcode>,
}

impl NormalizationEvent {
    /// Create a new normalization event
    pub fn new(
        witness: NormalizationWitness,
        cause: NormalizationCause,
        triggering_opcode: Option<zkir_spec::Opcode>,
    ) -> Self {
        Self {
            witness,
            cause,
            triggering_opcode,
        }
    }

    /// Create an observation point normalization event
    pub fn observation_point(
        cycle: u64,
        pc: u64,
        register: Register,
        result: &NormalizationResult,
        normalized_bits: u8,
        limb_bits: u8,
        opcode: zkir_spec::Opcode,
    ) -> Self {
        let witness = NormalizationWitness::new(cycle, pc, register, result, normalized_bits, limb_bits);
        Self::new(witness, NormalizationCause::ObservationPoint, Some(opcode))
    }

    /// Create an overflow normalization event
    pub fn overflow(
        cycle: u64,
        pc: u64,
        register: Register,
        result: &NormalizationResult,
        normalized_bits: u8,
        limb_bits: u8,
    ) -> Self {
        let witness = NormalizationWitness::new(cycle, pc, register, result, normalized_bits, limb_bits);
        Self::new(witness, NormalizationCause::Overflow, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::NormalizationResult;

    #[test]
    fn test_normalization_witness_creation() {
        let result = NormalizationResult::new([1048676, 5], [100, 6], [1, 0]);

        let witness = NormalizationWitness::new(
            42,                // cycle
            0x1000,            // pc
            Register::R1,      // register
            &result,
            20,                // normalized_bits
            30,                // limb_bits
        );

        assert_eq!(witness.cycle, 42);
        assert_eq!(witness.pc, 0x1000);
        assert_eq!(witness.register, Register::R1);
        assert_eq!(witness.accumulated_limbs, [1048676, 5]);
        assert_eq!(witness.normalized_limbs, [100, 6]);
        assert_eq!(witness.carries, [1, 0]);
        assert!(witness.has_carries());
        assert_eq!(witness.total_carry(), 1);
    }

    #[test]
    fn test_witness_verification() {
        // Valid normalization
        let result = NormalizationResult::new([1048676, 5], [100, 6], [1, 0]);
        let witness = NormalizationWitness::new(0, 0, Register::R1, &result, 20, 30);

        assert!(witness.verify(), "Valid normalization should verify");

        // Invalid normalization (wrong carry)
        let bad_witness = NormalizationWitness {
            cycle: 0,
            pc: 0,
            register: Register::R1,
            accumulated_limbs: [1048676, 5],
            normalized_limbs: [100, 6],
            carries: [2, 0],  // Wrong carry!
            normalized_bits: 20,
            limb_bits: 30,
        };

        assert!(!bad_witness.verify(), "Invalid normalization should not verify");
    }

    #[test]
    fn test_normalization_event_creation() {
        let result = NormalizationResult::new([100, 200], [100, 200], [0, 0]);

        let event = NormalizationEvent::observation_point(
            10,
            0x2000,
            Register::R2,
            &result,
            20,
            30,
            zkir_spec::Opcode::Beq,
        );

        assert_eq!(event.cause, NormalizationCause::ObservationPoint);
        assert_eq!(event.triggering_opcode, Some(zkir_spec::Opcode::Beq));
        assert_eq!(event.witness.cycle, 10);
        assert_eq!(event.witness.register, Register::R2);
    }

    #[test]
    fn test_overflow_event() {
        let result = NormalizationResult::new([2000000, 1500000], [100, 200], [10, 5]);

        let event = NormalizationEvent::overflow(
            15,
            0x3000,
            Register::R3,
            &result,
            20,
            30,
        );

        assert_eq!(event.cause, NormalizationCause::Overflow);
        assert_eq!(event.triggering_opcode, None);
        assert_eq!(event.witness.cycle, 15);
    }

    #[test]
    fn test_register_index() {
        let result = NormalizationResult::new([0, 0], [0, 0], [0, 0]);
        let witness = NormalizationWitness::new(0, 0, Register::R5, &result, 20, 30);

        assert_eq!(witness.register_index(), 5);
    }

    #[test]
    fn test_no_carries() {
        let result = NormalizationResult::new([100, 200], [100, 200], [0, 0]);
        let witness = NormalizationWitness::new(0, 0, Register::R1, &result, 20, 30);

        assert!(!witness.has_carries());
        assert_eq!(witness.total_carry(), 0);
    }

    #[test]
    fn test_carry_propagation_verification() {
        // Test case: 2^20 + 90 = 1048576 + 90 = 1048666
        // limb0 = 1048666 (exceeds 20 bits)
        // After normalization:
        //   carry0 = 1048666 >> 20 = 1
        //   norm0 = 1048666 & ((1<<20)-1) = 90
        //   limb1_with_carry = 0 + 1 = 1
        //   carry1 = 1 >> 20 = 0
        //   norm1 = 1 & ((1<<20)-1) = 1

        let result = NormalizationResult::new([1048666, 0], [90, 1], [1, 0]);
        let witness = NormalizationWitness::new(0, 0, Register::R1, &result, 20, 30);

        assert!(witness.verify(), "Carry propagation should verify correctly");
        assert!(witness.has_carries());
        assert_eq!(witness.total_carry(), 1);
    }
}
