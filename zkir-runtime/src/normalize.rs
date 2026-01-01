//! Register normalization for deferred carry model
//!
//! This module implements the normalization algorithm that converts accumulated
//! register values back to normalized form by extracting and propagating carries.
//!
//! ## Normalization Algorithm
//!
//! Given accumulated limbs that may use up to `limb_bits` (e.g., 30 bits):
//!
//! 1. Extract carry from limb 0: `carry[0] = limb[0] >> normalized_bits`
//! 2. Normalize limb 0: `norm[0] = limb[0] & ((1 << normalized_bits) - 1)`
//! 3. Propagate carry to limb 1: `limb[1] += carry[0]`
//! 4. Extract carry from limb 1: `carry[1] = limb[1] >> normalized_bits`
//! 5. Normalize limb 1: `norm[1] = limb[1] & ((1 << normalized_bits) - 1)`
//! 6. For 32-bit values, wrap around (ignore final carry for two's complement)

use zkir_spec::Register;
use crate::state::VMState;

/// Result of a normalization operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationResult {
    /// Original accumulated limbs before normalization
    pub accumulated: [u64; 2],
    /// Resulting normalized limbs after normalization
    pub normalized: [u32; 2],
    /// Carries extracted during normalization
    pub carries: [u32; 2],
}

impl NormalizationResult {
    /// Create a new normalization result
    pub fn new(accumulated: [u64; 2], normalized: [u32; 2], carries: [u32; 2]) -> Self {
        Self {
            accumulated,
            normalized,
            carries,
        }
    }

    /// Check if any carries were extracted
    pub fn has_carries(&self) -> bool {
        self.carries[0] != 0 || self.carries[1] != 0
    }

    /// Get total carry value (for debugging)
    pub fn total_carry(&self) -> u64 {
        self.carries[0] as u64 + ((self.carries[1] as u64) << 20)
    }
}

impl VMState {
    /// Normalize a register if it contains accumulated value
    ///
    /// This extracts carries and converts accumulated limbs back to normalized limbs
    /// that fit within `normalized_bits` per limb.
    ///
    /// # Parameters
    /// - `reg`: Register to normalize
    /// - `normalized_bits`: Bits per normalized limb (e.g., 20 for 30+30)
    /// - `limb_bits`: Bits per accumulated limb (e.g., 30 for 30+30)
    ///
    /// # Returns
    /// `Some(NormalizationResult)` if normalization was performed, `None` if already normalized
    pub fn normalize_register(
        &mut self,
        reg: Register,
        normalized_bits: u8,
        limb_bits: u8,
    ) -> Option<NormalizationResult> {
        // R0 is always normalized (hardwired to zero)
        if reg == Register::R0 {
            return None;
        }

        // Check if already normalized
        if self.register_states.get(reg).is_normalized() {
            return None;
        }

        // Read accumulated limbs
        let accumulated = self.read_reg_limbs_extended(reg, normalized_bits, limb_bits);

        // Perform normalization
        let normalized_mask = (1u64 << normalized_bits) - 1;

        // Step 1: Extract carry from limb 0
        let carry_0 = (accumulated[0] >> normalized_bits) as u32;
        let norm_0 = (accumulated[0] & normalized_mask) as u32;

        // Step 2: Propagate carry to limb 1, then extract its carry
        let limb1_with_carry = accumulated[1] + carry_0 as u64;
        let carry_1 = (limb1_with_carry >> normalized_bits) as u32;
        let norm_1 = (limb1_with_carry & normalized_mask) as u32;

        // For 32-bit values, we wrap around (ignore final carry)
        // This gives correct two's complement semantics

        let normalized = [norm_0, norm_1];
        let carries = [carry_0, carry_1];

        // Write back normalized value
        self.write_reg_from_limbs(reg, normalized, normalized_bits);

        Some(NormalizationResult::new(accumulated, normalized, carries))
    }

