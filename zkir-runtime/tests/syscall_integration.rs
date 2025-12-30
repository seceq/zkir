//! Syscall integration tests for ZKIR v3.4
//!
//! Tests syscall handling with full VM state and memory.

use zkir_runtime::{Memory, VMState, IOHandler, handle_syscall};
use zkir_runtime::syscall::{
    SYSCALL_EXIT, SYSCALL_READ, SYSCALL_WRITE,
    SYSCALL_SHA256, SYSCALL_KECCAK256, SYSCALL_BLAKE3, SYSCALL_POSEIDON2,
};
use zkir_runtime::state::HaltReason;
use zkir_spec::Register;

// ============================================================================
// Exit Syscall Tests
// ============================================================================

#[test]
fn test_exit_syscall_success_code() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![]);

    // Set up exit syscall with code 0 (success)
    state.write_reg(Register::R10, SYSCALL_EXIT);
    state.write_reg(Register::R11, 0);

    handle_syscall(&mut state, &mut memory, &mut io).unwrap();

    assert!(state.is_halted());
    assert_eq!(state.halt_reason, Some(HaltReason::Exit(0)));
}

#[test]
fn test_exit_syscall_error_code() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![]);

    // Set up exit syscall with error code 1
    state.write_reg(Register::R10, SYSCALL_EXIT);
    state.write_reg(Register::R11, 1);

    handle_syscall(&mut state, &mut memory, &mut io).unwrap();

    assert!(state.is_halted());
    assert_eq!(state.halt_reason, Some(HaltReason::Exit(1)));
}

#[test]
fn test_exit_syscall_large_code() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![]);

    // Set up exit syscall with large exit code
    state.write_reg(Register::R10, SYSCALL_EXIT);
    state.write_reg(Register::R11, 255);

    handle_syscall(&mut state, &mut memory, &mut io).unwrap();

    assert!(state.is_halted());
    assert_eq!(state.halt_reason, Some(HaltReason::Exit(255)));
}

// ============================================================================
// Read Syscall Tests
// ============================================================================

#[test]
fn test_read_syscall_single_value() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![42]);

    // Set up read syscall
    state.write_reg(Register::R10, SYSCALL_READ);

    handle_syscall(&mut state, &mut memory, &mut io).unwrap();

    // Result should be in R10
    assert_eq!(state.read_reg(Register::R10), 42);
}

#[test]
fn test_read_syscall_multiple_values() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![100, 200, 300]);

    // First read
    state.write_reg(Register::R10, SYSCALL_READ);
    handle_syscall(&mut state, &mut memory, &mut io).unwrap();
    assert_eq!(state.read_reg(Register::R10), 100);

    // Second read
    state.write_reg(Register::R10, SYSCALL_READ);
    handle_syscall(&mut state, &mut memory, &mut io).unwrap();
    assert_eq!(state.read_reg(Register::R10), 200);

    // Third read
    state.write_reg(Register::R10, SYSCALL_READ);
    handle_syscall(&mut state, &mut memory, &mut io).unwrap();
    assert_eq!(state.read_reg(Register::R10), 300);
}

#[test]
fn test_read_syscall_exhausted() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![42]);

    // First read - should get 42
    state.write_reg(Register::R10, SYSCALL_READ);
    handle_syscall(&mut state, &mut memory, &mut io).unwrap();
    assert_eq!(state.read_reg(Register::R10), 42);

    // Second read - should get 0 (exhausted)
    state.write_reg(Register::R10, SYSCALL_READ);
    handle_syscall(&mut state, &mut memory, &mut io).unwrap();
    assert_eq!(state.read_reg(Register::R10), 0);

    // Verify input is exhausted
    assert!(io.inputs_exhausted());
}

#[test]
fn test_read_syscall_empty_input() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![]);

    // Read with no input - should get 0
    state.write_reg(Register::R10, SYSCALL_READ);
    handle_syscall(&mut state, &mut memory, &mut io).unwrap();
    assert_eq!(state.read_reg(Register::R10), 0);
}

// ============================================================================
// Write Syscall Tests
// ============================================================================

#[test]
fn test_write_syscall_single_value() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![]);

    // Set up write syscall
    state.write_reg(Register::R10, SYSCALL_WRITE);
    state.write_reg(Register::R11, 123);

    handle_syscall(&mut state, &mut memory, &mut io).unwrap();

    assert_eq!(io.outputs(), &[123]);
}

#[test]
fn test_write_syscall_multiple_values() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![]);

    // Write multiple values
    for value in [10, 20, 30, 40] {
        state.write_reg(Register::R10, SYSCALL_WRITE);
        state.write_reg(Register::R11, value);
        handle_syscall(&mut state, &mut memory, &mut io).unwrap();
    }

    assert_eq!(io.outputs(), &[10, 20, 30, 40]);
}

