//! Syscall handlers for ZK IR v2.2

use zkir_spec::{Register, BABYBEAR_PRIME, MAX_30BIT};
use crate::state::{VMState, HaltReason};
use crate::io::IOHandler;
use crate::error::RuntimeError;

/// Syscall numbers (stored in a7)
pub mod syscall_nums {
    // Core syscalls
    pub const SYS_EXIT: u32 = 0x01;
    pub const SYS_READ: u32 = 0x10;
    pub const SYS_WRITE: u32 = 0x11;

    // Cryptographic syscalls
    pub const SYS_POSEIDON2: u32 = 0x12;
    pub const SYS_POSEIDON: u32 = 0x13;
    pub const SYS_SHA256_INIT: u32 = 0x20;
    pub const SYS_SHA256_UPDATE: u32 = 0x21;
    pub const SYS_SHA256_FINALIZE: u32 = 0x22;

    // Memory syscalls
    pub const SYS_MEMCPY: u32 = 0x30;
    pub const SYS_MEMSET: u32 = 0x31;
    pub const SYS_BRK: u32 = 0x32;
}

/// Handle ECALL syscall
pub fn handle_syscall(
    state: &mut VMState,
    io: &mut IOHandler,
) -> Result<(), RuntimeError> {
    let syscall_num = state.read_reg(Register::A7);

    match syscall_num {
        syscall_nums::SYS_EXIT => sys_exit(state)?,
        syscall_nums::SYS_READ => sys_read(state, io)?,
        syscall_nums::SYS_WRITE => sys_write(state, io)?,
        syscall_nums::SYS_POSEIDON2 => sys_poseidon2(state)?,
        syscall_nums::SYS_POSEIDON => sys_poseidon(state)?,
        syscall_nums::SYS_SHA256_INIT => sys_sha256_init(state)?,
        syscall_nums::SYS_SHA256_UPDATE => sys_sha256_update(state)?,
        syscall_nums::SYS_SHA256_FINALIZE => sys_sha256_finalize(state)?,
        syscall_nums::SYS_MEMCPY => sys_memcpy(state)?,
        syscall_nums::SYS_MEMSET => sys_memset(state)?,
        syscall_nums::SYS_BRK => sys_brk(state)?,
        _ => {
            return Err(RuntimeError::Halt(HaltReason::SyscallError {
                code: syscall_num,
                msg: format!("Unknown syscall: 0x{:02X}", syscall_num),
            }));
        }
    }

    Ok(())
}

// ========== Core Syscalls ==========

/// SYS_EXIT (0x01): Exit with code in a0
fn sys_exit(state: &mut VMState) -> Result<(), RuntimeError> {
    let exit_code = state.read_reg(Register::A0);

    if exit_code == 0 {
        // Normal exit
        state.halt(HaltReason::Halt);
    } else {
        // Exit with error code
        state.halt(HaltReason::SyscallError {
            code: exit_code,
            msg: format!("Program exited with code {}", exit_code),
        });
    }

    Ok(())
}

/// SYS_READ (0x10): Read public input into a0
fn sys_read(state: &mut VMState, io: &mut IOHandler) -> Result<(), RuntimeError> {
    match io.read() {
        Some(value) => {
            state.write_reg(Register::A0, value & MAX_30BIT);
            Ok(())
        }
        None => Err(RuntimeError::Halt(HaltReason::InputExhausted)),
    }
}

/// SYS_WRITE (0x11): Write a0 to public output
fn sys_write(state: &mut VMState, io: &mut IOHandler) -> Result<(), RuntimeError> {
    let value = state.read_reg(Register::A0);
    io.write(value);
    Ok(())
}

// ========== Cryptographic Syscalls ==========

/// SYS_POSEIDON2 (0x12): Poseidon2 permutation
/// Input: a0 = pointer to 12-word state array
/// Output: State modified in-place
fn sys_poseidon2(state: &mut VMState) -> Result<(), RuntimeError> {
    let ptr = state.read_reg(Register::A0);

    // Load 12-word state
    let mut poseidon_state = [0u32; 12];
    for i in 0..12 {
        let addr = (ptr + (i * 4) as u32) & MAX_30BIT;
        poseidon_state[i as usize] = state.memory.load_word(addr, state.cycle)?;
    }

    // Apply Poseidon2 permutation
    poseidon2_permutation(&mut poseidon_state);

    // Store result back
    for i in 0..12 {
        let addr = (ptr + (i * 4) as u32) & MAX_30BIT;
        state.memory.store_word(addr, poseidon_state[i as usize], state.cycle)?;
    }

    Ok(())
}

