//! Assembly parser for ZK IR v2.2
//!
//! Parses tokenized assembly into Instruction types.

use zkir_spec::{Instruction, Register};
use crate::error::{AssemblerError, Result};
use crate::lexer::{Token, Lexer};

/// Parse a single instruction from assembly text
pub fn parse_instruction(text: &str) -> Result<Instruction> {
    let mut lexer = Lexer::new(text);
    let tokens = lexer.tokenize()
        .map_err(|e| AssemblerError::SyntaxError {
            line: lexer.line(),
            column: lexer.col(),
            message: e,
        })?;

    // Filter out newlines and EOF
    let tokens: Vec<Token> = tokens.into_iter()
        .filter(|t| !matches!(t, Token::Newline | Token::Eof))
        .collect();

    if tokens.is_empty() {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: "Empty instruction".to_string(),
        });
    }

    parse_tokens(&tokens)
}

/// Parse register name (v2.2 RISC-V calling convention)
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

        // R5-R7 - temporaries t0-t2
        "t0" | "r5" => Ok(Register::R5),
        "t1" | "r6" => Ok(Register::R6),
        "t2" | "r7" => Ok(Register::R7),

        // R8 - frame pointer (also s0)
        "fp" | "s0" | "r8" => Ok(Register::R8),

        // R9 - saved register s1
        "s1" | "r9" => Ok(Register::R9),

        // R10-R17 - arguments/return values a0-a7
        "a0" | "r10" => Ok(Register::R10),
        "a1" | "r11" => Ok(Register::R11),
        "a2" | "r12" => Ok(Register::R12),
        "a3" | "r13" => Ok(Register::R13),
        "a4" | "r14" => Ok(Register::R14),
        "a5" | "r15" => Ok(Register::R15),
        "a6" | "r16" => Ok(Register::R16),
        "a7" | "r17" => Ok(Register::R17),

        // R18-R27 - saved registers s2-s11
        "s2" | "r18" => Ok(Register::R18),
        "s3" | "r19" => Ok(Register::R19),
        "s4" | "r20" => Ok(Register::R20),
        "s5" | "r21" => Ok(Register::R21),
        "s6" | "r22" => Ok(Register::R22),
        "s7" | "r23" => Ok(Register::R23),
        "s8" | "r24" => Ok(Register::R24),
        "s9" | "r25" => Ok(Register::R25),
        "s10" | "r26" => Ok(Register::R26),
        "s11" | "r27" => Ok(Register::R27),

        // R28-R31 - temporaries t3-t6
        "t3" | "r28" => Ok(Register::R28),
        "t4" | "r29" => Ok(Register::R29),
        "t5" | "r30" => Ok(Register::R30),
        "t6" | "r31" => Ok(Register::R31),

        _ => Err(AssemblerError::InvalidRegister(name.to_string())),
    }
}

/// Parse tokens into an instruction
fn parse_tokens(tokens: &[Token]) -> Result<Instruction> {
    if tokens.is_empty() {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: "Empty instruction".to_string(),
        });
    }

    let mnemonic = match &tokens[0] {
        Token::Identifier(s) => s.to_lowercase(),
        _ => return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Expected instruction mnemonic, got {:?}", tokens[0]),
        }),
    };

    let operands = &tokens[1..];

    parse_mnemonic(&mnemonic, operands)
}

