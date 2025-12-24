//! Integration tests for ZK IR v2.2 runtime

use zkir_spec::{Instruction, Program, Register};
use zkir_assembler::encoder::encode;
use zkir_runtime::{VM, VMConfig};
use zkir_runtime::syscall::syscall_nums;

#[test]
fn test_simple_halt() {
    // Program: HALT
    let code = vec![encode(&Instruction::Halt)];
    let program = Program::new(code);

    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.cycles, 1);
}

#[test]
fn test_add_instruction() {
    // Program:
    //   ADDI a0, zero, 10
    //   ADDI a1, zero, 32
    //   ADD a2, a0, a1
    //   HALT
    let code = vec![
        encode(&Instruction::Addi {
            rd: Register::A0,
            rs1: Register::ZERO,
            imm: 10,
        }),
        encode(&Instruction::Addi {
            rd: Register::A1,
            rs1: Register::ZERO,
            imm: 32,
        }),
        encode(&Instruction::Add {
            rd: Register::A2,
            rs1: Register::A0,
            rs2: Register::A1,
        }),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.cycles, 4);
}

#[test]
fn test_read_write() {
    // Program:
    //   READ a0
    //   READ a1
    //   ADD a2, a0, a1
    //   WRITE a2
    //   HALT
    let code = vec![
        encode(&Instruction::Read { rd: Register::A0 }),
        encode(&Instruction::Read { rd: Register::A1 }),
        encode(&Instruction::Add {
            rd: Register::A2,
            rs1: Register::A0,
            rs2: Register::A1,
        }),
        encode(&Instruction::Write { rs1: Register::A2 }),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![5, 7], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.cycles, 5);
    assert_eq!(result.outputs, vec![12]);
}

