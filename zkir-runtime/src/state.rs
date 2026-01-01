//! VM state management for ZKIR v3.4

use zkir_spec::{Register, ValueBound, NUM_REGISTERS};
use crate::register_state::RegisterStateTracker;

/// Reason for VM halt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HaltReason {
    /// Ebreak instruction executed
    Ebreak,
    /// Exit syscall called
    Exit(u64),
    /// Cycle limit reached
    CycleLimit,
}

/// VM execution state
///
/// The VM state tracks:
/// - 16 general-purpose registers (R0-R15)
/// - Value bounds for each register (for range checking)
/// - Register states (normalized vs accumulated) for deferred carry model
/// - Program counter (PC)
/// - Cycle counter
/// - Halt status
#[derive(Debug, Clone)]
pub struct VMState {
    /// Program counter (byte address)
    pub pc: u64,

    /// General purpose registers (R0-R15)
    /// Values are stored as raw u64, arithmetic operations use Value40
    pub regs: [u64; NUM_REGISTERS],

    /// Value bounds for each register (for range check optimization)
    /// R0 bound is always 0 bits (constant zero)
    pub bounds: [ValueBound; NUM_REGISTERS],

    /// Register state tracker for deferred carry model
    ///
    /// Tracks whether each register contains a normalized or accumulated value.
    /// Used by the deferred carry model (30+30 architecture) to know when
    /// normalization is required.
    pub register_states: RegisterStateTracker,

    /// Cycle counter
    pub cycles: u64,

    /// Halt reason (if halted)
    pub halt_reason: Option<HaltReason>,
}

impl VMState {
    /// Create new VM state with entry point
    pub fn new(entry_point: u64) -> Self {
        // Initialize all bounds to 40-bit program width (default conservative bound)
        let default_bound = ValueBound::from_program_width(40);
        let mut bounds = [default_bound; NUM_REGISTERS];

        // R0 is constant zero, so bound is 0 bits
        bounds[0] = ValueBound::from_constant(0);

        Self {
            pc: entry_point,
            regs: [0; NUM_REGISTERS],
            bounds,
            register_states: RegisterStateTracker::new(),
            cycles: 0,
            halt_reason: None,
        }
    }

    /// Read register value
    ///
    /// R0 is hardwired to zero in ZKIR v3.4
    pub fn read_reg(&self, reg: Register) -> u64 {
        if reg == Register::R0 {
            0  // R0 is always zero
        } else {
            self.regs[reg as usize]
        }
    }

    /// Write register value
    ///
    /// Writing to R0 has no effect
    pub fn write_reg(&mut self, reg: Register, value: u64) {
        if reg != Register::R0 {
            self.regs[reg as usize] = value;
        }
    }

    /// Read bound for a register
    pub fn read_bound(&self, reg: Register) -> ValueBound {
        self.bounds[reg as usize]
    }

    /// Write bound for a register
    ///
    /// Writing bound for R0 has no effect (R0 is always 0 bits)
    pub fn write_bound(&mut self, reg: Register, bound: ValueBound) {
        if reg != Register::R0 {
            self.bounds[reg as usize] = bound;
        }
    }

    /// Write register value and bound together
    ///
    /// This is the preferred method when both value and bound are known.
    pub fn write_reg_with_bound(&mut self, reg: Register, value: u64, bound: ValueBound) {
        self.write_reg(reg, value);
        self.write_bound(reg, bound);
    }

    /// Check if VM is halted
    pub fn is_halted(&self) -> bool {
        self.halt_reason.is_some()
    }

    /// Halt the VM with given reason
    pub fn halt(&mut self, reason: HaltReason) {
        self.halt_reason = Some(reason);
    }

    /// Increment cycle counter
    pub fn inc_cycles(&mut self) {
        self.cycles += 1;
    }

    /// Advance PC by offset
    pub fn advance_pc(&mut self, offset: i64) {
        self.pc = (self.pc as i64 + offset) as u64;
    }

    // ========================================================================
    // Deferred Carry Model Support
    // ========================================================================

    /// Read register as 2-limb representation
    ///
    /// For deferred carry model: splits the 64-bit value into two limbs
    /// based on the config's limb_bits or normalized_bits.
    ///
    /// # Parameters
    /// - `normalized_bits`: Number of bits per normalized limb (e.g., 20 for 30+30)
    ///
    /// # Returns
    /// `[limb0, limb1]` where each limb is extracted using the normalized_bits width
    pub fn read_reg_as_limbs(&self, reg: Register, normalized_bits: u8) -> [u32; 2] {
        let value = self.read_reg(reg);
        let mask = (1u64 << normalized_bits) - 1;
        [
            (value & mask) as u32,
            ((value >> normalized_bits) & mask) as u32,
        ]
    }

