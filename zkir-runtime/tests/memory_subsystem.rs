//! Integration tests for the memory subsystem
//!
//! Tests memory access, protection, and trace collection.

use zkir_runtime::{Memory, MemoryRegion};
use zkir_spec::memory::*;

#[test]
fn test_memory_regions() {
    let mem = Memory::new();

    // Reserved region (0x0000 - 0x0FFF)
    assert_eq!(mem.get_region(0x0), MemoryRegion::Reserved);
    assert_eq!(mem.get_region(0x500), MemoryRegion::Reserved);
    assert_eq!(mem.get_region(RESERVED_SIZE - 1), MemoryRegion::Reserved);

    // Code region (0x1000 - 0x0FFF_FFFF)
    assert_eq!(mem.get_region(CODE_BASE), MemoryRegion::Code);
    assert_eq!(mem.get_region(CODE_BASE + 0x1000), MemoryRegion::Code);
    assert_eq!(mem.get_region(DATA_BASE - 1), MemoryRegion::Code);

    // Data region (0x1000_0000 - 0x1FFF_FFFF)
    assert_eq!(mem.get_region(DATA_BASE), MemoryRegion::Data);
    assert_eq!(mem.get_region(DATA_BASE + 0x100000), MemoryRegion::Data);
    assert_eq!(mem.get_region(HEAP_BASE - 1), MemoryRegion::Data);

    // Heap region (0x2000_0000 - heap_break)
    assert_eq!(mem.get_region(HEAP_BASE), MemoryRegion::Heap);

    // Stack region (near top of address space)
    assert_eq!(mem.get_region(STACK_TOP - 100), MemoryRegion::Stack);
    assert_eq!(mem.get_region(STACK_TOP - 0x10000), MemoryRegion::Stack);
}

#[test]
fn test_region_writability() {
    assert!(!MemoryRegion::Reserved.is_writable());
    assert!(!MemoryRegion::Code.is_writable());
    assert!(MemoryRegion::Data.is_writable());
    assert!(MemoryRegion::Heap.is_writable());
    assert!(MemoryRegion::Stack.is_writable());
}

#[test]
fn test_region_readability() {
    // All regions should be readable
    assert!(MemoryRegion::Reserved.is_readable());
    assert!(MemoryRegion::Code.is_readable());
    assert!(MemoryRegion::Data.is_readable());
    assert!(MemoryRegion::Heap.is_readable());
    assert!(MemoryRegion::Stack.is_readable());
}

#[test]
fn test_byte_read_write() {
    let mut mem = Memory::new();

    // Write and read in data region
    mem.write_u8(DATA_BASE, 0x42).unwrap();
    assert_eq!(mem.read_u8(DATA_BASE).unwrap(), 0x42);

    mem.write_u8(DATA_BASE + 1, 0xFF).unwrap();
    assert_eq!(mem.read_u8(DATA_BASE + 1).unwrap(), 0xFF);

    // Multiple bytes
    for i in 0..256 {
        mem.write_u8(DATA_BASE + 100 + i, i as u8).unwrap();
    }
    for i in 0..256 {
        assert_eq!(mem.read_u8(DATA_BASE + 100 + i).unwrap(), i as u8);
    }
}

#[test]
fn test_halfword_read_write() {
    let mut mem = Memory::new();

    mem.write_u16(DATA_BASE, 0x1234).unwrap();
    assert_eq!(mem.read_u16(DATA_BASE).unwrap(), 0x1234);

    mem.write_u16(DATA_BASE + 2, 0xABCD).unwrap();
    assert_eq!(mem.read_u16(DATA_BASE + 2).unwrap(), 0xABCD);
}

#[test]
fn test_word_read_write() {
    let mut mem = Memory::new();

    mem.write_u32(DATA_BASE, 0xDEADBEEF).unwrap();
    assert_eq!(mem.read_u32(DATA_BASE).unwrap(), 0xDEADBEEF);

    mem.write_u32(DATA_BASE + 4, 0xCAFEBABE).unwrap();
    assert_eq!(mem.read_u32(DATA_BASE + 4).unwrap(), 0xCAFEBABE);
}

#[test]
fn test_doubleword_read_write() {
    let mut mem = Memory::new();

    mem.write_u64(DATA_BASE, 0xDEADBEEFCAFEBABE).unwrap();
    assert_eq!(mem.read_u64(DATA_BASE).unwrap(), 0xDEADBEEFCAFEBABE);
}

#[test]
fn test_little_endian() {
    let mut mem = Memory::new();

    // Write a word
    mem.write_u32(DATA_BASE, 0x04030201).unwrap();

    // Read individual bytes - should be little-endian
    assert_eq!(mem.read_u8(DATA_BASE).unwrap(), 0x01);
    assert_eq!(mem.read_u8(DATA_BASE + 1).unwrap(), 0x02);
    assert_eq!(mem.read_u8(DATA_BASE + 2).unwrap(), 0x03);
    assert_eq!(mem.read_u8(DATA_BASE + 3).unwrap(), 0x04);
}

