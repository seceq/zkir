//! Stress tests for ZKIR
//!
//! Tests with large programs, many iterations, and edge cases.

use zkir_assembler::assemble;
use zkir_runtime::{VM, VMConfig, HaltReason};
use zkir_spec::{Instruction, Program, Register};

fn create_program_from_instructions(instructions: Vec<Instruction>) -> Program {
    let mut program = Program::new();
    let code: Vec<u32> = instructions
        .iter()
        .map(|inst| zkir_assembler::encode(inst))
        .collect();
    program.code = code;
    program.header.code_size = (program.code.len() * 4) as u32;
    program
}

// ============================================================================
// Large Program Tests
// ============================================================================

#[test]
fn test_1000_instructions() {
    let mut instructions = vec![];

    // Generate 1000 add instructions
    for _ in 0..1000 {
        instructions.push(Instruction::Add {
            rd: Register::R1,
            rs1: Register::R1,
            rs2: Register::R0,
        });
    }

    // Exit
    instructions.push(Instruction::Addi {
        rd: Register::R10,
        rs1: Register::R0,
        imm: 0,
    });
    instructions.push(Instruction::Addi {
        rd: Register::R11,
        rs1: Register::R0,
        imm: 0,
    });
    instructions.push(Instruction::Ecall);

    let program = create_program_from_instructions(instructions);
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Exit(0));
    assert_eq!(result.cycles, 1003); // 1000 adds + 2 addi + ecall
}

#[test]
fn test_many_labels_program() {
    let mut source = String::new();

    // Generate program with 100 labels
    for i in 0..100 {
        source.push_str(&format!("label{}:\n", i));
        source.push_str("    add r1, r1, r0\n");
    }
    source.push_str("    ecall\n");

    let program = assemble(&source).unwrap();
    assert_eq!(program.code.len(), 101);
}

// ============================================================================
// Long Running Tests
// ============================================================================

#[test]
fn test_tight_loop_many_iterations() {
    // Loop 10000 times
    // Syscall convention: R10 = syscall number, R11 = first arg
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        addi r1, zero, 0        # counter
        addi r2, zero, 10000    # limit

    loop:
        addi r1, r1, 1          # counter++
        bne r1, r2, -4          # loop if counter != limit

        # Exit
        addi t2, zero, 0        # syscall: exit (R10)
        addi a0, zero, 0        # exit code (R11)
        ecall
    "#;

    let program = assemble(source).unwrap();
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Exit(0));
}

#[test]
fn test_nested_loops() {
    // Double nested loop: 100 * 100 = 10000 iterations
    // Syscall convention: R10 = syscall number, R11 = first arg
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        addi r1, zero, 0        # outer counter
        addi r3, zero, 100      # limit

    outer:
        addi r2, zero, 0        # inner counter

    inner:
        addi r2, r2, 1          # inner++
        bne r2, r3, -4          # inner loop

        addi r1, r1, 1          # outer++
        bne r1, r3, -16         # outer loop

        # Exit
        addi t2, zero, 0        # syscall: exit (R10)
        addi a0, zero, 0        # exit code (R11)
        ecall
    "#;

    let program = assemble(source).unwrap();
    let mut config = VMConfig::default();
    config.max_cycles = 100_000; // Allow more cycles

    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Exit(0));
}

// ============================================================================
// Cycle Limit Tests
// ============================================================================

#[test]
fn test_cycle_limit_enforcement() {
    // Infinite loop
    let source = r#"
    loop:
        jal zero, 0    # Jump to self (infinite loop)
    "#;

    let program = assemble(source).unwrap();
    let mut config = VMConfig::default();
    config.max_cycles = 100;

    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::CycleLimit);
    assert_eq!(result.cycles, 100);
}

