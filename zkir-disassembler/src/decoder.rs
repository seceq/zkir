//! Instruction decoder for ZKIR v3.4 (32-bit encoding)
//!
//! Decodes 32-bit words into Instruction structs.
//! This is the inverse of the encoder in zkir-assembler.
//!
//! ## Instruction Formats (32 bits total, 7-bit opcode field):
//! - R-type:  [opcode:7][rd:4][rs1:4][rs2:4][funct:13]  = 7+4+4+4+13 = 32 bits
//! - I-type:  [opcode:7][rd:4][rs1:4][imm:17]           = 7+4+4+17 = 32 bits
//! - S-type:  [opcode:7][rs1:4][rs2:4][imm:17]          = 7+4+4+17 = 32 bits
//! - B-type:  [opcode:7][rs1:4][rs2:4][offset:17]       = 7+4+4+17 = 32 bits
//! - J-type:  [opcode:7][rd:4][offset:21]               = 7+4+21 = 32 bits
//!
//! Note: Despite documentation claiming "6-bit opcodes", the actual opcode values
//! range from 0x00-0x51 which requires 7 bits. We use 7 bits for the opcode field.

use zkir_spec::{Instruction, Opcode, Register};
use crate::error::{DisassemblerError, Result};

/// Decode 32-bit instruction word
pub fn decode(word: u32) -> Result<Instruction> {
    // Extract 7-bit opcode (bits 6:0)
    let opcode_byte = (word & 0x7F) as u8;

    // Get opcode enum from byte
    let opcode = Opcode::from_u8(opcode_byte)
        .ok_or(DisassemblerError::UnknownOpcode(opcode_byte))?;

    match opcode {
        // ========== Arithmetic (R-type: 0x00-0x07) ==========
        Opcode::Add => decode_r_type(word, |rd, rs1, rs2| Instruction::Add { rd, rs1, rs2 }),
        Opcode::Sub => decode_r_type(word, |rd, rs1, rs2| Instruction::Sub { rd, rs1, rs2 }),
        Opcode::Mul => decode_r_type(word, |rd, rs1, rs2| Instruction::Mul { rd, rs1, rs2 }),
        Opcode::Mulh => decode_r_type(word, |rd, rs1, rs2| Instruction::Mulh { rd, rs1, rs2 }),
        Opcode::Divu => decode_r_type(word, |rd, rs1, rs2| Instruction::Divu { rd, rs1, rs2 }),
        Opcode::Remu => decode_r_type(word, |rd, rs1, rs2| Instruction::Remu { rd, rs1, rs2 }),
        Opcode::Div => decode_r_type(word, |rd, rs1, rs2| Instruction::Div { rd, rs1, rs2 }),
        Opcode::Rem => decode_r_type(word, |rd, rs1, rs2| Instruction::Rem { rd, rs1, rs2 }),

        // ========== Immediate Arithmetic (I-type: 0x08) ==========
        Opcode::Addi => decode_i_type(word, |rd, rs1, imm| Instruction::Addi { rd, rs1, imm }),

        // ========== Logical (R-type: 0x10-0x12) ==========
        Opcode::And => decode_r_type(word, |rd, rs1, rs2| Instruction::And { rd, rs1, rs2 }),
        Opcode::Or => decode_r_type(word, |rd, rs1, rs2| Instruction::Or { rd, rs1, rs2 }),
        Opcode::Xor => decode_r_type(word, |rd, rs1, rs2| Instruction::Xor { rd, rs1, rs2 }),

        // ========== Immediate Logical (I-type: 0x13-0x15) ==========
        Opcode::Andi => decode_i_type(word, |rd, rs1, imm| Instruction::Andi { rd, rs1, imm }),
        Opcode::Ori => decode_i_type(word, |rd, rs1, imm| Instruction::Ori { rd, rs1, imm }),
        Opcode::Xori => decode_i_type(word, |rd, rs1, imm| Instruction::Xori { rd, rs1, imm }),

        // ========== Shift (R-type: 0x18-0x1A) ==========
        Opcode::Sll => decode_r_type(word, |rd, rs1, rs2| Instruction::Sll { rd, rs1, rs2 }),
        Opcode::Srl => decode_r_type(word, |rd, rs1, rs2| Instruction::Srl { rd, rs1, rs2 }),
        Opcode::Sra => decode_r_type(word, |rd, rs1, rs2| Instruction::Sra { rd, rs1, rs2 }),

        // ========== Shift Immediate (I-type: 0x1B-0x1D) ==========
        Opcode::Slli => decode_shift(word, |rd, rs1, shamt| Instruction::Slli { rd, rs1, shamt }),
        Opcode::Srli => decode_shift(word, |rd, rs1, shamt| Instruction::Srli { rd, rs1, shamt }),
        Opcode::Srai => decode_shift(word, |rd, rs1, shamt| Instruction::Srai { rd, rs1, shamt }),

        // ========== Compare (R-type: 0x20-0x25) ==========
        Opcode::Sltu => decode_r_type(word, |rd, rs1, rs2| Instruction::Sltu { rd, rs1, rs2 }),
        Opcode::Sgeu => decode_r_type(word, |rd, rs1, rs2| Instruction::Sgeu { rd, rs1, rs2 }),
        Opcode::Slt => decode_r_type(word, |rd, rs1, rs2| Instruction::Slt { rd, rs1, rs2 }),
        Opcode::Sge => decode_r_type(word, |rd, rs1, rs2| Instruction::Sge { rd, rs1, rs2 }),
        Opcode::Seq => decode_r_type(word, |rd, rs1, rs2| Instruction::Seq { rd, rs1, rs2 }),
        Opcode::Sne => decode_r_type(word, |rd, rs1, rs2| Instruction::Sne { rd, rs1, rs2 }),

        // ========== Conditional Move (R-type: 0x26-0x28) ==========
        Opcode::Cmov => decode_r_type(word, |rd, rs1, rs2| Instruction::Cmov { rd, rs1, rs2 }),
        Opcode::Cmovz => decode_r_type(word, |rd, rs1, rs2| Instruction::Cmovz { rd, rs1, rs2 }),
        Opcode::Cmovnz => decode_r_type(word, |rd, rs1, rs2| Instruction::Cmovnz { rd, rs1, rs2 }),

        // ========== Load (I-type: 0x30-0x35) ==========
        Opcode::Lb => decode_i_type(word, |rd, rs1, imm| Instruction::Lb { rd, rs1, imm }),
        Opcode::Lbu => decode_i_type(word, |rd, rs1, imm| Instruction::Lbu { rd, rs1, imm }),
        Opcode::Lh => decode_i_type(word, |rd, rs1, imm| Instruction::Lh { rd, rs1, imm }),
        Opcode::Lhu => decode_i_type(word, |rd, rs1, imm| Instruction::Lhu { rd, rs1, imm }),
        Opcode::Lw => decode_i_type(word, |rd, rs1, imm| Instruction::Lw { rd, rs1, imm }),
        Opcode::Ld => decode_i_type(word, |rd, rs1, imm| Instruction::Ld { rd, rs1, imm }),

        // ========== Store (S-type: 0x38-0x3B) ==========
        Opcode::Sb => decode_store(word, |rs1, rs2, imm| Instruction::Sb { rs1, rs2, imm }),
        Opcode::Sh => decode_store(word, |rs1, rs2, imm| Instruction::Sh { rs1, rs2, imm }),
        Opcode::Sw => decode_store(word, |rs1, rs2, imm| Instruction::Sw { rs1, rs2, imm }),
        Opcode::Sd => decode_store(word, |rs1, rs2, imm| Instruction::Sd { rs1, rs2, imm }),

        // ========== Branch (B-type: 0x40-0x45) ==========
        Opcode::Beq => decode_b_type(word, |rs1, rs2, offset| Instruction::Beq { rs1, rs2, offset }),
        Opcode::Bne => decode_b_type(word, |rs1, rs2, offset| Instruction::Bne { rs1, rs2, offset }),
        Opcode::Blt => decode_b_type(word, |rs1, rs2, offset| Instruction::Blt { rs1, rs2, offset }),
        Opcode::Bge => decode_b_type(word, |rs1, rs2, offset| Instruction::Bge { rs1, rs2, offset }),
        Opcode::Bltu => decode_b_type(word, |rs1, rs2, offset| Instruction::Bltu { rs1, rs2, offset }),
        Opcode::Bgeu => decode_b_type(word, |rs1, rs2, offset| Instruction::Bgeu { rs1, rs2, offset }),

        // ========== Jump (J-type: 0x48, I-type: 0x49) ==========
        Opcode::Jal => decode_j_type(word, |rd, offset| Instruction::Jal { rd, offset }),
        Opcode::Jalr => decode_i_type(word, |rd, rs1, imm| Instruction::Jalr { rd, rs1, imm }),

        // ========== System (0x50-0x51) ==========
        Opcode::Ecall => Ok(Instruction::Ecall),
        Opcode::Ebreak => Ok(Instruction::Ebreak),
    }
}

