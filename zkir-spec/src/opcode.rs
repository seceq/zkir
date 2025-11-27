//! Opcode definitions for ZK IR instructions.

use serde::{Deserialize, Serialize};

/// All opcodes supported by ZK IR.
///
/// Opcodes are organized into categories:
/// - 0x01-0x0F: Arithmetic
/// - 0x10-0x1F: Immediate arithmetic
/// - 0x20-0x2F: Logic/bitwise
/// - 0x30-0x3F: Comparison
/// - 0x40-0x4F: Memory
/// - 0x50-0x5F: Control flow (jumps/branches)
/// - 0x60-0x6F: Function calls
/// - 0x70-0x7F: Constants/moves
/// - 0x80-0x8F: Field operations
/// - 0x90-0x9F: ZK primitives
/// - 0xA0-0xAF: I/O
/// - 0xF0-0xFF: System
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Opcode {
    // ============ Arithmetic (0x01-0x0F) ============
    /// Addition: dst = src1 + src2
    Add = 0x01,
    /// Subtraction: dst = src1 - src2
    Sub = 0x02,
    /// Multiplication: dst = src1 * src2
    Mul = 0x03,
    /// Unsigned division: dst = src1 / src2
    Div = 0x04,
    /// Signed division: dst = src1 / src2
    SDiv = 0x05,
    /// Unsigned modulo: dst = src1 % src2
    Mod = 0x06,
    /// Signed modulo: dst = src1 % src2
    SMod = 0x07,
    /// Negation: dst = -src1
    Neg = 0x08,

    // ============ Immediate Arithmetic (0x10-0x1F) ============
    /// Add immediate: dst = src1 + imm
    AddI = 0x10,
    /// Subtract immediate: dst = src1 - imm
    SubI = 0x11,
    /// Multiply immediate: dst = src1 * imm
    MulI = 0x12,

    // ============ Logic/Bitwise (0x20-0x2F) ============
    /// Bitwise AND: dst = src1 & src2
    And = 0x20,
    /// Bitwise OR: dst = src1 | src2
    Or = 0x21,
    /// Bitwise XOR: dst = src1 ^ src2
    Xor = 0x22,
    /// Bitwise NOT: dst = ~src1
    Not = 0x23,
    /// Shift left: dst = src1 << src2
    Shl = 0x24,
    /// Logical shift right: dst = src1 >> src2
    Shr = 0x25,
    /// Arithmetic shift right: dst = src1 >> src2 (sign-extended)
    Sar = 0x26,

    // ============ Comparison (0x30-0x3F) ============
    /// Equal: dst = (src1 == src2) ? 1 : 0
    Eq = 0x30,
    /// Not equal: dst = (src1 != src2) ? 1 : 0
    Ne = 0x31,
    /// Less than (signed): dst = (src1 < src2) ? 1 : 0
    Lt = 0x32,
    /// Less than or equal (signed): dst = (src1 <= src2) ? 1 : 0
    Le = 0x33,
    /// Greater than (signed): dst = (src1 > src2) ? 1 : 0
    Gt = 0x34,
    /// Greater than or equal (signed): dst = (src1 >= src2) ? 1 : 0
    Ge = 0x35,
    /// Less than (unsigned): dst = (src1 < src2) ? 1 : 0
    Ltu = 0x36,
    /// Greater than or equal (unsigned): dst = (src1 >= src2) ? 1 : 0
    Geu = 0x37,

    // ============ Memory (0x40-0x4F) ============
    /// Load word: dst = memory[base + offset]
    Load = 0x40,
    /// Store word: memory[base + offset] = src
    Store = 0x41,
    /// Load byte (zero-extended): dst = memory[base + offset] & 0xFF
    Load8 = 0x42,
    /// Load halfword (zero-extended): dst = memory[base + offset] & 0xFFFF
    Load16 = 0x43,
    /// Store byte: memory[base + offset] = src & 0xFF
    Store8 = 0x44,
    /// Store halfword: memory[base + offset] = src & 0xFFFF
    Store16 = 0x45,

    // ============ Control Flow - Jumps (0x50-0x5F) ============
    /// Unconditional jump: pc = target
    Jmp = 0x50,
    /// Indirect jump: pc = src1
    JmpI = 0x51,
    /// Branch if equal: if (src1 == src2) pc = target
    Beq = 0x52,
    /// Branch if not equal: if (src1 != src2) pc = target
    Bne = 0x53,
    /// Branch if less than (signed): if (src1 < src2) pc = target
    Blt = 0x54,
    /// Branch if greater than or equal (signed): if (src1 >= src2) pc = target
    Bge = 0x55,
    /// Branch if less than (unsigned): if (src1 < src2) pc = target
    Bltu = 0x56,
    /// Branch if greater than or equal (unsigned): if (src1 >= src2) pc = target
    Bgeu = 0x57,

    // ============ Function Calls (0x60-0x6F) ============
    /// Call function: push(pc+1), pc = target
    Call = 0x60,
    /// Indirect call: push(pc+1), pc = src1
    CallI = 0x61,
    /// Return: pc = pop()
    Ret = 0x62,

    // ============ Constants/Moves (0x70-0x7F) ============
    /// Load immediate: dst = imm32
    Li = 0x70,
    /// Load upper immediate: dst = imm32 << 32
    Lui = 0x71,
    /// Move: dst = src1
    Mov = 0x72,

    // ============ Field Operations (0x80-0x8F) ============
    /// Field addition: fdst = fsrc1 + fsrc2 (mod p)
    FAdd = 0x80,
    /// Field subtraction: fdst = fsrc1 - fsrc2 (mod p)
    FSub = 0x81,
    /// Field multiplication: fdst = fsrc1 * fsrc2 (mod p)
    FMul = 0x82,
    /// Field inverse: fdst = fsrc1^(-1) (mod p)
    FInv = 0x83,
    /// Field negation: fdst = -fsrc1 (mod p)
    FNeg = 0x84,

    // ============ ZK Primitives (0x90-0x9F) ============
    /// Poseidon hash (2 inputs): fdst = Poseidon(fsrc1, fsrc2)
    Hash = 0x90,
    /// Poseidon hash (4 inputs): fdst = Poseidon(fsrc1, fsrc2, fsrc3, fsrc4)
    Hash4 = 0x91,
    /// Assert equality: assert(src1 == src2)
    AssertEq = 0x92,
    /// Assert zero: assert(src1 == 0)
    AssertZero = 0x93,
    /// Range check: assert(src1 < 2^imm)
    RangeCheck = 0x94,

    // ============ I/O (0xA0-0xAF) ============
    /// Read input: dst = read_input()
    Read = 0xA0,
    /// Write output: write_output(src1)
    Write = 0xA1,
    /// Commit public value: commit(src1)
    Commit = 0xA2,

    // ============ System (0xF0-0xFF) ============
    /// No operation
    Nop = 0xF0,
    /// Halt execution
    Halt = 0xF1,
    /// Invalid instruction (causes trap)
    Invalid = 0xFF,
}

