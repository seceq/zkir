//! Instruction representation and encoding for ZK IR.

use serde::{Deserialize, Serialize};
use crate::opcode::Opcode;
use crate::register::{Register, FieldRegister};
use crate::error::ZkIrError;

/// A single ZK IR instruction.
///
/// Instructions are encoded as 64-bit values with the following format:
/// ```text
/// ┌────────┬────────┬────────┬────────┬────────────────────────────┐
/// │ Opcode │  Dst   │  Src1  │  Src2  │       Immediate            │
/// │ 8 bits │ 8 bits │ 8 bits │ 8 bits │        32 bits             │
/// └────────┴────────┴────────┴────────┴────────────────────────────┘
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    // ============ Arithmetic ============
    /// ADD dst, src1, src2: dst = src1 + src2
    Add { dst: Register, src1: Register, src2: Register },
    /// SUB dst, src1, src2: dst = src1 - src2
    Sub { dst: Register, src1: Register, src2: Register },
    /// MUL dst, src1, src2: dst = src1 * src2
    Mul { dst: Register, src1: Register, src2: Register },
    /// DIV dst, src1, src2: dst = src1 / src2 (unsigned)
    Div { dst: Register, src1: Register, src2: Register },
    /// SDIV dst, src1, src2: dst = src1 / src2 (signed)
    SDiv { dst: Register, src1: Register, src2: Register },
    /// MOD dst, src1, src2: dst = src1 % src2 (unsigned)
    Mod { dst: Register, src1: Register, src2: Register },
    /// SMOD dst, src1, src2: dst = src1 % src2 (signed)
    SMod { dst: Register, src1: Register, src2: Register },
    /// NEG dst, src1: dst = -src1
    Neg { dst: Register, src1: Register },

    // ============ Immediate Arithmetic ============
    /// ADDI dst, src1, imm: dst = src1 + imm
    AddI { dst: Register, src1: Register, imm: i32 },
    /// SUBI dst, src1, imm: dst = src1 - imm
    SubI { dst: Register, src1: Register, imm: i32 },
    /// MULI dst, src1, imm: dst = src1 * imm
    MulI { dst: Register, src1: Register, imm: i32 },

    // ============ Logic ============
    /// AND dst, src1, src2: dst = src1 & src2
    And { dst: Register, src1: Register, src2: Register },
    /// OR dst, src1, src2: dst = src1 | src2
    Or { dst: Register, src1: Register, src2: Register },
    /// XOR dst, src1, src2: dst = src1 ^ src2
    Xor { dst: Register, src1: Register, src2: Register },
    /// NOT dst, src1: dst = ~src1
    Not { dst: Register, src1: Register },
    /// SHL dst, src1, src2: dst = src1 << src2
    Shl { dst: Register, src1: Register, src2: Register },
    /// SHR dst, src1, src2: dst = src1 >> src2 (logical)
    Shr { dst: Register, src1: Register, src2: Register },
    /// SAR dst, src1, src2: dst = src1 >> src2 (arithmetic)
    Sar { dst: Register, src1: Register, src2: Register },

    // ============ Comparison ============
    /// EQ dst, src1, src2: dst = (src1 == src2) ? 1 : 0
    Eq { dst: Register, src1: Register, src2: Register },
    /// NE dst, src1, src2: dst = (src1 != src2) ? 1 : 0
    Ne { dst: Register, src1: Register, src2: Register },
    /// LT dst, src1, src2: dst = (src1 < src2) ? 1 : 0 (signed)
    Lt { dst: Register, src1: Register, src2: Register },
    /// LE dst, src1, src2: dst = (src1 <= src2) ? 1 : 0 (signed)
    Le { dst: Register, src1: Register, src2: Register },
    /// GT dst, src1, src2: dst = (src1 > src2) ? 1 : 0 (signed)
    Gt { dst: Register, src1: Register, src2: Register },
    /// GE dst, src1, src2: dst = (src1 >= src2) ? 1 : 0 (signed)
    Ge { dst: Register, src1: Register, src2: Register },
    /// LTU dst, src1, src2: dst = (src1 < src2) ? 1 : 0 (unsigned)
    Ltu { dst: Register, src1: Register, src2: Register },
    /// GEU dst, src1, src2: dst = (src1 >= src2) ? 1 : 0 (unsigned)
    Geu { dst: Register, src1: Register, src2: Register },

    // ============ Memory ============
    /// LOAD dst, offset(base): dst = memory[base + offset]
    Load { dst: Register, base: Register, offset: i32 },
    /// STORE offset(base), src: memory[base + offset] = src
    Store { src: Register, base: Register, offset: i32 },
    /// LOAD8 dst, offset(base): dst = memory[base + offset] & 0xFF
    Load8 { dst: Register, base: Register, offset: i32 },
    /// LOAD16 dst, offset(base): dst = memory[base + offset] & 0xFFFF
    Load16 { dst: Register, base: Register, offset: i32 },
    /// STORE8 offset(base), src: memory[base + offset] = src & 0xFF
    Store8 { src: Register, base: Register, offset: i32 },
    /// STORE16 offset(base), src: memory[base + offset] = src & 0xFFFF
    Store16 { src: Register, base: Register, offset: i32 },

    // ============ Control Flow ============
    /// JMP target: pc = target
    Jmp { target: u32 },
    /// JMPI src: pc = src
    JmpI { src: Register },
    /// BEQ src1, src2, target: if (src1 == src2) pc = target
    Beq { src1: Register, src2: Register, target: u32 },
    /// BNE src1, src2, target: if (src1 != src2) pc = target
    Bne { src1: Register, src2: Register, target: u32 },
    /// BLT src1, src2, target: if (src1 < src2) pc = target (signed)
    Blt { src1: Register, src2: Register, target: u32 },
    /// BGE src1, src2, target: if (src1 >= src2) pc = target (signed)
    Bge { src1: Register, src2: Register, target: u32 },
    /// BLTU src1, src2, target: if (src1 < src2) pc = target (unsigned)
    Bltu { src1: Register, src2: Register, target: u32 },
    /// BGEU src1, src2, target: if (src1 >= src2) pc = target (unsigned)
    Bgeu { src1: Register, src2: Register, target: u32 },

    // ============ Function Calls ============
    /// CALL target: push(pc+1), pc = target
    Call { target: u32 },
    /// CALLI src: push(pc+1), pc = src
    CallI { src: Register },
    /// RET: pc = pop()
    Ret,

    // ============ Constants ============
    /// LI dst, imm: dst = imm (32-bit)
    Li { dst: Register, imm: u32 },
    /// LUI dst, imm: dst = imm << 32
    Lui { dst: Register, imm: u32 },
    /// MOV dst, src: dst = src
    Mov { dst: Register, src: Register },

    // ============ Field Operations ============
    /// FADD fdst, fsrc1, fsrc2: fdst = fsrc1 + fsrc2 (field)
    FAdd { dst: FieldRegister, src1: FieldRegister, src2: FieldRegister },
    /// FSUB fdst, fsrc1, fsrc2: fdst = fsrc1 - fsrc2 (field)
    FSub { dst: FieldRegister, src1: FieldRegister, src2: FieldRegister },
    /// FMUL fdst, fsrc1, fsrc2: fdst = fsrc1 * fsrc2 (field)
    FMul { dst: FieldRegister, src1: FieldRegister, src2: FieldRegister },
    /// FINV fdst, fsrc1: fdst = fsrc1^(-1) (field inverse)
    FInv { dst: FieldRegister, src: FieldRegister },
    /// FNEG fdst, fsrc1: fdst = -fsrc1 (field)
    FNeg { dst: FieldRegister, src: FieldRegister },

    // ============ ZK Primitives ============
    /// HASH fdst, fsrc1, fsrc2: fdst = Poseidon(fsrc1, fsrc2)
    Hash { dst: FieldRegister, src1: FieldRegister, src2: FieldRegister },
    /// HASH4 fdst, fsrc1, fsrc2, fsrc3, fsrc4: fdst = Poseidon(fsrc1..4)
    Hash4 { 
        dst: FieldRegister, 
        src1: FieldRegister, 
        src2: FieldRegister,
        src3: FieldRegister,
        src4: FieldRegister,
    },
    /// ASSERT_EQ src1, src2: assert(src1 == src2)
    AssertEq { src1: Register, src2: Register },
    /// ASSERT_ZERO src: assert(src == 0)
    AssertZero { src: Register },
    /// RANGE_CHECK src, bits: assert(src < 2^bits)
    RangeCheck { src: Register, bits: u8 },

    // ============ I/O ============
    /// READ dst: dst = read_input()
    Read { dst: Register },
    /// WRITE src: write_output(src)
    Write { src: Register },
    /// COMMIT src: commit_public(src)
    Commit { src: Register },

    // ============ System ============
    /// NOP: no operation
    Nop,
    /// HALT: stop execution
    Halt,
    /// INVALID: invalid instruction (trap)
    Invalid,
}

