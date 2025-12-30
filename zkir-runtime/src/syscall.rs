//! Syscall handling for ZKIR v3.4
//!
//! Implements I/O and crypto syscalls:
//! - Exit (0): Halt with exit code
//! - Read (1): Read from input tape
//! - Write (2): Write to output tape
//! - SHA-256 (3): Cryptographic hash
//! - Poseidon2 (4): ZK-friendly hash
//! - Keccak-256 (5): Ethereum-compatible hash
//! - Blake3 (6): High-performance hash

use crate::error::{RuntimeError, Result};
use crate::state::{VMState, HaltReason};
use crate::memory::Memory;
use crate::crypto;

/// Syscall numbers
pub const SYSCALL_EXIT: u64 = 0;
pub const SYSCALL_READ: u64 = 1;
pub const SYSCALL_WRITE: u64 = 2;
pub const SYSCALL_SHA256: u64 = 3;
pub const SYSCALL_POSEIDON2: u64 = 4;
pub const SYSCALL_KECCAK256: u64 = 5;
pub const SYSCALL_BLAKE3: u64 = 6;

/// I/O handler for syscalls
///
/// Maintains input and output tapes for the VM.
#[derive(Debug, Clone)]
pub struct IOHandler {
    /// Input tape (consumed sequentially)
    inputs: Vec<u64>,

    /// Current position in input tape
    input_pos: usize,

    /// Output tape (written sequentially)
    outputs: Vec<u64>,
}

impl IOHandler {
    /// Create new I/O handler with given inputs
    pub fn new(inputs: Vec<u64>) -> Self {
        Self {
            inputs,
            input_pos: 0,
            outputs: Vec::new(),
        }
    }

    /// Read next value from input tape
    ///
    /// Returns 0 if input is exhausted.
    pub fn read(&mut self) -> u64 {
        if self.input_pos < self.inputs.len() {
            let value = self.inputs[self.input_pos];
            self.input_pos += 1;
            value
        } else {
            0 // Return 0 if no more inputs
        }
    }

    /// Write value to output tape
    pub fn write(&mut self, value: u64) {
        self.outputs.push(value);
    }

    /// Get outputs
    pub fn outputs(&self) -> &[u64] {
        &self.outputs
    }

    /// Check if all inputs have been consumed
    pub fn inputs_exhausted(&self) -> bool {
        self.input_pos >= self.inputs.len()
    }
}

