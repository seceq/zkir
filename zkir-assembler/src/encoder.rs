//! Instruction encoding to 30-bit format (ZK IR v2.2)
//!
//! Instructions are 30 bits stored in 32-bit slots (bits 31:30 = 0)

use zkir_spec::{Instruction, Register};

/// Encode instruction to 32-bit word (30-bit instruction, upper 2 bits zero)
pub fn encode(instr: &Instruction) -> u32 {
    match instr {
        // ========== R-type ALU (opcode = 0000) ==========

        // Arithmetic
        Instruction::Add { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b000, 0b00, 0b0000),
        Instruction::Sub { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b000, 0b01, 0b0000),
        Instruction::Mul { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b000, 0b10, 0b0000),
        Instruction::Mulh { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b000, 0b11, 0b0000),
        Instruction::Mulhu { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b101, 0b10, 0b0000),
        Instruction::Mulhsu { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b101, 0b11, 0b0000),

        // Division
        Instruction::Div { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b110, 0b00, 0b0000),
        Instruction::Divu { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b110, 0b01, 0b0000),
        Instruction::Rem { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b110, 0b10, 0b0000),
        Instruction::Remu { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b110, 0b11, 0b0000),

        // Logic
        Instruction::And { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b001, 0b00, 0b0000),
        Instruction::Andn { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b001, 0b01, 0b0000),
        Instruction::Or { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b001, 0b10, 0b0000),
        Instruction::Orn { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b001, 0b11, 0b0000),
        Instruction::Xor { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b010, 0b00, 0b0000),
        Instruction::Xnor { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b010, 0b01, 0b0000),

        // Shift
        Instruction::Sll { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b010, 0b10, 0b0000),
        Instruction::Srl { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b011, 0b00, 0b0000),
        Instruction::Sra { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b011, 0b01, 0b0000),
        Instruction::Rol { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b010, 0b11, 0b0000),
        Instruction::Ror { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b011, 0b10, 0b0000),

        // Compare
        Instruction::Slt { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b100, 0b00, 0b0000),
        Instruction::Sltu { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b100, 0b01, 0b0000),
        Instruction::Min { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b100, 0b10, 0b0000),
        Instruction::Max { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b100, 0b11, 0b0000),
        Instruction::Minu { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b101, 0b00, 0b0000),
        Instruction::Maxu { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b101, 0b01, 0b0000),

        // Bit Manipulation
        Instruction::Clz { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b011, 0b11, 0b0000),
        Instruction::Ctz { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b111, 0b10, 0b0000),
        Instruction::Cpop { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b111, 0b01, 0b0000),
        Instruction::Rev8 { rd, rs1, rs2 } => encode_r_type(*rd, *rs1, *rs2, 0b111, 0b00, 0b0000),

        // Conditional Move (ext2)
        Instruction::Cmovz { rd, rs1, rs2 } => encode_r_type_ext2(*rd, *rs1, *rs2, 0b000000),
        Instruction::Cmovnz { rd, rs1, rs2 } => encode_r_type_ext2(*rd, *rs1, *rs2, 0b000001),

        // Field Operations (special encoding: bits 29:24 = 111111)
        Instruction::Fadd { rd, rs1, rs2 } => encode_field_op(*rd, *rs1, *rs2, 0b000),
        Instruction::Fsub { rd, rs1, rs2 } => encode_field_op(*rd, *rs1, *rs2, 0b001),
        Instruction::Fmul { rd, rs1, rs2 } => encode_field_op(*rd, *rs1, *rs2, 0b010),
        Instruction::Fneg { rd, rs1, rs2 } => encode_field_op(*rd, *rs1, *rs2, 0b011),
        Instruction::Finv { rd, rs1, rs2 } => encode_field_op(*rd, *rs1, *rs2, 0b100),

        // ========== I-type Immediate (opcode = 0001) ==========
        Instruction::Addi { rd, rs1, imm } => encode_i_type(*rd, *rs1, *imm as u32, 0b000, 0b0001),
        Instruction::Slti { rd, rs1, imm } => encode_i_type(*rd, *rs1, *imm as u32, 0b010, 0b0001),
        Instruction::Sltiu { rd, rs1, imm } => encode_i_type(*rd, *rs1, *imm as u32, 0b011, 0b0001),
        Instruction::Xori { rd, rs1, imm } => encode_i_type(*rd, *rs1, *imm as u32, 0b100, 0b0001),
        Instruction::Ori { rd, rs1, imm } => encode_i_type(*rd, *rs1, *imm as u32, 0b110, 0b0001),
        Instruction::Andi { rd, rs1, imm } => encode_i_type(*rd, *rs1, *imm as u32, 0b111, 0b0001),
        Instruction::Slli { rd, rs1, shamt } => encode_i_type(*rd, *rs1, *shamt as u32, 0b001, 0b0001),
        Instruction::Srli { rd, rs1, shamt } => encode_i_type(*rd, *rs1, *shamt as u32, 0b101, 0b0001),
        Instruction::Srai { rd, rs1, shamt } => encode_i_type(*rd, *rs1, (*shamt as u32) | (1 << 12), 0b101, 0b0001),

        // ========== Load (opcode = 0010) ==========
        Instruction::Lb { rd, rs1, imm } => encode_i_type(*rd, *rs1, *imm as u32, 0b000, 0b0010),
        Instruction::Lh { rd, rs1, imm } => encode_i_type(*rd, *rs1, *imm as u32, 0b001, 0b0010),
        Instruction::Lw { rd, rs1, imm } => encode_i_type(*rd, *rs1, *imm as u32, 0b010, 0b0010),
        Instruction::Lbu { rd, rs1, imm } => encode_i_type(*rd, *rs1, *imm as u32, 0b100, 0b0010),
        Instruction::Lhu { rd, rs1, imm } => encode_i_type(*rd, *rs1, *imm as u32, 0b101, 0b0010),

        // ========== Store (opcode = 0011) ==========
        Instruction::Sb { rs1, rs2, imm } => encode_s_type(*rs1, *rs2, *imm as u32, 0b000, 0b0011),
        Instruction::Sh { rs1, rs2, imm } => encode_s_type(*rs1, *rs2, *imm as u32, 0b001, 0b0011),
        Instruction::Sw { rs1, rs2, imm } => encode_s_type(*rs1, *rs2, *imm as u32, 0b010, 0b0011),

        // ========== Branch (opcodes 0100-1001) ==========
        Instruction::Beq { rs1, rs2, imm } => encode_b_type(*rs1, *rs2, *imm as u32, 0b0100),
        Instruction::Bne { rs1, rs2, imm } => encode_b_type(*rs1, *rs2, *imm as u32, 0b0101),
        Instruction::Blt { rs1, rs2, imm } => encode_b_type(*rs1, *rs2, *imm as u32, 0b0110),
        Instruction::Bge { rs1, rs2, imm } => encode_b_type(*rs1, *rs2, *imm as u32, 0b0111),
        Instruction::Bltu { rs1, rs2, imm } => encode_b_type(*rs1, *rs2, *imm as u32, 0b1000),
        Instruction::Bgeu { rs1, rs2, imm } => encode_b_type(*rs1, *rs2, *imm as u32, 0b1001),

        // ========== Upper Immediate (opcodes 1010-1011) ==========
        Instruction::Lui { rd, imm } => encode_u_type(*rd, *imm as u32, 0b1010),
        Instruction::Auipc { rd, imm } => encode_u_type(*rd, *imm as u32, 0b1011),

        // ========== Jump (opcodes 1100-1101) ==========
        Instruction::Jal { rd, imm } => encode_j_type(*rd, *imm as u32, 0b1100),
        Instruction::Jalr { rd, rs1, imm } => encode_i_type(*rd, *rs1, *imm as u32, 0b000, 0b1101),

        // ========== ZK Operations (opcode = 1110) ==========
        Instruction::Read { rd } => encode_z_type(*rd, Register::R0, 0b00000, 0, 0b1110),
        Instruction::Write { rs1 } => encode_z_type(Register::R0, *rs1, 0b00001, 0, 0b1110),
        Instruction::Hint { rd } => encode_z_type(*rd, Register::R0, 0b00010, 0, 0b1110),
        Instruction::Commit { rs1 } => encode_z_type(Register::R0, *rs1, 0b00011, 0, 0b1110),
        Instruction::AssertEq { rs1, rs2 } => encode_z_type_rs2(*rs1, *rs2, 0b00100, 0b1110),
        Instruction::AssertNe { rs1, rs2 } => encode_z_type_rs2(*rs1, *rs2, 0b00101, 0b1110),
        Instruction::AssertZero { rs1 } => encode_z_type(Register::R0, *rs1, 0b00110, 0, 0b1110),
        Instruction::RangeCheck { rs1, bits } => encode_z_type(Register::R0, *rs1, 0b00111, *bits as u32, 0b1110),
        Instruction::Debug { rs1 } => encode_z_type(Register::R0, *rs1, 0b01000, 0, 0b1110),
        Instruction::Halt => encode_z_type(Register::R0, Register::R0, 0b11111, 0, 0b1110),

        // ========== System (opcode = 1111) ==========
        Instruction::Ecall => encode_i_type(Register::R0, Register::R0, 0, 0b000, 0b1111),
        Instruction::Ebreak => encode_i_type(Register::R0, Register::R0, 1, 0b000, 0b1111),
    }
}