impl Instruction {
    /// Get the opcode for this instruction
    pub fn opcode(&self) -> Opcode {
        match self {
            Instruction::Add { .. } => Opcode::Add,
            Instruction::Sub { .. } => Opcode::Sub,
            Instruction::Mul { .. } => Opcode::Mul,
            Instruction::Div { .. } => Opcode::Div,
            Instruction::SDiv { .. } => Opcode::SDiv,
            Instruction::Mod { .. } => Opcode::Mod,
            Instruction::SMod { .. } => Opcode::SMod,
            Instruction::Neg { .. } => Opcode::Neg,

            Instruction::AddI { .. } => Opcode::AddI,
            Instruction::SubI { .. } => Opcode::SubI,
            Instruction::MulI { .. } => Opcode::MulI,

            Instruction::And { .. } => Opcode::And,
            Instruction::Or { .. } => Opcode::Or,
            Instruction::Xor { .. } => Opcode::Xor,
            Instruction::Not { .. } => Opcode::Not,
            Instruction::Shl { .. } => Opcode::Shl,
            Instruction::Shr { .. } => Opcode::Shr,
            Instruction::Sar { .. } => Opcode::Sar,

            Instruction::Eq { .. } => Opcode::Eq,
            Instruction::Ne { .. } => Opcode::Ne,
            Instruction::Lt { .. } => Opcode::Lt,
            Instruction::Le { .. } => Opcode::Le,
            Instruction::Gt { .. } => Opcode::Gt,
            Instruction::Ge { .. } => Opcode::Ge,
            Instruction::Ltu { .. } => Opcode::Ltu,
            Instruction::Geu { .. } => Opcode::Geu,

            Instruction::Load { .. } => Opcode::Load,
            Instruction::Store { .. } => Opcode::Store,
            Instruction::Load8 { .. } => Opcode::Load8,
            Instruction::Load16 { .. } => Opcode::Load16,
            Instruction::Store8 { .. } => Opcode::Store8,
            Instruction::Store16 { .. } => Opcode::Store16,

            Instruction::Jmp { .. } => Opcode::Jmp,
            Instruction::JmpI { .. } => Opcode::JmpI,
            Instruction::Beq { .. } => Opcode::Beq,
            Instruction::Bne { .. } => Opcode::Bne,
            Instruction::Blt { .. } => Opcode::Blt,
            Instruction::Bge { .. } => Opcode::Bge,
            Instruction::Bltu { .. } => Opcode::Bltu,
            Instruction::Bgeu { .. } => Opcode::Bgeu,

            Instruction::Call { .. } => Opcode::Call,
            Instruction::CallI { .. } => Opcode::CallI,
            Instruction::Ret => Opcode::Ret,

            Instruction::Li { .. } => Opcode::Li,
            Instruction::Lui { .. } => Opcode::Lui,
            Instruction::Mov { .. } => Opcode::Mov,

            Instruction::FAdd { .. } => Opcode::FAdd,
            Instruction::FSub { .. } => Opcode::FSub,
            Instruction::FMul { .. } => Opcode::FMul,
            Instruction::FInv { .. } => Opcode::FInv,
            Instruction::FNeg { .. } => Opcode::FNeg,

            Instruction::Hash { .. } => Opcode::Hash,
            Instruction::Hash4 { .. } => Opcode::Hash4,
            Instruction::AssertEq { .. } => Opcode::AssertEq,
            Instruction::AssertZero { .. } => Opcode::AssertZero,
            Instruction::RangeCheck { .. } => Opcode::RangeCheck,

            Instruction::Read { .. } => Opcode::Read,
            Instruction::Write { .. } => Opcode::Write,
            Instruction::Commit { .. } => Opcode::Commit,

            Instruction::Nop => Opcode::Nop,
            Instruction::Halt => Opcode::Halt,
            Instruction::Invalid => Opcode::Invalid,
        }
    }