/// Handle a syscall
///
/// Syscall convention:
/// - a0 (R10): syscall number
/// - a1 (R11): first argument (input_ptr for crypto)
/// - a2 (R12): second argument (input_len for crypto)
/// - a3 (R13): third argument (output_ptr for crypto)
/// - Return value in a0 (R10)
///
/// Crypto syscalls:
/// - SHA-256: a1=input_ptr, a2=input_len, a3=output_ptr (32 bytes)
/// - Poseidon2: a1=input_ptr, a2=input_len, a3=output_ptr
/// - Keccak-256: a1=input_ptr, a2=input_len, a3=output_ptr (32 bytes)
/// - Blake3: a1=input_ptr, a2=input_len, a3=output_ptr (32 bytes)
pub fn handle_syscall(state: &mut VMState, memory: &mut Memory, io: &mut IOHandler) -> Result<()> {
    use zkir_spec::Register;

    let syscall_num = state.read_reg(Register::R10); // a0

    match syscall_num {
        SYSCALL_EXIT => {
            // Exit with code from a1 (R11)
            let exit_code = state.read_reg(Register::R11);
            state.halt(HaltReason::Exit(exit_code));
            Ok(())
        }

        SYSCALL_READ => {
            // Read from input tape into a0 (R10)
            let value = io.read();
            state.write_reg(Register::R10, value);
            Ok(())
        }

        SYSCALL_WRITE => {
            // Write from a1 (R11) to output tape
            let value = state.read_reg(Register::R11);
            io.write(value);
            Ok(())
        }

        SYSCALL_SHA256 => {
            // SHA-256: a1=input_ptr, a2=input_len, a3=output_ptr
            let input_ptr = state.read_reg(Register::R11);
            let input_len = state.read_reg(Register::R12);
            let output_ptr = state.read_reg(Register::R13);

            let bound = crypto::sha256_hash(memory, input_ptr, input_len, output_ptr)?;

            // Store bound in state for range check optimization (via special bound tracking)
            // For now, return success in a0
            state.write_reg(Register::R10, 0);

            // Write output bound to a designated register (R14 = t4)
            // This allows the program to track crypto output bounds
            state.write_bound(Register::R14, bound);

            Ok(())
        }

        SYSCALL_POSEIDON2 => {
            // Poseidon2: a1=input_ptr, a2=input_len, a3=output_ptr
            let input_ptr = state.read_reg(Register::R11);
            let input_len = state.read_reg(Register::R12);
            let output_ptr = state.read_reg(Register::R13);

            let _bound = crypto::poseidon2_hash(memory, input_ptr, input_len, output_ptr)?;
            state.write_reg(Register::R10, 0);
            Ok(())
        }

        SYSCALL_KECCAK256 => {
            // Keccak-256: a1=input_ptr, a2=input_len, a3=output_ptr
            let input_ptr = state.read_reg(Register::R11);
            let input_len = state.read_reg(Register::R12);
            let output_ptr = state.read_reg(Register::R13);

            let _bound = crypto::keccak256_hash(memory, input_ptr, input_len, output_ptr)?;
            state.write_reg(Register::R10, 0);
            Ok(())
        }

        SYSCALL_BLAKE3 => {
            // Blake3: a1=input_ptr, a2=input_len, a3=output_ptr
            let input_ptr = state.read_reg(Register::R11);
            let input_len = state.read_reg(Register::R12);
            let output_ptr = state.read_reg(Register::R13);

            let _bound = crypto::blake3_hash(memory, input_ptr, input_len, output_ptr)?;
            state.write_reg(Register::R10, 0);
            Ok(())
        }

        _ => Err(RuntimeError::InvalidSyscall {
            syscall: syscall_num,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zkir_spec::Register;

    #[test]
    fn test_io_handler_read() {
        let mut io = IOHandler::new(vec![10, 20, 30]);

        assert_eq!(io.read(), 10);
        assert_eq!(io.read(), 20);
        assert_eq!(io.read(), 30);
        assert_eq!(io.read(), 0); // Exhausted, returns 0
        assert!(io.inputs_exhausted());
    }

    #[test]
    fn test_io_handler_write() {
        let mut io = IOHandler::new(vec![]);

        io.write(100);
        io.write(200);
        io.write(300);

        assert_eq!(io.outputs(), &[100, 200, 300]);
    }

    #[test]
    fn test_syscall_exit() {
        let mut state = VMState::new(0);
        let mut memory = Memory::new();
        let mut io = IOHandler::new(vec![]);

        // Set up exit syscall with code 42
        state.write_reg(Register::R10, SYSCALL_EXIT);
        state.write_reg(Register::R11, 42);

        handle_syscall(&mut state, &mut memory, &mut io).unwrap();

        assert!(state.is_halted());
        assert_eq!(state.halt_reason, Some(HaltReason::Exit(42)));
    }

    #[test]
    fn test_syscall_read() {
        let mut state = VMState::new(0);
        let mut memory = Memory::new();
        let mut io = IOHandler::new(vec![123, 456]);

        // Read syscall
        state.write_reg(Register::R10, SYSCALL_READ);
        handle_syscall(&mut state, &mut memory, &mut io).unwrap();

        assert_eq!(state.read_reg(Register::R10), 123);

        // Read again
        state.write_reg(Register::R10, SYSCALL_READ);
        handle_syscall(&mut state, &mut memory, &mut io).unwrap();

        assert_eq!(state.read_reg(Register::R10), 456);
    }

    #[test]
    fn test_syscall_write() {
        let mut state = VMState::new(0);
        let mut memory = Memory::new();
        let mut io = IOHandler::new(vec![]);

        // Write syscall
        state.write_reg(Register::R10, SYSCALL_WRITE);
        state.write_reg(Register::R11, 999);
        handle_syscall(&mut state, &mut memory, &mut io).unwrap();

        assert_eq!(io.outputs(), &[999]);

        // Write again
        state.write_reg(Register::R10, SYSCALL_WRITE);
        state.write_reg(Register::R11, 888);
        handle_syscall(&mut state, &mut memory, &mut io).unwrap();

        assert_eq!(io.outputs(), &[999, 888]);
    }

    #[test]
    fn test_invalid_syscall() {
        let mut state = VMState::new(0);
        let mut memory = Memory::new();
        let mut io = IOHandler::new(vec![]);

        // Invalid syscall number
        state.write_reg(Register::R10, 999);

        let result = handle_syscall(&mut state, &mut memory, &mut io);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidSyscall { syscall: 999 }
        ));
    }

    #[test]
    fn test_syscall_sha256() {
        let mut state = VMState::new(0);
        let mut memory = Memory::new();
        let mut io = IOHandler::new(vec![]);

        // Write "hello" to memory at 0x1000
        let input_ptr = 0x1000;
        memory.write_u8(input_ptr, b'h').unwrap();
        memory.write_u8(input_ptr + 1, b'e').unwrap();
        memory.write_u8(input_ptr + 2, b'l').unwrap();
        memory.write_u8(input_ptr + 3, b'l').unwrap();
        memory.write_u8(input_ptr + 4, b'o').unwrap();

        // Set up SHA-256 syscall
        let output_ptr = 0x2000;
        state.write_reg(Register::R10, SYSCALL_SHA256);
        state.write_reg(Register::R11, input_ptr);
        state.write_reg(Register::R12, 5); // "hello" length
        state.write_reg(Register::R13, output_ptr);

        handle_syscall(&mut state, &mut memory, &mut io).unwrap();

        // Check return value (should be 0 for success)
        assert_eq!(state.read_reg(Register::R10), 0);

        // Verify output bound was set (32 bits for SHA-256)
        let bound = state.read_bound(Register::R14);
        assert_eq!(bound.max_bits, 32);

        // Verify hash output (SHA-256("hello"))
        let expected = [
            0x2cf24dba, 0x5fb0a30e, 0x26e83b2a, 0xc5b9e29e, 0x1b161e5c, 0x1fa7425e, 0x73043362,
            0x938b9824,
        ];
        for (i, &exp) in expected.iter().enumerate() {
            let word = memory.read_u32(output_ptr + (i * 4) as u64).unwrap();
            assert_eq!(word, exp);
        }
    }
}
