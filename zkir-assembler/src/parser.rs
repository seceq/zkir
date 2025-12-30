//! Assembly parser for ZKIR v3.4
//!
//! Parses assembly lines with config directive support.

use zkir_spec::Register;
use crate::error::{AssemblerError, Result};
use crate::lexer::Token;
use logos::Logos;

/// Parse register name
pub fn parse_register(name: &str) -> Result<Register> {
    let name = name.trim().to_lowercase();

    match name.as_str() {
        // R0 - hardwired zero
        "zero" | "r0" => Ok(Register::R0),

        // R1 - return address
        "ra" | "r1" => Ok(Register::R1),

        // R2 - stack pointer
        "sp" | "r2" => Ok(Register::R2),

        // R3 - global pointer
        "gp" | "r3" => Ok(Register::R3),

        // R4 - thread pointer
        "tp" | "r4" => Ok(Register::R4),

        // R5 - frame pointer
        "fp" | "r5" => Ok(Register::R5),

        // R6-R7 - saved registers s0-s1
        "s0" | "r6" => Ok(Register::R6),
        "s1" | "r7" => Ok(Register::R7),

        // R8-R10 - temporaries t0-t2
        "t0" | "r8" => Ok(Register::R8),
        "t1" | "r9" => Ok(Register::R9),
        "t2" | "r10" => Ok(Register::R10),

        // R11-R15 - arguments a0-a5
        "a0" | "r11" => Ok(Register::R11),
        "a1" | "r12" => Ok(Register::R12),
        "a2" | "r13" => Ok(Register::R13),
        "a3" | "r14" => Ok(Register::R14),
        "a4" | "r15" => Ok(Register::R15),

        _ => Err(AssemblerError::InvalidRegister {
            line: 0,
            register: name.to_string(),
        }),
    }
}

/// Extract numeric value from token
pub fn extract_number(token: &Token) -> Result<i64> {
    match token {
        Token::Number(n) => Ok(*n),
        Token::Hex(n) => Ok(*n as i64),
        Token::Binary(n) => Ok(*n as i64),
        _ => Err(AssemblerError::SyntaxError {
            line: 0,
            message: format!("Expected number, got {:?}", token),
        }),
    }
}

/// Tokenize a line of assembly
pub fn tokenize(line: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut lexer = Token::lexer(line);

    while let Some(result) = lexer.next() {
        match result {
            Ok(token) => tokens.push(token),
            Err(_) => {
                return Err(AssemblerError::SyntaxError {
                    line: 0,
                    message: format!("Invalid token at position {}", lexer.span().start),
                });
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_register_v3_4() {
        // Zero register
        assert_eq!(parse_register("zero").unwrap(), Register::R0);
        assert_eq!(parse_register("r0").unwrap(), Register::R0);

        // Return address
        assert_eq!(parse_register("ra").unwrap(), Register::R1);
        assert_eq!(parse_register("r1").unwrap(), Register::R1);

        // Stack pointer
        assert_eq!(parse_register("sp").unwrap(), Register::R2);
        assert_eq!(parse_register("r2").unwrap(), Register::R2);

        // Arguments
        assert_eq!(parse_register("a0").unwrap(), Register::R11);
        assert_eq!(parse_register("a4").unwrap(), Register::R15);

        // Saved registers
        assert_eq!(parse_register("s0").unwrap(), Register::R6);
        assert_eq!(parse_register("s1").unwrap(), Register::R7);

        // Temporaries
        assert_eq!(parse_register("t0").unwrap(), Register::R8);
        assert_eq!(parse_register("t2").unwrap(), Register::R10);
    }

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("add r1, r2, r3").unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[0], Token::Identifier(_)));
        assert!(matches!(tokens[1], Token::Register(_)));
        assert!(matches!(tokens[2], Token::Comma));
    }

    #[test]
    fn test_tokenize_directive() {
        let tokens = tokenize(".config limb_bits 20").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Directive(_)));
    }

    #[test]
    fn test_extract_number() {
        let token = Token::Number(42);
        assert_eq!(extract_number(&token).unwrap(), 42);

        let token = Token::Hex(0x1A);
        assert_eq!(extract_number(&token).unwrap(), 26);

        let token = Token::Binary(0b1010);
        assert_eq!(extract_number(&token).unwrap(), 10);
    }
}
