//! Register definitions for ZK IR.

use serde::{Deserialize, Serialize};
use std::fmt;

/// General-purpose registers (r0-r31)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Register {
    /// Zero register - always reads as 0, writes are ignored
    R0 = 0,
    /// Return value / accumulator
    R1 = 1,
    /// Stack pointer
    R2 = 2,
    /// Frame pointer
    R3 = 3,
    /// Function argument 0
    R4 = 4,
    /// Function argument 1
    R5 = 5,
    /// Function argument 2
    R6 = 6,
    /// Function argument 3
    R7 = 7,
    /// Caller-saved temporary
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
    /// Callee-saved
    R16 = 16,
    R17 = 17,
    R18 = 18,
    R19 = 19,
    R20 = 20,
    R21 = 21,
    R22 = 22,
    R23 = 23,
    /// Reserved / temporaries
    R24 = 24,
    R25 = 25,
    R26 = 26,
    R27 = 27,
    R28 = 28,
    R29 = 29,
    R30 = 30,
    R31 = 31,
}

impl Register {
    /// Alias for zero register
    pub const ZERO: Register = Register::R0;
    /// Alias for return value register
    pub const RV: Register = Register::R1;
    /// Alias for stack pointer
    pub const SP: Register = Register::R2;
    /// Alias for frame pointer
    pub const FP: Register = Register::R3;
    /// Alias for first argument
    pub const A0: Register = Register::R4;
    /// Alias for second argument
    pub const A1: Register = Register::R5;
    /// Alias for third argument
    pub const A2: Register = Register::R6;
    /// Alias for fourth argument
    pub const A3: Register = Register::R7;

    /// Create register from index (0-31)
    pub fn from_index(index: u8) -> Option<Self> {
        if index < 32 {
            // Safe because we checked the range
            Some(unsafe { std::mem::transmute(index) })
        } else {
            None
        }
    }

    /// Get register index (0-31)
    pub fn index(self) -> u8 {
        self as u8
    }

    /// Check if this is the zero register
    pub fn is_zero(self) -> bool {
        self == Register::R0
    }

    /// Check if this is a caller-saved register
    pub fn is_caller_saved(self) -> bool {
        let idx = self.index();
        (8..=15).contains(&idx)
    }

    /// Check if this is a callee-saved register
    pub fn is_callee_saved(self) -> bool {
        let idx = self.index();
        (16..=23).contains(&idx)
    }

    /// Check if this is an argument register
    pub fn is_argument(self) -> bool {
        let idx = self.index();
        (4..=7).contains(&idx)
    }