/// Parse instruction mnemonic and operands
fn parse_mnemonic(mnemonic: &str, operands: &[Token]) -> Result<Instruction> {
    match mnemonic {
        // ========== System Instructions ==========
        "halt" => {
            expect_no_operands(mnemonic, operands)?;
            Ok(Instruction::Halt)
        }
        "ecall" => {
            expect_no_operands(mnemonic, operands)?;
            Ok(Instruction::Ecall)
        }
        "ebreak" => {
            expect_no_operands(mnemonic, operands)?;
            Ok(Instruction::Ebreak)
        }

        // ========== R-type Arithmetic ==========
        "add" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Add { rd, rs1, rs2 }),
        "sub" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Sub { rd, rs1, rs2 }),
        "mul" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Mul { rd, rs1, rs2 }),
        "mulh" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Mulh { rd, rs1, rs2 }),
        "mulhu" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Mulhu { rd, rs1, rs2 }),
        "mulhsu" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Mulhsu { rd, rs1, rs2 }),
        "div" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Div { rd, rs1, rs2 }),
        "divu" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Divu { rd, rs1, rs2 }),
        "rem" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Rem { rd, rs1, rs2 }),
        "remu" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Remu { rd, rs1, rs2 }),

        // ========== R-type Logic ==========
        "and" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::And { rd, rs1, rs2 }),
        "andn" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Andn { rd, rs1, rs2 }),
        "or" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Or { rd, rs1, rs2 }),
        "orn" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Orn { rd, rs1, rs2 }),
        "xor" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Xor { rd, rs1, rs2 }),
        "xnor" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Xnor { rd, rs1, rs2 }),

        // ========== R-type Shift ==========
        "sll" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Sll { rd, rs1, rs2 }),
        "srl" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Srl { rd, rs1, rs2 }),
        "sra" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Sra { rd, rs1, rs2 }),
        "rol" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Rol { rd, rs1, rs2 }),
        "ror" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Ror { rd, rs1, rs2 }),

        // ========== R-type Compare ==========
        "slt" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Slt { rd, rs1, rs2 }),
        "sltu" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Sltu { rd, rs1, rs2 }),
        "min" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Min { rd, rs1, rs2 }),
        "max" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Max { rd, rs1, rs2 }),
        "minu" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Minu { rd, rs1, rs2 }),
        "maxu" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Maxu { rd, rs1, rs2 }),

        // ========== Bit Manipulation ==========
        "clz" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Clz { rd, rs1, rs2 }),
        "ctz" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Ctz { rd, rs1, rs2 }),
        "cpop" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Cpop { rd, rs1, rs2 }),
        "rev8" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Rev8 { rd, rs1, rs2 }),

        // ========== Conditional Move ==========
        "cmovz" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Cmovz { rd, rs1, rs2 }),
        "cmovnz" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Cmovnz { rd, rs1, rs2 }),

        // ========== Field Arithmetic ==========
        "fadd" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Fadd { rd, rs1, rs2 }),
        "fsub" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Fsub { rd, rs1, rs2 }),
        "fmul" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Fmul { rd, rs1, rs2 }),
        "fneg" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Fneg { rd, rs1, rs2 }),
        "finv" => parse_r_type(mnemonic, operands, |rd, rs1, rs2| Instruction::Finv { rd, rs1, rs2 }),

        // ========== I-type Arithmetic Immediate ==========
        "addi" => parse_i_type(mnemonic, operands, |rd, rs1, imm| Instruction::Addi { rd, rs1, imm }),
        "slti" => parse_i_type(mnemonic, operands, |rd, rs1, imm| Instruction::Slti { rd, rs1, imm }),
        "sltiu" => parse_i_type(mnemonic, operands, |rd, rs1, imm| Instruction::Sltiu { rd, rs1, imm }),
        "xori" => parse_i_type(mnemonic, operands, |rd, rs1, imm| Instruction::Xori { rd, rs1, imm }),
        "ori" => parse_i_type(mnemonic, operands, |rd, rs1, imm| Instruction::Ori { rd, rs1, imm }),
        "andi" => parse_i_type(mnemonic, operands, |rd, rs1, imm| Instruction::Andi { rd, rs1, imm }),

        // ========== I-type Shift Immediate ==========
        "slli" => parse_shift_imm(mnemonic, operands, |rd, rs1, shamt| Instruction::Slli { rd, rs1, shamt }),
        "srli" => parse_shift_imm(mnemonic, operands, |rd, rs1, shamt| Instruction::Srli { rd, rs1, shamt }),
        "srai" => parse_shift_imm(mnemonic, operands, |rd, rs1, shamt| Instruction::Srai { rd, rs1, shamt }),

        // ========== Load Instructions ==========
        "lw" => parse_load(mnemonic, operands, |rd, rs1, imm| Instruction::Lw { rd, rs1, imm }),
        "lh" => parse_load(mnemonic, operands, |rd, rs1, imm| Instruction::Lh { rd, rs1, imm }),
        "lhu" => parse_load(mnemonic, operands, |rd, rs1, imm| Instruction::Lhu { rd, rs1, imm }),
        "lb" => parse_load(mnemonic, operands, |rd, rs1, imm| Instruction::Lb { rd, rs1, imm }),
        "lbu" => parse_load(mnemonic, operands, |rd, rs1, imm| Instruction::Lbu { rd, rs1, imm }),

        // ========== Store Instructions ==========
        "sw" => parse_store(mnemonic, operands, |rs1, rs2, imm| Instruction::Sw { rs1, rs2, imm }),
        "sh" => parse_store(mnemonic, operands, |rs1, rs2, imm| Instruction::Sh { rs1, rs2, imm }),
        "sb" => parse_store(mnemonic, operands, |rs1, rs2, imm| Instruction::Sb { rs1, rs2, imm }),

        // ========== Branch Instructions ==========
        "beq" => parse_branch(mnemonic, operands, |rs1, rs2, imm| Instruction::Beq { rs1, rs2, imm }),
        "bne" => parse_branch(mnemonic, operands, |rs1, rs2, imm| Instruction::Bne { rs1, rs2, imm }),
        "blt" => parse_branch(mnemonic, operands, |rs1, rs2, imm| Instruction::Blt { rs1, rs2, imm }),
        "bge" => parse_branch(mnemonic, operands, |rs1, rs2, imm| Instruction::Bge { rs1, rs2, imm }),
        "bltu" => parse_branch(mnemonic, operands, |rs1, rs2, imm| Instruction::Bltu { rs1, rs2, imm }),
        "bgeu" => parse_branch(mnemonic, operands, |rs1, rs2, imm| Instruction::Bgeu { rs1, rs2, imm }),

        // ========== Jump Instructions ==========
        "jal" => parse_jal(mnemonic, operands),
        "jalr" => parse_jalr(mnemonic, operands),

        // ========== Upper Immediate ==========
        "lui" => parse_u_type(mnemonic, operands, |rd, imm| Instruction::Lui { rd, imm }),
        "auipc" => parse_u_type(mnemonic, operands, |rd, imm| Instruction::Auipc { rd, imm }),

        // ========== ZK Operations ==========
        "read" => parse_zk_unary(mnemonic, operands, |rd| Instruction::Read { rd }),
        "write" => parse_zk_unary_rs(mnemonic, operands, |rs1| Instruction::Write { rs1 }),
        "hint" => parse_zk_unary(mnemonic, operands, |rd| Instruction::Hint { rd }),
        "commit" => parse_zk_unary_rs(mnemonic, operands, |rs1| Instruction::Commit { rs1 }),
        "assert_eq" => parse_zk_binary(mnemonic, operands, |rs1, rs2| Instruction::AssertEq { rs1, rs2 }),
        "assert_ne" => parse_zk_binary(mnemonic, operands, |rs1, rs2| Instruction::AssertNe { rs1, rs2 }),
        "assert_zero" => parse_zk_unary_rs(mnemonic, operands, |rs1| Instruction::AssertZero { rs1 }),
        "range_check" => parse_zk_range_check(mnemonic, operands),
        "debug" => parse_zk_unary_rs(mnemonic, operands, |rs1| Instruction::Debug { rs1 }),

        _ => Err(AssemblerError::UnknownInstruction(mnemonic.to_string())),
    }
}

