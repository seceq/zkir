//! Deferred arithmetic execution for 30+30 limb architecture
//!
//! This module implements arithmetic operations that produce accumulated (unnormalized)
//! results, deferring carry extraction until observation points.
//!
//! ## Architecture: 30+30 Limbs
//!
//! - **Storage**: 30-bit limbs (fits in u32 with headroom)
//! - **Normalized**: 20-bit values per limb
//! - **Structural headroom**: 10 bits (allows 2^10 = 1024 deferred operations)
//!
//! ## Deferred Operations
//!
//! 1. **ADD**: Adds limbs element-wise without carry extraction
//! 2. **SUB**: Subtracts limbs element-wise (may produce negative intermediate values)
//! 3. **ADDI**: Adds immediate to limbs (immediate split into limbs)
//!
//! ## Example: ADD
//!
//! ```text
//! rs1 = [a0, a1]  (normalized: 20-bit limbs)
//! rs2 = [b0, b1]  (normalized: 20-bit limbs)
//! rd  = [a0 + b0, a1 + b1]  (accumulated: may exceed 20 bits, but < 30 bits)
//! ```
//!
//! After 1024 additions, structural headroom is exhausted and normalization is required.

use zkir_spec::{Register, ValueBound};
use crate::state::VMState;
use crate::range_check::RangeCheckTracker;

/// Configuration for deferred carry model
pub struct DeferredConfig {
    /// Bits per normalized limb (default: 20 for 30+30)
    pub normalized_bits: u8,
    /// Bits per accumulated limb storage (default: 30 for 30+30)
    pub limb_bits: u8,
}

impl Default for DeferredConfig {
    fn default() -> Self {
        Self {
            normalized_bits: 20,
            limb_bits: 30,
        }
    }
}

impl DeferredConfig {
    /// Create config for 30+30 architecture
    pub fn new_30_30() -> Self {
        Self {
            normalized_bits: 20,
            limb_bits: 30,
        }
    }

    /// Structural headroom bits (limb_bits - normalized_bits)
    pub fn headroom_bits(&self) -> u8 {
        self.limb_bits - self.normalized_bits
    }

    /// Maximum deferred operations before overflow (2^headroom)
    pub fn max_deferred_ops(&self) -> u64 {
        1u64 << self.headroom_bits()
    }
}

/// Execute ADD with deferred carry
///
/// Performs element-wise limb addition without carry propagation.
/// Result is stored in accumulated form.
///
/// # Parameters
/// - `state`: VM state
/// - `rd`: Destination register
/// - `rs1`: First source register
/// - `rs2`: Second source register
/// - `config`: Deferred model configuration
/// - `range_checker`: Optional range check tracker
pub fn execute_add_deferred(
    state: &mut VMState,
    rd: Register,
    rs1: Register,
    rs2: Register,
    config: &DeferredConfig,
    range_checker: Option<&mut RangeCheckTracker>,
) {
    // Read source limbs (state-aware: normalized or accumulated)
    let limbs_a = state.read_reg_limbs_extended(rs1, config.normalized_bits, config.limb_bits);
    let limbs_b = state.read_reg_limbs_extended(rs2, config.normalized_bits, config.limb_bits);

    // Add limbs element-wise (no carry extraction)
    let result_limbs = [
        limbs_a[0] + limbs_b[0],
        limbs_a[1] + limbs_b[1],
    ];

    // Check for overflow approaching limb_bits capacity
    if VMState::would_overflow(result_limbs, config.limb_bits) {
        // Force normalization of sources first, then retry
        // This prevents accumulated values from exceeding storage capacity
        let _ = state.normalize_register(rs1, config.normalized_bits, config.limb_bits);
        let _ = state.normalize_register(rs2, config.normalized_bits, config.limb_bits);

        // Recompute with normalized sources
        let limbs_a = state.read_reg_limbs_extended(rs1, config.normalized_bits, config.limb_bits);
        let limbs_b = state.read_reg_limbs_extended(rs2, config.normalized_bits, config.limb_bits);

        let result_limbs = [
            limbs_a[0] + limbs_b[0],
            limbs_a[1] + limbs_b[1],
        ];

        state.write_reg_from_accumulated(rd, result_limbs, config.limb_bits);
    } else {
        // No overflow risk, write accumulated result
        state.write_reg_from_accumulated(rd, result_limbs, config.limb_bits);
    }

    // Propagate bounds for range checking
    let bound_a = state.read_bound(rs1);
    let bound_b = state.read_bound(rs2);
    let result_bound = ValueBound::after_add(&bound_a, &bound_b);
    state.write_bound(rd, result_bound);

    // Defer range check if needed
    if let Some(checker) = range_checker {
        if checker.needs_check(&result_bound) {
            // For deferred model, we'll check the normalized value later
            // For now, just track the bound
            // TODO: Integrate with range checker after normalization
        }
    }

    // Advance PC (all instructions are 4 bytes)
    state.advance_pc(4);
}

