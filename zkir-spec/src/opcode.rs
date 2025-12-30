//! # ZKIR v3.4 Opcode Definitions
//!
//! This module defines the opcode values for all ZKIR instructions.
//! Opcodes are 6 bits (0x00-0x3F).
//!
//! ## Opcode Encoding
//!
//! Opcodes are organized by instruction family:
//! - 0x00-0x08: Arithmetic (ADD, SUB, MUL, MULH, DIVU, REMU, DIV, REM, ADDI)
//! - 0x10-0x15: Logical (AND, OR, XOR, ANDI, ORI, XORI)
//! - 0x18-0x1D: Shift (SLL, SRL, SRA, SLLI, SRLI, SRAI)
//! - 0x20-0x28: Compare (SLTU, SGEU, SLT, SGE, SEQ, SNE) + Cmov (CMOV, CMOVZ, CMOVNZ)
//! - 0x30-0x35: Load (LB, LBU, LH, LHU, LW, LD)
//! - 0x38-0x3B: Store (SB, SH, SW, SD)
//! - 0x40-0x45: Branch (BEQ, BNE, BLT, BGE, BLTU, BGEU)
//! - 0x48-0x49: Jump (JAL, JALR)
//! - 0x50-0x51: System (ECALL, EBREAK)

use serde::{Deserialize, Serialize};

