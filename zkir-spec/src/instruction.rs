//! ZK IR v2.2 Instruction set (77 instructions)

use crate::register::Register;
use serde::{Deserialize, Serialize};

/// ZK IR Instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    // ========== R-type ALU (opcode = 0000) ==========

    // Arithmetic (ext=00, ext=10, ext=11)
    Add { rd: Register, rs1: Register, rs2: Register },
    Sub { rd: Register, rs1: Register, rs2: Register },
    Mul { rd: Register, rs1: Register, rs2: Register },
    Mulh { rd: Register, rs1: Register, rs2: Register },
    Mulhu { rd: Register, rs1: Register, rs2: Register },
    Mulhsu { rd: Register, rs1: Register, rs2: Register },

    // Division (ext=00)
    Div { rd: Register, rs1: Register, rs2: Register },
    Divu { rd: Register, rs1: Register, rs2: Register },
    Rem { rd: Register, rs1: Register, rs2: Register },
    Remu { rd: Register, rs1: Register, rs2: Register },

    // Logic (ext=00, ext=01)
    And { rd: Register, rs1: Register, rs2: Register },
    Andn { rd: Register, rs1: Register, rs2: Register },
    Or { rd: Register, rs1: Register, rs2: Register },
    Orn { rd: Register, rs1: Register, rs2: Register },
    Xor { rd: Register, rs1: Register, rs2: Register },
    Xnor { rd: Register, rs1: Register, rs2: Register },

    // Shift (ext=00, ext=01, ext=10)
    Sll { rd: Register, rs1: Register, rs2: Register },
    Srl { rd: Register, rs1: Register, rs2: Register },
    Sra { rd: Register, rs1: Register, rs2: Register },
    Rol { rd: Register, rs1: Register, rs2: Register },
    Ror { rd: Register, rs1: Register, rs2: Register },

    // Compare (ext=00, ext=10)
    Slt { rd: Register, rs1: Register, rs2: Register },
    Sltu { rd: Register, rs1: Register, rs2: Register },
    Min { rd: Register, rs1: Register, rs2: Register },
    Max { rd: Register, rs1: Register, rs2: Register },
    Minu { rd: Register, rs1: Register, rs2: Register },
    Maxu { rd: Register, rs1: Register, rs2: Register },

    // Bit Manipulation (ext=01, ext=10, ext=11)
    Clz { rd: Register, rs1: Register, rs2: Register },
    Ctz { rd: Register, rs1: Register, rs2: Register },
    Cpop { rd: Register, rs1: Register, rs2: Register },
    Rev8 { rd: Register, rs1: Register, rs2: Register },

    // Conditional Move (ext2=000000, ext2=000001)
    Cmovz { rd: Register, rs1: Register, rs2: Register },
    Cmovnz { rd: Register, rs1: Register, rs2: Register },

    // Field Operations (bits 29:24 = 111111)
    Fadd { rd: Register, rs1: Register, rs2: Register },
    Fsub { rd: Register, rs1: Register, rs2: Register },
    Fmul { rd: Register, rs1: Register, rs2: Register },
    Fneg { rd: Register, rs1: Register, rs2: Register },
    Finv { rd: Register, rs1: Register, rs2: Register },

    // ========== I-type Immediate (opcode = 0001) ==========
    Addi { rd: Register, rs1: Register, imm: i16 },
    Slti { rd: Register, rs1: Register, imm: i16 },
    Sltiu { rd: Register, rs1: Register, imm: i16 },
    Xori { rd: Register, rs1: Register, imm: i16 },
    Ori { rd: Register, rs1: Register, imm: i16 },
    Andi { rd: Register, rs1: Register, imm: i16 },
    Slli { rd: Register, rs1: Register, shamt: u8 },
    Srli { rd: Register, rs1: Register, shamt: u8 },
    Srai { rd: Register, rs1: Register, shamt: u8 },

    // ========== Load (opcode = 0010) ==========
    Lb { rd: Register, rs1: Register, imm: i16 },
    Lh { rd: Register, rs1: Register, imm: i16 },
    Lw { rd: Register, rs1: Register, imm: i16 },
    Lbu { rd: Register, rs1: Register, imm: i16 },
    Lhu { rd: Register, rs1: Register, imm: i16 },

    // ========== Store (opcode = 0011) ==========
    Sb { rs1: Register, rs2: Register, imm: i16 },
    Sh { rs1: Register, rs2: Register, imm: i16 },
    Sw { rs1: Register, rs2: Register, imm: i16 },

    // ========== Branch (opcodes 0100-1001) ==========
    Beq { rs1: Register, rs2: Register, imm: i16 },
    Bne { rs1: Register, rs2: Register, imm: i16 },
    Blt { rs1: Register, rs2: Register, imm: i16 },
    Bge { rs1: Register, rs2: Register, imm: i16 },
    Bltu { rs1: Register, rs2: Register, imm: i16 },
    Bgeu { rs1: Register, rs2: Register, imm: i16 },

    // ========== Upper Immediate (opcodes 1010-1011) ==========
    Lui { rd: Register, imm: i32 },
    Auipc { rd: Register, imm: i32 },

    // ========== Jump (opcodes 1100-1101) ==========
    Jal { rd: Register, imm: i32 },
    Jalr { rd: Register, rs1: Register, imm: i16 },

    // ========== ZK Operations (opcode = 1110) ==========
    Read { rd: Register },
    Write { rs1: Register },
    Hint { rd: Register },
    Commit { rs1: Register },
    AssertEq { rs1: Register, rs2: Register },
    AssertNe { rs1: Register, rs2: Register },
    AssertZero { rs1: Register },
    RangeCheck { rs1: Register, bits: u8 },
    Debug { rs1: Register },
    Halt,

    // ========== System (opcode = 1111) ==========
    Ecall,
    Ebreak,
}

