//! # Instruction Encoding Constants and Helpers
//!
//! This module provides centralized constants and helper functions for
//! ZKIR v3.4 instruction encoding/decoding.
//!
//! ## Instruction Format (32-bit)
//!
//! ```text
//! R-type: [opcode:7][rd:4][rs1:4][rs2:4][funct:13]
//! I-type: [opcode:7][rd:4][rs1:4][imm:17]
//! S-type: [opcode:7][rs1:4][rs2:4][imm:17]
//! B-type: [opcode:7][rs1:4][rs2:4][offset:17]
//! J-type: [opcode:7][rd:4][offset:21]
//! ```

use crate::Opcode;

// ============================================================================
// Bit Position Constants
// ============================================================================

/// Opcode field: bits 0-6 (7 bits)
pub const OPCODE_SHIFT: u32 = 0;

/// Destination register field: bits 7-10 (4 bits)
pub const RD_SHIFT: u32 = 7;

/// Source register 1 field: bits 11-14 (4 bits)
pub const RS1_SHIFT: u32 = 11;

/// Source register 2 field: bits 15-18 (4 bits)
pub const RS2_SHIFT: u32 = 15;

/// Immediate field for I/S/B-type: bits 15-31 (17 bits)
pub const IMM_SHIFT: u32 = 15;

/// Function field for R-type: bits 19-31 (13 bits)
pub const FUNCT_SHIFT: u32 = 19;

/// Offset field for J-type: bits 11-31 (21 bits)
pub const OFFSET_SHIFT: u32 = 11;

// ============================================================================
// Field Masks
// ============================================================================

/// Opcode mask (7 bits)
pub const OPCODE_MASK: u32 = 0x7F;

/// Register field mask (4 bits)
pub const REGISTER_MASK: u32 = 0xF;

/// Immediate field mask for I/S/B-type (17 bits)
pub const IMM_MASK: u32 = 0x1FFFF;

/// Function field mask for R-type (13 bits)
pub const FUNCT_MASK: u32 = 0x1FFF;

/// Offset field mask for J-type (21 bits)
pub const OFFSET_MASK: u32 = 0x1FFFFF;

/// Sign bit position in 17-bit immediate
pub const IMM_SIGN_BIT: u32 = 16;

/// Sign extension value for 17-bit immediate (2^17)
pub const IMM_SIGN_EXTEND: u32 = 1 << 17;

// ============================================================================
// Field Extraction Functions
// ============================================================================

/// Extract opcode from instruction (bits 0-6)
#[inline]
pub const fn extract_opcode(inst: u32) -> u32 {
    inst & OPCODE_MASK
}

/// Extract destination register from instruction (bits 7-10)
#[inline]
pub const fn extract_rd(inst: u32) -> u32 {
    (inst >> RD_SHIFT) & REGISTER_MASK
}

/// Extract source register 1 from instruction (bits 11-14)
#[inline]
pub const fn extract_rs1(inst: u32) -> u32 {
    (inst >> RS1_SHIFT) & REGISTER_MASK
}

/// Extract source register 2 from instruction (bits 15-18)
#[inline]
pub const fn extract_rs2(inst: u32) -> u32 {
    (inst >> RS2_SHIFT) & REGISTER_MASK
}

/// Extract immediate from I-type instruction (bits 15-31, 17 bits)
#[inline]
pub const fn extract_imm(inst: u32) -> u32 {
    (inst >> IMM_SHIFT) & IMM_MASK
}

/// Extract immediate with sign extension
#[inline]
pub const fn extract_imm_signed(inst: u32) -> i32 {
    let imm = extract_imm(inst);
    if imm & (1 << IMM_SIGN_BIT) != 0 {
        // Sign extend: imm - 2^17
        (imm as i32) - (IMM_SIGN_EXTEND as i32)
    } else {
        imm as i32
    }
}

/// Extract function field from R-type instruction (bits 19-31, 13 bits)
#[inline]
pub const fn extract_funct(inst: u32) -> u32 {
    (inst >> FUNCT_SHIFT) & FUNCT_MASK
}

/// Extract offset from J-type instruction (bits 11-31, 21 bits)
#[inline]
pub const fn extract_offset(inst: u32) -> u32 {
    (inst >> OFFSET_SHIFT) & OFFSET_MASK
}

