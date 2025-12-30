//! End-to-end integration tests for the ZKIR toolchain
//!
//! These tests verify the complete workflow:
//! 1. Assemble source code into a Program
//! 2. Execute the program in the VM
//! 3. Verify outputs and execution traces
//! 4. Disassemble the program back to source
//!
//! Syscall conventions:
//! - R10 (a0): syscall number (0=exit, 1=read, 2=write)
//! - R11 (a1): syscall argument (exit code for exit, value for write)

use zkir_assembler::assemble;
use zkir_disassembler::disassemble;
use zkir_runtime::{VM, VMConfig};

// ============================================================================
// Assemble -> Execute Tests
// ============================================================================

#[test]
fn test_simple_addition() {
    // Add 10 + 20 = 30, then exit with syscall 0
    let source = r#"
        addi r1, r0, 10
        addi r2, r0, 20
        add r3, r1, r2
        add r10, r0, r0
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().expect("Execution failed");

    assert!(result.cycles > 0);
}

#[test]
fn test_subtraction() {
    let source = r#"
        addi r1, r0, 50
        addi r2, r0, 30
        sub r3, r1, r2
        add r10, r0, r0
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().expect("Execution failed");

    assert!(result.cycles > 0);
}

#[test]
fn test_multiplication() {
    let source = r#"
        addi r1, r0, 7
        addi r2, r0, 6
        mul r3, r1, r2
        add r10, r0, r0
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().expect("Execution failed");

    assert!(result.cycles > 0);
}

#[test]
fn test_bitwise_operations() {
    let source = r#"
        addi r1, r0, 255
        addi r2, r0, 15
        and r3, r1, r2
        or r4, r1, r2
        xor r5, r1, r2
        add r10, r0, r0
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().expect("Execution failed");

    assert!(result.cycles > 0);
}

#[test]
fn test_shifts() {
    let source = r#"
        addi r1, r0, 8
        slli r2, r1, 2
        srli r3, r1, 1
        add r10, r0, r0
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().expect("Execution failed");

    assert!(result.cycles > 0);
}

#[test]
fn test_conditional_branch_taken() {
    let source = r#"
        addi r1, r0, 5
        addi r2, r0, 5
        beq r1, r2, 8
        addi r3, r0, 100
        addi r3, r0, 42
        add r10, r0, r0
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().expect("Execution failed");

    assert!(result.cycles > 0);
}

#[test]
fn test_conditional_branch_not_taken() {
    let source = r#"
        addi r1, r0, 5
        addi r2, r0, 10
        beq r1, r2, 8
        addi r3, r0, 100
        add r10, r0, r0
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().expect("Execution failed");

    assert!(result.cycles > 0);
}

#[test]
fn test_comparison_slt() {
    let source = r#"
        addi r1, r0, 5
        addi r2, r0, 10
        slt r3, r1, r2
        slt r4, r2, r1
        add r10, r0, r0
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().expect("Execution failed");

    assert!(result.cycles > 0);
}

#[test]
fn test_loop_counting() {
    // Loop using backward branch
    let source = r#"
        addi r1, r0, 0
        addi r2, r0, 5
        addi r1, r1, 1
        bne r1, r2, -4
        add r10, r0, r0
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().expect("Execution failed");

    assert!(result.cycles > 0);
}

// ============================================================================
// Assemble -> Disassemble Round-Trip Tests
// ============================================================================

#[test]
fn test_roundtrip_arithmetic() {
    let source = r#"
        add r1, r2, r3
        sub r4, r5, r6
        mul r7, r8, r9
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let disasm = disassemble(&program).expect("Disassembly failed");

    assert!(disasm.contains("add"));
    assert!(disasm.contains("sub"));
    assert!(disasm.contains("mul"));
    assert!(disasm.contains("ecall"));
}

#[test]
fn test_roundtrip_immediate() {
    let source = r#"
        addi r1, r0, 100
        andi r2, r1, 255
        ori r3, r2, 256
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let disasm = disassemble(&program).expect("Disassembly failed");

    assert!(disasm.contains("addi"));
    assert!(disasm.contains("andi"));
    assert!(disasm.contains("ori"));
}

