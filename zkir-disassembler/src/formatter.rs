//! Instruction formatting to assembly text

use zkir_spec::{Instruction, Register};

/// Format instruction as assembly text
pub fn format(instr: &Instruction) -> String {
    match instr {
        // System
        Instruction::Ecall => "ecall".to_string(),
        Instruction::Ebreak => "ebreak".to_string(),

        // Arithmetic
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

        // Arithmetic Immediate
        Instruction::Addi { rd, rs1, imm } => {
            format!("addi {}, {}, {}", format_reg(*rd), format_reg(*rs1), imm)
        }

        // Load/Store
        Instruction::Lw { rd, rs1, imm } => {
            format!("lw {}, {}({})", format_reg(*rd), imm, format_reg(*rs1))
        }
        Instruction::Sw { rs1, rs2, imm } => {
            format!("sw {}, {}({})", format_reg(*rs2), imm, format_reg(*rs1))
        }

        // Branches
        Instruction::Beq { rs1, rs2, imm } => {
            format!("beq {}, {}, {}", format_reg(*rs1), format_reg(*rs2), imm)
        }
        Instruction::Bne { rs1, rs2, imm } => {
            format!("bne {}, {}, {}", format_reg(*rs1), format_reg(*rs2), imm)
        }

        // Jumps
        Instruction::Jal { rd, imm } => {
            format!("jal {}, {}", format_reg(*rd), imm)
        }
        Instruction::Jalr { rd, rs1, imm } => {
            format!("jalr {}, {}({})", format_reg(*rd), imm, format_reg(*rs1))
        }

        // Upper Immediate
        Instruction::Lui { rd, imm } => {
            format!("lui {}, {}", format_reg(*rd), imm)
        }
        Instruction::Auipc { rd, imm } => {
            format!("auipc {}, {}", format_reg(*rd), imm)
        }

        // ZK-Custom
        Instruction::AssertEq { rs1, rs2 } => {
            format!("assert_eq {}, {}", format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::AssertNe { rs1, rs2 } => {
            format!("assert_ne {}, {}", format_reg(*rs1), format_reg(*rs2))
        }
        Instruction::AssertZero { rs1 } => {
            format!("assert_zero {}", format_reg(*rs1))
        }
        Instruction::Commit { rs1 } => {
            format!("commit {}", format_reg(*rs1))
        }
        Instruction::Halt => "halt".to_string(),

        // ZK I/O
        Instruction::Read { rd } => {
            format!("read {}", format_reg(*rd))
        }
        Instruction::Write { rs1 } => {
            format!("write {}", format_reg(*rs1))
        }
        Instruction::Hint { rd } => {
            format!("hint {}", format_reg(*rd))
        }

        // Default for unimplemented
        _ => format!("??? (not yet formatted)"),
    }
}

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
    fn test_format_halt() {
        assert_eq!(format(&Instruction::Halt), "halt");
    }

    #[test]
    fn test_format_add() {
        let instr = Instruction::Add {
            rd: Register::R10,
            rs1: Register::R11,
            rs2: Register::R12,
        };
        assert_eq!(format(&instr), "add a0, a1, a2");
    }
}