/// Extract offset with sign extension for J-type
#[inline]
pub const fn extract_offset_signed(inst: u32) -> i32 {
    let offset = extract_offset(inst);
    if offset & (1 << 20) != 0 {
        // Sign extend: offset - 2^21
        (offset as i32) - (1 << 21)
    } else {
        offset as i32
    }
}

// ============================================================================
// S-type/B-type Specific Extraction
// ============================================================================

/// Extract rs1 from S-type/B-type instruction (bits 7-10)
/// Note: S-type and B-type don't have rd, rs1 is at rd position
#[inline]
pub const fn extract_stype_rs1(inst: u32) -> u32 {
    (inst >> RD_SHIFT) & REGISTER_MASK
}

/// Extract rs2 from S-type/B-type instruction (bits 11-14)
#[inline]
pub const fn extract_stype_rs2(inst: u32) -> u32 {
    (inst >> RS1_SHIFT) & REGISTER_MASK
}

/// Extract immediate from S-type instruction (bits 15-31, 17 bits)
#[inline]
pub const fn extract_stype_imm(inst: u32) -> u32 {
    (inst >> IMM_SHIFT) & IMM_MASK
}

// ============================================================================
// Instruction Encoding Functions
// ============================================================================

/// Encode R-type instruction
#[inline]
pub const fn encode_rtype(opcode: Opcode, rd: u32, rs1: u32, rs2: u32, funct: u32) -> u32 {
    (opcode.to_u8() as u32)
        | ((rd & REGISTER_MASK) << RD_SHIFT)
        | ((rs1 & REGISTER_MASK) << RS1_SHIFT)
        | ((rs2 & REGISTER_MASK) << RS2_SHIFT)
        | ((funct & FUNCT_MASK) << FUNCT_SHIFT)
}

/// Encode I-type instruction
#[inline]
pub const fn encode_itype(opcode: Opcode, rd: u32, rs1: u32, imm: u32) -> u32 {
    (opcode.to_u8() as u32)
        | ((rd & REGISTER_MASK) << RD_SHIFT)
        | ((rs1 & REGISTER_MASK) << RS1_SHIFT)
        | ((imm & IMM_MASK) << IMM_SHIFT)
}

/// Encode S-type instruction (stores)
#[inline]
pub const fn encode_stype(opcode: Opcode, rs1: u32, rs2: u32, imm: u32) -> u32 {
    (opcode.to_u8() as u32)
        | ((rs1 & REGISTER_MASK) << RD_SHIFT)
        | ((rs2 & REGISTER_MASK) << RS1_SHIFT)
        | ((imm & IMM_MASK) << IMM_SHIFT)
}

/// Encode B-type instruction (branches)
#[inline]
pub const fn encode_btype(opcode: Opcode, rs1: u32, rs2: u32, offset: u32) -> u32 {
    encode_stype(opcode, rs1, rs2, offset)
}

/// Encode J-type instruction (jumps)
#[inline]
pub const fn encode_jtype(opcode: Opcode, rd: u32, offset: u32) -> u32 {
    (opcode.to_u8() as u32)
        | ((rd & REGISTER_MASK) << RD_SHIFT)
        | ((offset & OFFSET_MASK) << OFFSET_SHIFT)
}

// ============================================================================
// Instruction Type Detection
// ============================================================================

/// Check if instruction is S-type (store) based on opcode
#[inline]
pub fn is_stype(inst: u32) -> bool {
    Opcode::is_store_raw(extract_opcode(inst))
}

/// Check if instruction is B-type (branch) based on opcode
#[inline]
pub fn is_btype(inst: u32) -> bool {
    Opcode::is_branch_raw(extract_opcode(inst))
}

/// Check if instruction is J-type (jump) based on opcode
#[inline]
pub fn is_jtype(inst: u32) -> bool {
    Opcode::is_jump_raw(extract_opcode(inst))
}