#[test]
fn test_cycle_limit_exact() {
    // Program that runs exactly N cycles
    let mut instructions = vec![];

    // 50 NOPs (add r0, r0, r0)
    for _ in 0..50 {
        instructions.push(Instruction::Add {
            rd: Register::R0,
            rs1: Register::R0,
            rs2: Register::R0,
        });
    }

    instructions.push(Instruction::Ebreak);

    let program = create_program_from_instructions(instructions);
    let mut config = VMConfig::default();
    config.max_cycles = 100; // Enough cycles

    let vm = VM::new(program, vec![], config);
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Ebreak);
    assert_eq!(result.cycles, 51);
}

// ============================================================================
// Memory Stress Tests
// ============================================================================

#[test]
fn test_many_memory_operations() {
    // Store and load 100 values
    let mut source = String::new();

    source.push_str("    addi r1, zero, 0x1000    # base address\n");
    source.push_str("    addi r2, zero, 1         # value\n");

    // Store 100 values
    for i in 0..100 {
        let offset = i * 4;
        source.push_str(&format!("    sw r2, {}(r1)\n", offset));
        source.push_str("    addi r2, r2, 1\n");
    }

    source.push_str("    addi t2, zero, 0\n");  // syscall: exit (R10)
    source.push_str("    addi a0, zero, 0\n");  // exit code (R11)
    source.push_str("    ecall\n");

    let program = assemble(&source).unwrap();
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Exit(0));
}

#[test]
fn test_sparse_memory_access() {
    // Access memory at widely separated addresses
    // Syscall convention: R10 = syscall number, R11 = first arg
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        addi r1, zero, 42

        # Store at various addresses
        addi r2, zero, 0x1000
        sw r1, 0(r2)

        addi r2, zero, 0x2000
        sw r1, 0(r2)

        addi r2, zero, 0x3000
        sw r1, 0(r2)

        # Exit
        addi t2, zero, 0        # syscall: exit (R10)
        addi a0, zero, 0        # exit code (R11)
        ecall
    "#;

    let program = assemble(source).unwrap();
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Exit(0));
}

// ============================================================================
// Arithmetic Stress Tests
// ============================================================================

#[test]
fn test_repeated_multiplication() {
    // Multiply repeatedly (tests bound accumulation)
    let mut instructions = vec![];

    // r1 = 2
    instructions.push(Instruction::Addi {
        rd: Register::R1,
        rs1: Register::R0,
        imm: 2,
    });

    // r2 = 1
    instructions.push(Instruction::Addi {
        rd: Register::R2,
        rs1: Register::R0,
        imm: 1,
    });

    // Multiply r2 by 2, 20 times (2^20 = 1048576)
    for _ in 0..20 {
        instructions.push(Instruction::Mul {
            rd: Register::R2,
            rs1: Register::R2,
            rs2: Register::R1,
        });
    }

    // Exit
    instructions.push(Instruction::Addi {
        rd: Register::R10,
        rs1: Register::R0,
        imm: 0,
    });
    instructions.push(Instruction::Addi {
        rd: Register::R11,
        rs1: Register::R0,
        imm: 0,
    });
    instructions.push(Instruction::Ecall);

    let program = create_program_from_instructions(instructions);
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Exit(0));
}

#[test]
fn test_all_arithmetic_ops() {
    // Syscall convention: R10 = syscall number, R11 = first arg
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        addi r1, zero, 100
        addi r2, zero, 7

        # Test all arithmetic operations
        add r3, r1, r2      # 107
        sub r4, r1, r2      # 93
        mul r5, r1, r2      # 700
        divu r6, r1, r2     # 14
        remu r7, r1, r2     # 2

        # Exit
        addi t2, zero, 0        # syscall: exit (R10)
        addi a0, zero, 0        # exit code (R11)
        ecall
    "#;

    let program = assemble(source).unwrap();
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Exit(0));
}

// ============================================================================
// Branch Stress Tests
// ============================================================================

#[test]
fn test_many_branches() {
    // Program with many branch instructions
    // Syscall convention: R10 = syscall number, R11 = first arg
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let mut source = String::new();

    source.push_str("    addi r1, zero, 0\n");
    source.push_str("    addi r2, zero, 1\n");

    // 50 conditional branches (all taken)
    for _ in 0..50 {
        source.push_str("    bne r1, r2, 4\n"); // Branch over next instruction
        source.push_str("    add r1, r1, r1\n"); // Should be skipped
    }

    source.push_str("    addi t2, zero, 0\n");  // syscall: exit (R10)
    source.push_str("    addi a0, zero, 0\n");  // exit code (R11)
    source.push_str("    ecall\n");

    let program = assemble(&source).unwrap();
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Exit(0));
}