    /// Get canonical name for this register
    pub fn name(self) -> &'static str {
        match self {
            Register::R0 => "r0",
            Register::R1 => "r1",
            Register::R2 => "r2",
            Register::R3 => "r3",
            Register::R4 => "r4",
            Register::R5 => "r5",
            Register::R6 => "r6",
            Register::R7 => "r7",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::R10 => "r10",
            Register::R11 => "r11",
            Register::R12 => "r12",
            Register::R13 => "r13",
            Register::R14 => "r14",
            Register::R15 => "r15",
            Register::R16 => "r16",
            Register::R17 => "r17",
            Register::R18 => "r18",
            Register::R19 => "r19",
            Register::R20 => "r20",
            Register::R21 => "r21",
            Register::R22 => "r22",
            Register::R23 => "r23",
            Register::R24 => "r24",
            Register::R25 => "r25",
            Register::R26 => "r26",
            Register::R27 => "r27",
            Register::R28 => "r28",
            Register::R29 => "r29",
            Register::R30 => "r30",
            Register::R31 => "r31",
        }
    }

    /// Parse register from name (e.g., "r1", "sp", "fp")
    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.to_lowercase();
        match name.as_str() {
            "r0" | "zero" => Some(Register::R0),
            "r1" | "rv" | "ra" => Some(Register::R1),
            "r2" | "sp" => Some(Register::R2),
            "r3" | "fp" => Some(Register::R3),
            "r4" | "a0" => Some(Register::R4),
            "r5" | "a1" => Some(Register::R5),
            "r6" | "a2" => Some(Register::R6),
            "r7" | "a3" => Some(Register::R7),
            "r8" | "t0" => Some(Register::R8),
            "r9" | "t1" => Some(Register::R9),
            "r10" | "t2" => Some(Register::R10),
            "r11" | "t3" => Some(Register::R11),
            "r12" | "t4" => Some(Register::R12),
            "r13" | "t5" => Some(Register::R13),
            "r14" | "t6" => Some(Register::R14),
            "r15" | "t7" => Some(Register::R15),
            "r16" | "s0" => Some(Register::R16),
            "r17" | "s1" => Some(Register::R17),
            "r18" | "s2" => Some(Register::R18),
            "r19" | "s3" => Some(Register::R19),
            "r20" | "s4" => Some(Register::R20),
            "r21" | "s5" => Some(Register::R21),
            "r22" | "s6" => Some(Register::R22),
            "r23" | "s7" => Some(Register::R23),
            "r24" => Some(Register::R24),
            "r25" => Some(Register::R25),
            "r26" => Some(Register::R26),
            "r27" => Some(Register::R27),
            "r28" => Some(Register::R28),
            "r29" => Some(Register::R29),
            "r30" => Some(Register::R30),
            "r31" => Some(Register::R31),
            _ => None,
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Field registers (f0-f15) for 256-bit field element operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum FieldRegister {
    F0 = 0,
    F1 = 1,
    F2 = 2,
    F3 = 3,
    F4 = 4,
    F5 = 5,
    F6 = 6,
    F7 = 7,
    F8 = 8,
    F9 = 9,
    F10 = 10,
    F11 = 11,
    F12 = 12,
    F13 = 13,
    F14 = 14,
    F15 = 15,
}

impl FieldRegister {
    /// Create field register from index (0-15)
    pub fn from_index(index: u8) -> Option<Self> {
        if index < 16 {
            Some(unsafe { std::mem::transmute(index) })
        } else {
            None
        }
    }

    /// Get register index (0-15)
    pub fn index(self) -> u8 {
        self as u8
    }

    /// Get canonical name
    pub fn name(self) -> &'static str {
        match self {
            FieldRegister::F0 => "f0",
            FieldRegister::F1 => "f1",
            FieldRegister::F2 => "f2",
            FieldRegister::F3 => "f3",
            FieldRegister::F4 => "f4",
            FieldRegister::F5 => "f5",
            FieldRegister::F6 => "f6",
            FieldRegister::F7 => "f7",
            FieldRegister::F8 => "f8",
            FieldRegister::F9 => "f9",
            FieldRegister::F10 => "f10",
            FieldRegister::F11 => "f11",
            FieldRegister::F12 => "f12",
            FieldRegister::F13 => "f13",
            FieldRegister::F14 => "f14",
            FieldRegister::F15 => "f15",
        }
    }

    /// Parse from name
    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.to_lowercase();
        match name.as_str() {
            "f0" => Some(FieldRegister::F0),
            "f1" => Some(FieldRegister::F1),
            "f2" => Some(FieldRegister::F2),
            "f3" => Some(FieldRegister::F3),
            "f4" => Some(FieldRegister::F4),
            "f5" => Some(FieldRegister::F5),
            "f6" => Some(FieldRegister::F6),
            "f7" => Some(FieldRegister::F7),
            "f8" => Some(FieldRegister::F8),
            "f9" => Some(FieldRegister::F9),
            "f10" => Some(FieldRegister::F10),
            "f11" => Some(FieldRegister::F11),
            "f12" => Some(FieldRegister::F12),
            "f13" => Some(FieldRegister::F13),
            "f14" => Some(FieldRegister::F14),
            "f15" => Some(FieldRegister::F15),
            _ => None,
        }
    }
}

impl fmt::Display for FieldRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_index_roundtrip() {
        for i in 0..32u8 {
            let reg = Register::from_index(i).unwrap();
            assert_eq!(reg.index(), i);
        }
    }

    #[test]
    fn test_register_name_roundtrip() {
        for i in 0..32u8 {
            let reg = Register::from_index(i).unwrap();
            let name = reg.name();
            let parsed = Register::from_name(name).unwrap();
            assert_eq!(reg, parsed);
        }
    }

    #[test]
    fn test_register_aliases() {
        assert_eq!(Register::from_name("sp"), Some(Register::SP));
        assert_eq!(Register::from_name("fp"), Some(Register::FP));
        assert_eq!(Register::from_name("zero"), Some(Register::ZERO));
        assert_eq!(Register::from_name("a0"), Some(Register::A0));
    }

    #[test]
    fn test_field_register_roundtrip() {
        for i in 0..16u8 {
            let reg = FieldRegister::from_index(i).unwrap();
            assert_eq!(reg.index(), i);
        }
    }
}
