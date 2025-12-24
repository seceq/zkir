//! Instruction decoder for ZK IR v2.2 (30-bit encoding)

use zkir_spec::{Instruction, Register};
use crate::error::{DisassemblerError, Result};

/// Decode 32-bit word containing 30-bit instruction
pub fn decode(word: u32) -> Result<Instruction> {
    // Validate bits 31:30 are zero
    if word & 0xC0000000 != 0 {
        return Err(DisassemblerError::InvalidEncoding(word));
    }

    let opcode = (word & 0xF) as u8;

    match opcode {
        0b0000 => decode_alu(word),        // R-type ALU
        0b0001 => decode_alui(word),       // I-type immediate
        0b0010 => decode_load(word),       // Load
        0b0011 => decode_store(word),      // Store
        0b0100 => decode_beq(word),        // BEQ
        0b0101 => decode_bne(word),        // BNE
        0b0110 => decode_blt(word),        // BLT
        0b0111 => decode_bge(word),        // BGE
        0b1000 => decode_bltu(word),       // BLTU
        0b1001 => decode_bgeu(word),       // BGEU
        0b1010 => decode_lui(word),        // LUI
        0b1011 => decode_auipc(word),      // AUIPC
        0b1100 => decode_jal(word),        // JAL
        0b1101 => decode_jalr(word),       // JALR
        0b1110 => decode_zkop(word),       // ZK operations
        0b1111 => decode_system(word),     // System
        _ => Err(DisassemblerError::UnknownOpcode(opcode)),
    }
}

/// Decode R-type ALU (opcode = 0000)
fn decode_alu(word: u32) -> Result<Instruction> {
    let rd = decode_register((word >> 4) & 0x1F)?;
    let rs1 = decode_register((word >> 9) & 0x1F)?;
    let rs2 = decode_register((word >> 14) & 0x1F)?;
    let funct = (word >> 19) & 0x7;
    let ext = (word >> 22) & 0x3;
    let ext2 = (word >> 24) & 0x3F;

    // Check for field operations (bits 29:24 = 111111)
    if ext2 == 0b111111 {
        return match funct {
            0b000 => Ok(Instruction::Fadd { rd, rs1, rs2 }),
            0b001 => Ok(Instruction::Fsub { rd, rs1, rs2 }),
            0b010 => Ok(Instruction::Fmul { rd, rs1, rs2 }),
            0b011 => Ok(Instruction::Fneg { rd, rs1, rs2 }),
            0b100 => Ok(Instruction::Finv { rd, rs1, rs2 }),
            _ => Err(DisassemblerError::InvalidEncoding(word)),
        };
    }

    // Check for ext2 instructions (ext=11, funct=111)
    if ext == 0b11 && funct == 0b111 {
        return match ext2 {
            0b000000 => Ok(Instruction::Cmovz { rd, rs1, rs2 }),
            0b000001 => Ok(Instruction::Cmovnz { rd, rs1, rs2 }),
            _ => Err(DisassemblerError::InvalidEncoding(word)),
        };
    }

    // Standard R-type decoding
    match (funct, ext) {
        // Arithmetic
        (0b000, 0b00) => Ok(Instruction::Add { rd, rs1, rs2 }),
        (0b000, 0b01) => Ok(Instruction::Sub { rd, rs1, rs2 }),
        (0b000, 0b10) => Ok(Instruction::Mul { rd, rs1, rs2 }),
        (0b000, 0b11) => Ok(Instruction::Mulh { rd, rs1, rs2 }),

        // Logic
        (0b001, 0b00) => Ok(Instruction::And { rd, rs1, rs2 }),
        (0b001, 0b01) => Ok(Instruction::Andn { rd, rs1, rs2 }),
        (0b001, 0b10) => Ok(Instruction::Or { rd, rs1, rs2 }),
        (0b001, 0b11) => Ok(Instruction::Orn { rd, rs1, rs2 }),

        // Shift
        (0b010, 0b00) => Ok(Instruction::Xor { rd, rs1, rs2 }),
        (0b010, 0b01) => Ok(Instruction::Xnor { rd, rs1, rs2 }),
        (0b010, 0b10) => Ok(Instruction::Sll { rd, rs1, rs2 }),
        (0b010, 0b11) => Ok(Instruction::Rol { rd, rs1, rs2 }),

        (0b011, 0b00) => Ok(Instruction::Srl { rd, rs1, rs2 }),
        (0b011, 0b01) => Ok(Instruction::Sra { rd, rs1, rs2 }),
        (0b011, 0b10) => Ok(Instruction::Ror { rd, rs1, rs2 }),
        (0b011, 0b11) => Ok(Instruction::Clz { rd, rs1, rs2 }),

        // Compare
        (0b100, 0b00) => Ok(Instruction::Slt { rd, rs1, rs2 }),
        (0b100, 0b01) => Ok(Instruction::Sltu { rd, rs1, rs2 }),
        (0b100, 0b10) => Ok(Instruction::Min { rd, rs1, rs2 }),
        (0b100, 0b11) => Ok(Instruction::Max { rd, rs1, rs2 }),

        (0b101, 0b00) => Ok(Instruction::Minu { rd, rs1, rs2 }),
        (0b101, 0b01) => Ok(Instruction::Maxu { rd, rs1, rs2 }),
        (0b101, 0b10) => Ok(Instruction::Mulhu { rd, rs1, rs2 }),
        (0b101, 0b11) => Ok(Instruction::Mulhsu { rd, rs1, rs2 }),

        // Division
        (0b110, 0b00) => Ok(Instruction::Div { rd, rs1, rs2 }),
        (0b110, 0b01) => Ok(Instruction::Divu { rd, rs1, rs2 }),
        (0b110, 0b10) => Ok(Instruction::Rem { rd, rs1, rs2 }),
        (0b110, 0b11) => Ok(Instruction::Remu { rd, rs1, rs2 }),

        // Bit manipulation
        (0b111, 0b00) => Ok(Instruction::Rev8 { rd, rs1, rs2 }),
        (0b111, 0b01) => Ok(Instruction::Cpop { rd, rs1, rs2 }),
        (0b111, 0b10) => Ok(Instruction::Ctz { rd, rs1, rs2 }),

        _ => Err(DisassemblerError::InvalidEncoding(word)),
    }
}

