//! Integration tests for ZKIR Disassembler v3.4
//!
//! Tests the complete disassembly workflow including:
//! - Instruction decoding
//! - Output formatting
//! - Error handling for invalid encodings

use zkir_disassembler::{disassemble, decode, format, DisassemblerError};
use zkir_spec::{Instruction, Program, Opcode, Register, Config};

// ============================================================================
// Decode Tests - R-type Instructions
// ============================================================================

#[test]
fn test_decode_all_arithmetic() {
    let opcodes = [
        (Opcode::Add, "Add"),
        (Opcode::Sub, "Sub"),
        (Opcode::Mul, "Mul"),
        (Opcode::Mulh, "Mulh"),
        (Opcode::Div, "Div"),
        (Opcode::Divu, "Divu"),
        (Opcode::Rem, "Rem"),
        (Opcode::Remu, "Remu"),
    ];

    for (opcode, _name) in opcodes {
        let mut word = opcode.to_u8() as u32;
        word |= 1 << 7;  // rd = r1
        word |= 2 << 11; // rs1 = r2
        word |= 3 << 15; // rs2 = r3

        let instr = decode(word).unwrap();
        // Verify it decoded successfully (specific variant depends on opcode)
        assert!(matches!(
            instr,
            Instruction::Add { .. }
                | Instruction::Sub { .. }
                | Instruction::Mul { .. }
                | Instruction::Mulh { .. }
                | Instruction::Div { .. }
                | Instruction::Divu { .. }
                | Instruction::Rem { .. }
                | Instruction::Remu { .. }
        ));
    }
}

#[test]
fn test_decode_all_logical() {
    let opcodes = [Opcode::And, Opcode::Or, Opcode::Xor];

    for opcode in opcodes {
        let mut word = opcode.to_u8() as u32;
        word |= 1 << 7;
        word |= 2 << 11;
        word |= 3 << 15;

        let instr = decode(word).unwrap();
        assert!(matches!(
            instr,
            Instruction::And { .. } | Instruction::Or { .. } | Instruction::Xor { .. }
        ));
    }
}

#[test]
fn test_decode_all_shifts() {
    let opcodes = [Opcode::Sll, Opcode::Srl, Opcode::Sra];

    for opcode in opcodes {
        let mut word = opcode.to_u8() as u32;
        word |= 1 << 7;
        word |= 2 << 11;
        word |= 3 << 15;

        let instr = decode(word).unwrap();
        assert!(matches!(
            instr,
            Instruction::Sll { .. } | Instruction::Srl { .. } | Instruction::Sra { .. }
        ));
    }
}

#[test]
fn test_decode_all_compares() {
    let opcodes = [
        Opcode::Slt,
        Opcode::Sltu,
        Opcode::Sge,
        Opcode::Sgeu,
        Opcode::Seq,
        Opcode::Sne,
    ];

    for opcode in opcodes {
        let mut word = opcode.to_u8() as u32;
        word |= 1 << 7;
        word |= 2 << 11;
        word |= 3 << 15;

        let instr = decode(word).unwrap();
        assert!(matches!(
            instr,
            Instruction::Slt { .. }
                | Instruction::Sltu { .. }
                | Instruction::Sge { .. }
                | Instruction::Sgeu { .. }
                | Instruction::Seq { .. }
                | Instruction::Sne { .. }
        ));
    }
}

#[test]
fn test_decode_all_cmov() {
    let opcodes = [Opcode::Cmov, Opcode::Cmovz, Opcode::Cmovnz];

    for opcode in opcodes {
        let mut word = opcode.to_u8() as u32;
        word |= 1 << 7;
        word |= 2 << 11;
        word |= 3 << 15;

        let instr = decode(word).unwrap();
        assert!(matches!(
            instr,
            Instruction::Cmov { .. } | Instruction::Cmovz { .. } | Instruction::Cmovnz { .. }
        ));
    }
}

// ============================================================================
// Decode Tests - I-type Instructions
// ============================================================================

#[test]
fn test_decode_addi() {
    let mut word = Opcode::Addi.to_u8() as u32;
    word |= 1 << 7;    // rd = r1
    word |= 2 << 11;   // rs1 = r2
    word |= 100 << 15; // imm = 100

    let instr = decode(word).unwrap();
    assert_eq!(
        instr,
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R2,
            imm: 100
        }
    );
}

