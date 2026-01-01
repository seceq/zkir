//! Instruction execution for ZKIR v3.4
//!
//! Executes all 47 instructions with proper field arithmetic using Value40.
//!
//! # Bound Propagation
//!
//! For range check optimization, each instruction should propagate value bounds:
//! 1. Read bounds from source registers
//! 2. Compute resulting bound using ValueBound propagation methods
//! 3. Write result bound to destination register
//!
//! Currently, bound propagation is partially implemented for demonstration.
//! Full integration requires updating all 47 instructions.

use crate::error::{RuntimeError, Result};
use crate::memory::Memory;
use crate::range_check::RangeCheckTracker;
use crate::state::{VMState, HaltReason};
use crate::deferred::{DeferredConfig, execute_add_deferred, execute_sub_deferred, execute_addi_deferred};
use crate::normalization_witness::NormalizationEvent;
use zkir_spec::{Instruction, Value, Value40, ValueBound, Register, Opcode};

/// Execute a single instruction
///
/// Updates the VM state and memory according to the instruction semantics.
/// Optionally defers range checks when bounds exceed program width.
///
/// # Parameters
/// - `inst`: Instruction to execute
/// - `state`: VM state (registers, PC, bounds)
/// - `memory`: Memory subsystem
/// - `range_checker`: Optional range check tracker for deferred checking
///
/// Returns Ok(()) for normal execution, Err for runtime errors.
pub fn execute(
    inst: &Instruction,
    state: &mut VMState,
    memory: &mut Memory,
    range_checker: Option<&mut RangeCheckTracker>,
) -> Result<()> {
    match inst {
        // ===== Arithmetic Operations =====
        Instruction::Add { rd, rs1, rs2 } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(state.read_reg(*rs2));
            let result = a.wrapping_add(b);

            // Propagate bounds: result bound = max(a, b) + 1 bit
            let bound_a = state.read_bound(*rs1);
            let bound_b = state.read_bound(*rs2);
            let result_bound = ValueBound::after_add(&bound_a, &bound_b);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);

            // Defer range check if needed
            if let Some(checker) = range_checker {
                if checker.needs_check(&result_bound) {
                    checker.defer(result, result_bound, state.pc);
                }
            }

            state.advance_pc(4);
        }

        Instruction::Sub { rd, rs1, rs2 } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(state.read_reg(*rs2));
            let result = a.wrapping_sub(b);

            // Propagate bounds: result bound = max(a.bits, b.bits)
            let bound_a = state.read_bound(*rs1);
            let bound_b = state.read_bound(*rs2);
            let result_bound = ValueBound::after_sub(&bound_a, &bound_b);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);
            state.advance_pc(4);
        }

        Instruction::Mul { rd, rs1, rs2 } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(state.read_reg(*rs2));
            let result = a.wrapping_mul(b);

            // Propagate bounds: result bound = a.bits + b.bits
            let bound_a = state.read_bound(*rs1);
            let bound_b = state.read_bound(*rs2);
            let result_bound = ValueBound::after_mul(&bound_a, &bound_b);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);

            // Defer range check if needed
            if let Some(checker) = range_checker {
                if checker.needs_check(&result_bound) {
                    checker.defer(result, result_bound, state.pc);
                }
            }

            state.advance_pc(4);
        }

        Instruction::Mulh { rd, rs1, rs2 } => {
            let a = state.read_reg(*rs1);
            let b = state.read_reg(*rs2);
            // High 40 bits of 80-bit product
            let product = (a as u128) * (b as u128);
            let high = ((product >> 40) & 0xFF_FFFF_FFFF) as u64;

            // Propagate bounds: high bits have tighter bound (same as mul result)
            let bound_a = state.read_bound(*rs1);
            let bound_b = state.read_bound(*rs2);
            let result_bound = ValueBound::after_mul(&bound_a, &bound_b);

            state.write_reg_with_bound(*rd, high, result_bound);
            state.advance_pc(4);
        }

        Instruction::Div { rd, rs1, rs2 } => {
            let dividend = state.read_reg(*rs1) as i64;
            let divisor = state.read_reg(*rs2) as i64;
            if divisor == 0 {
                return Err(RuntimeError::DivisionByZero { pc: state.pc });
            }
            let quotient = dividend.wrapping_div(divisor) as u64;

            // Propagate bounds: quotient bound ≤ dividend bound
            let bound_a = state.read_bound(*rs1);
            let bound_b = state.read_bound(*rs2);
            let result_bound = ValueBound::after_div(&bound_a, &bound_b);

            state.write_reg_with_bound(*rd, quotient, result_bound);
            state.advance_pc(4);
        }

        Instruction::Divu { rd, rs1, rs2 } => {
            let dividend = state.read_reg(*rs1);
            let divisor = state.read_reg(*rs2);
            if divisor == 0 {
                return Err(RuntimeError::DivisionByZero { pc: state.pc });
            }
            let quotient = dividend / divisor;

            // Propagate bounds: quotient bound ≤ dividend bound
            let bound_a = state.read_bound(*rs1);
            let bound_b = state.read_bound(*rs2);
            let result_bound = ValueBound::after_div(&bound_a, &bound_b);

            state.write_reg_with_bound(*rd, quotient, result_bound);
            state.advance_pc(4);
        }

        Instruction::Rem { rd, rs1, rs2 } => {
            let dividend = state.read_reg(*rs1) as i64;
            let divisor = state.read_reg(*rs2) as i64;
            if divisor == 0 {
                return Err(RuntimeError::DivisionByZero { pc: state.pc });
            }
            let remainder = dividend.wrapping_rem(divisor) as u64;

            // Propagate bounds: remainder bound < divisor bound
            let bound_a = state.read_bound(*rs1);
            let bound_b = state.read_bound(*rs2);
            let result_bound = ValueBound::after_div(&bound_a, &bound_b);

            state.write_reg_with_bound(*rd, remainder, result_bound);
            state.advance_pc(4);
        }

        Instruction::Remu { rd, rs1, rs2 } => {
            let dividend = state.read_reg(*rs1);
            let divisor = state.read_reg(*rs2);
            if divisor == 0 {
                return Err(RuntimeError::DivisionByZero { pc: state.pc });
            }
            let remainder = dividend % divisor;

            // Propagate bounds: remainder bound < divisor bound
            let bound_a = state.read_bound(*rs1);
            let bound_b = state.read_bound(*rs2);
            let result_bound = ValueBound::after_div(&bound_a, &bound_b);

            state.write_reg_with_bound(*rd, remainder, result_bound);
            state.advance_pc(4);
        }

        Instruction::Addi { rd, rs1, imm } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(*imm as u64);
            let result = a.wrapping_add(b);

            // Propagate bounds: immediate has known constant bound
            let bound_a = state.read_bound(*rs1);
            let bound_imm = ValueBound::from_constant(*imm as u64);
            let result_bound = ValueBound::after_add(&bound_a, &bound_imm);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);
            state.advance_pc(4);
        }

        // ===== Logical Operations =====
        Instruction::And { rd, rs1, rs2 } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(state.read_reg(*rs2));
            let result = a.bitwise_and(b);

            // Propagate bounds: AND reduces bound to min of inputs
            let bound_a = state.read_bound(*rs1);
            let bound_b = state.read_bound(*rs2);
            let result_bound = ValueBound::after_and(&bound_a, &bound_b);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);
            state.advance_pc(4);
        }

        Instruction::Or { rd, rs1, rs2 } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(state.read_reg(*rs2));
            let result = a.bitwise_or(b);

            // Propagate bounds: OR takes max of inputs
            let bound_a = state.read_bound(*rs1);
            let bound_b = state.read_bound(*rs2);
            let result_bound = ValueBound::after_or(&bound_a, &bound_b);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);
            state.advance_pc(4);
        }

        Instruction::Xor { rd, rs1, rs2 } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(state.read_reg(*rs2));
            let result = a.bitwise_xor(b);

            // Propagate bounds: XOR takes max of inputs
            let bound_a = state.read_bound(*rs1);
            let bound_b = state.read_bound(*rs2);
            let result_bound = ValueBound::after_xor(&bound_a, &bound_b);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);
            state.advance_pc(4);
        }

        Instruction::Andi { rd, rs1, imm } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(*imm as u64);
            let result = a.bitwise_and(b);

            // Propagate bounds: AND with immediate has tight bound
            let bound_a = state.read_bound(*rs1);
            let bound_imm = ValueBound::from_constant(*imm as u64);
            let result_bound = ValueBound::after_and(&bound_a, &bound_imm);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);
            state.advance_pc(4);
        }

        Instruction::Ori { rd, rs1, imm } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(*imm as u64);
            let result = a.bitwise_or(b);

            // Propagate bounds: OR with immediate
            let bound_a = state.read_bound(*rs1);
            let bound_imm = ValueBound::from_constant(*imm as u64);
            let result_bound = ValueBound::after_or(&bound_a, &bound_imm);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);
            state.advance_pc(4);
        }

        Instruction::Xori { rd, rs1, imm } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(*imm as u64);
            let result = a.bitwise_xor(b);

            // Propagate bounds: XOR with immediate
            let bound_a = state.read_bound(*rs1);
            let bound_imm = ValueBound::from_constant(*imm as u64);
            let result_bound = ValueBound::after_xor(&bound_a, &bound_imm);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);
            state.advance_pc(4);
        }

        // ===== Shift Operations =====
        Instruction::Sll { rd, rs1, rs2 } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let shift = (state.read_reg(*rs2) & 0x3F) as u32; // Mask to 6 bits
            let result = a.left_shift(shift);

            // Propagate bounds: left shift increases bound (SHL needs max_bits param)
            let bound_val = state.read_bound(*rs1);
            let result_bound = ValueBound::after_shl(&bound_val, shift, 40);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);
            state.advance_pc(4);
        }

        Instruction::Srl { rd, rs1, rs2 } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let shift = (state.read_reg(*rs2) & 0x3F) as u32;
            let result = a.right_shift(shift);

            // Propagate bounds: right shift reduces bound
            let bound_val = state.read_bound(*rs1);
            let result_bound = ValueBound::after_srl(&bound_val, shift);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);
            state.advance_pc(4);
        }

        Instruction::Sra { rd, rs1, rs2 } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let shift = (state.read_reg(*rs2) & 0x3F) as u32;
            let result = a.arithmetic_right_shift(shift, 40);

            // Propagate bounds: arithmetic right shift preserves sign bit (needs data_bits)
            let bound_val = state.read_bound(*rs1);
            let result_bound = ValueBound::after_sra(&bound_val, shift, 40);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);
            state.advance_pc(4);
        }

        Instruction::Slli { rd, rs1, shamt } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let result = a.left_shift(*shamt as u32);

            // Propagate bounds: immediate shift amount is known
            let bound_val = state.read_bound(*rs1);
            let result_bound = ValueBound::after_shl(&bound_val, *shamt as u32, 40);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);
            state.advance_pc(4);
        }

        Instruction::Srli { rd, rs1, shamt } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let result = a.right_shift(*shamt as u32);

            // Propagate bounds: immediate shift amount is known
            let bound_val = state.read_bound(*rs1);
            let result_bound = ValueBound::after_srl(&bound_val, *shamt as u32);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);
            state.advance_pc(4);
        }

        Instruction::Srai { rd, rs1, shamt } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let result = a.arithmetic_right_shift(*shamt as u32, 40);

            // Propagate bounds: immediate shift amount is known
            let bound_val = state.read_bound(*rs1);
            let result_bound = ValueBound::after_sra(&bound_val, *shamt as u32, 40);

            state.write_reg_with_bound(*rd, result.to_u64(), result_bound);
            state.advance_pc(4);
        }

        // ===== Comparison Operations =====
        Instruction::Slt { rd, rs1, rs2 } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(state.read_reg(*rs2));
            let result = if a.signed_lt(b, 40) { 1 } else { 0 };

            // Propagate bounds: result is always 0 or 1 (boolean)
            let result_bound = ValueBound::after_cmp();

            state.write_reg_with_bound(*rd, result, result_bound);
            state.advance_pc(4);
        }

        Instruction::Sltu { rd, rs1, rs2 } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(state.read_reg(*rs2));
            let result = if a.unsigned_lt(b) { 1 } else { 0 };

            // Propagate bounds: result is always 0 or 1 (boolean)
            let result_bound = ValueBound::after_cmp();

            state.write_reg_with_bound(*rd, result, result_bound);
            state.advance_pc(4);
        }

        Instruction::Sge { rd, rs1, rs2 } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(state.read_reg(*rs2));
            let result = if !a.signed_lt(b, 40) { 1 } else { 0 };

            // Propagate bounds: result is always 0 or 1 (boolean)
            let result_bound = ValueBound::after_cmp();

            state.write_reg_with_bound(*rd, result, result_bound);
            state.advance_pc(4);
        }

        Instruction::Sgeu { rd, rs1, rs2 } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(state.read_reg(*rs2));
            let result = if !a.unsigned_lt(b) { 1 } else { 0 };

            // Propagate bounds: result is always 0 or 1 (boolean)
            let result_bound = ValueBound::after_cmp();

            state.write_reg_with_bound(*rd, result, result_bound);
            state.advance_pc(4);
        }

        Instruction::Seq { rd, rs1, rs2 } => {
            let a = state.read_reg(*rs1);
            let b = state.read_reg(*rs2);
            let result = if a == b { 1 } else { 0 };

            // Propagate bounds: result is always 0 or 1 (boolean)
            let result_bound = ValueBound::after_cmp();

            state.write_reg_with_bound(*rd, result, result_bound);
            state.advance_pc(4);
        }

        Instruction::Sne { rd, rs1, rs2 } => {
            let a = state.read_reg(*rs1);
            let b = state.read_reg(*rs2);
            let result = if a != b { 1 } else { 0 };

            // Propagate bounds: result is always 0 or 1 (boolean)
            let result_bound = ValueBound::after_cmp();

            state.write_reg_with_bound(*rd, result, result_bound);
            state.advance_pc(4);
        }

        // ===== Conditional Move Operations =====
        Instruction::Cmov { rd, rs1, rs2 } => {
            let cond = state.read_reg(*rs2) != 0;
            if cond {
                // Propagate bounds: conditional move uses source bound
                // For prover, we conservatively use max of both paths
                let bound_src = state.read_bound(*rs1);
                let bound_dst = state.read_bound(*rd);
                let result_bound = ValueBound::computed(bound_src.max_bits.max(bound_dst.max_bits));

                state.write_reg_with_bound(*rd, state.read_reg(*rs1), result_bound);
            }
            state.advance_pc(4);
        }

        Instruction::Cmovz { rd, rs1, rs2 } => {
            let cond = state.read_reg(*rs2) == 0;
            if cond {
                // Propagate bounds: conditional move uses source bound
                // For prover, we conservatively use max of both paths
                let bound_src = state.read_bound(*rs1);
                let bound_dst = state.read_bound(*rd);
                let result_bound = ValueBound::computed(bound_src.max_bits.max(bound_dst.max_bits));

                state.write_reg_with_bound(*rd, state.read_reg(*rs1), result_bound);
            }
            state.advance_pc(4);
        }

        Instruction::Cmovnz { rd, rs1, rs2 } => {
            let cond = state.read_reg(*rs2) != 0;
            if cond {
                // Propagate bounds: conditional move uses source bound
                // For prover, we conservatively use max of both paths
                let bound_src = state.read_bound(*rs1);
                let bound_dst = state.read_bound(*rd);
                let result_bound = ValueBound::computed(bound_src.max_bits.max(bound_dst.max_bits));

                state.write_reg_with_bound(*rd, state.read_reg(*rs1), result_bound);
            }
            state.advance_pc(4);
        }

        // ===== Load Operations =====
        Instruction::Lb { rd, rs1, imm } => {
            let addr = state.read_reg(*rs1).wrapping_add(*imm as u64);
            let byte = memory.read_u8(addr)? as i8;
            let value = byte as i64 as u64;

            // Propagate bounds: signed 8-bit load (sign-extended)
            let result_bound = ValueBound::from_type_width(8);

            state.write_reg_with_bound(*rd, value, result_bound);
            state.advance_pc(4);
        }

        Instruction::Lbu { rd, rs1, imm } => {
            let addr = state.read_reg(*rs1).wrapping_add(*imm as u64);
            let byte = memory.read_u8(addr)?;
            let value = byte as u64;

            // Propagate bounds: unsigned 8-bit load
            let result_bound = ValueBound::from_type_width(8);

            state.write_reg_with_bound(*rd, value, result_bound);
            state.advance_pc(4);
        }

        Instruction::Lh { rd, rs1, imm } => {
            let addr = state.read_reg(*rs1).wrapping_add(*imm as u64);
            let halfword = memory.read_u16(addr)? as i16;
            let value = halfword as i64 as u64;

            // Propagate bounds: signed 16-bit load (sign-extended)
            let result_bound = ValueBound::from_type_width(16);

            state.write_reg_with_bound(*rd, value, result_bound);
            state.advance_pc(4);
        }

        Instruction::Lhu { rd, rs1, imm } => {
            let addr = state.read_reg(*rs1).wrapping_add(*imm as u64);
            let halfword = memory.read_u16(addr)?;
            let value = halfword as u64;

            // Propagate bounds: unsigned 16-bit load
            let result_bound = ValueBound::from_type_width(16);

            state.write_reg_with_bound(*rd, value, result_bound);
            state.advance_pc(4);
        }

        Instruction::Lw { rd, rs1, imm } => {
            let addr = state.read_reg(*rs1).wrapping_add(*imm as u64);
            let word = memory.read_u32(addr)?;
            let value = word as u64;

            // Propagate bounds: 32-bit load
            let result_bound = ValueBound::from_type_width(32);

            state.write_reg_with_bound(*rd, value, result_bound);
            state.advance_pc(4);
        }

        Instruction::Ld { rd, rs1, imm } => {
            let addr = state.read_reg(*rs1).wrapping_add(*imm as u64);
            let dword = memory.read_u64(addr)?;

            // Propagate bounds: 64-bit load (but limited to 40-bit program width)
            let result_bound = ValueBound::from_type_width(40);

            state.write_reg_with_bound(*rd, dword, result_bound);
            state.advance_pc(4);
        }

        // ===== Store Operations =====
        Instruction::Sb { rs1, rs2, imm } => {
            let addr = state.read_reg(*rs1).wrapping_add(*imm as u64);
            let value = (state.read_reg(*rs2) & 0xFF) as u8;
            memory.write_u8(addr, value)?;
            state.advance_pc(4);
        }

        Instruction::Sh { rs1, rs2, imm } => {
            let addr = state.read_reg(*rs1).wrapping_add(*imm as u64);
            let value = (state.read_reg(*rs2) & 0xFFFF) as u16;
            memory.write_u16(addr, value)?;
            state.advance_pc(4);
        }

        Instruction::Sw { rs1, rs2, imm } => {
            let addr = state.read_reg(*rs1).wrapping_add(*imm as u64);
            let value = (state.read_reg(*rs2) & 0xFFFFFFFF) as u32;
            memory.write_u32(addr, value)?;
            state.advance_pc(4);
        }

        Instruction::Sd { rs1, rs2, imm } => {
            let addr = state.read_reg(*rs1).wrapping_add(*imm as u64);
            let value = state.read_reg(*rs2);
            memory.write_u64(addr, value)?;
            state.advance_pc(4);
        }

        // ===== Branch Operations =====
        Instruction::Beq { rs1, rs2, offset } => {
            let a = state.read_reg(*rs1);
            let b = state.read_reg(*rs2);
            if a == b {
                state.advance_pc(*offset as i64);
            } else {
                state.advance_pc(4);
            }
        }

        Instruction::Bne { rs1, rs2, offset } => {
            let a = state.read_reg(*rs1);
            let b = state.read_reg(*rs2);
            if a != b {
                state.advance_pc(*offset as i64);
            } else {
                state.advance_pc(4);
            }
        }

        Instruction::Blt { rs1, rs2, offset } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(state.read_reg(*rs2));
            if a.signed_lt(b, 40) {
                state.advance_pc(*offset as i64);
            } else {
                state.advance_pc(4);
            }
        }

        Instruction::Bge { rs1, rs2, offset } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(state.read_reg(*rs2));
            if !a.signed_lt(b, 40) {
                state.advance_pc(*offset as i64);
            } else {
                state.advance_pc(4);
            }
        }

        Instruction::Bltu { rs1, rs2, offset } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(state.read_reg(*rs2));
            if a.unsigned_lt(b) {
                state.advance_pc(*offset as i64);
            } else {
                state.advance_pc(4);
            }
        }

        Instruction::Bgeu { rs1, rs2, offset } => {
            let a = Value40::from_u64(state.read_reg(*rs1));
            let b = Value40::from_u64(state.read_reg(*rs2));
            if !a.unsigned_lt(b) {
                state.advance_pc(*offset as i64);
            } else {
                state.advance_pc(4);
            }
        }

        // ===== Jump Operations =====
        Instruction::Jal { rd, offset } => {
            let return_addr = state.pc + 4;

            // Propagate bounds: return address is a PC value (tight bound)
            let result_bound = ValueBound::from_constant(return_addr);

            state.write_reg_with_bound(*rd, return_addr, result_bound);
            state.advance_pc(*offset as i64);
        }

        Instruction::Jalr { rd, rs1, imm } => {
            let return_addr = state.pc + 4;
            let target = state.read_reg(*rs1).wrapping_add(*imm as u64);

            // Propagate bounds: return address is a PC value (tight bound)
            let result_bound = ValueBound::from_constant(return_addr);

            state.write_reg_with_bound(*rd, return_addr, result_bound);
            state.pc = target & !1; // Clear LSB for alignment
        }

        // ===== System Operations =====
        Instruction::Ecall => {
            // Syscall is handled externally by vm.rs
            // Just advance PC here
            state.advance_pc(4);
        }

        Instruction::Ebreak => {
            state.halt(HaltReason::Ebreak);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zkir_spec::Register;

    fn setup() -> (VMState, Memory) {
        let state = VMState::new(0);
        let memory = Memory::new();
        (state, memory)
    }

    #[test]
    fn test_arithmetic_add() {
        let (mut state, mut memory) = setup();
        state.write_reg(Register::R1, 100);
        state.write_reg(Register::R2, 50);

        let inst = Instruction::Add {
            rd: Register::R3,
            rs1: Register::R1,
            rs2: Register::R2,
        };
        execute(&inst, &mut state, &mut memory, None).unwrap();

        assert_eq!(state.read_reg(Register::R3), 150);
        assert_eq!(state.pc, 4);
    }

    #[test]
    fn test_arithmetic_sub() {
        let (mut state, mut memory) = setup();
        state.write_reg(Register::R1, 100);
        state.write_reg(Register::R2, 30);

        let inst = Instruction::Sub {
            rd: Register::R3,
            rs1: Register::R1,
            rs2: Register::R2,
        };
        execute(&inst, &mut state, &mut memory, None).unwrap();

        assert_eq!(state.read_reg(Register::R3), 70);
    }

    #[test]
    fn test_logical_and() {
        let (mut state, mut memory) = setup();
        state.write_reg(Register::R1, 0b1100);
        state.write_reg(Register::R2, 0b1010);

        let inst = Instruction::And {
            rd: Register::R3,
            rs1: Register::R1,
            rs2: Register::R2,
        };
        execute(&inst, &mut state, &mut memory, None).unwrap();

        assert_eq!(state.read_reg(Register::R3), 0b1000);
    }

    #[test]
    fn test_shift_left() {
        let (mut state, mut memory) = setup();
        state.write_reg(Register::R1, 0b11);
        state.write_reg(Register::R2, 4);

        let inst = Instruction::Sll {
            rd: Register::R3,
            rs1: Register::R1,
            rs2: Register::R2,
        };
        execute(&inst, &mut state, &mut memory, None).unwrap();

        assert_eq!(state.read_reg(Register::R3), 0b110000);
    }

    #[test]
    fn test_comparison_slt() {
        let (mut state, mut memory) = setup();
        state.write_reg(Register::R1, 10);
        state.write_reg(Register::R2, 20);

        let inst = Instruction::Slt {
            rd: Register::R3,
            rs1: Register::R1,
            rs2: Register::R2,
        };
        execute(&inst, &mut state, &mut memory, None).unwrap();

        assert_eq!(state.read_reg(Register::R3), 1);
    }

    #[test]
    fn test_load_store() {
        let (mut state, mut memory) = setup();
        state.write_reg(Register::R1, 0x1000);
        state.write_reg(Register::R2, 0x12345678);

        // Store word
        let inst = Instruction::Sw {
            rs1: Register::R1,
            rs2: Register::R2,
            imm: 0,
        };
        execute(&inst, &mut state, &mut memory, None).unwrap();

        // Load word
        let inst = Instruction::Lw {
            rd: Register::R3,
            rs1: Register::R1,
            imm: 0,
        };
        execute(&inst, &mut state, &mut memory, None).unwrap();

        assert_eq!(state.read_reg(Register::R3), 0x12345678);
    }

    #[test]
    fn test_branch_taken() {
        let (mut state, mut memory) = setup();
        state.write_reg(Register::R1, 10);
        state.write_reg(Register::R2, 10);

        let inst = Instruction::Beq {
            rs1: Register::R1,
            rs2: Register::R2,
            offset: 100,
        };
        execute(&inst, &mut state, &mut memory, None).unwrap();

        assert_eq!(state.pc as i64, 100);
    }

    #[test]
    fn test_branch_not_taken() {
        let (mut state, mut memory) = setup();
        state.write_reg(Register::R1, 10);
        state.write_reg(Register::R2, 20);

        let inst = Instruction::Beq {
            rs1: Register::R1,
            rs2: Register::R2,
            offset: 100,
        };
        execute(&inst, &mut state, &mut memory, None).unwrap();

        assert_eq!(state.pc, 4);
    }

    #[test]
    fn test_jal() {
        let (mut state, mut memory) = setup();

        let inst = Instruction::Jal {
            rd: Register::R1,
            offset: 1000,
        };
        execute(&inst, &mut state, &mut memory, None).unwrap();

        assert_eq!(state.read_reg(Register::R1), 4); // Return address
        assert_eq!(state.pc as i64, 1000);
    }

    #[test]
    fn test_ebreak() {
        let (mut state, mut memory) = setup();

        let inst = Instruction::Ebreak;
        execute(&inst, &mut state, &mut memory, None).unwrap();

        assert!(state.is_halted());
        assert_eq!(state.halt_reason, Some(HaltReason::Ebreak));
    }

    #[test]
    fn test_division_by_zero() {
        let (mut state, mut memory) = setup();
        state.write_reg(Register::R1, 100);
        state.write_reg(Register::R2, 0);

        let inst = Instruction::Div {
            rd: Register::R3,
            rs1: Register::R1,
            rs2: Register::R2,
        };
        let result = execute(&inst, &mut state, &mut memory, None);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::DivisionByZero { .. }
        ));
    }
}