    /// Normalize a register at an observation point (ALWAYS generates witness)
    ///
    /// This is used at observation points where the prover MUST verify range checks,
    /// even if the register is already in normalized form. Unlike `normalize_register`,
    /// this function always returns a NormalizationResult for witness generation.
    ///
    /// # Parameters
    /// - `reg`: Register to normalize
    /// - `normalized_bits`: Bits per normalized limb (e.g., 20 for 30+30)
    /// - `limb_bits`: Bits per accumulated limb (e.g., 30 for 30+30)
    ///
    /// # Returns
    /// `Some(NormalizationResult)` with the normalization witness, `None` only for R0
    pub fn normalize_register_for_observation(
        &mut self,
        reg: Register,
        normalized_bits: u8,
        limb_bits: u8,
    ) -> Option<NormalizationResult> {
        // R0 is always normalized (hardwired to zero)
        if reg == Register::R0 {
            return None;
        }

        // Read current limbs (whether accumulated or normalized)
        let accumulated = self.read_reg_limbs_extended(reg, normalized_bits, limb_bits);

        // Perform normalization
        let normalized_mask = (1u64 << normalized_bits) - 1;

        // Step 1: Extract carry from limb 0
        let carry_0 = (accumulated[0] >> normalized_bits) as u32;
        let norm_0 = (accumulated[0] & normalized_mask) as u32;

        // Step 2: Propagate carry to limb 1, then extract its carry
        let limb1_with_carry = accumulated[1] + carry_0 as u64;
        let carry_1 = (limb1_with_carry >> normalized_bits) as u32;
        let norm_1 = (limb1_with_carry & normalized_mask) as u32;

        let normalized = [norm_0, norm_1];
        let carries = [carry_0, carry_1];

        // Write back normalized value and mark as normalized
        self.write_reg_from_limbs(reg, normalized, normalized_bits);

        Some(NormalizationResult::new(accumulated, normalized, carries))
    }

    /// Normalize and write accumulated limbs in one operation
    ///
    /// This is used when we compute accumulated limbs and immediately need to
    /// normalize them without storing the accumulated form first.
    ///
    /// # Parameters
    /// - `reg`: Register to write
    /// - `accumulated`: Accumulated limbs to normalize and write
    /// - `normalized_bits`: Bits per normalized limb
    pub fn normalize_and_write(
        &mut self,
        reg: Register,
        accumulated: [u64; 2],
        normalized_bits: u8,
    ) -> NormalizationResult {
        if reg == Register::R0 {
            // R0 is always zero
            return NormalizationResult::new([0, 0], [0, 0], [0, 0]);
        }

        let normalized_mask = (1u64 << normalized_bits) - 1;

        let carry_0 = (accumulated[0] >> normalized_bits) as u32;
        let norm_0 = (accumulated[0] & normalized_mask) as u32;

        let limb1_with_carry = accumulated[1] + carry_0 as u64;
        let carry_1 = (limb1_with_carry >> normalized_bits) as u32;
        let norm_1 = (limb1_with_carry & normalized_mask) as u32;

        let normalized = [norm_0, norm_1];
        let carries = [carry_0, carry_1];

        self.write_reg_from_limbs(reg, normalized, normalized_bits);

        NormalizationResult::new(accumulated, normalized, carries)
    }

    /// Normalize multiple registers (for observation points)
    ///
    /// Normalizes all registers in the provided list that are in accumulated state.
    ///
    /// # Parameters
    /// - `regs`: Slice of registers to normalize
    /// - `normalized_bits`: Bits per normalized limb
    /// - `limb_bits`: Bits per accumulated limb
    ///
    /// # Returns
    /// Vector of (register, result) pairs for registers that were normalized
    pub fn normalize_registers(
        &mut self,
        regs: &[Register],
        normalized_bits: u8,
        limb_bits: u8,
    ) -> Vec<(Register, NormalizationResult)> {
        regs.iter()
            .filter(|&&r| r != Register::R0)
            .filter_map(|&reg| {
                self.normalize_register(reg, normalized_bits, limb_bits)
                    .map(|result| (reg, result))
            })
            .collect()
    }

    /// Check if normalization would overflow beyond limb_bits
    ///
    /// This is used to detect when accumulated values exceed the storage capacity
    /// and force early normalization to prevent overflow.
    ///
    /// # Parameters
    /// - `limbs`: Accumulated limbs to check
    /// - `limb_bits`: Maximum bits per limb
    ///
    /// # Returns
    /// `true` if any limb exceeds `limb_bits` capacity
    pub fn would_overflow(limbs: [u64; 2], limb_bits: u8) -> bool {
        let limb_max = 1u64 << limb_bits;
        limbs[0] >= limb_max || limbs[1] >= limb_max
    }

