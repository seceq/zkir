# ZK IR Specification

## Overview

ZK IR is a 32-bit register-based instruction set designed for zero-knowledge proof generation using Plonky3 with the Baby Bear field.

**Key Design Decisions:**
- 32-bit registers only (no field registers)
- Baby Bear field (p = 2^31 - 2^27 + 1)
- RISC-V inspired 32-bit instruction encoding
- Syscalls for cryptographic operations
- Register pairs for 64-bit values (software convention)

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    ZK IR Architecture                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Integer Registers (32 × 32-bit):                               │
│  ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬───────────┐  │
│  │  r0 │  r1 │  r2 │  r3 │ r4  │ r5  │ r6  │ r7  │ r8-r31    │  │
│  │ =0  │  rv │  sp │  fp │ a0  │ a1  │ a2  │ a3  │ temp/saved│  │
│  └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴───────────┘  │
│                                                                 │
│  NO field registers (use syscalls for crypto)                   │
│                                                                 │
│  Special Registers:                                             │
│  ┌─────────────┐                                                │
│  │     PC      │  Program counter (32-bit)                      │
│  └─────────────┘                                                │
│                                                                 │
│  Memory: 32-bit addressable, 32-bit words, little-endian        │
│  Proving Field: Baby Bear (p = 2^31 - 2^27 + 1)                 │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Design Rationale: No Field Registers

Field registers were removed after analysis showed they hurt performance:

| Factor | With Field Regs | Without | Winner |
|--------|----------------|---------|--------|
| Trace columns | 32 + 32 = 64 | 32 | Without |
| Constraints/cycle | ~96 | ~48 | Without |
| Implementation | Complex | Simple | Without |
| Flexibility | Fixed | Syscalls | Without |

**Register pairs handle 64/128-bit arithmetic efficiently. Syscalls handle crypto.**

---

## Field Specification

### Baby Bear Prime

```
p = 2^31 - 2^27 + 1 = 2013265921 = 0x78000001

Properties:
- 31-bit prime
- Fits in u32 with room for lazy reduction
- 2-adicity: 27 (excellent for FFT)
- Plonky3's native field
```

### Rust Type

```rust
/// Baby Bear field element
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct BabyBear(pub u32);

impl BabyBear {
    pub const MODULUS: u32 = 2013265921;
    pub const ZERO: Self = BabyBear(0);
    pub const ONE: Self = BabyBear(1);
}
```

---

## Register Specification

### Integer Registers (32 × 32-bit)

| Register | Alias | Purpose | Saver |
|----------|-------|---------|-------|
| r0 | zero | Hardwired zero | - |
| r1 | rv | Return value | Caller |
| r2 | sp | Stack pointer | Callee |
| r3 | fp | Frame pointer | Callee |
| r4 | a0 | Argument 0 / Return low | Caller |
| r5 | a1 | Argument 1 / Return high | Caller |
| r6 | a2 | Argument 2 | Caller |
| r7 | a3 | Argument 3 | Caller |
| r8-r15 | t0-t7 | Temporaries | Caller |
| r16-r23 | s0-s7 | Saved registers | Callee |
| r24-r27 | t8-t11 | More temporaries | Caller |
| r28 | gp | Global pointer | - |
| r29 | tp | Thread pointer | - |
| r30 | ra | Return address | Caller |
| r31 | - | Reserved | - |

### Rust Definition

```rust
pub const NUM_REGISTERS: usize = 32;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Register {
    R0 = 0,   // zero - hardwired to 0
    R1 = 1,   // rv   - return value
    R2 = 2,   // sp   - stack pointer
    R3 = 3,   // fp   - frame pointer
    R4 = 4,   // a0   - argument 0
    R5 = 5,   // a1   - argument 1
    R6 = 6,   // a2   - argument 2
    R7 = 7,   // a3   - argument 3
    R8 = 8,   // t0
    R9 = 9,   // t1
    R10 = 10, // t2
    R11 = 11, // t3
    R12 = 12, // t4
    R13 = 13, // t5
    R14 = 14, // t6
    R15 = 15, // t7
    R16 = 16, // s0
    R17 = 17, // s1
    R18 = 18, // s2
    R19 = 19, // s3
    R20 = 20, // s4
    R21 = 21, // s5
    R22 = 22, // s6
    R23 = 23, // s7
    R24 = 24, // t8
    R25 = 25, // t9
    R26 = 26, // t10
    R27 = 27, // t11
    R28 = 28, // gp
    R29 = 29, // tp
    R30 = 30, // ra
    R31 = 31, // reserved
}

impl Register {
    pub const ZERO: Self = Self::R0;
    pub const RV: Self = Self::R1;
    pub const SP: Self = Self::R2;
    pub const FP: Self = Self::R3;
    pub const A0: Self = Self::R4;
    pub const A1: Self = Self::R5;
    pub const A2: Self = Self::R6;
    pub const A3: Self = Self::R7;
    pub const RA: Self = Self::R30;
}
```

