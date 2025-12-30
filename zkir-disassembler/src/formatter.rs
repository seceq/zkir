//! Instruction formatting to assembly text for ZKIR v3.4

use zkir_spec::{Instruction, Register};

/// Format instruction as assembly text
pub fn format(instr: &Instruction) -> String {
    match instr {
        // ========== System ==========
        Instruction::Ecall => "ecall".to_string(),
        Instruction::Ebreak => "ebreak".to_string(),

        // ========== Arithmetic (R-type) ==========
        Instruction::Add { rd, rs1, rs2 } => {
            format!("add {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Sub { rd, rs1, rs2 } => {
            format!("sub {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Mul { rd, rs1, rs2 } => {
            format!("mul {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Div { rd, rs1, rs2 } => {
            format!("div {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Divu { rd, rs1, rs2 } => {
            format!("divu {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Rem { rd, rs1, rs2 } => {
            format!("rem {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Remu { rd, rs1, rs2 } => {
            format!("remu {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Mulh { rd, rs1, rs2 } => {
            format!("mulh {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }

        // ========== Logical (R-type) ==========
        Instruction::And { rd, rs1, rs2 } => {
            format!("and {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Or { rd, rs1, rs2 } => {
            format!("or {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Xor { rd, rs1, rs2 } => {
            format!("xor {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Sll { rd, rs1, rs2 } => {
            format!("sll {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Slt { rd, rs1, rs2 } => {
            format!("slt {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Sltu { rd, rs1, rs2 } => {
            format!("sltu {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Sge { rd, rs1, rs2 } => {
            format!("sge {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Sgeu { rd, rs1, rs2 } => {
            format!("sgeu {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Seq { rd, rs1, rs2 } => {
            format!("seq {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Sne { rd, rs1, rs2 } => {
            format!("sne {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }

        // ========== Conditional Move (R-type) ==========
        Instruction::Cmov { rd, rs1, rs2 } => {
            format!("cmov {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Cmovz { rd, rs1, rs2 } => {
            format!("cmovz {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::Cmovnz { rd, rs1, rs2 } => {
            format!("cmovnz {}, {}, {}", format_reg(*rd), format_reg(*rs1), format_reg(*rs2))
        }

        // ========== Immediate Arithmetic (I-type) ==========
        Instruction::Addi { rd, rs1, imm } => {
            format!("addi {}, {}, {}", format_reg(*rd), format_reg(*rs1), imm)
        }

        // ========== Immediate Logical (I-type) ==========
        Instruction::Andi { rd, rs1, imm } => {
            format!("andi {}, {}, {}", format_reg(*rd), format_reg(*rs1), imm)
        }
        Instruction::Ori { rd, rs1, imm } => {
            format!("ori {}, {}, {}", format_reg(*rd), format_reg(*rs1), imm)
        }
        Instruction::Xori { rd, rs1, imm } => {
            format!("xori {}, {}, {}", format_reg(*rd), format_reg(*rs1), imm)
        }

        // ========== Shift Immediate (I-type) ==========
        Instruction::Slli { rd, rs1, shamt } => {
            format!("slli {}, {}, {}", format_reg(*rd), format_reg(*rs1), shamt)
        }
        Instruction::Srli { rd, rs1, shamt } => {
            format!("srli {}, {}, {}", format_reg(*rd), format_reg(*rs1), shamt)
        }
        Instruction::Srai { rd, rs1, shamt } => {
            format!("srai {}, {}, {}", format_reg(*rd), format_reg(*rs1), shamt)
        }

        // ========== Load (I-type) ==========
        Instruction::Lb { rd, rs1, imm } => {
            format!("lb {}, {}({})", format_reg(*rd), imm, format_reg(*rs1))
        }
        Instruction::Lh { rd, rs1, imm } => {
            format!("lh {}, {}({})", format_reg(*rd), imm, format_reg(*rs1))
        }
        Instruction::Lw { rd, rs1, imm } => {
            format!("lw {}, {}({})", format_reg(*rd), imm, format_reg(*rs1))
        }
        Instruction::Ld { rd, rs1, imm } => {
            format!("ld {}, {}({})", format_reg(*rd), imm, format_reg(*rs1))
        }

        // ========== Store (S-type) ==========
        Instruction::Sb { rs1, rs2, imm } => {
            format!("sb {}, {}({})", format_reg(*rs2), imm, format_reg(*rs1))
        }
        Instruction::Sh { rs1, rs2, imm } => {
            format!("sh {}, {}({})", format_reg(*rs2), imm, format_reg(*rs1))
        }
        Instruction::Sw { rs1, rs2, imm } => {
            format!("sw {}, {}({})", format_reg(*rs2), imm, format_reg(*rs1))
        }
        Instruction::Sd { rs1, rs2, imm } => {
            format!("sd {}, {}({})", format_reg(*rs2), imm, format_reg(*rs1))
        }

        // ========== Branch (B-type) ==========
        Instruction::Beq { rs1, rs2, offset } => {
            format!("beq {}, {}, {}", format_reg(*rs1), format_reg(*rs2), offset)
        }
        Instruction::Bne { rs1, rs2, offset } => {
            format!("bne {}, {}, {}", format_reg(*rs1), format_reg(*rs2), offset)
        }
        Instruction::Blt { rs1, rs2, offset } => {
            format!("blt {}, {}, {}", format_reg(*rs1), format_reg(*rs2), offset)
        }
        Instruction::Bge { rs1, rs2, offset } => {
            format!("bge {}, {}, {}", format_reg(*rs1), format_reg(*rs2), offset)
        }
        Instruction::Bltu { rs1, rs2, offset } => {
            format!("bltu {}, {}, {}", format_reg(*rs1), format_reg(*rs2), offset)
        }
        Instruction::Bgeu { rs1, rs2, offset } => {
            format!("bgeu {}, {}, {}", format_reg(*rs1), format_reg(*rs2), offset)
        }

        // ========== Jump (J-type) ==========
        Instruction::Jal { rd, offset } => {
            format!("jal {}, {}", format_reg(*rd), offset)
        }
        Instruction::Jalr { rd, rs1, imm } => {
            format!("jalr {}, {}({})", format_reg(*rd), imm, format_reg(*rs1))
        }

        // Fallback for any instruction not explicitly handled
        _ => format!("??? (unknown instruction)"),
    }
}

/// Format register using its name (e.g., "a0" instead of "r10")
fn format_reg(reg: Register) -> String {
    reg.name().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_ecall() {
        assert_eq!(format(&Instruction::Ecall), "ecall");
    }

    #[test]
    fn test_format_ebreak() {
        assert_eq!(format(&Instruction::Ebreak), "ebreak");
    }

    #[test]
    fn test_format_add() {
        let instr = Instruction::Add {
            rd: Register::R4,   // a0
            rs1: Register::R5,  // a1
            rs2: Register::R6,  // a2
        };
        assert_eq!(format(&instr), "add a0, a1, a2");
    }

    #[test]
    fn test_format_addi() {
        let instr = Instruction::Addi {
            rd: Register::R4,   // a0
            rs1: Register::R5,  // a1
            imm: 100,
        };
        assert_eq!(format(&instr), "addi a0, a1, 100");
    }

    #[test]
    fn test_format_lw() {
        let instr = Instruction::Lw {
            rd: Register::R4,   // a0
            rs1: Register::R2,  // sp
            imm: 16,
        };
        assert_eq!(format(&instr), "lw a0, 16(sp)");
    }

    #[test]
    fn test_format_sw() {
        let instr = Instruction::Sw {
            rs1: Register::R2,  // sp
            rs2: Register::R4,  // a0
            imm: 16,
        };
        assert_eq!(format(&instr), "sw a0, 16(sp)");
    }

    #[test]
    fn test_format_beq() {
        let instr = Instruction::Beq {
            rs1: Register::R4,  // a0
            rs2: Register::R5,  // a1
            offset: 8,
        };
        assert_eq!(format(&instr), "beq a0, a1, 8");
    }

    #[test]
    fn test_format_jal() {
        let instr = Instruction::Jal {
            rd: Register::R1,   // ra
            offset: 100,
        };
        assert_eq!(format(&instr), "jal ra, 100");
    }

    #[test]
    fn test_format_cmov() {
        let instr = Instruction::Cmov {
            rd: Register::R4,   // a0
            rs1: Register::R5,  // a1
            rs2: Register::R6,  // a2
        };
        assert_eq!(format(&instr), "cmov a0, a1, a2");
    }

    #[test]
    fn test_format_slli() {
        let instr = Instruction::Slli {
            rd: Register::R4,   // a0
            rs1: Register::R5,  // a1
            shamt: 5,
        };
        assert_eq!(format(&instr), "slli a0, a1, 5");
    }

    #[test]
    fn test_format_negative_immediate() {
        let instr = Instruction::Addi {
            rd: Register::R4,
            rs1: Register::R5,
            imm: -1,
        };
        assert_eq!(format(&instr), "addi a0, a1, -1");
    }
}
