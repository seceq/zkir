//! Instruction encoding to 32-bit RISC-V format

use zkir_spec::Instruction;

/// Encode instruction to 32-bit word
pub fn encode(instr: &Instruction) -> u32 {
    match instr {
        // System instructions
        Instruction::Ecall => {
            // I-type: opcode=0x73, funct3=0x0, imm=0x000
            0x00000073
        }
        Instruction::Ebreak => {
            // I-type: opcode=0x73, funct3=0x0, imm=0x001
            0x00100073
        }

        // ZK-Custom: HALT (opcode=0x0B, funct7=0x7F, funct3=0x7)
        Instruction::Halt => {
            0xFE0003B // Placeholder encoding
        }

        // TODO: Implement all instruction encodings
        _ => 0x00000013, // NOP (ADDI zero, zero, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_ecall() {
        assert_eq!(encode(&Instruction::Ecall), 0x00000073);
    }

    #[test]
    fn test_encode_ebreak() {
        assert_eq!(encode(&Instruction::Ebreak), 0x00100073);
    }
}