/// Encode R-type instruction
/// Format: | unused(6) | ext(2) | funct(3) | rs2(5) | rs1(5) | rd(5) | opcode(4) |
fn encode_r_type(rd: Register, rs1: Register, rs2: Register, funct: u32, ext: u32, opcode: u32) -> u32 {
    let mut instr = 0u32;
    instr |= opcode & 0xF;                      // bits 3:0
    instr |= (rd.index() as u32 & 0x1F) << 4;   // bits 8:4
    instr |= (rs1.index() as u32 & 0x1F) << 9;  // bits 13:9
    instr |= (rs2.index() as u32 & 0x1F) << 14; // bits 18:14
    instr |= (funct & 0x7) << 19;               // bits 21:19
    instr |= (ext & 0x3) << 22;                 // bits 23:22
    // bits 29:24 = 0 (unused)
    instr
}

/// Encode R-type with ext2 extension (for CMOVZ/CMOVNZ)
/// Format: | ext2(6) | ext=11 | funct=111 | rs2(5) | rs1(5) | rd(5) | opcode(4) |
fn encode_r_type_ext2(rd: Register, rs1: Register, rs2: Register, ext2: u32) -> u32 {
    let mut instr = 0u32;
    instr |= 0b0000;                            // opcode = 0000
    instr |= (rd.index() as u32 & 0x1F) << 4;   // bits 8:4
    instr |= (rs1.index() as u32 & 0x1F) << 9;  // bits 13:9
    instr |= (rs2.index() as u32 & 0x1F) << 14; // bits 18:14
    instr |= 0b111 << 19;                       // funct = 111
    instr |= 0b11 << 22;                        // ext = 11
    instr |= (ext2 & 0x3F) << 24;               // bits 29:24 = ext2
    instr
}