// ========== Helper Functions ==========

fn expect_no_operands(mnemonic: &str, operands: &[Token]) -> Result<()> {
    if !operands.is_empty() {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} takes no operands", mnemonic),
        });
    }
    Ok(())
}

/// Parse R-type: rd, rs1, rs2
fn parse_r_type<F>(mnemonic: &str, operands: &[Token], constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register, Register, Register) -> Instruction,
{
    let (rd, rs1, rs2) = parse_three_regs(mnemonic, operands)?;
    Ok(constructor(rd, rs1, rs2))
}

/// Parse I-type: rd, rs1, imm
fn parse_i_type<F>(mnemonic: &str, operands: &[Token], constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register, Register, i16) -> Instruction,
{
    // rd, comma, rs1, comma, imm
    if operands.len() != 5 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 3 operands: rd, rs1, imm", mnemonic),
        });
    }

    let rd = extract_register(&operands[0])?;
    expect_comma(&operands[1])?;
    let rs1 = extract_register(&operands[2])?;
    expect_comma(&operands[3])?;
    let imm = extract_imm12(&operands[4])?;

    Ok(constructor(rd, rs1, imm))
}

/// Parse shift immediate: rd, rs1, shamt
fn parse_shift_imm<F>(mnemonic: &str, operands: &[Token], constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register, Register, u8) -> Instruction,
{
    if operands.len() != 5 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 3 operands: rd, rs1, shamt", mnemonic),
        });
    }

    let rd = extract_register(&operands[0])?;
    expect_comma(&operands[1])?;
    let rs1 = extract_register(&operands[2])?;
    expect_comma(&operands[3])?;
    let shamt = extract_shamt(&operands[4])?;

    Ok(constructor(rd, rs1, shamt))
}