#[test]
fn test_decode_addi_negative() {
    let mut word = Opcode::Addi.to_u8() as u32;
    word |= 1 << 7;          // rd = r1
    word |= 2 << 11;         // rs1 = r2
    word |= 0x1FFFF << 15;   // imm = -1 in 17-bit two's complement

    let instr = decode(word).unwrap();
    assert_eq!(
        instr,
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R2,
            imm: -1
        }
    );
}

#[test]
fn test_decode_all_logical_immediate() {
    let opcodes = [Opcode::Andi, Opcode::Ori, Opcode::Xori];

    for opcode in opcodes {
        let mut word = opcode.to_u8() as u32;
        word |= 1 << 7;
        word |= 2 << 11;
        word |= 0xFF << 15;

        let instr = decode(word).unwrap();
        assert!(matches!(
            instr,
            Instruction::Andi { .. } | Instruction::Ori { .. } | Instruction::Xori { .. }
        ));
    }
}

#[test]
fn test_decode_shift_immediate() {
    let opcodes = [Opcode::Slli, Opcode::Srli, Opcode::Srai];

    for opcode in opcodes {
        let mut word = opcode.to_u8() as u32;
        word |= 1 << 7;   // rd = r1
        word |= 2 << 11;  // rs1 = r2
        word |= 5 << 15;  // shamt = 5

        let instr = decode(word).unwrap();
        assert!(matches!(
            instr,
            Instruction::Slli { shamt: 5, .. }
                | Instruction::Srli { shamt: 5, .. }
                | Instruction::Srai { shamt: 5, .. }
        ));
    }
}

// ============================================================================
// Decode Tests - Load/Store Instructions
// ============================================================================

#[test]
fn test_decode_all_loads() {
    let opcodes = [
        Opcode::Lb,
        Opcode::Lbu,
        Opcode::Lh,
        Opcode::Lhu,
        Opcode::Lw,
        Opcode::Ld,
    ];

    for opcode in opcodes {
        let mut word = opcode.to_u8() as u32;
        word |= 1 << 7;   // rd = r1
        word |= 2 << 11;  // rs1 = r2
        word |= 16 << 15; // imm = 16

        let instr = decode(word).unwrap();
        assert!(matches!(
            instr,
            Instruction::Lb { imm: 16, .. }
                | Instruction::Lbu { imm: 16, .. }
                | Instruction::Lh { imm: 16, .. }
                | Instruction::Lhu { imm: 16, .. }
                | Instruction::Lw { imm: 16, .. }
                | Instruction::Ld { imm: 16, .. }
        ));
    }
}

#[test]
fn test_decode_all_stores() {
    let opcodes = [Opcode::Sb, Opcode::Sh, Opcode::Sw, Opcode::Sd];

    for opcode in opcodes {
        let mut word = opcode.to_u8() as u32;
        word |= 2 << 7;   // rs1 = r2 (base address)
        word |= 1 << 11;  // rs2 = r1 (value to store)
        word |= 16 << 15; // imm = 16

        let instr = decode(word).unwrap();
        assert!(matches!(
            instr,
            Instruction::Sb { imm: 16, .. }
                | Instruction::Sh { imm: 16, .. }
                | Instruction::Sw { imm: 16, .. }
                | Instruction::Sd { imm: 16, .. }
        ));
    }
}

// ============================================================================
// Decode Tests - Branch Instructions
// ============================================================================

#[test]
fn test_decode_all_branches() {
    let opcodes = [
        Opcode::Beq,
        Opcode::Bne,
        Opcode::Blt,
        Opcode::Bge,
        Opcode::Bltu,
        Opcode::Bgeu,
    ];

    for opcode in opcodes {
        let mut word = opcode.to_u8() as u32;
        word |= 1 << 7;  // rs1 = r1
        word |= 2 << 11; // rs2 = r2
        word |= 8 << 15; // offset = 8

        let instr = decode(word).unwrap();
        assert!(matches!(
            instr,
            Instruction::Beq { offset: 8, .. }
                | Instruction::Bne { offset: 8, .. }
                | Instruction::Blt { offset: 8, .. }
                | Instruction::Bge { offset: 8, .. }
                | Instruction::Bltu { offset: 8, .. }
                | Instruction::Bgeu { offset: 8, .. }
        ));
    }
}