#[test]
fn test_alignment_requirements() {
    let mut mem = Memory::new();

    // Halfword must be 2-byte aligned
    assert!(mem.write_u16(DATA_BASE + 1, 0x1234).is_err());
    assert!(mem.read_u16(DATA_BASE + 1).is_err());

    // Word must be 4-byte aligned
    assert!(mem.write_u32(DATA_BASE + 1, 0x12345678).is_err());
    assert!(mem.write_u32(DATA_BASE + 2, 0x12345678).is_err());
    assert!(mem.write_u32(DATA_BASE + 3, 0x12345678).is_err());
    assert!(mem.read_u32(DATA_BASE + 1).is_err());

    // Doubleword must be 8-byte aligned
    assert!(mem.write_u64(DATA_BASE + 4, 0x123456789ABCDEF0).is_err());
    assert!(mem.read_u64(DATA_BASE + 4).is_err());
}

#[test]
fn test_uninitialized_reads_zero() {
    let mut mem = Memory::new();

    // Reading uninitialized memory should return 0
    assert_eq!(mem.read_u8(DATA_BASE + 0x10000).unwrap(), 0);
    assert_eq!(mem.read_u16(DATA_BASE + 0x10000).unwrap(), 0);
    assert_eq!(mem.read_u32(DATA_BASE + 0x10000).unwrap(), 0);
    assert_eq!(mem.read_u64(DATA_BASE + 0x10000).unwrap(), 0);
}

#[test]
fn test_sparse_memory() {
    let mut mem = Memory::new();

    // Write to widely separated addresses
    mem.write_u32(DATA_BASE, 0xAAAA).unwrap();
    mem.write_u32(DATA_BASE + 0x100000, 0xBBBB).unwrap();
    mem.write_u32(HEAP_BASE + 0x500000, 0xCCCC).unwrap();

    // All should be readable
    assert_eq!(mem.read_u32(DATA_BASE).unwrap(), 0xAAAA);
    assert_eq!(mem.read_u32(DATA_BASE + 0x100000).unwrap(), 0xBBBB);
    assert_eq!(mem.read_u32(HEAP_BASE + 0x500000).unwrap(), 0xCCCC);

    // Memory between them should be zero
    assert_eq!(mem.read_u32(DATA_BASE + 0x50000).unwrap(), 0);
}

#[test]
fn test_load_code() {
    let mut mem = Memory::new();

    let code = vec![0x12345678u32, 0xABCDEF00, 0x11223344];
    mem.load_code(&code, CODE_BASE).unwrap();

    // Read back the code
    assert_eq!(mem.read_u32(CODE_BASE).unwrap(), 0x12345678);
    assert_eq!(mem.read_u32(CODE_BASE + 4).unwrap(), 0xABCDEF00);
    assert_eq!(mem.read_u32(CODE_BASE + 8).unwrap(), 0x11223344);
}

#[test]
fn test_code_protection_after_load() {
    let mut mem = Memory::new();

    let code = vec![0x12345678u32];
    mem.load_code(&code, CODE_BASE).unwrap();

    // Writing to code section should fail
    let result = mem.write_u32(CODE_BASE, 0xDEADBEEF);
    assert!(result.is_err());

    // Reading should still work
    assert_eq!(mem.read_u32(CODE_BASE).unwrap(), 0x12345678);
}

#[test]
fn test_reserved_region_protection() {
    let mut mem = Memory::new();

    // Writing to reserved region should fail
    assert!(mem.write_u8(0x100, 0xFF).is_err());
    assert!(mem.write_u16(0x100, 0xFFFF).is_err());
    assert!(mem.write_u32(0x100, 0xFFFFFFFF).is_err());
}

#[test]
fn test_disable_protection() {
    let mut mem = Memory::new();

    mem.set_strict_protection(false);

    // Now writing to reserved should work
    mem.write_u8(0x100, 0xFF).unwrap();
    assert_eq!(mem.read_u8(0x100).unwrap(), 0xFF);

    // Re-enable protection
    mem.set_strict_protection(true);
    assert!(mem.write_u8(0x200, 0xFF).is_err());
}

#[test]
fn test_trace_collection() {
    let mut mem = Memory::with_trace();

    mem.write_u32(DATA_BASE, 0x12345678).unwrap();
    let _ = mem.read_u32(DATA_BASE).unwrap();

    let trace = mem.get_trace();

    // Should have one write and one read
    assert_eq!(trace.len(), 2);

    // First operation is write
    assert!(trace[0].is_write());
    assert_eq!(trace[0].address, DATA_BASE);
    assert_eq!(trace[0].value, 0x12345678);

    // Second operation is read
    assert!(trace[1].is_read());
    assert_eq!(trace[1].address, DATA_BASE);
    assert_eq!(trace[1].value, 0x12345678);
}

