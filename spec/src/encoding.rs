//! Encoding utilities for ZK IR bytecode.

use crate::instruction::Instruction;
use crate::error::ZkIrError;

/// Instruction encoder/decoder utilities
pub struct Encoder;

impl Encoder {
    /// Encode a slice of instructions to bytes
    pub fn encode_instructions(instructions: &[Instruction]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(instructions.len() * 8);
        for instr in instructions {
            bytes.extend_from_slice(&instr.encode().to_le_bytes());
        }
        bytes
    }

    /// Decode instructions from bytes
    pub fn decode_instructions(bytes: &[u8]) -> Result<Vec<Instruction>, ZkIrError> {
        if bytes.len() % 8 != 0 {
            return Err(ZkIrError::InvalidFormat(
                "Instruction bytes must be multiple of 8".into(),
            ));
        }

        let mut instructions = Vec::with_capacity(bytes.len() / 8);
        for chunk in bytes.chunks_exact(8) {
            let encoded = u64::from_le_bytes(chunk.try_into().unwrap());
            instructions.push(Instruction::decode(encoded)?);
        }
        Ok(instructions)
    }

    /// Calculate size of encoded instructions in bytes
    pub fn instructions_size(count: usize) -> usize {
        count * 8
    }
}

/// Encode a 32-bit immediate value
pub fn encode_imm32(value: i32) -> u32 {
    value as u32
}

/// Decode a 32-bit immediate value
pub fn decode_imm32(encoded: u32) -> i32 {
    encoded as i32
}

/// Encode a branch offset (signed)
pub fn encode_branch_offset(offset: i32) -> u32 {
    offset as u32
}

/// Decode a branch offset (signed)
pub fn decode_branch_offset(encoded: u32) -> i32 {
    encoded as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::register::Register;

    #[test]
    fn test_encode_decode_instructions() {
        let instructions = vec![
            Instruction::Add {
                dst: Register::R1,
                src1: Register::R2,
                src2: Register::R3,
            },
            Instruction::Li {
                dst: Register::R4,
                imm: 100,
            },
            Instruction::Halt,
        ];

        let bytes = Encoder::encode_instructions(&instructions);
        let decoded = Encoder::decode_instructions(&bytes).unwrap();

        assert_eq!(instructions, decoded);
    }

    #[test]
    fn test_imm32_roundtrip() {
        let values = [0i32, 1, -1, i32::MAX, i32::MIN, 12345, -12345];
        for value in values {
            let encoded = encode_imm32(value);
            let decoded = decode_imm32(encoded);
            assert_eq!(value, decoded);
        }
    }
}
