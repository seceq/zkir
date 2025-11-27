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

fn parse_mnemonic(mnemonic: &str, _operands: &str) -> Result<Instruction> {
    match mnemonic {
        "halt" => Ok(Instruction::Halt),
        "ecall" => Ok(Instruction::Ecall),
        "ebreak" => Ok(Instruction::Ebreak),
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
}