impl Opcode {
    /// Create opcode from byte value
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Opcode::Add),
            0x02 => Some(Opcode::Sub),
            0x03 => Some(Opcode::Mul),
            0x04 => Some(Opcode::Div),
            0x05 => Some(Opcode::SDiv),
            0x06 => Some(Opcode::Mod),
            0x07 => Some(Opcode::SMod),
            0x08 => Some(Opcode::Neg),

            0x10 => Some(Opcode::AddI),
            0x11 => Some(Opcode::SubI),
            0x12 => Some(Opcode::MulI),

            0x20 => Some(Opcode::And),
            0x21 => Some(Opcode::Or),
            0x22 => Some(Opcode::Xor),
            0x23 => Some(Opcode::Not),
            0x24 => Some(Opcode::Shl),
            0x25 => Some(Opcode::Shr),
            0x26 => Some(Opcode::Sar),

            0x30 => Some(Opcode::Eq),
            0x31 => Some(Opcode::Ne),
            0x32 => Some(Opcode::Lt),
            0x33 => Some(Opcode::Le),
            0x34 => Some(Opcode::Gt),
            0x35 => Some(Opcode::Ge),
            0x36 => Some(Opcode::Ltu),
            0x37 => Some(Opcode::Geu),

            0x40 => Some(Opcode::Load),
            0x41 => Some(Opcode::Store),
            0x42 => Some(Opcode::Load8),
            0x43 => Some(Opcode::Load16),
            0x44 => Some(Opcode::Store8),
            0x45 => Some(Opcode::Store16),

            0x50 => Some(Opcode::Jmp),
            0x51 => Some(Opcode::JmpI),
            0x52 => Some(Opcode::Beq),
            0x53 => Some(Opcode::Bne),
            0x54 => Some(Opcode::Blt),
            0x55 => Some(Opcode::Bge),
            0x56 => Some(Opcode::Bltu),
            0x57 => Some(Opcode::Bgeu),

            0x60 => Some(Opcode::Call),
            0x61 => Some(Opcode::CallI),
            0x62 => Some(Opcode::Ret),

            0x70 => Some(Opcode::Li),
            0x71 => Some(Opcode::Lui),
            0x72 => Some(Opcode::Mov),

            0x80 => Some(Opcode::FAdd),
            0x81 => Some(Opcode::FSub),
            0x82 => Some(Opcode::FMul),
            0x83 => Some(Opcode::FInv),
            0x84 => Some(Opcode::FNeg),

            0x90 => Some(Opcode::Hash),
            0x91 => Some(Opcode::Hash4),
            0x92 => Some(Opcode::AssertEq),
            0x93 => Some(Opcode::AssertZero),
            0x94 => Some(Opcode::RangeCheck),

            0xA0 => Some(Opcode::Read),
            0xA1 => Some(Opcode::Write),
            0xA2 => Some(Opcode::Commit),

            0xF0 => Some(Opcode::Nop),
            0xF1 => Some(Opcode::Halt),
            0xFF => Some(Opcode::Invalid),

            _ => None,
        }
    }

    /// Convert opcode to byte value
    pub fn to_byte(self) -> u8 {
        self as u8
    }

    /// Get the mnemonic string for this opcode
    pub fn mnemonic(self) -> &'static str {
        match self {
            Opcode::Add => "ADD",
            Opcode::Sub => "SUB",
            Opcode::Mul => "MUL",
            Opcode::Div => "DIV",
            Opcode::SDiv => "SDIV",
            Opcode::Mod => "MOD",
            Opcode::SMod => "SMOD",
            Opcode::Neg => "NEG",

            Opcode::AddI => "ADDI",
            Opcode::SubI => "SUBI",
            Opcode::MulI => "MULI",

            Opcode::And => "AND",
            Opcode::Or => "OR",
            Opcode::Xor => "XOR",
            Opcode::Not => "NOT",
            Opcode::Shl => "SHL",
            Opcode::Shr => "SHR",
            Opcode::Sar => "SAR",

            Opcode::Eq => "EQ",
            Opcode::Ne => "NE",
            Opcode::Lt => "LT",
            Opcode::Le => "LE",
            Opcode::Gt => "GT",
            Opcode::Ge => "GE",
            Opcode::Ltu => "LTU",
            Opcode::Geu => "GEU",

            Opcode::Load => "LOAD",
            Opcode::Store => "STORE",
            Opcode::Load8 => "LOAD8",
            Opcode::Load16 => "LOAD16",
            Opcode::Store8 => "STORE8",
            Opcode::Store16 => "STORE16",

            Opcode::Jmp => "JMP",
            Opcode::JmpI => "JMPI",
            Opcode::Beq => "BEQ",
            Opcode::Bne => "BNE",
            Opcode::Blt => "BLT",
            Opcode::Bge => "BGE",
            Opcode::Bltu => "BLTU",
            Opcode::Bgeu => "BGEU",

            Opcode::Call => "CALL",
            Opcode::CallI => "CALLI",
            Opcode::Ret => "RET",

            Opcode::Li => "LI",
            Opcode::Lui => "LUI",
            Opcode::Mov => "MOV",

            Opcode::FAdd => "FADD",
            Opcode::FSub => "FSUB",
            Opcode::FMul => "FMUL",
            Opcode::FInv => "FINV",
            Opcode::FNeg => "FNEG",

            Opcode::Hash => "HASH",
            Opcode::Hash4 => "HASH4",
            Opcode::AssertEq => "ASSERT_EQ",
            Opcode::AssertZero => "ASSERT_ZERO",
            Opcode::RangeCheck => "RANGE_CHECK",

            Opcode::Read => "READ",
            Opcode::Write => "WRITE",
            Opcode::Commit => "COMMIT",

            Opcode::Nop => "NOP",
            Opcode::Halt => "HALT",
            Opcode::Invalid => "INVALID",
        }
    }

    /// Parse mnemonic string to opcode
    pub fn from_mnemonic(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "ADD" => Some(Opcode::Add),
            "SUB" => Some(Opcode::Sub),
            "MUL" => Some(Opcode::Mul),
            "DIV" => Some(Opcode::Div),
            "SDIV" => Some(Opcode::SDiv),
            "MOD" => Some(Opcode::Mod),
            "SMOD" => Some(Opcode::SMod),
            "NEG" => Some(Opcode::Neg),

            "ADDI" => Some(Opcode::AddI),
            "SUBI" => Some(Opcode::SubI),
            "MULI" => Some(Opcode::MulI),

            "AND" => Some(Opcode::And),
            "OR" => Some(Opcode::Or),
            "XOR" => Some(Opcode::Xor),
            "NOT" => Some(Opcode::Not),
            "SHL" => Some(Opcode::Shl),
            "SHR" => Some(Opcode::Shr),
            "SAR" => Some(Opcode::Sar),

            "EQ" => Some(Opcode::Eq),
            "NE" => Some(Opcode::Ne),
            "LT" => Some(Opcode::Lt),
            "LE" => Some(Opcode::Le),
            "GT" => Some(Opcode::Gt),
            "GE" => Some(Opcode::Ge),
            "LTU" => Some(Opcode::Ltu),
            "GEU" => Some(Opcode::Geu),

            "LOAD" => Some(Opcode::Load),
            "STORE" => Some(Opcode::Store),
            "LOAD8" => Some(Opcode::Load8),
            "LOAD16" => Some(Opcode::Load16),
            "STORE8" => Some(Opcode::Store8),
            "STORE16" => Some(Opcode::Store16),

            "JMP" => Some(Opcode::Jmp),
            "JMPI" => Some(Opcode::JmpI),
            "BEQ" => Some(Opcode::Beq),
            "BNE" => Some(Opcode::Bne),
            "BLT" => Some(Opcode::Blt),
            "BGE" => Some(Opcode::Bge),
            "BLTU" => Some(Opcode::Bltu),
            "BGEU" => Some(Opcode::Bgeu),

            "CALL" => Some(Opcode::Call),
            "CALLI" => Some(Opcode::CallI),
            "RET" => Some(Opcode::Ret),

            "LI" => Some(Opcode::Li),
            "LUI" => Some(Opcode::Lui),
            "MOV" => Some(Opcode::Mov),

            "FADD" => Some(Opcode::FAdd),
            "FSUB" => Some(Opcode::FSub),
            "FMUL" => Some(Opcode::FMul),
            "FINV" => Some(Opcode::FInv),
            "FNEG" => Some(Opcode::FNeg),

            "HASH" => Some(Opcode::Hash),
            "HASH4" => Some(Opcode::Hash4),
            "ASSERT_EQ" => Some(Opcode::AssertEq),
            "ASSERT_ZERO" => Some(Opcode::AssertZero),
            "RANGE_CHECK" => Some(Opcode::RangeCheck),

            "READ" => Some(Opcode::Read),
            "WRITE" => Some(Opcode::Write),
            "COMMIT" => Some(Opcode::Commit),

            "NOP" => Some(Opcode::Nop),
            "HALT" => Some(Opcode::Halt),
            "INVALID" => Some(Opcode::Invalid),

            _ => None,
        }
    }

    /// Get the instruction format for this opcode
    pub fn format(self) -> InstructionFormat {
        match self {
            // R-format: opcode dst src1 src2
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div |
            Opcode::SDiv | Opcode::Mod | Opcode::SMod |
            Opcode::And | Opcode::Or | Opcode::Xor |
            Opcode::Shl | Opcode::Shr | Opcode::Sar |
            Opcode::Eq | Opcode::Ne | Opcode::Lt | Opcode::Le |
            Opcode::Gt | Opcode::Ge | Opcode::Ltu | Opcode::Geu |
            Opcode::FAdd | Opcode::FSub | Opcode::FMul |
            Opcode::Hash => InstructionFormat::R,

            // R2-format: opcode dst src1 (unary)
            Opcode::Neg | Opcode::Not | Opcode::Mov |
            Opcode::FInv | Opcode::FNeg => InstructionFormat::R2,

            // I-format: opcode dst src1 imm32
            Opcode::AddI | Opcode::SubI | Opcode::MulI |
            Opcode::Load | Opcode::Store |
            Opcode::Load8 | Opcode::Load16 | Opcode::Store8 | Opcode::Store16 |
            Opcode::RangeCheck => InstructionFormat::I,

            // J-format: opcode target
            Opcode::Jmp | Opcode::Call => InstructionFormat::J,

            // B-format: opcode src1 src2 target
            Opcode::Beq | Opcode::Bne | Opcode::Blt | Opcode::Bge |
            Opcode::Bltu | Opcode::Bgeu => InstructionFormat::B,

            // U-format: opcode dst imm32
            Opcode::Li | Opcode::Lui | Opcode::Read => InstructionFormat::U,

            // S-format: opcode src1 (single register)
            Opcode::JmpI | Opcode::CallI | Opcode::Write | Opcode::Commit |
            Opcode::AssertZero => InstructionFormat::S,

            // A-format: opcode src1 src2 (assert)
            Opcode::AssertEq => InstructionFormat::A,

            // N-format: opcode only
            Opcode::Ret | Opcode::Nop | Opcode::Halt | Opcode::Invalid => InstructionFormat::N,

            // Special
            Opcode::Hash4 => InstructionFormat::R4,
        }
    }

    /// Estimated number of constraints for this opcode
    pub fn estimated_constraints(self) -> u32 {
        match self {
            // Simple field operations
            Opcode::Add | Opcode::Sub | Opcode::Mul |
            Opcode::FAdd | Opcode::FSub | Opcode::FMul => 1,

            Opcode::Neg | Opcode::FNeg | Opcode::Mov => 1,

            // Division requires more (inverse + range check)
            Opcode::Div | Opcode::SDiv | Opcode::Mod | Opcode::SMod => 15,

            // Field inverse
            Opcode::FInv => 2,

            // Bitwise (requires decomposition)
            Opcode::And | Opcode::Or | Opcode::Xor | Opcode::Not => 25,
            Opcode::Shl | Opcode::Shr | Opcode::Sar => 20,

            // Comparison
            Opcode::Eq | Opcode::Ne | Opcode::Lt | Opcode::Le |
            Opcode::Gt | Opcode::Ge | Opcode::Ltu | Opcode::Geu => 3,

            // Memory (includes consistency check)
            Opcode::Load | Opcode::Store => 5,
            Opcode::Load8 | Opcode::Load16 | Opcode::Store8 | Opcode::Store16 => 8,

            // Control flow
            Opcode::Jmp | Opcode::JmpI => 1,
            Opcode::Beq | Opcode::Bne | Opcode::Blt | Opcode::Bge |
            Opcode::Bltu | Opcode::Bgeu => 4,

            // Function calls
            Opcode::Call | Opcode::CallI => 6,
            Opcode::Ret => 4,

            // Constants
            Opcode::Li | Opcode::Lui => 1,
            Opcode::AddI | Opcode::SubI | Opcode::MulI => 2,

            // Hash (Poseidon)
            Opcode::Hash => 250,
            Opcode::Hash4 => 400,

            // Assertions
            Opcode::AssertEq => 1,
            Opcode::AssertZero => 1,
            Opcode::RangeCheck => 32, // depends on bit width

            // I/O
            Opcode::Read | Opcode::Write | Opcode::Commit => 2,

            // System
            Opcode::Nop => 0,
            Opcode::Halt => 1,
            Opcode::Invalid => 1,
        }
    }
}