/// SYS_POSEIDON (0x13): Original Poseidon hash
/// Input: a0 = pointer to state array, a1 = number of elements
/// Output: State modified in-place
fn sys_poseidon(state: &mut VMState) -> Result<(), RuntimeError> {
    let ptr = state.read_reg(Register::A0);
    let count = state.read_reg(Register::A1).min(16); // Limit to reasonable size

    // Load state
    let mut poseidon_state = vec![0u32; count as usize];
    for i in 0..count {
        let addr = (ptr + (i * 4)) & MAX_30BIT;
        poseidon_state[i as usize] = state.memory.load_word(addr, state.cycle)?;
    }

    // Apply Poseidon permutation (simplified - real implementation would use proper constants)
    poseidon_permutation(&mut poseidon_state);

    // Store result back
    for i in 0..count {
        let addr = (ptr + (i * 4)) & MAX_30BIT;
        state.memory.store_word(addr, poseidon_state[i as usize], state.cycle)?;
    }

    Ok(())
}

/// SHA-256 context stored in memory
struct Sha256Context {
    state: [u32; 8],
    buffer: [u8; 64],
    buffer_len: usize,
    total_len: u64,
}

/// SYS_SHA256_INIT (0x20): Initialize SHA-256 context
/// Input: a0 = pointer to context (32 bytes minimum)
fn sys_sha256_init(state: &mut VMState) -> Result<(), RuntimeError> {
    let ptr = state.read_reg(Register::A0);

    // SHA-256 initial hash values
    let initial_state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    // Store initial state
    for i in 0..8 {
        let addr = (ptr + (i * 4)) & MAX_30BIT;
        state.memory.store_word(addr, initial_state[i as usize], state.cycle)?;
    }

    // Clear buffer length and total length (stored after state)
    let buffer_len_addr = (ptr + 32) & MAX_30BIT;
    let total_len_addr = (ptr + 36) & MAX_30BIT;
    state.memory.store_word(buffer_len_addr, 0, state.cycle)?;
    state.memory.store_word(total_len_addr, 0, state.cycle)?;

    Ok(())
}

/// SYS_SHA256_UPDATE (0x21): Update SHA-256 with data
/// Input: a0 = context pointer, a1 = data pointer, a2 = length
fn sys_sha256_update(state: &mut VMState) -> Result<(), RuntimeError> {
    let _ctx_ptr = state.read_reg(Register::A0);
    let _data_ptr = state.read_reg(Register::A1);
    let _len = state.read_reg(Register::A2);

    // TODO: Implement full SHA-256 update
    // For now, this is a placeholder
    tracing::warn!("SHA256_UPDATE not fully implemented");

    Ok(())
}

/// SYS_SHA256_FINALIZE (0x22): Finalize SHA-256 and get digest
/// Input: a0 = context pointer, a1 = output pointer (32 bytes)
fn sys_sha256_finalize(state: &mut VMState) -> Result<(), RuntimeError> {
    let ctx_ptr = state.read_reg(Register::A0);
    let out_ptr = state.read_reg(Register::A1);

    // Read current state
    let mut digest = [0u32; 8];
    for i in 0..8 {
        let addr = (ctx_ptr + (i * 4)) & MAX_30BIT;
        digest[i as usize] = state.memory.load_word(addr, state.cycle)?;
    }

    // TODO: Apply final padding and compression
    // For now, just copy the state as digest

    // Write digest to output
    for i in 0..8 {
        let addr = (out_ptr + (i * 4)) & MAX_30BIT;
        state.memory.store_word(addr, digest[i as usize], state.cycle)?;
    }

    Ok(())
}

// ========== Memory Syscalls ==========

/// SYS_MEMCPY (0x30): Copy memory region
/// Input: a0 = dest, a1 = src, a2 = length (in words)
fn sys_memcpy(state: &mut VMState) -> Result<(), RuntimeError> {
    let dest = state.read_reg(Register::A0);
    let src = state.read_reg(Register::A1);
    let len = state.read_reg(Register::A2).min(1024); // Limit to reasonable size

    for i in 0..len {
        let src_addr = (src + (i * 4)) & MAX_30BIT;
        let dest_addr = (dest + (i * 4)) & MAX_30BIT;
        let value = state.memory.load_word(src_addr, state.cycle)?;
        state.memory.store_word(dest_addr, value, state.cycle)?;
    }

    state.write_reg(Register::A0, dest);
    Ok(())
}