/// Decode I-type immediate (opcode = 0001)
fn decode_alui(word: u32) -> Result<Instruction> {
    let rd = decode_register((word >> 4) & 0x1F)?;
    let rs1 = decode_register((word >> 9) & 0x1F)?;
    let imm_raw = (word >> 14) & 0x1FFF;
    let funct = (word >> 27) & 0x7;

    // Sign-extend 13-bit immediate
    let imm = sign_extend(imm_raw, 13) as i16;

    match funct {
        0b000 => Ok(Instruction::Addi { rd, rs1, imm }),
        0b001 => Ok(Instruction::Slli { rd, rs1, shamt: (imm_raw & 0x1F) as u8 }),
        0b010 => Ok(Instruction::Slti { rd, rs1, imm }),
        0b011 => Ok(Instruction::Sltiu { rd, rs1, imm }),
        0b100 => Ok(Instruction::Xori { rd, rs1, imm }),
        0b101 => {
            // SRLI or SRAI based on bit 12
            if (imm_raw >> 12) & 1 == 1 {
                Ok(Instruction::Srai { rd, rs1, shamt: (imm_raw & 0x1F) as u8 })
            } else {
                Ok(Instruction::Srli { rd, rs1, shamt: (imm_raw & 0x1F) as u8 })
            }
        }
        0b110 => Ok(Instruction::Ori { rd, rs1, imm }),
        0b111 => Ok(Instruction::Andi { rd, rs1, imm }),
        _ => Err(DisassemblerError::InvalidEncoding(word)),
    }
}

/// Decode Load (opcode = 0010)
fn decode_load(word: u32) -> Result<Instruction> {
    let rd = decode_register((word >> 4) & 0x1F)?;
    let rs1 = decode_register((word >> 9) & 0x1F)?;
    let imm_raw = (word >> 14) & 0x1FFF;
    let funct = (word >> 27) & 0x7;

    let imm = sign_extend(imm_raw, 13) as i16;

    match funct {
        0b000 => Ok(Instruction::Lb { rd, rs1, imm }),
        0b001 => Ok(Instruction::Lh { rd, rs1, imm }),
        0b010 => Ok(Instruction::Lw { rd, rs1, imm }),
        0b100 => Ok(Instruction::Lbu { rd, rs1, imm }),
        0b101 => Ok(Instruction::Lhu { rd, rs1, imm }),
        _ => Err(DisassemblerError::InvalidEncoding(word)),
    }
}

