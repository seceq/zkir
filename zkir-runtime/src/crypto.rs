//! Cryptographic syscalls for ZKIR v3.4
//!
//! Implements crypto operations with adaptive internal representation:
//! - SHA-256: 32-bit algorithm, 44-bit min internal, 12-bit headroom
//! - Poseidon2: 31-bit algorithm, 40-bit min internal, 9-bit headroom
//! - Keccak-256: 64-bit algorithm, 80-bit min internal, 16-bit headroom
//! - Blake3: 32-bit algorithm, 44-bit min internal, 12-bit headroom
//!
//! # Design
//!
//! Each crypto operation:
//! 1. Reads input from memory (bounded values)
//! 2. Executes with adaptive internal width (max(min_internal, program_bits))
//! 3. Outputs bounded to algorithm_bits (e.g., 32 bits for SHA-256)
//! 4. Returns ValueBound for output (enables range check optimization)

use crate::error::{RuntimeError, Result};
use crate::memory::Memory;
use zkir_spec::{CryptoType, ValueBound, Sha256Witness};
use sha2::{Sha256, Digest};
use sha3::Keccak256;

// SHA-256 constants
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

// SHA-256 initial hash values (first 32 bits of fractional parts of square roots of first 8 primes)
const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// Rotate right
#[inline]
fn rotr(x: u32, n: u32) -> u32 {
    (x >> n) | (x << (32 - n))
}

/// SHA-256 Sigma0
#[inline]
fn sigma0(x: u32) -> u32 {
    rotr(x, 2) ^ rotr(x, 13) ^ rotr(x, 22)
}

/// SHA-256 Sigma1
#[inline]
fn sigma1(x: u32) -> u32 {
    rotr(x, 6) ^ rotr(x, 11) ^ rotr(x, 25)
}

/// SHA-256 sigma0 (lowercase - for message schedule)
#[inline]
fn lower_sigma0(x: u32) -> u32 {
    rotr(x, 7) ^ rotr(x, 18) ^ (x >> 3)
}

/// SHA-256 sigma1 (lowercase - for message schedule)
#[inline]
fn lower_sigma1(x: u32) -> u32 {
    rotr(x, 17) ^ rotr(x, 19) ^ (x >> 10)
}

/// SHA-256 Ch function
#[inline]
fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

/// SHA-256 Maj function
#[inline]
fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

/// SHA-256 hash function (without witness collection)
///
/// Algorithm: 32-bit native (outputs 256 bits = 8×32-bit words)
/// Internal: 44-bit minimum (12-bit headroom for ~320 operations)
/// Output bound: 32 bits per word (tight)
///
/// # Parameters
/// - `memory`: Memory subsystem
/// - `input_ptr`: Pointer to input data
/// - `input_len`: Length of input in bytes
/// - `output_ptr`: Pointer to output buffer (32 bytes)
///
/// # Returns
/// - `Ok(ValueBound)`: Bound for output values (32 bits)
/// - `Err(RuntimeError)`: If memory access fails
pub fn sha256_hash(
    memory: &mut Memory,
    input_ptr: u64,
    input_len: u64,
    output_ptr: u64,
) -> Result<ValueBound> {
    sha256_hash_with_witness(memory, input_ptr, input_len, output_ptr, None)
}

/// Pad message for SHA-256 (single block only)
fn pad_message(input: &[u8]) -> Vec<u8> {
    let mut padded = input.to_vec();
    let msg_len = input.len() as u64;

    // Append '1' bit (0x80)
    padded.push(0x80);

    // Pad with zeros until length ≡ 448 (mod 512), or 56 (mod 64) bytes
    while padded.len() % 64 != 56 {
        padded.push(0);
    }

    // Append message length as 64-bit big-endian
    padded.extend_from_slice(&(msg_len * 8).to_be_bytes());

    padded
}

