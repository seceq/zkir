//! Assembly parser

use zkir_spec::{Instruction, Register};
use crate::error::{AssemblerError, Result};

/// Parse a single instruction from assembly text
pub fn parse_instruction(text: &str) -> Result<Instruction> {
    let text = text.trim();

    // Split into mnemonic and operands
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.is_empty() {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: "Empty instruction".to_string(),
        });
    }

    let mnemonic = parts[0].to_lowercase();
    let operands = if parts.len() > 1 {
        parts[1..].join(" ")
    } else {
        String::new()
    };

    parse_mnemonic(&mnemonic, &operands)
}

/// Parse register name
pub fn parse_register(name: &str) -> Result<Register> {
    let name = name.trim().to_lowercase();

    match name.as_str() {
        "zero" | "r0" => Ok(Register::R0),
        "rv" | "r1" => Ok(Register::R1),
        "sp" | "r2" => Ok(Register::R2),
        "fp" | "r3" => Ok(Register::R3),
        "a0" | "r4" => Ok(Register::R4),
        "a1" | "r5" => Ok(Register::R5),
        "a2" | "r6" => Ok(Register::R6),
        "a3" | "r7" => Ok(Register::R7),
        "t0" | "r8" => Ok(Register::R8),
        "t1" | "r9" => Ok(Register::R9),
        "t2" | "r10" => Ok(Register::R10),
        "t3" | "r11" => Ok(Register::R11),
        "t4" | "r12" => Ok(Register::R12),
        "t5" | "r13" => Ok(Register::R13),
        "t6" | "r14" => Ok(Register::R14),
        "t7" | "r15" => Ok(Register::R15),
        "s0" | "r16" => Ok(Register::R16),
        "s1" | "r17" => Ok(Register::R17),
        "s2" | "r18" => Ok(Register::R18),
        "s3" | "r19" => Ok(Register::R19),
        "s4" | "r20" => Ok(Register::R20),
        "s5" | "r21" => Ok(Register::R21),
        "s6" | "r22" => Ok(Register::R22),
        "s7" | "r23" => Ok(Register::R23),
        "t8" | "r24" => Ok(Register::R24),
        "t9" | "r25" => Ok(Register::R25),
        "t10" | "r26" => Ok(Register::R26),
        "t11" | "r27" => Ok(Register::R27),
        "gp" | "r28" => Ok(Register::R28),
        "tp" | "r29" => Ok(Register::R29),
        "ra" | "r30" => Ok(Register::R30),
        "r31" => Ok(Register::R31),
        _ => Err(AssemblerError::InvalidRegister(name.to_string())),
    }
}

/// Parse single register operand
fn parse_single_register(operands: &str) -> Result<Register> {
    let operands = operands.trim();
    if operands.is_empty() {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: "Expected register operand".to_string(),
        });
    }
    parse_register(operands)
}

/// Parse two register operands: rs1, rs2
fn parse_two_registers(operands: &str) -> Result<(Register, Register)> {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: "Expected 2 register operands".to_string(),
        });
    }

    let rs1 = parse_register(parts[0])?;
    let rs2 = parse_register(parts[1])?;
    Ok((rs1, rs2))
}

/// Parse register and immediate: rs1, imm
fn parse_register_immediate(operands: &str) -> Result<(Register, u8)> {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: "Expected register and immediate".to_string(),
        });
    }

    let rs1 = parse_register(parts[0])?;
    let imm_str = parts[1].trim();

    let imm = if imm_str.starts_with("0x") || imm_str.starts_with("0X") {
        u8::from_str_radix(&imm_str[2..], 16)
            .map_err(|_| AssemblerError::SyntaxError {
                line: 0,
                column: 0,
                message: format!("Invalid immediate value: {}", imm_str),
            })?
    } else {
        imm_str.parse::<u8>()
            .map_err(|_| AssemblerError::SyntaxError {
                line: 0,
                column: 0,
                message: format!("Invalid immediate value: {}", imm_str),
            })?
    };

    Ok((rs1, imm))
}

