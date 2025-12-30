//! VM state management for ZKIR v3.4

use zkir_spec::{Register, ValueBound, NUM_REGISTERS};

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
}