---

## Multi-Word Value Conventions

### 64-bit Values (Register Pairs)

```
Convention: Use consecutive even-odd register pairs
  - Low 32 bits:  rN     (even register)
  - High 32 bits: rN+1   (odd register)

Standard pairs:
  (a0, a1) = (r4, r5)   - Argument/return
  (a2, a3) = (r6, r7)   - Argument
  (t0, t1) = (r8, r9)   - Temporary
  (t2, t3) = (r10, r11) - Temporary
  (s0, s1) = (r16, r17) - Saved
```

### 128-bit Values (Register Quads)

```
Convention: Four consecutive registers
  - Bits 0-31:   rN
  - Bits 32-63:  rN+1
  - Bits 64-95:  rN+2
  - Bits 96-127: rN+3

Standard quads:
  (a0, a1, a2, a3) = (r4, r5, r6, r7)
  (t0, t1, t2, t3) = (r8, r9, r10, r11)
```

### Example: 64-bit Addition

```asm
; Add 64-bit values: result = a + b
; Input:  a in (r4, r5), b in (r6, r7)
; Output: result in (r4, r5)

add   r4, r4, r6        ; low = a_lo + b_lo
sltu  t0, r4, r6        ; t0 = carry (1 if overflow)
add   r5, r5, r7        ; high = a_hi + b_hi
add   r5, r5, t0        ; high += carry

; Cost: 4 instructions, ~12 constraints
; Much better than 64-bit registers (which add 64 columns to trace)
```

### Example: 64-bit Comparison (a < b unsigned)

```asm
; Compare 64-bit: result = (a < b) ? 1 : 0
; Input:  a in (r4, r5), b in (r6, r7)
; Output: t0 = result

sltu  t0, r5, r7        ; t0 = (a_hi < b_hi)
xor   t1, r5, r7        ; t1 = (a_hi == b_hi) ? 0 : non-zero
sltiu t1, t1, 1         ; t1 = (a_hi == b_hi) ? 1 : 0
sltu  t2, r4, r6        ; t2 = (a_lo < b_lo)
and   t2, t2, t1        ; t2 = equal_hi && lo_less
or    t0, t0, t2        ; result = hi_less || (equal_hi && lo_less)
```

---

## Instruction Encoding (32-bit, RISC-V Compatible)

### Formats

```
R-type (register-register):
┌─────────┬───────┬───────┬────────┬───────┬─────────┐
│ funct7  │  rs2  │  rs1  │ funct3 │   rd  │ opcode  │
│  7 bits │ 5 bits│ 5 bits│ 3 bits │ 5 bits│ 7 bits  │
└─────────┴───────┴───────┴────────┴───────┴─────────┘
 31     25 24   20 19   15 14    12 11    7 6       0

I-type (immediate):
┌──────────────────┬───────┬────────┬───────┬─────────┐
│     imm[11:0]    │  rs1  │ funct3 │   rd  │ opcode  │
│     12 bits      │ 5 bits│ 3 bits │ 5 bits│ 7 bits  │
└──────────────────┴───────┴────────┴───────┴─────────┘
 31              20 19   15 14    12 11    7 6       0

S-type (store):
┌─────────┬───────┬───────┬────────┬─────────┬─────────┐
│imm[11:5]│  rs2  │  rs1  │ funct3 │imm[4:0] │ opcode  │
│  7 bits │ 5 bits│ 5 bits│ 3 bits │ 5 bits  │ 7 bits  │
└─────────┴───────┴───────┴────────┴─────────┴─────────┘
 31     25 24   20 19   15 14    12 11      7 6       0

B-type (branch):
┌───┬────────┬───────┬───────┬────────┬────────┬───┬─────────┐
│[12]│[10:5] │  rs2  │  rs1  │ funct3 │ [4:1]  │[11]│ opcode  │
│1 b │ 6 bits│ 5 bits│ 5 bits│ 3 bits │ 4 bits │1 b │ 7 bits  │
└───┴────────┴───────┴───────┴────────┴────────┴───┴─────────┘
 31  30    25 24   20 19   15 14    12 11     8  7  6       0

U-type (upper immediate):
┌────────────────────────────┬───────┬─────────┐
│        imm[31:12]          │   rd  │ opcode  │
│         20 bits            │ 5 bits│ 7 bits  │
└────────────────────────────┴───────┴─────────┘
 31                        12 11    7 6       0

J-type (jump):
┌───┬──────────┬───┬──────────┬───────┬─────────┐
│[20]│ [10:1]  │[11]│ [19:12]  │   rd  │ opcode  │
│1 b │ 10 bits │1 b │  8 bits  │ 5 bits│ 7 bits  │
└───┴──────────┴───┴──────────┴───────┴─────────┘
 31  30      21  20  19      12 11    7 6       0
```

