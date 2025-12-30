//! Main assembler logic with label resolution and config directive support

use zkir_spec::{Program, Instruction, Config, memory::CODE_BASE};
use crate::error::{Result, AssemblerError};
use crate::parser::{parse_register, tokenize, extract_number};
use crate::encoder::encode;
use crate::lexer::Token;
use std::collections::HashMap;

/// Assembly item (parsed line)
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Item {
    Label(String),
    Instruction(Instruction),
    ConfigDirective { key: String, value: u64 },
    Empty,
}

/// Assemble source code into a program
///
/// Supports:
/// - All v3.4 instructions
/// - `.config` directives for limb configuration
/// - Labels for branch/jump targets
/// - Comments (# style)
///
/// # Example
/// ```
/// use zkir_assembler::assemble;
///
/// let source = r#"
///     .config limb_bits 20
///     .config data_limbs 2
///
/// main:
///     add a0, zero, zero
///     ecall
/// "#;
///
/// let program = assemble(source).unwrap();
/// ```
pub fn assemble(source: &str) -> Result<Program> {
    // First pass: parse all lines and collect config/labels
    let (items, labels, config) = first_pass(source)?;

    // Second pass: encode instructions with resolved labels
    let code = second_pass(&items, &labels)?;

    // Create program with configuration
    let mut program = Program::with_config(config)
        .map_err(|e| AssemblerError::SpecError(e.into()))?;
    program.code = code;
    program.header.code_size = (program.code.len() * 4) as u32;

    Ok(program)
}

/// First pass: parse lines and collect config/labels
fn first_pass(source: &str) -> Result<(Vec<Item>, HashMap<String, u64>, Config)> {
    let mut items = Vec::new();
    let mut labels = HashMap::new();
    let mut config = Config::DEFAULT;
    let mut pc = CODE_BASE;

    for (line_num, line_text) in source.lines().enumerate() {
        let line_text = line_text.trim();

        // Skip empty lines and comments
        if line_text.is_empty() || line_text.starts_with('#') {
            items.push(Item::Empty);
            continue;
        }

        // Remove inline comments
        let line_text = if let Some(pos) = line_text.find('#') {
            line_text[..pos].trim()
        } else {
            line_text
        };

        if line_text.is_empty() {
            items.push(Item::Empty);
            continue;
        }

        // Tokenize the line
        let tokens = tokenize(line_text)?;
        if tokens.is_empty() {
            items.push(Item::Empty);
            continue;
        }

        // Check for label (identifier followed by colon)
        if tokens.len() >= 2 {
            if let (Token::Identifier(name), Token::Colon) = (&tokens[0], &tokens[1]) {
                // Validate label name
                if !is_valid_label(name) {
                    return Err(AssemblerError::SyntaxError {
                        line: line_num + 1,
                        message: format!("Invalid label name: {}", name),
                    });
                }

                // Check for duplicate
                if labels.contains_key(name) {
                    return Err(AssemblerError::SyntaxError {
                        line: line_num + 1,
                        message: format!("Duplicate label: {}", name),
                    });
                }

                labels.insert(name.clone(), pc);
                items.push(Item::Label(name.clone()));

                // Check if there's an instruction after the label
                if tokens.len() > 2 {
                    let instr = parse_instruction_tokens(&tokens[2..], line_num)?;
                    items.push(Item::Instruction(instr));
                    pc += 4;
                }
                continue;
            }
        }

        // Check for directive
        if let Token::Directive(directive) = &tokens[0] {
            if directive == "config" {
                if tokens.len() != 3 {
                    return Err(AssemblerError::SyntaxError {
                        line: line_num + 1,
                        message: ".config requires 2 arguments: key value".to_string(),
                    });
                }

                let key = match &tokens[1] {
                    Token::Identifier(k) => k.clone(),
                    _ => {
                        return Err(AssemblerError::SyntaxError {
                            line: line_num + 1,
                            message: "Config key must be an identifier".to_string(),
                        });
                    }
                };

                let value = extract_number(&tokens[2])? as u64;

                // Apply config
                match key.as_str() {
                    "limb_bits" => {
                        config.limb_bits = value as u8;
                        config.validate().map_err(|e| AssemblerError::ConfigError {
                            line: line_num + 1,
                            source: e,
                        })?;
                    }
                    "data_limbs" => {
                        config.data_limbs = value as u8;
                        config.validate().map_err(|e| AssemblerError::ConfigError {
                            line: line_num + 1,
                            source: e,
                        })?;
                    }
                    "addr_limbs" => {
                        config.addr_limbs = value as u8;
                        config.validate().map_err(|e| AssemblerError::ConfigError {
                            line: line_num + 1,
                            source: e,
                        })?;
                    }
                    _ => {
                        return Err(AssemblerError::InvalidConfigValue {
                            line: line_num + 1,
                            key: key.clone(),
                            value: value.to_string(),
                        });
                    }
                }

                items.push(Item::ConfigDirective { key, value });
            } else {
                // Ignore other directives (e.g., .text, .data)
                items.push(Item::Empty);
            }
            continue;
        }

        // Parse as instruction
        let instr = parse_instruction_tokens(&tokens, line_num)?;
        items.push(Item::Instruction(instr));
        pc += 4;
    }

    Ok((items, labels, config))
}