    /// Encode instruction to 64-bit value
    pub fn encode(&self) -> u64 {
        let opcode = self.opcode().to_byte() as u64;
        
        match self {
            // R-format: opcode | dst | src1 | src2 | 0
            Instruction::Add { dst, src1, src2 } |
            Instruction::Sub { dst, src1, src2 } |
            Instruction::Mul { dst, src1, src2 } |
            Instruction::Div { dst, src1, src2 } |
            Instruction::SDiv { dst, src1, src2 } |
            Instruction::Mod { dst, src1, src2 } |
            Instruction::SMod { dst, src1, src2 } |
            Instruction::And { dst, src1, src2 } |
            Instruction::Or { dst, src1, src2 } |
            Instruction::Xor { dst, src1, src2 } |
            Instruction::Shl { dst, src1, src2 } |
            Instruction::Shr { dst, src1, src2 } |
            Instruction::Sar { dst, src1, src2 } |
            Instruction::Eq { dst, src1, src2 } |
            Instruction::Ne { dst, src1, src2 } |
            Instruction::Lt { dst, src1, src2 } |
            Instruction::Le { dst, src1, src2 } |
            Instruction::Gt { dst, src1, src2 } |
            Instruction::Ge { dst, src1, src2 } |
            Instruction::Ltu { dst, src1, src2 } |
            Instruction::Geu { dst, src1, src2 } => {
                opcode | ((dst.index() as u64) << 8) | ((src1.index() as u64) << 16) | ((src2.index() as u64) << 24)
            }

            // R2-format: opcode | dst | src1 | 0 | 0
            Instruction::Neg { dst, src1 } |
            Instruction::Not { dst, src1 } |
            Instruction::Mov { dst, src: src1 } => {
                opcode | ((dst.index() as u64) << 8) | ((src1.index() as u64) << 16)
            }

            // I-format: opcode | dst | src1 | imm32
            Instruction::AddI { dst, src1, imm } |
            Instruction::SubI { dst, src1, imm } |
            Instruction::MulI { dst, src1, imm } => {
                opcode | ((dst.index() as u64) << 8) | ((src1.index() as u64) << 16) | ((*imm as u32 as u64) << 32)
            }

            // Memory format: opcode | dst/src | base | offset32
            Instruction::Load { dst, base, offset } => {
                opcode | ((dst.index() as u64) << 8) | ((base.index() as u64) << 16) | ((*offset as u32 as u64) << 32)
            }
            Instruction::Store { src, base, offset } => {
                opcode | ((src.index() as u64) << 8) | ((base.index() as u64) << 16) | ((*offset as u32 as u64) << 32)
            }
            Instruction::Load8 { dst, base, offset } |
            Instruction::Load16 { dst, base, offset } => {
                opcode | ((dst.index() as u64) << 8) | ((base.index() as u64) << 16) | ((*offset as u32 as u64) << 32)
            }
            Instruction::Store8 { src, base, offset } |
            Instruction::Store16 { src, base, offset } => {
                opcode | ((src.index() as u64) << 8) | ((base.index() as u64) << 16) | ((*offset as u32 as u64) << 32)
            }

            // J-format: opcode | 0 | target48
            Instruction::Jmp { target } |
            Instruction::Call { target } => {
                opcode | ((*target as u64) << 32)
            }

            // B-format: opcode | src1 | src2 | target32
            Instruction::Beq { src1, src2, target } |
            Instruction::Bne { src1, src2, target } |
            Instruction::Blt { src1, src2, target } |
            Instruction::Bge { src1, src2, target } |
            Instruction::Bltu { src1, src2, target } |
            Instruction::Bgeu { src1, src2, target } => {
                opcode | ((src1.index() as u64) << 8) | ((src2.index() as u64) << 16) | ((*target as u64) << 32)
            }

            // S-format: opcode | src | 0 | 0
            Instruction::JmpI { src } |
            Instruction::CallI { src } |
            Instruction::Write { src } |
            Instruction::Commit { src } |
            Instruction::AssertZero { src } => {
                opcode | ((src.index() as u64) << 8)
            }

            // U-format: opcode | dst | imm32
            Instruction::Li { dst, imm } |
            Instruction::Lui { dst, imm } => {
                opcode | ((dst.index() as u64) << 8) | ((*imm as u64) << 32)
            }

            Instruction::Read { dst } => {
                opcode | ((dst.index() as u64) << 8)
            }

            // A-format: opcode | src1 | src2
            Instruction::AssertEq { src1, src2 } => {
                opcode | ((src1.index() as u64) << 8) | ((src2.index() as u64) << 16)
            }

            Instruction::RangeCheck { src, bits } => {
                opcode | ((src.index() as u64) << 8) | ((*bits as u64) << 32)
            }

            // Field operations (use field register indices)
            Instruction::FAdd { dst, src1, src2 } |
            Instruction::FSub { dst, src1, src2 } |
            Instruction::FMul { dst, src1, src2 } |
            Instruction::Hash { dst, src1, src2 } => {
                opcode | ((dst.index() as u64) << 8) | ((src1.index() as u64) << 16) | ((src2.index() as u64) << 24)
            }

            Instruction::FInv { dst, src } |
            Instruction::FNeg { dst, src } => {
                opcode | ((dst.index() as u64) << 8) | ((src.index() as u64) << 16)
            }

            Instruction::Hash4 { dst, src1, src2, src3, src4 } => {
                opcode | ((dst.index() as u64) << 8) | ((src1.index() as u64) << 16) | 
                ((src2.index() as u64) << 24) | ((src3.index() as u64) << 32) | ((src4.index() as u64) << 40)
            }

            // N-format: opcode only
            Instruction::Ret |
            Instruction::Nop |
            Instruction::Halt |
            Instruction::Invalid => opcode,
        }
    }

