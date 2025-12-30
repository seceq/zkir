//! Memory subsystem for ZKIR v3.4
//!
//! Memory is byte-addressed with configurable address width (default 40-bit).
//! Uses sparse page-based storage for efficiency.
//!
//! Load/Store operations:
//! - LB/LBU: Load byte (sign/zero extend)
//! - LH/LHU: Load halfword (2 bytes, sign/zero extend)
//! - LW: Load word (4 bytes)
//! - LD: Load doubleword (8 bytes)
//! - SB: Store byte
//! - SH: Store halfword
//! - SW: Store word
//! - SD: Store doubleword
//!
//! ## Memory Regions
//! - Reserved: 0x0000_0000 - 0x0000_0FFF (4 KB)
//! - Code:     0x0000_1000 - 0x0FFF_FFFF (256 MB)
//! - Data:     0x1000_0000 - 0x1FFF_FFFF (256 MB)
//! - Heap:     0x2000_0000 - onwards
//! - Stack:    down from 0xFF_FFFF_FFFF

use crate::error::{RuntimeError, Result};
use zkir_spec::memory::*;
use zkir_spec::{MemoryOp, ValueBound};
use std::collections::HashMap;

/// Memory region for access validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegion {
    Reserved,
    Code,
    Data,
    Heap,
    Stack,
}

const PAGE_SIZE: usize = 4096;  // 4KB pages

impl MemoryRegion {
    /// Determine the region for a given address
    ///
    /// Memory layout (40-bit address space):
    /// - Reserved: 0x0000_0000 - 0x0000_0FFF (4 KB)
    /// - Code:     0x0000_1000 - 0x0FFF_FFFF (256 MB - 4KB)
    /// - Data:     0x1000_0000 - 0x1FFF_FFFF (256 MB)
    /// - Heap:     0x2000_0000 - heap_break
    /// - Stack:    stack_top - 1 MB to stack_top
    pub fn from_address(addr: u64, heap_break: u64, stack_top: u64) -> Self {
        if addr < RESERVED_SIZE {
            MemoryRegion::Reserved
        } else if addr >= CODE_BASE && addr < DATA_BASE {
            MemoryRegion::Code
        } else if addr >= DATA_BASE && addr < HEAP_BASE {
            MemoryRegion::Data
        } else if addr >= HEAP_BASE && addr < heap_break {
            MemoryRegion::Heap
        } else if addr > stack_top - (DEFAULT_STACK_SIZE as u64) {
            MemoryRegion::Stack
        } else {
            // Unmapped regions - treat as heap for flexibility
            MemoryRegion::Heap
        }
    }

    /// Check if writes are allowed to this region
    pub fn is_writable(&self) -> bool {
        match self {
            MemoryRegion::Reserved => false,
            MemoryRegion::Code => false, // Code is read-only after loading
            MemoryRegion::Data => true,
            MemoryRegion::Heap => true,
            MemoryRegion::Stack => true,
        }
    }

    /// Check if reads are allowed from this region
    pub fn is_readable(&self) -> bool {
        // All regions are readable
        true
    }
}

/// Sparse page-based memory
#[derive(Debug, Clone)]
pub struct Memory {
    /// Memory pages (4KB each)
    pages: HashMap<u64, Vec<u8>>,

    /// Stack top address
    stack_top: u64,

    /// Heap break address
    heap_break: u64,

    /// Memory operation trace (for proof generation)
    trace: Vec<MemoryOp>,

    /// Enable trace collection
    trace_enabled: bool,

    /// Current timestamp (cycle counter for trace)
    timestamp: u64,

    /// Enable strict memory protection (default: true for execution, false for loading)
    strict_protection: bool,

    /// Code has been loaded (enables code section protection)
    code_loaded: bool,
}