/// SYS_MEMSET (0x31): Set memory region
/// Input: a0 = dest, a1 = value, a2 = length (in words)
fn sys_memset(state: &mut VMState) -> Result<(), RuntimeError> {
    let dest = state.read_reg(Register::A0);
    let value = state.read_reg(Register::A1) & MAX_30BIT;
    let len = state.read_reg(Register::A2).min(1024); // Limit to reasonable size

    for i in 0..len {
        let addr = (dest + (i * 4)) & MAX_30BIT;
        state.memory.store_word(addr, value, state.cycle)?;
    }

    state.write_reg(Register::A0, dest);
    Ok(())
}

/// SYS_BRK (0x32): Adjust heap break
/// Input: a0 = new break address
/// Output: a0 = actual break address
fn sys_brk(_state: &mut VMState) -> Result<(), RuntimeError> {
    // TODO: Implement heap management
    // For now, this is a placeholder
    tracing::warn!("SYS_BRK not fully implemented");
    Ok(())
}

// ========== Poseidon2 Implementation ==========

/// Poseidon2 permutation for Baby Bear field
/// This is a simplified placeholder implementation
fn poseidon2_permutation(state: &mut [u32; 12]) {
    const ROUNDS_FULL: usize = 8;
    const ROUNDS_PARTIAL: usize = 13;

    // Full rounds (beginning)
    for _ in 0..ROUNDS_FULL / 2 {
        add_round_constants(state);
        apply_sbox_full(state);
        apply_mds(state);
    }

    // Partial rounds
    for _ in 0..ROUNDS_PARTIAL {
        add_round_constants(state);
        apply_sbox_partial(state);
        apply_mds(state);
    }

    // Full rounds (end)
    for _ in 0..ROUNDS_FULL / 2 {
        add_round_constants(state);
        apply_sbox_full(state);
        apply_mds(state);
    }
}

/// Add round constants (simplified - should use proper constants)
fn add_round_constants(state: &mut [u32; 12]) {
    // Placeholder: add dummy constants
    for (i, s) in state.iter_mut().enumerate() {
        *s = field_add(*s, (i as u32 * 123456789) % BABYBEAR_PRIME);
    }
}

/// Apply S-box to all elements
fn apply_sbox_full(state: &mut [u32; 12]) {
    for s in state.iter_mut() {
        *s = sbox(*s);
    }
}

/// Apply S-box to first element only
fn apply_sbox_partial(state: &mut [u32; 12]) {
    state[0] = sbox(state[0]);
}

/// S-box: x^7 mod p
fn sbox(x: u32) -> u32 {
    let x = x as u64;
    let p = BABYBEAR_PRIME as u64;

    let x2 = (x * x) % p;
    let x3 = (x2 * x) % p;
    let x4 = (x2 * x2) % p;
    let x7 = (x4 * x3) % p;

    x7 as u32
}

/// Apply MDS matrix (simplified - should use proper MDS matrix)
fn apply_mds(state: &mut [u32; 12]) {
    let mut new_state = [0u32; 12];

    // Simple mixing (not cryptographically secure, just for demonstration)
    for i in 0..12 {
        let mut sum = 0u64;
        for j in 0..12 {
            let coeff = ((i + j + 1) * 1000000007) % BABYBEAR_PRIME;
            sum += (state[j as usize] as u64 * coeff as u64) % BABYBEAR_PRIME as u64;
        }
        new_state[i as usize] = (sum % BABYBEAR_PRIME as u64) as u32;
    }

    state.copy_from_slice(&new_state);
}

/// Field addition modulo Baby Bear prime
fn field_add(a: u32, b: u32) -> u32 {
    ((a as u64 + b as u64) % BABYBEAR_PRIME as u64) as u32
}

// ========== Poseidon Implementation ==========

/// Original Poseidon permutation (simplified placeholder)
fn poseidon_permutation(state: &mut [u32]) {
    let rounds = 8;

    for _ in 0..rounds {
        // Add round constants
        for (i, s) in state.iter_mut().enumerate() {
            *s = field_add(*s, (i as u32 * 987654321) % BABYBEAR_PRIME);
        }

        // Apply S-box
        for s in state.iter_mut() {
            *s = sbox(*s);
        }

        // Simple mixing
        let sum: u64 = state.iter().map(|&x| x as u64).sum();
        for s in state.iter_mut() {
            *s = field_add(*s, (sum % BABYBEAR_PRIME as u64) as u32);
        }
    }
}