/// Parse load: rd, offset(rs1)
fn parse_load<F>(mnemonic: &str, operands: &[Token], constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register, Register, i16) -> Instruction,
{
    // rd, comma, offset, lparen, rs1, rparen
    if operands.len() != 6 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires format: rd, offset(rs1)", mnemonic),
        });
    }

    let rd = extract_register(&operands[0])?;
    expect_comma(&operands[1])?;
    let offset = extract_imm12(&operands[2])?;
    expect_lparen(&operands[3])?;
    let rs1 = extract_register(&operands[4])?;
    expect_rparen(&operands[5])?;

    Ok(constructor(rd, rs1, offset))
}

/// Parse store: rs2, offset(rs1)
fn parse_store<F>(mnemonic: &str, operands: &[Token], constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register, Register, i16) -> Instruction,
{
    // rs2, comma, offset, lparen, rs1, rparen
    if operands.len() != 6 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires format: rs2, offset(rs1)", mnemonic),
        });
    }

    let rs2 = extract_register(&operands[0])?;
    expect_comma(&operands[1])?;
    let offset = extract_imm12(&operands[2])?;
    expect_lparen(&operands[3])?;
    let rs1 = extract_register(&operands[4])?;
    expect_rparen(&operands[5])?;

    Ok(constructor(rs1, rs2, offset))
}

/// Parse branch: rs1, rs2, offset
fn parse_branch<F>(mnemonic: &str, operands: &[Token], constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register, Register, i16) -> Instruction,
{
    if operands.len() != 5 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 3 operands: rs1, rs2, offset", mnemonic),
        });
    }

    let rs1 = extract_register(&operands[0])?;
    expect_comma(&operands[1])?;
    let rs2 = extract_register(&operands[2])?;
    expect_comma(&operands[3])?;
    let offset = extract_branch_offset(&operands[4])?;

    Ok(constructor(rs1, rs2, offset))
}

/// Parse JAL: rd, offset
fn parse_jal(mnemonic: &str, operands: &[Token]) -> Result<Instruction> {
    if operands.len() != 3 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 2 operands: rd, offset", mnemonic),
        });
    }

    let rd = extract_register(&operands[0])?;
    expect_comma(&operands[1])?;
    let imm = extract_jump_offset(&operands[2])?;

    Ok(Instruction::Jal { rd, imm })
}

/// Parse JALR: rd, rs1, offset
fn parse_jalr(mnemonic: &str, operands: &[Token]) -> Result<Instruction> {
    if operands.len() != 5 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 3 operands: rd, rs1, offset", mnemonic),
        });
    }

    let rd = extract_register(&operands[0])?;
    expect_comma(&operands[1])?;
    let rs1 = extract_register(&operands[2])?;
    expect_comma(&operands[3])?;
    let imm = extract_imm12(&operands[4])?;

    Ok(Instruction::Jalr { rd, rs1, imm })
}

/// Parse U-type: rd, imm
fn parse_u_type<F>(mnemonic: &str, operands: &[Token], constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register, i32) -> Instruction,
{
    if operands.len() != 3 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 2 operands: rd, imm", mnemonic),
        });
    }

    let rd = extract_register(&operands[0])?;
    expect_comma(&operands[1])?;
    let imm = extract_imm20(&operands[2])?;

    Ok(constructor(rd, imm))
}

/// Parse ZK unary (rd only): rd
fn parse_zk_unary<F>(mnemonic: &str, operands: &[Token], constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register) -> Instruction,
{
    if operands.len() != 1 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 1 operand: rd", mnemonic),
        });
    }

    let rd = extract_register(&operands[0])?;
    Ok(constructor(rd))
}

/// Parse ZK unary (rs1 only): rs1
fn parse_zk_unary_rs<F>(mnemonic: &str, operands: &[Token], constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register) -> Instruction,
{
    if operands.len() != 1 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 1 operand: rs1", mnemonic),
        });
    }

    let rs1 = extract_register(&operands[0])?;
    Ok(constructor(rs1))
}

/// Parse ZK binary: rs1, rs2
fn parse_zk_binary<F>(mnemonic: &str, operands: &[Token], constructor: F) -> Result<Instruction>
where
    F: FnOnce(Register, Register) -> Instruction,
{
    if operands.len() != 3 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 2 operands: rs1, rs2", mnemonic),
        });
    }

    let rs1 = extract_register(&operands[0])?;
    expect_comma(&operands[1])?;
    let rs2 = extract_register(&operands[2])?;

    Ok(constructor(rs1, rs2))
}