/// Decode Store (opcode = 0011)
fn decode_store(word: u32) -> Result<Instruction> {
    let funct = (word >> 4) & 0x7;
    let rs1 = decode_register((word >> 7) & 0x1F)?;
    let rs2 = decode_register((word >> 12) & 0x1F)?;
    let imm_raw = (word >> 17) & 0x1FFF;

    let imm = sign_extend(imm_raw, 13) as i16;

    match funct {
        0b000 => Ok(Instruction::Sb { rs1, rs2, imm }),
        0b001 => Ok(Instruction::Sh { rs1, rs2, imm }),
        0b010 => Ok(Instruction::Sw { rs1, rs2, imm }),
        _ => Err(DisassemblerError::InvalidEncoding(word)),
    }
}

/// Decode branch instructions
fn decode_branch(word: u32) -> Result<(Register, Register, i16)> {
    let rs2 = decode_register((word >> 4) & 0x1F)?;
    let rs1 = decode_register((word >> 9) & 0x1F)?;
    let imm_raw = (word >> 14) & 0xFFFF;
    let imm = sign_extend(imm_raw, 16) as i16;
    Ok((rs1, rs2, imm))
}

fn decode_beq(word: u32) -> Result<Instruction> {
    let (rs1, rs2, imm) = decode_branch(word)?;
    Ok(Instruction::Beq { rs1, rs2, imm })
}

fn decode_bne(word: u32) -> Result<Instruction> {
    let (rs1, rs2, imm) = decode_branch(word)?;
    Ok(Instruction::Bne { rs1, rs2, imm })
}

fn decode_blt(word: u32) -> Result<Instruction> {
    let (rs1, rs2, imm) = decode_branch(word)?;
    Ok(Instruction::Blt { rs1, rs2, imm })
}

fn decode_bge(word: u32) -> Result<Instruction> {
    let (rs1, rs2, imm) = decode_branch(word)?;
    Ok(Instruction::Bge { rs1, rs2, imm })
}

fn decode_bltu(word: u32) -> Result<Instruction> {
    let (rs1, rs2, imm) = decode_branch(word)?;
    Ok(Instruction::Bltu { rs1, rs2, imm })
}

fn decode_bgeu(word: u32) -> Result<Instruction> {
    let (rs1, rs2, imm) = decode_branch(word)?;
    Ok(Instruction::Bgeu { rs1, rs2, imm })
}

/// Decode LUI (opcode = 1010)
fn decode_lui(word: u32) -> Result<Instruction> {
    let rd = decode_register((word >> 4) & 0x1F)?;
    let imm_raw = (word >> 9) & 0x1FFFFF;
    let imm = sign_extend(imm_raw, 21) as i32;
    Ok(Instruction::Lui { rd, imm })
}

/// Decode AUIPC (opcode = 1011)
fn decode_auipc(word: u32) -> Result<Instruction> {
    let rd = decode_register((word >> 4) & 0x1F)?;
    let imm_raw = (word >> 9) & 0x1FFFFF;
    let imm = sign_extend(imm_raw, 21) as i32;
    Ok(Instruction::Auipc { rd, imm })
}

/// Decode JAL (opcode = 1100)
fn decode_jal(word: u32) -> Result<Instruction> {
    let rd = decode_register((word >> 4) & 0x1F)?;
    let imm_raw = (word >> 9) & 0x1FFFFF;
    let imm = sign_extend(imm_raw, 21) as i32;
    Ok(Instruction::Jal { rd, imm })
}

/// Decode JALR (opcode = 1101)
fn decode_jalr(word: u32) -> Result<Instruction> {
    let rd = decode_register((word >> 4) & 0x1F)?;
    let rs1 = decode_register((word >> 9) & 0x1F)?;
    let imm_raw = (word >> 14) & 0x1FFF;
    let imm = sign_extend(imm_raw, 13) as i16;
    Ok(Instruction::Jalr { rd, rs1, imm })
}

/// Decode ZK operations (opcode = 1110)
fn decode_zkop(word: u32) -> Result<Instruction> {
    let rd = decode_register((word >> 7) & 0x1F)?;
    let rs1 = decode_register((word >> 12) & 0x1F)?;
    let imm = ((word >> 17) & 0xFF) as u8;
    let func = (word >> 25) & 0x1F;

    match func {
        0b00000 => Ok(Instruction::Read { rd }),
        0b00001 => Ok(Instruction::Write { rs1 }),
        0b00010 => Ok(Instruction::Hint { rd }),
        0b00011 => Ok(Instruction::Commit { rs1 }),
        0b00100 => {
            // ASSERT_EQ uses rs2 in rd position
            let rs2 = rd;
            Ok(Instruction::AssertEq { rs1, rs2 })
        }
        0b00101 => {
            // ASSERT_NE uses rs2 in rd position
            let rs2 = rd;
            Ok(Instruction::AssertNe { rs1, rs2 })
        }
        0b00110 => Ok(Instruction::AssertZero { rs1 }),
        0b00111 => Ok(Instruction::RangeCheck { rs1, bits: imm }),
        0b01000 => Ok(Instruction::Debug { rs1 }),
        0b11111 => Ok(Instruction::Halt),
        _ => Err(DisassemblerError::InvalidEncoding(word)),
    }
}