/// Parse message block into 16 words (big-endian)
fn parse_message_block(block: &[u8]) -> [u32; 16] {
    let mut words = [0u32; 16];
    for i in 0..16 {
        let offset = i * 4;
        words[i] = u32::from_be_bytes([
            block[offset],
            block[offset + 1],
            block[offset + 2],
            block[offset + 3],
        ]);
    }
    words
}

/// Compute SHA-256 message schedule from message block
fn compute_message_schedule(message_block: &[u32; 16]) -> [u32; 64] {
    let mut w = [0u32; 64];

    // First 16 words are from the message block
    w[0..16].copy_from_slice(message_block);

    // Extend the rest using the schedule formula
    for i in 16..64 {
        w[i] = lower_sigma1(w[i - 2])
            .wrapping_add(w[i - 7])
            .wrapping_add(lower_sigma0(w[i - 15]))
            .wrapping_add(w[i - 16]);
    }

    w
}

/// SHA-256 compression function with optional witness collection
fn sha256_compress(
    message_block: &[u32; 16],
    initial_state: [u32; 8],
    mut witness: Option<&mut Sha256Witness>,
) -> [u32; 8] {
    // Compute message schedule
    let w = compute_message_schedule(message_block);

    // Initialize working variables
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = initial_state;

    // Main compression loop (64 rounds)
    for i in 0..64 {
        let t1 = h
            .wrapping_add(sigma1(e))
            .wrapping_add(ch(e, f, g))
            .wrapping_add(K[i])
            .wrapping_add(w[i]);

        let t2 = sigma0(a).wrapping_add(maj(a, b, c));

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);

        // Record round state if witness collection is enabled
        if let Some(wit) = witness.as_mut() {
            wit.record_round(i, [a, b, c, d, e, f, g, h]);
        }
    }

    // Add compressed chunk to current hash value
    [
        initial_state[0].wrapping_add(a),
        initial_state[1].wrapping_add(b),
        initial_state[2].wrapping_add(c),
        initial_state[3].wrapping_add(d),
        initial_state[4].wrapping_add(e),
        initial_state[5].wrapping_add(f),
        initial_state[6].wrapping_add(g),
        initial_state[7].wrapping_add(h),
    ]
}

/// SHA-256 hash function with optional witness collection
///
/// This version can collect intermediate round states for proof generation.
///
/// # Parameters
/// - `memory`: Memory subsystem
/// - `input_ptr`: Pointer to input data
/// - `input_len`: Length of input in bytes
/// - `output_ptr`: Pointer to output buffer (32 bytes)
/// - `witness`: Optional witness to populate with intermediate states
///
/// # Returns
/// - `Ok(ValueBound)`: Bound for output values (32 bits)
/// - `Err(RuntimeError)`: If memory access fails
pub fn sha256_hash_with_witness(
    memory: &mut Memory,
    input_ptr: u64,
    input_len: u64,
    output_ptr: u64,
    mut witness: Option<&mut Sha256Witness>,
) -> Result<ValueBound> {
    // Read input data from memory
    let mut input = Vec::with_capacity(input_len as usize);
    for i in 0..input_len {
        let byte = memory.read_u8(input_ptr + i)?;
        input.push(byte);
    }

    // For witness collection with full round states, we only support single-block messages (< 56 bytes)
    // For longer messages without witness, we use the optimized library
    if witness.is_some() && input_len >= 56 {
        return Err(RuntimeError::Other(
            "SHA-256 witness collection only supports messages < 56 bytes".to_string(),
        ));
    }

    // If no witness collection, use optimized standard library
    if witness.is_none() {
        let mut hasher = Sha256::new();
        hasher.update(&input);
        let hash = hasher.finalize();

        // Write output to memory
        for (i, chunk) in hash.chunks(4).enumerate() {
            let word = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            memory.write_u32(output_ptr + (i * 4) as u64, word)?;
        }

        return Ok(ValueBound::from_crypto(CryptoType::Sha256));
    }

    // Manual SHA-256 computation with witness collection
    let padded = pad_message(&input);

    // We only support single block for now
    if padded.len() != 64 {
        return Err(RuntimeError::Other(
            "Message padding resulted in multiple blocks".to_string(),
        ));
    }

    // Parse message block
    let message_block = parse_message_block(&padded[0..64]);

    // Compute message schedule
    let message_schedule = compute_message_schedule(&message_block);

    // Perform compression with witness collection
    let final_state = if let Some(ref mut w) = witness {
        // Set up witness fields before compression
        w.message_block = message_block;
        w.initial_state = H0;
        w.message_schedule = message_schedule;

        // Run compression and record final state
        let state = sha256_compress(&message_block, H0, Some(w));
        w.final_state = state;
        state
    } else {
        sha256_compress(&message_block, H0, None)
    };

    // Write output to memory (big-endian)
    for (i, &word) in final_state.iter().enumerate() {
        memory.write_u32(output_ptr + (i * 4) as u64, word)?;
    }

    Ok(ValueBound::from_crypto(CryptoType::Sha256))
}

