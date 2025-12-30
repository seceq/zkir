//! Cross-module interaction tests
//!
//! Tests the integration between assembler, disassembler, and runtime.

use zkir_assembler::assemble;
use zkir_disassembler::{disassemble, decode};
use zkir_runtime::{VM, VMConfig, HaltReason};
use zkir_spec::{Instruction, Program, Register, Opcode, Config};

// ============================================================================
// Assembler -> Runtime Tests
// ============================================================================

#[test]
fn test_assembled_program_runs() {
    // Syscall convention: R10 = syscall number, R11 = first arg
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        addi t2, zero, 0    # syscall: exit (R10)
        addi a0, zero, 42   # exit code (R11)
        ecall
    "#;

    let program = assemble(source).unwrap();
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Exit(42));
}

#[test]
fn test_assembled_io_program() {
    // Syscall convention: R10 = syscall number, R11 = first arg, return in R10
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        # Read input
        addi t2, zero, 1    # syscall: read (R10)
        ecall               # result in t2 (R10)

        # Write output (copy t2 to a0)
        addi a0, t2, 0      # a0 = t2 (input value, R11)
        addi t2, zero, 2    # syscall: write (R10)
        ecall

        # Exit
        addi t2, zero, 0    # syscall: exit (R10)
        addi a0, zero, 0    # exit code 0 (R11)
        ecall
    "#;

    let program = assemble(source).unwrap();
    let vm = VM::new(program, vec![123], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.outputs, vec![123]);
    assert_eq!(result.halt_reason, HaltReason::Exit(0));
}

#[test]
fn test_assembled_arithmetic() {
    // Syscall convention: R10 = syscall number, R11 = first arg
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        # Compute 10 + 20 + 30
        addi r1, zero, 10
        addi r2, zero, 20
        addi r3, zero, 30
        add r4, r1, r2      # r4 = 30
        add r4, r4, r3      # r4 = 60

        # Write result
        addi a0, r4, 0      # a0 = result (R11)
        addi t2, zero, 2    # syscall: write (R10)
        ecall

        # Exit
        addi t2, zero, 0    # syscall: exit (R10)
        addi a0, zero, 0    # exit code (R11)
        ecall
    "#;

    let program = assemble(source).unwrap();
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.outputs, vec![60]);
}

// ============================================================================
// Runtime -> Disassembler Tests
// ============================================================================

#[test]
fn test_executed_program_disassembles() {
    let source = r#"
        add r1, r2, r3
        sub r4, r5, r6
        ecall
    "#;

    let program = assemble(source).unwrap();

    // Execute it
    let vm = VM::new(program.clone(), vec![], VMConfig::default());
    let _result = vm.run();

    // Now disassemble the original program
    let asm = disassemble(&program).unwrap();

    assert!(asm.contains("add"));
    assert!(asm.contains("sub"));
    assert!(asm.contains("ecall"));
}

// ============================================================================
// Full Roundtrip Tests (Assemble -> Execute -> Disassemble)
// ============================================================================

#[test]
fn test_full_roundtrip_simple() {
    let source = "ecall";

    // Assemble
    let program = assemble(source).unwrap();

    // Execute (will just exit)
    let vm = VM::new(program.clone(), vec![], VMConfig::default());
    let _result = vm.run();

    // Disassemble
    let asm = disassemble(&program).unwrap();

    assert!(asm.contains("ecall"));
}

#[test]
fn test_full_roundtrip_complex() {
    // Syscall convention: R10 = syscall number, R11 = first arg
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        .config limb_bits 20
        .config data_limbs 2

        # Fibonacci sequence - compute f(5)
        addi r1, zero, 0      # f(0) = 0
        addi r2, zero, 1      # f(1) = 1
        addi r3, zero, 4      # counter (4 iterations: f(2), f(3), f(4), f(5))

    loop:
        add r4, r1, r2        # f(n) = f(n-1) + f(n-2)
        addi r1, r2, 0        # f(n-2) = f(n-1)
        addi r2, r4, 0        # f(n-1) = f(n)
        addi r3, r3, -1       # counter--
        bne r3, zero, -16     # loop if counter != 0

        # Write result
        addi a0, r2, 0        # a0 = result (R11)
        addi t2, zero, 2      # syscall: write (R10)
        ecall

        # Exit
        addi t2, zero, 0      # syscall: exit (R10)
        addi a0, zero, 0      # exit code (R11)
        ecall
    "#;

    // Assemble
    let program = assemble(source).unwrap();

    // Execute
    let vm = VM::new(program.clone(), vec![], VMConfig::default());
    let result = vm.run().unwrap();

    // Verify Fibonacci result: f(5) = 5
    assert_eq!(result.outputs, vec![5]);

    // Disassemble
    let asm = disassemble(&program).unwrap();

    // Should contain key instructions
    assert!(asm.contains("addi"));
    assert!(asm.contains("add"));
    assert!(asm.contains("bne"));
    assert!(asm.contains("ecall"));

    // Should show config
    assert!(asm.contains("20"));
}