/// Decode System (opcode = 1111)
fn decode_system(word: u32) -> Result<Instruction> {
    let imm = (word >> 14) & 0x1FFF;

    match imm {
        0 => Ok(Instruction::Ecall),
        1 => Ok(Instruction::Ebreak),
        _ => Err(DisassemblerError::InvalidEncoding(word)),
    }
}

/// Decode register from 5-bit index
fn decode_register(index: u32) -> Result<Register> {
    Register::from_index(index as usize)
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
        // ADD r5, r6, r7
        // opcode=0000, rd=5, rs1=6, rs2=7, funct=000, ext=00
        let word = 0b0000_00_000_00111_00110_00101_0000;

        let instr = decode(word).unwrap();
        assert_eq!(
            instr,
            Instruction::Add {
                rd: Register::R5,
                rs1: Register::R6,
                rs2: Register::R7
            }
        );
    }

    #[test]
    fn test_decode_addi() {
        // ADDI r5, r6, 100
        // opcode=0001, rd=5, rs1=6, imm=100, funct=000
        let word = 0b000_0000001100100_00110_00101_0001;

        let instr = decode(word).unwrap();
        assert_eq!(
            instr,
            Instruction::Addi {
                rd: Register::R5,
                rs1: Register::R6,
                imm: 100
            }
        );
    }

    #[test]
    fn test_decode_fadd() {
        // FADD r5, r6, r7
        // opcode=0000, rd=5, rs1=6, rs2=7, funct=000, ext=00, marker=111111
        let word = 0b111111_00_000_00111_00110_00101_0000;

        let instr = decode(word).unwrap();
        assert_eq!(
            instr,
            Instruction::Fadd {
                rd: Register::R5,
                rs1: Register::R6,
                rs2: Register::R7
            }
        );
    }

    #[test]
    fn test_decode_ecall() {
        // ECALL: opcode=1111, imm=0
        let word = 0b000_0000000000000_00000_00000_1111;

        let instr = decode(word).unwrap();
        assert_eq!(instr, Instruction::Ecall);
    }

    #[test]
    fn test_decode_halt() {
        // HALT: opcode=1110, func=11111, rd=0, rs1=0, imm=0
        // Z-type format: | func(5) | imm(8) | rs1(5) | rd(5) | opcode(4) |
        let word = 0x3E00000E; // 0b111110000000000000000000001110

        let instr = decode(word).unwrap();
        assert_eq!(instr, Instruction::Halt);
    }

    #[test]
    fn test_invalid_bits_31_30() {
        // Word with bits 31:30 set should fail
        let word = 0x80000000;
        assert!(decode(word).is_err());
    }

    #[test]
    fn test_sign_extend() {
        assert_eq!(sign_extend(0b1111111111111, 13), -1);
        assert_eq!(sign_extend(0b0000000000001, 13), 1);
        assert_eq!(sign_extend(0b1000000000000, 13), -4096);
        assert_eq!(sign_extend(0b0111111111111, 13), 4095);
    }

    #[test]
    fn test_decode_lw() {
        // LW r5, 100(r6)
        // opcode=0010, rd=5, rs1=6, imm=100, funct=010
        let word = 0b010_0000001100100_00110_00101_0010;

        let instr = decode(word).unwrap();
        assert_eq!(
            instr,
            Instruction::Lw {
                rd: Register::R5,
                rs1: Register::R6,
                imm: 100
            }
        );
    }

    #[test]
    fn test_decode_beq() {
        // BEQ r5, r6, 10
        // opcode=0100, rs1=5, rs2=6, imm=10
        let word = 0b0000000000001010_00101_00110_0100;

        let instr = decode(word).unwrap();
        assert_eq!(
            instr,
            Instruction::Beq {
                rs1: Register::R5,
                rs2: Register::R6,
                imm: 10
            }
        );
    }
}