/// Second pass: encode instructions
fn second_pass(items: &[Item], _labels: &HashMap<String, u64>) -> Result<Vec<u32>> {
    let mut code = Vec::new();

    for item in items.iter() {
        if let Item::Instruction(instr) = item {
            let encoded = encode(instr);
            code.push(encoded);
        }
    }

    Ok(code)
}

/// Parse tokens into an instruction
fn parse_instruction_tokens(tokens: &[Token], line_num: usize) -> Result<Instruction> {
    if tokens.is_empty() {
        return Err(AssemblerError::SyntaxError {
            line: line_num + 1,
            message: "Empty instruction".to_string(),
        });
    }

    let mnemonic = match &tokens[0] {
        Token::Identifier(s) => s.to_lowercase(),
        _ => {
            return Err(AssemblerError::SyntaxError {
                line: line_num + 1,
                message: format!("Expected instruction mnemonic, got {:?}", tokens[0]),
            });
        }
    };

    let operands = &tokens[1..];

    parse_mnemonic(&mnemonic, operands, line_num)
}

/// Parse instruction mnemonic and operands
fn parse_mnemonic(mnemonic: &str, operands: &[Token], line_num: usize) -> Result<Instruction> {
    match mnemonic {
        // ========== System Instructions ==========
        "ecall" => {
            expect_no_operands(operands, line_num)?;
            Ok(Instruction::Ecall)
        }
        "ebreak" => {
            expect_no_operands(operands, line_num)?;
            Ok(Instruction::Ebreak)
        }

        // ========== R-type Arithmetic ==========
        "add" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Add { rd, rs1, rs2 }),
        "sub" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Sub { rd, rs1, rs2 }),
        "mul" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Mul { rd, rs1, rs2 }),
        "mulh" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Mulh { rd, rs1, rs2 }),
        "div" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Div { rd, rs1, rs2 }),
        "divu" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Divu { rd, rs1, rs2 }),
        "rem" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Rem { rd, rs1, rs2 }),
        "remu" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Remu { rd, rs1, rs2 }),

        // ========== R-type Logical ==========
        "and" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::And { rd, rs1, rs2 }),
        "or" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Or { rd, rs1, rs2 }),
        "xor" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Xor { rd, rs1, rs2 }),

        // ========== R-type Shift ==========
        "sll" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Sll { rd, rs1, rs2 }),
        "srl" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Srl { rd, rs1, rs2 }),
        "sra" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Sra { rd, rs1, rs2 }),

        // ========== R-type Compare ==========
        "slt" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Slt { rd, rs1, rs2 }),
        "sltu" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Sltu { rd, rs1, rs2 }),
        "sge" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Sge { rd, rs1, rs2 }),
        "sgeu" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Sgeu { rd, rs1, rs2 }),
        "seq" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Seq { rd, rs1, rs2 }),
        "sne" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Sne { rd, rs1, rs2 }),

        // ========== R-type Conditional Move ==========
        "cmov" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Cmov { rd, rs1, rs2 }),
        "cmovz" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Cmovz { rd, rs1, rs2 }),
        "cmovnz" => parse_r_type(operands, line_num, |rd, rs1, rs2| Instruction::Cmovnz { rd, rs1, rs2 }),

        // ========== I-type Arithmetic ==========
        "addi" => parse_i_type(operands, line_num, |rd, rs1, imm| Instruction::Addi { rd, rs1, imm }),
        "xori" => parse_i_type(operands, line_num, |rd, rs1, imm| Instruction::Xori { rd, rs1, imm }),
        "ori" => parse_i_type(operands, line_num, |rd, rs1, imm| Instruction::Ori { rd, rs1, imm }),
        "andi" => parse_i_type(operands, line_num, |rd, rs1, imm| Instruction::Andi { rd, rs1, imm }),

        // ========== I-type Shift ==========
        "slli" => parse_shift_imm(operands, line_num, |rd, rs1, shamt| Instruction::Slli { rd, rs1, shamt }),
        "srli" => parse_shift_imm(operands, line_num, |rd, rs1, shamt| Instruction::Srli { rd, rs1, shamt }),
        "srai" => parse_shift_imm(operands, line_num, |rd, rs1, shamt| Instruction::Srai { rd, rs1, shamt }),

        // ========== Load ==========
        "lw" => parse_load(operands, line_num, |rd, rs1, imm| Instruction::Lw { rd, rs1, imm }),
        "lh" => parse_load(operands, line_num, |rd, rs1, imm| Instruction::Lh { rd, rs1, imm }),
        "lhu" => parse_load(operands, line_num, |rd, rs1, imm| Instruction::Lhu { rd, rs1, imm }),
        "lb" => parse_load(operands, line_num, |rd, rs1, imm| Instruction::Lb { rd, rs1, imm }),
        "lbu" => parse_load(operands, line_num, |rd, rs1, imm| Instruction::Lbu { rd, rs1, imm }),
        "ld" => parse_load(operands, line_num, |rd, rs1, imm| Instruction::Ld { rd, rs1, imm }),

        // ========== Store ==========
        "sw" => parse_store(operands, line_num, |rs1, rs2, imm| Instruction::Sw { rs1, rs2, imm }),
        "sh" => parse_store(operands, line_num, |rs1, rs2, imm| Instruction::Sh { rs1, rs2, imm }),
        "sb" => parse_store(operands, line_num, |rs1, rs2, imm| Instruction::Sb { rs1, rs2, imm }),
        "sd" => parse_store(operands, line_num, |rs1, rs2, imm| Instruction::Sd { rs1, rs2, imm }),

        // ========== Branch ==========
        "beq" => parse_branch(operands, line_num, |rs1, rs2, offset| Instruction::Beq { rs1, rs2, offset }),
        "bne" => parse_branch(operands, line_num, |rs1, rs2, offset| Instruction::Bne { rs1, rs2, offset }),
        "blt" => parse_branch(operands, line_num, |rs1, rs2, offset| Instruction::Blt { rs1, rs2, offset }),
        "bge" => parse_branch(operands, line_num, |rs1, rs2, offset| Instruction::Bge { rs1, rs2, offset }),
        "bltu" => parse_branch(operands, line_num, |rs1, rs2, offset| Instruction::Bltu { rs1, rs2, offset }),
        "bgeu" => parse_branch(operands, line_num, |rs1, rs2, offset| Instruction::Bgeu { rs1, rs2, offset }),

        // ========== Jump ==========
        "jal" => parse_jal(operands, line_num),
        "jalr" => parse_jalr(operands, line_num),

        _ => Err(AssemblerError::InvalidInstruction {
            line: line_num + 1,
            instruction: mnemonic.to_string(),
        }),
    }
}

