//! Integration tests for ZKIR Assembler v3.4
//!
//! Tests the complete assembly workflow including:
//! - Instruction parsing and encoding
//! - Label resolution
//! - Configuration directives
//! - Error handling for malformed input

use zkir_assembler::{assemble, encode, parse_register, AssemblerError};
use zkir_spec::{Instruction, Register, Opcode};

// ============================================================================
// Basic Assembly Tests
// ============================================================================

#[test]
fn test_assemble_empty_program() {
    let source = "";
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 0);
}

#[test]
fn test_assemble_comments_only() {
    let source = r#"
        # This is a comment
        # Another comment
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 0);
}

#[test]
fn test_assemble_single_instruction() {
    let source = "ecall";
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 1);
}

#[test]
fn test_assemble_multiple_instructions() {
    let source = r#"
        add r1, r2, r3
        sub r4, r5, r6
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 3);
}

// ============================================================================
// R-type Instruction Tests
// ============================================================================

#[test]
fn test_assemble_all_r_type_arithmetic() {
    let source = r#"
        add r1, r2, r3
        sub r1, r2, r3
        mul r1, r2, r3
        mulh r1, r2, r3
        div r1, r2, r3
        divu r1, r2, r3
        rem r1, r2, r3
        remu r1, r2, r3
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 9);
}

#[test]
fn test_assemble_all_r_type_logical() {
    let source = r#"
        and r1, r2, r3
        or r1, r2, r3
        xor r1, r2, r3
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 4);
}

#[test]
fn test_assemble_all_r_type_shift() {
    let source = r#"
        sll r1, r2, r3
        srl r1, r2, r3
        sra r1, r2, r3
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 4);
}

#[test]
fn test_assemble_all_r_type_compare() {
    let source = r#"
        slt r1, r2, r3
        sltu r1, r2, r3
        sge r1, r2, r3
        sgeu r1, r2, r3
        seq r1, r2, r3
        sne r1, r2, r3
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 7);
}

#[test]
fn test_assemble_all_r_type_cmov() {
    let source = r#"
        cmov r1, r2, r3
        cmovz r1, r2, r3
        cmovnz r1, r2, r3
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 4);
}

// ============================================================================
// I-type Instruction Tests
// ============================================================================

#[test]
fn test_assemble_all_i_type_arithmetic() {
    let source = r#"
        addi r1, r2, 100
        addi r1, r2, -100
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 3);
}

#[test]
fn test_assemble_all_i_type_logical() {
    let source = r#"
        andi r1, r2, 0xFF
        ori r1, r2, 0xFF
        xori r1, r2, 0xFF
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 4);
}

#[test]
fn test_assemble_shift_immediate() {
    let source = r#"
        slli r1, r2, 5
        srli r1, r2, 5
        srai r1, r2, 5
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 4);
}

// ============================================================================
// Load/Store Instruction Tests
// ============================================================================

#[test]
fn test_assemble_all_loads() {
    let source = r#"
        lb r1, 0(r2)
        lbu r1, 0(r2)
        lh r1, 0(r2)
        lhu r1, 0(r2)
        lw r1, 0(r2)
        ld r1, 0(r2)
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 7);
}

#[test]
fn test_assemble_all_stores() {
    let source = r#"
        sb r1, 0(r2)
        sh r1, 0(r2)
        sw r1, 0(r2)
        sd r1, 0(r2)
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 5);
}

#[test]
fn test_assemble_load_with_offset() {
    let source = r#"
        lw r1, 100(r2)
        lw r1, -100(r2)
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 3);
}

// ============================================================================
// Branch Instruction Tests
// ============================================================================

#[test]
fn test_assemble_all_branches() {
    let source = r#"
        beq r1, r2, 8
        bne r1, r2, 8
        blt r1, r2, 8
        bge r1, r2, 8
        bltu r1, r2, 8
        bgeu r1, r2, 8
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 7);
}

#[test]
fn test_assemble_branch_negative_offset() {
    let source = r#"
        beq r1, r2, -8
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 2);
}

// ============================================================================
// Jump Instruction Tests
// ============================================================================

#[test]
fn test_assemble_jal() {
    let source = r#"
        jal r1, 100
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 2);
}

#[test]
fn test_assemble_jalr() {
    let source = r#"
        jalr r1, r2, 100
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 2);
}

// ============================================================================
// System Instruction Tests
// ============================================================================

#[test]
fn test_assemble_ecall_ebreak() {
    let source = r#"
        ecall
        ebreak
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 2);

    // Verify opcodes
    assert_eq!(program.code[0] & 0x7F, Opcode::Ecall.to_u8() as u32);
    assert_eq!(program.code[1] & 0x7F, Opcode::Ebreak.to_u8() as u32);
}