    /// Decode instruction from 64-bit value
    pub fn decode(value: u64) -> Result<Self, ZkIrError> {
        let opcode_byte = (value & 0xFF) as u8;
        let opcode = Opcode::from_byte(opcode_byte)
            .ok_or(ZkIrError::InvalidOpcode(opcode_byte))?;

        let dst_byte = ((value >> 8) & 0xFF) as u8;
        let src1_byte = ((value >> 16) & 0xFF) as u8;
        let src2_byte = ((value >> 24) & 0xFF) as u8;
        let imm32 = (value >> 32) as u32;

        let dst = || Register::from_index(dst_byte).ok_or(ZkIrError::InvalidRegister(dst_byte));
        let src1 = || Register::from_index(src1_byte).ok_or(ZkIrError::InvalidRegister(src1_byte));
        let src2 = || Register::from_index(src2_byte).ok_or(ZkIrError::InvalidRegister(src2_byte));
        let fdst = || FieldRegister::from_index(dst_byte).ok_or(ZkIrError::InvalidRegister(dst_byte));
        let fsrc1 = || FieldRegister::from_index(src1_byte).ok_or(ZkIrError::InvalidRegister(src1_byte));
        let fsrc2 = || FieldRegister::from_index(src2_byte).ok_or(ZkIrError::InvalidRegister(src2_byte));

        let instr = match opcode {
            Opcode::Add => Instruction::Add { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Sub => Instruction::Sub { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Mul => Instruction::Mul { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Div => Instruction::Div { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::SDiv => Instruction::SDiv { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Mod => Instruction::Mod { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::SMod => Instruction::SMod { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Neg => Instruction::Neg { dst: dst()?, src1: src1()? },

            Opcode::AddI => Instruction::AddI { dst: dst()?, src1: src1()?, imm: imm32 as i32 },
            Opcode::SubI => Instruction::SubI { dst: dst()?, src1: src1()?, imm: imm32 as i32 },
            Opcode::MulI => Instruction::MulI { dst: dst()?, src1: src1()?, imm: imm32 as i32 },

            Opcode::And => Instruction::And { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Or => Instruction::Or { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Xor => Instruction::Xor { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Not => Instruction::Not { dst: dst()?, src1: src1()? },
            Opcode::Shl => Instruction::Shl { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Shr => Instruction::Shr { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Sar => Instruction::Sar { dst: dst()?, src1: src1()?, src2: src2()? },

            Opcode::Eq => Instruction::Eq { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Ne => Instruction::Ne { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Lt => Instruction::Lt { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Le => Instruction::Le { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Gt => Instruction::Gt { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Ge => Instruction::Ge { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Ltu => Instruction::Ltu { dst: dst()?, src1: src1()?, src2: src2()? },
            Opcode::Geu => Instruction::Geu { dst: dst()?, src1: src1()?, src2: src2()? },

            Opcode::Load => Instruction::Load { dst: dst()?, base: src1()?, offset: imm32 as i32 },
            Opcode::Store => Instruction::Store { src: dst()?, base: src1()?, offset: imm32 as i32 },
            Opcode::Load8 => Instruction::Load8 { dst: dst()?, base: src1()?, offset: imm32 as i32 },
            Opcode::Load16 => Instruction::Load16 { dst: dst()?, base: src1()?, offset: imm32 as i32 },
            Opcode::Store8 => Instruction::Store8 { src: dst()?, base: src1()?, offset: imm32 as i32 },
            Opcode::Store16 => Instruction::Store16 { src: dst()?, base: src1()?, offset: imm32 as i32 },

            Opcode::Jmp => Instruction::Jmp { target: imm32 },
            Opcode::JmpI => Instruction::JmpI { src: dst()? },
            Opcode::Beq => Instruction::Beq { src1: dst()?, src2: src1()?, target: imm32 },
            Opcode::Bne => Instruction::Bne { src1: dst()?, src2: src1()?, target: imm32 },
            Opcode::Blt => Instruction::Blt { src1: dst()?, src2: src1()?, target: imm32 },
            Opcode::Bge => Instruction::Bge { src1: dst()?, src2: src1()?, target: imm32 },
            Opcode::Bltu => Instruction::Bltu { src1: dst()?, src2: src1()?, target: imm32 },
            Opcode::Bgeu => Instruction::Bgeu { src1: dst()?, src2: src1()?, target: imm32 },

            Opcode::Call => Instruction::Call { target: imm32 },
            Opcode::CallI => Instruction::CallI { src: dst()? },
            Opcode::Ret => Instruction::Ret,

            Opcode::Li => Instruction::Li { dst: dst()?, imm: imm32 },
            Opcode::Lui => Instruction::Lui { dst: dst()?, imm: imm32 },
            Opcode::Mov => Instruction::Mov { dst: dst()?, src: src1()? },

            Opcode::FAdd => Instruction::FAdd { dst: fdst()?, src1: fsrc1()?, src2: fsrc2()? },
            Opcode::FSub => Instruction::FSub { dst: fdst()?, src1: fsrc1()?, src2: fsrc2()? },
            Opcode::FMul => Instruction::FMul { dst: fdst()?, src1: fsrc1()?, src2: fsrc2()? },
            Opcode::FInv => Instruction::FInv { dst: fdst()?, src: fsrc1()? },
            Opcode::FNeg => Instruction::FNeg { dst: fdst()?, src: fsrc1()? },

            Opcode::Hash => Instruction::Hash { dst: fdst()?, src1: fsrc1()?, src2: fsrc2()? },
            Opcode::Hash4 => {
                let src3_byte = (imm32 & 0xFF) as u8;
                let src4_byte = ((imm32 >> 8) & 0xFF) as u8;
                Instruction::Hash4 {
                    dst: fdst()?,
                    src1: fsrc1()?,
                    src2: fsrc2()?,
                    src3: FieldRegister::from_index(src3_byte).ok_or(ZkIrError::InvalidRegister(src3_byte))?,
                    src4: FieldRegister::from_index(src4_byte).ok_or(ZkIrError::InvalidRegister(src4_byte))?,
                }
            }
            Opcode::AssertEq => Instruction::AssertEq { src1: dst()?, src2: src1()? },
            Opcode::AssertZero => Instruction::AssertZero { src: dst()? },
            Opcode::RangeCheck => Instruction::RangeCheck { src: dst()?, bits: imm32 as u8 },

            Opcode::Read => Instruction::Read { dst: dst()? },
            Opcode::Write => Instruction::Write { src: dst()? },
            Opcode::Commit => Instruction::Commit { src: dst()? },

            Opcode::Nop => Instruction::Nop,
            Opcode::Halt => Instruction::Halt,
            Opcode::Invalid => Instruction::Invalid,
        };

        Ok(instr)
    }

    /// Create a new halt instruction
    pub fn new_halt() -> Self {
        Instruction::Halt
    }

    /// Create a new nop instruction
    pub fn new_nop() -> Self {
        Instruction::Nop
    }

    /// Create a new R-format instruction (3 registers: dst, src1, src2)
    pub fn new_r(opcode: Opcode, dst: Register, src1: Register, src2: Register) -> Self {
        match opcode {
            Opcode::Add => Instruction::Add { dst, src1, src2 },
            Opcode::Sub => Instruction::Sub { dst, src1, src2 },
            Opcode::Mul => Instruction::Mul { dst, src1, src2 },
            Opcode::Div => Instruction::Div { dst, src1, src2 },
            Opcode::SDiv => Instruction::SDiv { dst, src1, src2 },
            Opcode::Mod => Instruction::Mod { dst, src1, src2 },
            Opcode::SMod => Instruction::SMod { dst, src1, src2 },
            Opcode::And => Instruction::And { dst, src1, src2 },
            Opcode::Or => Instruction::Or { dst, src1, src2 },
            Opcode::Xor => Instruction::Xor { dst, src1, src2 },
            Opcode::Shl => Instruction::Shl { dst, src1, src2 },
            Opcode::Shr => Instruction::Shr { dst, src1, src2 },
            Opcode::Sar => Instruction::Sar { dst, src1, src2 },
            Opcode::Eq => Instruction::Eq { dst, src1, src2 },
            Opcode::Ne => Instruction::Ne { dst, src1, src2 },
            Opcode::Lt => Instruction::Lt { dst, src1, src2 },
            Opcode::Le => Instruction::Le { dst, src1, src2 },
            Opcode::Gt => Instruction::Gt { dst, src1, src2 },
            Opcode::Ge => Instruction::Ge { dst, src1, src2 },
            Opcode::Ltu => Instruction::Ltu { dst, src1, src2 },
            Opcode::Geu => Instruction::Geu { dst, src1, src2 },
            _ => panic!("Opcode {:?} is not R-format", opcode),
        }
    }

    /// Create a new R2-format instruction (2 registers: dst, src)
    pub fn new_r2(opcode: Opcode, dst: Register, src: Register) -> Self {
        match opcode {
            Opcode::Neg => Instruction::Neg { dst, src1: src },
            Opcode::Not => Instruction::Not { dst, src1: src },
            Opcode::Mov => Instruction::Mov { dst, src },
            _ => panic!("Opcode {:?} is not R2-format", opcode),
        }
    }

    /// Create a new I-format instruction (dst, src, immediate)
    pub fn new_i(opcode: Opcode, dst: Register, src: Register, imm: i32) -> Self {
        match opcode {
            Opcode::AddI => Instruction::AddI { dst, src1: src, imm },
            Opcode::SubI => Instruction::SubI { dst, src1: src, imm },
            Opcode::MulI => Instruction::MulI { dst, src1: src, imm },
            Opcode::Load => Instruction::Load { dst, base: src, offset: imm },
            Opcode::Load8 => Instruction::Load8 { dst, base: src, offset: imm },
            Opcode::Load16 => Instruction::Load16 { dst, base: src, offset: imm },
            _ => panic!("Opcode {:?} is not I-format", opcode),
        }
    }

    /// Create a new branch instruction
    pub fn new_branch(opcode: Opcode, src1: Register, src2: Register, target: u32) -> Self {
        match opcode {
            Opcode::Beq => Instruction::Beq { src1, src2, target },
            Opcode::Bne => Instruction::Bne { src1, src2, target },
            Opcode::Blt => Instruction::Blt { src1, src2, target },
            Opcode::Bge => Instruction::Bge { src1, src2, target },
            Opcode::Bltu => Instruction::Bltu { src1, src2, target },
            Opcode::Bgeu => Instruction::Bgeu { src1, src2, target },
            _ => panic!("Opcode {:?} is not a branch", opcode),
        }
    }

    /// Create a load immediate instruction
    pub fn new_li(dst: Register, imm: u32) -> Self {
        Instruction::Li { dst, imm }
    }

    /// Create a jump instruction
    pub fn new_jmp(target: u32) -> Self {
        Instruction::Jmp { target }
    }

    /// Create a call instruction
    pub fn new_call(target: u32) -> Self {
        Instruction::Call { target }
    }

    /// Create a return instruction
    pub fn new_ret() -> Self {
        Instruction::Ret
    }

    /// Check if this instruction is a terminator (changes control flow)
    pub fn is_terminator(&self) -> bool {
        matches!(
            self,
            Instruction::Jmp { .. } |
            Instruction::JmpI { .. } |
            Instruction::Beq { .. } |
            Instruction::Bne { .. } |
            Instruction::Blt { .. } |
            Instruction::Bge { .. } |
            Instruction::Bltu { .. } |
            Instruction::Bgeu { .. } |
            Instruction::Call { .. } |
            Instruction::CallI { .. } |
            Instruction::Ret |
            Instruction::Halt
        )
    }

    /// Check if this instruction modifies memory
    pub fn modifies_memory(&self) -> bool {
        matches!(
            self,
            Instruction::Store { .. } |
            Instruction::Store8 { .. } |
            Instruction::Store16 { .. }
        )
    }

    /// Check if this instruction reads from memory
    pub fn reads_memory(&self) -> bool {
        matches!(
            self,
            Instruction::Load { .. } |
            Instruction::Load8 { .. } |
            Instruction::Load16 { .. }
        )
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Arithmetic
            Instruction::Add { dst, src1, src2 } => write!(f, "ADD {}, {}, {}", dst, src1, src2),
            Instruction::Sub { dst, src1, src2 } => write!(f, "SUB {}, {}, {}", dst, src1, src2),
            Instruction::Mul { dst, src1, src2 } => write!(f, "MUL {}, {}, {}", dst, src1, src2),
            Instruction::Div { dst, src1, src2 } => write!(f, "DIV {}, {}, {}", dst, src1, src2),
            Instruction::SDiv { dst, src1, src2 } => write!(f, "SDIV {}, {}, {}", dst, src1, src2),
            Instruction::Mod { dst, src1, src2 } => write!(f, "MOD {}, {}, {}", dst, src1, src2),
            Instruction::SMod { dst, src1, src2 } => write!(f, "SMOD {}, {}, {}", dst, src1, src2),
            Instruction::Neg { dst, src1 } => write!(f, "NEG {}, {}", dst, src1),

            // Immediate Arithmetic
            Instruction::AddI { dst, src1, imm } => write!(f, "ADDI {}, {}, {}", dst, src1, imm),
            Instruction::SubI { dst, src1, imm } => write!(f, "SUBI {}, {}, {}", dst, src1, imm),
            Instruction::MulI { dst, src1, imm } => write!(f, "MULI {}, {}, {}", dst, src1, imm),

            // Logic
            Instruction::And { dst, src1, src2 } => write!(f, "AND {}, {}, {}", dst, src1, src2),
            Instruction::Or { dst, src1, src2 } => write!(f, "OR {}, {}, {}", dst, src1, src2),
            Instruction::Xor { dst, src1, src2 } => write!(f, "XOR {}, {}, {}", dst, src1, src2),
            Instruction::Not { dst, src1 } => write!(f, "NOT {}, {}", dst, src1),
            Instruction::Shl { dst, src1, src2 } => write!(f, "SHL {}, {}, {}", dst, src1, src2),
            Instruction::Shr { dst, src1, src2 } => write!(f, "SHR {}, {}, {}", dst, src1, src2),
            Instruction::Sar { dst, src1, src2 } => write!(f, "SAR {}, {}, {}", dst, src1, src2),

            // Comparison
            Instruction::Eq { dst, src1, src2 } => write!(f, "EQ {}, {}, {}", dst, src1, src2),
            Instruction::Ne { dst, src1, src2 } => write!(f, "NE {}, {}, {}", dst, src1, src2),
            Instruction::Lt { dst, src1, src2 } => write!(f, "LT {}, {}, {}", dst, src1, src2),
            Instruction::Le { dst, src1, src2 } => write!(f, "LE {}, {}, {}", dst, src1, src2),
            Instruction::Gt { dst, src1, src2 } => write!(f, "GT {}, {}, {}", dst, src1, src2),
            Instruction::Ge { dst, src1, src2 } => write!(f, "GE {}, {}, {}", dst, src1, src2),
            Instruction::Ltu { dst, src1, src2 } => write!(f, "LTU {}, {}, {}", dst, src1, src2),
            Instruction::Geu { dst, src1, src2 } => write!(f, "GEU {}, {}, {}", dst, src1, src2),

            // Memory
            Instruction::Load { dst, base, offset } => write!(f, "LOAD {}, {}({})", dst, offset, base),
            Instruction::Store { src, base, offset } => write!(f, "STORE {}({}), {}", offset, base, src),
            Instruction::Load8 { dst, base, offset } => write!(f, "LOAD8 {}, {}({})", dst, offset, base),
            Instruction::Load16 { dst, base, offset } => write!(f, "LOAD16 {}, {}({})", dst, offset, base),
            Instruction::Store8 { src, base, offset } => write!(f, "STORE8 {}({}), {}", offset, base, src),
            Instruction::Store16 { src, base, offset } => write!(f, "STORE16 {}({}), {}", offset, base, src),

            // Control Flow
            Instruction::Jmp { target } => write!(f, "JMP {}", target),
            Instruction::JmpI { src } => write!(f, "JMPI {}", src),
            Instruction::Beq { src1, src2, target } => write!(f, "BEQ {}, {}, {}", src1, src2, target),
            Instruction::Bne { src1, src2, target } => write!(f, "BNE {}, {}, {}", src1, src2, target),
            Instruction::Blt { src1, src2, target } => write!(f, "BLT {}, {}, {}", src1, src2, target),
            Instruction::Bge { src1, src2, target } => write!(f, "BGE {}, {}, {}", src1, src2, target),
            Instruction::Bltu { src1, src2, target } => write!(f, "BLTU {}, {}, {}", src1, src2, target),
            Instruction::Bgeu { src1, src2, target } => write!(f, "BGEU {}, {}, {}", src1, src2, target),

            // Function Calls
            Instruction::Call { target } => write!(f, "CALL {}", target),
            Instruction::CallI { src } => write!(f, "CALLI {}", src),
            Instruction::Ret => write!(f, "RET"),

            // Constants
            Instruction::Li { dst, imm } => write!(f, "LI {}, {}", dst, imm),
            Instruction::Lui { dst, imm } => write!(f, "LUI {}, {}", dst, imm),
            Instruction::Mov { dst, src } => write!(f, "MOV {}, {}", dst, src),

            // Field Operations
            Instruction::FAdd { dst, src1, src2 } => write!(f, "FADD {}, {}, {}", dst, src1, src2),
            Instruction::FSub { dst, src1, src2 } => write!(f, "FSUB {}, {}, {}", dst, src1, src2),
            Instruction::FMul { dst, src1, src2 } => write!(f, "FMUL {}, {}, {}", dst, src1, src2),
            Instruction::FInv { dst, src } => write!(f, "FINV {}, {}", dst, src),
            Instruction::FNeg { dst, src } => write!(f, "FNEG {}, {}", dst, src),

            // ZK Primitives
            Instruction::Hash { dst, src1, src2 } => write!(f, "HASH {}, {}, {}", dst, src1, src2),
            Instruction::Hash4 { dst, src1, src2, src3, src4 } => {
                write!(f, "HASH4 {}, {}, {}, {}, {}", dst, src1, src2, src3, src4)
            }
            Instruction::AssertEq { src1, src2 } => write!(f, "ASSERT_EQ {}, {}", src1, src2),
            Instruction::AssertZero { src } => write!(f, "ASSERT_ZERO {}", src),
            Instruction::RangeCheck { src, bits } => write!(f, "RANGE_CHECK {}, {}", src, bits),

            // I/O
            Instruction::Read { dst } => write!(f, "READ {}", dst),
            Instruction::Write { src } => write!(f, "WRITE {}", src),
            Instruction::Commit { src } => write!(f, "COMMIT {}", src),

            // System
            Instruction::Nop => write!(f, "NOP"),
            Instruction::Halt => write!(f, "HALT"),
            Instruction::Invalid => write!(f, "INVALID"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let instructions = vec![
            Instruction::Add { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Li { dst: Register::R5, imm: 42 },
            Instruction::Load { dst: Register::R1, base: Register::R2, offset: 100 },
            Instruction::Beq { src1: Register::R1, src2: Register::R2, target: 1000 },
            Instruction::Halt,
            Instruction::Nop,
        ];

        for instr in instructions {
            let encoded = instr.encode();
            let decoded = Instruction::decode(encoded).unwrap();
            assert_eq!(instr, decoded);
        }
    }

    #[test]
    fn test_opcode_consistency() {
        let instr = Instruction::Add { dst: Register::R1, src1: Register::R2, src2: Register::R3 };
        assert_eq!(instr.opcode(), Opcode::Add);
    }

    #[test]
    fn test_all_instructions_encode_decode() {
        use crate::register::FieldRegister;

        // Test all instruction variants
        let instructions = vec![
            // Arithmetic
            Instruction::Add { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Sub { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Mul { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Div { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::SDiv { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Mod { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::SMod { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Neg { dst: Register::R1, src1: Register::R2 },

            // Immediate
            Instruction::AddI { dst: Register::R1, src1: Register::R2, imm: 100 },
            Instruction::SubI { dst: Register::R1, src1: Register::R2, imm: -50 },
            Instruction::MulI { dst: Register::R1, src1: Register::R2, imm: 10 },

            // Logic
            Instruction::And { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Or { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Xor { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Not { dst: Register::R1, src1: Register::R2 },
            Instruction::Shl { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Shr { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Sar { dst: Register::R1, src1: Register::R2, src2: Register::R3 },

            // Comparison
            Instruction::Eq { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Ne { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Lt { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Le { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Gt { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Ge { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Ltu { dst: Register::R1, src1: Register::R2, src2: Register::R3 },
            Instruction::Geu { dst: Register::R1, src1: Register::R2, src2: Register::R3 },

            // Memory
            Instruction::Load { dst: Register::R1, base: Register::R2, offset: 100 },
            Instruction::Store { src: Register::R1, base: Register::R2, offset: 100 },
            Instruction::Load8 { dst: Register::R1, base: Register::R2, offset: 50 },
            Instruction::Load16 { dst: Register::R1, base: Register::R2, offset: 50 },
            Instruction::Store8 { src: Register::R1, base: Register::R2, offset: 50 },
            Instruction::Store16 { src: Register::R1, base: Register::R2, offset: 50 },

            // Control flow
            Instruction::Jmp { target: 1000 },
            Instruction::JmpI { src: Register::R1 },
            Instruction::Beq { src1: Register::R1, src2: Register::R2, target: 500 },
            Instruction::Bne { src1: Register::R1, src2: Register::R2, target: 500 },
            Instruction::Blt { src1: Register::R1, src2: Register::R2, target: 500 },
            Instruction::Bge { src1: Register::R1, src2: Register::R2, target: 500 },
            Instruction::Bltu { src1: Register::R1, src2: Register::R2, target: 500 },
            Instruction::Bgeu { src1: Register::R1, src2: Register::R2, target: 500 },

            // Function calls
            Instruction::Call { target: 2000 },
            Instruction::CallI { src: Register::R1 },
            Instruction::Ret,

            // Constants
            Instruction::Li { dst: Register::R1, imm: 0xDEADBEEF },
            Instruction::Lui { dst: Register::R1, imm: 0x12345678 },
            Instruction::Mov { dst: Register::R1, src: Register::R2 },

            // Field ops
            Instruction::FAdd { dst: FieldRegister::F0, src1: FieldRegister::F1, src2: FieldRegister::F2 },
            Instruction::FSub { dst: FieldRegister::F0, src1: FieldRegister::F1, src2: FieldRegister::F2 },
            Instruction::FMul { dst: FieldRegister::F0, src1: FieldRegister::F1, src2: FieldRegister::F2 },
            Instruction::FInv { dst: FieldRegister::F0, src: FieldRegister::F1 },
            Instruction::FNeg { dst: FieldRegister::F0, src: FieldRegister::F1 },

            // ZK primitives
            Instruction::Hash { dst: FieldRegister::F0, src1: FieldRegister::F1, src2: FieldRegister::F2 },
            Instruction::Hash4 {
                dst: FieldRegister::F0,
                src1: FieldRegister::F1,
                src2: FieldRegister::F2,
                src3: FieldRegister::F3,
                src4: FieldRegister::F4,
            },
            Instruction::AssertEq { src1: Register::R1, src2: Register::R2 },
            Instruction::AssertZero { src: Register::R1 },
            Instruction::RangeCheck { src: Register::R1, bits: 32 },

            // I/O
            Instruction::Read { dst: Register::R1 },
            Instruction::Write { src: Register::R1 },
            Instruction::Commit { src: Register::R1 },

            // System
            Instruction::Nop,
            Instruction::Halt,
            Instruction::Invalid,
        ];

        for instr in instructions {
            let encoded = instr.encode();
            let decoded = Instruction::decode(encoded).expect(&format!("Failed to decode {:?}", instr));
            assert_eq!(instr, decoded, "Roundtrip failed for {:?}", instr);
        }
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_register() -> impl Strategy<Value = Register> {
        (0u8..32).prop_map(|i| Register::from_index(i).unwrap())
    }

    fn arb_field_register() -> impl Strategy<Value = crate::register::FieldRegister> {
        (0u8..16).prop_map(|i| crate::register::FieldRegister::from_index(i).unwrap())
    }

    proptest! {
        #[test]
        fn test_add_encode_decode(
            dst in arb_register(),
            src1 in arb_register(),
            src2 in arb_register()
        ) {
            let instr = Instruction::Add { dst, src1, src2 };
            let encoded = instr.encode();
            let decoded = Instruction::decode(encoded).unwrap();
            prop_assert_eq!(instr, decoded);
        }

        #[test]
        fn test_li_encode_decode(dst in arb_register(), imm: u32) {
            let instr = Instruction::Li { dst, imm };
            let encoded = instr.encode();
            let decoded = Instruction::decode(encoded).unwrap();
            prop_assert_eq!(instr, decoded);
        }

        #[test]
        fn test_load_encode_decode(
            dst in arb_register(),
            base in arb_register(),
            offset: i32
        ) {
            let instr = Instruction::Load { dst, base, offset };
            let encoded = instr.encode();
            let decoded = Instruction::decode(encoded).unwrap();
            prop_assert_eq!(instr, decoded);
        }

        #[test]
        fn test_branch_encode_decode(
            src1 in arb_register(),
            src2 in arb_register(),
            target: u32
        ) {
            let instr = Instruction::Beq { src1, src2, target };
            let encoded = instr.encode();
            let decoded = Instruction::decode(encoded).unwrap();
            prop_assert_eq!(instr, decoded);
        }

        #[test]
        fn test_field_ops_encode_decode(
            dst in arb_field_register(),
            src1 in arb_field_register(),
            src2 in arb_field_register()
        ) {
            let instr = Instruction::FAdd { dst, src1, src2 };
            let encoded = instr.encode();
            let decoded = Instruction::decode(encoded).unwrap();
            prop_assert_eq!(instr, decoded);
        }
    }
}