/// Decode R-type instruction
/// Format: [opcode:7][rd:4][rs1:4][rs2:4][funct:13]
fn decode_r_type<F>(word: u32, constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register, Register, Register) -> Instruction,
{
    let rd = decode_register((word >> 7) & 0xF)?;
    let rs1 = decode_register((word >> 11) & 0xF)?;
    let rs2 = decode_register((word >> 15) & 0xF)?;
    Ok(constructor(rd, rs1, rs2))
}

/// Decode I-type instruction
/// Format: [opcode:7][rd:4][rs1:4][imm:17]
fn decode_i_type<F>(word: u32, constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register, Register, i32) -> Instruction,
{
    let rd = decode_register((word >> 7) & 0xF)?;
    let rs1 = decode_register((word >> 11) & 0xF)?;
    let imm_raw = (word >> 15) & 0x1FFFF;
    let imm = sign_extend(imm_raw, 17);
    Ok(constructor(rd, rs1, imm))
}

/// Decode shift instruction (I-type with shamt instead of full immediate)
/// Format: [opcode:7][rd:4][rs1:4][shamt:17] (shamt uses only lower bits)
fn decode_shift<F>(word: u32, constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register, Register, u8) -> Instruction,
{
    let rd = decode_register((word >> 7) & 0xF)?;
    let rs1 = decode_register((word >> 11) & 0xF)?;
    let shamt = ((word >> 15) & 0xFF) as u8;
    Ok(constructor(rd, rs1, shamt))
}