/// Instruction opcode (6 bits, values 0x00-0x51)
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Opcode {
    // ========== Arithmetic (0x00-0x08) ==========
    /// ADD: rd = rs1 + rs2
    Add = 0x00,
    /// SUB: rd = rs1 - rs2
    Sub = 0x01,
    /// MUL: rd = (rs1 * rs2) lower bits
    Mul = 0x02,
    /// MULH: rd = (rs1 * rs2) upper bits
    Mulh = 0x03,
    /// DIVU: rd = rs1 / rs2 (unsigned)
    Divu = 0x04,
    /// REMU: rd = rs1 % rs2 (unsigned)
    Remu = 0x05,
    /// DIV: rd = rs1 / rs2 (signed)
    Div = 0x06,
    /// REM: rd = rs1 % rs2 (signed)
    Rem = 0x07,
    /// ADDI: rd = rs1 + imm
    Addi = 0x08,

    // ========== Logical (0x10-0x15) ==========
    /// AND: rd = rs1 & rs2
    And = 0x10,
    /// OR: rd = rs1 | rs2
    Or = 0x11,
    /// XOR: rd = rs1 ^ rs2
    Xor = 0x12,
    /// ANDI: rd = rs1 & imm
    Andi = 0x13,
    /// ORI: rd = rs1 | imm
    Ori = 0x14,
    /// XORI: rd = rs1 ^ imm
    Xori = 0x15,

    // ========== Shift (0x18-0x1D) ==========
    /// SLL: rd = rs1 << rs2
    Sll = 0x18,
    /// SRL: rd = rs1 >> rs2 (logical)
    Srl = 0x19,
    /// SRA: rd = rs1 >> rs2 (arithmetic)
    Sra = 0x1A,
    /// SLLI: rd = rs1 << shamt
    Slli = 0x1B,
    /// SRLI: rd = rs1 >> shamt (logical)
    Srli = 0x1C,
    /// SRAI: rd = rs1 >> shamt (arithmetic)
    Srai = 0x1D,

    // ========== Compare (0x20-0x25) ==========
    /// SLTU: rd = (rs1 < rs2) ? 1 : 0 (unsigned)
    Sltu = 0x20,
    /// SGEU: rd = (rs1 >= rs2) ? 1 : 0 (unsigned)
    Sgeu = 0x21,
    /// SLT: rd = (rs1 < rs2) ? 1 : 0 (signed)
    Slt = 0x22,
    /// SGE: rd = (rs1 >= rs2) ? 1 : 0 (signed)
    Sge = 0x23,
    /// SEQ: rd = (rs1 == rs2) ? 1 : 0
    Seq = 0x24,
    /// SNE: rd = (rs1 != rs2) ? 1 : 0
    Sne = 0x25,

    // ========== Conditional Move (0x26-0x28) ==========
    /// CMOV: rd = (rs2 != 0) ? rs1 : rd
    Cmov = 0x26,
    /// CMOVZ: rd = (rs2 == 0) ? rs1 : rd
    Cmovz = 0x27,
    /// CMOVNZ: rd = (rs2 != 0) ? rs1 : rd
    Cmovnz = 0x28,

    // ========== Load (0x30-0x35) ==========
    /// LB: rd = sign_extend(mem[rs1 + imm][7:0])
    Lb = 0x30,
    /// LBU: rd = zero_extend(mem[rs1 + imm][7:0])
    Lbu = 0x31,
    /// LH: rd = sign_extend(mem[rs1 + imm][15:0])
    Lh = 0x32,
    /// LHU: rd = zero_extend(mem[rs1 + imm][15:0])
    Lhu = 0x33,
    /// LW: rd = sign_extend(mem[rs1 + imm][31:0])
    Lw = 0x34,
    /// LD: rd = mem[rs1 + imm][59:0]
    Ld = 0x35,

    // ========== Store (0x38-0x3B) ==========
    /// SB: mem[rs1 + imm][7:0] = rs2[7:0]
    Sb = 0x38,
    /// SH: mem[rs1 + imm][15:0] = rs2[15:0]
    Sh = 0x39,
    /// SW: mem[rs1 + imm][31:0] = rs2[31:0]
    Sw = 0x3A,
    /// SD: mem[rs1 + imm][59:0] = rs2[59:0]
    Sd = 0x3B,

    // ========== Branch (0x40-0x45) ==========
    /// BEQ: if (rs1 == rs2) PC += offset
    Beq = 0x40,
    /// BNE: if (rs1 != rs2) PC += offset
    Bne = 0x41,
    /// BLT: if (rs1 < rs2) PC += offset (signed)
    Blt = 0x42,
    /// BGE: if (rs1 >= rs2) PC += offset (signed)
    Bge = 0x43,
    /// BLTU: if (rs1 < rs2) PC += offset (unsigned)
    Bltu = 0x44,
    /// BGEU: if (rs1 >= rs2) PC += offset (unsigned)
    Bgeu = 0x45,

    // ========== Jump (0x48-0x49) ==========
    /// JAL: rd = PC + 4; PC += offset
    Jal = 0x48,
    /// JALR: rd = PC + 4; PC = (rs1 + imm) & ~1
    Jalr = 0x49,

    // ========== System (0x50-0x51) ==========
    /// ECALL: System call
    Ecall = 0x50,
    /// EBREAK: Breakpoint
    Ebreak = 0x51,
}

impl Opcode {
    /// Opcode width in bits
    pub const BITS: usize = 7;

    /// Opcode mask (0x7F for 7 bits)
    pub const MASK: u32 = 0x7F;

    /// Try to convert from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            // Arithmetic
            0x00 => Some(Opcode::Add),
            0x01 => Some(Opcode::Sub),
            0x02 => Some(Opcode::Mul),
            0x03 => Some(Opcode::Mulh),
            0x04 => Some(Opcode::Divu),
            0x05 => Some(Opcode::Remu),
            0x06 => Some(Opcode::Div),
            0x07 => Some(Opcode::Rem),
            0x08 => Some(Opcode::Addi),

            // Logical
            0x10 => Some(Opcode::And),
            0x11 => Some(Opcode::Or),
            0x12 => Some(Opcode::Xor),
            0x13 => Some(Opcode::Andi),
            0x14 => Some(Opcode::Ori),
            0x15 => Some(Opcode::Xori),

            // Shift
            0x18 => Some(Opcode::Sll),
            0x19 => Some(Opcode::Srl),
            0x1A => Some(Opcode::Sra),
            0x1B => Some(Opcode::Slli),
            0x1C => Some(Opcode::Srli),
            0x1D => Some(Opcode::Srai),

