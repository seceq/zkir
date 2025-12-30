//! Instruction validation for ZKIR v3.4
//!
//! Validates decoded instructions for semantic correctness before execution.
//! This catches issues that the decoder may not detect, such as:
//! - Immediate values out of range
//! - Invalid shift amounts
//! - Misaligned branch offsets
//! - Writes to R0 (which is ignored)

use crate::{Instruction, Register};
use thiserror::Error;

/// Validation error types
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("Immediate value {value} out of range [{min}, {max}] for {instruction}")]
    ImmediateOutOfRange {
        instruction: &'static str,
        value: i32,
        min: i32,
        max: i32,
    },

    #[error("Shift amount {shamt} exceeds maximum {max} for {instruction}")]
    ShiftAmountOutOfRange {
        instruction: &'static str,
        shamt: u8,
        max: u8,
    },

    #[error("Branch offset {offset} is not aligned to {alignment} bytes")]
    MisalignedBranchOffset { offset: i32, alignment: u32 },

    #[error("Jump offset {offset} is not aligned to {alignment} bytes")]
    MisalignedJumpOffset { offset: i32, alignment: u32 },
}

/// Validation warning types (not errors, but worth noting)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationWarning {
    /// Writing to R0 has no effect (R0 is hardwired to zero)
    WriteToR0 { instruction: &'static str },

    /// Unconditional branch (comparing register with itself)
    UnconditionalBranch { instruction: &'static str },

    /// No-op instruction (e.g., ADD r0, r0, r0)
    NoOp { instruction: &'static str },
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Create an empty validation result
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Maximum immediate value for I-type instructions (17-bit signed)
const I_TYPE_IMM_MAX: i32 = (1 << 16) - 1;
const I_TYPE_IMM_MIN: i32 = -(1 << 16);

/// Maximum offset for B-type instructions (17-bit signed)
const B_TYPE_OFFSET_MAX: i32 = (1 << 16) - 1;
const B_TYPE_OFFSET_MIN: i32 = -(1 << 16);

/// Maximum offset for J-type instructions (21-bit signed)
const J_TYPE_OFFSET_MAX: i32 = (1 << 20) - 1;
const J_TYPE_OFFSET_MIN: i32 = -(1 << 20);

/// Maximum shift amount for 40-bit values
const MAX_SHIFT_AMOUNT: u8 = 63; // Allow up to 63 for flexibility

/// Validate a single instruction
pub fn validate(inst: &Instruction) -> ValidationResult {
    let mut result = ValidationResult::new();

    match inst {
        // Arithmetic R-type
        Instruction::Add { rd, rs1, rs2 }
        | Instruction::Sub { rd, rs1, rs2 }
        | Instruction::Mul { rd, rs1, rs2 }
        | Instruction::Mulh { rd, rs1, rs2 }
        | Instruction::Div { rd, rs1, rs2 }
        | Instruction::Divu { rd, rs1, rs2 }
        | Instruction::Rem { rd, rs1, rs2 }
        | Instruction::Remu { rd, rs1, rs2 } => {
            check_write_to_r0(&mut result, *rd, inst.mnemonic());
            check_noop_r_type(&mut result, *rd, *rs1, *rs2, inst.mnemonic());
        }

        // Immediate arithmetic
        Instruction::Addi { rd, imm, .. } => {
            check_write_to_r0(&mut result, *rd, inst.mnemonic());
            check_immediate_range(&mut result, *imm, I_TYPE_IMM_MIN, I_TYPE_IMM_MAX, inst.mnemonic());
        }

        // Logical R-type
        Instruction::And { rd, rs1, rs2 }
        | Instruction::Or { rd, rs1, rs2 }
        | Instruction::Xor { rd, rs1, rs2 } => {
            check_write_to_r0(&mut result, *rd, inst.mnemonic());
            check_noop_r_type(&mut result, *rd, *rs1, *rs2, inst.mnemonic());
        }

        // Immediate logical
        Instruction::Andi { rd, imm, .. }
        | Instruction::Ori { rd, imm, .. }
        | Instruction::Xori { rd, imm, .. } => {
            check_write_to_r0(&mut result, *rd, inst.mnemonic());
            check_immediate_range(&mut result, *imm, I_TYPE_IMM_MIN, I_TYPE_IMM_MAX, inst.mnemonic());
        }

        // Shift R-type
        Instruction::Sll { rd, .. }
        | Instruction::Srl { rd, .. }
        | Instruction::Sra { rd, .. } => {
            check_write_to_r0(&mut result, *rd, inst.mnemonic());
        }

        // Shift immediate
        Instruction::Slli { rd, shamt, .. }
        | Instruction::Srli { rd, shamt, .. }
        | Instruction::Srai { rd, shamt, .. } => {
            check_write_to_r0(&mut result, *rd, inst.mnemonic());
            check_shift_amount(&mut result, *shamt, MAX_SHIFT_AMOUNT, inst.mnemonic());
        }

        // Compare R-type
        Instruction::Slt { rd, .. }
        | Instruction::Sltu { rd, .. }
        | Instruction::Sge { rd, .. }
        | Instruction::Sgeu { rd, .. }
        | Instruction::Seq { rd, .. }
        | Instruction::Sne { rd, .. } => {
            check_write_to_r0(&mut result, *rd, inst.mnemonic());
        }

        // Conditional move
        Instruction::Cmov { rd, .. }
        | Instruction::Cmovz { rd, .. }
        | Instruction::Cmovnz { rd, .. } => {
            check_write_to_r0(&mut result, *rd, inst.mnemonic());
        }

        // Load instructions
        Instruction::Lb { rd, imm, .. }
        | Instruction::Lbu { rd, imm, .. }
        | Instruction::Lh { rd, imm, .. }
        | Instruction::Lhu { rd, imm, .. }
        | Instruction::Lw { rd, imm, .. }
        | Instruction::Ld { rd, imm, .. } => {
            check_write_to_r0(&mut result, *rd, inst.mnemonic());
            check_immediate_range(&mut result, *imm, I_TYPE_IMM_MIN, I_TYPE_IMM_MAX, inst.mnemonic());
        }

        // Store instructions
        Instruction::Sb { imm, .. }
        | Instruction::Sh { imm, .. }
        | Instruction::Sw { imm, .. }
        | Instruction::Sd { imm, .. } => {
            check_immediate_range(&mut result, *imm, I_TYPE_IMM_MIN, I_TYPE_IMM_MAX, inst.mnemonic());
        }

        // Branch instructions
        Instruction::Beq { rs1, rs2, offset }
        | Instruction::Bne { rs1, rs2, offset }
        | Instruction::Blt { rs1, rs2, offset }
        | Instruction::Bge { rs1, rs2, offset }
        | Instruction::Bltu { rs1, rs2, offset }
        | Instruction::Bgeu { rs1, rs2, offset } => {
            check_branch_offset_range(&mut result, *offset, B_TYPE_OFFSET_MIN, B_TYPE_OFFSET_MAX);
            // Check for unconditional branch (comparing register with itself)
            if rs1 == rs2 {
                match inst {
                    Instruction::Beq { .. } | Instruction::Bge { .. } | Instruction::Bgeu { .. } => {
                        // rs1 == rs1 is always true
                        result.add_warning(ValidationWarning::UnconditionalBranch {
                            instruction: inst.mnemonic(),
                        });
                    }
                    Instruction::Bne { .. } | Instruction::Blt { .. } | Instruction::Bltu { .. } => {
                        // rs1 != rs1 is always false (noop)
                        result.add_warning(ValidationWarning::NoOp {
                            instruction: inst.mnemonic(),
                        });
                    }
                    _ => {}
                }
            }
        }

        // Jump instructions
        Instruction::Jal { rd, offset } => {
            check_write_to_r0(&mut result, *rd, inst.mnemonic());
            check_jump_offset_range(&mut result, *offset, J_TYPE_OFFSET_MIN, J_TYPE_OFFSET_MAX);
        }

        Instruction::Jalr { rd, imm, .. } => {
            check_write_to_r0(&mut result, *rd, inst.mnemonic());
            check_immediate_range(&mut result, *imm, I_TYPE_IMM_MIN, I_TYPE_IMM_MAX, inst.mnemonic());
        }

        // System instructions - no validation needed
        Instruction::Ecall | Instruction::Ebreak => {}
    }

    result
}

/// Validate a sequence of instructions
pub fn validate_program(instructions: &[Instruction]) -> Vec<(usize, ValidationResult)> {
    instructions
        .iter()
        .enumerate()
        .map(|(i, inst)| (i, validate(inst)))
        .filter(|(_, result)| !result.errors.is_empty() || !result.warnings.is_empty())
        .collect()
}

// Helper functions

fn check_write_to_r0(result: &mut ValidationResult, rd: Register, instruction: &'static str) {
    if rd == Register::R0 {
        result.add_warning(ValidationWarning::WriteToR0 { instruction });
    }
}

fn check_immediate_range(
    result: &mut ValidationResult,
    value: i32,
    min: i32,
    max: i32,
    instruction: &'static str,
) {
    if value < min || value > max {
        result.add_error(ValidationError::ImmediateOutOfRange {
            instruction,
            value,
            min,
            max,
        });
    }
}

fn check_shift_amount(
    result: &mut ValidationResult,
    shamt: u8,
    max: u8,
    instruction: &'static str,
) {
    if shamt > max {
        result.add_error(ValidationError::ShiftAmountOutOfRange {
            instruction,
            shamt,
            max,
        });
    }
}

fn check_branch_offset_range(result: &mut ValidationResult, offset: i32, min: i32, max: i32) {
    if offset < min || offset > max {
        result.add_error(ValidationError::ImmediateOutOfRange {
            instruction: "branch",
            value: offset,
            min,
            max,
        });
    }
    // Branch offsets should be aligned to 4 bytes (instruction size)
    if offset % 4 != 0 {
        result.add_error(ValidationError::MisalignedBranchOffset {
            offset,
            alignment: 4,
        });
    }
}

fn check_jump_offset_range(result: &mut ValidationResult, offset: i32, min: i32, max: i32) {
    if offset < min || offset > max {
        result.add_error(ValidationError::ImmediateOutOfRange {
            instruction: "jal",
            value: offset,
            min,
            max,
        });
    }
    // Jump offsets should be aligned to 4 bytes (instruction size)
    if offset % 4 != 0 {
        result.add_error(ValidationError::MisalignedJumpOffset {
            offset,
            alignment: 4,
        });
    }
}

fn check_noop_r_type(
    result: &mut ValidationResult,
    rd: Register,
    rs1: Register,
    rs2: Register,
    instruction: &'static str,
) {
    // ADD r0, r0, r0 or similar is a no-op
    if rd == Register::R0 && rs1 == Register::R0 && rs2 == Register::R0 {
        result.add_warning(ValidationWarning::NoOp { instruction });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_add() {
        let inst = Instruction::Add {
            rd: Register::R1,
            rs1: Register::R2,
            rs2: Register::R3,
        };
        let result = validate(&inst);
        assert!(result.is_valid());
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_write_to_r0_warning() {
        let inst = Instruction::Add {
            rd: Register::R0,
            rs1: Register::R1,
            rs2: Register::R2,
        };
        let result = validate(&inst);
        assert!(result.is_valid()); // Still valid, just a warning
        assert!(result.has_warnings());
        assert!(matches!(
            result.warnings[0],
            ValidationWarning::WriteToR0 { .. }
        ));
    }

    #[test]
    fn test_noop_warning() {
        let inst = Instruction::Add {
            rd: Register::R0,
            rs1: Register::R0,
            rs2: Register::R0,
        };
        let result = validate(&inst);
        assert!(result.is_valid());
        assert!(result.has_warnings());
        // Should have both WriteToR0 and NoOp warnings
        assert!(result.warnings.len() >= 1);
    }

    #[test]
    fn test_shift_amount_valid() {
        let inst = Instruction::Slli {
            rd: Register::R1,
            rs1: Register::R2,
            shamt: 40,
        };
        let result = validate(&inst);
        assert!(result.is_valid());
    }

    #[test]
    fn test_shift_amount_out_of_range() {
        let inst = Instruction::Slli {
            rd: Register::R1,
            rs1: Register::R2,
            shamt: 100, // Too large
        };
        let result = validate(&inst);
        assert!(!result.is_valid());
        assert!(matches!(
            result.errors[0],
            ValidationError::ShiftAmountOutOfRange { .. }
        ));
    }

    #[test]
    fn test_branch_aligned() {
        let inst = Instruction::Beq {
            rs1: Register::R1,
            rs2: Register::R2,
            offset: 8,
        };
        let result = validate(&inst);
        assert!(result.is_valid());
    }

    #[test]
    fn test_branch_misaligned() {
        let inst = Instruction::Beq {
            rs1: Register::R1,
            rs2: Register::R2,
            offset: 7, // Not aligned to 4
        };
        let result = validate(&inst);
        assert!(!result.is_valid());
        assert!(matches!(
            result.errors[0],
            ValidationError::MisalignedBranchOffset { .. }
        ));
    }

    #[test]
    fn test_unconditional_branch_warning() {
        let inst = Instruction::Beq {
            rs1: Register::R1,
            rs2: Register::R1, // Same register
            offset: 8,
        };
        let result = validate(&inst);
        assert!(result.is_valid());
        assert!(result.has_warnings());
        assert!(matches!(
            result.warnings[0],
            ValidationWarning::UnconditionalBranch { .. }
        ));
    }

    #[test]
    fn test_never_taken_branch_warning() {
        let inst = Instruction::Bne {
            rs1: Register::R1,
            rs2: Register::R1, // Same register, never not equal
            offset: 8,
        };
        let result = validate(&inst);
        assert!(result.is_valid());
        assert!(result.has_warnings());
        assert!(matches!(
            result.warnings[0],
            ValidationWarning::NoOp { .. }
        ));
    }

    #[test]
    fn test_jal_valid() {
        let inst = Instruction::Jal {
            rd: Register::R1,
            offset: 100,
        };
        let result = validate(&inst);
        assert!(result.is_valid());
    }

    #[test]
    fn test_jal_misaligned() {
        let inst = Instruction::Jal {
            rd: Register::R1,
            offset: 101, // Not aligned
        };
        let result = validate(&inst);
        assert!(!result.is_valid());
        assert!(matches!(
            result.errors[0],
            ValidationError::MisalignedJumpOffset { .. }
        ));
    }

    #[test]
    fn test_system_instructions() {
        let ecall = Instruction::Ecall;
        let result = validate(&ecall);
        assert!(result.is_valid());
        assert!(!result.has_warnings());

        let ebreak = Instruction::Ebreak;
        let result = validate(&ebreak);
        assert!(result.is_valid());
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_validate_program() {
        let program = vec![
            Instruction::Add {
                rd: Register::R1,
                rs1: Register::R2,
                rs2: Register::R3,
            },
            Instruction::Add {
                rd: Register::R0, // Warning: write to R0
                rs1: Register::R1,
                rs2: Register::R2,
            },
            Instruction::Beq {
                rs1: Register::R1,
                rs2: Register::R2,
                offset: 7, // Error: misaligned
            },
        ];

        let results = validate_program(&program);
        assert_eq!(results.len(), 2); // Two instructions have issues

        // Instruction 1 has a warning
        assert_eq!(results[0].0, 1);
        assert!(results[0].1.is_valid());
        assert!(results[0].1.has_warnings());

        // Instruction 2 has an error
        assert_eq!(results[1].0, 2);
        assert!(!results[1].1.is_valid());
    }
}
