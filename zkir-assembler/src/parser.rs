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

/// Parse immediate value (handles decimal, hex, binary)
fn parse_immediate(text: &str) -> Result<i32> {
    let text = text.trim();

    if text.starts_with("0x") || text.starts_with("0X") {
        i32::from_str_radix(&text[2..], 16)
            .map_err(|_| AssemblerError::SyntaxError {
                line: 0,
                column: 0,
                message: format!("Invalid hex immediate: {}", text),
            })
    } else if text.starts_with("0b") || text.starts_with("0B") {
        i32::from_str_radix(&text[2..], 2)
            .map_err(|_| AssemblerError::SyntaxError {
                line: 0,
                column: 0,
                message: format!("Invalid binary immediate: {}", text),
            })
    } else {
        text.parse::<i32>()
            .map_err(|_| AssemblerError::SyntaxError {
                line: 0,
                column: 0,
                message: format!("Invalid immediate: {}", text),
            })
    }
}

/// Parse R-type instruction: mnemonic rd, rs1, rs2
fn parse_r_type(mnemonic: &str, operands: &str) -> Result<(Register, Register, Register)> {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() != 3 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 3 operands: rd, rs1, rs2", mnemonic),
        });
    }

    let rd = parse_register(parts[0])?;
    let rs1 = parse_register(parts[1])?;
    let rs2 = parse_register(parts[2])?;

    Ok((rd, rs1, rs2))
}

/// Parse I-type instruction: mnemonic rd, rs1, imm
fn parse_i_type(mnemonic: &str, operands: &str) -> Result<(Register, Register, i16)> {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() != 3 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 3 operands: rd, rs1, imm", mnemonic),
        });
    }

    let rd = parse_register(parts[0])?;
    let rs1 = parse_register(parts[1])?;
    let imm_val = parse_immediate(parts[2])?;

    if imm_val < -2048 || imm_val > 2047 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Immediate value {} out of range (must be -2048 to 2047)", imm_val),
        });
    }

    let imm = imm_val as i16;

    Ok((rd, rs1, imm))
}

/// Parse memory operand format: offset(register)
fn parse_memory_operand(mem_operand: &str) -> Result<(Register, i16)> {
    if let Some(paren_pos) = mem_operand.find('(') {
        let offset_str = &mem_operand[..paren_pos];
        let reg_str = &mem_operand[paren_pos+1..];

        if !reg_str.ends_with(')') {
            return Err(AssemblerError::SyntaxError {
                line: 0,
                column: 0,
                message: format!("Missing closing parenthesis in: {}", mem_operand),
            });
        }

        let reg_str = &reg_str[..reg_str.len()-1];
        let imm = if offset_str.is_empty() {
            0
        } else {
            let imm_val = parse_immediate(offset_str)?;
            if imm_val < -2048 || imm_val > 2047 {
                return Err(AssemblerError::SyntaxError {
                    line: 0,
                    column: 0,
                    message: format!("Offset value {} out of range (must be -2048 to 2047)", imm_val),
                });
            }
            imm_val as i16
        };
        let rs1 = parse_register(reg_str)?;

        Ok((rs1, imm))
    } else {
        Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Invalid memory operand: {}", mem_operand),
        })
    }
}

/// Parse I-type load instruction: mnemonic rd, offset(rs1)
fn parse_load(mnemonic: &str, operands: &str) -> Result<(Register, Register, i16)> {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 2 operands: rd, offset(rs1)", mnemonic),
        });
    }

    let rd = parse_register(parts[0])?;
    let (rs1, imm) = parse_memory_operand(parts[1])?;

    Ok((rd, rs1, imm))
}

/// Parse S-type store instruction: mnemonic rs2, offset(rs1)
fn parse_store(mnemonic: &str, operands: &str) -> Result<(Register, Register, i16)> {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 2 operands: rs2, offset(rs1)", mnemonic),
        });
    }

    let rs2 = parse_register(parts[0])?;
    let (rs1, imm) = parse_memory_operand(parts[1])?;

    Ok((rs1, rs2, imm))
}