### Opcode Map

```
[6:0] Primary Opcode:

0x03  LOAD      (I-type)  LW, LH, LB, LHU, LBU
0x13  OP-IMM    (I-type)  ADDI, SLTI, XORI, ORI, ANDI, SLLI, SRLI, SRAI
0x17  AUIPC     (U-type)  Add upper immediate to PC
0x23  STORE     (S-type)  SW, SH, SB
0x33  OP        (R-type)  ADD, SUB, MUL, DIV, AND, OR, XOR, SLL, SRL, SRA, SLT
0x37  LUI       (U-type)  Load upper immediate
0x63  BRANCH    (B-type)  BEQ, BNE, BLT, BGE, BLTU, BGEU
0x67  JALR      (I-type)  Jump and link register
0x6F  JAL       (J-type)  Jump and link
0x73  SYSTEM    (I-type)  ECALL, EBREAK

0x0B  ZK-CUSTOM (R-type)  ASSERT, COMMIT, RANGE_CHECK
0x5B  ZK-IO     (I-type)  READ, WRITE, HINT
```

---

## Instruction Set

### Arithmetic (R-type: opcode = 0x33)

| Instruction | funct7 | funct3 | Description |
|-------------|--------|--------|-------------|
| ADD rd, rs1, rs2 | 0x00 | 0x0 | rd = rs1 + rs2 |
| SUB rd, rs1, rs2 | 0x20 | 0x0 | rd = rs1 - rs2 |
| MUL rd, rs1, rs2 | 0x01 | 0x0 | rd = (rs1 × rs2)[31:0] |
| MULH rd, rs1, rs2 | 0x01 | 0x1 | rd = (rs1 × rs2)[63:32] (signed) |
| MULHU rd, rs1, rs2 | 0x01 | 0x3 | rd = (rs1 × rs2)[63:32] (unsigned) |
| DIV rd, rs1, rs2 | 0x01 | 0x4 | rd = rs1 / rs2 (signed) |
| DIVU rd, rs1, rs2 | 0x01 | 0x5 | rd = rs1 / rs2 (unsigned) |
| REM rd, rs1, rs2 | 0x01 | 0x6 | rd = rs1 % rs2 (signed) |
| REMU rd, rs1, rs2 | 0x01 | 0x7 | rd = rs1 % rs2 (unsigned) |

### Arithmetic Immediate (I-type: opcode = 0x13)

| Instruction | funct3 | Description |
|-------------|--------|-------------|
| ADDI rd, rs1, imm | 0x0 | rd = rs1 + sext(imm) |
| SLTI rd, rs1, imm | 0x2 | rd = (rs1 < sext(imm)) ? 1 : 0 |
| SLTIU rd, rs1, imm | 0x3 | rd = (rs1 <u sext(imm)) ? 1 : 0 |
| XORI rd, rs1, imm | 0x4 | rd = rs1 ^ sext(imm) |
| ORI rd, rs1, imm | 0x6 | rd = rs1 \| sext(imm) |
| ANDI rd, rs1, imm | 0x7 | rd = rs1 & sext(imm) |
| SLLI rd, rs1, shamt | 0x1 | rd = rs1 << shamt |
| SRLI rd, rs1, shamt | 0x5 | rd = rs1 >> shamt (logical) |
| SRAI rd, rs1, shamt | 0x5 | rd = rs1 >> shamt (arithmetic) |

