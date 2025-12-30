//! Integration tests for bounds tracking and instruction validation

use zkir_spec::{
    BoundSource, CryptoType, Instruction, Register, ValueBound,
    validate, validate_program, ValidationError, ValidationWarning,
};

// ============================================================================
// Bounds Tracking Tests
// ============================================================================

#[test]
fn test_constant_bounds() {
    let bound = ValueBound::from_constant(255);
    assert_eq!(bound.max_bits, 8);
    assert!(!bound.needs_range_check(8));
    assert!(!bound.needs_range_check(16));

    let large_bound = ValueBound::from_constant(0xFFFF);
    assert_eq!(large_bound.max_bits, 16);
    assert!(large_bound.needs_range_check(8));
    assert!(!large_bound.needs_range_check(16));
}

#[test]
fn test_type_width_bounds() {
    let imm12 = ValueBound::from_type_width(12);
    assert_eq!(imm12.max_bits, 12);
    assert!(matches!(imm12.source, BoundSource::TypeWidth(12)));

    let imm20 = ValueBound::from_type_width(20);
    assert_eq!(imm20.max_bits, 20);
}

#[test]
fn test_crypto_bounds() {
    // SHA-256 output is 32 bits per word
    let sha_bound = ValueBound::from_crypto(CryptoType::Sha256);
    assert_eq!(sha_bound.max_bits, 32);
    assert!(!sha_bound.needs_range_check(32));
    assert!(!sha_bound.needs_range_check(40));

    // Keccak-256 output is 64 bits per word
    let keccak_bound = ValueBound::from_crypto(CryptoType::Keccak256);
    assert_eq!(keccak_bound.max_bits, 64);
    assert!(keccak_bound.needs_range_check(32));
    assert!(!keccak_bound.needs_range_check(64));
}

#[test]
fn test_crypto_internal_representation() {
    let sha = CryptoType::Sha256;

    // internal = max(min_internal, program_bits)
    // min_internal for SHA-256 is 44
    assert_eq!(sha.internal_bits(40), 44);
    assert_eq!(sha.internal_bits(60), 60);
    assert_eq!(sha.internal_bits(80), 80);

    // Post-crypto headroom
    assert_eq!(sha.post_crypto_headroom(40), 8); // 40 - 32 = 8
}

#[test]
fn test_bound_propagation_add() {
    let a = ValueBound::from_type_width(8);
    let b = ValueBound::from_type_width(8);

    let sum = ValueBound::after_add(&a, &b);
    // Adding two 8-bit values can produce up to 9 bits
    assert_eq!(sum.max_bits, 9);
}

#[test]
fn test_bound_propagation_mul() {
    let a = ValueBound::from_type_width(8);
    let b = ValueBound::from_type_width(8);

    let product = ValueBound::after_mul(&a, &b);
    // Multiplying two 8-bit values can produce up to 16 bits
    assert_eq!(product.max_bits, 16);
}

#[test]
fn test_bound_propagation_and() {
    let a = ValueBound::from_type_width(16);
    let b = ValueBound::from_type_width(8);

    let result = ValueBound::after_and(&a, &b);
    // AND with 8-bit value can't exceed 8 bits
    assert_eq!(result.max_bits, 8);
}

#[test]
fn test_bound_propagation_shift() {
    let a = ValueBound::from_type_width(8);

    let left = ValueBound::after_shl(&a, 4, 64);
    assert_eq!(left.max_bits, 12); // 8 + 4

    let right = ValueBound::after_srl(&a, 4);
    assert_eq!(right.max_bits, 4); // 8 - 4
}

#[test]
fn test_comparison_bounds() {
    let cmp = ValueBound::after_cmp();
    // Comparison always produces 1-bit result (0 or 1)
    assert_eq!(cmp.max_bits, 1);
}

// ============================================================================
// Instruction Validation Tests
// ============================================================================

#[test]
fn test_validate_add_instruction() {
    let add = Instruction::Add {
        rd: Register::R1,
        rs1: Register::R2,
        rs2: Register::R3,
    };
    let result = validate(&add);

    assert!(result.errors.is_empty());
    assert!(result.warnings.is_empty());
}

#[test]
fn test_validate_write_to_r0() {
    // Writing to r0 should produce a warning (r0 is hardwired to zero)
    let add_to_r0 = Instruction::Add {
        rd: Register::R0,
        rs1: Register::R1,
        rs2: Register::R2,
    };
    let result = validate(&add_to_r0);

    assert!(result.errors.is_empty());
    assert!(result.warnings.len() == 1);
    assert!(matches!(
        result.warnings[0],
        ValidationWarning::WriteToR0 { .. }
    ));
}