/// Instruction format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionFormat {
    /// R-format: opcode dst src1 src2 (3 registers)
    R,
    /// R2-format: opcode dst src1 (2 registers, unary)
    R2,
    /// R4-format: opcode dst src1 src2 src3 src4 (5 registers, for Hash4)
    R4,
    /// I-format: opcode dst src1 imm32 (immediate)
    I,
    /// J-format: opcode target (jump target)
    J,
    /// B-format: opcode src1 src2 target (branch)
    B,
    /// U-format: opcode dst imm32 (upper immediate / load immediate)
    U,
    /// S-format: opcode src1 (single register source)
    S,
    /// A-format: opcode src1 src2 (assert two values)
    A,
    /// N-format: opcode only (no operands)
    N,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_roundtrip() {
        for byte in 0..=255u8 {
            if let Some(opcode) = Opcode::from_byte(byte) {
                assert_eq!(opcode.to_byte(), byte);
            }
        }
    }

    #[test]
    fn test_mnemonic_roundtrip() {
        let opcodes = [
            Opcode::Add, Opcode::Sub, Opcode::Mul, Opcode::Load,
            Opcode::Store, Opcode::Jmp, Opcode::Call, Opcode::Ret,
            Opcode::Hash, Opcode::Halt,
        ];

        for opcode in opcodes {
            let mnemonic = opcode.mnemonic();
            let parsed = Opcode::from_mnemonic(mnemonic);
            assert_eq!(parsed, Some(opcode));
        }
    }
}
