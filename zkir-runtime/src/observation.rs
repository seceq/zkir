//! Observation point detection for deferred carry model
//!
//! This module identifies instructions that require normalized inputs (observation points).
//! At these points, accumulated values must be normalized before execution.
//!
//! ## Observation Points
//!
//! Instructions fall into categories based on their normalization requirements:
//!
//! 1. **Branches**: Need exact value comparison
//! 2. **Comparisons**: Need exact value comparison for set-less-than
//! 3. **Memory Stores**: Values written to memory must be canonical
//! 4. **Bitwise Operations**: Need exact bit patterns
//! 5. **Shifts**: Need exact values and shift amounts
//! 6. **Multiplication**: Operands must be normalized for correct product
//! 7. **Division**: Operands must be normalized for correct quotient/remainder

use zkir_spec::Opcode;

/// Check if an opcode requires normalized inputs (is an observation point)
///
/// # Returns
/// `true` if the instruction requires normalized operands before execution
pub fn is_observation_point(opcode: Opcode) -> bool {
    matches!(
        opcode,
        // Branches - need exact comparison
        Opcode::Beq | Opcode::Bne | Opcode::Blt | Opcode::Bge | Opcode::Bltu | Opcode::Bgeu |

        // Comparisons - need exact comparison (Seq, Sne, Slt, Sltu, Sge, Sgeu)
        Opcode::Seq | Opcode::Sne | Opcode::Slt | Opcode::Sltu | Opcode::Sge | Opcode::Sgeu |

        // Memory stores - values must be canonical
        Opcode::Sw | Opcode::Sh | Opcode::Sb |

        // Bitwise operations - need exact bit patterns
        Opcode::And | Opcode::Or | Opcode::Xor |
        Opcode::Andi | Opcode::Ori | Opcode::Xori |

        // Shifts - need exact values
        Opcode::Sll | Opcode::Srl | Opcode::Sra |
        Opcode::Slli | Opcode::Srli | Opcode::Srai |

        // Multiplication - operands must be normalized
        Opcode::Mul | Opcode::Mulh |

        // Division - operands must be normalized
        Opcode::Div | Opcode::Divu | Opcode::Rem | Opcode::Remu
    )
}

/// Get the source registers that need normalization for an instruction
///
/// Returns register indices (not Register enum) of source operands that must
/// be normalized before instruction execution.
///
/// # Parameters
/// - `opcode`: The instruction opcode
/// - `rs1`: First source register index (0-15)
/// - `rs2`: Second source register index (0-15)
///
/// # Returns
/// Vector of register indices that need normalization
pub fn get_normalize_sources(opcode: Opcode, rs1: u8, rs2: u8) -> Vec<u8> {
    match opcode {
        // R-type instructions that use both operands
        Opcode::Beq | Opcode::Bne | Opcode::Blt | Opcode::Bge | Opcode::Bltu | Opcode::Bgeu |
        Opcode::Seq | Opcode::Sne | Opcode::Slt | Opcode::Sltu | Opcode::Sge | Opcode::Sgeu |
        Opcode::And | Opcode::Or | Opcode::Xor |
        Opcode::Sll | Opcode::Srl | Opcode::Sra |
        Opcode::Mul | Opcode::Mulh |
        Opcode::Div | Opcode::Divu | Opcode::Rem | Opcode::Remu => {
            vec![rs1, rs2]
        }

        // I-type operations (only rs1 needs normalization)
        Opcode::Andi | Opcode::Ori | Opcode::Xori |
        Opcode::Slli | Opcode::Srli | Opcode::Srai => {
            vec![rs1]
        }

        // Stores: normalize the value being stored (rs2)
        // Note: rs1 (base address) also needs normalization for address calculation
        Opcode::Sw | Opcode::Sh | Opcode::Sb => {
            vec![rs1, rs2]  // Both base and value
        }

        // Memory loads: normalize address base (rs1)
        Opcode::Lw | Opcode::Lh | Opcode::Lb | Opcode::Lhu | Opcode::Lbu => {
            vec![rs1]
        }

        // Arithmetic operations don't require normalization (deferred model)
        Opcode::Add | Opcode::Sub | Opcode::Addi => {
            vec![]
        }

        // Other instructions
        _ => vec![],
    }
}

/// Check if instruction can produce accumulated output
///
/// Returns `true` if the instruction's result can be stored in accumulated
/// (unnormalized) form for deferred carry extraction.
pub fn can_defer_output(opcode: Opcode) -> bool {
    matches!(
        opcode,
        Opcode::Add | Opcode::Sub | Opcode::Addi |
        Opcode::Mul  // MUL result can also be deferred
    )
}

/// Get category of an instruction for normalization purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionCategory {
    /// Deferred arithmetic (ADD, SUB, ADDI)
    DeferredArithmetic,
    /// Observation point requiring normalized inputs
    ObservationPoint,
    /// Other instruction (no special handling)
    Other,
}