/// Decode store instruction (S-type: base address in first register field)
/// Format: [opcode:7][rs1:4][rs2:4][imm:17]
fn decode_store<F>(word: u32, constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register, Register, i32) -> Instruction,
{
    let rs1 = decode_register((word >> 7) & 0xF)?;  // base address
    let rs2 = decode_register((word >> 11) & 0xF)?; // value to store
    let imm_raw = (word >> 15) & 0x1FFFF;
    let imm = sign_extend(imm_raw, 17);
    Ok(constructor(rs1, rs2, imm))
}

/// Decode B-type instruction
/// Format: [opcode:7][rs1:4][rs2:4][offset:17]
fn decode_b_type<F>(word: u32, constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register, Register, i32) -> Instruction,
{
    let rs1 = decode_register((word >> 7) & 0xF)?;
    let rs2 = decode_register((word >> 11) & 0xF)?;
    let offset_raw = (word >> 15) & 0x1FFFF;
    let offset = sign_extend(offset_raw, 17);
    Ok(constructor(rs1, rs2, offset))
}

/// Decode J-type instruction
/// Format: [opcode:7][rd:4][offset:21]
fn decode_j_type<F>(word: u32, constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register, i32) -> Instruction,
{
    let rd = decode_register((word >> 7) & 0xF)?;
    let offset_raw = (word >> 11) & 0x1FFFFF;
    let offset = sign_extend(offset_raw, 21);
    Ok(constructor(rd, offset))
}