#[test]
fn test_alternating_branches() {
    // Branches that alternate between taken and not taken
    // Syscall convention: R10 = syscall number, R11 = first arg
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        addi r1, zero, 1
        addi r2, zero, 0
        addi r3, zero, 50   # iterations

    loop:
        # Swap r1 and r2 using r4
        addi r4, r1, 0
        addi r1, r2, 0
        addi r2, r4, 0

        # Decrement counter
        addi r3, r3, -1
        bne r3, zero, -16       # branch back to loop: (4 instructions * 4 bytes)

        addi t2, zero, 0        # syscall: exit (R10)
        addi a0, zero, 0        # exit code (R11)
        ecall
    "#;

    let program = assemble(source).unwrap();
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Exit(0));
}

// ============================================================================
// IO Stress Tests
// ============================================================================

#[test]
fn test_many_io_operations() {
    // Syscall convention: R10 = syscall number, R11 = first arg, return in R10
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        # Read 5 inputs and write them back
        addi r3, zero, 5    # count

    loop:
        # Read
        addi t2, zero, 1        # syscall: read (R10)
        ecall

        # Write (t2 now has the value from read)
        addi a0, t2, 0          # a0 = value (R11)
        addi t2, zero, 2        # syscall: write (R10)
        ecall

        # Decrement
        addi r3, r3, -1
        bne r3, zero, -24       # branch back to loop: (6 instructions * 4 bytes)

        # Exit
        addi t2, zero, 0        # syscall: exit (R10)
        addi a0, zero, 0        # exit code (R11)
        ecall
    "#;

    let program = assemble(source).unwrap();
    let inputs = vec![1, 2, 3, 4, 5];
    let vm = VM::new(program, inputs, VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.outputs, vec![1, 2, 3, 4, 5]);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_division_by_one() {
    // Syscall convention: R10 = syscall number, R11 = first arg
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        addi r1, zero, 12345
        addi r2, zero, 1
        divu r3, r1, r2

        # Write result
        addi a0, r3, 0          # a0 = result (R11)
        addi t2, zero, 2        # syscall: write (R10)
        ecall

        addi t2, zero, 0        # syscall: exit (R10)
        addi a0, zero, 0        # exit code (R11)
        ecall
    "#;

    let program = assemble(source).unwrap();
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.outputs, vec![12345]);
}

#[test]
fn test_self_modifying_registers() {
    // Operations where rd == rs1 or rd == rs2
    // Syscall convention: R10 = syscall number, R11 = first arg
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        addi r1, zero, 10

        # r1 = r1 + r1
        add r1, r1, r1      # 20

        # r1 = r1 + r1
        add r1, r1, r1      # 40

        # r1 = r1 + r1
        add r1, r1, r1      # 80

        # Write result
        addi a0, r1, 0          # a0 = result (R11)
        addi t2, zero, 2        # syscall: write (R10)
        ecall

        addi t2, zero, 0        # syscall: exit (R10)
        addi a0, zero, 0        # exit code (R11)
        ecall
    "#;

    let program = assemble(source).unwrap();
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.outputs, vec![80]);
}

#[test]
fn test_zero_register_destination() {
    // Writing to zero register should have no effect
    // Syscall convention: R10 = syscall number, R11 = first arg
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        addi zero, zero, 100    # Should be ignored

        # Write zero (should still be 0)
        addi a0, zero, 0        # a0 = 0 (R11)
        addi t2, zero, 2        # syscall: write (R10)
        ecall

        addi t2, zero, 0        # syscall: exit (R10)
        addi a0, zero, 0        # exit code (R11)
        ecall
    "#;

    let program = assemble(source).unwrap();
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.outputs, vec![0]);
}