// ============================================================================
// Encode/Decode Roundtrip Tests
// ============================================================================

#[test]
fn test_encode_decode_all_instructions() {
    let instructions = vec![
        Instruction::Add { rd: Register::R1, rs1: Register::R2, rs2: Register::R3 },
        Instruction::Sub { rd: Register::R4, rs1: Register::R5, rs2: Register::R6 },
        Instruction::Mul { rd: Register::R7, rs1: Register::R8, rs2: Register::R9 },
        Instruction::Div { rd: Register::R10, rs1: Register::R11, rs2: Register::R12 },
        Instruction::And { rd: Register::R1, rs1: Register::R2, rs2: Register::R3 },
        Instruction::Or { rd: Register::R1, rs1: Register::R2, rs2: Register::R3 },
        Instruction::Xor { rd: Register::R1, rs1: Register::R2, rs2: Register::R3 },
        Instruction::Sll { rd: Register::R1, rs1: Register::R2, rs2: Register::R3 },
        Instruction::Srl { rd: Register::R1, rs1: Register::R2, rs2: Register::R3 },
        Instruction::Sra { rd: Register::R1, rs1: Register::R2, rs2: Register::R3 },
        Instruction::Addi { rd: Register::R1, rs1: Register::R2, imm: 100 },
        Instruction::Addi { rd: Register::R1, rs1: Register::R2, imm: -100 },
        Instruction::Andi { rd: Register::R1, rs1: Register::R2, imm: 0xFF },
        Instruction::Slli { rd: Register::R1, rs1: Register::R2, shamt: 5 },
        Instruction::Lw { rd: Register::R1, rs1: Register::R2, imm: 16 },
        Instruction::Sw { rs1: Register::R2, rs2: Register::R1, imm: 16 },
        Instruction::Beq { rs1: Register::R1, rs2: Register::R2, offset: 8 },
        Instruction::Bne { rs1: Register::R1, rs2: Register::R2, offset: -8 },
        Instruction::Jal { rd: Register::R1, offset: 100 },
        Instruction::Jalr { rd: Register::R1, rs1: Register::R2, imm: 50 },
        Instruction::Ecall,
        Instruction::Ebreak,
    ];

    for original in instructions {
        let encoded = zkir_assembler::encode(&original);
        let decoded = decode(encoded).unwrap();
        assert_eq!(decoded, original, "Roundtrip failed for {:?}", original);
    }
}

#[test]
fn test_encode_decode_edge_immediates() {
    // Test edge case immediate values
    let test_immediates = vec![
        0,
        1,
        -1,
        127,
        -128,
        255,
        -256,
        32767,
        -32768,
        65535,
        -65536,
    ];

    for imm in test_immediates {
        let original = Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R2,
            imm,
        };

        let encoded = zkir_assembler::encode(&original);
        let decoded = decode(encoded).unwrap();
        assert_eq!(decoded, original, "Failed for immediate {}", imm);
    }
}

// ============================================================================
// Error Propagation Tests
// ============================================================================

#[test]
fn test_assemble_error_does_not_crash_runtime() {
    let source = "invalid instruction";
    let result = assemble(source);

    assert!(result.is_err());
    // Error should be AssemblerError, not a panic
}

#[test]
fn test_decode_error_does_not_crash_disassembler() {
    let invalid_opcode = 0x7F as u32;
    let result = decode(invalid_opcode);

    assert!(result.is_err());
    // Error should be DisassemblerError, not a panic
}

#[test]
fn test_disassemble_with_invalid_instruction() {
    let mut program = Program::new();
    // Mix valid and invalid encodings
    program.code = vec![
        Opcode::Add.to_u8() as u32 | (1 << 7) | (2 << 11) | (3 << 15),
        0x7F, // Invalid opcode
        Opcode::Ecall.to_u8() as u32,
    ];
    program.header.code_size = 12;

    let asm = disassemble(&program).unwrap();

    // Should still produce output, marking the invalid instruction
    assert!(asm.contains("add"));
    assert!(asm.contains("ERROR") || asm.contains("Unknown") || asm.contains("Invalid"));
    assert!(asm.contains("ecall"));
}

