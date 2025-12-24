//! Instruction execution for ZK IR v2.2

use zkir_spec::{Instruction, BABYBEAR_PRIME, MAX_30BIT};
use crate::state::{VMState, HaltReason};
use crate::io::IOHandler;
use crate::error::RuntimeError;
use crate::syscall::handle_syscall;

/// Mask to 30 bits
#[inline]
fn mask30(value: u32) -> u32 {
    value & MAX_30BIT
}

/// Sign extend from 30 bits to i32
#[inline]
fn sign_extend_30(value: u32) -> i32 {
    ((value << 2) as i32) >> 2
}

/// Execute single instruction
pub fn execute(
    instr: &Instruction,
    state: &mut VMState,
    io: &mut IOHandler,
) -> Result<(), RuntimeError> {
    match instr {
        // ========== R-type ALU (opcode = 0000) ==========

        // Arithmetic
        Instruction::Add { rd, rs1, rs2 } => {
            let result = mask30(state.read_reg(*rs1).wrapping_add(state.read_reg(*rs2)));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Sub { rd, rs1, rs2 } => {
            let result = mask30(state.read_reg(*rs1).wrapping_sub(state.read_reg(*rs2)));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Mul { rd, rs1, rs2 } => {
            let a = state.read_reg(*rs1) as u64;
            let b = state.read_reg(*rs2) as u64;
            let result = mask30((a * b) as u32);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Mulh { rd, rs1, rs2 } => {
            let a = sign_extend_30(state.read_reg(*rs1)) as i64;
            let b = sign_extend_30(state.read_reg(*rs2)) as i64;
            let result = mask30(((a * b) >> 30) as u32);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Mulhu { rd, rs1, rs2 } => {
            let a = state.read_reg(*rs1) as u64;
            let b = state.read_reg(*rs2) as u64;
            let result = mask30(((a * b) >> 30) as u32);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Mulhsu { rd, rs1, rs2 } => {
            let a = sign_extend_30(state.read_reg(*rs1)) as i64;
            let b = state.read_reg(*rs2) as u64;
            let result = mask30(((a as i64 * b as i64) >> 30) as u32);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Div { rd, rs1, rs2 } => {
            let divisor = state.read_reg(*rs2);
            if divisor == 0 {
                return Err(RuntimeError::Halt(HaltReason::DivisionByZero { pc: state.pc }));
            }
            let a = sign_extend_30(state.read_reg(*rs1));
            let b = sign_extend_30(divisor);
            let result = mask30((a / b) as u32);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Divu { rd, rs1, rs2 } => {
            let divisor = state.read_reg(*rs2);
            if divisor == 0 {
                return Err(RuntimeError::Halt(HaltReason::DivisionByZero { pc: state.pc }));
            }
            let result = mask30(state.read_reg(*rs1) / divisor);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Rem { rd, rs1, rs2 } => {
            let divisor = state.read_reg(*rs2);
            if divisor == 0 {
                return Err(RuntimeError::Halt(HaltReason::DivisionByZero { pc: state.pc }));
            }
            let a = sign_extend_30(state.read_reg(*rs1));
            let b = sign_extend_30(divisor);
            let result = mask30((a % b) as u32);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Remu { rd, rs1, rs2 } => {
            let divisor = state.read_reg(*rs2);
            if divisor == 0 {
                return Err(RuntimeError::Halt(HaltReason::DivisionByZero { pc: state.pc }));
            }
            let result = mask30(state.read_reg(*rs1) % divisor);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        // Logic
        Instruction::And { rd, rs1, rs2 } => {
            let result = mask30(state.read_reg(*rs1) & state.read_reg(*rs2));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Andn { rd, rs1, rs2 } => {
            let result = mask30(state.read_reg(*rs1) & !state.read_reg(*rs2));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Or { rd, rs1, rs2 } => {
            let result = mask30(state.read_reg(*rs1) | state.read_reg(*rs2));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Orn { rd, rs1, rs2 } => {
            let result = mask30(state.read_reg(*rs1) | !state.read_reg(*rs2));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Xor { rd, rs1, rs2 } => {
            let result = mask30(state.read_reg(*rs1) ^ state.read_reg(*rs2));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Xnor { rd, rs1, rs2 } => {
            let result = mask30(!(state.read_reg(*rs1) ^ state.read_reg(*rs2)));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        // Shift
        Instruction::Sll { rd, rs1, rs2 } => {
            let shamt = state.read_reg(*rs2) & 0x1F;
            let result = mask30(state.read_reg(*rs1) << shamt);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Srl { rd, rs1, rs2 } => {
            let shamt = state.read_reg(*rs2) & 0x1F;
            let result = mask30(state.read_reg(*rs1) >> shamt);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Sra { rd, rs1, rs2 } => {
            let shamt = state.read_reg(*rs2) & 0x1F;
            let value = sign_extend_30(state.read_reg(*rs1));
            let result = mask30((value >> shamt) as u32);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Rol { rd, rs1, rs2 } => {
            let shamt = state.read_reg(*rs2) & 0x1F;
            let value = state.read_reg(*rs1);
            let result = mask30((value << shamt) | (value >> (30 - shamt)));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Ror { rd, rs1, rs2 } => {
            let shamt = state.read_reg(*rs2) & 0x1F;
            let value = state.read_reg(*rs1);
            let result = mask30((value >> shamt) | (value << (30 - shamt)));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        // Compare
        Instruction::Slt { rd, rs1, rs2 } => {
            let a = sign_extend_30(state.read_reg(*rs1));
            let b = sign_extend_30(state.read_reg(*rs2));
            state.write_reg(*rd, if a < b { 1 } else { 0 });
            state.pc += 4;
        }

        Instruction::Sltu { rd, rs1, rs2 } => {
            let a = state.read_reg(*rs1);
            let b = state.read_reg(*rs2);
            state.write_reg(*rd, if a < b { 1 } else { 0 });
            state.pc += 4;
        }

        Instruction::Min { rd, rs1, rs2 } => {
            let a = sign_extend_30(state.read_reg(*rs1));
            let b = sign_extend_30(state.read_reg(*rs2));
            state.write_reg(*rd, mask30(a.min(b) as u32));
            state.pc += 4;
        }

        Instruction::Max { rd, rs1, rs2 } => {
            let a = sign_extend_30(state.read_reg(*rs1));
            let b = sign_extend_30(state.read_reg(*rs2));
            state.write_reg(*rd, mask30(a.max(b) as u32));
            state.pc += 4;
        }

        Instruction::Minu { rd, rs1, rs2 } => {
            let result = state.read_reg(*rs1).min(state.read_reg(*rs2));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Maxu { rd, rs1, rs2 } => {
            let result = state.read_reg(*rs1).max(state.read_reg(*rs2));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        // Bit Manipulation
        Instruction::Clz { rd, rs1, .. } => {
            let value = state.read_reg(*rs1);
            let result = if value == 0 { 30 } else { value.leading_zeros() - 2 };
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Ctz { rd, rs1, .. } => {
            let value = state.read_reg(*rs1);
            let result = if value == 0 { 30 } else { value.trailing_zeros() };
            state.write_reg(*rd, result.min(30));
            state.pc += 4;
        }

        Instruction::Cpop { rd, rs1, .. } => {
            let value = state.read_reg(*rs1) & MAX_30BIT;
            state.write_reg(*rd, value.count_ones());
            state.pc += 4;
        }

        Instruction::Rev8 { rd, rs1, .. } => {
            let value = state.read_reg(*rs1);
            let result = mask30(value.swap_bytes());
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        // Conditional Move
        Instruction::Cmovz { rd, rs1, rs2 } => {
            if state.read_reg(*rs2) == 0 {
                state.write_reg(*rd, state.read_reg(*rs1));
            }
            state.pc += 4;
        }

        Instruction::Cmovnz { rd, rs1, rs2 } => {
            if state.read_reg(*rs2) != 0 {
                state.write_reg(*rd, state.read_reg(*rs1));
            }
            state.pc += 4;
        }

        // Field Operations
        Instruction::Fadd { rd, rs1, rs2 } => {
            let a = state.read_reg(*rs1) as u64;
            let b = state.read_reg(*rs2) as u64;
            let result = ((a + b) % BABYBEAR_PRIME as u64) as u32;
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Fsub { rd, rs1, rs2 } => {
            let a = state.read_reg(*rs1) as u64;
            let b = state.read_reg(*rs2) as u64;
            let result = ((a + BABYBEAR_PRIME as u64 - b) % BABYBEAR_PRIME as u64) as u32;
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Fmul { rd, rs1, rs2 } => {
            let a = state.read_reg(*rs1) as u64;
            let b = state.read_reg(*rs2) as u64;
            let result = ((a * b) % BABYBEAR_PRIME as u64) as u32;
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Fneg { rd, rs1, .. } => {
            let a = state.read_reg(*rs1);
            let result = if a == 0 { 0 } else { BABYBEAR_PRIME - a };
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Finv { rd, rs1, .. } => {
            let a = state.read_reg(*rs1);
            if a == 0 {
                return Err(RuntimeError::Halt(HaltReason::DivisionByZero { pc: state.pc }));
            }
            // Fermat's little theorem: a^(p-2) mod p
            let result = field_inv(a, BABYBEAR_PRIME);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        // ========== I-type Immediate (opcode = 0001) ==========

        Instruction::Addi { rd, rs1, imm } => {
            let result = mask30(state.read_reg(*rs1).wrapping_add(*imm as u32));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Slti { rd, rs1, imm } => {
            let a = sign_extend_30(state.read_reg(*rs1));
            state.write_reg(*rd, if a < (*imm as i32) { 1 } else { 0 });
            state.pc += 4;
        }

        Instruction::Sltiu { rd, rs1, imm } => {
            let a = state.read_reg(*rs1);
            let b = *imm as u32;
            state.write_reg(*rd, if a < b { 1 } else { 0 });
            state.pc += 4;
        }

        Instruction::Xori { rd, rs1, imm } => {
            let result = mask30(state.read_reg(*rs1) ^ (*imm as u32));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Ori { rd, rs1, imm } => {
            let result = mask30(state.read_reg(*rs1) | (*imm as u32));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Andi { rd, rs1, imm } => {
            let result = mask30(state.read_reg(*rs1) & (*imm as u32));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Slli { rd, rs1, shamt } => {
            let result = mask30(state.read_reg(*rs1) << shamt);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Srli { rd, rs1, shamt } => {
            let result = mask30(state.read_reg(*rs1) >> shamt);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Srai { rd, rs1, shamt } => {
            let value = sign_extend_30(state.read_reg(*rs1));
            let result = mask30((value >> shamt) as u32);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        // ========== Load (opcode = 0010) ==========

        Instruction::Lb { rd, rs1, imm } => {
            let addr = mask30(state.read_reg(*rs1).wrapping_add(*imm as u32));
            let value = state.memory.load_byte_signed(addr, state.cycle)?;
            state.write_reg(*rd, value);
            state.pc += 4;
        }

        Instruction::Lh { rd, rs1, imm } => {
            let addr = mask30(state.read_reg(*rs1).wrapping_add(*imm as u32));
            let value = state.memory.load_half_signed(addr, state.cycle)?;
            state.write_reg(*rd, value);
            state.pc += 4;
        }

        Instruction::Lw { rd, rs1, imm } => {
            let addr = mask30(state.read_reg(*rs1).wrapping_add(*imm as u32));
            let value = state.memory.load_word(addr, state.cycle)?;
            state.write_reg(*rd, mask30(value));
            state.pc += 4;
        }

        Instruction::Lbu { rd, rs1, imm } => {
            let addr = mask30(state.read_reg(*rs1).wrapping_add(*imm as u32));
            let value = state.memory.load_byte(addr, state.cycle)?;
            state.write_reg(*rd, value);
            state.pc += 4;
        }

        Instruction::Lhu { rd, rs1, imm } => {
            let addr = mask30(state.read_reg(*rs1).wrapping_add(*imm as u32));
            let value = state.memory.load_half(addr, state.cycle)?;
            state.write_reg(*rd, value);
            state.pc += 4;
        }

        // ========== Store (opcode = 0011) ==========

        Instruction::Sb { rs1, rs2, imm } => {
            let addr = mask30(state.read_reg(*rs1).wrapping_add(*imm as u32));
            let value = state.read_reg(*rs2);
            state.memory.store_byte(addr, value, state.cycle)?;
            state.pc += 4;
        }

        Instruction::Sh { rs1, rs2, imm } => {
            let addr = mask30(state.read_reg(*rs1).wrapping_add(*imm as u32));
            let value = state.read_reg(*rs2);
            state.memory.store_half(addr, value, state.cycle)?;
            state.pc += 4;
        }

        Instruction::Sw { rs1, rs2, imm } => {
            let addr = mask30(state.read_reg(*rs1).wrapping_add(*imm as u32));
            let value = mask30(state.read_reg(*rs2));
            state.memory.store_word(addr, value, state.cycle)?;
            state.pc += 4;
        }

        // ========== Branch (opcodes 0100-1001) ==========

        Instruction::Beq { rs1, rs2, imm } => {
            if state.read_reg(*rs1) == state.read_reg(*rs2) {
                state.pc = mask30(state.pc.wrapping_add(*imm as u32));
            } else {
                state.pc += 4;
            }
        }

        Instruction::Bne { rs1, rs2, imm } => {
            if state.read_reg(*rs1) != state.read_reg(*rs2) {
                state.pc = mask30(state.pc.wrapping_add(*imm as u32));
            } else {
                state.pc += 4;
            }
        }

        Instruction::Blt { rs1, rs2, imm } => {
            let a = sign_extend_30(state.read_reg(*rs1));
            let b = sign_extend_30(state.read_reg(*rs2));
            if a < b {
                state.pc = mask30(state.pc.wrapping_add(*imm as u32));
            } else {
                state.pc += 4;
            }
        }

        Instruction::Bge { rs1, rs2, imm } => {
            let a = sign_extend_30(state.read_reg(*rs1));
            let b = sign_extend_30(state.read_reg(*rs2));
            if a >= b {
                state.pc = mask30(state.pc.wrapping_add(*imm as u32));
            } else {
                state.pc += 4;
            }
        }

        Instruction::Bltu { rs1, rs2, imm } => {
            if state.read_reg(*rs1) < state.read_reg(*rs2) {
                state.pc = mask30(state.pc.wrapping_add(*imm as u32));
            } else {
                state.pc += 4;
            }
        }

        Instruction::Bgeu { rs1, rs2, imm } => {
            if state.read_reg(*rs1) >= state.read_reg(*rs2) {
                state.pc = mask30(state.pc.wrapping_add(*imm as u32));
            } else {
                state.pc += 4;
            }
        }

        // ========== Upper Immediate (opcodes 1010-1011) ==========

        Instruction::Lui { rd, imm } => {
            let result = mask30((*imm as u32) << 12);
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        Instruction::Auipc { rd, imm } => {
            let result = mask30(state.pc.wrapping_add((*imm as u32) << 12));
            state.write_reg(*rd, result);
            state.pc += 4;
        }

        // ========== Jump (opcodes 1100-1101) ==========

        Instruction::Jal { rd, imm } => {
            state.write_reg(*rd, mask30(state.pc + 4));
            state.pc = mask30(state.pc.wrapping_add(*imm as u32));
        }

        Instruction::Jalr { rd, rs1, imm } => {
            let target = mask30(state.read_reg(*rs1).wrapping_add(*imm as u32));
            state.write_reg(*rd, mask30(state.pc + 4));
            state.pc = target;
        }

        // ========== ZK Operations (opcode = 1110) ==========

        Instruction::Read { rd } => {
            match io.read() {
                Some(value) => {
                    state.write_reg(*rd, mask30(value));
                    state.pc += 4;
                }
                None => {
                    return Err(RuntimeError::Halt(HaltReason::InputExhausted));
                }
            }
        }

        Instruction::Write { rs1 } => {
            let value = state.read_reg(*rs1);
            io.write(value);
            state.pc += 4;
        }

        Instruction::Hint { rd } => {
            match io.read_hint() {
                Some(value) => {
                    state.write_reg(*rd, mask30(value));
                    state.pc += 4;
                }
                None => {
                    // Hints are optional, write 0 if not available
                    state.write_reg(*rd, 0);
                    state.pc += 4;
                }
            }
        }

        Instruction::Commit { rs1 } => {
            let value = state.read_reg(*rs1);
            io.commit(value);
            state.pc += 4;
        }

        Instruction::AssertEq { rs1, rs2 } => {
            let a = state.read_reg(*rs1);
            let b = state.read_reg(*rs2);
            if a != b {
                return Err(RuntimeError::Halt(HaltReason::AssertionFailed {
                    pc: state.pc,
                    msg: format!("ASSERT_EQ failed: {} != {}", a, b),
                }));
            }
            state.pc += 4;
        }

        Instruction::AssertNe { rs1, rs2 } => {
            let a = state.read_reg(*rs1);
            let b = state.read_reg(*rs2);
            if a == b {
                return Err(RuntimeError::Halt(HaltReason::AssertionFailed {
                    pc: state.pc,
                    msg: format!("ASSERT_NE failed: {} == {}", a, b),
                }));
            }
            state.pc += 4;
        }

        Instruction::AssertZero { rs1 } => {
            let value = state.read_reg(*rs1);
            if value != 0 {
                return Err(RuntimeError::Halt(HaltReason::AssertionFailed {
                    pc: state.pc,
                    msg: format!("ASSERT_ZERO failed: {} != 0", value),
                }));
            }
            state.pc += 4;
        }

        Instruction::RangeCheck { rs1, bits } => {
            let value = state.read_reg(*rs1);
            let max = (1u32 << bits) - 1;
            if value > max {
                return Err(RuntimeError::Halt(HaltReason::AssertionFailed {
                    pc: state.pc,
                    msg: format!("RANGE_CHECK failed: {} > {} (max for {} bits)", value, max, bits),
                }));
            }
            state.pc += 4;
        }

        Instruction::Debug { rs1 } => {
            let value = state.read_reg(*rs1);
            tracing::debug!("DEBUG at PC={:08X}: {}", state.pc, value);
            state.pc += 4;
        }

        Instruction::Halt => {
            state.halt(HaltReason::Halt);
        }

        // ========== System (opcode = 1111) ==========

        Instruction::Ecall => {
            handle_syscall(state, io)?;
            state.pc += 4;
        }

        Instruction::Ebreak => {
            // Breakpoint for debugging
            tracing::debug!("EBREAK at PC={:08X}", state.pc);
            state.pc += 4;
        }
    }

    Ok(())
}

/// Compute modular inverse using Fermat's little theorem: a^(p-2) mod p
fn field_inv(a: u32, p: u32) -> u32 {
    // Fast exponentiation
    let mut result = 1u64;
    let mut base = a as u64;
    let mut exp = (p - 2) as u64;
    let modulus = p as u64;

    while exp > 0 {
        if exp & 1 == 1 {
            result = (result * base) % modulus;
        }
        base = (base * base) % modulus;
        exp >>= 1;
    }

    result as u32
}
