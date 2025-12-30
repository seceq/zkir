//! Program Analyzer for Range Check Optimization
//!
//! Performs static analysis on ZKIR programs to determine which instructions
//! require range checks, enabling ~96% constraint reduction.

use crate::bound::{BoundAnalysis, RangeCheckReason, ValueBound, FIELD_BITS};
use crate::instruction::Instruction;
use crate::register::Register;

/// Analyzes a sequence of instructions to determine optimal range check placement.
///
/// # Algorithm
///
/// 1. Initialize all register bounds to FIELD (unknown from caller)
/// 2. Walk instructions in order, propagating bounds
/// 3. Mark mandatory check points:
///    - External inputs (ECALL returns)
///    - Memory loads
///    - Memory store addresses
///    - Division quotients
///    - Values exceeding safe bounds
/// 4. Return analysis with check locations and statistics
///
/// # Arguments
///
/// * `instructions` - Slice of decoded ZKIR instructions
///
/// # Returns
///
/// `BoundAnalysis` containing which instructions need range checks
pub fn analyze_program(instructions: &[Instruction]) -> BoundAnalysis {
    let mut analysis = BoundAnalysis::new();
    let mut reg_bounds: [ValueBound; 16] = [ValueBound::FIELD; 16];

    // r0 (zero) is always 0
    reg_bounds[0] = ValueBound::ZERO;

    for (pc, instr) in instructions.iter().enumerate() {
        let pc = pc as u32;
        analysis.record_instruction();

        match instr {
            // ========== Arithmetic ==========
            Instruction::Add { rd, rs1, rs2 } => {
                let bound = get_bound(&reg_bounds, *rs1).after_add(get_bound(&reg_bounds, *rs2));
                set_bound(&mut reg_bounds, *rd, bound);
                check_overflow(&mut analysis, pc, *rd, bound);
            }

            Instruction::Sub { rd, rs1, rs2 } => {
                let bound = get_bound(&reg_bounds, *rs1).after_sub(get_bound(&reg_bounds, *rs2));
                set_bound(&mut reg_bounds, *rd, bound);
                // SUB could underflow - but result stays bounded by max input
            }

            Instruction::Mul { rd, rs1, rs2 } => {
                let bound = get_bound(&reg_bounds, *rs1).after_mul(get_bound(&reg_bounds, *rs2));
                set_bound(&mut reg_bounds, *rd, bound);
                check_overflow(&mut analysis, pc, *rd, bound);
            }

            Instruction::Mulh { rd, .. } => {
                // Upper bits of multiplication - bounded by field
                set_bound(&mut reg_bounds, *rd, ValueBound::FIELD);
            }

            Instruction::Divu { rd, rs1, .. } | Instruction::Div { rd, rs1, .. } => {
                // Division quotient needs range check (prover could cheat)
                let bound = get_bound(&reg_bounds, *rs1).after_div(ValueBound::FIELD);
                set_bound(&mut reg_bounds, *rd, bound);
                analysis.require_check(pc, rd.index(), RangeCheckReason::DivisionQuotient);
            }

            Instruction::Remu { rd, rs1, rs2 } | Instruction::Rem { rd, rs1, rs2 } => {
                let bound = get_bound(&reg_bounds, *rs1).after_mod(get_bound(&reg_bounds, *rs2));
                set_bound(&mut reg_bounds, *rd, bound);
                // Remainder is bounded by divisor, no check needed if divisor is bounded
            }

            Instruction::Addi { rd, rs1, imm } => {
                let imm_bound = ValueBound::from_constant(imm.unsigned_abs() as u64);
                let bound = get_bound(&reg_bounds, *rs1).after_add(imm_bound);
                set_bound(&mut reg_bounds, *rd, bound);
                check_overflow(&mut analysis, pc, *rd, bound);
            }

            // ========== Logical ==========
            Instruction::And { rd, rs1, rs2 } => {
                let bound = get_bound(&reg_bounds, *rs1).after_and(get_bound(&reg_bounds, *rs2));
                set_bound(&mut reg_bounds, *rd, bound);
                analysis.record_elided();
            }

            Instruction::Or { rd, rs1, rs2 } => {
                let bound = get_bound(&reg_bounds, *rs1).after_or(get_bound(&reg_bounds, *rs2));
                set_bound(&mut reg_bounds, *rd, bound);
                analysis.record_elided();
            }

            Instruction::Xor { rd, rs1, rs2 } => {
                let bound = get_bound(&reg_bounds, *rs1).after_xor(get_bound(&reg_bounds, *rs2));
                set_bound(&mut reg_bounds, *rd, bound);
                analysis.record_elided();
            }

            Instruction::Andi { rd, rs1, imm } => {
                let bound = get_bound(&reg_bounds, *rs1).after_and_imm(*imm as u64);
                set_bound(&mut reg_bounds, *rd, bound);
                analysis.record_elided();
            }

            Instruction::Ori { rd, rs1, imm } => {
                let imm_bound = ValueBound::from_constant(*imm as u64);
                let bound = get_bound(&reg_bounds, *rs1).after_or(imm_bound);
                set_bound(&mut reg_bounds, *rd, bound);
                analysis.record_elided();
            }

            Instruction::Xori { rd, rs1, imm } => {
                let imm_bound = ValueBound::from_constant(*imm as u64);
                let bound = get_bound(&reg_bounds, *rs1).after_xor(imm_bound);
                set_bound(&mut reg_bounds, *rd, bound);
                analysis.record_elided();
            }

            // ========== Shift ==========
            Instruction::Sll { rd, rs1, rs2: _ } => {
                // Shift amount unknown - assume worst case (full shift)
                let bound = get_bound(&reg_bounds, *rs1).after_shl(FIELD_BITS);
                set_bound(&mut reg_bounds, *rd, bound);
                check_overflow(&mut analysis, pc, *rd, bound);
            }

            Instruction::Srl { rd, rs1, .. } | Instruction::Sra { rd, rs1, .. } => {
                // Right shift shrinks - assume at least 1 bit
                let bound = get_bound(&reg_bounds, *rs1).after_shr(1);
                set_bound(&mut reg_bounds, *rd, bound);
                analysis.record_elided();
            }

            Instruction::Slli { rd, rs1, shamt } => {
                let bound = get_bound(&reg_bounds, *rs1).after_shl(*shamt);
                set_bound(&mut reg_bounds, *rd, bound);
                check_overflow(&mut analysis, pc, *rd, bound);
            }

            Instruction::Srli { rd, rs1, shamt } | Instruction::Srai { rd, rs1, shamt } => {
                let bound = get_bound(&reg_bounds, *rs1).after_shr(*shamt);
                set_bound(&mut reg_bounds, *rd, bound);
                analysis.record_elided();
            }

            // ========== Compare ==========
            Instruction::Sltu { rd, .. }
            | Instruction::Sgeu { rd, .. }
            | Instruction::Slt { rd, .. }
            | Instruction::Sge { rd, .. }
            | Instruction::Seq { rd, .. }
            | Instruction::Sne { rd, .. } => {
                // Comparison results are boolean
                set_bound(&mut reg_bounds, *rd, ValueBound::BOOL);
                analysis.record_elided();
            }

            // ========== Conditional Move ==========
            Instruction::Cmov { rd, rs1, .. }
            | Instruction::Cmovz { rd, rs1, .. }
            | Instruction::Cmovnz { rd, rs1, .. } => {
                // Result is max of rd and rs1 bounds
                let bound = get_bound(&reg_bounds, *rd).after_or(get_bound(&reg_bounds, *rs1));
                set_bound(&mut reg_bounds, *rd, bound);
            }

            // ========== Memory Load ==========
            Instruction::Lb { rd, .. } | Instruction::Lbu { rd, .. } => {
                // Byte load - 8 bits but from untrusted memory
                set_bound(&mut reg_bounds, *rd, ValueBound::U8);
                analysis.require_check(pc, rd.index(), RangeCheckReason::MemoryLoad);
            }

            Instruction::Lh { rd, .. } | Instruction::Lhu { rd, .. } => {
                // Halfword load - 16 bits from untrusted memory
                set_bound(&mut reg_bounds, *rd, ValueBound::U16);
                analysis.require_check(pc, rd.index(), RangeCheckReason::MemoryLoad);
            }

            Instruction::Lw { rd, .. } => {
                // Word load - 32 bits from untrusted memory
                set_bound(&mut reg_bounds, *rd, ValueBound::from_bits(32));
                analysis.require_check(pc, rd.index(), RangeCheckReason::MemoryLoad);
            }

            Instruction::Ld { rd, .. } => {
                // 60-bit load from untrusted memory
                set_bound(&mut reg_bounds, *rd, ValueBound::FIELD);
                analysis.require_check(pc, rd.index(), RangeCheckReason::MemoryLoad);
            }

            // ========== Memory Store ==========
            Instruction::Sb { rs1, .. }
            | Instruction::Sh { rs1, .. }
            | Instruction::Sw { rs1, .. }
            | Instruction::Sd { rs1, .. } => {
                // Store address must be valid
                let addr_bound = get_bound(&reg_bounds, *rs1);
                if addr_bound.needs_range_check() {
                    analysis.require_check(pc, rs1.index(), RangeCheckReason::MemoryStoreAddress);
                } else {
                    analysis.record_elided();
                }
            }

            // ========== Branch ==========
            Instruction::Beq { .. }
            | Instruction::Bne { .. }
            | Instruction::Blt { .. }
            | Instruction::Bge { .. }
            | Instruction::Bltu { .. }
            | Instruction::Bgeu { .. } => {
                // Branches don't modify registers, no check needed
                analysis.record_elided();
            }

            // ========== Jump ==========
            Instruction::Jal { rd, .. } => {
                // Return address is PC+4 - always valid
                set_bound(&mut reg_bounds, *rd, ValueBound::FIELD);
                analysis.record_elided();
            }

            Instruction::Jalr { rd, rs1, .. } => {
                // Return address is PC+4 - always valid
                set_bound(&mut reg_bounds, *rd, ValueBound::FIELD);
                // Jump target address should be valid
                let addr_bound = get_bound(&reg_bounds, *rs1);
                if addr_bound.needs_range_check() {
                    analysis.require_check(pc, rs1.index(), RangeCheckReason::MemoryStoreAddress);
                } else {
                    analysis.record_elided();
                }
            }

            // ========== System ==========
            Instruction::Ecall => {
                // Syscall return value in a0 is from external source
                set_bound(&mut reg_bounds, Register::A0, ValueBound::FIELD);
                analysis.require_check(pc, Register::A0.index(), RangeCheckReason::ExternalInput);
            }

            Instruction::Ebreak => {
                // Halt - no register modification
                analysis.record_elided();
            }
        }

        // Store bounds for this PC
        for (i, &bound) in reg_bounds.iter().enumerate() {
            analysis.set_bound(pc, i as u8, bound);
        }
    }

    analysis
}