// ============================================================================
// Configuration Consistency Tests
// ============================================================================

#[test]
fn test_config_preserved_through_pipeline() {
    let source = r#"
        .config limb_bits 30
        .config data_limbs 3
        .config addr_limbs 2

        ecall
    "#;

    // Assemble
    let program = assemble(source).unwrap();

    // Verify config
    let config = program.config();
    assert_eq!(config.limb_bits, 30);
    assert_eq!(config.data_limbs, 3);
    assert_eq!(config.addr_limbs, 2);

    // Disassemble
    let asm = disassemble(&program).unwrap();

    // Config should be shown
    assert!(asm.contains("30"));
    assert!(asm.contains("90-bit")); // 30 * 3 = 90
}

// ============================================================================
// Memory Operation Tests
// ============================================================================

#[test]
fn test_memory_roundtrip() {
    // Syscall convention: R10 = syscall number, R11 = first arg
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        # Store a value to memory
        addi r1, zero, 42       # value
        addi r2, zero, 0x1000   # address
        sw r1, 0(r2)            # store

        # Load it back
        lw r3, 0(r2)            # load

        # Write result
        addi a0, r3, 0          # a0 = result (R11)
        addi t2, zero, 2        # syscall: write (R10)
        ecall

        # Exit
        addi t2, zero, 0        # syscall: exit (R10)
        addi a0, zero, 0        # exit code (R11)
        ecall
    "#;

    let program = assemble(source).unwrap();
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.outputs, vec![42]);
}

// ============================================================================
// Branch/Jump Tests
// ============================================================================

#[test]
fn test_conditional_branch() {
    // Syscall convention: R10 = syscall number, R11 = first arg
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        addi r1, zero, 10
        addi r2, zero, 10

        # If r1 == r2, skip next instruction
        beq r1, r2, 8

        # This should be skipped
        addi r3, zero, 1

        # This should execute
        addi r4, zero, 2

        # Write r3 (should be 0)
        addi a0, r3, 0          # a0 = r3 (R11)
        addi t2, zero, 2        # syscall: write (R10)
        ecall

        # Exit
        addi t2, zero, 0        # syscall: exit (R10)
        addi a0, zero, 0        # exit code (R11)
        ecall
    "#;

    let program = assemble(source).unwrap();
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    // r3 should be 0 (instruction was skipped)
    assert_eq!(result.outputs, vec![0]);
}

#[test]
fn test_loop_with_branch() {
    // Syscall convention: R10 = syscall number, R11 = first arg
    // t2 maps to R10, a0 maps to R11 in the assembler ABI
    let source = r#"
        # Sum 1 to 5
        addi r1, zero, 0    # sum = 0
        addi r2, zero, 1    # i = 1
        addi r3, zero, 6    # limit = 6

    loop:
        add r1, r1, r2      # sum += i
        addi r2, r2, 1      # i++
        bne r2, r3, -8      # loop if i != limit

        # Write sum
        addi a0, r1, 0      # a0 = sum (R11)
        addi t2, zero, 2    # syscall: write (R10)
        ecall

        # Exit
        addi t2, zero, 0    # syscall: exit (R10)
        addi a0, zero, 0    # exit code (R11)
        ecall
    "#;

    let program = assemble(source).unwrap();
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    // Sum of 1+2+3+4+5 = 15
    assert_eq!(result.outputs, vec![15]);
}

// ============================================================================
// Trace Collection Tests
// ============================================================================

#[test]
fn test_trace_roundtrip() {
    let source = r#"
        addi r1, zero, 100
        addi r2, zero, 200
        add r3, r1, r2
        ebreak
    "#;

    let program = assemble(source).unwrap();

    let mut config = VMConfig::default();
    config.enable_execution_trace = true;

    let vm = VM::new(program.clone(), vec![], config);
    let result = vm.run().unwrap();

    // Should have trace entries
    assert_eq!(result.execution_trace.len(), 4);

    // Each entry should have valid structure
    for (i, row) in result.execution_trace.iter().enumerate() {
        assert_eq!(row.cycle, i as u64);
        assert_eq!(row.registers.len(), 16);
    }
}