// ============================================================================
// Label Tests
// ============================================================================

#[test]
fn test_assemble_with_labels() {
    let source = r#"
    start:
        add r1, r2, r3
    loop:
        sub r1, r1, r4
        bne r1, zero, -4
    end:
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 4);
}

#[test]
fn test_assemble_label_on_same_line() {
    let source = r#"
    start: add r1, r2, r3
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 2);
}

#[test]
fn test_assemble_underscore_label() {
    let source = r#"
    _start:
        ecall
    _end_:
        ebreak
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 2);
}

// ============================================================================
// Configuration Directive Tests
// ============================================================================

#[test]
fn test_assemble_with_config() {
    let source = r#"
        .config limb_bits 20
        .config data_limbs 2
        .config addr_limbs 2
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 1);

    let config = program.config();
    assert_eq!(config.limb_bits, 20);
    assert_eq!(config.data_limbs, 2);
    assert_eq!(config.addr_limbs, 2);
}

#[test]
fn test_assemble_with_different_config() {
    let source = r#"
        .config limb_bits 30
        .config data_limbs 3
        .config addr_limbs 2
        ecall
    "#;
    let program = assemble(source).unwrap();

    let config = program.config();
    assert_eq!(config.limb_bits, 30);
    assert_eq!(config.data_limbs, 3);
}

// ============================================================================
// Register Name Tests
// ============================================================================

#[test]
fn test_assemble_all_register_names() {
    // Test numeric register names r0-r15
    let source = r#"
        add r0, r1, r2
        add r3, r4, r5
        add r6, r7, r8
        add r9, r10, r11
        add r12, r13, r14
        add r15, r0, r1
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 7);
}

#[test]
fn test_assemble_abi_register_names() {
    // Test ABI register names
    let source = r#"
        add zero, ra, sp
        add gp, tp, t0
        add a0, a1, a2
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 4);
}

// ============================================================================
// Comment Tests
// ============================================================================

#[test]
fn test_assemble_inline_comments() {
    let source = r#"
        add r1, r2, r3  # This is an inline comment
        sub r4, r5, r6  # Another comment
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 3);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_assemble_invalid_instruction() {
    let source = "foobar r1, r2, r3";
    let result = assemble(source);
    assert!(result.is_err());

    if let Err(AssemblerError::InvalidInstruction { line, instruction }) = result {
        assert_eq!(line, 1);
        assert_eq!(instruction, "foobar");
    } else {
        panic!("Expected InvalidInstruction error");
    }
}

#[test]
fn test_assemble_missing_operands() {
    let source = "add r1, r2";  // Missing third operand
    let result = assemble(source);
    assert!(result.is_err());
}

#[test]
fn test_assemble_duplicate_label() {
    let source = r#"
    start:
        add r1, r2, r3
    start:
        ecall
    "#;
    let result = assemble(source);
    assert!(result.is_err());
}

#[test]
fn test_assemble_invalid_config_key() {
    let source = r#"
        .config invalid_key 100
        ecall
    "#;
    let result = assemble(source);
    assert!(result.is_err());
}

#[test]
fn test_assemble_invalid_config_value() {
    let source = r#"
        .config limb_bits 5
        ecall
    "#;
    let result = assemble(source);
    // limb_bits must be 16-30, so 5 should fail
    assert!(result.is_err());
}

// ============================================================================
// Encoder Tests
// ============================================================================

#[test]
fn test_encode_r_type() {
    let instr = Instruction::Add {
        rd: Register::R1,
        rs1: Register::R2,
        rs2: Register::R3,
    };
    let encoded = encode(&instr);

    // Verify opcode
    assert_eq!(encoded & 0x7F, Opcode::Add.to_u8() as u32);

    // Verify rd (bits 10:7)
    assert_eq!((encoded >> 7) & 0xF, 1);

    // Verify rs1 (bits 14:11)
    assert_eq!((encoded >> 11) & 0xF, 2);

    // Verify rs2 (bits 18:15)
    assert_eq!((encoded >> 15) & 0xF, 3);
}

#[test]
fn test_encode_i_type() {
    let instr = Instruction::Addi {
        rd: Register::R1,
        rs1: Register::R2,
        imm: 100,
    };
    let encoded = encode(&instr);

    // Verify opcode
    assert_eq!(encoded & 0x7F, Opcode::Addi.to_u8() as u32);

    // Verify rd (bits 10:7)
    assert_eq!((encoded >> 7) & 0xF, 1);

    // Verify rs1 (bits 14:11)
    assert_eq!((encoded >> 11) & 0xF, 2);

    // Verify immediate (bits 31:15)
    assert_eq!((encoded >> 15) & 0x1FFFF, 100);
}

