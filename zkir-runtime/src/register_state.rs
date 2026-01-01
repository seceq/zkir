//! Register state tracking for deferred carry model
//!
//! This module provides infrastructure to track whether each register contains
//! a normalized value (fits within `normalized_bits` per limb) or an accumulated
//! value (may use up to `limb_bits` per limb).
//!
//! ## Deferred Carry Model
//!
//! In the 30+30 architecture with deferred carries:
//! - **Normalized**: Each limb fits in `normalized_bits` (20 bits for 30+30)
//! - **Accumulated**: Limbs may use up to `limb_bits` (30 bits for 30+30)
//!
//! Arithmetic operations (ADD, SUB, ADDI) produce accumulated values.
//! Normalization occurs at observation points (branches, stores, bitwise ops, etc.).

use zkir_spec::Register;

/// State of a register's value representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterState {
    /// Value is normalized (each limb fits in normalized_bits)
    ///
    /// Example for 30+30: limb0 < 2^20, limb1 < 2^20
    Normalized,

    /// Value is accumulated (limbs may exceed normalized_bits but fit in limb_bits)
    ///
    /// Example for 30+30: limb0 < 2^30, limb1 < 2^30
    ///
    /// This state occurs after deferred arithmetic operations before normalization.
    Accumulated,
}

impl Default for RegisterState {
    fn default() -> Self {
        RegisterState::Normalized
    }
}

impl RegisterState {
    /// Check if this state requires normalization
    pub fn needs_normalization(&self) -> bool {
        matches!(self, RegisterState::Accumulated)
    }

    /// Check if this state is normalized
    pub fn is_normalized(&self) -> bool {
        matches!(self, RegisterState::Normalized)
    }
}

/// Tracks the state of all 16 registers
///
/// ## Invariants
///
/// - R0 is always Normalized (hardwired to zero)
/// - State changes only occur through explicit set operations
/// - State persists across operations until explicitly changed
#[derive(Debug, Clone)]
pub struct RegisterStateTracker {
    /// State of each register (R0-R15)
    states: [RegisterState; 16],
}

impl RegisterStateTracker {
    /// Create a new tracker with all registers normalized
    pub fn new() -> Self {
        Self {
            states: [RegisterState::Normalized; 16],
        }
    }

    /// Get state of a register
    ///
    /// R0 always returns Normalized regardless of internal state.
    pub fn get(&self, reg: Register) -> RegisterState {
        if reg == Register::R0 {
            // R0 is always normalized (hardwired to zero)
            RegisterState::Normalized
        } else {
            self.states[reg as usize]
        }
    }

    /// Set state of a register
    ///
    /// Setting R0 state has no effect (always Normalized).
    pub fn set(&mut self, reg: Register, state: RegisterState) {
        if reg != Register::R0 {
            self.states[reg as usize] = state;
        }
    }

    /// Check if register needs normalization
    pub fn needs_normalization(&self, reg: Register) -> bool {
        self.get(reg).needs_normalization()
    }

    /// Mark register as normalized
    ///
    /// Convenience method equivalent to `set(reg, RegisterState::Normalized)`.
    pub fn mark_normalized(&mut self, reg: Register) {
        self.set(reg, RegisterState::Normalized);
    }

    /// Mark register as accumulated
    ///
    /// Convenience method equivalent to `set(reg, RegisterState::Accumulated)`.
    pub fn mark_accumulated(&mut self, reg: Register) {
        self.set(reg, RegisterState::Accumulated);
    }

    /// Reset all registers to normalized
    ///
    /// Used when starting fresh execution or resetting VM state.
    pub fn reset(&mut self) {
        self.states = [RegisterState::Normalized; 16];
    }

    /// Get number of registers that need normalization
    ///
    /// Useful for debugging and statistics.
    pub fn count_accumulated(&self) -> usize {
        (1..16)  // Skip R0
            .filter(|&i| self.states[i] == RegisterState::Accumulated)
            .count()
    }

