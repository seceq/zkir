//! ZK IR Instruction set

use crate::register::Register;
use serde::{Deserialize, Serialize};

/// ZK IR Instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    // Arithmetic (R-type)
    Add { rd: Register, rs1: Register, rs2: Register },
    Sub { rd: Register, rs1: Register, rs2: Register },
    Mul { rd: Register, rs1: Register, rs2: Register },
    Mulh { rd: Register, rs1: Register, rs2: Register },
    Mulhu { rd: Register, rs1: Register, rs2: Register },
    Div { rd: Register, rs1: Register, rs2: Register },
    Divu { rd: Register, rs1: Register, rs2: Register },
    Rem { rd: Register, rs1: Register, rs2: Register },
    Remu { rd: Register, rs1: Register, rs2: Register },

    // Arithmetic Immediate (I-type)
    Addi { rd: Register, rs1: Register, imm: i16 },
    Slti { rd: Register, rs1: Register, imm: i16 },
    Sltiu { rd: Register, rs1: Register, imm: i16 },
    Xori { rd: Register, rs1: Register, imm: i16 },
    Ori { rd: Register, rs1: Register, imm: i16 },
    Andi { rd: Register, rs1: Register, imm: i16 },
    Slli { rd: Register, rs1: Register, shamt: u8 },
    Srli { rd: Register, rs1: Register, shamt: u8 },
    Srai { rd: Register, rs1: Register, shamt: u8 },

    // Logic (R-type)
    And { rd: Register, rs1: Register, rs2: Register },
    Or { rd: Register, rs1: Register, rs2: Register },
    Xor { rd: Register, rs1: Register, rs2: Register },
    Sll { rd: Register, rs1: Register, rs2: Register },
    Srl { rd: Register, rs1: Register, rs2: Register },
    Sra { rd: Register, rs1: Register, rs2: Register },
    Slt { rd: Register, rs1: Register, rs2: Register },
    Sltu { rd: Register, rs1: Register, rs2: Register },

    // Load (I-type)
    Lw { rd: Register, rs1: Register, imm: i16 },
    Lh { rd: Register, rs1: Register, imm: i16 },
    Lhu { rd: Register, rs1: Register, imm: i16 },
    Lb { rd: Register, rs1: Register, imm: i16 },
    Lbu { rd: Register, rs1: Register, imm: i16 },

    // Store (S-type)
    Sw { rs1: Register, rs2: Register, imm: i16 },
    Sh { rs1: Register, rs2: Register, imm: i16 },
    Sb { rs1: Register, rs2: Register, imm: i16 },

    // Branch (B-type)
    Beq { rs1: Register, rs2: Register, imm: i16 },
    Bne { rs1: Register, rs2: Register, imm: i16 },
    Blt { rs1: Register, rs2: Register, imm: i16 },
    Bge { rs1: Register, rs2: Register, imm: i16 },
    Bltu { rs1: Register, rs2: Register, imm: i16 },
    Bgeu { rs1: Register, rs2: Register, imm: i16 },

    // Jump (J-type and I-type)
    Jal { rd: Register, imm: i32 },
    Jalr { rd: Register, rs1: Register, imm: i16 },

    // Upper Immediate (U-type)
    Lui { rd: Register, imm: i32 },
    Auipc { rd: Register, imm: i32 },

    // System
    Ecall,
    Ebreak,

    // ZK-Custom (opcode = 0x0B)
    AssertEq { rs1: Register, rs2: Register },
    AssertNe { rs1: Register, rs2: Register },
    AssertZero { rs1: Register },
    RangeCheck { rs1: Register, bits: u8 },
    Commit { rs1: Register },
    Halt,

    // ZK I/O (opcode = 0x5B)
    Read { rd: Register },
    Write { rs1: Register },
    Hint { rd: Register },
}