// ========== Helper Functions ==========

fn expect_no_operands(operands: &[Token], line_num: usize) -> Result<()> {
    if !operands.is_empty() {
        return Err(AssemblerError::SyntaxError {
            line: line_num + 1,
            message: "Instruction takes no operands".to_string(),
        });
    }
    Ok(())
}

/// Parse R-type: rd, rs1, rs2
fn parse_r_type<F>(operands: &[Token], line_num: usize, constructor: F) -> Result<Instruction>
where
    F: FnOnce(zkir_spec::Register, zkir_spec::Register, zkir_spec::Register) -> Instruction,
{
    if operands.len() != 5 {
        return Err(AssemblerError::SyntaxError {
            line: line_num + 1,
            message: "R-type requires 3 operands: rd, rs1, rs2".to_string(),
        });
    }

    let rd = extract_register(&operands[0], line_num)?;
    expect_comma(&operands[1], line_num)?;
    let rs1 = extract_register(&operands[2], line_num)?;
    expect_comma(&operands[3], line_num)?;
    let rs2 = extract_register(&operands[4], line_num)?;

    Ok(constructor(rd, rs1, rs2))
}

/// Parse I-type: rd, rs1, imm
fn parse_i_type<F>(operands: &[Token], line_num: usize, constructor: F) -> Result<Instruction>
where
    F: FnOnce(zkir_spec::Register, zkir_spec::Register, i32) -> Instruction,
{
    if operands.len() != 5 {
        return Err(AssemblerError::SyntaxError {
            line: line_num + 1,
            message: "I-type requires 3 operands: rd, rs1, imm".to_string(),
        });
    }

    let rd = extract_register(&operands[0], line_num)?;
    expect_comma(&operands[1], line_num)?;
    let rs1 = extract_register(&operands[2], line_num)?;
    expect_comma(&operands[3], line_num)?;
    let imm = extract_number(&operands[4])? as i32;

    Ok(constructor(rd, rs1, imm))
}