/// Parse B-type branch instruction: mnemonic rs1, rs2, offset
fn parse_b_type(mnemonic: &str, operands: &str) -> Result<(Register, Register, i16)> {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() != 3 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 3 operands: rs1, rs2, offset", mnemonic),
        });
    }

    let rs1 = parse_register(parts[0])?;
    let rs2 = parse_register(parts[1])?;
    let imm_val = parse_immediate(parts[2])?;

    if imm_val < -4096 || imm_val > 4094 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Branch offset {} out of range (must be -4096 to 4094)", imm_val),
        });
    }

    let imm = imm_val as i16;

    Ok((rs1, rs2, imm))
}

/// Parse U-type instruction: mnemonic rd, imm
fn parse_u_type(mnemonic: &str, operands: &str) -> Result<(Register, i32)> {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 2 operands: rd, imm", mnemonic),
        });
    }

    let rd = parse_register(parts[0])?;
    let imm = parse_immediate(parts[1])?;

    Ok((rd, imm))
}

/// Parse shift amount (must be 0-31)
fn parse_shamt(text: &str) -> Result<u8> {
    let shamt = parse_immediate(text)?;
    if shamt < 0 || shamt > 31 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Shift amount must be 0-31, got {}", shamt),
        });
    }
    Ok(shamt as u8)
}

/// Parse single register operand: mnemonic rs1
fn parse_single_register(operands: &str) -> Result<Register> {
    let operands = operands.trim();
    if operands.is_empty() {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: "Missing register operand".to_string(),
        });
    }
    parse_register(operands)
}

/// Parse two register operands: mnemonic rs1, rs2
fn parse_two_registers(operands: &str) -> Result<(Register, Register)> {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: "Requires 2 operands: rs1, rs2".to_string(),
        });
    }

    let rs1 = parse_register(parts[0])?;
    let rs2 = parse_register(parts[1])?;

    Ok((rs1, rs2))
}

/// Parse register and u8 operands: mnemonic rs1, value
fn parse_register_u8(operands: &str) -> Result<(Register, u8)> {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: "Requires 2 operands: rs1, value".to_string(),
        });
    }

    let rs1 = parse_register(parts[0])?;
    let value = parse_immediate(parts[1])?;

    if value < 0 || value > 255 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Value must be 0-255, got {}", value),
        });
    }

    Ok((rs1, value as u8))
}