/// Parse range_check: rs1, bits
fn parse_zk_range_check(mnemonic: &str, operands: &[Token]) -> Result<Instruction> {
    if operands.len() != 3 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 2 operands: rs1, bits", mnemonic),
        });
    }

    let rs1 = extract_register(&operands[0])?;
    expect_comma(&operands[1])?;
    let bits = extract_shamt(&operands[2])?; // reuse shamt extraction (0-31 range)

    Ok(Instruction::RangeCheck { rs1, bits })
}

/// Parse three registers: rd, rs1, rs2
fn parse_three_regs(mnemonic: &str, operands: &[Token]) -> Result<(Register, Register, Register)> {
    if operands.len() != 5 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("{} requires 3 operands: rd, rs1, rs2", mnemonic),
        });
    }

    let rd = extract_register(&operands[0])?;
    expect_comma(&operands[1])?;
    let rs1 = extract_register(&operands[2])?;
    expect_comma(&operands[3])?;
    let rs2 = extract_register(&operands[4])?;

    Ok((rd, rs1, rs2))
}

// ========== Token Extraction ==========

fn extract_register(token: &Token) -> Result<Register> {
    match token {
        Token::Register(name) => parse_register(name),
        _ => Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Expected register, got {:?}", token),
        }),
    }
}

fn extract_imm12(token: &Token) -> Result<i16> {
    let value = match token {
        Token::Number(n) => *n as i32,
        Token::HexNumber(n) => *n as i32,
        Token::BinNumber(n) => *n as i32,
        _ => return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Expected immediate value, got {:?}", token),
        }),
    };

    if value < -2048 || value > 2047 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Immediate value {} out of range (must be -2048 to 2047)", value),
        });
    }

    Ok(value as i16)
}

fn extract_imm20(token: &Token) -> Result<i32> {
    match token {
        Token::Number(n) => Ok(*n as i32),
        Token::HexNumber(n) => Ok(*n as i32),
        Token::BinNumber(n) => Ok(*n as i32),
        _ => Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Expected immediate value, got {:?}", token),
        }),
    }
}

fn extract_shamt(token: &Token) -> Result<u8> {
    let value = match token {
        Token::Number(n) => *n,
        Token::HexNumber(n) => *n as i64,
        Token::BinNumber(n) => *n as i64,
        _ => return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Expected shift amount, got {:?}", token),
        }),
    };

    if value < 0 || value > 31 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Shift amount must be 0-31, got {}", value),
        });
    }

    Ok(value as u8)
}

fn extract_branch_offset(token: &Token) -> Result<i16> {
    let value = match token {
        Token::Number(n) => *n as i32,
        Token::HexNumber(n) => *n as i32,
        Token::BinNumber(n) => *n as i32,
        _ => return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Expected branch offset, got {:?}", token),
        }),
    };

    if value < -4096 || value > 4094 {
        return Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Branch offset {} out of range (must be -4096 to 4094)", value),
        });
    }

    Ok(value as i16)
}

fn extract_jump_offset(token: &Token) -> Result<i32> {
    match token {
        Token::Number(n) => Ok(*n as i32),
        Token::HexNumber(n) => Ok(*n as i32),
        Token::BinNumber(n) => Ok(*n as i32),
        _ => Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Expected jump offset, got {:?}", token),
        }),
    }
}

fn expect_comma(token: &Token) -> Result<()> {
    match token {
        Token::Comma => Ok(()),
        _ => Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Expected comma, got {:?}", token),
        }),
    }
}

fn expect_lparen(token: &Token) -> Result<()> {
    match token {
        Token::LParen => Ok(()),
        _ => Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Expected '(', got {:?}", token),
        }),
    }
}