            // Compare
            0x20 => Some(Opcode::Sltu),
            0x21 => Some(Opcode::Sgeu),
            0x22 => Some(Opcode::Slt),
            0x23 => Some(Opcode::Sge),
            0x24 => Some(Opcode::Seq),
            0x25 => Some(Opcode::Sne),

            // Conditional Move
            0x26 => Some(Opcode::Cmov),
            0x27 => Some(Opcode::Cmovz),
            0x28 => Some(Opcode::Cmovnz),

            // Load
            0x30 => Some(Opcode::Lb),
            0x31 => Some(Opcode::Lbu),
            0x32 => Some(Opcode::Lh),
            0x33 => Some(Opcode::Lhu),
            0x34 => Some(Opcode::Lw),
            0x35 => Some(Opcode::Ld),

            // Store
            0x38 => Some(Opcode::Sb),
            0x39 => Some(Opcode::Sh),
            0x3A => Some(Opcode::Sw),
            0x3B => Some(Opcode::Sd),

            // Branch
            0x40 => Some(Opcode::Beq),
            0x41 => Some(Opcode::Bne),
            0x42 => Some(Opcode::Blt),
            0x43 => Some(Opcode::Bge),
            0x44 => Some(Opcode::Bltu),
            0x45 => Some(Opcode::Bgeu),

            // Jump
            0x48 => Some(Opcode::Jal),
            0x49 => Some(Opcode::Jalr),

            // System
            0x50 => Some(Opcode::Ecall),
            0x51 => Some(Opcode::Ebreak),