#[test]
fn test_encode_negative_immediate() {
    let instr = Instruction::Addi {
        rd: Register::R1,
        rs1: Register::R2,
        imm: -1,
    };
    let encoded = encode(&instr);

    // Verify immediate is encoded as two's complement in 17 bits
    assert_eq!((encoded >> 15) & 0x1FFFF, 0x1FFFF);
}

#[test]
fn test_encode_system_instructions() {
    let ecall = encode(&Instruction::Ecall);
    let ebreak = encode(&Instruction::Ebreak);

    assert_eq!(ecall & 0x7F, Opcode::Ecall.to_u8() as u32);
    assert_eq!(ebreak & 0x7F, Opcode::Ebreak.to_u8() as u32);
}

// ============================================================================
// Parse Register Tests
// ============================================================================

#[test]
fn test_parse_register_numeric() {
    for i in 0..16 {
        let name = format!("r{}", i);
        let reg = parse_register(&name).unwrap();
        assert_eq!(reg.index(), i);
    }
}

#[test]
fn test_parse_register_zero() {
    let reg = parse_register("zero").unwrap();
    assert_eq!(reg, Register::R0);
}

#[test]
fn test_parse_register_abi_names() {
    assert_eq!(parse_register("zero").unwrap(), Register::R0);
    assert_eq!(parse_register("ra").unwrap(), Register::R1);
    assert_eq!(parse_register("sp").unwrap(), Register::R2);
    assert_eq!(parse_register("a0").unwrap(), Register::R11);  // a0-a4 map to R11-R15 in ZKIR
}

#[test]
fn test_parse_register_invalid() {
    assert!(parse_register("r16").is_err());
    assert!(parse_register("x0").is_err());
    assert!(parse_register("invalid").is_err());
}

// ============================================================================
// Roundtrip Tests
// ============================================================================

#[test]
fn test_assemble_encode_consistency() {
    // Manually encode an instruction and verify it matches assembler output
    let source = "add r1, r2, r3";
    let program = assemble(source).unwrap();

    let manual = encode(&Instruction::Add {
        rd: Register::R1,
        rs1: Register::R2,
        rs2: Register::R3,
    });

    assert_eq!(program.code[0], manual);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_assemble_whitespace_handling() {
    let source = "   add    r1  ,  r2  ,  r3   ";
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 1);
}

#[test]
fn test_assemble_case_insensitive_instructions() {
    let source = r#"
        ADD r1, r2, r3
        Add r4, r5, r6
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 3);
}

#[test]
fn test_assemble_hex_immediate() {
    let source = r#"
        addi r1, r2, 0x100
        addi r3, r4, 0xFF
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 3);
}

#[test]
fn test_assemble_binary_immediate() {
    let source = r#"
        addi r1, r2, 0b1010
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert_eq!(program.code.len(), 2);
}

// ============================================================================
// Complex Program Tests
// ============================================================================

#[test]
fn test_assemble_fibonacci_program() {
    let source = r#"
        # Compute first 10 Fibonacci numbers
        .config limb_bits 20
        .config data_limbs 2

        # Initialize: f(0) = 0, f(1) = 1
        addi r1, zero, 0      # r1 = 0
        addi r2, zero, 1      # r2 = 1
        addi r3, zero, 10     # r3 = counter

    loop:
        add r4, r1, r2        # r4 = r1 + r2
        addi r1, r2, 0        # r1 = r2
        addi r2, r4, 0        # r2 = r4
        addi r3, r3, -1       # r3--
        bne r3, zero, -16     # loop if r3 != 0

        # Exit with result in r2
        addi a0, zero, 0      # syscall: exit
        addi a1, r2, 0        # exit code = result
        ecall
    "#;
    let program = assemble(source).unwrap();
    assert!(program.code.len() > 0);
}

#[test]
fn test_assemble_memory_copy_program() {
    let source = r#"
        # Copy 4 words from src to dst
        addi r1, zero, 0x1000   # src address
        addi r2, zero, 0x2000   # dst address
        addi r3, zero, 4        # count

    copy_loop:
        lw r4, 0(r1)            # load word
        sw r4, 0(r2)            # store word
        addi r1, r1, 4          # src++
        addi r2, r2, 4          # dst++
        addi r3, r3, -1         # count--
        bne r3, zero, -20       # loop

        ecall
    "#;
    let program = assemble(source).unwrap();
    assert!(program.code.len() > 0);
}