/// Parse shift immediate: rd, rs1, shamt
fn parse_shift_imm<F>(operands: &[Token], line_num: usize, constructor: F) -> Result<Instruction>
where
    F: FnOnce(zkir_spec::Register, zkir_spec::Register, u8) -> Instruction,
{
    if operands.len() != 5 {
        return Err(AssemblerError::SyntaxError {
            line: line_num + 1,
            message: "Shift requires 3 operands: rd, rs1, shamt".to_string(),
        });
    }

    let rd = extract_register(&operands[0], line_num)?;
    expect_comma(&operands[1], line_num)?;
    let rs1 = extract_register(&operands[2], line_num)?;
    expect_comma(&operands[3], line_num)?;
    let shamt = extract_number(&operands[4])? as u8;

    Ok(constructor(rd, rs1, shamt))
}

/// Parse load: rd, offset(rs1)
fn parse_load<F>(operands: &[Token], line_num: usize, constructor: F) -> Result<Instruction>
where
    F: FnOnce(zkir_spec::Register, zkir_spec::Register, i32) -> Instruction,
{
    if operands.len() != 6 {
        return Err(AssemblerError::SyntaxError {
            line: line_num + 1,
            message: "Load requires format: rd, offset(rs1)".to_string(),
        });
    }

    let rd = extract_register(&operands[0], line_num)?;
    expect_comma(&operands[1], line_num)?;
    let offset = extract_number(&operands[2])? as i32;
    expect_lparen(&operands[3], line_num)?;
    let rs1 = extract_register(&operands[4], line_num)?;
    expect_rparen(&operands[5], line_num)?;

    Ok(constructor(rd, rs1, offset))
}

/// Parse store: rs2, offset(rs1)
fn parse_store<F>(operands: &[Token], line_num: usize, constructor: F) -> Result<Instruction>
where
    F: FnOnce(zkir_spec::Register, zkir_spec::Register, i32) -> Instruction,
{
    if operands.len() != 6 {
        return Err(AssemblerError::SyntaxError {
            line: line_num + 1,
            message: "Store requires format: rs2, offset(rs1)".to_string(),
        });
    }

    let rs2 = extract_register(&operands[0], line_num)?;
    expect_comma(&operands[1], line_num)?;
    let offset = extract_number(&operands[2])? as i32;
    expect_lparen(&operands[3], line_num)?;
    let rs1 = extract_register(&operands[4], line_num)?;
    expect_rparen(&operands[5], line_num)?;

    Ok(constructor(rs1, rs2, offset))
}