fn expect_rparen(token: &Token) -> Result<()> {
    match token {
        Token::RParen => Ok(()),
        _ => Err(AssemblerError::SyntaxError {
            line: 0,
            column: 0,
            message: format!("Expected ')', got {:?}", token),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Register Parsing Tests ==========
    #[test]
    fn test_parse_register_v2_2() {
        // Zero register
        assert_eq!(parse_register("zero").unwrap(), Register::R0);
        assert_eq!(parse_register("r0").unwrap(), Register::R0);

        // Return address
        assert_eq!(parse_register("ra").unwrap(), Register::R1);
        assert_eq!(parse_register("r1").unwrap(), Register::R1);

        // Stack pointer
        assert_eq!(parse_register("sp").unwrap(), Register::R2);
        assert_eq!(parse_register("r2").unwrap(), Register::R2);

        // Arguments (v2.2: a0=R10, a1=R11, etc.)
        assert_eq!(parse_register("a0").unwrap(), Register::R10);
        assert_eq!(parse_register("r10").unwrap(), Register::R10);
        assert_eq!(parse_register("a1").unwrap(), Register::R11);
        assert_eq!(parse_register("a7").unwrap(), Register::R17);

        // Temporaries
        assert_eq!(parse_register("t0").unwrap(), Register::R5);
        assert_eq!(parse_register("t1").unwrap(), Register::R6);
        assert_eq!(parse_register("t2").unwrap(), Register::R7);
        assert_eq!(parse_register("t3").unwrap(), Register::R28);
        assert_eq!(parse_register("t6").unwrap(), Register::R31);

        // Saved registers
        assert_eq!(parse_register("s0").unwrap(), Register::R8);
        assert_eq!(parse_register("fp").unwrap(), Register::R8);
        assert_eq!(parse_register("s1").unwrap(), Register::R9);
        assert_eq!(parse_register("s11").unwrap(), Register::R27);
    }

    // ========== System Instructions ==========
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

    // ========== R-type Arithmetic ==========
    #[test]
    fn test_parse_add() {
        let instr = parse_instruction("add a0, a1, a2").unwrap();
        assert_eq!(instr, Instruction::Add {
            rd: Register::R10,
            rs1: Register::R11,
            rs2: Register::R12,
        });
    }

    #[test]
    fn test_parse_sub() {
        let instr = parse_instruction("sub t0, t1, t2").unwrap();
        assert_eq!(instr, Instruction::Sub {
            rd: Register::R5,
            rs1: Register::R6,
            rs2: Register::R7,
        });
    }

    #[test]
    fn test_parse_mul() {
        let instr = parse_instruction("mul s2, s3, s4").unwrap();
        assert_eq!(instr, Instruction::Mul {
            rd: Register::R18,
            rs1: Register::R19,
            rs2: Register::R20,
        });
    }

    // ========== Field Arithmetic ==========
    #[test]
    fn test_parse_fadd() {
        let instr = parse_instruction("fadd a0, a1, a2").unwrap();
        assert_eq!(instr, Instruction::Fadd {
            rd: Register::R10,
            rs1: Register::R11,
            rs2: Register::R12,
        });
    }

    #[test]
    fn test_parse_fneg() {
        let instr = parse_instruction("fneg a0, a1, a2").unwrap();
        assert_eq!(instr, Instruction::Fneg {
            rd: Register::R10,
            rs1: Register::R11,
            rs2: Register::R12,
        });
    }

    // ========== I-type Instructions ==========
    #[test]
    fn test_parse_addi() {
        let instr = parse_instruction("addi a0, a1, 100").unwrap();
        assert_eq!(instr, Instruction::Addi {
            rd: Register::R10,
            rs1: Register::R11,
            imm: 100,
        });
    }

    #[test]
    fn test_parse_slli() {
        let instr = parse_instruction("slli a0, a1, 4").unwrap();
        assert_eq!(instr, Instruction::Slli {
            rd: Register::R10,
            rs1: Register::R11,
            shamt: 4,
        });
    }

    // ========== Load/Store ==========
    #[test]
    fn test_parse_lw() {
        let instr = parse_instruction("lw a0, 16(sp)").unwrap();
        assert_eq!(instr, Instruction::Lw {
            rd: Register::R10,
            rs1: Register::R2,
            imm: 16,
        });
    }

    #[test]
    fn test_parse_sw() {
        let instr = parse_instruction("sw a0, 16(sp)").unwrap();
        assert_eq!(instr, Instruction::Sw {
            rs1: Register::R2,
            rs2: Register::R10,
            imm: 16,
        });
    }

    // ========== Branches ==========
    #[test]
    fn test_parse_beq() {
        let instr = parse_instruction("beq a0, a1, 16").unwrap();
        assert_eq!(instr, Instruction::Beq {
            rs1: Register::R10,
            rs2: Register::R11,
            imm: 16,
        });
    }

    // ========== ZK Operations ==========
    #[test]
    fn test_parse_read() {
        let instr = parse_instruction("read a0").unwrap();
        assert_eq!(instr, Instruction::Read {
            rd: Register::R10,
        });
    }

    #[test]
    fn test_parse_write() {
        let instr = parse_instruction("write a0").unwrap();
        assert_eq!(instr, Instruction::Write {
            rs1: Register::R10,
        });
    }
}