/// Encode field operation (special R-type encoding)
/// Format: | 111111 | ext=00 | funct | rs2(5) | rs1(5) | rd(5) | opcode=0000 |
fn encode_field_op(rd: Register, rs1: Register, rs2: Register, funct: u32) -> u32 {
    let mut instr = 0u32;
    instr |= 0b0000;                            // opcode = 0000
    instr |= (rd.index() as u32 & 0x1F) << 4;   // bits 8:4
    instr |= (rs1.index() as u32 & 0x1F) << 9;  // bits 13:9
    instr |= (rs2.index() as u32 & 0x1F) << 14; // bits 18:14
    instr |= (funct & 0x7) << 19;               // bits 21:19
    instr |= 0b00 << 22;                        // ext = 00
    instr |= 0b111111 << 24;                    // bits 29:24 = 111111 (field op marker)
    instr
}

/// Encode I-type instruction
/// Format: | funct(3) | imm(13) | rs1(5) | rd(5) | opcode(4) |
fn encode_i_type(rd: Register, rs1: Register, imm: u32, funct: u32, opcode: u32) -> u32 {
    let mut instr = 0u32;
    instr |= opcode & 0xF;                      // bits 3:0
    instr |= (rd.index() as u32 & 0x1F) << 4;   // bits 8:4
    instr |= (rs1.index() as u32 & 0x1F) << 9;  // bits 13:9
    instr |= (imm & 0x1FFF) << 14;              // bits 26:14 (13-bit imm)
    instr |= (funct & 0x7) << 27;               // bits 29:27
    instr
}

/// Encode S-type instruction (Store)
/// Format: | imm(13) | rs2(5) | rs1(5) | funct(3) | opcode(4) |
fn encode_s_type(rs1: Register, rs2: Register, imm: u32, funct: u32, opcode: u32) -> u32 {
    let mut instr = 0u32;
    instr |= opcode & 0xF;                      // bits 3:0
    instr |= (funct & 0x7) << 4;                // bits 6:4
    instr |= (rs1.index() as u32 & 0x1F) << 7;  // bits 11:7
    instr |= (rs2.index() as u32 & 0x1F) << 12; // bits 16:12
    instr |= (imm & 0x1FFF) << 17;              // bits 29:17 (13-bit imm)
    instr
}

/// Encode B-type instruction (Branch)
/// Same as S-type but with different semantics
fn encode_b_type(rs1: Register, rs2: Register, imm: u32, opcode: u32) -> u32 {
    let mut instr = 0u32;
    instr |= opcode & 0xF;                      // bits 3:0
    instr |= (rs2.index() as u32 & 0x1F) << 4;  // bits 8:4 (note: rs2 in rd position)
    instr |= (rs1.index() as u32 & 0x1F) << 9;  // bits 13:9
    instr |= (imm & 0xFFFF) << 14;              // bits 29:14 (16-bit imm)
    instr
}

/// Encode U-type instruction (Upper Immediate)
/// Format: | imm_hi(21) | rd(5) | opcode(4) |
fn encode_u_type(rd: Register, imm: u32, opcode: u32) -> u32 {
    let mut instr = 0u32;
    instr |= opcode & 0xF;                      // bits 3:0
    instr |= (rd.index() as u32 & 0x1F) << 4;   // bits 8:4
    instr |= (imm & 0x1FFFFF) << 9;             // bits 29:9 (21-bit imm)
    instr
}