    /// Write register from 2-limb representation (normalized)
    ///
    /// Reconstructs value from normalized limbs and marks register as normalized.
    ///
    /// # Parameters
    /// - `limbs`: `[limb0, limb1]` normalized limbs
    /// - `normalized_bits`: Bits per limb (e.g., 20 for 30+30)
    pub fn write_reg_from_limbs(&mut self, reg: Register, limbs: [u32; 2], normalized_bits: u8) {
        if reg != Register::R0 {
            let value = (limbs[0] as u64) | ((limbs[1] as u64) << normalized_bits);
            self.write_reg(reg, value);
            self.register_states.mark_normalized(reg);
        }
    }

    /// Write register from accumulated limbs (may exceed normalized_bits)
    ///
    /// For deferred carry model: stores accumulated limbs that may use up to
    /// `limb_bits` (30 bits for 30+30 architecture) per limb.
    ///
    /// The value is packed as: `limb0 | (limb1 << limb_bits)`
    /// and the register is marked as Accumulated.
    ///
    /// # Parameters
    /// - `limbs`: `[limb0, limb1]` accumulated limbs (may exceed normalized_bits)
    /// - `limb_bits`: Storage bits per limb (e.g., 30 for 30+30 architecture)
    pub fn write_reg_from_accumulated(&mut self, reg: Register, limbs: [u64; 2], limb_bits: u8) {
        if reg != Register::R0 {
            // Pack accumulated limbs using limb_bits
            // Note: This allows limbs to exceed normalized_bits
            let value = limbs[0] | (limbs[1] << limb_bits);
            self.write_reg(reg, value);
            self.register_states.mark_accumulated(reg);
        }
    }

    /// Read register limbs (handles both normalized and accumulated states)
    ///
    /// Reads limbs according to the register's current state:
    /// - If Normalized: extracts using `normalized_bits`
    /// - If Accumulated: extracts using `limb_bits`
    ///
    /// # Returns
    /// `[limb0, limb1]` as u64 to handle accumulated values
    pub fn read_reg_limbs_extended(&self, reg: Register, normalized_bits: u8, limb_bits: u8) -> [u64; 2] {
        let value = self.read_reg(reg);

        if self.register_states.get(reg).is_normalized() {
            // Extract using normalized_bits
            let mask = (1u64 << normalized_bits) - 1;
            [
                value & mask,
                (value >> normalized_bits) & mask,
            ]
        } else {
            // Extract using limb_bits (accumulated state)
            let mask = (1u64 << limb_bits) - 1;
            [
                value & mask,
                (value >> limb_bits) & mask,
            ]
        }
    }

    /// Get all registers in normalized form for trace capture
    ///
    /// Phase 7: When capturing execution traces with the deferred carry model enabled,
    /// we normalize all register values before adding them to the trace. This ensures
    /// the prover sees consistent normalized values (20-bit limbs) rather than having
    /// to handle accumulated values (30-bit limbs).
    ///
    /// This is simpler than adding normalization witness columns to the prover.
    pub fn get_normalized_regs(&self, normalized_bits: u8, limb_bits: u8) -> [u64; NUM_REGISTERS] {
        let mut normalized_regs = [0u64; NUM_REGISTERS];

        for i in 0..NUM_REGISTERS {
            let reg = Register::from_index(i as u8).expect("Invalid register index");
            let value = self.read_reg(reg);

            // If already normalized, use as-is
            if self.register_states.get(reg).is_normalized() {
                normalized_regs[i] = value;
            } else {
                // Accumulated values are packed with limb_bits (30), need to repack with normalized_bits (20)
                // Strategy: Unpack with limb_bits, take mod 2^40, repack with normalized_bits

                // Unpack accumulated limbs
                let mask_limb = (1u64 << limb_bits) - 1;
                let limb0_acc = value & mask_limb;
                let limb1_acc = (value >> limb_bits) & mask_limb;

                // Reconstruct full 60-bit value
                let value_60 = limb0_acc | (limb1_acc << limb_bits);

                // Take modulo 2^40 (for 40-bit arithmetic)
                let value_40 = value_60 & ((1u64 << 40) - 1);

                // Repack with normalized_bits (value_40 is already in the right form)
                normalized_regs[i] = value_40;
            }
        }

        normalized_regs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let state = VMState::new(0x1000);
        assert_eq!(state.pc, 0x1000);
        assert_eq!(state.cycles, 0);
        assert!(!state.is_halted());
    }