#[test]
fn test_decode_branch_negative_offset() {
    let mut word = Opcode::Beq.to_u8() as u32;
    word |= 1 << 7;          // rs1 = r1
    word |= 2 << 11;         // rs2 = r2
    word |= 0x1FFF8 << 15;   // offset = -8 in 17-bit two's complement

    let instr = decode(word).unwrap();
    if let Instruction::Beq { offset, .. } = instr {
        assert_eq!(offset, -8);
    } else {
        panic!("Expected Beq instruction");
    }
}

// ============================================================================
// Decode Tests - Jump Instructions
// ============================================================================

#[test]
fn test_decode_jal() {
    let mut word = Opcode::Jal.to_u8() as u32;
    word |= 1 << 7;    // rd = r1
    word |= 100 << 11; // offset = 100 (21-bit field)

    let instr = decode(word).unwrap();
    assert_eq!(
        instr,
        Instruction::Jal {
            rd: Register::R1,
            offset: 100
        }
    );
}

#[test]
fn test_decode_jalr() {
    let mut word = Opcode::Jalr.to_u8() as u32;
    word |= 1 << 7;    // rd = r1
    word |= 2 << 11;   // rs1 = r2
    word |= 100 << 15; // imm = 100

    let instr = decode(word).unwrap();
    assert_eq!(
        instr,
        Instruction::Jalr {
            rd: Register::R1,
            rs1: Register::R2,
            imm: 100
        }
    );
}

// ============================================================================
// Decode Tests - System Instructions
// ============================================================================

#[test]
fn test_decode_ecall() {
    let word = Opcode::Ecall.to_u8() as u32;
    let instr = decode(word).unwrap();
    assert_eq!(instr, Instruction::Ecall);
}

#[test]
fn test_decode_ebreak() {
    let word = Opcode::Ebreak.to_u8() as u32;
    let instr = decode(word).unwrap();
    assert_eq!(instr, Instruction::Ebreak);
}

// ============================================================================
// Decode Error Tests
// ============================================================================

#[test]
fn test_decode_unknown_opcode() {
    let word = 0x7F; // 0x7F is not a valid opcode
    let result = decode(word);
    assert!(result.is_err());

    if let Err(DisassemblerError::UnknownOpcode(opcode)) = result {
        assert_eq!(opcode, 0x7F);
    } else {
        panic!("Expected UnknownOpcode error");
    }
}

#[test]
fn test_decode_invalid_opcodes() {
    // Test various invalid opcodes beyond the valid range
    let invalid_opcodes = [0x52, 0x53, 0x60, 0x7F];

    for &opcode in &invalid_opcodes {
        let word = opcode as u32;
        let result = decode(word);
        assert!(result.is_err(), "Opcode {:#x} should be invalid", opcode);
    }
}

// ============================================================================
// Format Tests
// ============================================================================

#[test]
fn test_format_r_type() {
    let instr = Instruction::Add {
        rd: Register::R1,
        rs1: Register::R2,
        rs2: Register::R3,
    };
    let formatted = format(&instr);
    assert!(formatted.contains("add"));
    // Formatter uses ABI names: R1=ra, R2=sp, R3=fp
    assert!(formatted.contains("ra"));
    assert!(formatted.contains("sp"));
    assert!(formatted.contains("fp"));
}

#[test]
fn test_format_i_type() {
    let instr = Instruction::Addi {
        rd: Register::R1,
        rs1: Register::R2,
        imm: 100,
    };
    let formatted = format(&instr);
    assert!(formatted.contains("addi"));
    // Formatter uses ABI names: R1=ra, R2=sp
    assert!(formatted.contains("ra"));
    assert!(formatted.contains("sp"));
    assert!(formatted.contains("100"));
}

#[test]
fn test_format_negative_immediate() {
    let instr = Instruction::Addi {
        rd: Register::R1,
        rs1: Register::R2,
        imm: -50,
    };
    let formatted = format(&instr);
    assert!(formatted.contains("-50") || formatted.contains("−50"));
}

#[test]
fn test_format_load() {
    let instr = Instruction::Lw {
        rd: Register::R1,
        rs1: Register::R2,
        imm: 16,
    };
    let formatted = format(&instr);
    assert!(formatted.contains("lw"));
    // Load format: lw rd, imm(rs1)
    assert!(formatted.contains("16"));
}