fn parse_mnemonic(mnemonic: &str, operands: &str) -> Result<Instruction> {
    match mnemonic {
        // System
        "halt" => Ok(Instruction::Halt),
        "ecall" => Ok(Instruction::Ecall),
        "ebreak" => Ok(Instruction::Ebreak),

        // ZK I/O
        "read" => {
            let rd = parse_single_register(operands)?;
            Ok(Instruction::Read { rd })
        }
        "write" => {
            let rs1 = parse_single_register(operands)?;
            Ok(Instruction::Write { rs1 })
        }
        "hint" => {
            let rd = parse_single_register(operands)?;
            Ok(Instruction::Hint { rd })
        }

        // ZK Custom
        "assert_eq" => {
            let (rs1, rs2) = parse_two_registers(operands)?;
            Ok(Instruction::AssertEq { rs1, rs2 })
        }
        "assert_ne" => {
            let (rs1, rs2) = parse_two_registers(operands)?;
            Ok(Instruction::AssertNe { rs1, rs2 })
        }
        "assert_zero" => {
            let rs1 = parse_single_register(operands)?;
            Ok(Instruction::AssertZero { rs1 })
        }
        "range_check" => {
            let (rs1, bits) = parse_register_immediate(operands)?;
            Ok(Instruction::RangeCheck { rs1, bits })
        }
        "commit" => {
            let rs1 = parse_single_register(operands)?;
            Ok(Instruction::Commit { rs1 })
        }

        _ => Err(AssemblerError::UnknownInstruction(mnemonic.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_register() {
        assert_eq!(parse_register("zero").unwrap(), Register::R0);
        assert_eq!(parse_register("r0").unwrap(), Register::R0);
        assert_eq!(parse_register("sp").unwrap(), Register::R2);
        assert_eq!(parse_register("a0").unwrap(), Register::R4);
        assert_eq!(parse_register("t0").unwrap(), Register::R8);
        assert_eq!(parse_register("s0").unwrap(), Register::R16);
        assert_eq!(parse_register("ra").unwrap(), Register::R30);
    }

    #[test]
    fn test_parse_halt() {
        let instr = parse_instruction("halt").unwrap();
        assert_eq!(instr, Instruction::Halt);
    }

    #[test]
    fn test_parse_ecall() {
        let instr = parse_instruction("ecall").unwrap();
        assert_eq!(instr, Instruction::Ecall);
    }

    #[test]
    fn test_parse_ebreak() {
        let instr = parse_instruction("ebreak").unwrap();
        assert_eq!(instr, Instruction::Ebreak);
    }

    // ZK I/O tests
    #[test]
    fn test_parse_read() {
        let instr = parse_instruction("read a0").unwrap();
        assert_eq!(instr, Instruction::Read { rd: Register::R4 });
    }

    #[test]
    fn test_parse_write() {
        let instr = parse_instruction("write a1").unwrap();
        assert_eq!(instr, Instruction::Write { rs1: Register::R5 });
    }

    #[test]
    fn test_parse_hint() {
        let instr = parse_instruction("hint t0").unwrap();
        assert_eq!(instr, Instruction::Hint { rd: Register::R8 });
    }

    // ZK Custom tests
    #[test]
    fn test_parse_assert_eq() {
        let instr = parse_instruction("assert_eq a0, a1").unwrap();
        assert_eq!(instr, Instruction::AssertEq {
            rs1: Register::R4,
            rs2: Register::R5,
        });
    }

    #[test]
    fn test_parse_assert_ne() {
        let instr = parse_instruction("assert_ne t0, t1").unwrap();
        assert_eq!(instr, Instruction::AssertNe {
            rs1: Register::R8,
            rs2: Register::R9,
        });
    }

    #[test]
    fn test_parse_assert_zero() {
        let instr = parse_instruction("assert_zero a2").unwrap();
        assert_eq!(instr, Instruction::AssertZero { rs1: Register::R6 });
    }

    #[test]
    fn test_parse_range_check() {
        let instr = parse_instruction("range_check a0, 32").unwrap();
        assert_eq!(instr, Instruction::RangeCheck {
            rs1: Register::R4,
            bits: 32,
        });
    }

    #[test]
    fn test_parse_range_check_hex() {
        let instr = parse_instruction("range_check a0, 0x10").unwrap();
        assert_eq!(instr, Instruction::RangeCheck {
            rs1: Register::R4,
            bits: 16,
        });
    }

    #[test]
    fn test_parse_commit() {
        let instr = parse_instruction("commit s0").unwrap();
        assert_eq!(instr, Instruction::Commit { rs1: Register::R16 });
    }

    // Error cases
    #[test]
    fn test_parse_read_no_operand() {
        let result = parse_instruction("read");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_assert_eq_one_operand() {
        let result = parse_instruction("assert_eq a0");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_range_check_no_immediate() {
        let result = parse_instruction("range_check a0");
        assert!(result.is_err());
    }
}
