//! VM state for ZK IR

use zkir_spec::{Register, NUM_REGISTERS};
use crate::memory::Memory;

/// VM state
#[derive(Debug, Clone)]
pub struct VMState {
    /// Integer registers (r0-r31)
    pub registers: [u32; NUM_REGISTERS],

    /// Program counter
    pub pc: u32,

    /// Memory
    pub memory: Memory,

    /// Cycle count
    pub cycle: u64,

    /// Halted flag
    pub halted: bool,

    /// Halt reason
    pub halt_reason: Option<HaltReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HaltReason {
    /// Normal HALT instruction
    Halt,
    /// Assertion failed
    AssertionFailed { pc: u32, msg: String },
    /// Invalid instruction
    InvalidInstruction { pc: u32, word: u32 },
    /// Division by zero
    DivisionByZero { pc: u32 },
    /// Out of cycles
    OutOfCycles,
    /// Memory error
    MemoryError { address: u32, msg: String },
    /// Syscall error
    SyscallError { code: u32, msg: String },
    /// No more inputs
    InputExhausted,
}

impl VMState {
    pub fn new(stack_size: u32, heap_size: u32) -> Self {
        let mut state = VMState {
            registers: [0; NUM_REGISTERS],
            pc: zkir_spec::CODE_BASE,
            memory: Memory::new(stack_size, heap_size),
            cycle: 0,
            halted: false,
            halt_reason: None,
        };

        // Initialize stack pointer
        state.registers[Register::SP.index()] = zkir_spec::STACK_TOP;
        state.registers[Register::FP.index()] = zkir_spec::STACK_TOP;

        state
    }

    /// Read register (r0 always returns 0)
    #[inline]
    pub fn read_reg(&self, reg: Register) -> u32 {
        if reg == Register::ZERO {
            0
        } else {
            self.registers[reg.index()]
        }
    }

    /// Write register (writes to r0 are ignored)
    #[inline]
    pub fn write_reg(&mut self, reg: Register, value: u32) {
        if reg != Register::ZERO {
            self.registers[reg.index()] = value;
        }
    }

    /// Halt execution
    pub fn halt(&mut self, reason: HaltReason) {
        self.halted = true;
        self.halt_reason = Some(reason);
    }
}
