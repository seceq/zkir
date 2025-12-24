//! Register definitions for ZK IR v2.2 (RISC-V calling convention)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Number of registers
pub const NUM_REGISTERS: usize = 32;

/// Register (r0-r31)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Register {
    R0 = 0,   // zero - hardwired to 0
    R1 = 1,   // ra   - return address
    R2 = 2,   // sp   - stack pointer
    R3 = 3,   // gp   - global pointer
    R4 = 4,   // tp   - thread pointer
    R5 = 5,   // t0   - temporary (caller-saved)
    R6 = 6,   // t1
    R7 = 7,   // t2
    R8 = 8,   // fp/s0 - frame pointer (callee-saved)
    R9 = 9,   // s1   - saved register (callee-saved)
    R10 = 10, // a0   - argument 0 / return value (caller-saved)
    R11 = 11, // a1   - argument 1 / return value (caller-saved)
    R12 = 12, // a2   - argument 2 (caller-saved)
    R13 = 13, // a3   - argument 3 (caller-saved)
    R14 = 14, // a4   - argument 4 (caller-saved)
    R15 = 15, // a5   - argument 5 (caller-saved)
    R16 = 16, // a6   - argument 6 (caller-saved)
    R17 = 17, // a7   - argument 7 (caller-saved)
    R18 = 18, // s2   - saved register (callee-saved)
    R19 = 19, // s3
    R20 = 20, // s4
    R21 = 21, // s5
    R22 = 22, // s6
    R23 = 23, // s7
    R24 = 24, // s8
    R25 = 25, // s9
    R26 = 26, // s10
    R27 = 27, // s11
    R28 = 28, // t3   - temporary (caller-saved)
    R29 = 29, // t4
    R30 = 30, // t5
    R31 = 31, // t6
}

impl Register {
    pub const ZERO: Self = Self::R0;
    pub const RA: Self = Self::R1;
    pub const SP: Self = Self::R2;
    pub const GP: Self = Self::R3;
    pub const TP: Self = Self::R4;
    pub const FP: Self = Self::R8;
    pub const A0: Self = Self::R10;
    pub const A1: Self = Self::R11;
    pub const A2: Self = Self::R12;
    pub const A3: Self = Self::R13;
    pub const A4: Self = Self::R14;
    pub const A5: Self = Self::R15;
    pub const A6: Self = Self::R16;
    pub const A7: Self = Self::R17;

    #[inline]
    pub fn from_index(index: usize) -> Option<Self> {
        if index < 32 {
            Some(unsafe { std::mem::transmute(index as u8) })
        } else {
            None
        }
    }

    #[inline]
    pub fn index(self) -> usize {
        self as usize
    }

    #[inline]
    pub fn is_zero(self) -> bool {
        self == Self::ZERO
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::R0 => "zero",
            Self::R1 => "ra",
            Self::R2 => "sp",
            Self::R3 => "gp",
            Self::R4 => "tp",
            Self::R5 => "t0",
            Self::R6 => "t1",
            Self::R7 => "t2",
            Self::R8 => "fp",
            Self::R9 => "s1",
            Self::R10 => "a0",
            Self::R11 => "a1",
            Self::R12 => "a2",
            Self::R13 => "a3",
            Self::R14 => "a4",
            Self::R15 => "a5",
            Self::R16 => "a6",
            Self::R17 => "a7",
            Self::R18 => "s2",
            Self::R19 => "s3",
            Self::R20 => "s4",
            Self::R21 => "s5",
            Self::R22 => "s6",
            Self::R23 => "s7",
            Self::R24 => "s8",
            Self::R25 => "s9",
            Self::R26 => "s10",
            Self::R27 => "s11",
            Self::R28 => "t3",
            Self::R29 => "t4",
            Self::R30 => "t5",
            Self::R31 => "t6",
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
