# ZKIR Specification v3.4

**Architecture**: Variable Limb (16-30 bit limbs × 1-4 data limbs)
**Default**: 40-bit Split (20+20)
**Field**: Mersenne 31 (p = 2^31 - 1)

---

## 1. Overview

ZKIR v3.4 is a zero-knowledge virtual machine with configurable data width for optimal constraint efficiency.

| Feature | Description |
|---------|-------------|
| Variable Limbs | 16, 18, 20 (default), 22, 24, 26, 28, 30-bit |
| Data Limbs | 1, 2 (default), 3, 4 limbs per value |
| Address Limbs | 1, 2 (default) limbs per address |
| Default | 40-bit data (2 × 20-bit) |
| 16 Registers | r0-r15, 4-bit encoding |
| 32-bit Instructions | Fits in single word |
| Unified Chunks | chunk_bits = limb_bits / 2 |

---

## 2. Architecture

### 2.1 Variable Limb Configuration

```
┌─────────────────────────────────────────────────────────┐
│                    Configuration                         │
├─────────────────────────────────────────────────────────┤
│ Limb Bits: 16 │ 18 │ 20* │ 22 │ 24 │ 26 │ 28 │ 30      │
│ Data Limbs:      1 │ 2* │ 3 │ 4                         │
│ Addr Limbs:      1 │ 2*                                 │
│                                                         │
│ * = default                                             │
└─────────────────────────────────────────────────────────┘
```

### 2.2 Default Configuration (20-bit × 2)

```
Value (40 bits):
┌──────────────────┬──────────────────┐
│   limb1 (20b)    │   limb0 (20b)    │
│   bits 39-20     │   bits 19-0      │
└──────────────────┴──────────────────┘

Storage: 2 field elements per value
Range: [0, 2^40 - 1]
```

### 2.3 Type Mapping (Default 40-bit)

| Type | Bits | Fits in 40-bit? | Notes |
|------|------|-----------------|-------|
| i8/u8 | 8 | Yes | Zero-extended |
| i16/u16 | 16 | Yes | Zero-extended |
| i32/u32 | 32 | Yes | 8-bit headroom |
| i64/u64 | 64 | No | Use 3+ limbs |

### 2.4 Configuration Examples

| Config | Data Bits | Addr Bits | Table Size | Use Case |
|--------|-----------|-----------|------------|----------|
| 16×2 | 32 | 32 | 1 KB | Minimal embedded |
| 20×2 | 40 | 40 | 4 KB | Default (i32) |
| 20×3 | 60 | 40 | 4 KB | i64 support |
| 20×4 | 80 | 40 | 4 KB | Large integers |
| 30×2 | 60 | 60 | 128 KB | v3.3 compatible |

---

## 3. Range Checking

### 3.1 Unified Chunk Strategy

**Key invariant**: chunk_bits = limb_bits / 2

This ensures exactly 2 chunks per limb across all configurations.

### 3.2 Chunk Configuration

| Limb Bits | Chunk Bits | Table Size | Memory | L1 Cache |
|-----------|------------|------------|--------|----------|
| 16 | 8 | 256 | 1 KB | All |
| 18 | 9 | 512 | 2 KB | All |
| **20** | **10** | **1,024** | **4 KB** | **All** |
| 22 | 11 | 2,048 | 8 KB | All |
| 24 | 12 | 4,096 | 16 KB | Most |
| 26 | 13 | 8,192 | 32 KB | Large |
| 28 | 14 | 16,384 | 64 KB | GPU |
| 30 | 15 | 32,768 | 128 KB | L2 |

### 3.3 Decomposition

```
decompose(limb: u32, limb_bits: usize) -> [u16; 2]:
    chunk_bits = limb_bits / 2
    mask = (1 << chunk_bits) - 1
    lo = limb & mask
    hi = (limb >> chunk_bits) & mask
    return [lo, hi]
```

### 3.4 Lookups per Value

| Data Limbs | Chunks | Lookups |
|------------|--------|---------|
| 1 | 2 | 2 |
| 2 | 4 | 4 |
| 3 | 6 | 6 |
| 4 | 8 | 8 |

---

## 4. Register File

16 logical registers, each containing (data_limbs × limb_bits) bits:

| Register | Alias | Purpose |
|----------|-------|---------|
| r0 | zero | Hardwired zero |
| r1 | ra | Return address |
| r2 | sp | Stack pointer |
| r3 | gp | Global pointer |
| r4 | tp | Thread pointer |
| r5-r7 | t0-t2 | Temporaries |
| r8 | fp/s0 | Frame pointer |
| r9 | s1 | Saved register |
| r10-r11 | a0-a1 | Arguments/Return |
| r12-r15 | a2-a5 | Arguments |