#[test]
fn test_write_syscall_large_value() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![]);

    // Write large value
    state.write_reg(Register::R10, SYSCALL_WRITE);
    state.write_reg(Register::R11, 0xFFFFFFFF);

    handle_syscall(&mut state, &mut memory, &mut io).unwrap();

    assert_eq!(io.outputs(), &[0xFFFFFFFF]);
}

// ============================================================================
// Read/Write Combined Tests
// ============================================================================

#[test]
fn test_read_process_write() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![5, 10, 15]);

    // Read three values, add them, and write result
    let mut sum = 0u64;

    for _ in 0..3 {
        state.write_reg(Register::R10, SYSCALL_READ);
        handle_syscall(&mut state, &mut memory, &mut io).unwrap();
        sum += state.read_reg(Register::R10);
    }

    // Write the sum
    state.write_reg(Register::R10, SYSCALL_WRITE);
    state.write_reg(Register::R11, sum);
    handle_syscall(&mut state, &mut memory, &mut io).unwrap();

    assert_eq!(io.outputs(), &[30]); // 5 + 10 + 15 = 30
}

#[test]
fn test_echo_input_to_output() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![1, 2, 3, 4, 5]);

    // Echo all inputs to output
    for _ in 0..5 {
        // Read
        state.write_reg(Register::R10, SYSCALL_READ);
        handle_syscall(&mut state, &mut memory, &mut io).unwrap();
        let value = state.read_reg(Register::R10);

        // Write
        state.write_reg(Register::R10, SYSCALL_WRITE);
        state.write_reg(Register::R11, value);
        handle_syscall(&mut state, &mut memory, &mut io).unwrap();
    }

    assert_eq!(io.outputs(), &[1, 2, 3, 4, 5]);
}

// ============================================================================
// Invalid Syscall Tests
// ============================================================================

#[test]
fn test_invalid_syscall() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![]);

    // Invalid syscall number
    state.write_reg(Register::R10, 999);

    let result = handle_syscall(&mut state, &mut memory, &mut io);
    assert!(result.is_err());
}

#[test]
fn test_syscall_7_invalid() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![]);

    // Syscall 7 is not defined
    state.write_reg(Register::R10, 7);

    let result = handle_syscall(&mut state, &mut memory, &mut io);
    assert!(result.is_err());
}

// ============================================================================
// SHA-256 Syscall Tests
// ============================================================================

#[test]
fn test_sha256_syscall_hello() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![]);

    // Write "hello" to memory at aligned address
    let input_ptr = 0x1000u64;
    memory.write_u8(input_ptr, b'h').unwrap();
    memory.write_u8(input_ptr + 1, b'e').unwrap();
    memory.write_u8(input_ptr + 2, b'l').unwrap();
    memory.write_u8(input_ptr + 3, b'l').unwrap();
    memory.write_u8(input_ptr + 4, b'o').unwrap();

    // Set up SHA-256 syscall
    let output_ptr = 0x2000u64;
    state.write_reg(Register::R10, SYSCALL_SHA256);
    state.write_reg(Register::R11, input_ptr);
    state.write_reg(Register::R12, 5); // "hello" length
    state.write_reg(Register::R13, output_ptr);

    handle_syscall(&mut state, &mut memory, &mut io).unwrap();

    // Check return value (should be 0 for success)
    assert_eq!(state.read_reg(Register::R10), 0);

    // Verify hash output (SHA-256("hello"))
    let expected = [
        0x2cf24dba, 0x5fb0a30e, 0x26e83b2a, 0xc5b9e29e,
        0x1b161e5c, 0x1fa7425e, 0x73043362, 0x938b9824,
    ];
    for (i, &exp) in expected.iter().enumerate() {
        let word = memory.read_u32(output_ptr + (i * 4) as u64).unwrap();
        assert_eq!(word, exp, "SHA-256 word {} mismatch", i);
    }
}

#[test]
fn test_sha256_syscall_empty() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![]);

    // Set up SHA-256 syscall with empty input
    let input_ptr = 0x1000u64;
    let output_ptr = 0x2000u64;
    state.write_reg(Register::R10, SYSCALL_SHA256);
    state.write_reg(Register::R11, input_ptr);
    state.write_reg(Register::R12, 0); // Empty input
    state.write_reg(Register::R13, output_ptr);

    handle_syscall(&mut state, &mut memory, &mut io).unwrap();

    // Check return value (should be 0 for success)
    assert_eq!(state.read_reg(Register::R10), 0);

    // SHA-256 of empty string: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    let expected = [
        0xe3b0c442, 0x98fc1c14, 0x9afbf4c8, 0x996fb924,
        0x27ae41e4, 0x649b934c, 0xa495991b, 0x7852b855,
    ];
    for (i, &exp) in expected.iter().enumerate() {
        let word = memory.read_u32(output_ptr + (i * 4) as u64).unwrap();
        assert_eq!(word, exp, "SHA-256 empty word {} mismatch", i);
    }
}