#[test]
fn test_trace_bounds() {
    let mut mem = Memory::with_trace();

    // Byte: 8 bits
    mem.write_u8(DATA_BASE, 0xFF).unwrap();
    // Halfword: 16 bits
    mem.write_u16(DATA_BASE + 2, 0xFFFF).unwrap();
    // Word: 32 bits
    mem.write_u32(DATA_BASE + 4, 0xFFFFFFFF).unwrap();

    let trace = mem.get_trace();

    assert_eq!(trace[0].bound.max_bits, 8);
    assert_eq!(trace[1].bound.max_bits, 16);
    assert_eq!(trace[2].bound.max_bits, 32);
}

#[test]
fn test_trace_timestamps() {
    let mut mem = Memory::with_trace();

    mem.set_timestamp(100);
    mem.write_u32(DATA_BASE, 0xAAAA).unwrap();

    mem.set_timestamp(200);
    let _ = mem.read_u32(DATA_BASE).unwrap();

    let trace = mem.get_trace();

    assert_eq!(trace[0].timestamp, 100);
    assert_eq!(trace[1].timestamp, 200);
}

#[test]
fn test_trace_disable() {
    let mut mem = Memory::with_trace();

    mem.write_u32(DATA_BASE, 0x1111).unwrap();

    mem.set_trace_enabled(false);
    mem.write_u32(DATA_BASE + 4, 0x2222).unwrap();

    mem.set_trace_enabled(true);
    mem.write_u32(DATA_BASE + 8, 0x3333).unwrap();

    let trace = mem.get_trace();

    // set_trace_enabled(false) clears the trace, so we only have the third write
    assert_eq!(trace.len(), 1);
    assert_eq!(trace[0].value, 0x3333);
}

#[test]
fn test_stack_operations() {
    let mut mem = Memory::new();

    // STACK_TOP is 0xFF_FFFF_FFFF which ends in 0xFF, not aligned to 8
    // We need to align the stack pointer to 8 bytes first
    let sp = (STACK_TOP - 8) & !7; // Align to 8 bytes

    mem.write_u64(sp, 0xDEADBEEFCAFEBABE).unwrap();
    assert_eq!(mem.read_u64(sp).unwrap(), 0xDEADBEEFCAFEBABE);

    // Multiple pushes
    for i in 0..10u64 {
        let addr = sp - i * 8;
        mem.write_u64(addr, i).unwrap();
    }

    for i in 0..10u64 {
        let addr = sp - i * 8;
        assert_eq!(mem.read_u64(addr).unwrap(), i);
    }
}

#[test]
fn test_heap_break() {
    let mem = Memory::new();

    // Initial heap break is at HEAP_BASE
    let initial_break = mem.heap_break();
    assert_eq!(initial_break, HEAP_BASE);
}

#[test]
fn test_cross_page_access() {
    let mut mem = Memory::new();

    // Write data that spans a page boundary (4KB pages)
    // Use an address that is 4-byte aligned but spans pages
    let page_boundary = DATA_BASE + 0x1000 - 4; // 4 bytes before page boundary, 4-byte aligned

    mem.write_u32(page_boundary, 0x12345678).unwrap();
    assert_eq!(mem.read_u32(page_boundary).unwrap(), 0x12345678);

    // Write at page boundary itself
    let at_boundary = DATA_BASE + 0x1000;
    mem.write_u32(at_boundary, 0xAABBCCDD).unwrap();
    assert_eq!(mem.read_u32(at_boundary).unwrap(), 0xAABBCCDD);

    // Verify bytes are in correct positions (little-endian)
    assert_eq!(mem.read_u8(page_boundary).unwrap(), 0x78);
    assert_eq!(mem.read_u8(page_boundary + 1).unwrap(), 0x56);
    assert_eq!(mem.read_u8(page_boundary + 2).unwrap(), 0x34);
    assert_eq!(mem.read_u8(page_boundary + 3).unwrap(), 0x12);
}

#[test]
fn test_sorted_trace() {
    let mut mem = Memory::with_trace();

    // Write to different addresses at different timestamps
    // Use 4-byte aligned addresses (DATA_BASE is 0x1000_0000, already aligned)
    mem.set_timestamp(3);
    mem.write_u32(DATA_BASE + 100, 0xAAAA).unwrap();
    mem.set_timestamp(1);
    mem.write_u32(DATA_BASE, 0xBBBB).unwrap();
    mem.set_timestamp(2);
    mem.write_u32(DATA_BASE + 48, 0xCCCC).unwrap(); // Use 48 (4-byte aligned) instead of 50

    // Get sorted trace - sorted by timestamp
    let sorted = mem.get_sorted_trace();

    // Should be sorted by timestamp
    assert_eq!(sorted[0].timestamp, 1);
    assert_eq!(sorted[1].timestamp, 2);
    assert_eq!(sorted[2].timestamp, 3);
}

#[test]
fn test_clear_trace() {
    let mut mem = Memory::with_trace();

    mem.write_u32(DATA_BASE, 0xAAAA).unwrap();
    assert!(!mem.get_trace().is_empty());

    mem.clear_trace();
    assert!(mem.get_trace().is_empty());
}
