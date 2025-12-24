//! Lexer for ZK IR assembly language

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // Instructions
    Identifier(String),

    // Registers
    Register(String),

    // Literals
    Number(i64),      // Decimal: 123, -456
    HexNumber(u32),   // Hex: 0x1234
    BinNumber(u32),   // Binary: 0b1010

    // Symbols
    Comma,            // ,
    Colon,            // :
    LParen,           // (
    RParen,           // )

    // Directives
    Directive(String), // .text, .data, .align, etc.

    // Special
    Newline,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "identifier({})", s),
            Token::Register(r) => write!(f, "register({})", r),
            Token::Number(n) => write!(f, "number({})", n),
            Token::HexNumber(n) => write!(f, "hex(0x{:x})", n),
            Token::BinNumber(n) => write!(f, "bin(0b{:b})", n),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Directive(d) => write!(f, ".{}", d),
            Token::Newline => write!(f, "\\n"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn current(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        let pos = self.pos + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.current()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        // Skip until end of line
        while let Some(ch) = self.current() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let is_negative = self.current() == Some('-');
        if is_negative {
            self.advance();
        }

        // Check for hex (0x) or binary (0b)
        if self.current() == Some('0') {
            if self.peek(1) == Some('x') || self.peek(1) == Some('X') {
                // Hexadecimal
                self.advance(); // '0'
                self.advance(); // 'x'
                let hex_str = self.read_hex_digits();
                return u32::from_str_radix(&hex_str, 16)
                    .map(Token::HexNumber)
                    .map_err(|e| format!("Invalid hex number at line {}: {}", self.line, e));
            } else if self.peek(1) == Some('b') || self.peek(1) == Some('B') {
                // Binary
                self.advance(); // '0'
                self.advance(); // 'b'
                let bin_str = self.read_bin_digits();
                return u32::from_str_radix(&bin_str, 2)
                    .map(Token::BinNumber)
                    .map_err(|e| format!("Invalid binary number at line {}: {}", self.line, e));
            }
        }

        // Decimal number
        let mut num_str = String::new();
        if is_negative {
            num_str.push('-');
        }

        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if num_str.is_empty() || num_str == "-" {
            return Err(format!("Invalid number at line {}", self.line));
        }

        num_str.parse::<i64>()
            .map(Token::Number)
            .map_err(|e| format!("Invalid number at line {}: {}", self.line, e))
    }

    fn read_hex_digits(&mut self) -> String {
        let mut result = String::new();
        while let Some(ch) = self.current() {
            if ch.is_ascii_hexdigit() {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn read_bin_digits(&mut self) -> String {
        let mut result = String::new();
        while let Some(ch) = self.current() {
            if ch == '0' || ch == '1' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();

        match self.current() {
            None => Ok(Token::Eof),
            Some('\n') => {
                self.advance();
                Ok(Token::Newline)
            }
            Some('#') | Some(';') => {
                self.skip_comment();
                self.next_token() // Skip to next token
            }
            Some(',') => {
                self.advance();
                Ok(Token::Comma)
            }
            Some(':') => {
                self.advance();
                Ok(Token::Colon)
            }
            Some('(') => {
                self.advance();
                Ok(Token::LParen)
            }
            Some(')') => {
                self.advance();
                Ok(Token::RParen)
            }
            Some('.') => {
                self.advance();
                let directive = self.read_identifier();
                Ok(Token::Directive(directive))
            }
            Some(ch) if ch == '-' || ch.is_ascii_digit() => {
                self.read_number()
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();

                // Check if it's a register
                if is_register(&ident) {
                    Ok(Token::Register(ident))
                } else {
                    Ok(Token::Identifier(ident))
                }
            }
            Some(ch) => {
                Err(format!("Unexpected character '{}' at line {}, col {}", ch, self.line, self.col))
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token, Token::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn col(&self) -> usize {
        self.col
    }
}

/// Check if identifier is a register name
fn is_register(s: &str) -> bool {
    matches!(
        s,
        // Numbered registers
        "r0" | "r1" | "r2" | "r3" | "r4" | "r5" | "r6" | "r7" |
        "r8" | "r9" | "r10" | "r11" | "r12" | "r13" | "r14" | "r15" |
        "r16" | "r17" | "r18" | "r19" | "r20" | "r21" | "r22" | "r23" |
        "r24" | "r25" | "r26" | "r27" | "r28" | "r29" | "r30" | "r31" |
        // Named registers (RISC-V convention)
        "zero" | "ra" | "sp" | "gp" | "tp" |
        "t0" | "t1" | "t2" | "t3" | "t4" | "t5" | "t6" |
        "fp" | "s0" | "s1" | "s2" | "s3" | "s4" | "s5" | "s6" | "s7" | "s8" | "s9" | "s10" | "s11" |
        "a0" | "a1" | "a2" | "a3" | "a4" | "a5" | "a6" | "a7"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_simple() {
        let mut lexer = Lexer::new("add a0, a1, a2");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 7); // add, a0, ,, a1, ,, a2, EOF
        assert!(matches!(tokens[0], Token::Identifier(_)));
        assert!(matches!(tokens[1], Token::Register(_)));
    }

    #[test]
    fn test_lex_numbers() {
        let mut lexer = Lexer::new("123 -456 0x1A 0b1010");
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0], Token::Number(123)));
        assert!(matches!(tokens[1], Token::Number(-456)));
        assert!(matches!(tokens[2], Token::HexNumber(0x1A)));
        assert!(matches!(tokens[3], Token::BinNumber(0b1010)));
    }

    #[test]
    fn test_lex_directive() {
        let mut lexer = Lexer::new(".text\n.data");
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(&tokens[0], Token::Directive(s) if s == "text"));
        assert!(matches!(tokens[1], Token::Newline));
        assert!(matches!(&tokens[2], Token::Directive(s) if s == "data"));
    }

    #[test]
    fn test_lex_comment() {
        let mut lexer = Lexer::new("add a0, a1, a2 # this is a comment\n");
        let tokens = lexer.tokenize().unwrap();

        // Comment should be skipped
        assert_eq!(tokens.len(), 8); // add, a0, comma, a1, comma, a2, newline, EOF
    }

    #[test]
    fn test_lex_label() {
        let mut lexer = Lexer::new("loop:\n  add a0, a1, a2");
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(&tokens[0], Token::Identifier(s) if s == "loop"));
        assert!(matches!(tokens[1], Token::Colon));
    }

    #[test]
    fn test_lex_memory_operand() {
        let mut lexer = Lexer::new("lw a0, 16(sp)");
        let tokens = lexer.tokenize().unwrap();

        // lw, a0, comma, 16, lparen, sp, rparen
        assert!(matches!(&tokens[0], Token::Identifier(s) if s == "lw"));
        assert!(matches!(tokens[3], Token::Number(16)));
        assert!(matches!(tokens[4], Token::LParen));
        assert!(matches!(tokens[6], Token::RParen));
    }
}