    #[test]
    fn test_r0_hardwired_zero() {
        let mut state = VMState::new(0);

        // R0 always reads as zero
        assert_eq!(state.read_reg(Register::R0), 0);

        // Writing to R0 has no effect
        state.write_reg(Register::R0, 42);
        assert_eq!(state.read_reg(Register::R0), 0);
        assert_eq!(state.regs[0], 0);
    }

    #[test]
    fn test_register_read_write() {
        let mut state = VMState::new(0);

        // Write and read R1
        state.write_reg(Register::R1, 100);
        assert_eq!(state.read_reg(Register::R1), 100);

        // Write and read R15
        state.write_reg(Register::R15, 0xFF_FFFF_FFFF);
        assert_eq!(state.read_reg(Register::R15), 0xFF_FFFF_FFFF);
    }

    #[test]
    fn test_halt() {
        let mut state = VMState::new(0);

        assert!(!state.is_halted());

        state.halt(HaltReason::Ebreak);
        assert!(state.is_halted());
        assert_eq!(state.halt_reason, Some(HaltReason::Ebreak));
    }

    #[test]
    fn test_cycles() {
        let mut state = VMState::new(0);

        assert_eq!(state.cycles, 0);

        state.inc_cycles();
        assert_eq!(state.cycles, 1);

        state.inc_cycles();
        state.inc_cycles();
        assert_eq!(state.cycles, 3);
    }

    #[test]
    fn test_pc_advance() {
        let mut state = VMState::new(0x1000);

        // Advance forward
        state.advance_pc(4);
        assert_eq!(state.pc, 0x1004);

        // Advance backward
        state.advance_pc(-8);
        assert_eq!(state.pc, 0x0FFC);
    }

    #[test]
    fn test_bound_tracking() {
        let mut state = VMState::new(0);

        // R0 bound is always 0 (constant zero)
        assert_eq!(state.read_bound(Register::R0).max_bits, 0);

        // Other registers start with program width (40 bits)
        assert_eq!(state.read_bound(Register::R1).max_bits, 40);

        // Write a tighter bound
        let bound_32 = ValueBound::from_type_width(32);
        state.write_bound(Register::R1, bound_32);
        assert_eq!(state.read_bound(Register::R1).max_bits, 32);

        // Writing to R0 bound has no effect
        state.write_bound(Register::R0, ValueBound::from_type_width(64));
        assert_eq!(state.read_bound(Register::R0).max_bits, 0);
    }

    #[test]
    fn test_write_reg_with_bound() {
        let mut state = VMState::new(0);

        // Write value and bound together
        let value = 100u64;
        let bound = ValueBound::from_constant(value);
        state.write_reg_with_bound(Register::R2, value, bound);

        assert_eq!(state.read_reg(Register::R2), 100);
        assert_eq!(state.read_bound(Register::R2).max_bits, 7); // 100 needs 7 bits
    }

    #[test]
    fn test_normalize_accumulated_value() {
        let mut state = VMState::new(0);

        // Test case: ADDI R1, R0, 100 produces accumulated [100, 0]
        // Packed with limb_bits=30: 100 | (0 << 30) = 100
        let accumulated_limbs = [100u64, 0u64];
        let value_30bit = accumulated_limbs[0] | (accumulated_limbs[1] << 30);
        state.write_reg_from_accumulated(Register::R1, accumulated_limbs, 30);

        // Get normalized registers
        let normalized = state.get_normalized_regs(20, 30);

        // Expected: [100, 0] normalized is still [100, 0]
        // Packed with normalized_bits=20: 100 | (0 << 20) = 100
        assert_eq!(normalized[1], value_30bit);

        // Test case 2: Accumulated value [1048660, 1048575] (from ADDI R2, R2, -16 with R2=100)
        // This represents the two's complement arithmetic result
        let accumulated_limbs2 = [1048660u64, 1048575u64];
        state.write_reg_from_accumulated(Register::R2, accumulated_limbs2, 30);

        let normalized2 = state.get_normalized_regs(20, 30);

        // Calculate expected normalized value
        // The accumulated value is packed with 30-bit limbs
        let value_60bit = accumulated_limbs2[0] | (accumulated_limbs2[1] << 30);
        // We want the value modulo 2^40 (40-bit arithmetic)
        let value_40bit = value_60bit & ((1u64 << 40) - 1);
        // This should equal the same value repacked with 20-bit normalized limbs
        let expected = value_40bit;

        assert_eq!(normalized2[2], expected,
            "Accumulated [1048660, 1048575] should normalize correctly");
    }
}