/// Encode J-type instruction (Jump)
/// Format: | imm(21) | rd(5) | opcode(4) |
fn encode_j_type(rd: Register, imm: u32, opcode: u32) -> u32 {
    // Same as U-type for v2.2
    encode_u_type(rd, imm, opcode)
}

/// Encode Z-type instruction (ZK Operations)
/// Format: | func(5) | imm(8) | rs1(5) | rd(5) | opcode(4) |
fn encode_z_type(rd: Register, rs1: Register, func: u32, imm: u32, opcode: u32) -> u32 {
    let mut instr = 0u32;
    instr |= opcode & 0xF;                      // bits 3:0
    instr |= (rd.index() as u32 & 0x1F) << 7;   // bits 11:7 (note: different position)
    instr |= (rs1.index() as u32 & 0x1F) << 12; // bits 16:12
    instr |= (imm & 0xFF) << 17;                // bits 24:17
    instr |= (func & 0x1F) << 25;               // bits 29:25
    instr
}

/// Encode Z-type with rs2 (for ASSERT_EQ, ASSERT_NE)
fn encode_z_type_rs2(rs1: Register, rs2: Register, func: u32, opcode: u32) -> u32 {
    let mut instr = 0u32;
    instr |= opcode & 0xF;                      // bits 3:0
    instr |= (rs2.index() as u32 & 0x1F) << 7;  // bits 11:7 (rs2 in rd position)
    instr |= (rs1.index() as u32 & 0x1F) << 12; // bits 16:12
    instr |= (func & 0x1F) << 25;               // bits 29:25
    instr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_add() {
        // ADD r5, r6, r7
        let instr = encode(&Instruction::Add {
            rd: Register::R5,
            rs1: Register::R6,
            rs2: Register::R7,
        });

        // opcode=0000, rd=5, rs1=6, rs2=7, funct=000, ext=00
        let opcode = instr & 0xF;
        let rd = (instr >> 4) & 0x1F;
        let rs1 = (instr >> 9) & 0x1F;
        let rs2 = (instr >> 14) & 0x1F;

        assert_eq!(opcode, 0b0000);
        assert_eq!(rd, 5);
        assert_eq!(rs1, 6);
        assert_eq!(rs2, 7);
    }

    #[test]
    fn test_encode_addi() {
        // ADDI r5, r6, 100
        let instr = encode(&Instruction::Addi {
            rd: Register::R5,
            rs1: Register::R6,
            imm: 100,
        });

        let opcode = instr & 0xF;
        let rd = (instr >> 4) & 0x1F;
        let rs1 = (instr >> 9) & 0x1F;
        let imm = (instr >> 14) & 0x1FFF;

        assert_eq!(opcode, 0b0001);
        assert_eq!(rd, 5);
        assert_eq!(rs1, 6);
        assert_eq!(imm, 100);
    }

    #[test]
    fn test_encode_fadd() {
        // FADD r5, r6, r7
        let instr = encode(&Instruction::Fadd {
            rd: Register::R5,
            rs1: Register::R6,
            rs2: Register::R7,
        });

        // Check field op marker (bits 29:24 = 111111)
        let marker = (instr >> 24) & 0x3F;
        assert_eq!(marker, 0b111111);
    }

    #[test]
    fn test_encode_ecall() {
        let instr = encode(&Instruction::Ecall);
        let opcode = instr & 0xF;
        assert_eq!(opcode, 0b1111);
    }

    #[test]
    fn test_encode_halt() {
        let instr = encode(&Instruction::Halt);
        let opcode = instr & 0xF;
        let func = (instr >> 25) & 0x1F;
        assert_eq!(opcode, 0b1110);
        assert_eq!(func, 0b11111);
    }

    #[test]
    fn test_instruction_fits_30bits() {
        // Test that all instructions fit in 30 bits (bits 31:30 = 0)
        let instructions = vec![
            Instruction::Add { rd: Register::R5, rs1: Register::R6, rs2: Register::R7 },
            Instruction::Addi { rd: Register::R5, rs1: Register::R6, imm: 100 },
            Instruction::Lw { rd: Register::R5, rs1: Register::R6, imm: 0 },
            Instruction::Sw { rs1: Register::R5, rs2: Register::R6, imm: 0 },
            Instruction::Beq { rs1: Register::R5, rs2: Register::R6, imm: 10 },
            Instruction::Jal { rd: Register::R1, imm: 100 },
            Instruction::Ecall,
            Instruction::Halt,
        ];

        for instr in instructions {
            let encoded = encode(&instr);
            assert_eq!(encoded & 0xC0000000, 0, "Instruction has bits set in 31:30");
        }
    }
}