/// Execute instruction with deferred carry model and normalization at observation points
///
/// This function wraps the regular execute() function and adds:
/// 1. Normalization at observation points (branches, stores, bitwise, MUL, comparisons)
/// 2. Deferred arithmetic for ADD/SUB/ADDI operations
/// 3. Normalization witness collection for proof generation
///
/// # Parameters
/// - `inst`: Instruction to execute
/// - `state`: VM state
/// - `memory`: Memory subsystem
/// - `range_checker`: Optional range check tracker
/// - `deferred_config`: Configuration for deferred model (limb bits, etc.)
/// - `cycle`: Current cycle number (for witness generation)
/// - `pc`: Program counter (for witness generation)
///
/// # Returns
/// Vector of normalization events that occurred during this instruction
pub fn execute_with_deferred(
    inst: &Instruction,
    state: &mut VMState,
    memory: &mut Memory,
    range_checker: Option<&mut RangeCheckTracker>,
    deferred_config: Option<&DeferredConfig>,
    cycle: u64,
    pc: u64,
) -> Result<Vec<NormalizationEvent>> {
    let mut normalization_events = Vec::new();

    let default_config = DeferredConfig::default();
    let config = deferred_config.unwrap_or(&default_config);

    // Helper macro to normalize rs1 with witness, rs2 without
    macro_rules! norm_two {
        ($rs1:expr, $rs2:expr, $opc:expr) => {{
            if $rs1 != Register::R0 {
                if let Some(result) = state.normalize_register_for_observation($rs1, config.normalized_bits, config.limb_bits) {
                    normalization_events.push(NormalizationEvent::observation_point(
                        cycle, pc, $rs1, &result, config.normalized_bits, config.limb_bits, $opc,
                    ));
                }
            }
            if $rs2 != Register::R0 {
                let _ = state.normalize_register($rs2, config.normalized_bits, config.limb_bits);
            }
        }};
    }

    // Helper macro to normalize rs1 with witness
    macro_rules! norm_one {
        ($rs1:expr, $opc:expr) => {{
            if $rs1 != Register::R0 {
                if let Some(result) = state.normalize_register_for_observation($rs1, config.normalized_bits, config.limb_bits) {
                    normalization_events.push(NormalizationEvent::observation_point(
                        cycle, pc, $rs1, &result, config.normalized_bits, config.limb_bits, $opc,
                    ));
                }
            }
        }};
    }

    // Normalize observation point source registers before execution
    // TEMPORARY: Only normalize first source register to work around
    // architectural limitation (prover supports only one normalization per row)
    match inst {
        // Branches
        Instruction::Beq { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Beq),
        Instruction::Bne { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Bne),
        Instruction::Blt { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Blt),
        Instruction::Bge { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Bge),
        Instruction::Bltu { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Bltu),
        Instruction::Bgeu { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Bgeu),

        // Stores
        Instruction::Sw { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Sw),
        Instruction::Sh { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Sh),
        Instruction::Sb { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Sb),

        // Bitwise R-type
        Instruction::And { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::And),
        Instruction::Or { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Or),
        Instruction::Xor { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Xor),
        Instruction::Sll { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Sll),
        Instruction::Srl { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Srl),
        Instruction::Sra { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Sra),

        // Bitwise I-type
        Instruction::Andi { rs1, .. } => norm_one!(*rs1, Opcode::Andi),
        Instruction::Ori { rs1, .. } => norm_one!(*rs1, Opcode::Ori),
        Instruction::Xori { rs1, .. } => norm_one!(*rs1, Opcode::Xori),
        Instruction::Slli { rs1, .. } => norm_one!(*rs1, Opcode::Slli),
        Instruction::Srli { rs1, .. } => norm_one!(*rs1, Opcode::Srli),
        Instruction::Srai { rs1, .. } => norm_one!(*rs1, Opcode::Srai),

        // MUL/DIV
        Instruction::Mul { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Mul),
        Instruction::Mulh { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Mulh),
        Instruction::Div { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Div),
        Instruction::Divu { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Divu),
        Instruction::Rem { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Rem),
        Instruction::Remu { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Remu),

        // Comparisons
        Instruction::Seq { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Seq),
        Instruction::Sne { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Sne),
        Instruction::Slt { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Slt),
        Instruction::Sltu { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Sltu),
        Instruction::Sge { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Sge),
        Instruction::Sgeu { rs1, rs2, .. } => norm_two!(*rs1, *rs2, Opcode::Sgeu),

        // All other instructions don't require normalization
        _ => {}
    }

    // Execute the instruction
    // For ADD/SUB/ADDI, use deferred arithmetic
    match inst {
        Instruction::Add { rd, rs1, rs2 } => {
            execute_add_deferred(state, *rd, *rs1, *rs2, config, range_checker);
        }
        Instruction::Sub { rd, rs1, rs2 } => {
            execute_sub_deferred(state, *rd, *rs1, *rs2, config, range_checker);
        }
        Instruction::Addi { rd, rs1, imm } => {
            execute_addi_deferred(state, *rd, *rs1, *imm as u64, config, range_checker);
        }
        _ => {
            // All other instructions use regular execution
            execute(inst, state, memory, range_checker)?;
        }
    }

    Ok(normalization_events)
}
