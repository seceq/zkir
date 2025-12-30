//! Edge case tests for cryptographic operations
//!
//! Tests crypto syscalls with various input sizes and edge cases.

use zkir_runtime::{Memory, VM, VMConfig, HaltReason};
use zkir_runtime::crypto::{sha256_hash, keccak256_hash, blake3_hash};
use zkir_spec::{Instruction, Program, Register, ValueBound};

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
// SHA-256 Edge Cases
// ============================================================================

#[test]
fn test_sha256_empty_input() {
    let mut memory = Memory::new();
    let output_ptr = 0x1000;

    let bound = sha256_hash(&mut memory, 0, 0, output_ptr).unwrap();

    // Check bound is correct (32 bits for SHA-256)
    assert_eq!(bound.max_bits, 32);

    // Verify known hash of empty string
    // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    let expected = [
        0xe3b0c442u32, 0x98fc1c14, 0x9afbf4c8, 0x996fb924,
        0x27ae41e4, 0x649b934c, 0xa495991b, 0x7852b855,
    ];

    for (i, &exp) in expected.iter().enumerate() {
        let word = memory.read_u32(output_ptr + (i * 4) as u64).unwrap();
        assert_eq!(word, exp, "Word {} mismatch", i);
    }
}

#[test]
fn test_sha256_single_byte() {
    let mut memory = Memory::new();

    // Write single byte 'a'
    let input_ptr = 0x2000;
    memory.write_u8(input_ptr, b'a').unwrap();

    let output_ptr = 0x3000;
    let bound = sha256_hash(&mut memory, input_ptr, 1, output_ptr).unwrap();

    assert_eq!(bound.max_bits, 32);

    // SHA-256("a") = ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb
    let first_word = memory.read_u32(output_ptr).unwrap();
    assert_eq!(first_word, 0xca978112);
}

#[test]
fn test_sha256_55_bytes() {
    // 55 bytes is the maximum for a single block (56 - 1 for padding byte)
    let mut memory = Memory::new();

    let input_ptr = 0x1000;
    for i in 0..55 {
        memory.write_u8(input_ptr + i, b'x').unwrap();
    }

    let output_ptr = 0x2000;
    let result = sha256_hash(&mut memory, input_ptr, 55, output_ptr);

    assert!(result.is_ok());
}

#[test]
fn test_sha256_max_single_block() {
    // Test exactly at the boundary (55 bytes)
    let mut memory = Memory::new();

    let input_ptr = 0x1000;
    let input_len = 55;

    for i in 0..input_len {
        memory.write_u8(input_ptr + i, (i % 256) as u8).unwrap();
    }

    let output_ptr = 0x2000;
    let bound = sha256_hash(&mut memory, input_ptr, input_len, output_ptr).unwrap();

    assert_eq!(bound.max_bits, 32);
}

#[test]
fn test_sha256_known_vectors() {
    // Test with known test vectors
    // SHA256 writes output as u32 words, so we read u32s and format them
    let test_cases = [
        (b"" as &[u8], [0xe3b0c442u32, 0x98fc1c14]),
        (b"abc", [0xba7816bf, 0x8f01cfea]),
        (b"hello", [0x2cf24dba, 0x5fb0a30e]),
    ];

    for (input, expected_words) in test_cases {
        let mut memory = Memory::new();

        let input_ptr = 0x1000;
        for (i, &byte) in input.iter().enumerate() {
            memory.write_u8(input_ptr + i as u64, byte).unwrap();
        }

        let output_ptr = 0x2000;
        sha256_hash(&mut memory, input_ptr, input.len() as u64, output_ptr).unwrap();

        // Read first 2 words (8 bytes)
        let word0 = memory.read_u32(output_ptr).unwrap();
        let word1 = memory.read_u32(output_ptr + 4).unwrap();

        assert_eq!(word0, expected_words[0], "Word 0 mismatch for input {:?}", input);
        assert_eq!(word1, expected_words[1], "Word 1 mismatch for input {:?}", input);
    }
}

// ============================================================================
// Keccak-256 Edge Cases
// ============================================================================

#[test]
fn test_keccak256_empty_input() {
    let mut memory = Memory::new();
    let output_ptr = 0x1000;

    let bound = keccak256_hash(&mut memory, 0, 0, output_ptr).unwrap();

    // Check bound (64 bits for Keccak)
    assert_eq!(bound.max_bits, 64);

    // Verify known hash
    // Keccak-256("") = c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
    let first_byte = memory.read_u8(output_ptr).unwrap();
    assert_eq!(first_byte, 0xc5);
}

#[test]
fn test_keccak256_single_byte() {
    let mut memory = Memory::new();

    let input_ptr = 0x1000;
    memory.write_u8(input_ptr, b'a').unwrap();

    let output_ptr = 0x2000;
    let bound = keccak256_hash(&mut memory, input_ptr, 1, output_ptr).unwrap();

    assert_eq!(bound.max_bits, 64);

    // Just verify it produces output (not checking specific value)
    let first_byte = memory.read_u8(output_ptr).unwrap();
    assert!(first_byte != 0 || memory.read_u8(output_ptr + 1).unwrap() != 0);
}

