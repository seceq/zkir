//! # Lexer for ZKIR v3.4 Assembly Language

use logos::Logos;

/// Tokens for ZKIR assembly
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t]+")] // Skip whitespace (not newlines)
#[logos(skip r"#[^\n]*")] // Skip comments
pub enum Token {
    /// Identifier (instruction mnemonics, labels)
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    /// Register (r0-r15, or ABI names)
    #[regex(r"r([0-9]|1[0-5])", |lex| lex.slice().to_string())]
    #[regex(r"(zero|ra|sp|gp|tp|fp|s[01]|t[0-2]|a[0-5])", |lex| lex.slice().to_string())]
    Register(String),

    /// Decimal number
    #[regex(r"-?[0-9]+", |lex| lex.slice().parse().ok())]
    Number(i64),

    /// Hexadecimal number
    #[regex(r"0x[0-9a-fA-F]+", |lex| u64::from_str_radix(&lex.slice()[2..], 16).ok())]
    Hex(u64),

    /// Binary number
    #[regex(r"0b[01]+", |lex| u64::from_str_radix(&lex.slice()[2..], 2).ok())]
    Binary(u64),

    /// Directive (.config, .text, .data, etc.)
    #[regex(r"\.[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice()[1..].to_string())]
    Directive(String),

    /// Comma
    #[token(",")]
    Comma,

    /// Colon (for labels)
    #[token(":")]
    Colon,

    /// Left parenthesis
    #[token("(")]
    LParen,

    /// Right parenthesis
    #[token(")")]
    RParen,

    /// Newline
    #[regex(r"\n")]
    Newline,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_registers() {
        let mut lex = Token::lexer("r0 r15 zero ra sp");
        assert_eq!(lex.next(), Some(Ok(Token::Register("r0".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::Register("r15".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::Register("zero".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::Register("ra".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::Register("sp".to_string()))));
    }

    #[test]
    fn test_lexer_numbers() {
        let mut lex = Token::lexer("42 -10 0x1A 0b1010");
        assert_eq!(lex.next(), Some(Ok(Token::Number(42))));
        assert_eq!(lex.next(), Some(Ok(Token::Number(-10))));
        assert_eq!(lex.next(), Some(Ok(Token::Hex(0x1A))));
        assert_eq!(lex.next(), Some(Ok(Token::Binary(0b1010))));
    }

    #[test]
    fn test_lexer_directive() {
        let mut lex = Token::lexer(".config .text .data");
        assert_eq!(lex.next(), Some(Ok(Token::Directive("config".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::Directive("text".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::Directive("data".to_string()))));
    }

    #[test]
    fn test_lexer_instruction() {
        let mut lex = Token::lexer("add r1, r2, r3");
        assert_eq!(lex.next(), Some(Ok(Token::Identifier("add".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::Register("r1".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::Comma)));
        assert_eq!(lex.next(), Some(Ok(Token::Register("r2".to_string()))));
        assert_eq!(lex.next(), Some(Ok(Token::Comma)));
        assert_eq!(lex.next(), Some(Ok(Token::Register("r3".to_string()))));
    }
}