/// Parse branch: rs1, rs2, offset
fn parse_branch<F>(operands: &[Token], line_num: usize, constructor: F) -> Result<Instruction>
where
    F: FnOnce(zkir_spec::Register, zkir_spec::Register, i32) -> Instruction,
{
    if operands.len() != 5 {
        return Err(AssemblerError::SyntaxError {
            line: line_num + 1,
            message: "Branch requires 3 operands: rs1, rs2, offset".to_string(),
        });
    }

    let rs1 = extract_register(&operands[0], line_num)?;
    expect_comma(&operands[1], line_num)?;
    let rs2 = extract_register(&operands[2], line_num)?;
    expect_comma(&operands[3], line_num)?;
    let offset = extract_number(&operands[4])? as i32;

    Ok(constructor(rs1, rs2, offset))
}

/// Parse JAL: rd, offset
fn parse_jal(operands: &[Token], line_num: usize) -> Result<Instruction> {
    if operands.len() != 3 {
        return Err(AssemblerError::SyntaxError {
            line: line_num + 1,
            message: "JAL requires 2 operands: rd, offset".to_string(),
        });
    }

    let rd = extract_register(&operands[0], line_num)?;
    expect_comma(&operands[1], line_num)?;
    let offset = extract_number(&operands[2])? as i32;

    Ok(Instruction::Jal { rd, offset })
}

/// Parse JALR: rd, rs1, offset
fn parse_jalr(operands: &[Token], line_num: usize) -> Result<Instruction> {
    if operands.len() != 5 {
        return Err(AssemblerError::SyntaxError {
            line: line_num + 1,
            message: "JALR requires 3 operands: rd, rs1, offset".to_string(),
        });
    }

    let rd = extract_register(&operands[0], line_num)?;
    expect_comma(&operands[1], line_num)?;
    let rs1 = extract_register(&operands[2], line_num)?;
    expect_comma(&operands[3], line_num)?;
    let imm = extract_number(&operands[4])? as i32;

    Ok(Instruction::Jalr { rd, rs1, imm })
}

// ========== Token Extraction ==========

fn extract_register(token: &Token, line_num: usize) -> Result<zkir_spec::Register> {
    match token {
        Token::Register(name) => parse_register(name),
        _ => Err(AssemblerError::SyntaxError {
            line: line_num + 1,
            message: format!("Expected register, got {:?}", token),
        }),
    }
}

fn expect_comma(token: &Token, line_num: usize) -> Result<()> {
    match token {
        Token::Comma => Ok(()),
        _ => Err(AssemblerError::SyntaxError {
            line: line_num + 1,
            message: format!("Expected comma, got {:?}", token),
        }),
    }
}

fn expect_lparen(token: &Token, line_num: usize) -> Result<()> {
    match token {
        Token::LParen => Ok(()),
        _ => Err(AssemblerError::SyntaxError {
            line: line_num + 1,
            message: format!("Expected '(', got {:?}", token),
        }),
    }
}

fn expect_rparen(token: &Token, line_num: usize) -> Result<()> {
    match token {
        Token::RParen => Ok(()),
        _ => Err(AssemblerError::SyntaxError {
            line: line_num + 1,
            message: format!("Expected ')', got {:?}", token),
        }),
    }
}

/// Check if a label name is valid
fn is_valid_label(label: &str) -> bool {
    if label.is_empty() {
        return false;
    }

    let first = label.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    label.chars().all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_simple() {
        let source = r#"
            ecall
            ebreak
        "#;

        let program = assemble(source).unwrap();
        assert_eq!(program.code.len(), 2);
    }

    #[test]
    fn test_assemble_with_config() {
        let source = r#"
            .config limb_bits 20
            .config data_limbs 2
            .config addr_limbs 2

            add r1, r2, r3
            ecall
        "#;

        let program = assemble(source).unwrap();
        assert_eq!(program.code.len(), 2);
    }

    #[test]
    fn test_assemble_with_labels() {
        let source = r#"
        start:
            add a0, zero, zero
            beq a0, zero, 8
            add a1, zero, zero
        end:
            ebreak
        "#;

        let program = assemble(source).unwrap();
        assert_eq!(program.code.len(), 4);
    }

    #[test]
    fn test_is_valid_label() {
        assert!(is_valid_label("main"));
        assert!(is_valid_label("_start"));
        assert!(is_valid_label("loop_1"));
        assert!(!is_valid_label("123"));
        assert!(!is_valid_label(""));
    }
}