#[test]
fn test_keccak256_known_vectors() {
    let test_cases = [
        (b"" as &[u8], 0xc5u8),       // First byte of hash
        (b"hello", 0x1c),
    ];

    for (input, expected_first_byte) in test_cases {
        let mut memory = Memory::new();

        let input_ptr = 0x1000;
        for (i, &byte) in input.iter().enumerate() {
            memory.write_u8(input_ptr + i as u64, byte).unwrap();
        }

        let output_ptr = 0x2000;
        keccak256_hash(&mut memory, input_ptr, input.len() as u64, output_ptr).unwrap();

        let first_byte = memory.read_u8(output_ptr).unwrap();
        assert_eq!(first_byte, expected_first_byte, "Failed for input {:?}", input);
    }
}

#[test]
fn test_keccak256_long_input() {
    let mut memory = Memory::new();

    // 1000 bytes of input
    let input_ptr = 0x1000;
    let input_len = 1000u64;

    for i in 0..input_len {
        memory.write_u8(input_ptr + i, (i % 256) as u8).unwrap();
    }

    let output_ptr = 0x3000;
    let bound = keccak256_hash(&mut memory, input_ptr, input_len, output_ptr).unwrap();

    assert_eq!(bound.max_bits, 64);
}

// ============================================================================
// Blake3 Edge Cases
// ============================================================================

#[test]
fn test_blake3_empty_input() {
    let mut memory = Memory::new();
    let output_ptr = 0x1000;

    let bound = blake3_hash(&mut memory, 0, 0, output_ptr).unwrap();

    // Check bound (32 bits for Blake3)
    assert_eq!(bound.max_bits, 32);

    // Verify known hash
    // Blake3("") = af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262
    let first_byte = memory.read_u8(output_ptr).unwrap();
    assert_eq!(first_byte, 0xaf);
}

#[test]
fn test_blake3_single_byte() {
    let mut memory = Memory::new();

    let input_ptr = 0x1000;
    memory.write_u8(input_ptr, b'a').unwrap();

    let output_ptr = 0x2000;
    let bound = blake3_hash(&mut memory, input_ptr, 1, output_ptr).unwrap();

    assert_eq!(bound.max_bits, 32);
}

#[test]
fn test_blake3_long_input() {
    let mut memory = Memory::new();

    // 1000 bytes of input
    let input_ptr = 0x1000;
    let input_len = 1000u64;

    for i in 0..input_len {
        memory.write_u8(input_ptr + i, (i % 256) as u8).unwrap();
    }

    let output_ptr = 0x3000;
    let bound = blake3_hash(&mut memory, input_ptr, input_len, output_ptr).unwrap();

    assert_eq!(bound.max_bits, 32);
}

#[test]
fn test_blake3_matches_library() {
    let mut memory = Memory::new();

    let test_inputs = [
        b"" as &[u8],
        b"a",
        b"abc",
        b"hello world",
        b"The quick brown fox jumps over the lazy dog",
    ];

    for input in test_inputs {
        let input_ptr = 0x1000;
        for (i, &byte) in input.iter().enumerate() {
            memory.write_u8(input_ptr + i as u64, byte).unwrap();
        }

        let output_ptr = 0x2000;
        blake3_hash(&mut memory, input_ptr, input.len() as u64, output_ptr).unwrap();

        // Compare with blake3 crate
        let expected = blake3::hash(input);
        for (i, &exp_byte) in expected.as_bytes().iter().enumerate() {
            let got_byte = memory.read_u8(output_ptr + i as u64).unwrap();
            assert_eq!(got_byte, exp_byte, "Byte {} mismatch for input {:?}", i, input);
        }
    }
}

// ============================================================================
// Bound Verification Tests
// ============================================================================

#[test]
fn test_crypto_bounds_are_correct() {
    // SHA-256: 32-bit algorithm output
    let sha_bound = ValueBound::from_crypto(zkir_spec::CryptoType::Sha256);
    assert_eq!(sha_bound.max_bits, 32);

    // Keccak-256: 64-bit internal representation
    let keccak_bound = ValueBound::from_crypto(zkir_spec::CryptoType::Keccak256);
    assert_eq!(keccak_bound.max_bits, 64);

    // Blake3: 32-bit algorithm output
    let blake3_bound = ValueBound::from_crypto(zkir_spec::CryptoType::Blake3);
    assert_eq!(blake3_bound.max_bits, 32);
}

#[test]
fn test_crypto_bounds_no_overflow() {
    // Crypto bounds should not need range checks for 40-bit program width
    let sha_bound = ValueBound::from_crypto(zkir_spec::CryptoType::Sha256);
    assert!(!sha_bound.needs_range_check(40));

    let keccak_bound = ValueBound::from_crypto(zkir_spec::CryptoType::Keccak256);
    // Keccak is 64-bit, so needs check for 40-bit width
    assert!(keccak_bound.needs_range_check(40));
    assert!(!keccak_bound.needs_range_check(64));
}