            _ => None,
        }
    }

    /// Convert to u8
    #[inline]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Extract opcode from 32-bit instruction word
    #[inline]
    pub fn from_instruction(instruction: u32) -> Option<Self> {
        Self::from_u8((instruction & Self::MASK) as u8)
    }

    /// Check if this is an arithmetic opcode
    #[inline]
    pub const fn is_arithmetic(self) -> bool {
        matches!(
            self,
            Opcode::Add
                | Opcode::Sub
                | Opcode::Mul
                | Opcode::Mulh
                | Opcode::Divu
                | Opcode::Remu
                | Opcode::Div
                | Opcode::Rem
                | Opcode::Addi
        )
    }

    /// Check if this is a logical opcode
    #[inline]
    pub const fn is_logical(self) -> bool {
        matches!(
            self,
            Opcode::And | Opcode::Or | Opcode::Xor | Opcode::Andi | Opcode::Ori | Opcode::Xori
        )
    }

    /// Check if this is a shift opcode
    #[inline]
    pub const fn is_shift(self) -> bool {
        matches!(
            self,
            Opcode::Sll
                | Opcode::Srl
                | Opcode::Sra
                | Opcode::Slli
                | Opcode::Srli
                | Opcode::Srai
        )
    }

    /// Check if this is a compare opcode
    #[inline]
    pub const fn is_compare(self) -> bool {
        matches!(
            self,
            Opcode::Sltu | Opcode::Sgeu | Opcode::Slt | Opcode::Sge | Opcode::Seq | Opcode::Sne
        )
    }

    /// Check if this is a conditional move opcode
    #[inline]
    pub const fn is_cmov(self) -> bool {
        matches!(self, Opcode::Cmov | Opcode::Cmovz | Opcode::Cmovnz)
    }

    /// Check if this is a load opcode
    #[inline]
    pub const fn is_load(self) -> bool {
        matches!(
            self,
            Opcode::Lb | Opcode::Lbu | Opcode::Lh | Opcode::Lhu | Opcode::Lw | Opcode::Ld
        )
    }

    /// Check if this is a store opcode
    #[inline]
    pub const fn is_store(self) -> bool {
        matches!(self, Opcode::Sb | Opcode::Sh | Opcode::Sw | Opcode::Sd)
    }

    /// Check if this is a branch opcode
    #[inline]
    pub const fn is_branch(self) -> bool {
        matches!(
            self,
            Opcode::Beq | Opcode::Bne | Opcode::Blt | Opcode::Bge | Opcode::Bltu | Opcode::Bgeu
        )
    }

    /// Check if this is a jump opcode
    #[inline]
    pub const fn is_jump(self) -> bool {
        matches!(self, Opcode::Jal | Opcode::Jalr)
    }

    /// Check if this is a system opcode
    #[inline]
    pub const fn is_system(self) -> bool {
        matches!(self, Opcode::Ecall | Opcode::Ebreak)
    }

    /// Check if this is an immediate instruction (I-type)
    #[inline]
    pub const fn uses_immediate(self) -> bool {
        matches!(
            self,
            Opcode::Addi
                | Opcode::Andi
                | Opcode::Ori
                | Opcode::Xori
                | Opcode::Slli
                | Opcode::Srli
                | Opcode::Srai
                | Opcode::Lb
                | Opcode::Lbu
                | Opcode::Lh
                | Opcode::Lhu
                | Opcode::Lw
                | Opcode::Ld
                | Opcode::Sb
                | Opcode::Sh
                | Opcode::Sw
                | Opcode::Sd
                | Opcode::Jalr
        )
    }

    /// Get the instruction family
    #[inline]
    pub const fn family(self) -> InstructionFamily {
        if self.is_arithmetic() {
            InstructionFamily::Arithmetic
        } else if self.is_logical() {
            InstructionFamily::Logical
        } else if self.is_shift() {
            InstructionFamily::Shift
        } else if self.is_compare() {
            InstructionFamily::Compare
        } else if self.is_cmov() {
            InstructionFamily::Cmov
        } else if self.is_load() {
            InstructionFamily::Load
        } else if self.is_store() {
            InstructionFamily::Store
        } else if self.is_branch() {
            InstructionFamily::Branch
        } else if self.is_jump() {
            InstructionFamily::Jump
        } else {
            InstructionFamily::System
        }
    }

    // ========================================================================
    // Raw opcode check functions (for u32 opcode values from instructions)
    // ========================================================================

    /// Check if raw opcode value is arithmetic (0x00-0x08)
    #[inline]
    pub fn is_arithmetic_raw(opcode: u32) -> bool {
        Self::from_u8(opcode as u8).map_or(false, |o| o.is_arithmetic())
    }

    /// Check if raw opcode value is logical (0x10-0x15)
    #[inline]
    pub fn is_logical_raw(opcode: u32) -> bool {
        Self::from_u8(opcode as u8).map_or(false, |o| o.is_logical())
    }

    /// Check if raw opcode value is shift (0x18-0x1D)
    #[inline]
    pub fn is_shift_raw(opcode: u32) -> bool {
        Self::from_u8(opcode as u8).map_or(false, |o| o.is_shift())
    }

    /// Check if raw opcode value is compare (0x20-0x25)
    #[inline]
    pub fn is_compare_raw(opcode: u32) -> bool {
        Self::from_u8(opcode as u8).map_or(false, |o| o.is_compare())
    }

    /// Check if raw opcode value is conditional move (0x26-0x28)
    #[inline]
    pub fn is_cmov_raw(opcode: u32) -> bool {
        Self::from_u8(opcode as u8).map_or(false, |o| o.is_cmov())
    }

    /// Check if raw opcode value is load (0x30-0x35)
    #[inline]
    pub fn is_load_raw(opcode: u32) -> bool {
        Self::from_u8(opcode as u8).map_or(false, |o| o.is_load())
    }

    /// Check if raw opcode value is store (0x38-0x3B)
    #[inline]
    pub fn is_store_raw(opcode: u32) -> bool {
        Self::from_u8(opcode as u8).map_or(false, |o| o.is_store())
    }

    /// Check if raw opcode value is branch (0x40-0x45)
    #[inline]
    pub fn is_branch_raw(opcode: u32) -> bool {
        Self::from_u8(opcode as u8).map_or(false, |o| o.is_branch())
    }

    /// Check if raw opcode value is jump (0x48-0x49)
    #[inline]
    pub fn is_jump_raw(opcode: u32) -> bool {
        Self::from_u8(opcode as u8).map_or(false, |o| o.is_jump())
    }

    /// Check if raw opcode value is system (0x50-0x51)
    #[inline]
    pub fn is_system_raw(opcode: u32) -> bool {
        Self::from_u8(opcode as u8).map_or(false, |o| o.is_system())
    }

    /// Get instruction family from raw opcode value
    #[inline]
    pub fn family_raw(opcode: u32) -> Option<InstructionFamily> {
        Self::from_u8(opcode as u8).map(|o| o.family())
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Opcode::Add => "add",
            Opcode::Sub => "sub",
            Opcode::Mul => "mul",
            Opcode::Mulh => "mulh",
            Opcode::Divu => "divu",
            Opcode::Remu => "remu",
            Opcode::Div => "div",
            Opcode::Rem => "rem",
            Opcode::Addi => "addi",
            Opcode::And => "and",
            Opcode::Or => "or",
            Opcode::Xor => "xor",
            Opcode::Andi => "andi",
            Opcode::Ori => "ori",
            Opcode::Xori => "xori",
            Opcode::Sll => "sll",
            Opcode::Srl => "srl",
            Opcode::Sra => "sra",
            Opcode::Slli => "slli",
            Opcode::Srli => "srli",
            Opcode::Srai => "srai",
            Opcode::Sltu => "sltu",
            Opcode::Sgeu => "sgeu",
            Opcode::Slt => "slt",
            Opcode::Sge => "sge",
            Opcode::Seq => "seq",
            Opcode::Sne => "sne",
            Opcode::Cmov => "cmov",
            Opcode::Cmovz => "cmovz",
            Opcode::Cmovnz => "cmovnz",
            Opcode::Lb => "lb",
            Opcode::Lbu => "lbu",
            Opcode::Lh => "lh",
            Opcode::Lhu => "lhu",
            Opcode::Lw => "lw",
            Opcode::Ld => "ld",
            Opcode::Sb => "sb",
            Opcode::Sh => "sh",
            Opcode::Sw => "sw",
            Opcode::Sd => "sd",
            Opcode::Beq => "beq",
            Opcode::Bne => "bne",
            Opcode::Blt => "blt",
            Opcode::Bge => "bge",
            Opcode::Bltu => "bltu",
            Opcode::Bgeu => "bgeu",
            Opcode::Jal => "jal",
            Opcode::Jalr => "jalr",
            Opcode::Ecall => "ecall",
            Opcode::Ebreak => "ebreak",
        };
        write!(f, "{}", name)
    }
}