### Logic (R-type: opcode = 0x33)

| Instruction | funct7 | funct3 | Description |
|-------------|--------|--------|-------------|
| AND rd, rs1, rs2 | 0x00 | 0x7 | rd = rs1 & rs2 |
| OR rd, rs1, rs2 | 0x00 | 0x6 | rd = rs1 \| rs2 |
| XOR rd, rs1, rs2 | 0x00 | 0x4 | rd = rs1 ^ rs2 |
| SLL rd, rs1, rs2 | 0x00 | 0x1 | rd = rs1 << (rs2 & 0x1F) |
| SRL rd, rs1, rs2 | 0x00 | 0x5 | rd = rs1 >> (rs2 & 0x1F) |
| SRA rd, rs1, rs2 | 0x20 | 0x5 | rd = rs1 >>a (rs2 & 0x1F) |
| SLT rd, rs1, rs2 | 0x00 | 0x2 | rd = (rs1 < rs2) ? 1 : 0 |
| SLTU rd, rs1, rs2 | 0x00 | 0x3 | rd = (rs1 <u rs2) ? 1 : 0 |

### Load (I-type: opcode = 0x03)

| Instruction | funct3 | Description |
|-------------|--------|-------------|
| LW rd, imm(rs1) | 0x2 | rd = mem[rs1 + sext(imm)] |
| LH rd, imm(rs1) | 0x1 | rd = sext(mem16[rs1 + sext(imm)]) |
| LHU rd, imm(rs1) | 0x5 | rd = zext(mem16[rs1 + sext(imm)]) |
| LB rd, imm(rs1) | 0x0 | rd = sext(mem8[rs1 + sext(imm)]) |
| LBU rd, imm(rs1) | 0x4 | rd = zext(mem8[rs1 + sext(imm)]) |

### Store (S-type: opcode = 0x23)

| Instruction | funct3 | Description |
|-------------|--------|-------------|
| SW rs2, imm(rs1) | 0x2 | mem[rs1 + sext(imm)] = rs2 |
| SH rs2, imm(rs1) | 0x1 | mem16[rs1 + sext(imm)] = rs2[15:0] |
| SB rs2, imm(rs1) | 0x0 | mem8[rs1 + sext(imm)] = rs2[7:0] |

### Branch (B-type: opcode = 0x63)

| Instruction | funct3 | Description |
|-------------|--------|-------------|
| BEQ rs1, rs2, imm | 0x0 | if (rs1 == rs2) PC += sext(imm) |
| BNE rs1, rs2, imm | 0x1 | if (rs1 != rs2) PC += sext(imm) |
| BLT rs1, rs2, imm | 0x4 | if (rs1 < rs2) PC += sext(imm) |
| BGE rs1, rs2, imm | 0x5 | if (rs1 >= rs2) PC += sext(imm) |
| BLTU rs1, rs2, imm | 0x6 | if (rs1 <u rs2) PC += sext(imm) |
| BGEU rs1, rs2, imm | 0x7 | if (rs1 >=u rs2) PC += sext(imm) |

### Jump

| Instruction | Opcode | Type | Description |
|-------------|--------|------|-------------|
| JAL rd, imm | 0x6F | J | rd = PC+4; PC += sext(imm) |
| JALR rd, rs1, imm | 0x67 | I | rd = PC+4; PC = (rs1+sext(imm)) & ~1 |

### Upper Immediate

| Instruction | Opcode | Type | Description |
|-------------|--------|------|-------------|
| LUI rd, imm | 0x37 | U | rd = imm << 12 |
| AUIPC rd, imm | 0x17 | U | rd = PC + (imm << 12) |

### System (opcode = 0x73)

| Instruction | imm | Description |
|-------------|-----|-------------|
| ECALL | 0x000 | System call (see syscall table) |
| EBREAK | 0x001 | Breakpoint |

### ZK-Custom (opcode = 0x0B)

