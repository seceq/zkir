//! Instruction decoder

use zkir_spec::{Instruction, Register};
use crate::error::{DisassemblerError, Result};

/// Decode 32-bit instruction word
pub fn decode(word: u32) -> Result<Instruction> {
    let opcode = (word & 0x7F) as u8;

    match opcode {
        0x73 => decode_system(word),
        0x0B => decode_zk_custom(word),
        0x5B => decode_zk_io(word),
        0x33 => decode_r_type(word),
        0x13 => decode_i_type_alu(word),
        0x03 => decode_i_type_load(word),
        0x23 => decode_s_type(word),
        0x63 => decode_b_type(word),
        0x6F => decode_j_type(word),
        0x67 => decode_jalr(word),
        0x37 => decode_lui(word),
        0x17 => decode_auipc(word),
        _ => Err(DisassemblerError::UnknownOpcode(opcode)),
    }
}

fn decode_system(word: u32) -> Result<Instruction> {
    let funct3 = (word >> 12) & 0x7;
    let imm = (word >> 20) & 0xFFF;

    match (funct3, imm) {
        (0x0, 0x000) => Ok(Instruction::Ecall),
        (0x0, 0x001) => Ok(Instruction::Ebreak),
        _ => Err(DisassemblerError::InvalidEncoding(word)),
    }
}

fn decode_zk_custom(word: u32) -> Result<Instruction> {
    let funct7 = (word >> 25) & 0x7F;
    let funct3 = (word >> 12) & 0x7;

    match (funct7, funct3) {
        (0x7F, 0x7) => Ok(Instruction::Halt),
        _ => {
            let rs1 = decode_register((word >> 15) & 0x1F)?;
            let rs2 = decode_register((word >> 20) & 0x1F)?;

            match (funct7, funct3) {
                (0x00, 0x0) => Ok(Instruction::AssertEq { rs1, rs2 }),
                (0x00, 0x1) => Ok(Instruction::AssertNe { rs1, rs2 }),
                (0x01, 0x0) => Ok(Instruction::AssertZero { rs1 }),
                (0x10, 0x0) => Ok(Instruction::Commit { rs1 }),
                _ => Err(DisassemblerError::InvalidEncoding(word)),
            }
        }
    }
}

fn decode_zk_io(word: u32) -> Result<Instruction> {
    let rd = decode_register((word >> 7) & 0x1F)?;
    let rs1 = decode_register((word >> 15) & 0x1F)?;
    let funct3 = (word >> 12) & 0x7;

    match funct3 {
        0x0 => Ok(Instruction::Read { rd }),
        0x1 => Ok(Instruction::Write { rs1 }),
        0x2 => Ok(Instruction::Hint { rd }),
        _ => Err(DisassemblerError::InvalidEncoding(word)),
    }
}

fn decode_r_type(_word: u32) -> Result<Instruction> {
    // TODO: Implement R-type decoding
    Err(DisassemblerError::InvalidEncoding(_word))
}

fn decode_i_type_alu(_word: u32) -> Result<Instruction> {
    // TODO: Implement I-type ALU decoding
    Err(DisassemblerError::InvalidEncoding(_word))
}

fn decode_i_type_load(_word: u32) -> Result<Instruction> {
    // TODO: Implement I-type load decoding
    Err(DisassemblerError::InvalidEncoding(_word))
}

fn decode_s_type(_word: u32) -> Result<Instruction> {
    // TODO: Implement S-type decoding
    Err(DisassemblerError::InvalidEncoding(_word))
}

fn decode_b_type(_word: u32) -> Result<Instruction> {
    // TODO: Implement B-type decoding
    Err(DisassemblerError::InvalidEncoding(_word))
}

fn decode_j_type(_word: u32) -> Result<Instruction> {
    // TODO: Implement J-type decoding
    Err(DisassemblerError::InvalidEncoding(_word))
}

fn decode_jalr(_word: u32) -> Result<Instruction> {
    // TODO: Implement JALR decoding
    Err(DisassemblerError::InvalidEncoding(_word))
}

fn decode_lui(_word: u32) -> Result<Instruction> {
    // TODO: Implement LUI decoding
    Err(DisassemblerError::InvalidEncoding(_word))
}

fn decode_auipc(_word: u32) -> Result<Instruction> {
    // TODO: Implement AUIPC decoding
    Err(DisassemblerError::InvalidEncoding(_word))
}

fn decode_register(index: u32) -> Result<Register> {
    Register::from_index(index as usize)
        .ok_or(DisassemblerError::InvalidEncoding(index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_ecall() {
        let instr = decode(0x00000073).unwrap();
        assert_eq!(instr, Instruction::Ecall);
    }

    #[test]
    fn test_decode_ebreak() {
        let instr = decode(0x00100073).unwrap();
        assert_eq!(instr, Instruction::Ebreak);
    }
}