#[test]
fn test_branch_taken() {
    // Program:
    //   ADDI a0, zero, 5
    //   ADDI a1, zero, 5
    //   BEQ a0, a1, 8    ; Skip next instruction
    //   ADDI a2, zero, 1 ; Should be skipped
    //   ADDI a2, zero, 2 ; Should execute
    //   HALT
    let code = vec![
        encode(&Instruction::Addi {
            rd: Register::A0,
            rs1: Register::ZERO,
            imm: 5,
        }),
        encode(&Instruction::Addi {
            rd: Register::A1,
            rs1: Register::ZERO,
            imm: 5,
        }),
        encode(&Instruction::Beq {
            rs1: Register::A0,
            rs2: Register::A1,
            imm: 8,
        }),
        encode(&Instruction::Addi {
            rd: Register::A2,
            rs1: Register::ZERO,
            imm: 1,
        }),
        encode(&Instruction::Addi {
            rd: Register::A2,
            rs1: Register::ZERO,
            imm: 2,
        }),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    // Should execute: addi, addi, beq, addi(2), halt = 5 cycles
    assert_eq!(result.cycles, 5);
}

#[test]
fn test_branch_not_taken() {
    // Program:
    //   ADDI a0, zero, 5
    //   ADDI a1, zero, 3
    //   BEQ a0, a1, 8    ; Should not take
    //   ADDI a2, zero, 1 ; Should execute
    //   HALT
    let code = vec![
        encode(&Instruction::Addi {
            rd: Register::A0,
            rs1: Register::ZERO,
            imm: 5,
        }),
        encode(&Instruction::Addi {
            rd: Register::A1,
            rs1: Register::ZERO,
            imm: 3,
        }),
        encode(&Instruction::Beq {
            rs1: Register::A0,
            rs2: Register::A1,
            imm: 8,
        }),
        encode(&Instruction::Addi {
            rd: Register::A2,
            rs1: Register::ZERO,
            imm: 1,
        }),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.cycles, 5);
}

#[test]
fn test_jal() {
    // Program:
    //   JAL ra, 8        ; Jump forward 8 bytes (2 instructions)
    //   ADDI a0, zero, 1 ; Should be skipped
    //   ADDI a0, zero, 2 ; Should execute
    //   HALT
    let code = vec![
        encode(&Instruction::Jal {
            rd: Register::RA,
            imm: 8,
        }),
        encode(&Instruction::Addi {
            rd: Register::A0,
            rs1: Register::ZERO,
            imm: 1,
        }),
        encode(&Instruction::Addi {
            rd: Register::A0,
            rs1: Register::ZERO,
            imm: 2,
        }),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.cycles, 3); // jal, addi(2), halt
}

#[test]
fn test_lui_auipc() {
    // Program:
    //   LUI a0, 0x12345
    //   AUIPC a1, 0
    //   HALT
    let code = vec![
        encode(&Instruction::Lui {
            rd: Register::A0,
            imm: 0x12345,
        }),
        encode(&Instruction::Auipc {
            rd: Register::A1,
            imm: 0,
        }),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.cycles, 3);
}

#[test]
fn test_field_add() {
    // Program:
    //   ADDI a0, zero, 100
    //   ADDI a1, zero, 200
    //   FADD a2, a0, a1
    //   HALT
    let code = vec![
        encode(&Instruction::Addi {
            rd: Register::A0,
            rs1: Register::ZERO,
            imm: 100,
        }),
        encode(&Instruction::Addi {
            rd: Register::A1,
            rs1: Register::ZERO,
            imm: 200,
        }),
        encode(&Instruction::Fadd {
            rd: Register::A2,
            rs1: Register::A0,
            rs2: Register::A1,
        }),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.cycles, 4);
}

#[test]
fn test_assert_eq_success() {
    // Program:
    //   ADDI a0, zero, 42
    //   ADDI a1, zero, 42
    //   ASSERT_EQ a0, a1
    //   HALT
    let code = vec![
        encode(&Instruction::Addi {
            rd: Register::A0,
            rs1: Register::ZERO,
            imm: 42,
        }),
        encode(&Instruction::Addi {
            rd: Register::A1,
            rs1: Register::ZERO,
            imm: 42,
        }),
        encode(&Instruction::AssertEq {
            rs1: Register::A0,
            rs2: Register::A1,
        }),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.cycles, 4);
}

#[test]
fn test_assert_eq_failure() {
    // Program:
    //   ADDI a0, zero, 42
    //   ADDI a1, zero, 43
    //   ASSERT_EQ a0, a1  ; Should fail
    //   HALT
    let code = vec![
        encode(&Instruction::Addi {
            rd: Register::A0,
            rs1: Register::ZERO,
            imm: 42,
        }),
        encode(&Instruction::Addi {
            rd: Register::A1,
            rs1: Register::ZERO,
            imm: 43,
        }),
        encode(&Instruction::AssertEq {
            rs1: Register::A0,
            rs2: Register::A1,
        }),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    // Should halt on assertion
    assert!(matches!(
        result.halt_reason,
        zkir_runtime::HaltReason::AssertionFailed { .. }
    ));
}

#[test]
fn test_range_check_success() {
    // Program:
    //   ADDI a0, zero, 15
    //   RANGE_CHECK a0, 8  ; Check a0 fits in 8 bits
    //   HALT
    let code = vec![
        encode(&Instruction::Addi {
            rd: Register::A0,
            rs1: Register::ZERO,
            imm: 15,
        }),
        encode(&Instruction::RangeCheck {
            rs1: Register::A0,
            bits: 8,
        }),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.cycles, 3);
}

#[test]
fn test_division_by_zero() {
    // Program:
    //   ADDI a0, zero, 10
    //   ADDI a1, zero, 0
    //   DIV a2, a0, a1  ; Should fail
    //   HALT
    let code = vec![
        encode(&Instruction::Addi {
            rd: Register::A0,
            rs1: Register::ZERO,
            imm: 10,
        }),
        encode(&Instruction::Addi {
            rd: Register::A1,
            rs1: Register::ZERO,
            imm: 0,
        }),
        encode(&Instruction::Div {
            rd: Register::A2,
            rs1: Register::A0,
            rs2: Register::A1,
        }),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert!(matches!(
        result.halt_reason,
        zkir_runtime::HaltReason::DivisionByZero { .. }
    ));
}

#[test]
fn test_commit() {
    // Program:
    //   ADDI a0, zero, 42
    //   COMMIT a0
    //   HALT
    let code = vec![
        encode(&Instruction::Addi {
            rd: Register::A0,
            rs1: Register::ZERO,
            imm: 42,
        }),
        encode(&Instruction::Commit { rs1: Register::A0 }),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.cycles, 3);
    assert_eq!(result.commitments, vec![42]);
}

// ========== Syscall Tests ==========

#[test]
fn test_syscall_exit() {
    // Program:
    //   ADDI a7, zero, SYS_EXIT
    //   ADDI a0, zero, 0  ; exit code 0
    //   ECALL
    //   HALT  ; Should not reach here
    let code = vec![
        encode(&Instruction::Addi {
            rd: Register::A7,
            rs1: Register::ZERO,
            imm: syscall_nums::SYS_EXIT as i16,
        }),
        encode(&Instruction::Addi {
            rd: Register::A0,
            rs1: Register::ZERO,
            imm: 0,
        }),
        encode(&Instruction::Ecall),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    // Should halt on SYS_EXIT
    assert_eq!(result.cycles, 3);
}

#[test]
fn test_syscall_read_write() {
    // Program:
    //   ADDI a7, zero, SYS_READ
    //   ECALL             ; Read into a0
    //   ADDI a7, zero, SYS_WRITE
    //   ECALL             ; Write from a0
    //   HALT
    let code = vec![
        encode(&Instruction::Addi {
            rd: Register::A7,
            rs1: Register::ZERO,
            imm: syscall_nums::SYS_READ as i16,
        }),
        encode(&Instruction::Ecall),
        encode(&Instruction::Addi {
            rd: Register::A7,
            rs1: Register::ZERO,
            imm: syscall_nums::SYS_WRITE as i16,
        }),
        encode(&Instruction::Ecall),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![42], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.outputs, vec![42]);
}

#[test]
fn test_syscall_memcpy() {
    // Program:
    //   ADDI a0, sp, -64  ; dest
    //   ADDI a1, sp, -128 ; src
    //   ADDI a2, zero, 4  ; length (4 words)
    //   ; Store some data at src
    //   ADDI t0, zero, 100
    //   SW t0, a1, 0
    //   ADDI t0, zero, 200
    //   SW t0, a1, 4
    //   ; Call memcpy
    //   ADDI a7, zero, SYS_MEMCPY
    //   ECALL
    //   ; Load from dest
    //   LW a3, a0, 0
    //   LW a4, a0, 4
    //   HALT
    let code = vec![
        // Setup dest and src pointers
        encode(&Instruction::Addi {
            rd: Register::A0,
            rs1: Register::SP,
            imm: -64,
        }),
        encode(&Instruction::Addi {
            rd: Register::A1,
            rs1: Register::SP,
            imm: -128,
        }),
        encode(&Instruction::Addi {
            rd: Register::A2,
            rs1: Register::ZERO,
            imm: 4,
        }),
        // Store test data
        encode(&Instruction::Addi {
            rd: Register::R5, // t0
            rs1: Register::ZERO,
            imm: 100,
        }),
        encode(&Instruction::Sw {
            rs1: Register::A1,
            rs2: Register::R5,
            imm: 0,
        }),
        encode(&Instruction::Addi {
            rd: Register::R5,
            rs1: Register::ZERO,
            imm: 200,
        }),
        encode(&Instruction::Sw {
            rs1: Register::A1,
            rs2: Register::R5,
            imm: 4,
        }),
        // Call memcpy
        encode(&Instruction::Addi {
            rd: Register::A7,
            rs1: Register::ZERO,
            imm: syscall_nums::SYS_MEMCPY as i16,
        }),
        encode(&Instruction::Ecall),
        // Load from dest
        encode(&Instruction::Lw {
            rd: Register::A3,
            rs1: Register::A0,
            imm: 0,
        }),
        encode(&Instruction::Lw {
            rd: Register::A4,
            rs1: Register::A0,
            imm: 4,
        }),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.cycles, 12);
}

#[test]
fn test_syscall_memset() {
    // Program:
    //   ADDI a0, sp, -64  ; dest
    //   ADDI a1, zero, 42 ; value
    //   ADDI a2, zero, 4  ; length (4 words)
    //   ADDI a7, zero, SYS_MEMSET
    //   ECALL
    //   ; Load from dest to verify
    //   LW a3, a0, 0
    //   LW a4, a0, 4
    //   HALT
    let code = vec![
        encode(&Instruction::Addi {
            rd: Register::A0,
            rs1: Register::SP,
            imm: -64,
        }),
        encode(&Instruction::Addi {
            rd: Register::A1,
            rs1: Register::ZERO,
            imm: 42,
        }),
        encode(&Instruction::Addi {
            rd: Register::A2,
            rs1: Register::ZERO,
            imm: 4,
        }),
        encode(&Instruction::Addi {
            rd: Register::A7,
            rs1: Register::ZERO,
            imm: syscall_nums::SYS_MEMSET as i16,
        }),
        encode(&Instruction::Ecall),
        encode(&Instruction::Lw {
            rd: Register::A3,
            rs1: Register::A0,
            imm: 0,
        }),
        encode(&Instruction::Lw {
            rd: Register::A4,
            rs1: Register::A0,
            imm: 4,
        }),
        encode(&Instruction::Halt),
    ];
    let program = Program::new(code);

    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.cycles, 8);
}