/// Poseidon2 hash function (placeholder)
///
/// Algorithm: 31-bit native (field arithmetic over prime ~2^31)
/// Internal: 40-bit minimum (9-bit headroom for ~200 operations)
/// Output bound: 31 bits (tight)
///
/// TODO: Implement Poseidon2 permutation
pub fn poseidon2_hash(
    _memory: &mut Memory,
    _input_ptr: u64,
    _input_len: u64,
    _output_ptr: u64,
) -> Result<ValueBound> {
    Err(RuntimeError::Other(
        "Poseidon2 not yet implemented".to_string(),
    ))
}

/// Keccak-256 hash function
///
/// Algorithm: 64-bit native (operates on 64-bit lanes)
/// Internal: 80-bit minimum (16-bit headroom for ~50 operations)
/// Output bound: 64 bits per lane (but hash output is 32 bytes = 4×64-bit words)
///
/// # Parameters
/// - `memory`: Memory subsystem
/// - `input_ptr`: Pointer to input data
/// - `input_len`: Length of input in bytes
/// - `output_ptr`: Pointer to output buffer (32 bytes)
///
/// # Returns
/// - `Ok(ValueBound)`: Bound for output values (64 bits)
/// - `Err(RuntimeError)`: If memory access fails
pub fn keccak256_hash(
    memory: &mut Memory,
    input_ptr: u64,
    input_len: u64,
    output_ptr: u64,
) -> Result<ValueBound> {
    // Read input data from memory
    let mut input = Vec::with_capacity(input_len as usize);
    for i in 0..input_len {
        let byte = memory.read_u8(input_ptr + i)?;
        input.push(byte);
    }

    // Compute Keccak-256 hash
    let mut hasher = Keccak256::new();
    hasher.update(&input);
    let hash = hasher.finalize();

    // Write output to memory (32 bytes)
    for (i, &byte) in hash.iter().enumerate() {
        memory.write_u8(output_ptr + i as u64, byte)?;
    }

    Ok(ValueBound::from_crypto(CryptoType::Keccak256))
}