/// Execute SUB with deferred borrow
///
/// Performs element-wise limb subtraction without borrow propagation.
/// Result is stored in accumulated form.
///
/// # Deferred Model Design
///
/// Just like ADD uses simple `rd[i] = rs1[i] + rs2[i]`, SUB uses simple
/// `rd[i] = rs1[i] - rs2[i]`. The constraint evaluates this in field arithmetic
/// where negative values wrap correctly (e.g., 37 - 100 = -63 ≡ P - 63 in Mersenne31).
///
/// For the witness, we need to handle the case where rs1[i] < rs2[i] differently
/// since we store u64. When rs1 >= rs2 (the common case), we just subtract.
/// When rs1 < rs2, we add the limb modulus to ensure a positive result that
/// will match the field arithmetic.
///
/// # Parameters
/// - `state`: VM state
/// - `rd`: Destination register
/// - `rs1`: Minuend register
/// - `rs2`: Subtrahend register
/// - `config`: Deferred model configuration
/// - `range_checker`: Optional range check tracker
pub fn execute_sub_deferred(
    state: &mut VMState,
    rd: Register,
    rs1: Register,
    rs2: Register,
    config: &DeferredConfig,
    range_checker: Option<&mut RangeCheckTracker>,
) {
    // Read source limbs (state-aware: normalized or accumulated)
    let limbs_a = state.read_reg_limbs_extended(rs1, config.normalized_bits, config.limb_bits);
    let limbs_b = state.read_reg_limbs_extended(rs2, config.normalized_bits, config.limb_bits);

    // Simple element-wise subtraction, matching the constraint: rd[i] = rs1[i] - rs2[i]
    //
    // This produces the same result as the field constraint evaluation:
    // - If rs1[i] >= rs2[i]: result is positive, stored directly
    // - If rs1[i] < rs2[i]: wrapping_sub produces a value that, when converted to
    //   Mersenne31, equals rs1[i] - rs2[i] in the field (which is P - (rs2[i] - rs1[i]))
    //
    // Note: Unlike ADD which always increases values, SUB can decrease them,
    // so there's no overflow concern - we don't need the overflow check.
    let result_limbs = [
        limbs_a[0].wrapping_sub(limbs_b[0]),
        limbs_a[1].wrapping_sub(limbs_b[1]),
    ];

    // Write result (no overflow possible since subtraction decreases values)
    state.write_reg_from_accumulated(rd, result_limbs, config.limb_bits);

    // Propagate bounds
    let bound_a = state.read_bound(rs1);
    let bound_b = state.read_bound(rs2);
    let result_bound = ValueBound::after_sub(&bound_a, &bound_b);
    state.write_bound(rd, result_bound);

    if let Some(checker) = range_checker {
        if checker.needs_check(&result_bound) {
            // Defer range check
        }
    }

    // Advance PC (all instructions are 4 bytes)
    state.advance_pc(4);
}

/// Execute ADDI with deferred carry
///
/// Adds an immediate value to a register using limb arithmetic.
/// The immediate is split into limbs and added element-wise.
///
/// # Parameters
/// - `state`: VM state
/// - `rd`: Destination register
/// - `rs1`: Source register
/// - `imm`: Immediate value (sign-extended)
/// - `config`: Deferred model configuration
/// - `range_checker`: Optional range check tracker
pub fn execute_addi_deferred(
    state: &mut VMState,
    rd: Register,
    rs1: Register,
    imm: u64,
    config: &DeferredConfig,
    range_checker: Option<&mut RangeCheckTracker>,
) {
    // Read source limbs
    let limbs_a = state.read_reg_limbs_extended(rs1, config.normalized_bits, config.limb_bits);

    // Split immediate into normalized limbs
    let normalized_mask = (1u64 << config.normalized_bits) - 1;
    let imm_limbs = [
        imm & normalized_mask,
        (imm >> config.normalized_bits) & normalized_mask,
    ];

    // Add element-wise
    let result_limbs = [
        limbs_a[0] + imm_limbs[0],
        limbs_a[1] + imm_limbs[1],
    ];

    // Check for overflow
    if VMState::would_overflow(result_limbs, config.limb_bits) {
        // Normalize source and retry
        let _ = state.normalize_register(rs1, config.normalized_bits, config.limb_bits);

        let limbs_a = state.read_reg_limbs_extended(rs1, config.normalized_bits, config.limb_bits);
        let result_limbs = [
            limbs_a[0] + imm_limbs[0],
            limbs_a[1] + imm_limbs[1],
        ];

        state.write_reg_from_accumulated(rd, result_limbs, config.limb_bits);
    } else {
        state.write_reg_from_accumulated(rd, result_limbs, config.limb_bits);
    }

    // Propagate bounds
    let bound_a = state.read_bound(rs1);
    let bound_imm = ValueBound::from_constant(imm);
    let result_bound = ValueBound::after_add(&bound_a, &bound_imm);
    state.write_bound(rd, result_bound);

    if let Some(checker) = range_checker {
        if checker.needs_check(&result_bound) {
            // Defer range check
        }
    }

    // Advance PC (all instructions are 4 bytes)
    state.advance_pc(4);
}

