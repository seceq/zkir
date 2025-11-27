//! Register definitions for ZK IR (32-bit only, no field registers)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Number of integer registers
pub const NUM_REGISTERS: usize = 32;

/// Integer register (r0-r31)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Register {
    R0 = 0,   // zero - hardwired to 0
    R1 = 1,   // rv   - return value
    R2 = 2,   // sp   - stack pointer
    R3 = 3,   // fp   - frame pointer
    R4 = 4,   // a0   - argument 0
    R5 = 5,   // a1   - argument 1
    R6 = 6,   // a2   - argument 2
    R7 = 7,   // a3   - argument 3
    R8 = 8,   // t0   - temporary
    R9 = 9,   // t1
    R10 = 10, // t2
    R11 = 11, // t3
    R12 = 12, // t4
    R13 = 13, // t5
    R14 = 14, // t6
    R15 = 15, // t7
    R16 = 16, // s0   - saved
    R17 = 17, // s1
    R18 = 18, // s2
    R19 = 19, // s3
    R20 = 20, // s4
    R21 = 21, // s5
    R22 = 22, // s6
    R23 = 23, // s7
    R24 = 24, // t8
    R25 = 25, // t9
    R26 = 26, // t10
    R27 = 27, // t11
    R28 = 28, // gp   - global pointer
    R29 = 29, // tp   - thread pointer
    R30 = 30, // ra   - return address
    R31 = 31, // reserved
}

impl Register {
    pub const ZERO: Self = Self::R0;
    pub const RV: Self = Self::R1;
    pub const SP: Self = Self::R2;
    pub const FP: Self = Self::R3;
    pub const A0: Self = Self::R4;
    pub const A1: Self = Self::R5;
    pub const A2: Self = Self::R6;
    pub const A3: Self = Self::R7;
    pub const RA: Self = Self::R30;

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
            Self::R1 => "rv",
            Self::R2 => "sp",
            Self::R3 => "fp",
            Self::R4 => "a0",
            Self::R5 => "a1",
            Self::R6 => "a2",
            Self::R7 => "a3",
            Self::R8 => "t0",
            Self::R9 => "t1",
            Self::R10 => "t2",
            Self::R11 => "t3",
            Self::R12 => "t4",
            Self::R13 => "t5",
            Self::R14 => "t6",
            Self::R15 => "t7",
            Self::R16 => "s0",
            Self::R17 => "s1",
            Self::R18 => "s2",
            Self::R19 => "s3",
            Self::R20 => "s4",
            Self::R21 => "s5",
            Self::R22 => "s6",
            Self::R23 => "s7",
            Self::R24 => "t8",
            Self::R25 => "t9",
            Self::R26 => "t10",
            Self::R27 => "t11",
            Self::R28 => "gp",
            Self::R29 => "tp",
            Self::R30 => "ra",
            Self::R31 => "r31",
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