#[test]
fn test_format_store() {
    let instr = Instruction::Sw {
        rs1: Register::R2,
        rs2: Register::R1,
        imm: 16,
    };
    let formatted = format(&instr);
    assert!(formatted.contains("sw"));
}

#[test]
fn test_format_branch() {
    let instr = Instruction::Beq {
        rs1: Register::R1,
        rs2: Register::R2,
        offset: 8,
    };
    let formatted = format(&instr);
    assert!(formatted.contains("beq"));
    assert!(formatted.contains("8"));
}

#[test]
fn test_format_system() {
    let ecall = format(&Instruction::Ecall);
    let ebreak = format(&Instruction::Ebreak);

    assert!(ecall.contains("ecall"));
    assert!(ebreak.contains("ebreak"));
}

// ============================================================================
// Disassemble Tests
// ============================================================================

#[test]
fn test_disassemble_empty_program() {
    let program = Program::new();
    let output = disassemble(&program).unwrap();

    assert!(output.contains("ZKIR v3.4"));
    assert!(output.contains("0 instructions"));
}

#[test]
fn test_disassemble_single_instruction() {
    let mut program = Program::new();
    program.code = vec![Opcode::Ecall.to_u8() as u32];
    program.header.code_size = 4;

    let output = disassemble(&program).unwrap();

    assert!(output.contains("ecall"));
    assert!(output.contains("1 instructions"));
}

#[test]
fn test_disassemble_multiple_instructions() {
    let mut program = Program::new();

    // Create a simple program
    let mut code = Vec::new();

    // ADD r1, r2, r3
    let mut word = Opcode::Add.to_u8() as u32;
    word |= 1 << 7;
    word |= 2 << 11;
    word |= 3 << 15;
    code.push(word);

    // ECALL
    code.push(Opcode::Ecall.to_u8() as u32);

    program.code = code;
    program.header.code_size = 8;

    let output = disassemble(&program).unwrap();

    assert!(output.contains("add"));
    assert!(output.contains("ecall"));
    assert!(output.contains("2 instructions"));
}

#[test]
fn test_disassemble_with_config() {
    let config = Config {
        limb_bits: 20,
        data_limbs: 2,
        addr_limbs: 2,
    };

    let mut program = Program::with_config(config).unwrap();
    program.code = vec![Opcode::Ecall.to_u8() as u32];
    program.header.code_size = 4;

    let output = disassemble(&program).unwrap();

    assert!(output.contains("Limb bits:  20"));
    assert!(output.contains("Data limbs: 2"));
    assert!(output.contains("40-bit"));
}

#[test]
fn test_disassemble_shows_addresses() {
    let mut program = Program::new();
    program.code = vec![
        Opcode::Ecall.to_u8() as u32,
        Opcode::Ebreak.to_u8() as u32,
    ];
    program.header.code_size = 8;

    let output = disassemble(&program).unwrap();

    // Should show addresses for each instruction
    // The format is "0xXXXXXXXX:"
    assert!(output.contains("0x"));
}

#[test]
fn test_disassemble_shows_hex_encoding() {
    let mut program = Program::new();
    program.code = vec![Opcode::Ecall.to_u8() as u32];
    program.header.code_size = 4;

    let output = disassemble(&program).unwrap();

    // Should show hex encoding
    let opcode_hex = format!("{:08X}", Opcode::Ecall.to_u8());
    assert!(output.contains(&opcode_hex) || output.contains("00000050"));
}

// ============================================================================
// Roundtrip Tests (with zkir-assembler)
// ============================================================================

#[test]
fn test_decode_encode_roundtrip() {
    // Encode an instruction using assembler, decode it, verify it matches
    let original = Instruction::Add {
        rd: Register::R5,
        rs1: Register::R10,
        rs2: Register::R15,
    };

    let encoded = zkir_assembler::encode(&original);
    let decoded = decode(encoded).unwrap();

    assert_eq!(decoded, original);
}

#[test]
fn test_decode_encode_all_registers() {
    // Test decoding with all 16 registers
    for rd in 0..16u8 {
        for rs1 in 0..16u8 {
            for rs2 in 0..16u8 {
                let mut word = Opcode::Add.to_u8() as u32;
                word |= (rd as u32) << 7;
                word |= (rs1 as u32) << 11;
                word |= (rs2 as u32) << 15;

                let instr = decode(word).unwrap();

                if let Instruction::Add { rd: r_rd, rs1: r_rs1, rs2: r_rs2 } = instr {
                    assert_eq!(r_rd.index(), rd);
                    assert_eq!(r_rs1.index(), rs1);
                    assert_eq!(r_rs2.index(), rs2);
                } else {
                    panic!("Expected Add instruction");
                }
            }
        }
    }
}