/// Blake3 hash function
///
/// Algorithm: 32-bit native (operates on 32-bit words)
/// Internal: 44-bit minimum (12-bit headroom for ~400 operations)
/// Output bound: 32 bits per word
///
/// # Parameters
/// - `memory`: Memory subsystem
/// - `input_ptr`: Pointer to input data
/// - `input_len`: Length of input in bytes
/// - `output_ptr`: Pointer to output buffer (32 bytes)
///
/// # Returns
/// - `Ok(ValueBound)`: Bound for output values (32 bits)
/// - `Err(RuntimeError)`: If memory access fails
pub fn blake3_hash(
    memory: &mut Memory,
    input_ptr: u64,
    input_len: u64,
    output_ptr: u64,
) -> Result<ValueBound> {
    // Read input data from memory
    let mut input = Vec::with_capacity(input_len as usize);
    for i in 0..input_len {
        let byte = memory.read_u8(input_ptr + i)?;
        input.push(byte);
    }

    // Compute Blake3 hash
    let hash = blake3::hash(&input);

    // Write output to memory (32 bytes)
    for (i, &byte) in hash.as_bytes().iter().enumerate() {
        memory.write_u8(output_ptr + i as u64, byte)?;
    }

    Ok(ValueBound::from_crypto(CryptoType::Blake3))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty() {
        let mut memory = Memory::new();

        // SHA-256 of empty input
        let output_ptr = 0x1000;
        let bound = sha256_hash(&mut memory, 0, 0, output_ptr).unwrap();

        // Check bound
        assert_eq!(bound.max_bits, 32);

        // Verify output: SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let expected = [
            0xe3b0c442, 0x98fc1c14, 0x9afbf4c8, 0x996fb924, 0x27ae41e4, 0x649b934c, 0xa495991b,
            0x7852b855,
        ];
        for (i, &exp) in expected.iter().enumerate() {
            let word = memory.read_u32(output_ptr + (i * 4) as u64).unwrap();
            assert_eq!(
                word, exp,
                "Word {} mismatch: got {:#x}, expected {:#x}",
                i, word, exp
            );
        }
    }

    #[test]
    fn test_sha256_hello() {
        let mut memory = Memory::new();

        // Write "hello" to memory
        let input_ptr = 0x2000;
        memory.write_u8(input_ptr, b'h').unwrap();
        memory.write_u8(input_ptr + 1, b'e').unwrap();
        memory.write_u8(input_ptr + 2, b'l').unwrap();
        memory.write_u8(input_ptr + 3, b'l').unwrap();
        memory.write_u8(input_ptr + 4, b'o').unwrap();

        // SHA-256 of "hello"
        let output_ptr = 0x3000;
        let bound = sha256_hash(&mut memory, input_ptr, 5, output_ptr).unwrap();

        // Check bound
        assert_eq!(bound.max_bits, 32);

        // Verify output: SHA-256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        let expected = [
            0x2cf24dba, 0x5fb0a30e, 0x26e83b2a, 0xc5b9e29e, 0x1b161e5c, 0x1fa7425e, 0x73043362,
            0x938b9824,
        ];
        for (i, &exp) in expected.iter().enumerate() {
            let word = memory.read_u32(output_ptr + (i * 4) as u64).unwrap();
            assert_eq!(
                word, exp,
                "Word {} mismatch: got {:#x}, expected {:#x}",
                i, word, exp
            );
        }
    }

    #[test]
    fn test_poseidon2_not_implemented() {
        let mut memory = Memory::new();
        let result = poseidon2_hash(&mut memory, 0, 0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_keccak256_empty() {
        let mut memory = Memory::new();

        // Keccak-256 of empty input
        let output_ptr = 0x1000;
        let bound = keccak256_hash(&mut memory, 0, 0, output_ptr).unwrap();

        // Check bound (64 bits for Keccak)
        assert_eq!(bound.max_bits, 64);

        // Verify output: Keccak-256("") = c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
        let expected: [u8; 32] = [
            0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c,
            0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0,
            0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b,
            0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70,
        ];
        for (i, &exp) in expected.iter().enumerate() {
            let byte = memory.read_u8(output_ptr + i as u64).unwrap();
            assert_eq!(byte, exp, "Byte {} mismatch: got {:#x}, expected {:#x}", i, byte, exp);
        }
    }

    #[test]
    fn test_keccak256_hello() {
        let mut memory = Memory::new();

        // Write "hello" to memory
        let input_ptr = 0x2000;
        memory.write_u8(input_ptr, b'h').unwrap();
        memory.write_u8(input_ptr + 1, b'e').unwrap();
        memory.write_u8(input_ptr + 2, b'l').unwrap();
        memory.write_u8(input_ptr + 3, b'l').unwrap();
        memory.write_u8(input_ptr + 4, b'o').unwrap();

        // Keccak-256 of "hello"
        let output_ptr = 0x3000;
        let bound = keccak256_hash(&mut memory, input_ptr, 5, output_ptr).unwrap();

        // Check bound
        assert_eq!(bound.max_bits, 64);

        // Verify output: Keccak-256("hello") = 1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8
        let expected: [u8; 32] = [
            0x1c, 0x8a, 0xff, 0x95, 0x06, 0x85, 0xc2, 0xed,
            0x4b, 0xc3, 0x17, 0x4f, 0x34, 0x72, 0x28, 0x7b,
            0x56, 0xd9, 0x51, 0x7b, 0x9c, 0x94, 0x81, 0x27,
            0x31, 0x9a, 0x09, 0xa7, 0xa3, 0x6d, 0xea, 0xc8,
        ];
        for (i, &exp) in expected.iter().enumerate() {
            let byte = memory.read_u8(output_ptr + i as u64).unwrap();
            assert_eq!(byte, exp, "Byte {} mismatch: got {:#x}, expected {:#x}", i, byte, exp);
        }
    }

    #[test]
    fn test_blake3_empty() {
        let mut memory = Memory::new();

        // Blake3 of empty input
        let output_ptr = 0x1000;
        let bound = blake3_hash(&mut memory, 0, 0, output_ptr).unwrap();

        // Check bound (32 bits for Blake3)
        assert_eq!(bound.max_bits, 32);

        // Verify output: Blake3("") = af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262
        let expected: [u8; 32] = [
            0xaf, 0x13, 0x49, 0xb9, 0xf5, 0xf9, 0xa1, 0xa6,
            0xa0, 0x40, 0x4d, 0xea, 0x36, 0xdc, 0xc9, 0x49,
            0x9b, 0xcb, 0x25, 0xc9, 0xad, 0xc1, 0x12, 0xb7,
            0xcc, 0x9a, 0x93, 0xca, 0xe4, 0x1f, 0x32, 0x62,
        ];
        for (i, &exp) in expected.iter().enumerate() {
            let byte = memory.read_u8(output_ptr + i as u64).unwrap();
            assert_eq!(byte, exp, "Byte {} mismatch: got {:#x}, expected {:#x}", i, byte, exp);
        }
    }

    #[test]
    fn test_blake3_hello() {
        let mut memory = Memory::new();

        // Write "hello" to memory
        let input_ptr = 0x2000;
        memory.write_u8(input_ptr, b'h').unwrap();
        memory.write_u8(input_ptr + 1, b'e').unwrap();
        memory.write_u8(input_ptr + 2, b'l').unwrap();
        memory.write_u8(input_ptr + 3, b'l').unwrap();
        memory.write_u8(input_ptr + 4, b'o').unwrap();

        // Blake3 of "hello"
        let output_ptr = 0x3000;
        let bound = blake3_hash(&mut memory, input_ptr, 5, output_ptr).unwrap();

        // Check bound
        assert_eq!(bound.max_bits, 32);

        // Verify with blake3 crate
        let expected = blake3::hash(b"hello");
        for (i, &exp) in expected.as_bytes().iter().enumerate() {
            let byte = memory.read_u8(output_ptr + i as u64).unwrap();
            assert_eq!(byte, exp, "Byte {} mismatch: got {:#x}, expected {:#x}", i, byte, exp);
        }
    }

    #[test]
    fn test_sha256_witness_collection() {
        let mut memory = Memory::new();

        // Write "hello" to memory
        let input_ptr = 0x2000;
        memory.write_u8(input_ptr, b'h').unwrap();
        memory.write_u8(input_ptr + 1, b'e').unwrap();
        memory.write_u8(input_ptr + 2, b'l').unwrap();
        memory.write_u8(input_ptr + 3, b'l').unwrap();
        memory.write_u8(input_ptr + 4, b'o').unwrap();

        // Create witness
        let mut witness = Sha256Witness::new(0);

        // SHA-256 with witness collection
        let output_ptr = 0x3000;
        let bound = sha256_hash_with_witness(
            &mut memory,
            input_ptr,
            5,
            output_ptr,
            Some(&mut witness),
        )
        .unwrap();

        // Check bound
        assert_eq!(bound.max_bits, 32);

        // Verify final state is populated
        let expected = [
            0x2cf24dba, 0x5fb0a30e, 0x26e83b2a, 0xc5b9e29e, 0x1b161e5c, 0x1fa7425e, 0x73043362,
            0x938b9824,
        ];
        assert_eq!(witness.final_state, expected);

        // Verify timestamp
        assert_eq!(witness.timestamp, 0);
    }

    #[test]
    fn test_sha256_witness_long_message_unsupported() {
        let mut memory = Memory::new();

        // Write 60 bytes (> 56 byte limit for single block)
        let input_ptr = 0x1000;
        for i in 0..60 {
            memory.write_u8(input_ptr + i, (i % 256) as u8).unwrap();
        }

        let mut witness = Sha256Witness::new(0);
        let output_ptr = 0x2000;

        // Should fail because message is too long for witness collection
        let result = sha256_hash_with_witness(
            &mut memory,
            input_ptr,
            60,
            output_ptr,
            Some(&mut witness),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_sha256_full_witness_empty() {
        let mut memory = Memory::new();
        let mut witness = Sha256Witness::new(42);

        let output_ptr = 0x1000;
        sha256_hash_with_witness(&mut memory, 0, 0, output_ptr, Some(&mut witness)).unwrap();

        // Check that all 64 rounds were recorded
        assert_eq!(witness.num_rounds(), 64);

        // Check initial state (SHA-256 IV)
        assert_eq!(witness.initial_state, H0);

        // Check final state matches expected hash of empty string
        let expected = [
            0xe3b0c442, 0x98fc1c14, 0x9afbf4c8, 0x996fb924,
            0x27ae41e4, 0x649b934c, 0xa495991b, 0x7852b855,
        ];
        assert_eq!(witness.final_state, expected);

        // Check timestamp
        assert_eq!(witness.timestamp, 42);

        // Verify message schedule was computed
        assert!(witness.message_schedule.iter().any(|&x| x != 0));
    }

    #[test]
    fn test_sha256_full_witness_hello() {
        let mut memory = Memory::new();

        // Write "hello" to memory
        let input_ptr = 0x2000;
        memory.write_u8(input_ptr, b'h').unwrap();
        memory.write_u8(input_ptr + 1, b'e').unwrap();
        memory.write_u8(input_ptr + 2, b'l').unwrap();
        memory.write_u8(input_ptr + 3, b'l').unwrap();
        memory.write_u8(input_ptr + 4, b'o').unwrap();

        let mut witness = Sha256Witness::new(100);
        let output_ptr = 0x3000;

        sha256_hash_with_witness(&mut memory, input_ptr, 5, output_ptr, Some(&mut witness))
            .unwrap();

        // Check all 64 rounds recorded
        assert_eq!(witness.num_rounds(), 64);

        // Check initial state
        assert_eq!(witness.initial_state, H0);

        // Check final state
        let expected = [
            0x2cf24dba, 0x5fb0a30e, 0x26e83b2a, 0xc5b9e29e,
            0x1b161e5c, 0x1fa7425e, 0x73043362, 0x938b9824,
        ];
        assert_eq!(witness.final_state, expected);

        // Verify message block was parsed (should contain "hello" in first word, big-endian)
        // "hell" = 0x68656c6c, "o\x80\x00\x00" = 0x6f800000 (with padding)
        assert_eq!(witness.message_block[0], 0x68656c6c);
        assert_eq!(witness.message_block[1], 0x6f800000);

        // Check that message schedule extends beyond first 16 words
        assert!(witness.message_schedule[16..].iter().any(|&x| x != 0));

        // Verify round states are different (compression is working)
        let first_round = witness.round_states[0];
        let last_round = witness.round_states[63];
        assert_ne!(first_round, last_round);
    }

    #[test]
    fn test_sha256_without_witness_still_works() {
        let mut memory = Memory::new();

        // Write test data
        let input_ptr = 0x1000;
        for i in 0..10 {
            memory.write_u8(input_ptr + i, b'a' + (i as u8 % 26)).unwrap();
        }

        let output_ptr = 0x2000;

        // Hash without witness (should use optimized path)
        sha256_hash(&mut memory, input_ptr, 10, output_ptr).unwrap();

        // Verify output was written (just check it's not all zeros)
        let first_word = memory.read_u32(output_ptr).unwrap();
        assert_ne!(first_word, 0);
    }

    #[test]
    fn test_sha256_witness_round_progression() {
        let mut memory = Memory::new();

        // Single byte input
        let input_ptr = 0x1000;
        memory.write_u8(input_ptr, 0x61).unwrap(); // 'a'

        let mut witness = Sha256Witness::new(0);
        let output_ptr = 0x2000;

        sha256_hash_with_witness(&mut memory, input_ptr, 1, output_ptr, Some(&mut witness))
            .unwrap();

        // Verify each round produces a different state
        for i in 0..63 {
            assert_ne!(
                witness.round_states[i],
                witness.round_states[i + 1],
                "Round {} and {} should have different states",
                i,
                i + 1
            );
        }
    }

    #[test]
    fn test_sha256_message_schedule_computation() {
        let mut memory = Memory::new();

        let input_ptr = 0x1000;
        memory.write_u8(input_ptr, b'X').unwrap();

        let mut witness = Sha256Witness::new(0);
        let output_ptr = 0x2000;

        sha256_hash_with_witness(&mut memory, input_ptr, 1, output_ptr, Some(&mut witness))
            .unwrap();

        // First 16 words of message schedule should match message block
        for i in 0..16 {
            assert_eq!(witness.message_schedule[i], witness.message_block[i]);
        }

        // Words 16-63 should be computed (non-zero for most messages)
        // At least some of them should be non-zero
        let extended_schedule = &witness.message_schedule[16..];
        assert!(extended_schedule.iter().any(|&x| x != 0));
    }

    #[test]
    fn test_sha256_correctness_against_library() {
        let mut memory = Memory::new();

        // Test with various inputs
        let test_cases = [
            b"" as &[u8],
            b"a",
            b"abc",
            b"message digest",
            b"abcdefghijklmnopqrstuvwxyz",
            b"The quick brown fox jumps over the lazy dog",
        ];

        for (idx, &input) in test_cases.iter().enumerate() {
            if input.len() >= 56 {
                continue; // Skip messages that are too long for witness collection
            }

            // Write input to memory
            let input_ptr = 0x1000;
            for (i, &byte) in input.iter().enumerate() {
                memory.write_u8(input_ptr + i as u64, byte).unwrap();
            }

            // Compute with witness
            let mut witness = Sha256Witness::new(idx as u64);
            let output_ptr = 0x2000;
            sha256_hash_with_witness(
                &mut memory,
                input_ptr,
                input.len() as u64,
                output_ptr,
                Some(&mut witness),
            )
            .unwrap();

            // Compute with standard library
            use sha2::{Digest, Sha256};
            let expected = Sha256::digest(input);

            // Compare
            for (i, chunk) in expected.chunks(4).enumerate() {
                let expected_word = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                assert_eq!(
                    witness.final_state[i], expected_word,
                    "Mismatch at word {} for input {:?}",
                    i, input
                );
            }
        }
    }
}