| Instruction | funct7 | funct3 | Description |
|-------------|--------|--------|-------------|
| ASSERT_EQ rs1, rs2 | 0x00 | 0x0 | Assert rs1 == rs2, halt if false |
| ASSERT_NE rs1, rs2 | 0x00 | 0x1 | Assert rs1 != rs2 |
| ASSERT_ZERO rs1 | 0x01 | 0x0 | Assert rs1 == 0 |
| RANGE_CHECK rs1, bits | 0x02 | 0x0 | Assert rs1 < 2^bits |
| COMMIT rs1 | 0x10 | 0x0 | Add rs1 to public commitments |
| HALT | 0x7F | 0x7 | Halt execution successfully |

### ZK I/O (opcode = 0x5B)

| Instruction | funct3 | Description |
|-------------|--------|-------------|
| READ rd | 0x0 | rd = next public input |
| WRITE rs1 | 0x1 | Output rs1 to public outputs |
| HINT rd | 0x2 | rd = next private hint (non-deterministic) |

---

## Syscalls (ECALL)

Syscalls provide cryptographic operations via dedicated prover chips.

### Calling Convention

```
a7 (r7):  Syscall number
a0-a5:    Arguments (pointers, lengths, etc.)
a0:       Return value (0 = success, non-zero = error)
```

### Syscall Table

| Number | Name | Description |
|--------|------|-------------|
| 0x01 | SYS_POSEIDON2 | Poseidon2 hash |
| 0x02 | SYS_KECCAK256 | Keccak-256 hash |
| 0x03 | SYS_SHA256 | SHA-256 hash |
| 0x04 | SYS_BLAKE3 | BLAKE3 hash |
| 0x10 | SYS_ECDSA_VERIFY | ECDSA signature verify |
| 0x11 | SYS_ED25519_VERIFY | Ed25519 signature verify |
| 0x20 | SYS_BIGINT_ADD | 256-bit addition |
| 0x21 | SYS_BIGINT_MUL | 256-bit multiplication |
| 0x22 | SYS_BIGINT_MOD | 256-bit modular reduction |

### Syscall Details

#### SYS_POSEIDON2 (0x01)

```
Hash using Poseidon2 over Baby Bear field.

Input:
  a0: pointer to input (array of u32, field elements)
  a1: input length (number of field elements)
  a2: pointer to output (8 u32s = 256 bits)

Output:
  a0: 0 on success

Example:
  li    a7, 0x01          ; SYS_POSEIDON2
  la    a0, input_data    ; input pointer
  li    a1, 8             ; 8 field elements
  la    a2, output_hash   ; output pointer
  ecall
```

#### SYS_SHA256 (0x03)

```
SHA-256 hash.

Input:
  a0: pointer to input data (bytes)
  a1: input length in bytes
  a2: pointer to output (32 bytes)

Output:
  a0: 0 on success

Example:
  li    a7, 0x03          ; SYS_SHA256
  la    a0, message       ; message pointer
  li    a1, 64            ; 64 bytes
  la    a2, hash_out      ; output pointer
  ecall
```

#### SYS_ECDSA_VERIFY (0x10)

```
Verify ECDSA signature on secp256k1.

Input:
  a0: pointer to message hash (32 bytes)
  a1: pointer to signature (64 bytes: r || s)
  a2: pointer to public key (64 bytes: x || y)

Output:
  a0: 0 if valid, 1 if invalid

Example:
  li    a7, 0x10          ; SYS_ECDSA_VERIFY
  la    a0, msg_hash
  la    a1, signature
  la    a2, pubkey
  ecall
  bnez  a0, invalid_sig   ; branch if invalid
```

---

## Memory Model

### Address Space

```
0x00000000 ┌─────────────────┐
           │   Reserved      │
0x00001000 ├─────────────────┤
           │   Code (RX)     │
0x10000000 ├─────────────────┤
           │   Data (RW)     │
0x80000000 ├─────────────────┤
           │   Heap ↓        │
           │                 │
           │   Stack ↑       │
0xFFFF0000 ├─────────────────┤
           │   Reserved      │
0xFFFFFFFF └─────────────────┘
```

### Constants

```rust
pub const CODE_BASE: u32 = 0x0000_1000;
pub const DATA_BASE: u32 = 0x1000_0000;
pub const HEAP_BASE: u32 = 0x8000_0000;
pub const STACK_TOP: u32 = 0xFFFF_0000;

pub const DEFAULT_STACK_SIZE: u32 = 1 << 20;  // 1 MB
pub const DEFAULT_HEAP_SIZE: u32 = 1 << 20;   // 1 MB
```