/// Instruction family for selector columns
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstructionFamily {
    /// Arithmetic: ADD, SUB, MUL, MULH, DIV, REM, ADDI
    Arithmetic = 0,
    /// Logical: AND, OR, XOR, ANDI, ORI, XORI
    Logical = 1,
    /// Shift: SLL, SRL, SRA, SLLI, SRLI, SRAI
    Shift = 2,
    /// Compare: SLT, SLTU, SGE, SGEU, SEQ, SNE
    Compare = 3,
    /// Conditional Move: CMOV, CMOVZ, CMOVNZ
    Cmov = 4,
    /// Load: LB, LBU, LH, LHU, LW, LD
    Load = 5,
    /// Store: SB, SH, SW, SD
    Store = 6,
    /// Branch: BEQ, BNE, BLT, BGE, BLTU, BGEU
    Branch = 7,
    /// Jump: JAL, JALR
    Jump = 8,
    /// System: ECALL, EBREAK
    System = 9,
}

impl InstructionFamily {
    /// Total number of instruction families
    pub const COUNT: usize = 10;

    /// Convert from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(InstructionFamily::Arithmetic),
            1 => Some(InstructionFamily::Logical),
            2 => Some(InstructionFamily::Shift),
            3 => Some(InstructionFamily::Compare),
            4 => Some(InstructionFamily::Cmov),
            5 => Some(InstructionFamily::Load),
            6 => Some(InstructionFamily::Store),
            7 => Some(InstructionFamily::Branch),
            8 => Some(InstructionFamily::Jump),
            9 => Some(InstructionFamily::System),
            _ => None,
        }
    }

    /// Convert to u8
    #[inline]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }
}