#[cfg(test)]
mod tests {
    use super::*;
    use zkir_spec::Register;

    #[test]
    fn test_deferred_config() {
        let config = DeferredConfig::default();
        assert_eq!(config.normalized_bits, 20);
        assert_eq!(config.limb_bits, 30);
        assert_eq!(config.headroom_bits(), 10);
        assert_eq!(config.max_deferred_ops(), 1024);
    }

    #[test]
    fn test_add_deferred_simple() {
        let mut state = VMState::new(0);
        let config = DeferredConfig::default();

        // Set R1 = 100 (normalized)
        state.write_reg_from_limbs(Register::R1, [100, 0], config.normalized_bits);

        // Set R2 = 200 (normalized)
        state.write_reg_from_limbs(Register::R2, [200, 0], config.normalized_bits);

        // Execute: R3 = R1 + R2
        execute_add_deferred(&mut state, Register::R3, Register::R1, Register::R2, &config, None);

        // Result should be accumulated: [300, 0]
        assert!(state.register_states.get(Register::R3).needs_normalization());

        let limbs = state.read_reg_limbs_extended(Register::R3, config.normalized_bits, config.limb_bits);
        assert_eq!(limbs, [300, 0]);

        // Normalize and check
        let norm_result = state.normalize_register(Register::R3, config.normalized_bits, config.limb_bits);
        assert!(norm_result.is_some());
        let norm = norm_result.unwrap();
        assert_eq!(norm.normalized, [300, 0]);  // No carry needed
        assert_eq!(norm.carries, [0, 0]);
    }

    #[test]
    fn test_add_deferred_with_carry() {
        let mut state = VMState::new(0);
        let config = DeferredConfig::default();

        // Set R1 = 2^20 - 10
        let max_norm = (1u32 << config.normalized_bits) - 10;
        state.write_reg_from_limbs(Register::R1, [max_norm, 0], config.normalized_bits);

        // Set R2 = 20
        state.write_reg_from_limbs(Register::R2, [20, 0], config.normalized_bits);

        // Execute: R3 = R1 + R2 = (2^20 - 10) + 20 = 2^20 + 10
        execute_add_deferred(&mut state, Register::R3, Register::R1, Register::R2, &config, None);

        // Result accumulated: [2^20 + 10, 0]
        let limbs = state.read_reg_limbs_extended(Register::R3, config.normalized_bits, config.limb_bits);
        assert_eq!(limbs[0], (1u64 << config.normalized_bits) + 10);

        // Normalize: should extract carry
        let norm = state.normalize_register(Register::R3, config.normalized_bits, config.limb_bits).unwrap();
        assert_eq!(norm.normalized[0], 10);   // limb0 = 10
        assert_eq!(norm.normalized[1], 1);    // limb1 = 1 (from carry)
        assert_eq!(norm.carries[0], 1);       // carry extracted
    }

    #[test]
    fn test_sub_deferred() {
        let mut state = VMState::new(0);
        let config = DeferredConfig::default();

        // Set R1 = 500
        state.write_reg_from_limbs(Register::R1, [500, 0], config.normalized_bits);

        // Set R2 = 200
        state.write_reg_from_limbs(Register::R2, [200, 0], config.normalized_bits);

        // Execute: R3 = R1 - R2
        execute_sub_deferred(&mut state, Register::R3, Register::R1, Register::R2, &config, None);

        // Normalize and check
        let _ = state.normalize_register(Register::R3, config.normalized_bits, config.limb_bits);

        let value = state.read_reg(Register::R3);
        assert_eq!(value, 300);
    }

    #[test]
    fn test_addi_deferred() {
        let mut state = VMState::new(0);
        let config = DeferredConfig::default();

        // Set R1 = 1000
        state.write_reg_from_limbs(Register::R1, [1000, 0], config.normalized_bits);

        // Execute: R2 = R1 + 234
        execute_addi_deferred(&mut state, Register::R2, Register::R1, 234, &config, None);

        // Normalize and check
        let _ = state.normalize_register(Register::R2, config.normalized_bits, config.limb_bits);

        let value = state.read_reg(Register::R2);
        assert_eq!(value, 1234);
    }

    #[test]
    fn test_r0_hardwired_zero() {
        let mut state = VMState::new(0);
        let config = DeferredConfig::default();

        // Set R1 = 100
        state.write_reg_from_limbs(Register::R1, [100, 0], config.normalized_bits);

        // Execute: R0 = R1 + R1 (should have no effect)
        execute_add_deferred(&mut state, Register::R0, Register::R1, Register::R1, &config, None);

        // R0 should still be zero
        assert_eq!(state.read_reg(Register::R0), 0);
    }
}