impl Memory {
    /// Create new memory subsystem
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
            stack_top: STACK_TOP,
            heap_break: HEAP_BASE,
            trace: Vec::new(),
            trace_enabled: false,
            timestamp: 0,
            strict_protection: true,
            code_loaded: false,
        }
    }

    /// Create new memory subsystem with trace collection enabled
    pub fn with_trace() -> Self {
        Self {
            pages: HashMap::new(),
            stack_top: STACK_TOP,
            heap_break: HEAP_BASE,
            trace: Vec::new(),
            trace_enabled: true,
            timestamp: 0,
            strict_protection: true,
            code_loaded: false,
        }
    }

    /// Get the memory region for an address
    pub fn get_region(&self, addr: u64) -> MemoryRegion {
        MemoryRegion::from_address(addr, self.heap_break, self.stack_top)
    }

    /// Validate a write access
    fn validate_write(&self, addr: u64, size: usize) -> Result<()> {
        if !self.strict_protection {
            return Ok(());
        }

        let region = self.get_region(addr);

        // Check reserved region
        if region == MemoryRegion::Reserved {
            return Err(RuntimeError::InvalidMemoryAccess {
                address: addr,
                reason: "write to reserved memory region".to_string(),
            });
        }

        // Check code section protection (only after code is loaded)
        if self.code_loaded && region == MemoryRegion::Code {
            return Err(RuntimeError::InvalidMemoryAccess {
                address: addr,
                reason: "write to read-only code section".to_string(),
            });
        }

        // Check if access spans multiple regions (potential overflow)
        let end_addr = addr.saturating_add(size as u64 - 1);
        let end_region = self.get_region(end_addr);
        if region != end_region {
            // This is fine for data/heap/stack boundaries, but not for code
            if region == MemoryRegion::Code || end_region == MemoryRegion::Code {
                return Err(RuntimeError::InvalidMemoryAccess {
                    address: addr,
                    reason: "write spans code section boundary".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Enable or disable strict memory protection
    pub fn set_strict_protection(&mut self, enabled: bool) {
        self.strict_protection = enabled;
    }

    /// Check if strict protection is enabled
    pub fn is_strict_protection(&self) -> bool {
        self.strict_protection
    }

    /// Enable or disable trace collection
    pub fn set_trace_enabled(&mut self, enabled: bool) {
        self.trace_enabled = enabled;
        if !enabled {
            self.trace.clear();
        }
    }

    /// Check if trace collection is enabled
    pub fn is_trace_enabled(&self) -> bool {
        self.trace_enabled
    }

    /// Increment timestamp (should be called at each cycle)
    pub fn tick(&mut self) {
        self.timestamp += 1;
    }

    /// Set timestamp directly
    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = timestamp;
    }

    /// Get current timestamp
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Get the memory operation trace
    pub fn get_trace(&self) -> &[MemoryOp] {
        &self.trace
    }

    /// Get a sorted copy of the memory trace (sorted by timestamp, address, op_type)
    pub fn get_sorted_trace(&self) -> Vec<MemoryOp> {
        let mut sorted = self.trace.clone();
        sorted.sort();
        sorted
    }

    /// Clear the trace
    pub fn clear_trace(&mut self) {
        self.trace.clear();
        self.timestamp = 0;
    }

    /// Record a memory operation in the trace
    fn record_op(&mut self, address: u64, value: u64, is_write: bool, width: u8) {
        if self.trace_enabled {
            let bound = ValueBound::from_type_width(width as u32 * 8);
            let op = if is_write {
                MemoryOp::write(address, value, self.timestamp, bound, width)
            } else {
                MemoryOp::read(address, value, self.timestamp, bound, width)
            };
            self.trace.push(op);
        }
    }

    /// Load program code into memory
    ///
    /// This temporarily disables memory protection to allow writing to the code section.
    /// After loading, the code section becomes read-only.
    pub fn load_code(&mut self, code: &[u32], base: u64) -> Result<()> {
        // Temporarily disable protection to load code
        let was_protected = self.strict_protection;
        self.strict_protection = false;

        for (i, &word) in code.iter().enumerate() {
            let addr = base + (i * 4) as u64;
            self.write_u32(addr, word)?;
        }

        // Restore protection and mark code as loaded
        self.strict_protection = was_protected;
        self.code_loaded = true;

        Ok(())
    }

    /// Load program data section into memory
    ///
    /// Data section is loaded right after the code section.
    /// The base address should be CODE_BASE + code_size.
    pub fn load_data(&mut self, data: &[u8], base: u64) -> Result<()> {
        // Temporarily disable protection to load data
        let was_protected = self.strict_protection;
        self.strict_protection = false;

        for (i, &byte) in data.iter().enumerate() {
            let addr = base + i as u64;
            self.write_u8(addr, byte)?;
        }

        // Restore protection
        self.strict_protection = was_protected;

        Ok(())
    }

    /// Read single byte
    pub fn read_u8(&mut self, addr: u64) -> Result<u8> {
        let page_num = addr / PAGE_SIZE as u64;
        let offset = (addr % PAGE_SIZE as u64) as usize;

        let value = if let Some(page) = self.pages.get(&page_num) {
            page[offset]
        } else {
            0  // Uninitialized memory reads as zero
        };

        self.record_op(addr, value as u64, false, 1);
        Ok(value)
    }

    /// Write single byte
    pub fn write_u8(&mut self, addr: u64, value: u8) -> Result<()> {
        // Validate write access
        self.validate_write(addr, 1)?;

        let page_num = addr / PAGE_SIZE as u64;
        let offset = (addr % PAGE_SIZE as u64) as usize;

        let page = self.pages.entry(page_num)
            .or_insert_with(|| vec![0; PAGE_SIZE]);
        page[offset] = value;

        self.record_op(addr, value as u64, true, 1);
        Ok(())
    }

    /// Read halfword (2 bytes, little-endian)
    pub fn read_u16(&mut self, addr: u64) -> Result<u16> {
        if addr % 2 != 0 {
            return Err(RuntimeError::MisalignedAccess {
                address: addr,
                alignment: 2,
            });
        }

        // Temporarily disable tracing to avoid recording individual byte reads
        let trace_was_enabled = self.trace_enabled;
        self.trace_enabled = false;

        let b0 = self.read_u8(addr)? as u16;
        let b1 = self.read_u8(addr + 1)? as u16;
        let value = b0 | (b1 << 8);

        // Re-enable tracing and record the halfword read
        self.trace_enabled = trace_was_enabled;
        self.record_op(addr, value as u64, false, 2);

        Ok(value)
    }

    /// Write halfword (2 bytes, little-endian)
    pub fn write_u16(&mut self, addr: u64, value: u16) -> Result<()> {
        if addr % 2 != 0 {
            return Err(RuntimeError::MisalignedAccess {
                address: addr,
                alignment: 2,
            });
        }

        // Validate write access upfront to avoid partial writes
        self.validate_write(addr, 2)?;

        // Temporarily disable tracing to avoid recording individual byte writes
        let trace_was_enabled = self.trace_enabled;
        // Also disable protection since we've already validated
        let prot_was_enabled = self.strict_protection;
        self.strict_protection = false;
        self.trace_enabled = false;

        self.write_u8(addr, (value & 0xFF) as u8)?;
        self.write_u8(addr + 1, ((value >> 8) & 0xFF) as u8)?;

        // Re-enable tracing and protection, then record the halfword write
        self.trace_enabled = trace_was_enabled;
        self.strict_protection = prot_was_enabled;
        self.record_op(addr, value as u64, true, 2);

        Ok(())
    }

    /// Read word (4 bytes, little-endian)
    pub fn read_u32(&mut self, addr: u64) -> Result<u32> {
        if addr % 4 != 0 {
            return Err(RuntimeError::MisalignedAccess {
                address: addr,
                alignment: 4,
            });
        }

        // Temporarily disable tracing to avoid recording individual byte reads
        let trace_was_enabled = self.trace_enabled;
        self.trace_enabled = false;

        let b0 = self.read_u8(addr)? as u32;
        let b1 = self.read_u8(addr + 1)? as u32;
        let b2 = self.read_u8(addr + 2)? as u32;
        let b3 = self.read_u8(addr + 3)? as u32;
        let value = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);

        // Re-enable tracing and record the word read
        self.trace_enabled = trace_was_enabled;
        self.record_op(addr, value as u64, false, 4);

        Ok(value)
    }

    /// Write word (4 bytes, little-endian)
    pub fn write_u32(&mut self, addr: u64, value: u32) -> Result<()> {
        if addr % 4 != 0 {
            return Err(RuntimeError::MisalignedAccess {
                address: addr,
                alignment: 4,
            });
        }

        // Validate write access upfront to avoid partial writes
        self.validate_write(addr, 4)?;

        // Temporarily disable tracing and protection
        let trace_was_enabled = self.trace_enabled;
        let prot_was_enabled = self.strict_protection;
        self.trace_enabled = false;
        self.strict_protection = false;

        self.write_u8(addr, (value & 0xFF) as u8)?;
        self.write_u8(addr + 1, ((value >> 8) & 0xFF) as u8)?;
        self.write_u8(addr + 2, ((value >> 16) & 0xFF) as u8)?;
        self.write_u8(addr + 3, ((value >> 24) & 0xFF) as u8)?;

        // Re-enable tracing and protection, then record the word write
        self.trace_enabled = trace_was_enabled;
        self.strict_protection = prot_was_enabled;
        self.record_op(addr, value as u64, true, 4);

        Ok(())
    }

    /// Read doubleword (8 bytes, little-endian)
    pub fn read_u64(&mut self, addr: u64) -> Result<u64> {
        if addr % 8 != 0 {
            return Err(RuntimeError::MisalignedAccess {
                address: addr,
                alignment: 8,
            });
        }

        // Temporarily disable tracing to avoid recording individual word reads
        let trace_was_enabled = self.trace_enabled;
        self.trace_enabled = false;

        let low = self.read_u32(addr)? as u64;
        let high = self.read_u32(addr + 4)? as u64;
        let value = low | (high << 32);

        // Re-enable tracing and record the doubleword read
        self.trace_enabled = trace_was_enabled;
        self.record_op(addr, value, false, 8);

        Ok(value)
    }

    /// Write doubleword (8 bytes, little-endian)
    pub fn write_u64(&mut self, addr: u64, value: u64) -> Result<()> {
        if addr % 8 != 0 {
            return Err(RuntimeError::MisalignedAccess {
                address: addr,
                alignment: 8,
            });
        }

        // Validate write access upfront to avoid partial writes
        self.validate_write(addr, 8)?;

        // Temporarily disable tracing and protection
        let trace_was_enabled = self.trace_enabled;
        let prot_was_enabled = self.strict_protection;
        self.trace_enabled = false;
        self.strict_protection = false;

        self.write_u32(addr, (value & 0xFFFFFFFF) as u32)?;
        self.write_u32(addr + 4, ((value >> 32) & 0xFFFFFFFF) as u32)?;

        // Re-enable tracing and protection, then record the doubleword write
        self.trace_enabled = trace_was_enabled;
        self.strict_protection = prot_was_enabled;
        self.record_op(addr, value, true, 8);

        Ok(())
    }

    /// Get stack top address
    pub fn stack_top(&self) -> u64 {
        self.stack_top
    }

    /// Get heap break address
    pub fn heap_break(&self) -> u64 {
        self.heap_break
    }

    /// Set heap break (for brk syscall)
    pub fn set_heap_break(&mut self, addr: u64) {
        self.heap_break = addr;
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_access() {
        let mut mem = Memory::new();

        // Use DATA_BASE since CODE_BASE (0x1000) is protected after load
        mem.write_u8(DATA_BASE, 0x42).unwrap();
        assert_eq!(mem.read_u8(DATA_BASE).unwrap(), 0x42);

        mem.write_u8(DATA_BASE + 1, 0xFF).unwrap();
        assert_eq!(mem.read_u8(DATA_BASE + 1).unwrap(), 0xFF);
    }

    #[test]
    fn test_halfword_access() {
        let mut mem = Memory::new();

        mem.write_u16(0x1000, 0x1234).unwrap();
        assert_eq!(mem.read_u16(0x1000).unwrap(), 0x1234);

        // Little-endian: 0x1234 = [0x34, 0x12]
        assert_eq!(mem.read_u8(0x1000).unwrap(), 0x34);
        assert_eq!(mem.read_u8(0x1001).unwrap(), 0x12);
    }

    #[test]
    fn test_word_access() {
        let mut mem = Memory::new();

        mem.write_u32(0x1000, 0x12345678).unwrap();
        assert_eq!(mem.read_u32(0x1000).unwrap(), 0x12345678);

        // Little-endian: 0x12345678 = [0x78, 0x56, 0x34, 0x12]
        assert_eq!(mem.read_u8(0x1000).unwrap(), 0x78);
        assert_eq!(mem.read_u8(0x1001).unwrap(), 0x56);
        assert_eq!(mem.read_u8(0x1002).unwrap(), 0x34);
        assert_eq!(mem.read_u8(0x1003).unwrap(), 0x12);
    }

    #[test]
    fn test_doubleword_access() {
        let mut mem = Memory::new();

        mem.write_u64(0x1000, 0x123456789ABCDEF0).unwrap();
        assert_eq!(mem.read_u64(0x1000).unwrap(), 0x123456789ABCDEF0);
    }

    #[test]
    fn test_misaligned_access() {
        let mut mem = Memory::new();

        // Misaligned halfword
        assert!(mem.write_u16(0x1001, 0).is_err());
        assert!(mem.read_u16(0x1001).is_err());

        // Misaligned word
        assert!(mem.write_u32(0x1001, 0).is_err());
        assert!(mem.read_u32(0x1002).is_err());

        // Misaligned doubleword
        assert!(mem.write_u64(0x1004, 0).is_err());
        assert!(mem.read_u64(0x1001).is_err());
    }

    #[test]
    fn test_uninitialized_reads_zero() {
        let mut mem = Memory::new();

        // Uninitialized memory reads as zero
        assert_eq!(mem.read_u8(0x1000).unwrap(), 0);
        assert_eq!(mem.read_u16(0x2000).unwrap(), 0);
        assert_eq!(mem.read_u32(0x3000).unwrap(), 0);
        assert_eq!(mem.read_u64(0x4000).unwrap(), 0);
    }

    #[test]
    fn test_load_code() {
        let mut mem = Memory::new();

        let code = vec![0x12345678, 0xABCDEF00];
        mem.load_code(&code, CODE_BASE).unwrap();

        assert_eq!(mem.read_u32(CODE_BASE).unwrap(), 0x12345678);
        assert_eq!(mem.read_u32(CODE_BASE + 4).unwrap(), 0xABCDEF00);
    }

    #[test]
    fn test_sparse_storage() {
        let mut mem = Memory::new();

        // Write to widely separated addresses
        mem.write_u32(0x1000, 0xAAAA).unwrap();
        mem.write_u32(0x100000, 0xBBBB).unwrap();
        mem.write_u32(0x10000000, 0xCCCC).unwrap();

        // All should be accessible
        assert_eq!(mem.read_u32(0x1000).unwrap(), 0xAAAA);
        assert_eq!(mem.read_u32(0x100000).unwrap(), 0xBBBB);
        assert_eq!(mem.read_u32(0x10000000).unwrap(), 0xCCCC);

        // Pages should only be allocated as needed
        assert!(mem.pages.len() <= 3);
    }

    #[test]
    fn test_trace_disabled_by_default() {
        let mut mem = Memory::new();

        // Trace should be disabled by default
        assert!(!mem.is_trace_enabled());

        // Perform operations
        mem.write_u32(0x1000, 0x1234).unwrap();
        mem.read_u32(0x1000).unwrap();

        // No trace should be collected
        assert_eq!(mem.get_trace().len(), 0);
    }

    #[test]
    fn test_trace_collection() {
        let mut mem = Memory::with_trace();

        // Trace should be enabled
        assert!(mem.is_trace_enabled());

        // Perform operations at different timestamps
        mem.set_timestamp(0);
        mem.write_u32(0x1000, 0x1234).unwrap();

        mem.set_timestamp(1);
        mem.read_u32(0x1000).unwrap();

        mem.set_timestamp(2);
        mem.write_u16(0x2000, 0xABCD).unwrap();

        // Should have 3 operations in trace
        let trace = mem.get_trace();
        assert_eq!(trace.len(), 3);

        // Check first operation (write)
        assert!(trace[0].is_write());
        assert_eq!(trace[0].address, 0x1000);
        assert_eq!(trace[0].value, 0x1234);
        assert_eq!(trace[0].timestamp, 0);
        assert_eq!(trace[0].width, 4);

        // Check second operation (read)
        assert!(trace[1].is_read());
        assert_eq!(trace[1].address, 0x1000);
        assert_eq!(trace[1].value, 0x1234);
        assert_eq!(trace[1].timestamp, 1);
        assert_eq!(trace[1].width, 4);

        // Check third operation (write halfword)
        assert!(trace[2].is_write());
        assert_eq!(trace[2].address, 0x2000);
        assert_eq!(trace[2].value, 0xABCD);
        assert_eq!(trace[2].timestamp, 2);
        assert_eq!(trace[2].width, 2);
    }

    #[test]
    fn test_trace_sorting() {
        let mut mem = Memory::with_trace();

        // Create operations out of timestamp order
        mem.set_timestamp(5);
        mem.write_u32(0x3000, 0x3333).unwrap();

        mem.set_timestamp(1);
        mem.write_u32(0x1000, 0x1111).unwrap();

        mem.set_timestamp(3);
        mem.read_u32(0x2000).unwrap();

        // Unsorted trace
        let trace = mem.get_trace();
        assert_eq!(trace.len(), 3);
        assert_eq!(trace[0].timestamp, 5);
        assert_eq!(trace[1].timestamp, 1);
        assert_eq!(trace[2].timestamp, 3);

        // Sorted trace
        let sorted = mem.get_sorted_trace();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].timestamp, 1);
        assert_eq!(sorted[1].timestamp, 3);
        assert_eq!(sorted[2].timestamp, 5);
    }

    #[test]
    fn test_trace_enable_disable() {
        let mut mem = Memory::new();

        // Enable tracing
        mem.set_trace_enabled(true);
        assert!(mem.is_trace_enabled());

        mem.write_u32(0x1000, 0x1234).unwrap();
        assert_eq!(mem.get_trace().len(), 1);

        // Disable tracing (should clear trace)
        mem.set_trace_enabled(false);
        assert!(!mem.is_trace_enabled());
        assert_eq!(mem.get_trace().len(), 0);

        // Operations should not be traced
        mem.write_u32(0x2000, 0x5678).unwrap();
        assert_eq!(mem.get_trace().len(), 0);

        // Re-enable tracing
        mem.set_trace_enabled(true);
        mem.write_u32(0x3000, 0xABCD).unwrap();
        assert_eq!(mem.get_trace().len(), 1);
    }

    #[test]
    fn test_trace_clear() {
        let mut mem = Memory::with_trace();

        // Add some operations
        mem.write_u32(0x1000, 0x1234).unwrap();
        mem.tick();
        mem.read_u32(0x1000).unwrap();
        assert_eq!(mem.get_trace().len(), 2);
        assert_eq!(mem.timestamp(), 1);

        // Clear trace
        mem.clear_trace();
        assert_eq!(mem.get_trace().len(), 0);
        assert_eq!(mem.timestamp(), 0);

        // Trace should still be enabled
        assert!(mem.is_trace_enabled());
        mem.write_u32(0x2000, 0x5678).unwrap();
        assert_eq!(mem.get_trace().len(), 1);
    }

    #[test]
    fn test_trace_different_widths() {
        let mut mem = Memory::with_trace();

        // Test all access widths
        mem.write_u8(0x1000, 0xFF).unwrap();
        mem.tick();
        mem.write_u16(0x2000, 0xFFFF).unwrap();
        mem.tick();
        mem.write_u32(0x3000, 0xFFFFFFFF).unwrap();
        mem.tick();
        mem.write_u64(0x4000, 0xFFFFFFFFFFFF).unwrap();

        let trace = mem.get_trace();
        assert_eq!(trace.len(), 4);

        assert_eq!(trace[0].width, 1);
        assert_eq!(trace[1].width, 2);
        assert_eq!(trace[2].width, 4);
        assert_eq!(trace[3].width, 8);
    }

    #[test]
    fn test_trace_bounds() {
        let mut mem = Memory::with_trace();

        // Write different widths and check bounds
        mem.write_u8(0x1000, 0xFF).unwrap();
        mem.write_u16(0x2000, 0xFFFF).unwrap();
        mem.write_u32(0x3000, 0xFFFFFFFF).unwrap();

        let trace = mem.get_trace();

        // Byte: 8 bits
        assert_eq!(trace[0].bound.max_bits, 8);

        // Halfword: 16 bits
        assert_eq!(trace[1].bound.max_bits, 16);

        // Word: 32 bits
        assert_eq!(trace[2].bound.max_bits, 32);
    }

    #[test]
    fn test_memory_region_detection() {
        let mem = Memory::new();

        // Reserved region
        assert_eq!(mem.get_region(0x0), MemoryRegion::Reserved);
        assert_eq!(mem.get_region(0x100), MemoryRegion::Reserved);

        // Code region
        assert_eq!(mem.get_region(CODE_BASE), MemoryRegion::Code);
        assert_eq!(mem.get_region(CODE_BASE + 0x1000), MemoryRegion::Code);

        // Data region
        assert_eq!(mem.get_region(DATA_BASE), MemoryRegion::Data);
        assert_eq!(mem.get_region(DATA_BASE + 0x1000), MemoryRegion::Data);

        // Heap region (default starts at HEAP_BASE)
        assert_eq!(mem.get_region(HEAP_BASE), MemoryRegion::Heap);

        // Stack region (near STACK_TOP)
        assert_eq!(mem.get_region(STACK_TOP - 0x100), MemoryRegion::Stack);
    }

    #[test]
    fn test_write_to_reserved_region() {
        let mut mem = Memory::new();

        // Writing to reserved region should fail
        let result = mem.write_u8(0x100, 0xFF);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidMemoryAccess { .. }
        ));
    }

    #[test]
    fn test_code_protection_after_load() {
        let mut mem = Memory::new();

        // Load some code
        let code = vec![0x12345678u32];
        mem.load_code(&code, CODE_BASE).unwrap();

        // After loading, code section should be protected
        assert!(mem.code_loaded);

        // Writing to code section should fail
        let result = mem.write_u32(CODE_BASE, 0xDEADBEEF);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidMemoryAccess { .. }
        ));

        // Reading from code section should still work
        assert_eq!(mem.read_u32(CODE_BASE).unwrap(), 0x12345678);
    }

    #[test]
    fn test_data_region_writable() {
        let mut mem = Memory::new();

        // Data region should be writable
        mem.write_u32(DATA_BASE, 0xAAAA).unwrap();
        assert_eq!(mem.read_u32(DATA_BASE).unwrap(), 0xAAAA);

        // Heap region should be writable
        mem.write_u32(HEAP_BASE + 0x100, 0xBBBB).unwrap();
        assert_eq!(mem.read_u32(HEAP_BASE + 0x100).unwrap(), 0xBBBB);
    }

    #[test]
    fn test_disable_strict_protection() {
        let mut mem = Memory::new();

        // Disable strict protection
        mem.set_strict_protection(false);
        assert!(!mem.is_strict_protection());

        // Now writing to reserved should work (no validation)
        mem.write_u8(0x100, 0xFF).unwrap();
        assert_eq!(mem.read_u8(0x100).unwrap(), 0xFF);
    }
}