    /// Get all registers that need normalization
    ///
    /// Returns register indices (1-15) that are in Accumulated state.
    pub fn get_accumulated_registers(&self) -> Vec<usize> {
        (1..16)
            .filter(|&i| self.states[i] == RegisterState::Accumulated)
            .collect()
    }

    /// Convert to zkir-spec RegisterState array for trace
    ///
    /// Maps internal RegisterState to zkir_spec::RegisterState for inclusion
    /// in execution traces.
    pub fn to_spec_states(&self) -> [zkir_spec::RegisterState; 16] {
        let mut spec_states = [zkir_spec::RegisterState::Normalized; 16];
        for i in 0..16 {
            spec_states[i] = match self.states[i] {
                RegisterState::Normalized => zkir_spec::RegisterState::Normalized,
                RegisterState::Accumulated => zkir_spec::RegisterState::Accumulated,
            };
        }
        spec_states
    }
}

impl Default for RegisterStateTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let tracker = RegisterStateTracker::new();
        for i in 0..16 {
            let reg = unsafe { std::mem::transmute::<u8, Register>(i) };
            assert_eq!(tracker.get(reg), RegisterState::Normalized);
        }
    }

    #[test]
    fn test_r0_always_normalized() {
        let mut tracker = RegisterStateTracker::new();

        // Try to set R0 to accumulated
        tracker.set(Register::R0, RegisterState::Accumulated);

        // R0 should still be normalized
        assert_eq!(tracker.get(Register::R0), RegisterState::Normalized);
        assert!(!tracker.needs_normalization(Register::R0));
    }

    #[test]
    fn test_state_transitions() {
        let mut tracker = RegisterStateTracker::new();

        // Start normalized
        assert_eq!(tracker.get(Register::R1), RegisterState::Normalized);
        assert!(!tracker.needs_normalization(Register::R1));

        // Mark accumulated
        tracker.mark_accumulated(Register::R1);
        assert_eq!(tracker.get(Register::R1), RegisterState::Accumulated);
        assert!(tracker.needs_normalization(Register::R1));

        // Mark normalized again
        tracker.mark_normalized(Register::R1);
        assert_eq!(tracker.get(Register::R1), RegisterState::Normalized);
        assert!(!tracker.needs_normalization(Register::R1));
    }

    #[test]
    fn test_multiple_registers() {
        let mut tracker = RegisterStateTracker::new();

        // Mark several registers as accumulated
        tracker.mark_accumulated(Register::R1);
        tracker.mark_accumulated(Register::R2);
        tracker.mark_accumulated(Register::R5);

        // Check states
        assert_eq!(tracker.get(Register::R1), RegisterState::Accumulated);
        assert_eq!(tracker.get(Register::R2), RegisterState::Accumulated);
        assert_eq!(tracker.get(Register::R3), RegisterState::Normalized);
        assert_eq!(tracker.get(Register::R5), RegisterState::Accumulated);

        // Count accumulated
        assert_eq!(tracker.count_accumulated(), 3);

        // Get accumulated registers
        let accumulated = tracker.get_accumulated_registers();
        assert_eq!(accumulated, vec![1, 2, 5]);
    }

    #[test]
    fn test_reset() {
        let mut tracker = RegisterStateTracker::new();

        // Mark some registers as accumulated
        tracker.mark_accumulated(Register::R1);
        tracker.mark_accumulated(Register::R2);
        tracker.mark_accumulated(Register::R3);

        assert_eq!(tracker.count_accumulated(), 3);

        // Reset
        tracker.reset();

        // All should be normalized
        assert_eq!(tracker.count_accumulated(), 0);
        for i in 0..16 {
            let reg = unsafe { std::mem::transmute::<u8, Register>(i) };
            assert_eq!(tracker.get(reg), RegisterState::Normalized);
        }
    }

    #[test]
    fn test_state_helper_methods() {
        let state_norm = RegisterState::Normalized;
        let state_accum = RegisterState::Accumulated;

        assert!(state_norm.is_normalized());
        assert!(!state_norm.needs_normalization());

        assert!(!state_accum.is_normalized());
        assert!(state_accum.needs_normalization());
    }
}