// ============================================================================
// Keccak-256 Syscall Tests
// ============================================================================

#[test]
fn test_keccak256_syscall() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![]);

    // Write "hello" to memory
    let input_ptr = 0x1000u64;
    for (i, byte) in b"hello".iter().enumerate() {
        memory.write_u8(input_ptr + i as u64, *byte).unwrap();
    }

    // Set up Keccak-256 syscall
    let output_ptr = 0x2000u64;
    state.write_reg(Register::R10, SYSCALL_KECCAK256);
    state.write_reg(Register::R11, input_ptr);
    state.write_reg(Register::R12, 5);
    state.write_reg(Register::R13, output_ptr);

    handle_syscall(&mut state, &mut memory, &mut io).unwrap();

    // Check return value
    assert_eq!(state.read_reg(Register::R10), 0);
}

// ============================================================================
// Blake3 Syscall Tests
// ============================================================================

#[test]
fn test_blake3_syscall() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![]);

    // Write "hello" to memory
    let input_ptr = 0x1000u64;
    for (i, byte) in b"hello".iter().enumerate() {
        memory.write_u8(input_ptr + i as u64, *byte).unwrap();
    }

    // Set up Blake3 syscall
    let output_ptr = 0x2000u64;
    state.write_reg(Register::R10, SYSCALL_BLAKE3);
    state.write_reg(Register::R11, input_ptr);
    state.write_reg(Register::R12, 5);
    state.write_reg(Register::R13, output_ptr);

    handle_syscall(&mut state, &mut memory, &mut io).unwrap();

    // Check return value
    assert_eq!(state.read_reg(Register::R10), 0);
}

// ============================================================================
// Poseidon2 Syscall Tests
// ============================================================================

#[test]
fn test_poseidon2_syscall_not_implemented() {
    let mut state = VMState::new(0);
    let mut memory = Memory::new();
    let mut io = IOHandler::new(vec![]);

    // Write some data to memory
    let input_ptr = 0x1000u64;
    for (i, byte) in [1u8, 2, 3, 4].iter().enumerate() {
        memory.write_u8(input_ptr + i as u64, *byte).unwrap();
    }

    // Set up Poseidon2 syscall
    let output_ptr = 0x2000u64;
    state.write_reg(Register::R10, SYSCALL_POSEIDON2);
    state.write_reg(Register::R11, input_ptr);
    state.write_reg(Register::R12, 4);
    state.write_reg(Register::R13, output_ptr);

    // Poseidon2 is not yet implemented, should return an error
    let result = handle_syscall(&mut state, &mut memory, &mut io);
    assert!(result.is_err());
}

// ============================================================================
// IOHandler Tests
// ============================================================================

#[test]
fn test_io_handler_creation() {
    let io = IOHandler::new(vec![1, 2, 3]);
    assert!(!io.inputs_exhausted());
    assert!(io.outputs().is_empty());
}

#[test]
fn test_io_handler_empty() {
    let io = IOHandler::new(vec![]);
    assert!(io.inputs_exhausted());
    assert!(io.outputs().is_empty());
}

#[test]
fn test_io_handler_read_write_independence() {
    let mut io = IOHandler::new(vec![100, 200]);

    // Read and write are independent operations
    io.write(999);
    assert_eq!(io.read(), 100);
    io.write(888);
    assert_eq!(io.read(), 200);
    io.write(777);

    assert_eq!(io.outputs(), &[999, 888, 777]);
    assert!(io.inputs_exhausted());
}

// ============================================================================
// Syscall Constants Tests
// ============================================================================

#[test]
fn test_syscall_constants() {
    assert_eq!(SYSCALL_EXIT, 0);
    assert_eq!(SYSCALL_READ, 1);
    assert_eq!(SYSCALL_WRITE, 2);
    assert_eq!(SYSCALL_SHA256, 3);
    assert_eq!(SYSCALL_POSEIDON2, 4);
    assert_eq!(SYSCALL_KECCAK256, 5);
    assert_eq!(SYSCALL_BLAKE3, 6);
}

#[test]
fn test_syscall_constants_unique() {
    let syscalls = [
        SYSCALL_EXIT,
        SYSCALL_READ,
        SYSCALL_WRITE,
        SYSCALL_SHA256,
        SYSCALL_POSEIDON2,
        SYSCALL_KECCAK256,
        SYSCALL_BLAKE3,
    ];

    // All syscall numbers should be unique
    for (i, &s1) in syscalls.iter().enumerate() {
        for (j, &s2) in syscalls.iter().enumerate() {
            if i != j {
                assert_ne!(s1, s2, "Syscall {} and {} have same number", i, j);
            }
        }
    }
}
