//! Additional encoding tests for edge cases not covered in unit tests

use zkir_spec::encoding::*;
use zkir_spec::Opcode;

// ============================================================================
// Signed Extraction Edge Cases
// ============================================================================

#[test]
fn test_extract_imm_signed_edge_cases() {
    // -100 in 17-bit signed = 0x20000 - 100 = 0x1FF9C
    let neg_100 = 0x20000u32 - 100;
    let inst = (neg_100 << 15) | 0x08;
    assert_eq!(extract_imm_signed(inst), -100);

    // Max positive (bit 16 = 0, bits 0-15 all 1s = 65535)
    let max_pos = 0xFFFF;
    let inst_max = (max_pos << 15) | 0x08;
    assert_eq!(extract_imm_signed(inst_max), 65535);
}

#[test]
fn test_extract_offset_signed_edge_cases() {
    // Negative offset (-4 in 21-bit)
    let neg_4 = (1 << 21) - 4;
    let inst = (neg_4 << 11) | Opcode::Jal.to_u8() as u32;
    assert_eq!(extract_offset_signed(inst), -4);

    // Max negative (-1 in 21-bit)
    let neg_1 = 0x1FFFFF;
    let inst_max_neg = (neg_1 << 11) | Opcode::Jal.to_u8() as u32;
    assert_eq!(extract_offset_signed(inst_max_neg), -1);
}

#[test]
fn test_encode_itype_negative_imm() {
    let neg_imm = (1 << 17) - 50;
    let inst = encode_itype(Opcode::Addi, 1, 0, neg_imm);
    assert_eq!(extract_imm_signed(inst), -50);
}

// ============================================================================
// Roundtrip Encoding Tests (exhaustive register coverage)
// ============================================================================

#[test]
fn test_rtype_roundtrip_all_registers() {
    for rd in 0..16 {
        for rs1 in 0..16 {
            for rs2 in 0..16 {
                let inst = encode_rtype(Opcode::Add, rd, rs1, rs2, 0);
                assert_eq!(extract_rd(inst), rd);
                assert_eq!(extract_rs1(inst), rs1);
                assert_eq!(extract_rs2(inst), rs2);
            }
        }
    }
}

#[test]
fn test_stype_roundtrip_all_registers() {
    for rs1 in 0..16 {
        for rs2 in 0..16 {
            let inst = encode_stype(Opcode::Sw, rs1, rs2, 100);
            assert_eq!(extract_stype_rs1(inst), rs1);
            assert_eq!(extract_stype_rs2(inst), rs2);
        }
    }
}

// ============================================================================
// Constants Verification
// ============================================================================

#[test]
fn test_encoding_constants() {
    // Verify shift values match specification
    assert_eq!(OPCODE_SHIFT, 0);
    assert_eq!(RD_SHIFT, 7);
    assert_eq!(RS1_SHIFT, 11);
    assert_eq!(RS2_SHIFT, 15);
    assert_eq!(IMM_SHIFT, 15);
    assert_eq!(FUNCT_SHIFT, 19);
    assert_eq!(OFFSET_SHIFT, 11);

    // Verify masks
    assert_eq!(OPCODE_MASK, 0x7F);
    assert_eq!(REGISTER_MASK, 0xF);
    assert_eq!(IMM_MASK, 0x1FFFF);
    assert_eq!(FUNCT_MASK, 0x1FFF);
    assert_eq!(OFFSET_MASK, 0x1FFFFF);
}

#[test]
fn test_field_bit_widths() {
    assert_eq!(OPCODE_MASK.count_ones(), 7);
    assert_eq!(REGISTER_MASK.count_ones(), 4);
    assert_eq!(IMM_MASK.count_ones(), 17);
    assert_eq!(FUNCT_MASK.count_ones(), 13);
    assert_eq!(OFFSET_MASK.count_ones(), 21);
}

// ============================================================================
// JALR type detection edge case
// ============================================================================

#[test]
fn test_jalr_is_jtype() {
    // JALR uses I-type encoding but is_jtype checks opcode family
    let jalr = encode_itype(Opcode::Jalr, 1, 2, 0);
    assert!(is_jtype(jalr)); // checks opcode family, not encoding format
}