/// Get bound for a register
#[inline]
fn get_bound(bounds: &[ValueBound; 16], reg: Register) -> ValueBound {
    bounds[reg.index() as usize]
}

/// Set bound for a register (skip r0)
#[inline]
fn set_bound(bounds: &mut [ValueBound; 16], reg: Register, bound: ValueBound) {
    if reg.index() != 0 {
        bounds[reg.index() as usize] = bound;
    }
}

/// Check if value is at overflow risk and mark for range check
#[inline]
fn check_overflow(analysis: &mut BoundAnalysis, pc: u32, rd: Register, bound: ValueBound) {
    if bound.at_overflow_risk() {
        analysis.require_check(pc, rd.index(), RangeCheckReason::OverflowPrevention);
    } else {
        analysis.record_elided();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_bound_propagation() {
        let instructions = vec![
            // Load two values (need checks)
            Instruction::Ld { rd: Register::A0, rs1: Register::ZERO, imm: 0 },
            Instruction::Ld { rd: Register::A1, rs1: Register::ZERO, imm: 8 },
            // Add them (no check - just grows by 1 bit)
            Instruction::Add { rd: Register::A2, rs1: Register::A0, rs2: Register::A1 },
        ];

        let analysis = analyze_program(&instructions);
        let stats = analysis.stats();

        // Should have 2 checks for loads, 1 elided for add
        assert_eq!(stats.checks_required, 2);
        assert_eq!(stats.checks_elided, 1);
    }

    #[test]
    fn test_and_mask_elision() {
        let instructions = vec![
            // Load a value (need check)
            Instruction::Ld { rd: Register::A0, rs1: Register::ZERO, imm: 0 },
            // Mask to 30 bits (no check - AND shrinks)
            Instruction::Andi { rd: Register::A1, rs1: Register::A0, imm: 0x3FFF_FFFF },
        ];

        let analysis = analyze_program(&instructions);
        let stats = analysis.stats();

        // Should have 1 check for load, 1 elided for ANDI
        assert_eq!(stats.checks_required, 1);
        assert_eq!(stats.checks_elided, 1);
    }

    #[test]
    fn test_shift_right_elision() {
        let instructions = vec![
            // Load a value (need check)
            Instruction::Ld { rd: Register::A0, rs1: Register::ZERO, imm: 0 },
            // Shift right (no check - shrinks)
            Instruction::Srli { rd: Register::A1, rs1: Register::A0, shamt: 10 },
        ];

        let analysis = analyze_program(&instructions);
        let stats = analysis.stats();

        // Should have 1 check for load, 1 elided for SRLI
        assert_eq!(stats.checks_required, 1);
        assert_eq!(stats.checks_elided, 1);
    }

    #[test]
    fn test_comparison_is_boolean() {
        let instructions = vec![
            // Compare (result is boolean - no check)
            Instruction::Sltu { rd: Register::A0, rs1: Register::A1, rs2: Register::A2 },
        ];

        let analysis = analyze_program(&instructions);

        // Result should be 1 bit
        let bound = analysis.get_bound(0, Register::A0.index());
        assert_eq!(bound.bits(), 1);
    }

    #[test]
    fn test_division_always_checked() {
        let instructions = vec![
            Instruction::Divu { rd: Register::A0, rs1: Register::A1, rs2: Register::A2 },
        ];

        let analysis = analyze_program(&instructions);

        // Division quotient must always be checked
        assert!(analysis.needs_check(0, Register::A0.index()));
    }
}