#[test]
fn test_decode_encode_immediate_range() {
    // Test immediate values from -65536 to 65535 (17-bit signed range)
    let test_values = [-65536, -1000, -1, 0, 1, 1000, 65535];

    for &imm in &test_values {
        let original = Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R2,
            imm,
        };

        let encoded = zkir_assembler::encode(&original);
        let decoded = decode(encoded).unwrap();

        assert_eq!(decoded, original, "Failed for immediate value {}", imm);
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_decode_max_register_values() {
    // Test with r15 in all positions
    let mut word = Opcode::Add.to_u8() as u32;
    word |= 15 << 7;  // rd = r15
    word |= 15 << 11; // rs1 = r15
    word |= 15 << 15; // rs2 = r15

    let instr = decode(word).unwrap();
    if let Instruction::Add { rd, rs1, rs2 } = instr {
        assert_eq!(rd, Register::R15);
        assert_eq!(rs1, Register::R15);
        assert_eq!(rs2, Register::R15);
    } else {
        panic!("Expected Add instruction");
    }
}

#[test]
fn test_decode_zero_register() {
    // Test with r0/zero in all positions
    let mut word = Opcode::Add.to_u8() as u32;
    word |= 0 << 7;  // rd = r0
    word |= 0 << 11; // rs1 = r0
    word |= 0 << 15; // rs2 = r0

    let instr = decode(word).unwrap();
    if let Instruction::Add { rd, rs1, rs2 } = instr {
        assert_eq!(rd, Register::R0);
        assert_eq!(rs1, Register::R0);
        assert_eq!(rs2, Register::R0);
    } else {
        panic!("Expected Add instruction");
    }
}

#[test]
fn test_decode_max_jal_offset() {
    // JAL uses 21-bit signed offset
    let mut word = Opcode::Jal.to_u8() as u32;
    word |= 1 << 7;           // rd = r1
    word |= 0xFFFFF << 11;    // max positive 20-bit offset

    let instr = decode(word).unwrap();
    if let Instruction::Jal { rd, offset } = instr {
        assert_eq!(rd, Register::R1);
        // 0xFFFFF is 1048575, which is the max positive value in 21-bit signed
        assert!(offset > 0);
    } else {
        panic!("Expected Jal instruction");
    }
}

#[test]
fn test_decode_min_jal_offset() {
    // JAL with minimum (most negative) offset
    let mut word = Opcode::Jal.to_u8() as u32;
    word |= 1 << 7;           // rd = r1
    word |= 0x100000 << 11;   // sign bit set (21-bit two's complement)

    let instr = decode(word).unwrap();
    if let Instruction::Jal { rd, offset } = instr {
        assert_eq!(rd, Register::R1);
        assert!(offset < 0);
    } else {
        panic!("Expected Jal instruction");
    }
}

// ============================================================================
// Complete Program Disassembly Test
// ============================================================================

#[test]
fn test_disassemble_complete_program() {
    let mut program = Program::new();

    // Build a complete test program
    let mut code = Vec::new();

    // ADDI r1, r0, 10
    let mut word = Opcode::Addi.to_u8() as u32;
    word |= 1 << 7;
    word |= 0 << 11;
    word |= 10 << 15;
    code.push(word);

    // ADDI r2, r0, 20
    word = Opcode::Addi.to_u8() as u32;
    word |= 2 << 7;
    word |= 0 << 11;
    word |= 20 << 15;
    code.push(word);

    // ADD r3, r1, r2
    word = Opcode::Add.to_u8() as u32;
    word |= 3 << 7;
    word |= 1 << 11;
    word |= 2 << 15;
    code.push(word);

    // ECALL
    code.push(Opcode::Ecall.to_u8() as u32);

    program.code = code;
    program.header.code_size = 16;

    let output = disassemble(&program).unwrap();

    // Verify all instructions are present
    assert!(output.contains("addi"));
    assert!(output.contains("add"));
    assert!(output.contains("ecall"));

    // Verify program info
    assert!(output.contains("4 instructions"));
    assert!(output.contains("16 bytes"));
}
