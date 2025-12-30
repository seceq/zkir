//! Instruction encoding to 32-bit format (ZKIR v3.4)
//!
//! ## Instruction Formats (32 bits total, 7-bit opcode field):
//! - R-type:  [opcode:7][rd:4][rs1:4][rs2:4][funct:13]  = 7+4+4+4+13 = 32 bits
//! - I-type:  [opcode:7][rd:4][rs1:4][imm:17]           = 7+4+4+17 = 32 bits
//! - S-type:  [opcode:7][rs1:4][rs2:4][imm:17]          = 7+4+4+17 = 32 bits
//! - B-type:  [opcode:7][rs1:4][rs2:4][offset:17]       = 7+4+4+17 = 32 bits
//! - J-type:  [opcode:7][rd:4][offset:21]               = 7+4+21 = 32 bits
//!
//! Note: Despite documentation claiming "6-bit opcodes", the actual opcode values
//! range from 0x00-0x51 which requires 7 bits. We use 7 bits for the opcode field.

use zkir_spec::{Instruction, Opcode, Register};

/// Encode instruction to 32-bit word
///
/// Uses opcodes from the ZKIR v3.4 spec (values 0x00-0x51)
pub fn encode(instr: &Instruction) -> u32 {
    match instr {
        // ========== Arithmetic (R-type: 0x00-0x07) ==========
        Instruction::Add { rd, rs1, rs2 } => encode_r_type(Opcode::Add.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Sub { rd, rs1, rs2 } => encode_r_type(Opcode::Sub.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Mul { rd, rs1, rs2 } => encode_r_type(Opcode::Mul.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Mulh { rd, rs1, rs2 } => encode_r_type(Opcode::Mulh.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Divu { rd, rs1, rs2 } => encode_r_type(Opcode::Divu.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Remu { rd, rs1, rs2 } => encode_r_type(Opcode::Remu.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Div { rd, rs1, rs2 } => encode_r_type(Opcode::Div.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Rem { rd, rs1, rs2 } => encode_r_type(Opcode::Rem.to_u8() as u32, *rd, *rs1, *rs2, 0),

        // ========== Logical (R-type: 0x10-0x12) ==========
        Instruction::And { rd, rs1, rs2 } => encode_r_type(Opcode::And.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Or { rd, rs1, rs2 } => encode_r_type(Opcode::Or.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Xor { rd, rs1, rs2 } => encode_r_type(Opcode::Xor.to_u8() as u32, *rd, *rs1, *rs2, 0),

        // ========== Shift (R-type: 0x18-0x1A) ==========
        Instruction::Sll { rd, rs1, rs2 } => encode_r_type(Opcode::Sll.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Srl { rd, rs1, rs2 } => encode_r_type(Opcode::Srl.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Sra { rd, rs1, rs2 } => encode_r_type(Opcode::Sra.to_u8() as u32, *rd, *rs1, *rs2, 0),

        // ========== Compare (R-type: 0x20-0x25) ==========
        Instruction::Sltu { rd, rs1, rs2 } => encode_r_type(Opcode::Sltu.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Sgeu { rd, rs1, rs2 } => encode_r_type(Opcode::Sgeu.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Slt { rd, rs1, rs2 } => encode_r_type(Opcode::Slt.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Sge { rd, rs1, rs2 } => encode_r_type(Opcode::Sge.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Seq { rd, rs1, rs2 } => encode_r_type(Opcode::Seq.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Sne { rd, rs1, rs2 } => encode_r_type(Opcode::Sne.to_u8() as u32, *rd, *rs1, *rs2, 0),

        // ========== Conditional Move (R-type: 0x26-0x28) ==========
        Instruction::Cmov { rd, rs1, rs2 } => encode_r_type(Opcode::Cmov.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Cmovz { rd, rs1, rs2 } => encode_r_type(Opcode::Cmovz.to_u8() as u32, *rd, *rs1, *rs2, 0),
        Instruction::Cmovnz { rd, rs1, rs2 } => encode_r_type(Opcode::Cmovnz.to_u8() as u32, *rd, *rs1, *rs2, 0),

        // ========== Immediate Arithmetic (I-type: 0x08) ==========
        Instruction::Addi { rd, rs1, imm } => encode_i_type(Opcode::Addi.to_u8() as u32, *rd, *rs1, *imm),

        // ========== Immediate Logical (I-type: 0x13-0x15) ==========
        Instruction::Andi { rd, rs1, imm } => encode_i_type(Opcode::Andi.to_u8() as u32, *rd, *rs1, *imm),
        Instruction::Ori { rd, rs1, imm } => encode_i_type(Opcode::Ori.to_u8() as u32, *rd, *rs1, *imm),
        Instruction::Xori { rd, rs1, imm } => encode_i_type(Opcode::Xori.to_u8() as u32, *rd, *rs1, *imm),

        // ========== Shift Immediate (I-type: 0x1B-0x1D) ==========
        Instruction::Slli { rd, rs1, shamt } => encode_i_type(Opcode::Slli.to_u8() as u32, *rd, *rs1, *shamt as i32),
        Instruction::Srli { rd, rs1, shamt } => encode_i_type(Opcode::Srli.to_u8() as u32, *rd, *rs1, *shamt as i32),
        Instruction::Srai { rd, rs1, shamt } => encode_i_type(Opcode::Srai.to_u8() as u32, *rd, *rs1, *shamt as i32),

        // ========== Load (I-type: 0x30-0x35) ==========
        Instruction::Lb { rd, rs1, imm } => encode_i_type(Opcode::Lb.to_u8() as u32, *rd, *rs1, *imm),
        Instruction::Lbu { rd, rs1, imm } => encode_i_type(Opcode::Lbu.to_u8() as u32, *rd, *rs1, *imm),
        Instruction::Lh { rd, rs1, imm } => encode_i_type(Opcode::Lh.to_u8() as u32, *rd, *rs1, *imm),
        Instruction::Lhu { rd, rs1, imm } => encode_i_type(Opcode::Lhu.to_u8() as u32, *rd, *rs1, *imm),
        Instruction::Lw { rd, rs1, imm } => encode_i_type(Opcode::Lw.to_u8() as u32, *rd, *rs1, *imm),
        Instruction::Ld { rd, rs1, imm } => encode_i_type(Opcode::Ld.to_u8() as u32, *rd, *rs1, *imm),

        // ========== Store (S-type: 0x38-0x3B) ==========
        Instruction::Sb { rs1, rs2, imm } => encode_s_type(Opcode::Sb.to_u8() as u32, *rs1, *rs2, *imm),
        Instruction::Sh { rs1, rs2, imm } => encode_s_type(Opcode::Sh.to_u8() as u32, *rs1, *rs2, *imm),
        Instruction::Sw { rs1, rs2, imm } => encode_s_type(Opcode::Sw.to_u8() as u32, *rs1, *rs2, *imm),
        Instruction::Sd { rs1, rs2, imm } => encode_s_type(Opcode::Sd.to_u8() as u32, *rs1, *rs2, *imm),

        // ========== Branch (B-type: 0x40-0x45) ==========
        Instruction::Beq { rs1, rs2, offset } => encode_b_type(Opcode::Beq.to_u8() as u32, *rs1, *rs2, *offset),
        Instruction::Bne { rs1, rs2, offset } => encode_b_type(Opcode::Bne.to_u8() as u32, *rs1, *rs2, *offset),
        Instruction::Blt { rs1, rs2, offset } => encode_b_type(Opcode::Blt.to_u8() as u32, *rs1, *rs2, *offset),
        Instruction::Bge { rs1, rs2, offset } => encode_b_type(Opcode::Bge.to_u8() as u32, *rs1, *rs2, *offset),
        Instruction::Bltu { rs1, rs2, offset } => encode_b_type(Opcode::Bltu.to_u8() as u32, *rs1, *rs2, *offset),
        Instruction::Bgeu { rs1, rs2, offset } => encode_b_type(Opcode::Bgeu.to_u8() as u32, *rs1, *rs2, *offset),

        // ========== Jump (J-type: 0x48, I-type: 0x49) ==========
        Instruction::Jal { rd, offset } => encode_j_type(Opcode::Jal.to_u8() as u32, *rd, *offset),
        Instruction::Jalr { rd, rs1, imm } => encode_i_type(Opcode::Jalr.to_u8() as u32, *rd, *rs1, *imm),

        // ========== System (0x50-0x51) ==========
        Instruction::Ecall => encode_i_type(Opcode::Ecall.to_u8() as u32, Register::R0, Register::R0, 0),
        Instruction::Ebreak => encode_i_type(Opcode::Ebreak.to_u8() as u32, Register::R0, Register::R0, 0),
    }
}

/// Encode R-type instruction
/// Format: [opcode:7][rd:4][rs1:4][rs2:4][funct:13]
fn encode_r_type(opcode: u32, rd: Register, rs1: Register, rs2: Register, funct: u32) -> u32 {
    let mut instr = 0u32;
    instr |= opcode & 0x7F;                          // bits 6:0 (7-bit opcode)
    instr |= (rd as u32 & 0xF) << 7;                 // bits 10:7 (4-bit rd)
    instr |= (rs1 as u32 & 0xF) << 11;               // bits 14:11 (4-bit rs1)
    instr |= (rs2 as u32 & 0xF) << 15;               // bits 18:15 (4-bit rs2)
    instr |= (funct & 0x1FFF) << 19;                 // bits 31:19 (13-bit funct)
    instr
}

/// Encode I-type instruction
/// Format: [opcode:7][rd:4][rs1:4][imm:17]
fn encode_i_type(opcode: u32, rd: Register, rs1: Register, imm: i32) -> u32 {
    let mut instr = 0u32;
    instr |= opcode & 0x7F;                          // bits 6:0 (7-bit opcode)
    instr |= (rd as u32 & 0xF) << 7;                 // bits 10:7 (4-bit rd)
    instr |= (rs1 as u32 & 0xF) << 11;               // bits 14:11 (4-bit rs1)
    instr |= ((imm as u32) & 0x1FFFF) << 15;         // bits 31:15 (17-bit imm)
    instr
}

/// Encode S-type instruction (stores)
/// Format: [opcode:7][rs1:4][rs2:4][imm:17]
fn encode_s_type(opcode: u32, rs1: Register, rs2: Register, imm: i32) -> u32 {
    let mut instr = 0u32;
    instr |= opcode & 0x7F;                          // bits 6:0 (7-bit opcode)
    instr |= (rs1 as u32 & 0xF) << 7;                // bits 10:7 (4-bit rs1 - base address)
    instr |= (rs2 as u32 & 0xF) << 11;               // bits 14:11 (4-bit rs2 - value to store)
    instr |= ((imm as u32) & 0x1FFFF) << 15;         // bits 31:15 (17-bit imm)
    instr
}

/// Encode B-type instruction
/// Format: [opcode:7][rs1:4][rs2:4][offset:17]
fn encode_b_type(opcode: u32, rs1: Register, rs2: Register, offset: i32) -> u32 {
    let mut instr = 0u32;
    instr |= opcode & 0x7F;                          // bits 6:0 (7-bit opcode)
    instr |= (rs1 as u32 & 0xF) << 7;                // bits 10:7 (4-bit rs1)
    instr |= (rs2 as u32 & 0xF) << 11;               // bits 14:11 (4-bit rs2)
    instr |= ((offset as u32) & 0x1FFFF) << 15;      // bits 31:15 (17-bit offset)
    instr
}

/// Encode J-type instruction
/// Format: [opcode:7][rd:4][offset:21]
fn encode_j_type(opcode: u32, rd: Register, offset: i32) -> u32 {
    let mut instr = 0u32;
    instr |= opcode & 0x7F;                          // bits 6:0 (7-bit opcode)
    instr |= (rd as u32 & 0xF) << 7;                 // bits 10:7 (4-bit rd)
    instr |= ((offset as u32) & 0x1FFFFF) << 11;     // bits 31:11 (21-bit offset)
    instr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_add() {
        // ADD r4, r5, r6
        let instr = encode(&Instruction::Add {
            rd: Register::R4,
            rs1: Register::R5,
            rs2: Register::R6,
        });

        // Extract fields (7-bit opcode format)
        let opcode = instr & 0x7F;
        let rd = (instr >> 7) & 0xF;
        let rs1 = (instr >> 11) & 0xF;
        let rs2 = (instr >> 15) & 0xF;

        assert_eq!(opcode, Opcode::Add.to_u8() as u32);
        assert_eq!(rd, 4);
        assert_eq!(rs1, 5);
        assert_eq!(rs2, 6);
    }

    #[test]
    fn test_encode_addi() {
        // ADDI r4, r5, 100
        let instr = encode(&Instruction::Addi {
            rd: Register::R4,
            rs1: Register::R5,
            imm: 100,
        });

        let opcode = instr & 0x7F;
        let rd = (instr >> 7) & 0xF;
        let rs1 = (instr >> 11) & 0xF;
        let imm = ((instr >> 15) & 0x1FFFF) as i32;

        assert_eq!(opcode, Opcode::Addi.to_u8() as u32);
        assert_eq!(rd, 4);
        assert_eq!(rs1, 5);
        assert_eq!(imm, 100);
    }

    #[test]
    fn test_encode_and() {
        // AND r2, r3, r4
        let instr = encode(&Instruction::And {
            rd: Register::R2,
            rs1: Register::R3,
            rs2: Register::R4,
        });

        let opcode = instr & 0x7F;
        assert_eq!(opcode, Opcode::And.to_u8() as u32); // 0x10
    }

    #[test]
    fn test_encode_lw() {
        // LW r4, 16(r2)
        let instr = encode(&Instruction::Lw {
            rd: Register::R4,
            rs1: Register::R2,
            imm: 16,
        });

        let opcode = instr & 0x7F;
        assert_eq!(opcode, Opcode::Lw.to_u8() as u32); // 0x34
    }

    #[test]
    fn test_encode_beq() {
        // BEQ r4, r5, 8
        let instr = encode(&Instruction::Beq {
            rs1: Register::R4,
            rs2: Register::R5,
            offset: 8,
        });

        let opcode = instr & 0x7F;
        let rs1 = (instr >> 7) & 0xF;
        let rs2 = (instr >> 11) & 0xF;
        let offset = ((instr >> 15) & 0x1FFFF) as i32;

        assert_eq!(opcode, Opcode::Beq.to_u8() as u32); // 0x40
        assert_eq!(rs1, 4);
        assert_eq!(rs2, 5);
        assert_eq!(offset, 8);
    }

    #[test]
    fn test_encode_jal() {
        // JAL r1, 100
        let instr = encode(&Instruction::Jal {
            rd: Register::R1,
            offset: 100,
        });

        let opcode = instr & 0x7F;
        let rd = (instr >> 7) & 0xF;
        let offset = ((instr >> 11) & 0x1FFFFF) as i32;

        assert_eq!(opcode, Opcode::Jal.to_u8() as u32); // 0x48
        assert_eq!(rd, 1);
        assert_eq!(offset, 100);
    }

    #[test]
    fn test_encode_cmov() {
        // CMOV r4, r5, r6
        let instr = encode(&Instruction::Cmov {
            rd: Register::R4,
            rs1: Register::R5,
            rs2: Register::R6,
        });

        let opcode = instr & 0x7F;
        let rd = (instr >> 7) & 0xF;
        let rs1 = (instr >> 11) & 0xF;
        let rs2 = (instr >> 15) & 0xF;

        assert_eq!(opcode, Opcode::Cmov.to_u8() as u32); // 0x26
        assert_eq!(rd, 4);
        assert_eq!(rs1, 5);
        assert_eq!(rs2, 6);
    }

    #[test]
    fn test_encode_ecall() {
        let instr = encode(&Instruction::Ecall);
        let opcode = instr & 0x7F;
        assert_eq!(opcode, Opcode::Ecall.to_u8() as u32); // 0x50
    }

    #[test]
    fn test_encode_ebreak() {
        let instr = encode(&Instruction::Ebreak);
        let opcode = instr & 0x7F;
        assert_eq!(opcode, Opcode::Ebreak.to_u8() as u32); // 0x51
    }

    #[test]
    fn test_instruction_fits_32bits() {
        // Test that all instructions fit in 32 bits
        let instructions = vec![
            Instruction::Add { rd: Register::R4, rs1: Register::R5, rs2: Register::R6 },
            Instruction::Addi { rd: Register::R4, rs1: Register::R5, imm: 100 },
            Instruction::Lw { rd: Register::R4, rs1: Register::R2, imm: 0 },
            Instruction::Sw { rs1: Register::R2, rs2: Register::R4, imm: 0 },
            Instruction::Beq { rs1: Register::R4, rs2: Register::R5, offset: 10 },
            Instruction::Jal { rd: Register::R1, offset: 100 },
            Instruction::Cmov { rd: Register::R4, rs1: Register::R5, rs2: Register::R6 },
            Instruction::Ecall,
            Instruction::Ebreak,
        ];

        for instr in instructions {
            let _encoded = encode(&instr);
            // All u32 values fit in 32 bits by definition
        }
    }

    #[test]
    fn test_negative_immediate() {
        // Test sign extension for negative immediates
        let instr = encode(&Instruction::Addi {
            rd: Register::R4,
            rs1: Register::R5,
            imm: -1,
        });

        let imm_bits = (instr >> 15) & 0x1FFFF;
        // -1 in 17-bit two's complement should be all 1s
        assert_eq!(imm_bits, 0x1FFFF);
    }

    #[test]
    fn test_opcode_values() {
        // Verify opcodes match ZKIR v3.4 spec (using 7-bit field)
        assert_eq!(Opcode::Add.to_u8(), 0x00);
        assert_eq!(Opcode::Addi.to_u8(), 0x08);
        assert_eq!(Opcode::And.to_u8(), 0x10);
        assert_eq!(Opcode::Sll.to_u8(), 0x18);
        assert_eq!(Opcode::Sltu.to_u8(), 0x20);
        assert_eq!(Opcode::Cmov.to_u8(), 0x26);
        assert_eq!(Opcode::Lb.to_u8(), 0x30);
        assert_eq!(Opcode::Sb.to_u8(), 0x38);
        assert_eq!(Opcode::Beq.to_u8(), 0x40);
        assert_eq!(Opcode::Jal.to_u8(), 0x48);
        assert_eq!(Opcode::Ecall.to_u8(), 0x50);
    }
}