impl Instruction {
    /// Returns true if this instruction modifies the destination register
    pub fn writes_register(&self) -> bool {
        matches!(
            self,
            Self::Add { .. }
                | Self::Sub { .. }
                | Self::Mul { .. }
                | Self::Mulh { .. }
                | Self::Mulhu { .. }
                | Self::Mulhsu { .. }
                | Self::Div { .. }
                | Self::Divu { .. }
                | Self::Rem { .. }
                | Self::Remu { .. }
                | Self::And { .. }
                | Self::Andn { .. }
                | Self::Or { .. }
                | Self::Orn { .. }
                | Self::Xor { .. }
                | Self::Xnor { .. }
                | Self::Sll { .. }
                | Self::Srl { .. }
                | Self::Sra { .. }
                | Self::Rol { .. }
                | Self::Ror { .. }
                | Self::Slt { .. }
                | Self::Sltu { .. }
                | Self::Min { .. }
                | Self::Max { .. }
                | Self::Minu { .. }
                | Self::Maxu { .. }
                | Self::Clz { .. }
                | Self::Ctz { .. }
                | Self::Cpop { .. }
                | Self::Rev8 { .. }
                | Self::Cmovz { .. }
                | Self::Cmovnz { .. }
                | Self::Fadd { .. }
                | Self::Fsub { .. }
                | Self::Fmul { .. }
                | Self::Fneg { .. }
                | Self::Finv { .. }
                | Self::Addi { .. }
                | Self::Slti { .. }
                | Self::Sltiu { .. }
                | Self::Xori { .. }
                | Self::Ori { .. }
                | Self::Andi { .. }
                | Self::Slli { .. }
                | Self::Srli { .. }
                | Self::Srai { .. }
                | Self::Lb { .. }
                | Self::Lh { .. }
                | Self::Lw { .. }
                | Self::Lbu { .. }
                | Self::Lhu { .. }
                | Self::Lui { .. }
                | Self::Auipc { .. }
                | Self::Jal { .. }
                | Self::Jalr { .. }
                | Self::Read { .. }
                | Self::Hint { .. }
        )
    }

    /// Returns true if this instruction is a control flow instruction
    pub fn is_control_flow(&self) -> bool {
        matches!(
            self,
            Self::Beq { .. }
                | Self::Bne { .. }
                | Self::Blt { .. }
                | Self::Bge { .. }
                | Self::Bltu { .. }
                | Self::Bgeu { .. }
                | Self::Jal { .. }
                | Self::Jalr { .. }
                | Self::Ecall
                | Self::Ebreak
                | Self::Halt
        )
    }

    /// Returns true if this instruction accesses memory
    pub fn accesses_memory(&self) -> bool {
        matches!(
            self,
            Self::Lb { .. }
                | Self::Lh { .. }
                | Self::Lw { .. }
                | Self::Lbu { .. }
                | Self::Lhu { .. }
                | Self::Sb { .. }
                | Self::Sh { .. }
                | Self::Sw { .. }
        )
    }

    /// Returns true if this is a ZK-specific instruction
    pub fn is_zk_instruction(&self) -> bool {
        matches!(
            self,
            Self::Read { .. }
                | Self::Write { .. }
                | Self::Hint { .. }
                | Self::Commit { .. }
                | Self::AssertEq { .. }
                | Self::AssertNe { .. }
                | Self::AssertZero { .. }
                | Self::RangeCheck { .. }
                | Self::Debug { .. }
                | Self::Halt
        )
    }

    /// Returns true if this is a field arithmetic instruction
    pub fn is_field_instruction(&self) -> bool {
        matches!(
            self,
            Self::Fadd { .. }
                | Self::Fsub { .. }
                | Self::Fmul { .. }
                | Self::Fneg { .. }
                | Self::Finv { .. }
        )
    }
}
