//! ZKIR v3.4 Instruction Set
//!
//! 31-bit instructions with 6-bit opcode and 4-bit register fields.
//!
//! ## Instruction Formats
//! - R-type:  [opcode:6][rd:4][rs1:4][rs2:4][funct:13]
//! - I-type:  [opcode:6][rd:4][rs1:4][imm19:19]
//! - B-type:  [opcode:6][rs1:4][rs2:4][offset:17]
//! - J-type:  [opcode:6][rd:4][offset:21]
//! - R4-type: [opcode:6][rd:4][rs1:4][rs2:4][rs3:4][funct:9]

use crate::register::Register;
use serde::{Deserialize, Serialize};

/// ZKIR v3.4 Instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    // ========== Arithmetic ==========
    /// ADD: rd = rs1 + rs2
    Add { rd: Register, rs1: Register, rs2: Register },

    /// SUB: rd = rs1 - rs2
    Sub { rd: Register, rs1: Register, rs2: Register },

    /// MUL: rd = (rs1 * rs2) & ((1 << 60) - 1) (lower 60 bits)
    Mul { rd: Register, rs1: Register, rs2: Register },

    /// MULH: rd = (rs1 * rs2) >> 60 (upper bits of 60×60 multiply)
    Mulh { rd: Register, rs1: Register, rs2: Register },

    /// DIVU: rd = rs1 / rs2 (unsigned)
    Divu { rd: Register, rs1: Register, rs2: Register },

    /// REMU: rd = rs1 % rs2 (unsigned)
    Remu { rd: Register, rs1: Register, rs2: Register },

    /// DIV: rd = rs1 / rs2 (signed)
    Div { rd: Register, rs1: Register, rs2: Register },

    /// REM: rd = rs1 % rs2 (signed)
    Rem { rd: Register, rs1: Register, rs2: Register },

    /// ADDI: rd = rs1 + imm (sign-extended)
    Addi { rd: Register, rs1: Register, imm: i32 },

    // ========== Logical ==========
    /// AND: rd = rs1 & rs2
    And { rd: Register, rs1: Register, rs2: Register },

    /// OR: rd = rs1 | rs2
    Or { rd: Register, rs1: Register, rs2: Register },

    /// XOR: rd = rs1 ^ rs2
    Xor { rd: Register, rs1: Register, rs2: Register },

    /// ANDI: rd = rs1 & imm
    Andi { rd: Register, rs1: Register, imm: i32 },

    /// ORI: rd = rs1 | imm
    Ori { rd: Register, rs1: Register, imm: i32 },

    /// XORI: rd = rs1 ^ imm
    Xori { rd: Register, rs1: Register, imm: i32 },

    // ========== Shift ==========
    /// SLL: rd = rs1 << (rs2 & 0x3F) (logical left shift)
    Sll { rd: Register, rs1: Register, rs2: Register },

    /// SRL: rd = rs1 >> (rs2 & 0x3F) (logical right shift)
    Srl { rd: Register, rs1: Register, rs2: Register },

    /// SRA: rd = rs1 >> (rs2 & 0x3F) (arithmetic right shift)
    Sra { rd: Register, rs1: Register, rs2: Register },

    /// SLLI: rd = rs1 << shamt
    Slli { rd: Register, rs1: Register, shamt: u8 },

    /// SRLI: rd = rs1 >> shamt (logical)
    Srli { rd: Register, rs1: Register, shamt: u8 },

    /// SRAI: rd = rs1 >> shamt (arithmetic)
    Srai { rd: Register, rs1: Register, shamt: u8 },

    // ========== Compare ==========
    /// SLTU: rd = (rs1 < rs2) ? 1 : 0 (unsigned)
    Sltu { rd: Register, rs1: Register, rs2: Register },

    /// SGEU: rd = (rs1 >= rs2) ? 1 : 0 (unsigned)
    Sgeu { rd: Register, rs1: Register, rs2: Register },

    /// SLT: rd = (rs1 < rs2) ? 1 : 0 (signed)
    Slt { rd: Register, rs1: Register, rs2: Register },

    /// SGE: rd = (rs1 >= rs2) ? 1 : 0 (signed)
    Sge { rd: Register, rs1: Register, rs2: Register },

    /// SEQ: rd = (rs1 == rs2) ? 1 : 0
    Seq { rd: Register, rs1: Register, rs2: Register },

    /// SNE: rd = (rs1 != rs2) ? 1 : 0
    Sne { rd: Register, rs1: Register, rs2: Register },

    // ========== Conditional Move ==========
    /// CMOV: rd = (rs2 != 0) ? rs1 : rd
    Cmov { rd: Register, rs1: Register, rs2: Register },

    /// CMOVZ: rd = (rs2 == 0) ? rs1 : rd
    Cmovz { rd: Register, rs1: Register, rs2: Register },

    /// CMOVNZ: rd = (rs2 != 0) ? rs1 : rd
    Cmovnz { rd: Register, rs1: Register, rs2: Register },

    // ========== Memory - Load ==========
    /// LB: rd = sign_extend(mem[rs1 + imm][7:0])
    Lb { rd: Register, rs1: Register, imm: i32 },

    /// LBU: rd = zero_extend(mem[rs1 + imm][7:0])
    Lbu { rd: Register, rs1: Register, imm: i32 },

    /// LH: rd = sign_extend(mem[rs1 + imm][15:0])
    Lh { rd: Register, rs1: Register, imm: i32 },

    /// LHU: rd = zero_extend(mem[rs1 + imm][15:0])
    Lhu { rd: Register, rs1: Register, imm: i32 },

    /// LW: rd = sign_extend(mem[rs1 + imm][31:0])
    Lw { rd: Register, rs1: Register, imm: i32 },

    /// LD: rd = mem[rs1 + imm][59:0] (60-bit load)
    Ld { rd: Register, rs1: Register, imm: i32 },

    // ========== Memory - Store ==========
    /// SB: mem[rs1 + imm][7:0] = rs2[7:0]
    Sb { rs1: Register, rs2: Register, imm: i32 },

    /// SH: mem[rs1 + imm][15:0] = rs2[15:0]
    Sh { rs1: Register, rs2: Register, imm: i32 },

    /// SW: mem[rs1 + imm][31:0] = rs2[31:0]
    Sw { rs1: Register, rs2: Register, imm: i32 },

    /// SD: mem[rs1 + imm][59:0] = rs2[59:0] (60-bit store)
    Sd { rs1: Register, rs2: Register, imm: i32 },

    // ========== Branch ==========
    /// BEQ: if (rs1 == rs2) PC += offset
    Beq { rs1: Register, rs2: Register, offset: i32 },

    /// BNE: if (rs1 != rs2) PC += offset
    Bne { rs1: Register, rs2: Register, offset: i32 },

    /// BLT: if (rs1 < rs2) PC += offset (signed)
    Blt { rs1: Register, rs2: Register, offset: i32 },

    /// BGE: if (rs1 >= rs2) PC += offset (signed)
    Bge { rs1: Register, rs2: Register, offset: i32 },

    /// BLTU: if (rs1 < rs2) PC += offset (unsigned)
    Bltu { rs1: Register, rs2: Register, offset: i32 },

    /// BGEU: if (rs1 >= rs2) PC += offset (unsigned)
    Bgeu { rs1: Register, rs2: Register, offset: i32 },

    // ========== Jump ==========
    /// JAL: rd = PC + 4; PC += offset
    Jal { rd: Register, offset: i32 },

    /// JALR: rd = PC + 4; PC = (rs1 + imm) & ~1
    Jalr { rd: Register, rs1: Register, imm: i32 },

    // ========== System ==========
    /// ECALL: System call (a0 = syscall number)
    Ecall,

    /// EBREAK: Breakpoint / halt execution
    Ebreak,
}