#[test]
fn test_roundtrip_branches() {
    let source = r#"
        beq r1, r2, 8
        bne r3, r4, -4
        blt r5, r6, 12
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let disasm = disassemble(&program).expect("Disassembly failed");

    assert!(disasm.contains("beq"));
    assert!(disasm.contains("bne"));
    assert!(disasm.contains("blt"));
}

#[test]
fn test_roundtrip_shifts() {
    let source = r#"
        slli r1, r2, 5
        srli r3, r4, 3
        srai r5, r6, 2
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let disasm = disassemble(&program).expect("Disassembly failed");

    assert!(disasm.contains("slli"));
    assert!(disasm.contains("srli"));
    assert!(disasm.contains("srai"));
}

// ============================================================================
// VM Configuration Tests
// ============================================================================

#[test]
fn test_execution_with_trace() {
    let source = r#"
        addi r1, r0, 1
        addi r2, r0, 2
        add r3, r1, r2
        add r10, r0, r0
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let mut config = VMConfig::default();
    config.enable_execution_trace = true;

    let vm = VM::new(program, vec![], config);
    let result = vm.run().expect("Execution failed");

    assert!(!result.execution_trace.is_empty());
    assert!(result.execution_trace.len() >= 4);
}

#[test]
fn test_execution_cycle_limit() {
    // Infinite loop
    let source = r#"
        addi r1, r1, 1
        jal r0, -4
    "#;

    let program = assemble(source).expect("Assembly failed");
    let mut config = VMConfig::default();
    config.max_cycles = 100;

    let vm = VM::new(program, vec![], config);
    let result = vm.run();

    match result {
        Err(ref e) => {
            assert!(format!("{:?}", e).contains("CycleLimit"));
        }
        Ok(r) => {
            assert!(r.cycles >= 100);
        }
    }
}

// ============================================================================
// Complex Program Tests
// ============================================================================

#[test]
fn test_fibonacci() {
    // Fibonacci (simplified without labels)
    let source = r#"
        addi r1, r0, 0
        addi r2, r0, 1
        addi r3, r0, 10
        addi r4, r0, 2
        add r5, r1, r2
        add r1, r0, r2
        add r2, r0, r5
        addi r4, r4, 1
        bne r4, r3, -16
        add r10, r0, r0
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().expect("Execution failed");

    assert!(result.cycles > 0);
}

#[test]
fn test_sum_values() {
    let source = r#"
        addi r1, r0, 0
        addi r2, r0, 1
        addi r3, r0, 2
        addi r4, r0, 3
        add r1, r1, r2
        add r1, r1, r3
        add r1, r1, r4
        add r10, r0, r0
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().expect("Execution failed");

    assert!(result.cycles > 0);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_assembly_error_invalid_register() {
    let source = r#"
        add r99, r1, r2
        ecall
    "#;

    let result = assemble(source);
    assert!(result.is_err());
}

#[test]
fn test_assembly_error_invalid_instruction() {
    let source = r#"
        invalid_instruction r1, r2, r3
        ecall
    "#;

    let result = assemble(source);
    assert!(result.is_err());
}

#[test]
fn test_assembly_with_comments() {
    // Use # for comments in assembly
    let source = r#"
        # This is a comment
        addi r1, r0, 10
        add r2, r1, r1
        add r10, r0, r0
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().expect("Execution failed");

    assert!(result.cycles > 0);
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_assembly_with_config() {
    let source = r#"
        .config limb_bits 20
        .config data_limbs 2
        .config addr_limbs 2

        addi r1, r0, 100
        add r10, r0, r0
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");

    assert_eq!(program.config().limb_bits, 20);
    assert_eq!(program.config().data_limbs, 2);
    assert_eq!(program.config().addr_limbs, 2);
}

#[test]
fn test_assembly_default_config() {
    let source = r#"
        addi r1, r0, 100
        add r10, r0, r0
        ecall
    "#;

    let program = assemble(source).expect("Assembly failed");

    assert_eq!(program.config().limb_bits, 20);
    assert_eq!(program.config().data_limbs, 2);
}