impl std::fmt::Display for InstructionFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            InstructionFamily::Arithmetic => "arithmetic",
            InstructionFamily::Logical => "logical",
            InstructionFamily::Shift => "shift",
            InstructionFamily::Compare => "compare",
            InstructionFamily::Cmov => "cmov",
            InstructionFamily::Load => "load",
            InstructionFamily::Store => "store",
            InstructionFamily::Branch => "branch",
            InstructionFamily::Jump => "jump",
            InstructionFamily::System => "system",
        };
        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_values() {
        assert_eq!(Opcode::Add.to_u8(), 0x00);
        assert_eq!(Opcode::Addi.to_u8(), 0x08);
        assert_eq!(Opcode::And.to_u8(), 0x10);
        assert_eq!(Opcode::Sll.to_u8(), 0x18);
        assert_eq!(Opcode::Sltu.to_u8(), 0x20);
        assert_eq!(Opcode::Lb.to_u8(), 0x30);
        assert_eq!(Opcode::Sb.to_u8(), 0x38);
        assert_eq!(Opcode::Beq.to_u8(), 0x40);
        assert_eq!(Opcode::Jal.to_u8(), 0x48);
        assert_eq!(Opcode::Ecall.to_u8(), 0x50);
    }

    #[test]
    fn test_opcode_from_u8() {
        assert_eq!(Opcode::from_u8(0x00), Some(Opcode::Add));
        assert_eq!(Opcode::from_u8(0x10), Some(Opcode::And));
        assert_eq!(Opcode::from_u8(0xFF), None);
    }

    #[test]
    fn test_opcode_family() {
        assert_eq!(Opcode::Add.family(), InstructionFamily::Arithmetic);
        assert_eq!(Opcode::Addi.family(), InstructionFamily::Arithmetic);
        assert_eq!(Opcode::And.family(), InstructionFamily::Logical);
        assert_eq!(Opcode::Sll.family(), InstructionFamily::Shift);
        assert_eq!(Opcode::Slt.family(), InstructionFamily::Compare);
        assert_eq!(Opcode::Cmov.family(), InstructionFamily::Cmov);
        assert_eq!(Opcode::Lb.family(), InstructionFamily::Load);
        assert_eq!(Opcode::Sb.family(), InstructionFamily::Store);
        assert_eq!(Opcode::Beq.family(), InstructionFamily::Branch);
        assert_eq!(Opcode::Jal.family(), InstructionFamily::Jump);
        assert_eq!(Opcode::Ecall.family(), InstructionFamily::System);
    }

    #[test]
    fn test_opcode_from_instruction() {
        let instruction: u32 = 0x12345600 | Opcode::Add.to_u8() as u32;
        assert_eq!(Opcode::from_instruction(instruction), Some(Opcode::Add));

        let instruction: u32 = 0x12345600 | Opcode::Beq.to_u8() as u32;
        assert_eq!(Opcode::from_instruction(instruction), Some(Opcode::Beq));
    }

    #[test]
    fn test_instruction_family_count() {
        assert_eq!(InstructionFamily::COUNT, 10);
    }

    #[test]
    fn test_uses_immediate() {
        assert!(Opcode::Addi.uses_immediate());
        assert!(Opcode::Lw.uses_immediate());
        assert!(Opcode::Sw.uses_immediate());
        assert!(!Opcode::Add.uses_immediate());
        assert!(!Opcode::Beq.uses_immediate());
    }
}