impl Instruction {
    /// Get instruction mnemonic
    pub fn mnemonic(&self) -> &'static str {
        match self {
            Instruction::Add { .. } => "add",
            Instruction::Sub { .. } => "sub",
            Instruction::Mul { .. } => "mul",
            Instruction::Mulh { .. } => "mulh",
            Instruction::Divu { .. } => "divu",
            Instruction::Remu { .. } => "remu",
            Instruction::Div { .. } => "div",
            Instruction::Rem { .. } => "rem",
            Instruction::Addi { .. } => "addi",
            Instruction::And { .. } => "and",
            Instruction::Or { .. } => "or",
            Instruction::Xor { .. } => "xor",
            Instruction::Andi { .. } => "andi",
            Instruction::Ori { .. } => "ori",
            Instruction::Xori { .. } => "xori",
            Instruction::Sll { .. } => "sll",
            Instruction::Srl { .. } => "srl",
            Instruction::Sra { .. } => "sra",
            Instruction::Slli { .. } => "slli",
            Instruction::Srli { .. } => "srli",
            Instruction::Srai { .. } => "srai",
            Instruction::Sltu { .. } => "sltu",
            Instruction::Sgeu { .. } => "sgeu",
            Instruction::Slt { .. } => "slt",
            Instruction::Sge { .. } => "sge",
            Instruction::Seq { .. } => "seq",
            Instruction::Sne { .. } => "sne",
            Instruction::Cmov { .. } => "cmov",
            Instruction::Cmovz { .. } => "cmovz",
            Instruction::Cmovnz { .. } => "cmovnz",
            Instruction::Lb { .. } => "lb",
            Instruction::Lbu { .. } => "lbu",
            Instruction::Lh { .. } => "lh",
            Instruction::Lhu { .. } => "lhu",
            Instruction::Lw { .. } => "lw",
            Instruction::Ld { .. } => "ld",
            Instruction::Sb { .. } => "sb",
            Instruction::Sh { .. } => "sh",
            Instruction::Sw { .. } => "sw",
            Instruction::Sd { .. } => "sd",
            Instruction::Beq { .. } => "beq",
            Instruction::Bne { .. } => "bne",
            Instruction::Blt { .. } => "blt",
            Instruction::Bge { .. } => "bge",
            Instruction::Bltu { .. } => "bltu",
            Instruction::Bgeu { .. } => "bgeu",
            Instruction::Jal { .. } => "jal",
            Instruction::Jalr { .. } => "jalr",
            Instruction::Ecall => "ecall",
            Instruction::Ebreak => "ebreak",
        }
    }

    /// Check if this is a branch instruction
    pub fn is_branch(&self) -> bool {
        matches!(
            self,
            Instruction::Beq { .. }
                | Instruction::Bne { .. }
                | Instruction::Blt { .. }
                | Instruction::Bge { .. }
                | Instruction::Bltu { .. }
                | Instruction::Bgeu { .. }
        )
    }

    /// Check if this is a jump instruction
    pub fn is_jump(&self) -> bool {
        matches!(self, Instruction::Jal { .. } | Instruction::Jalr { .. })
    }

    /// Check if this is a load instruction
    pub fn is_load(&self) -> bool {
        matches!(
            self,
            Instruction::Lb { .. }
                | Instruction::Lbu { .. }
                | Instruction::Lh { .. }
                | Instruction::Lhu { .. }
                | Instruction::Lw { .. }
                | Instruction::Ld { .. }
        )
    }

    /// Check if this is a store instruction
    pub fn is_store(&self) -> bool {
        matches!(
            self,
            Instruction::Sb { .. }
                | Instruction::Sh { .. }
                | Instruction::Sw { .. }
                | Instruction::Sd { .. }
        )
    }

    /// Check if this is a system instruction
    pub fn is_system(&self) -> bool {
        matches!(self, Instruction::Ecall | Instruction::Ebreak)
    }

    /// Get destination register if present
    pub fn rd(&self) -> Option<Register> {
        match self {
            Instruction::Add { rd, .. }
            | Instruction::Sub { rd, .. }
            | Instruction::Mul { rd, .. }
            | Instruction::Mulh { rd, .. }
            | Instruction::Divu { rd, .. }
            | Instruction::Remu { rd, .. }
            | Instruction::Div { rd, .. }
            | Instruction::Rem { rd, .. }
            | Instruction::Addi { rd, .. }
            | Instruction::And { rd, .. }
            | Instruction::Or { rd, .. }
            | Instruction::Xor { rd, .. }
            | Instruction::Andi { rd, .. }
            | Instruction::Ori { rd, .. }
            | Instruction::Xori { rd, .. }
            | Instruction::Sll { rd, .. }
            | Instruction::Srl { rd, .. }
            | Instruction::Sra { rd, .. }
            | Instruction::Slli { rd, .. }
            | Instruction::Srli { rd, .. }
            | Instruction::Srai { rd, .. }
            | Instruction::Sltu { rd, .. }
            | Instruction::Sgeu { rd, .. }
            | Instruction::Slt { rd, .. }
            | Instruction::Sge { rd, .. }
            | Instruction::Seq { rd, .. }
            | Instruction::Sne { rd, .. }
            | Instruction::Cmov { rd, .. }
            | Instruction::Cmovz { rd, .. }
            | Instruction::Cmovnz { rd, .. }
            | Instruction::Lb { rd, .. }
            | Instruction::Lbu { rd, .. }
            | Instruction::Lh { rd, .. }
            | Instruction::Lhu { rd, .. }
            | Instruction::Lw { rd, .. }
            | Instruction::Ld { rd, .. }
            | Instruction::Jal { rd, .. }
            | Instruction::Jalr { rd, .. } => Some(*rd),
            _ => None,
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // R-type
            Instruction::Add { rd, rs1, rs2 }
            | Instruction::Sub { rd, rs1, rs2 }
            | Instruction::Mul { rd, rs1, rs2 }
            | Instruction::Mulh { rd, rs1, rs2 }
            | Instruction::Divu { rd, rs1, rs2 }
            | Instruction::Remu { rd, rs1, rs2 }
            | Instruction::Div { rd, rs1, rs2 }
            | Instruction::Rem { rd, rs1, rs2 }
            | Instruction::And { rd, rs1, rs2 }
            | Instruction::Or { rd, rs1, rs2 }
            | Instruction::Xor { rd, rs1, rs2 }
            | Instruction::Sll { rd, rs1, rs2 }
            | Instruction::Srl { rd, rs1, rs2 }
            | Instruction::Sra { rd, rs1, rs2 }
            | Instruction::Sltu { rd, rs1, rs2 }
            | Instruction::Sgeu { rd, rs1, rs2 }
            | Instruction::Slt { rd, rs1, rs2 }
            | Instruction::Sge { rd, rs1, rs2 }
            | Instruction::Seq { rd, rs1, rs2 }
            | Instruction::Sne { rd, rs1, rs2 }
            | Instruction::Cmov { rd, rs1, rs2 }
            | Instruction::Cmovz { rd, rs1, rs2 }
            | Instruction::Cmovnz { rd, rs1, rs2 } => {
                write!(f, "{} {}, {}, {}", self.mnemonic(), rd, rs1, rs2)
            }

            // I-type (immediate)
            Instruction::Addi { rd, rs1, imm }
            | Instruction::Andi { rd, rs1, imm }
            | Instruction::Ori { rd, rs1, imm }
            | Instruction::Xori { rd, rs1, imm } => {
                write!(f, "{} {}, {}, {}", self.mnemonic(), rd, rs1, imm)
            }

            // I-type (shift)
            Instruction::Slli { rd, rs1, shamt }
            | Instruction::Srli { rd, rs1, shamt }
            | Instruction::Srai { rd, rs1, shamt } => {
                write!(f, "{} {}, {}, {}", self.mnemonic(), rd, rs1, shamt)
            }

            // I-type (load)
            Instruction::Lb { rd, rs1, imm }
            | Instruction::Lbu { rd, rs1, imm }
            | Instruction::Lh { rd, rs1, imm }
            | Instruction::Lhu { rd, rs1, imm }
            | Instruction::Lw { rd, rs1, imm }
            | Instruction::Ld { rd, rs1, imm } => {
                write!(f, "{} {}, {}({})", self.mnemonic(), rd, imm, rs1)
            }

            // S-type (store)
            Instruction::Sb { rs1, rs2, imm }
            | Instruction::Sh { rs1, rs2, imm }
            | Instruction::Sw { rs1, rs2, imm }
            | Instruction::Sd { rs1, rs2, imm } => {
                write!(f, "{} {}, {}({})", self.mnemonic(), rs2, imm, rs1)
            }

            // B-type (branch)
            Instruction::Beq { rs1, rs2, offset }
            | Instruction::Bne { rs1, rs2, offset }
            | Instruction::Blt { rs1, rs2, offset }
            | Instruction::Bge { rs1, rs2, offset }
            | Instruction::Bltu { rs1, rs2, offset }
            | Instruction::Bgeu { rs1, rs2, offset } => {
                write!(f, "{} {}, {}, {}", self.mnemonic(), rs1, rs2, offset)
            }

            // J-type
            Instruction::Jal { rd, offset } => {
                write!(f, "{} {}, {}", self.mnemonic(), rd, offset)
            }

            Instruction::Jalr { rd, rs1, imm } => {
                write!(f, "{} {}, {}({})", self.mnemonic(), rd, imm, rs1)
            }

            // System
            Instruction::Ecall => write!(f, "ecall"),
            Instruction::Ebreak => write!(f, "ebreak"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mnemonic() {
        let inst = Instruction::Add {
            rd: Register::A0,
            rs1: Register::A1,
            rs2: Register::A2,
        };
        assert_eq!(inst.mnemonic(), "add");
    }

    #[test]
    fn test_is_branch() {
        let beq = Instruction::Beq {
            rs1: Register::A0,
            rs2: Register::A1,
            offset: 4,
        };
        assert!(beq.is_branch());

        let add = Instruction::Add {
            rd: Register::A0,
            rs1: Register::A1,
            rs2: Register::A2,
        };
        assert!(!add.is_branch());
    }

    #[test]
    fn test_rd() {
        let add = Instruction::Add {
            rd: Register::A0,
            rs1: Register::A1,
            rs2: Register::A2,
        };
        assert_eq!(add.rd(), Some(Register::A0));

        let ecall = Instruction::Ecall;
        assert_eq!(ecall.rd(), None);
    }

    #[test]
    fn test_display() {
        let add = Instruction::Add {
            rd: Register::A0,
            rs1: Register::A1,
            rs2: Register::A2,
        };
        assert_eq!(format!("{}", add), "add a0, a1, a2");

        let addi = Instruction::Addi {
            rd: Register::A0,
            rs1: Register::A1,
            imm: 42,
        };
        assert_eq!(format!("{}", addi), "addi a0, a1, 42");

        let lw = Instruction::Lw {
            rd: Register::A0,
            rs1: Register::SP,
            imm: 8,
        };
        assert_eq!(format!("{}", lw), "lw a0, 8(sp)");
    }
}