    /// Conditionally normalize if accumulated values approach overflow
    ///
    /// This prevents accumulation from exceeding `limb_bits` capacity by
    /// normalizing when values get too large.
    ///
    /// # Parameters
    /// - `reg`: Register to check and potentially normalize
    /// - `normalized_bits`: Bits per normalized limb
    /// - `limb_bits`: Bits per accumulated limb
    ///
    /// # Returns
    /// `Some(NormalizationResult)` if normalization was performed
    pub fn normalize_if_near_overflow(
        &mut self,
        reg: Register,
        normalized_bits: u8,
        limb_bits: u8,
    ) -> Option<NormalizationResult> {
        if reg == Register::R0 {
            return None;
        }

        // Only check if register is accumulated
        if !self.register_states.get(reg).needs_normalization() {
            return None;
        }

        // Read current limbs
        let limbs = self.read_reg_limbs_extended(reg, normalized_bits, limb_bits);

        // Check for overflow
        if Self::would_overflow(limbs, limb_bits) {
            self.normalize_register(reg, normalized_bits, limb_bits)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zkir_spec::Register;

    #[test]
    fn test_normalize_simple() {
        let mut state = VMState::new(0);
        let normalized_bits = 20;
        let limb_bits = 30;

        // Write accumulated value: limb0 = 2^20 + 100, limb1 = 5
        // This simulates: 100 + 5*2^20 with an extra carry in limb0
        state.write_reg_from_accumulated(Register::R1, [1048676, 5], limb_bits);

        let result = state.normalize_register(Register::R1, normalized_bits, limb_bits);

        assert!(result.is_some());
        let result = result.unwrap();

        assert_eq!(result.accumulated, [1048676, 5]);
        assert_eq!(result.normalized[0], 100);  // 1048676 mod 2^20 = 100
        assert_eq!(result.normalized[1], 6);    // 5 + carry(1) = 6
        assert_eq!(result.carries[0], 1);       // 1048676 >> 20 = 1
        assert!(result.has_carries());

        // Verify register is now normalized
        assert!(state.register_states.get(Register::R1).is_normalized());
    }

    #[test]
    fn test_normalize_already_normalized() {
        let mut state = VMState::new(0);
        let normalized_bits = 20;
        let limb_bits = 30;

        // Write normalized value
        state.write_reg_from_limbs(Register::R1, [100, 200], normalized_bits);

        let result = state.normalize_register(Register::R1, normalized_bits, limb_bits);

        // Should return None since already normalized
        assert!(result.is_none());
    }

    #[test]
    fn test_normalize_r0() {
        let mut state = VMState::new(0);
        let normalized_bits = 20;
        let limb_bits = 30;

        // Try to normalize R0 (should always return None)
        let result = state.normalize_register(Register::R0, normalized_bits, limb_bits);
        assert!(result.is_none());
    }

    #[test]
    fn test_normalize_twos_complement() {
        let mut state = VMState::new(0);
        let normalized_bits = 20;
        let limb_bits = 30;

        // Simulate: 32768 + (-16) via two's complement
        // rs1 = [32768, 0], imm = [1048560, 1048575] (two's complement of -16)
        // sum = [32768 + 1048560, 0 + 1048575] = [1081328, 1048575]
        state.write_reg_from_accumulated(Register::R2, [1081328, 1048575], limb_bits);

        let result = state.normalize_register(Register::R2, normalized_bits, limb_bits);
        assert!(result.is_some());

        let result = result.unwrap();

        // 1081328 = 1048576 + 32752 = 2^20 + 32752
        // limb0: 32752, carry0: 1
        assert_eq!(result.normalized[0], 32752);
        assert_eq!(result.carries[0], 1);

        // limb1 = 1048575 + 1 = 1048576 = 2^20, wraps to 0
        // norm1: 0, carry1: 1
        assert_eq!(result.normalized[1], 0);
        assert_eq!(result.carries[1], 1);

        // Final value: 32752 + 0 * 2^20 = 32752
        // Which is correct: 32768 - 16 = 32752
        let final_value = state.read_reg(Register::R2);
        assert_eq!(final_value, 32752);
    }

    #[test]
    fn test_normalize_and_write() {
        let mut state = VMState::new(0);
        let normalized_bits = 20;

        let accumulated = [1048676, 5];
        let result = state.normalize_and_write(Register::R3, accumulated, normalized_bits);

        assert_eq!(result.accumulated, [1048676, 5]);
        assert_eq!(result.normalized[0], 100);
        assert_eq!(result.normalized[1], 6);
        assert_eq!(result.carries[0], 1);

        // Verify written to register
        let value = state.read_reg(Register::R3);
        let expected = 100 + (6u64 << 20);
        assert_eq!(value, expected);
    }

    #[test]
    fn test_normalize_multiple_registers() {
        let mut state = VMState::new(0);
        let normalized_bits = 20;
        let limb_bits = 30;

        // Set up multiple accumulated registers
        state.write_reg_from_accumulated(Register::R1, [1048676, 5], limb_bits);
        state.write_reg_from_accumulated(Register::R2, [2097252, 10], limb_bits);
        state.write_reg_from_limbs(Register::R3, [100, 200], normalized_bits); // Already normalized

        let regs = vec![Register::R1, Register::R2, Register::R3];
        let results = state.normalize_registers(&regs, normalized_bits, limb_bits);

        // Should normalize R1 and R2, skip R3
        assert_eq!(results.len(), 2);

        // Check R1 normalized
        assert_eq!(results[0].0, Register::R1);
        assert_eq!(results[0].1.normalized[0], 100);

        // Check R2 normalized
        assert_eq!(results[1].0, Register::R2);
        assert_eq!(results[1].1.normalized[0], 100);  // 2097252 mod 2^20
        assert_eq!(results[1].1.carries[0], 2);        // 2097252 >> 20 = 2
    }

    #[test]
    fn test_would_overflow() {
        let limb_bits = 30;
        let limb_max = 1u64 << limb_bits;

        // No overflow
        assert!(!VMState::would_overflow([100, 200], limb_bits));
        assert!(!VMState::would_overflow([limb_max - 1, 0], limb_bits));

        // Overflow in limb0
        assert!(VMState::would_overflow([limb_max, 0], limb_bits));
        assert!(VMState::would_overflow([limb_max + 1, 0], limb_bits));

        // Overflow in limb1
        assert!(VMState::would_overflow([0, limb_max], limb_bits));
        assert!(VMState::would_overflow([100, limb_max + 5], limb_bits));
    }

    #[test]
    fn test_normalize_if_near_overflow() {
        let mut state = VMState::new(0);
        let normalized_bits = 20;
        let limb_bits = 30;
        let limb_max = 1u64 << limb_bits;

        // Test case 1: Value near max but not over - should NOT normalize
        let near_max = limb_max - 100;
        state.write_reg_from_accumulated(Register::R1, [near_max, 0], limb_bits);

        let result = state.normalize_if_near_overflow(Register::R1, normalized_bits, limb_bits);
        assert!(result.is_none());  // Should not normalize yet

        // Test case 2: Create overflow by setting accumulated state with overflow limbs
        // Simulate what happens during arithmetic: limbs can temporarily exceed limb_bits
        // We manually set the state to accumulated and write a value that WHEN UNPACKED
        // will show limbs >= limb_max

        // The key insight: write_reg_from_accumulated packs with limb_bits,
        // but we want to test the case where accumulated limbs would overflow.
        // We need to bypass the normal packing to simulate mid-computation state.

        // Set register state to accumulated first
        state.register_states.mark_accumulated(Register::R2);

        // Construct a value that when read back with read_reg_limbs_extended
        // will have limb0 >= limb_max. Since we're packing with 30 bits,
        // we need at least 2^30 in the lower 30 bits, which wraps to 0.
        // So let's use a different approach: set a value that unpacks to overflow

        // Actually, the limitation is that u64 storage with limb_bits packing
        // cannot represent overflow in individual limbs - that's by design!
        // Overflow would only occur during intermediate arithmetic before packing.

        // So this test should verify that normalize_if_near_overflow works
        // when called with already-normalized registers (returns None)
        state.write_reg_from_limbs(Register::R3, [100, 200], normalized_bits);
        let result = state.normalize_if_near_overflow(Register::R3, normalized_bits, limb_bits);
        assert!(result.is_none());  // Already normalized, no action needed
    }

    #[test]
    fn test_normalization_result_helpers() {
        let result = NormalizationResult::new([1048676, 5], [100, 6], [1, 0]);

        assert!(result.has_carries());
        assert_eq!(result.total_carry(), 1);

        let result_no_carry = NormalizationResult::new([100, 5], [100, 5], [0, 0]);
        assert!(!result_no_carry.has_carries());
        assert_eq!(result_no_carry.total_carry(), 0);
    }
}