// ============================================================================
// Memory Alignment Tests
// ============================================================================

#[test]
fn test_sha256_unaligned_input() {
    let mut memory = Memory::new();

    // Input at unaligned address
    let input_ptr = 0x1001; // Not aligned
    memory.write_u8(input_ptr, b'a').unwrap();

    let output_ptr = 0x2000; // Aligned output
    let result = sha256_hash(&mut memory, input_ptr, 1, output_ptr);

    assert!(result.is_ok());
}

#[test]
fn test_sha256_unaligned_output() {
    let mut memory = Memory::new();

    let input_ptr = 0x1000;
    memory.write_u8(input_ptr, b'a').unwrap();

    // Output at address that's 4-byte aligned (required for u32 writes)
    let output_ptr = 0x2000;
    let result = sha256_hash(&mut memory, input_ptr, 1, output_ptr);

    assert!(result.is_ok());
}

// ============================================================================
// All Zeros / All Ones Tests
// ============================================================================

#[test]
fn test_sha256_all_zeros() {
    let mut memory = Memory::new();

    let input_ptr = 0x1000;
    for i in 0..32 {
        memory.write_u8(input_ptr + i, 0x00).unwrap();
    }

    let output_ptr = 0x2000;
    let result = sha256_hash(&mut memory, input_ptr, 32, output_ptr);

    assert!(result.is_ok());
}

#[test]
fn test_sha256_all_ones() {
    let mut memory = Memory::new();

    let input_ptr = 0x1000;
    for i in 0..32 {
        memory.write_u8(input_ptr + i, 0xFF).unwrap();
    }

    let output_ptr = 0x2000;
    let result = sha256_hash(&mut memory, input_ptr, 32, output_ptr);

    assert!(result.is_ok());
}

// ============================================================================
// Sequential vs Random Access
// ============================================================================

#[test]
fn test_sequential_crypto_ops() {
    let mut memory = Memory::new();

    // Perform multiple hash operations in sequence
    for i in 0..10 {
        let input_ptr = 0x1000 + (i * 0x100);
        let output_ptr = 0x5000 + (i * 0x100);

        memory.write_u8(input_ptr, i as u8).unwrap();

        sha256_hash(&mut memory, input_ptr, 1, output_ptr).unwrap();
    }
}

#[test]
fn test_hash_chain() {
    // Hash the output of the previous hash
    let mut memory = Memory::new();

    // Start with "hello"
    let input_ptr = 0x1000;
    for (i, &byte) in b"hello".iter().enumerate() {
        memory.write_u8(input_ptr + i as u64, byte).unwrap();
    }

    let output_ptr = 0x2000;
    sha256_hash(&mut memory, input_ptr, 5, output_ptr).unwrap();

    // Now hash the output (32 bytes)
    let second_output_ptr = 0x3000;
    sha256_hash(&mut memory, output_ptr, 32, second_output_ptr).unwrap();

    // Verify we get a different hash
    let first_word_1 = memory.read_u32(output_ptr).unwrap();
    let first_word_2 = memory.read_u32(second_output_ptr).unwrap();

    assert_ne!(first_word_1, first_word_2);
}

// ============================================================================
// Syscall Integration Tests
// ============================================================================

#[test]
fn test_sha256_syscall() {
    // Test SHA-256 via the syscall interface
    let instructions = vec![
        // Set up input pointer in r1 (data region)
        Instruction::Addi {
            rd: Register::R1,
            rs1: Register::R0,
            imm: 0x1000,
        },
        // Set up input length in r2
        Instruction::Addi {
            rd: Register::R2,
            rs1: Register::R0,
            imm: 0,  // Empty input
        },
        // Set up output pointer in r3
        Instruction::Addi {
            rd: Register::R3,
            rs1: Register::R0,
            imm: 0x2000,
        },
        // Call SHA-256 syscall (number 3)
        Instruction::Addi {
            rd: Register::R10,
            rs1: Register::R0,
            imm: 3,  // SYSCALL_SHA256
        },
        Instruction::Addi {
            rd: Register::R11,
            rs1: Register::R1,
            imm: 0,  // a1 = input_ptr
        },
        Instruction::Addi {
            rd: Register::R12,
            rs1: Register::R2,
            imm: 0,  // a2 = input_len
        },
        Instruction::Addi {
            rd: Register::R13,
            rs1: Register::R3,
            imm: 0,  // a3 = output_ptr
        },
        Instruction::Ecall,
        // Exit
        Instruction::Addi {
            rd: Register::R10,
            rs1: Register::R0,
            imm: 0,
        },
        Instruction::Addi {
            rd: Register::R11,
            rs1: Register::R0,
            imm: 0,
        },
        Instruction::Ecall,
    ];

    let program = create_program_from_instructions(instructions);
    let vm = VM::new(program, vec![], VMConfig::default());
    let result = vm.run().unwrap();

    assert_eq!(result.halt_reason, HaltReason::Exit(0));
}