/// Check if instruction is I-type (has immediate, not S/B/J type)
#[inline]
pub fn is_itype(inst: u32) -> bool {
    let opcode = extract_opcode(inst);
    // I-type: ADDI, logical immediates, loads, JALR
    Opcode::from_u8(opcode as u8).map_or(false, |op| {
        matches!(op,
            Opcode::Addi |
            Opcode::Andi | Opcode::Ori | Opcode::Xori |
            Opcode::Slli | Opcode::Srli | Opcode::Srai |
            Opcode::Lb | Opcode::Lbu | Opcode::Lh | Opcode::Lhu | Opcode::Lw | Opcode::Ld |
            Opcode::Jalr
        )
    })
}

/// Check if instruction is R-type (register-register)
#[inline]
pub fn is_rtype(inst: u32) -> bool {
    let opcode = extract_opcode(inst);
    !is_stype(inst) && !is_btype(inst) && !is_jtype(inst) && !is_itype(inst) &&
    Opcode::from_u8(opcode as u8).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_opcode() {
        let inst = 0x12345678u32;
        assert_eq!(extract_opcode(inst), 0x78 & OPCODE_MASK);
    }

    #[test]
    fn test_extract_registers() {
        // Construct instruction with known register values
        // opcode=0x00, rd=5, rs1=10, rs2=15
        let inst = 0x00 | (5 << 7) | (10 << 11) | (15 << 15);
        assert_eq!(extract_rd(inst), 5);
        assert_eq!(extract_rs1(inst), 10);
        assert_eq!(extract_rs2(inst), 15);
    }

    #[test]
    fn test_extract_imm() {
        // Construct I-type with imm = 0x1234
        let inst = (0x1234u32 << 15) | 0x08; // ADDI opcode
        assert_eq!(extract_imm(inst), 0x1234);
    }

    #[test]
    fn test_extract_imm_signed() {
        // Positive immediate
        let inst_pos = (100u32 << 15) | 0x08;
        assert_eq!(extract_imm_signed(inst_pos), 100);

        // Negative immediate (bit 16 set)
        let neg_imm = 0x1FFFF; // All 17 bits set = -1
        let inst_neg = (neg_imm << 15) | 0x08;
        assert_eq!(extract_imm_signed(inst_neg), -1);
    }

    #[test]
    fn test_encode_rtype() {
        let inst = encode_rtype(Opcode::Add, 1, 2, 3, 0);
        assert_eq!(extract_opcode(inst), Opcode::Add.to_u8() as u32);
        assert_eq!(extract_rd(inst), 1);
        assert_eq!(extract_rs1(inst), 2);
        assert_eq!(extract_rs2(inst), 3);
    }

    #[test]
    fn test_encode_itype() {
        let inst = encode_itype(Opcode::Addi, 1, 2, 100);
        assert_eq!(extract_opcode(inst), Opcode::Addi.to_u8() as u32);
        assert_eq!(extract_rd(inst), 1);
        assert_eq!(extract_rs1(inst), 2);
        assert_eq!(extract_imm(inst), 100);
    }

    #[test]
    fn test_encode_stype() {
        let inst = encode_stype(Opcode::Sw, 1, 2, 50);
        assert_eq!(extract_opcode(inst), Opcode::Sw.to_u8() as u32);
        assert_eq!(extract_stype_rs1(inst), 1);
        assert_eq!(extract_stype_rs2(inst), 2);
        assert_eq!(extract_stype_imm(inst), 50);
    }

    #[test]
    fn test_encode_jtype() {
        let inst = encode_jtype(Opcode::Jal, 1, 0x1000);
        assert_eq!(extract_opcode(inst), Opcode::Jal.to_u8() as u32);
        assert_eq!(extract_rd(inst), 1);
        assert_eq!(extract_offset(inst), 0x1000);
    }

    #[test]
    fn test_type_detection() {
        let add = encode_rtype(Opcode::Add, 1, 2, 3, 0);
        let addi = encode_itype(Opcode::Addi, 1, 2, 100);
        let sw = encode_stype(Opcode::Sw, 1, 2, 50);
        let beq = encode_btype(Opcode::Beq, 1, 2, 100);
        let jal = encode_jtype(Opcode::Jal, 1, 0x1000);

        assert!(is_rtype(add));
        assert!(is_itype(addi));
        assert!(is_stype(sw));
        assert!(is_btype(beq));
        assert!(is_jtype(jal));
    }
}