/// Decode register from 4-bit index
fn decode_register(index: u32) -> Result<Register> {
    Register::from_index(index as u8)
        .ok_or(DisassemblerError::InvalidEncoding(index))
}

/// Sign-extend a value from n bits to 32 bits
fn sign_extend(value: u32, bits: u32) -> i32 {
    let shift = 32 - bits;
    ((value << shift) as i32) >> shift
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_add() {
        // ADD r1, r2, r3 (7-bit opcode format)
        let mut word = 0u32;
        word |= Opcode::Add.to_u8() as u32;  // opcode (7 bits)
        word |= 1 << 7;              // rd
        word |= 2 << 11;             // rs1
        word |= 3 << 15;             // rs2

        let instr = decode(word).unwrap();
        assert_eq!(
            instr,
            Instruction::Add {
                rd: Register::R1,
                rs1: Register::R2,
                rs2: Register::R3
            }
        );
    }

    #[test]
    fn test_decode_addi() {
        // ADDI r1, r2, 100 (7-bit opcode format)
        let mut word = 0u32;
        word |= Opcode::Addi.to_u8() as u32;  // opcode (7 bits)
        word |= 1 << 7;              // rd
        word |= 2 << 11;             // rs1
        word |= 100 << 15;           // imm (17 bits)

        let instr = decode(word).unwrap();
        assert_eq!(
            instr,
            Instruction::Addi {
                rd: Register::R1,
                rs1: Register::R2,
                imm: 100
            }
        );
    }

    #[test]
    fn test_decode_and() {
        // AND r1, r2, r3 (7-bit opcode format)
        let mut word = 0u32;
        word |= Opcode::And.to_u8() as u32;  // opcode (7 bits) = 0x10
        word |= 1 << 7;              // rd
        word |= 2 << 11;             // rs1
        word |= 3 << 15;             // rs2

        let instr = decode(word).unwrap();
        assert_eq!(
            instr,
            Instruction::And {
                rd: Register::R1,
                rs1: Register::R2,
                rs2: Register::R3
            }
        );
    }

    #[test]
    fn test_decode_lw() {
        // LW r1, 16(r2) (7-bit opcode format)
        let mut word = 0u32;
        word |= Opcode::Lw.to_u8() as u32;  // opcode (7 bits) = 0x34
        word |= 1 << 7;              // rd
        word |= 2 << 11;             // rs1
        word |= 16 << 15;            // imm (17 bits)

        let instr = decode(word).unwrap();
        assert_eq!(
            instr,
            Instruction::Lw {
                rd: Register::R1,
                rs1: Register::R2,
                imm: 16
            }
        );
    }

    #[test]
    fn test_decode_sw() {
        // SW rs2, imm(rs1) (7-bit opcode format)
        let mut word = 0u32;
        word |= Opcode::Sw.to_u8() as u32;  // opcode (7 bits) = 0x3A
        word |= 2 << 7;              // rs1 (base address)
        word |= 1 << 11;             // rs2 (value to store)
        word |= 16 << 15;            // imm (offset, 17 bits)

        let instr = decode(word).unwrap();
        assert_eq!(
            instr,
            Instruction::Sw {
                rs1: Register::R2,
                rs2: Register::R1,
                imm: 16
            }
        );
    }

    #[test]
    fn test_decode_beq() {
        // BEQ r1, r2, 8 (7-bit opcode format)
        let mut word = 0u32;
        word |= Opcode::Beq.to_u8() as u32;  // opcode (7 bits) = 0x40
        word |= 1 << 7;              // rs1
        word |= 2 << 11;             // rs2
        word |= 8 << 15;             // offset (17 bits)

        let instr = decode(word).unwrap();
        assert_eq!(
            instr,
            Instruction::Beq {
                rs1: Register::R1,
                rs2: Register::R2,
                offset: 8
            }
        );
    }

    #[test]
    fn test_decode_jal() {
        // JAL r1, 100 (7-bit opcode format)
        let mut word = 0u32;
        word |= Opcode::Jal.to_u8() as u32;  // opcode (7 bits) = 0x48
        word |= 1 << 7;              // rd
        word |= 100 << 11;           // offset (21 bits)

        let instr = decode(word).unwrap();
        assert_eq!(
            instr,
            Instruction::Jal {
                rd: Register::R1,
                offset: 100
            }
        );
    }

    #[test]
    fn test_decode_ecall() {
        let word = Opcode::Ecall.to_u8() as u32;  // 7-bit opcode = 0x50

        let instr = decode(word).unwrap();
        assert_eq!(instr, Instruction::Ecall);
    }

    #[test]
    fn test_decode_ebreak() {
        let word = Opcode::Ebreak.to_u8() as u32;  // 7-bit opcode = 0x51

        let instr = decode(word).unwrap();
        assert_eq!(instr, Instruction::Ebreak);
    }

    #[test]
    fn test_sign_extend() {
        // Test 17-bit sign extension
        assert_eq!(sign_extend(0x1FFFF, 17), -1);
        assert_eq!(sign_extend(0x00001, 17), 1);
        assert_eq!(sign_extend(0x10000, 17), -65536);
        assert_eq!(sign_extend(0x0FFFF, 17), 65535);

        // Test 21-bit sign extension (offset size for JAL)
        assert_eq!(sign_extend(0x1FFFFF, 21), -1);
        assert_eq!(sign_extend(0x100000, 21), -1048576);
    }

    #[test]
    fn test_decode_negative_immediate() {
        // ADDI r1, r2, -1 (7-bit opcode format)
        let mut word = 0u32;
        word |= Opcode::Addi.to_u8() as u32;  // opcode (7 bits)
        word |= 1 << 7;              // rd
        word |= 2 << 11;             // rs1
        word |= 0x1FFFF << 15;       // imm = -1 in 17-bit two's complement

        let instr = decode(word).unwrap();
        assert_eq!(
            instr,
            Instruction::Addi {
                rd: Register::R1,
                rs1: Register::R2,
                imm: -1
            }
        );
    }

    #[test]
    fn test_decode_slli() {
        // SLLI r1, r2, 5 (7-bit opcode format)
        let mut word = 0u32;
        word |= Opcode::Slli.to_u8() as u32;  // opcode (7 bits) = 0x1B
        word |= 1 << 7;              // rd
        word |= 2 << 11;             // rs1
        word |= 5 << 15;             // shamt (in 17-bit immediate field)

        let instr = decode(word).unwrap();
        assert_eq!(
            instr,
            Instruction::Slli {
                rd: Register::R1,
                rs1: Register::R2,
                shamt: 5
            }
        );
    }

    #[test]
    fn test_opcode_values() {
        // Verify opcodes match ZKIR v3.4 spec
        assert_eq!(Opcode::Add.to_u8(), 0x00);
        assert_eq!(Opcode::Addi.to_u8(), 0x08);
        assert_eq!(Opcode::And.to_u8(), 0x10);
        assert_eq!(Opcode::Sll.to_u8(), 0x18);
        assert_eq!(Opcode::Sltu.to_u8(), 0x20);
        assert_eq!(Opcode::Cmov.to_u8(), 0x26);
        assert_eq!(Opcode::Lb.to_u8(), 0x30);
        assert_eq!(Opcode::Sb.to_u8(), 0x38);
        assert_eq!(Opcode::Beq.to_u8(), 0x40);
        assert_eq!(Opcode::Jal.to_u8(), 0x48);
        assert_eq!(Opcode::Ecall.to_u8(), 0x50);
    }
}