fn parse_mnemonic(mnemonic: &str, operands: &str) -> Result<Instruction> {
    match mnemonic {
        // System
        "halt" => Ok(Instruction::Halt),
        "ecall" => Ok(Instruction::Ecall),
        "ebreak" => Ok(Instruction::Ebreak),

        // R-type: Arithmetic
        "add" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Add { rd, rs1, rs2 })
        }
        "sub" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Sub { rd, rs1, rs2 })
        }
        "mul" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Mul { rd, rs1, rs2 })
        }
        "mulh" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Mulh { rd, rs1, rs2 })
        }
        "mulhu" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Mulhu { rd, rs1, rs2 })
        }
        "div" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Div { rd, rs1, rs2 })
        }
        "divu" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Divu { rd, rs1, rs2 })
        }
        "rem" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Rem { rd, rs1, rs2 })
        }
        "remu" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Remu { rd, rs1, rs2 })
        }

        // R-type: Logic
        "and" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::And { rd, rs1, rs2 })
        }
        "or" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Or { rd, rs1, rs2 })
        }
        "xor" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Xor { rd, rs1, rs2 })
        }
        "sll" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Sll { rd, rs1, rs2 })
        }
        "srl" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Srl { rd, rs1, rs2 })
        }
        "sra" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Sra { rd, rs1, rs2 })
        }
        "slt" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Slt { rd, rs1, rs2 })
        }
        "sltu" => {
            let (rd, rs1, rs2) = parse_r_type(mnemonic, operands)?;
            Ok(Instruction::Sltu { rd, rs1, rs2 })
        }

        // I-type: Arithmetic Immediate
        "addi" => {
            let (rd, rs1, imm) = parse_i_type(mnemonic, operands)?;
            Ok(Instruction::Addi { rd, rs1, imm })
        }
        "slti" => {
            let (rd, rs1, imm) = parse_i_type(mnemonic, operands)?;
            Ok(Instruction::Slti { rd, rs1, imm })
        }
        "sltiu" => {
            let (rd, rs1, imm) = parse_i_type(mnemonic, operands)?;
            Ok(Instruction::Sltiu { rd, rs1, imm })
        }
        "xori" => {
            let (rd, rs1, imm) = parse_i_type(mnemonic, operands)?;
            Ok(Instruction::Xori { rd, rs1, imm })
        }
        "ori" => {
            let (rd, rs1, imm) = parse_i_type(mnemonic, operands)?;
            Ok(Instruction::Ori { rd, rs1, imm })
        }
        "andi" => {
            let (rd, rs1, imm) = parse_i_type(mnemonic, operands)?;
            Ok(Instruction::Andi { rd, rs1, imm })
        }

        // I-type: Shift Immediate (special handling for shamt)
        "slli" => {
            let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
            if parts.len() != 3 {
                return Err(AssemblerError::SyntaxError {
                    line: 0,
                    column: 0,
                    message: format!("{} requires 3 operands: rd, rs1, shamt", mnemonic),
                });
            }
            let rd = parse_register(parts[0])?;
            let rs1 = parse_register(parts[1])?;
            let shamt = parse_shamt(parts[2])?;
            Ok(Instruction::Slli { rd, rs1, shamt })
        }
        "srli" => {
            let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
            if parts.len() != 3 {
                return Err(AssemblerError::SyntaxError {
                    line: 0,
                    column: 0,
                    message: format!("{} requires 3 operands: rd, rs1, shamt", mnemonic),
                });
            }
            let rd = parse_register(parts[0])?;
            let rs1 = parse_register(parts[1])?;
            let shamt = parse_shamt(parts[2])?;
            Ok(Instruction::Srli { rd, rs1, shamt })
        }
        "srai" => {
            let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
            if parts.len() != 3 {
                return Err(AssemblerError::SyntaxError {
                    line: 0,
                    column: 0,
                    message: format!("{} requires 3 operands: rd, rs1, shamt", mnemonic),
                });
            }
            let rd = parse_register(parts[0])?;
            let rs1 = parse_register(parts[1])?;
            let shamt = parse_shamt(parts[2])?;
            Ok(Instruction::Srai { rd, rs1, shamt })
        }

        // I-type: Load
        "lw" => {
            let (rd, rs1, imm) = parse_load(mnemonic, operands)?;
            Ok(Instruction::Lw { rd, rs1, imm })
        }
        "lh" => {
            let (rd, rs1, imm) = parse_load(mnemonic, operands)?;
            Ok(Instruction::Lh { rd, rs1, imm })
        }
        "lhu" => {
            let (rd, rs1, imm) = parse_load(mnemonic, operands)?;
            Ok(Instruction::Lhu { rd, rs1, imm })
        }
        "lb" => {
            let (rd, rs1, imm) = parse_load(mnemonic, operands)?;
            Ok(Instruction::Lb { rd, rs1, imm })
        }
        "lbu" => {
            let (rd, rs1, imm) = parse_load(mnemonic, operands)?;
            Ok(Instruction::Lbu { rd, rs1, imm })
        }

        // S-type: Store
        "sw" => {
            let (rs1, rs2, imm) = parse_store(mnemonic, operands)?;
            Ok(Instruction::Sw { rs1, rs2, imm })
        }
        "sh" => {
            let (rs1, rs2, imm) = parse_store(mnemonic, operands)?;
            Ok(Instruction::Sh { rs1, rs2, imm })
        }
        "sb" => {
            let (rs1, rs2, imm) = parse_store(mnemonic, operands)?;
            Ok(Instruction::Sb { rs1, rs2, imm })
        }

        // B-type: Branch
        "beq" => {
            let (rs1, rs2, imm) = parse_b_type(mnemonic, operands)?;
            Ok(Instruction::Beq { rs1, rs2, imm })
        }
        "bne" => {
            let (rs1, rs2, imm) = parse_b_type(mnemonic, operands)?;
            Ok(Instruction::Bne { rs1, rs2, imm })
        }
        "blt" => {
            let (rs1, rs2, imm) = parse_b_type(mnemonic, operands)?;
            Ok(Instruction::Blt { rs1, rs2, imm })
        }
        "bge" => {
            let (rs1, rs2, imm) = parse_b_type(mnemonic, operands)?;
            Ok(Instruction::Bge { rs1, rs2, imm })
        }
        "bltu" => {
            let (rs1, rs2, imm) = parse_b_type(mnemonic, operands)?;
            Ok(Instruction::Bltu { rs1, rs2, imm })
        }
        "bgeu" => {
            let (rs1, rs2, imm) = parse_b_type(mnemonic, operands)?;
            Ok(Instruction::Bgeu { rs1, rs2, imm })
        }

        // J-type: Jump
        "jal" => {
            let (rd, imm) = parse_u_type(mnemonic, operands)?;
            Ok(Instruction::Jal { rd, imm })
        }
        "jalr" => {
            let (rd, rs1, imm) = parse_i_type(mnemonic, operands)?;
            Ok(Instruction::Jalr { rd, rs1, imm })
        }

        // U-type: Upper Immediate
        "lui" => {
            let (rd, imm) = parse_u_type(mnemonic, operands)?;
            Ok(Instruction::Lui { rd, imm })
        }
        "auipc" => {
            let (rd, imm) = parse_u_type(mnemonic, operands)?;
            Ok(Instruction::Auipc { rd, imm })
        }

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

        // ZK-Custom
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
            let (rs1, bits) = parse_register_u8(operands)?;
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

    // R-type tests
    #[test]
    fn test_parse_add() {
        let instr = parse_instruction("add a0, a1, a2").unwrap();
        assert_eq!(instr, Instruction::Add {
            rd: Register::R4,
            rs1: Register::R5,
            rs2: Register::R6,
        });
    }

    #[test]
    fn test_parse_sub() {
        let instr = parse_instruction("sub t0, t1, t2").unwrap();
        assert_eq!(instr, Instruction::Sub {
            rd: Register::R8,
            rs1: Register::R9,
            rs2: Register::R10,
        });
    }

    #[test]
    fn test_parse_mul() {
        let instr = parse_instruction("mul s0, s1, s2").unwrap();
        assert_eq!(instr, Instruction::Mul {
            rd: Register::R16,
            rs1: Register::R17,
            rs2: Register::R18,
        });
    }

    #[test]
    fn test_parse_div() {
        let instr = parse_instruction("div a0, a1, a2").unwrap();
        assert_eq!(instr, Instruction::Div {
            rd: Register::R4,
            rs1: Register::R5,
            rs2: Register::R6,
        });
    }

    #[test]
    fn test_parse_and() {
        let instr = parse_instruction("and a0, a1, a2").unwrap();
        assert_eq!(instr, Instruction::And {
            rd: Register::R4,
            rs1: Register::R5,
            rs2: Register::R6,
        });
    }

    #[test]
    fn test_parse_or() {
        let instr = parse_instruction("or a0, a1, a2").unwrap();
        assert_eq!(instr, Instruction::Or {
            rd: Register::R4,
            rs1: Register::R5,
            rs2: Register::R6,
        });
    }

    #[test]
    fn test_parse_xor() {
        let instr = parse_instruction("xor a0, a1, a2").unwrap();
        assert_eq!(instr, Instruction::Xor {
            rd: Register::R4,
            rs1: Register::R5,
            rs2: Register::R6,
        });
    }

    #[test]
    fn test_parse_sll() {
        let instr = parse_instruction("sll a0, a1, a2").unwrap();
        assert_eq!(instr, Instruction::Sll {
            rd: Register::R4,
            rs1: Register::R5,
            rs2: Register::R6,
        });
    }

    #[test]
    fn test_parse_srl() {
        let instr = parse_instruction("srl a0, a1, a2").unwrap();
        assert_eq!(instr, Instruction::Srl {
            rd: Register::R4,
            rs1: Register::R5,
            rs2: Register::R6,
        });
    }

    #[test]
    fn test_parse_sra() {
        let instr = parse_instruction("sra a0, a1, a2").unwrap();
        assert_eq!(instr, Instruction::Sra {
            rd: Register::R4,
            rs1: Register::R5,
            rs2: Register::R6,
        });
    }

    #[test]
    fn test_parse_slt() {
        let instr = parse_instruction("slt a0, a1, a2").unwrap();
        assert_eq!(instr, Instruction::Slt {
            rd: Register::R4,
            rs1: Register::R5,
            rs2: Register::R6,
        });
    }

    #[test]
    fn test_parse_sltu() {
        let instr = parse_instruction("sltu a0, a1, a2").unwrap();
        assert_eq!(instr, Instruction::Sltu {
            rd: Register::R4,
            rs1: Register::R5,
            rs2: Register::R6,
        });
    }

    // I-type ALU tests
    #[test]
    fn test_parse_addi() {
        let instr = parse_instruction("addi a0, a1, 100").unwrap();
        assert_eq!(instr, Instruction::Addi {
            rd: Register::R4,
            rs1: Register::R5,
            imm: 100,
        });
    }

    #[test]
    fn test_parse_addi_negative() {
        let instr = parse_instruction("addi a0, a1, -50").unwrap();
        assert_eq!(instr, Instruction::Addi {
            rd: Register::R4,
            rs1: Register::R5,
            imm: -50,
        });
    }

    #[test]
    fn test_parse_addi_hex() {
        let instr = parse_instruction("addi a0, a1, 0xFF").unwrap();
        assert_eq!(instr, Instruction::Addi {
            rd: Register::R4,
            rs1: Register::R5,
            imm: 0xFF,
        });
    }

    #[test]
    fn test_parse_andi() {
        let instr = parse_instruction("andi a0, a1, 0xFF").unwrap();
        assert_eq!(instr, Instruction::Andi {
            rd: Register::R4,
            rs1: Register::R5,
            imm: 0xFF,
        });
    }

    #[test]
    fn test_parse_ori() {
        let instr = parse_instruction("ori a0, a1, 0x10").unwrap();
        assert_eq!(instr, Instruction::Ori {
            rd: Register::R4,
            rs1: Register::R5,
            imm: 0x10,
        });
    }

    #[test]
    fn test_parse_xori() {
        let instr = parse_instruction("xori a0, a1, -1").unwrap();
        assert_eq!(instr, Instruction::Xori {
            rd: Register::R4,
            rs1: Register::R5,
            imm: -1,
        });
    }

    #[test]
    fn test_parse_slti() {
        let instr = parse_instruction("slti a0, a1, 42").unwrap();
        assert_eq!(instr, Instruction::Slti {
            rd: Register::R4,
            rs1: Register::R5,
            imm: 42,
        });
    }

    #[test]
    fn test_parse_sltiu() {
        let instr = parse_instruction("sltiu a0, a1, 42").unwrap();
        assert_eq!(instr, Instruction::Sltiu {
            rd: Register::R4,
            rs1: Register::R5,
            imm: 42,
        });
    }

    #[test]
    fn test_parse_slli() {
        let instr = parse_instruction("slli a0, a1, 4").unwrap();
        assert_eq!(instr, Instruction::Slli {
            rd: Register::R4,
            rs1: Register::R5,
            shamt: 4,
        });
    }

    #[test]
    fn test_parse_srli() {
        let instr = parse_instruction("srli a0, a1, 8").unwrap();
        assert_eq!(instr, Instruction::Srli {
            rd: Register::R4,
            rs1: Register::R5,
            shamt: 8,
        });
    }

    #[test]
    fn test_parse_srai() {
        let instr = parse_instruction("srai a0, a1, 16").unwrap();
        assert_eq!(instr, Instruction::Srai {
            rd: Register::R4,
            rs1: Register::R5,
            shamt: 16,
        });
    }

    // Load tests
    #[test]
    fn test_parse_lw() {
        let instr = parse_instruction("lw a0, 0(sp)").unwrap();
        assert_eq!(instr, Instruction::Lw {
            rd: Register::R4,
            rs1: Register::R2,
            imm: 0,
        });
    }

    #[test]
    fn test_parse_lw_offset() {
        let instr = parse_instruction("lw a0, 16(sp)").unwrap();
        assert_eq!(instr, Instruction::Lw {
            rd: Register::R4,
            rs1: Register::R2,
            imm: 16,
        });
    }

    #[test]
    fn test_parse_lw_negative_offset() {
        let instr = parse_instruction("lw a0, -8(fp)").unwrap();
        assert_eq!(instr, Instruction::Lw {
            rd: Register::R4,
            rs1: Register::R3,
            imm: -8,
        });
    }

    #[test]
    fn test_parse_lh() {
        let instr = parse_instruction("lh a0, 4(sp)").unwrap();
        assert_eq!(instr, Instruction::Lh {
            rd: Register::R4,
            rs1: Register::R2,
            imm: 4,
        });
    }

    #[test]
    fn test_parse_lhu() {
        let instr = parse_instruction("lhu a0, 4(sp)").unwrap();
        assert_eq!(instr, Instruction::Lhu {
            rd: Register::R4,
            rs1: Register::R2,
            imm: 4,
        });
    }

    #[test]
    fn test_parse_lb() {
        let instr = parse_instruction("lb a0, 1(sp)").unwrap();
        assert_eq!(instr, Instruction::Lb {
            rd: Register::R4,
            rs1: Register::R2,
            imm: 1,
        });
    }

    #[test]
    fn test_parse_lbu() {
        let instr = parse_instruction("lbu a0, 1(sp)").unwrap();
        assert_eq!(instr, Instruction::Lbu {
            rd: Register::R4,
            rs1: Register::R2,
            imm: 1,
        });
    }

    // Store tests
    #[test]
    fn test_parse_sw() {
        let instr = parse_instruction("sw a0, 0(sp)").unwrap();
        assert_eq!(instr, Instruction::Sw {
            rs1: Register::R2,
            rs2: Register::R4,
            imm: 0,
        });
    }

    #[test]
    fn test_parse_sw_offset() {
        let instr = parse_instruction("sw a0, 16(sp)").unwrap();
        assert_eq!(instr, Instruction::Sw {
            rs1: Register::R2,
            rs2: Register::R4,
            imm: 16,
        });
    }

    #[test]
    fn test_parse_sh() {
        let instr = parse_instruction("sh a0, 4(sp)").unwrap();
        assert_eq!(instr, Instruction::Sh {
            rs1: Register::R2,
            rs2: Register::R4,
            imm: 4,
        });
    }

    #[test]
    fn test_parse_sb() {
        let instr = parse_instruction("sb a0, 1(sp)").unwrap();
        assert_eq!(instr, Instruction::Sb {
            rs1: Register::R2,
            rs2: Register::R4,
            imm: 1,
        });
    }

    // Branch tests
    #[test]
    fn test_parse_beq() {
        let instr = parse_instruction("beq a0, a1, 16").unwrap();
        assert_eq!(instr, Instruction::Beq {
            rs1: Register::R4,
            rs2: Register::R5,
            imm: 16,
        });
    }

    #[test]
    fn test_parse_bne() {
        let instr = parse_instruction("bne a0, a1, -8").unwrap();
        assert_eq!(instr, Instruction::Bne {
            rs1: Register::R4,
            rs2: Register::R5,
            imm: -8,
        });
    }

    #[test]
    fn test_parse_blt() {
        let instr = parse_instruction("blt a0, a1, 100").unwrap();
        assert_eq!(instr, Instruction::Blt {
            rs1: Register::R4,
            rs2: Register::R5,
            imm: 100,
        });
    }

    #[test]
    fn test_parse_bge() {
        let instr = parse_instruction("bge a0, a1, 100").unwrap();
        assert_eq!(instr, Instruction::Bge {
            rs1: Register::R4,
            rs2: Register::R5,
            imm: 100,
        });
    }

    #[test]
    fn test_parse_bltu() {
        let instr = parse_instruction("bltu a0, a1, 100").unwrap();
        assert_eq!(instr, Instruction::Bltu {
            rs1: Register::R4,
            rs2: Register::R5,
            imm: 100,
        });
    }

    #[test]
    fn test_parse_bgeu() {
        let instr = parse_instruction("bgeu a0, a1, 100").unwrap();
        assert_eq!(instr, Instruction::Bgeu {
            rs1: Register::R4,
            rs2: Register::R5,
            imm: 100,
        });
    }

    // Jump tests
    #[test]
    fn test_parse_jal() {
        let instr = parse_instruction("jal ra, 1000").unwrap();
        assert_eq!(instr, Instruction::Jal {
            rd: Register::R30,
            imm: 1000,
        });
    }

    #[test]
    fn test_parse_jalr() {
        let instr = parse_instruction("jalr ra, a0, 0").unwrap();
        assert_eq!(instr, Instruction::Jalr {
            rd: Register::R30,
            rs1: Register::R4,
            imm: 0,
        });
    }

    // U-type tests
    #[test]
    fn test_parse_lui() {
        let instr = parse_instruction("lui a0, 0x12345").unwrap();
        assert_eq!(instr, Instruction::Lui {
            rd: Register::R4,
            imm: 0x12345,
        });
    }

    #[test]
    fn test_parse_auipc() {
        let instr = parse_instruction("auipc a0, 0x1000").unwrap();
        assert_eq!(instr, Instruction::Auipc {
            rd: Register::R4,
            imm: 0x1000,
        });
    }

    // ZK I/O tests
    #[test]
    fn test_parse_read() {
        let instr = parse_instruction("read a0").unwrap();
        assert_eq!(instr, Instruction::Read {
            rd: Register::R4,
        });
    }

    #[test]
    fn test_parse_write() {
        let instr = parse_instruction("write a0").unwrap();
        assert_eq!(instr, Instruction::Write {
            rs1: Register::R4,
        });
    }

    #[test]
    fn test_parse_hint() {
        let instr = parse_instruction("hint a0").unwrap();
        assert_eq!(instr, Instruction::Hint {
            rd: Register::R4,
        });
    }

    // ZK-Custom tests
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
        let instr = parse_instruction("assert_ne a0, a1").unwrap();
        assert_eq!(instr, Instruction::AssertNe {
            rs1: Register::R4,
            rs2: Register::R5,
        });
    }

    #[test]
    fn test_parse_assert_zero() {
        let instr = parse_instruction("assert_zero a0").unwrap();
        assert_eq!(instr, Instruction::AssertZero {
            rs1: Register::R4,
        });
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
        let instr = parse_instruction("commit a0").unwrap();
        assert_eq!(instr, Instruction::Commit {
            rs1: Register::R4,
        });
    }

    // Error cases
    #[test]
    fn test_parse_unknown_instruction() {
        let result = parse_instruction("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_register() {
        let result = parse_instruction("add invalid, a1, a2");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_wrong_operand_count() {
        let result = parse_instruction("add a0, a1");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_immediate() {
        let result = parse_instruction("addi a0, a1, notanumber");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_memory_operand() {
        let result = parse_instruction("lw a0, invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_paren() {
        let result = parse_instruction("lw a0, 16(sp");
        assert!(result.is_err());
    }
}