#[test]
fn test_validate_shift_amount() {
    // Valid shift amount (< 64 for flexibility)
    let slli_valid = Instruction::Slli {
        rd: Register::R1,
        rs1: Register::R2,
        shamt: 40,
    };
    let result = validate(&slli_valid);
    assert!(result.errors.is_empty());

    // Invalid shift amount (> 63)
    let slli_invalid = Instruction::Slli {
        rd: Register::R1,
        rs1: Register::R2,
        shamt: 100,
    };
    let result = validate(&slli_invalid);
    assert!(result.errors.len() == 1);
    assert!(matches!(
        result.errors[0],
        ValidationError::ShiftAmountOutOfRange { .. }
    ));
}

#[test]
fn test_validate_branch_alignment() {
    // Aligned branch offset (multiple of 4)
    let beq_aligned = Instruction::Beq {
        rs1: Register::R1,
        rs2: Register::R2,
        offset: 16,
    };
    let result = validate(&beq_aligned);
    assert!(result.errors.is_empty());

    // Misaligned branch offset
    let beq_misaligned = Instruction::Beq {
        rs1: Register::R1,
        rs2: Register::R2,
        offset: 6,
    };
    let result = validate(&beq_misaligned);
    assert!(result.errors.len() == 1);
    assert!(matches!(
        result.errors[0],
        ValidationError::MisalignedBranchOffset { .. }
    ));
}

#[test]
fn test_validate_jal_alignment() {
    // Aligned jump offset
    let jal_aligned = Instruction::Jal {
        rd: Register::R1,
        offset: 100,
    };
    let result = validate(&jal_aligned);
    assert!(result.errors.is_empty());

    // Misaligned jump offset
    let jal_misaligned = Instruction::Jal {
        rd: Register::R1,
        offset: 2,
    };
    let result = validate(&jal_misaligned);
    assert!(result.errors.len() == 1);
    assert!(matches!(
        result.errors[0],
        ValidationError::MisalignedJumpOffset { .. }
    ));
}

#[test]
fn test_validate_unconditional_branch_warning() {
    // BEQ with rs1 == rs2 is unconditional (always taken)
    let beq_unconditional = Instruction::Beq {
        rs1: Register::R5,
        rs2: Register::R5,
        offset: 8,
    };
    let result = validate(&beq_unconditional);

    assert!(result.errors.is_empty());
    // Should have unconditional branch warning
    assert!(result
        .warnings
        .iter()
        .any(|w| matches!(w, ValidationWarning::UnconditionalBranch { .. })));
}

#[test]
fn test_validate_never_taken_branch_warning() {
    // BNE with rs1 == rs2 is never taken - reported as NoOp
    let bne_never = Instruction::Bne {
        rs1: Register::R5,
        rs2: Register::R5,
        offset: 8,
    };
    let result = validate(&bne_never);

    assert!(result.errors.is_empty());
    assert!(result
        .warnings
        .iter()
        .any(|w| matches!(w, ValidationWarning::NoOp { .. })));
}

#[test]
fn test_validate_noop() {
    // ADD r0, r0, r0 is a no-op
    let noop = Instruction::Add {
        rd: Register::R0,
        rs1: Register::R0,
        rs2: Register::R0,
    };
    let result = validate(&noop);

    assert!(result.errors.is_empty());
    assert!(result
        .warnings
        .iter()
        .any(|w| matches!(w, ValidationWarning::NoOp { .. })));
}

#[test]
fn test_validate_system_instructions() {
    let ecall = Instruction::Ecall;
    let result = validate(&ecall);
    assert!(result.errors.is_empty());

    let ebreak = Instruction::Ebreak;
    let result = validate(&ebreak);
    assert!(result.errors.is_empty());
}

#[test]
fn test_validate_program() {
    let instructions = vec![
        // Valid
        Instruction::Add {
            rd: Register::R1,
            rs1: Register::R2,
            rs2: Register::R3,
        },
        // Warning: write to r0
        Instruction::Slli {
            rd: Register::R0,
            rs1: Register::R1,
            shamt: 16,
        },
        // Error: misaligned
        Instruction::Beq {
            rs1: Register::R1,
            rs2: Register::R2,
            offset: 6,
        },
        // Error: shift too large
        Instruction::Slli {
            rd: Register::R2,
            rs1: Register::R3,
            shamt: 100,
        },
    ];

    let results = validate_program(&instructions);

    // Should have results for instructions with issues
    assert!(!results.is_empty());

    // Find the misaligned branch error
    let has_branch_error = results.iter().any(|(idx, result)| {
        *idx == 2
            && result
                .errors
                .iter()
                .any(|e| matches!(e, ValidationError::MisalignedBranchOffset { .. }))
    });
    assert!(has_branch_error);

    // Find the shift error
    let has_shift_error = results.iter().any(|(idx, result)| {
        *idx == 3
            && result
                .errors
                .iter()
                .any(|e| matches!(e, ValidationError::ShiftAmountOutOfRange { .. }))
    });
    assert!(has_shift_error);
}
