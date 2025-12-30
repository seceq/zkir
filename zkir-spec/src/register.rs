//! Register definitions for ZKIR v3.4
//!
//! 16 registers (r0-r15), each containing 2 × 20-bit limbs in default mode (40-bit values).
//! 4-bit encoding for compact instruction format.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Number of registers
pub const NUM_REGISTERS: usize = 16;

/// Register (r0-r15)
///
/// ## Calling Convention
/// - r0 (zero): Hardwired to zero
/// - r1 (ra): Return address
/// - r2 (sp): Stack pointer
/// - r3 (fp): Frame pointer
/// - r4-r5 (a0-a1): Arguments/return values
/// - r6-r9 (a2-a5): Arguments
/// - r10-r13 (s0-s3): Saved registers (callee-saved)
/// - r14-r15 (t0-t1): Temporaries (caller-saved)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Register {
    R0 = 0,   // zero - hardwired to 0
    R1 = 1,   // ra   - return address
    R2 = 2,   // sp   - stack pointer
    R3 = 3,   // fp   - frame pointer
    R4 = 4,   // a0   - argument 0 / return value
    R5 = 5,   // a1   - argument 1 / return value
    R6 = 6,   // a2   - argument 2
    R7 = 7,   // a3   - argument 3
    R8 = 8,   // a4   - argument 4
    R9 = 9,   // a5   - argument 5
    R10 = 10, // s0   - saved register
    R11 = 11, // s1   - saved register
    R12 = 12, // s2   - saved register
    R13 = 13, // s3   - saved register
    R14 = 14, // t0   - temporary
    R15 = 15, // t1   - temporary
}

impl Register {
    // Constant aliases for common registers
    pub const ZERO: Self = Self::R0;
    pub const RA: Self = Self::R1;
    pub const SP: Self = Self::R2;
    pub const FP: Self = Self::R3;
    pub const A0: Self = Self::R4;
    pub const A1: Self = Self::R5;
    pub const A2: Self = Self::R6;
    pub const A3: Self = Self::R7;
    pub const A4: Self = Self::R8;
    pub const A5: Self = Self::R9;
    pub const S0: Self = Self::R10;
    pub const S1: Self = Self::R11;
    pub const S2: Self = Self::R12;
    pub const S3: Self = Self::R13;
    pub const T0: Self = Self::R14;
    pub const T1: Self = Self::R15;

    /// Create register from 4-bit index (0-15)
    #[inline]
    pub fn from_index(index: u8) -> Option<Self> {
        if index < 16 {
            Some(unsafe { std::mem::transmute(index) })
        } else {
            None
        }
    }

    /// Get the 4-bit index (0-15)
    #[inline]
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// Check if this is the zero register
    #[inline]
    pub const fn is_zero(self) -> bool {
        matches!(self, Self::R0)
    }

    /// Get the register name
    pub const fn name(self) -> &'static str {
        match self {
            Self::R0 => "zero",
            Self::R1 => "ra",
            Self::R2 => "sp",
            Self::R3 => "fp",
            Self::R4 => "a0",
            Self::R5 => "a1",
            Self::R6 => "a2",
            Self::R7 => "a3",
            Self::R8 => "a4",
            Self::R9 => "a5",
            Self::R10 => "s0",
            Self::R11 => "s1",
            Self::R12 => "s2",
            Self::R13 => "s3",
            Self::R14 => "t0",
            Self::R15 => "t1",
        }
    }

    /// Get the numeric name (r0-r15)
    pub fn numeric_name(self) -> String {
        format!("r{}", self.index())
    }

    /// Parse register from name (supports both aliases and numeric form)
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "zero" | "r0" => Some(Self::R0),
            "ra" | "r1" => Some(Self::R1),
            "sp" | "r2" => Some(Self::R2),
            "fp" | "r3" => Some(Self::R3),
            "a0" | "r4" => Some(Self::R4),
            "a1" | "r5" => Some(Self::R5),
            "a2" | "r6" => Some(Self::R6),
            "a3" | "r7" => Some(Self::R7),
            "a4" | "r8" => Some(Self::R8),
            "a5" | "r9" => Some(Self::R9),
            "s0" | "r10" => Some(Self::R10),
            "s1" | "r11" => Some(Self::R11),
            "s2" | "r12" => Some(Self::R12),
            "s3" | "r13" => Some(Self::R13),
            "t0" | "r14" => Some(Self::R14),
            "t1" | "r15" => Some(Self::R15),
            _ => None,
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_count() {
        assert_eq!(NUM_REGISTERS, 16);
    }

    #[test]
    fn test_register_index() {
        assert_eq!(Register::R0.index(), 0);
        assert_eq!(Register::R15.index(), 15);
        assert_eq!(Register::ZERO.index(), 0);
        assert_eq!(Register::RA.index(), 1);
        assert_eq!(Register::SP.index(), 2);
    }

    #[test]
    fn test_from_index() {
        assert_eq!(Register::from_index(0), Some(Register::R0));
        assert_eq!(Register::from_index(15), Some(Register::R15));
        assert_eq!(Register::from_index(16), None);
        assert_eq!(Register::from_index(255), None);
    }

    #[test]
    fn test_is_zero() {
        assert!(Register::R0.is_zero());
        assert!(Register::ZERO.is_zero());
        assert!(!Register::R1.is_zero());
        assert!(!Register::R15.is_zero());
    }

    #[test]
    fn test_names() {
        assert_eq!(Register::R0.name(), "zero");
        assert_eq!(Register::R1.name(), "ra");
        assert_eq!(Register::R2.name(), "sp");
        assert_eq!(Register::R3.name(), "fp");
        assert_eq!(Register::R4.name(), "a0");
        assert_eq!(Register::R10.name(), "s0");
        assert_eq!(Register::R14.name(), "t0");
    }

    #[test]
    fn test_from_name() {
        assert_eq!(Register::from_name("zero"), Some(Register::R0));
        assert_eq!(Register::from_name("r0"), Some(Register::R0));
        assert_eq!(Register::from_name("ra"), Some(Register::R1));
        assert_eq!(Register::from_name("r1"), Some(Register::R1));
        assert_eq!(Register::from_name("sp"), Some(Register::R2));
        assert_eq!(Register::from_name("a0"), Some(Register::R4));
        assert_eq!(Register::from_name("s0"), Some(Register::R10));
        assert_eq!(Register::from_name("t0"), Some(Register::R14));
        assert_eq!(Register::from_name("invalid"), None);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Register::R0), "zero");
        assert_eq!(format!("{}", Register::R1), "ra");
        assert_eq!(format!("{}", Register::R15), "t1");
    }

    #[test]
    fn test_numeric_name() {
        assert_eq!(Register::R0.numeric_name(), "r0");
        assert_eq!(Register::R15.numeric_name(), "r15");
    }
}