---

## Calling Convention

### Function Calls

```
Arguments:
  a0-a3 (r4-r7):     First 4 arguments (32-bit each)
  stack:             Additional arguments

Return:
  rv (r1):           32-bit return value
  (rv, a0):          64-bit return (low, high)

Caller-saved:  rv, a0-a3, t0-t11, ra
Callee-saved:  sp, fp, s0-s7
```

### 64-bit Arguments

```
64-bit arg 1:  (a0, a1)
64-bit arg 2:  (a2, a3)
Additional:    stack

64-bit return: (rv, a0) = (low, high)
```

### Stack Frame

```
High addr   ┌─────────────────┐
            │  Arg N          │
            │  ...            │
            │  Arg 5          │
            ├─────────────────┤
            │  Return addr    │
            ├─────────────────┤
            │  Saved FP       │ ← FP
            ├─────────────────┤
            │  Saved s0-s7    │
            ├─────────────────┤
            │  Local vars     │
            │  ...            │ ← SP
Low addr    └─────────────────┘
```

---

## Program Binary Format (.zkbc)

### Header

```rust
#[repr(C)]
pub struct ProgramHeader {
    pub magic: u32,         // "ZKIR" = 0x5A4B4952
    pub version: u32,       // Binary format version
    pub flags: u32,         // Reserved
    pub entry_point: u32,   // Entry address
    pub code_size: u32,     // Code section size (bytes)
    pub data_size: u32,     // Data section size (bytes)
    pub bss_size: u32,      // BSS size (bytes)
}
```

### File Layout

```
┌──────────────────────────────┐
│  Header (28 bytes)           │
├──────────────────────────────┤
│  Code Section                │
│  (instructions, 4 bytes each)│
├──────────────────────────────┤
│  Data Section                │
│  (initialized data)          │
├──────────────────────────────┤
│  Symbol Table (optional)     │
└──────────────────────────────┘
```

---

## Assembly Syntax

### Example Program

```asm
.section .text
.global _start

_start:
    ; Read input
    read    a0              ; n = input
    
    ; Call fibonacci
    jal     ra, fib
    
    ; Output result
    write   rv
    halt

; Fibonacci function
; Input: a0 = n
; Output: rv = fib(n)
fib:
    li      t0, 2
    blt     a0, t0, fib_base
    
    ; Iterative fibonacci
    li      t0, 0           ; prev = 0
    li      t1, 1           ; curr = 1
    li      t2, 1           ; i = 1
    
fib_loop:
    beq     t2, a0, fib_done
    add     t3, t0, t1      ; next = prev + curr
    mv      t0, t1          ; prev = curr
    mv      t1, t3          ; curr = next
    addi    t2, t2, 1       ; i++
    j       fib_loop
    
fib_done:
    mv      rv, t1
    ret
    
fib_base:
    mv      rv, a0
    ret

.section .data
; No data needed
```

### Pseudo-instructions

| Pseudo | Expansion |
|--------|-----------|
| `li rd, imm` | `lui` + `addi` |
| `la rd, label` | `auipc` + `addi` |
| `mv rd, rs` | `addi rd, rs, 0` |
| `not rd, rs` | `xori rd, rs, -1` |
| `neg rd, rs` | `sub rd, zero, rs` |
| `j offset` | `jal zero, offset` |
| `jr rs` | `jalr zero, rs, 0` |
| `ret` | `jalr zero, ra, 0` |
| `call label` | `jal ra, label` |
| `nop` | `addi zero, zero, 0` |

---

## Constraint Costs

| Instruction | Constraints | Notes |
|-------------|-------------|-------|
| ADD/SUB/ADDI | ~3 | Arithmetic + range |
| MUL | ~5 | 32×32 multiply |
| DIV/REM | ~30 | Expensive |
| AND/OR/XOR | ~35 | Bit decomposition |
| SLL/SRL/SRA | ~40 | Shift logic |
| LW/SW | ~40 | Memory permutation |
| LB/LH/etc | ~45 | Partial word |
| BEQ/BNE/etc | ~10 | Comparison |
| JAL/JALR | ~5 | PC update |
| ECALL (hash) | ~200-300 | Via dedicated chip |