---

## 5. Instruction Set

### 5.1 Instruction Formats (32-bit)

```
R-type:  [opcode:7][rd:4][rs1:4][rs2:4][funct:13]
I-type:  [opcode:7][rd:4][rs1:4][imm:17]
S-type:  [opcode:7][rs1:4][rs2:4][imm:17]
B-type:  [opcode:7][rs1:4][rs2:4][offset:17]
J-type:  [opcode:7][rd:4][offset:21]
```

### 5.2 Instructions

| Category | Instructions |
|----------|--------------|
| Arithmetic | ADD, SUB, MUL, DIV, REM, ADDI, SUBI, MULI |
| Logical | AND, OR, XOR, NOT, ANDI, ORI, XORI |
| Shift | SLL, SRL, SRA, SLLI, SRLI, SRAI |
| Compare | SLT, SLTU, SEQ, SNE, SLTI, SLTUI |
| Conditional | CMOV, CMOVZ, CMOVN |
| Memory | LB, LH, LW, LD, SB, SH, SW, SD |
| Branch | BEQ, BNE, BLT, BGE, BLTU, BGEU |
| Jump | JAL, JALR |
| System | ECALL, EBREAK |

---

## 6. Memory Model

### 6.1 Address Space (Default 40-bit)

```
0x00_0000_0000  Reserved
0x00_0000_1000  Code (256 MB)
0x00_1000_0000  Static Data (256 MB)
0x00_2000_0000  Heap (~1 TB)
0x40_0000_0000  Stack (~768 GB)
0xFF_FFFF_FFFF  Max 40-bit address
```

### 6.2 Memory Operations

| Op | Bits | Alignment |
|----|------|-----------|
| LB/SB | 8 | 1 byte |
| LH/SH | 16 | 2 bytes |
| LW/SW | 32 | 4 bytes |
| LD/SD | 64 | 8 bytes |

---

## 7. Deferred Range Checking

### 7.1 Headroom Analysis

| Config | Data Bits | i32 Headroom | Max Deferred Adds |
|--------|-----------|--------------|-------------------|
| 16×2 | 32 | 0 | 1 |
| 18×2 | 36 | 4 | 16 |
| **20×2** | **40** | **8** | **256** |
| 22×2 | 44 | 12 | 4,096 |
| 24×2 | 48 | 16 | 65,536 |
| 20×3 | 60 | 28 | 268M |

### 7.2 Checkpoint Rules

Range checks are deferred until:
- Memory store operations
- Branch/jump instructions
- Function calls (ECALL, JALR)
- Division/remainder operations

---

## 8. Cryptographic Syscalls

### 8.1 Adaptive Internal Representation

Crypto syscalls use **adaptive internal widths** based on program configuration:

```
crypto_internal = max(min_internal, program_bits)
```

**Minimum Internal Representation** (guarantees zero intermediate range checks):

| Syscall | Algorithm | Min Internal | Min Headroom | Ops Needed |
|---------|-----------|--------------|--------------|------------|
| SHA-256 | 32-bit | 44-bit | 12 bits | ~320 |
| Blake3 | 32-bit | 44-bit | 12 bits | ~400 |
| Poseidon2 | 31-bit | 40-bit | 9 bits | ~200 |
| Keccak-256 | 64-bit | 80-bit | 16 bits | ~50 |

**Adaptive Behavior**: When program_bits > min_internal, crypto uses program_bits:

| Program | SHA-256 Internal | Headroom |
|---------|------------------|----------|
| 40-bit | 44-bit (min) | 12 bits |
| 60-bit | 60-bit (program) | 28 bits |
| 80-bit | 80-bit (program) | 48 bits |

### 8.2 Boundary Conversion

```
Input:  program_value[data_bits-1:0] → crypto_input[crypto_bits-1:0]
        (truncate high bits if program > crypto)

Output: crypto_output[crypto_bits-1:0] → program_value[data_bits-1:0]
        (zero-extend high bits if program > crypto)
```

### 8.3 Post-Crypto Headroom

After crypto output is converted to program representation:

| Syscall | Algorithm | In 40-bit Program | In 60-bit Program | In 80-bit Program |
|---------|-----------|-------------------|-------------------|-------------------|
| SHA-256 | 32-bit | 8 bits (256 ops) | 28 bits (268M ops) | 48 bits |
| Blake3 | 32-bit | 8 bits (256 ops) | 28 bits (268M ops) | 48 bits |
| Poseidon2 | 31-bit | 9 bits (512 ops) | 29 bits (537M ops) | 49 bits |
| Keccak-256 | 64-bit | **Truncated** | **Truncated** | 16 bits (65K ops) |

### 8.4 Range Check After Crypto Output