/// Categorize an instruction for deferred carry model
pub fn categorize_instruction(opcode: Opcode) -> InstructionCategory {
    if matches!(opcode, Opcode::Add | Opcode::Sub | Opcode::Addi) {
        InstructionCategory::DeferredArithmetic
    } else if is_observation_point(opcode) {
        InstructionCategory::ObservationPoint
    } else {
        InstructionCategory::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branches_are_observation_points() {
        assert!(is_observation_point(Opcode::Beq));
        assert!(is_observation_point(Opcode::Bne));
        assert!(is_observation_point(Opcode::Blt));
        assert!(is_observation_point(Opcode::Bge));
        assert!(is_observation_point(Opcode::Bltu));
        assert!(is_observation_point(Opcode::Bgeu));
    }

    #[test]
    fn test_comparisons_are_observation_points() {
        assert!(is_observation_point(Opcode::Seq));
        assert!(is_observation_point(Opcode::Sne));
        assert!(is_observation_point(Opcode::Slt));
        assert!(is_observation_point(Opcode::Sltu));
        assert!(is_observation_point(Opcode::Sge));
        assert!(is_observation_point(Opcode::Sgeu));
    }

    #[test]
    fn test_stores_are_observation_points() {
        assert!(is_observation_point(Opcode::Sw));
        assert!(is_observation_point(Opcode::Sh));
        assert!(is_observation_point(Opcode::Sb));
    }

    #[test]
    fn test_bitwise_are_observation_points() {
        assert!(is_observation_point(Opcode::And));
        assert!(is_observation_point(Opcode::Or));
        assert!(is_observation_point(Opcode::Xor));
        assert!(is_observation_point(Opcode::Andi));
        assert!(is_observation_point(Opcode::Ori));
        assert!(is_observation_point(Opcode::Xori));
    }

    #[test]
    fn test_shifts_are_observation_points() {
        assert!(is_observation_point(Opcode::Sll));
        assert!(is_observation_point(Opcode::Srl));
        assert!(is_observation_point(Opcode::Sra));
        assert!(is_observation_point(Opcode::Slli));
        assert!(is_observation_point(Opcode::Srli));
        assert!(is_observation_point(Opcode::Srai));
    }

    #[test]
    fn test_mul_div_are_observation_points() {
        assert!(is_observation_point(Opcode::Mul));
        assert!(is_observation_point(Opcode::Mulh));
        assert!(is_observation_point(Opcode::Div));
        assert!(is_observation_point(Opcode::Divu));
        assert!(is_observation_point(Opcode::Rem));
        assert!(is_observation_point(Opcode::Remu));
    }

    #[test]
    fn test_arithmetic_not_observation_points() {
        assert!(!is_observation_point(Opcode::Add));
        assert!(!is_observation_point(Opcode::Sub));
        assert!(!is_observation_point(Opcode::Addi));
    }

    #[test]
    fn test_loads_normalize_address() {
        // Loads need address normalization, so we normalize rs1
        // They normalize address via get_normalize_sources even though
        // they're not marked as strict observation points in all contexts
        let sources = get_normalize_sources(Opcode::Lw, 1, 0);
        assert_eq!(sources, vec![1]);  // Address base needs normalization
    }

    #[test]
    fn test_normalize_sources_branches() {
        let sources = get_normalize_sources(Opcode::Beq, 1, 2);
        assert_eq!(sources, vec![1, 2]);
    }

    #[test]
    fn test_normalize_sources_stores() {
        let sources = get_normalize_sources(Opcode::Sw, 3, 4);
        assert_eq!(sources, vec![3, 4]);  // Both base and value
    }

    #[test]
    fn test_normalize_sources_immediate() {
        let sources = get_normalize_sources(Opcode::Andi, 5, 0);
        assert_eq!(sources, vec![5]);  // Only rs1
    }

    #[test]
    fn test_normalize_sources_arithmetic() {
        let sources = get_normalize_sources(Opcode::Add, 1, 2);
        assert!(sources.is_empty());  // No normalization needed

        let sources = get_normalize_sources(Opcode::Addi, 3, 0);
        assert!(sources.is_empty());
    }

    #[test]
    fn test_can_defer_output() {
        assert!(can_defer_output(Opcode::Add));
        assert!(can_defer_output(Opcode::Sub));
        assert!(can_defer_output(Opcode::Addi));
        assert!(can_defer_output(Opcode::Mul));

        assert!(!can_defer_output(Opcode::And));
        assert!(!can_defer_output(Opcode::Beq));
        assert!(!can_defer_output(Opcode::Sw));
    }

    #[test]
    fn test_categorize_instruction() {
        assert_eq!(
            categorize_instruction(Opcode::Add),
            InstructionCategory::DeferredArithmetic
        );
        assert_eq!(
            categorize_instruction(Opcode::Addi),
            InstructionCategory::DeferredArithmetic
        );
        assert_eq!(
            categorize_instruction(Opcode::Beq),
            InstructionCategory::ObservationPoint
        );
        assert_eq!(
            categorize_instruction(Opcode::And),
            InstructionCategory::ObservationPoint
        );
        assert_eq!(
            categorize_instruction(Opcode::Ebreak),
            InstructionCategory::Other
        );
    }
}