**Rule:** Range check is needed ONLY when `algorithm_bits > program_bits`.

| Condition | Range Check | Reason |
|-----------|-------------|--------|
| `algorithm_bits <= program_bits` | **SKIP** | Output fits in program registers |
| `algorithm_bits > program_bits` | **REQUIRED** | Truncation needed |

**Note:** The `internal_bits` (used during crypto execution) does NOT affect this decision.
The crypto output is always bounded by `algorithm_bits`, not `internal_bits`.

### 8.5 Syscall Numbers

```
SYS_EXIT     = 0    SYS_READ    = 1    SYS_WRITE   = 2
SYS_MEMCPY   = 3    SYS_MEMSET  = 4    SYS_BRK     = 5

SYS_SHA256   = 100  SYS_KECCAK  = 101  SYS_POSEIDON = 102
SYS_BLAKE3   = 103
```

---

## 9. Binary Format

### 9.1 Header (32 bytes)

```
Offset  Size  Field
0x00    4     magic         "ZKIR" (0x5A4B4952)
0x04    4     version       v3.4 (0x00030004)
0x08    1     limb_bits     16-30 (default: 20)
0x09    1     data_limbs    1-4 (default: 2)
0x0A    1     addr_limbs    1-2 (default: 2)
0x0B    1     flags         Reserved
0x0C    4     entry_point   Entry address
0x10    4     code_size     Code section size
0x14    4     data_size     Data section size
0x18    4     bss_size      BSS section size
0x1C    4     stack_size    Stack size hint
```

### 9.2 Configuration Validation

```
limb_bits: Must be even, in range [16, 30]
data_limbs: Must be in range [1, 4]
addr_limbs: Must be in range [1, 2]
chunk_bits: Derived as limb_bits / 2
```

---

## 10. Constraint Costs

### 10.1 Per-Operation (2-limb default)

| Operation | Constraints |
|-----------|-------------|
| ADD/SUB | 6 |
| MUL | 12 |
| DIV/REM | 24 |
| AND/OR/XOR | 4 |
| SHL/SHR/SRA | 8 |
| Range Check | 10 |
| LW/SW | 18 |
| Branch | 8 |

### 10.2 Scaling by Limb Count

| Operation | 1 limb | 2 limbs | 3 limbs | 4 limbs |
|-----------|--------|---------|---------|---------|
| ADD | 3 | 6 | 9 | 12 |
| Range Check | 5 | 10 | 15 | 20 |

---

## 11. Constants

```rust
// Default configuration
pub const LIMB_BITS: usize = 20;
pub const CHUNK_BITS: usize = 10;  // LIMB_BITS / 2
pub const DATA_LIMBS: usize = 2;
pub const ADDR_LIMBS: usize = 2;

pub const DATA_BITS: usize = 40;   // LIMB_BITS × DATA_LIMBS
pub const ADDR_BITS: usize = 40;   // LIMB_BITS × ADDR_LIMBS

pub const TABLE_SIZE: usize = 1024;  // 2^CHUNK_BITS
pub const TABLE_BYTES: usize = 4096; // TABLE_SIZE × 4

pub const LIMB_MASK: u32 = 0xFFFFF;  // (1 << LIMB_BITS) - 1
pub const CHUNK_MASK: u32 = 0x3FF;   // (1 << CHUNK_BITS) - 1

pub const HEADROOM_BITS: usize = 8;  // DATA_BITS - 32
pub const MAX_DEFERRED_ADDS: usize = 256;  // 2^HEADROOM_BITS

pub const NUM_REGISTERS: usize = 16;
pub const MERSENNE31_PRIME: u32 = 2147483647;
```

---

## 12. Migration from v3.3

### 12.1 Breaking Changes

1. Default limb size: 30-bit → 20-bit
2. Default value width: 60-bit → 40-bit
3. Header format: New configuration fields
4. Chunk strategy: 10/15-bit → limb_bits/2

### 12.2 Compatibility Mode

For v3.3 compatibility, use:
```
limb_bits: 30
data_limbs: 2
addr_limbs: 2
```

This produces identical behavior to v3.3 (60-bit values).

---

## 13. Documentation

For detailed specifications, see:

- [docs/ZKIR_SPEC_V3.4.md](docs/ZKIR_SPEC_V3.4.md) - Full specification
- [docs/ZKIR_LLVM_V3.4.md](docs/ZKIR_LLVM_V3.4.md) - LLVM backend integration
- [docs/ZKIR_PROVER_V3.4.md](docs/ZKIR_PROVER_V3.4.md) - Prover architecture
- [docs/PROVER_PERFORMANCE_ANALYSIS.md](docs/PROVER_PERFORMANCE_ANALYSIS.md) - Performance analysis

---

**Document Version**: 3.4
