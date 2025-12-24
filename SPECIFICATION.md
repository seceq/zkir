# ZK IR Specification v2.2

## Overview

ZK IR v2.2 is a **30-bit** register-based instruction set designed for zero-knowledge proof generation using Plonky3 with the Baby Bear field.

**Key Design Decisions:**
- 30-bit data width (fits in Baby Bear field, no range checks needed)
- 30-bit instructions stored in 32-bit slots
- Custom encoding optimized for ZK constraints (~30% fewer than RISC-V)
- Baby Bear field (p = 2^31 - 2^27 + 1)
- Harvard architecture (separate code/data memory)
- Field arithmetic instructions (FADD, FSUB, FMUL, FNEG, FINV)
- 77 total instructions

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    ZK IR v2.2 Architecture                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Registers (32 × 30-bit):                                        │
│  ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬───────────┐  │
│  │  r0 │  r1 │  r2 │  r3 │ r4  │ r5  │ ...│ r17 │ r18-r31   │  │
│  │zero │ ra  │ sp  │ gp  │ tp  │ t0  │    │ a7  │ s2-s11,t3-│  │
│  └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴───────────┘  │
│                                                                 │
│  Special Registers:                                             │
│  ┌─────────────┐                                                │
│  │     PC      │  Program counter (30-bit, stored as 32-bit)    │
│  └─────────────┘                                                │
│                                                                 │
│  Memory: 30-bit addressable, Harvard architecture               │
│  Instructions: 30 bits stored in 32-bit slots (bits 31:30 = 0) │
│  Proving Field: Baby Bear (p = 2,013,265,921)                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Field Specification

### Baby Bear Prime

```
p = 2^31 - 2^27 + 1 = 2,013,265,921 = 0x78000001

Properties:
- 31-bit prime
- 30-bit data range: 0 to 1,073,741,823 (2^30 - 1)
- All 30-bit values < p (no range checks required)
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
    pub const MODULUS: u32 = 2_013_265_921;
    pub const ZERO: Self = BabyBear(0);
    pub const ONE: Self = BabyBear(1);
}
```

---

## Register Specification

### Registers (32 × 30-bit)

| Register | Alias | Purpose | Saver |
|----------|-------|---------|-------|
| r0 | zero | Hardwired zero | - |
| r1 | ra | Return address | Caller |
| r2 | sp | Stack pointer | Callee |
| r3 | gp | Global pointer | - |
| r4 | tp | Thread pointer | - |
| r5-r7 | t0-t2 | Temporaries | Caller |
| r8 | fp/s0 | Frame pointer | Callee |
| r9 | s1 | Saved register | Callee |
| r10-r17 | a0-a7 | Arguments/returns | Caller |
| r18-r27 | s2-s11 | Saved registers | Callee |
| r28-r31 | t3-t6 | Temporaries | Caller |

### Rust Definition

```rust
pub const NUM_REGISTERS: usize = 32;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Register {
    R0 = 0,   // zero
    R1 = 1,   // ra - return address
    R2 = 2,   // sp - stack pointer
    R3 = 3,   // gp - global pointer
    R4 = 4,   // tp - thread pointer
    R5 = 5,   // t0 - temporary
    R6 = 6,   // t1
    R7 = 7,   // t2
    R8 = 8,   // fp/s0 - frame pointer
    R9 = 9,   // s1
    R10 = 10, // a0 - argument 0
    R11 = 11, // a1
    R12 = 12, // a2
    R13 = 13, // a3
    R14 = 14, // a4
    R15 = 15, // a5
    R16 = 16, // a6
    R17 = 17, // a7
    R18 = 18, // s2
    // ... s3-s11
    R28 = 28, // t3
    R29 = 29, // t4
    R30 = 30, // t5
    R31 = 31, // t6
}

impl Register {
    pub const ZERO: Self = Self::R0;
    pub const RA: Self = Self::R1;
    pub const SP: Self = Self::R2;
    pub const FP: Self = Self::R8;
    pub const A0: Self = Self::R10;
    pub const A1: Self = Self::R11;
}
```

---

## Instruction Encoding (30-bit Custom)

### Storage

```
Instructions: 30 bits stored in 32-bit slots
              Bits 31:30 unused (must be 0)
PC increment: +4 (byte-addressed)
Branch/jump:  word units (×4 bytes)
```

### Formats

```
R-type (Register-Register):
| unused | ext | funct |  rs2  |  rs1  |   rd  | opcode |
|  6-bit | 2-b | 3-bit | 5-bit | 5-bit | 5-bit |  4-bit |
  29-24  23-22  21-19   18-14   13-9    8-4      3-0

I-type (Immediate):
| funct |     imm     |  rs1  |   rd  | opcode |
| 3-bit |   13-bit    | 5-bit | 5-bit |  4-bit |
  29-27     26-14       13-9    8-4      3-0

S-type (Store):
|     imm     |  rs2  |  rs1  | funct | opcode |
|   13-bit    | 5-bit | 5-bit | 3-bit |  4-bit |
    29-17       16-12   11-7    6-4      3-0

U-type (Upper Immediate):
|        imm_hi           |   rd  | opcode |
|        21-bit           | 5-bit |  4-bit |
          29-9              8-4      3-0

Z-type (ZK Operations):
|  func  |  imm  |  rs1  |   rd  | opcode |
| 5-bit  | 8-bit | 5-bit | 5-bit |  4-bit |
  29-25    24-17   16-12   11-7     6-4     3-0
```

### Opcode Map (4-bit)

| Opcode | Name | Format | Category |
|--------|------|--------|----------|
| 0000 | ALU | R | R-type operations |
| 0001 | ALUI | I | Immediate ALU |
| 0010 | LOAD | I | Load instructions |
| 0011 | STORE | S | Store instructions |
| 0100 | BEQ | B | Branch equal |
| 0101 | BNE | B | Branch not equal |
| 0110 | BLT | B | Branch less than |
| 0111 | BGE | B | Branch ≥ |
| 1000 | BLTU | B | Branch < unsigned |
| 1001 | BGEU | B | Branch ≥ unsigned |
| 1010 | LUI | U | Load upper immediate |
| 1011 | AUIPC | U | Add upper imm to PC |
| 1100 | JAL | J | Jump and link |
| 1101 | JALR | I | Jump and link register |
| 1110 | ZKOP | Z | ZK operations |
| 1111 | SYSTEM | I | System instructions |

---

## Instruction Set (77 instructions)

### R-type ALU (opcode = 0000)

| funct | ext=00 | ext=01 | ext=10 | ext=11 |
|-------|--------|--------|--------|--------|
| 000 | ADD | SUB | MUL | MULH |
| 001 | AND | ANDN | OR | ORN |
| 010 | XOR | XNOR | SLL | ROL |
| 011 | SRL | SRA | ROR | CLZ |
| 100 | SLT | SLTU | MIN | MAX |
| 101 | MINU | MAXU | MULHU | MULHSU |
| 110 | DIV | DIVU | REM | REMU |
| 111 | REV8 | CPOP | CTZ | (ext2) |

**Field Operations** (bits 29:24 = 111111):
- FADD, FSUB, FMUL, FNEG, FINV

**Conditional Move** (ext2 when ext=11, funct=111):
- CMOVZ, CMOVNZ

### I-type Immediate (opcode = 0001)

| funct | Instruction |
|-------|-------------|
| 000 | ADDI |
| 001 | SLLI |
| 010 | SLTI |
| 011 | SLTIU |
| 100 | XORI |
| 101 | SRLI/SRAI |
| 110 | ORI |
| 111 | ANDI |

### Load (opcode = 0010)

| funct | Instruction |
|-------|-------------|
| 000 | LB |
| 001 | LH |
| 010 | LW |
| 100 | LBU |
| 101 | LHU |

### Store (opcode = 0011)

| funct | Instruction |
|-------|-------------|
| 000 | SB |
| 001 | SH |
| 010 | SW |

### Branch (opcodes 0100-1001)

- BEQ, BNE, BLT, BGE, BLTU, BGEU

### Upper Immediate (opcodes 1010-1011)

- LUI: `rd = imm << 9`
- AUIPC: `rd = PC + (imm << 9)`

### Jump (opcodes 1100-1101)

- JAL: `rd = PC + 4; PC += offset × 4`
- JALR: `rd = PC + 4; PC = (rs1 + imm) & ~3`

### ZK Operations (opcode = 1110)

| func | Instruction | Description |
|------|-------------|-------------|
| 00000 | READ rd | Read next public input |
| 00001 | WRITE rs1 | Write to public output |
| 00010 | HINT rd | Read next private hint |
| 00011 | COMMIT rs1 | Add to commitments |
| 00100 | ASSERT_EQ | Trap if rs1 != rs2 |
| 00101 | ASSERT_NE | Trap if rs1 == rs2 |
| 00110 | ASSERT_ZERO | Trap if rs1 != 0 |
| 00111 | RANGE_CHECK | Trap if rs1 >= 2^imm |
| 01000 | DEBUG rs1 | Emit rs1 (no-op in prover) |
| 11111 | HALT | Halt execution |

### System (opcode = 1111)

- ECALL: System call (a7 = syscall number)
- EBREAK: Breakpoint (halts, dumps state, proof fails)

---

## Syscalls (ECALL)

### Calling Convention

```
a7 (r17):  Syscall number
a0-a5:     Arguments
a0:        Return value (0 = success)
```

### Syscall Table

| Number | Name | Description | Constraints |
|--------|------|-------------|-------------|
| 0x01 | SYS_READ_PUBLIC | Read public input | ~3 |
| 0x02 | SYS_READ_PRIVATE | Read private input | ~3 |
| 0x03 | SYS_WRITE_OUTPUT | Write public output | ~3 |
| 0x04 | SYS_WRITE_COMMIT | Write commitment | ~3 |
| 0x05 | SYS_HINT | Non-deterministic advice | ~3 |
| 0x10 | SYS_SHA256 | SHA-256 hash | ~500 |
| 0x12 | SYS_POSEIDON2 | Poseidon2 hash (recommended) | ~300 |
| 0x13 | SYS_POSEIDON | Poseidon hash (legacy) | ~500 |
| 0xFF | SYS_HALT | Halt execution | ~1 |

#### SYS_POSEIDON2 (0x12) - Recommended

```
Hash using Poseidon2 over Baby Bear field.

Input:
  a0: input pointer (30-bit field elements)
  a1: input length (number of elements)
  a2: output pointer (8 elements)

Returns:
  a0 = 0 on success

Example:
  li    a7, 0x12          ; SYS_POSEIDON2
  la    a0, input_data
  li    a1, 8
  la    a2, output_hash
  ecall
```

#### SYS_SHA256 (0x10)

```
SHA-256 hash.

Input:
  a0: input pointer (bytes)
  a1: input length (bytes)
  a2: output pointer (32 bytes)

Returns:
  a0 = 0 on success
```

---

## Memory Model

### Address Space (30-bit)

```
0x00000000 ┌─────────────────┐
           │   Reserved      │
0x00001000 ├─────────────────┤
           │   Code (RX)     │
0x10000000 ├─────────────────┤
           │   Data (RW)     │
0x20000000 ├─────────────────┤
           │   Heap ↓        │
           │                 │
           │   Stack ↑       │
0x3FFFF000 ├─────────────────┤
           │   Reserved      │
0x3FFFFFFF └─────────────────┘
```

### Constants

```rust
pub const CODE_BASE: u32 = 0x0000_1000;
pub const DATA_BASE: u32 = 0x1000_0000;
pub const HEAP_BASE: u32 = 0x2000_0000;
pub const STACK_TOP: u32 = 0x3FFF_F000;
pub const MAX_30BIT: u32 = (1 << 30) - 1;
```

### Alignment Requirements

| Access | Alignment |
|--------|-----------|
| LW/SW | 4-byte aligned |
| LH/SH/LHU | 2-byte aligned |
| LB/SB/LBU | No requirement |

**Misaligned access traps** (proof fails).

---

## Behavioral Semantics

| Condition | Behavior |
|-----------|----------|
| Division by zero | Trap (proof fails) |
| Integer overflow | Wrap mod 2^30 |
| Misaligned access | Trap (proof fails) |
| Write to r0 | Ignored |
| Read from r0 | Returns 0 |
| EBREAK | Halts, dumps state, proof fails |
| DEBUG | No-op in prover, emits in interpreter |

---

## Calling Convention

### Function Calls

```
Arguments:     a0-a7 (r10-r17), then stack
Return:        a0 (r10), a1 (r11) if needed
Caller-saved:  ra, t0-t6, a0-a7
Callee-saved:  sp, fp, s0-s11
Stack:         Grows downward
```

### Stack Frame

```
High addr   ┌─────────────────┐
            │  Arg N          │
            │  ...            │
            ├─────────────────┤
            │  Return addr    │
            ├─────────────────┤
            │  Saved FP       │ ← FP
            ├─────────────────┤
            │  Saved s0-s11   │
            ├─────────────────┤
            │  Local vars     │ ← SP
Low addr    └─────────────────┘
```

---

## Program Binary Format (.zkbc)

### Header (28 bytes)

```rust
#[repr(C)]
pub struct ProgramHeader {
    pub magic: u32,         // "ZK22" = 0x5A4B3232
    pub version: u32,       // 0x00020002
    pub flags: u32,
    pub entry_point: u32,
    pub code_size: u32,     // bytes
    pub data_size: u32,     // bytes
    pub bss_size: u32,      // bytes
}
```

### File Layout

```
[Header (28 bytes)]
[Code Section (30-bit instructions in 32-bit slots)]
[Data Section (initialized data)]
[Symbol Table (optional)]
```

---

## Assembly Syntax

### Pseudo-Instructions

| Pseudo | Expansion |
|--------|-----------|
| `li rd, imm` | ADDI/LUI+ADDI |
| `mv rd, rs` | `addi rd, rs, 0` |
| `not rd, rs` | `xori rd, rs, -1` |
| `neg rd, rs` | `sub rd, zero, rs` |
| `j offset` | `jal zero, offset` |
| `jr rs` | `jalr zero, rs, 0` |
| `ret` | `jalr zero, ra, 0` |
| `call offset` | `jal ra, offset` |
| `nop` | `addi zero, zero, 0` |
| `seqz rd, rs` | `sltiu rd, rs, 1` |
| `snez rd, rs` | `sltu rd, zero, rs` |
| `beqz rs, off` | `beq rs, zero, off` |
| `bnez rs, off` | `bne rs, zero, off` |
| `fmov rd, rs` | `fadd rd, rs, zero` |

### Example Program

```asm
.section .text
.global _start

_start:
    ; Read input
    read    a0              ; n = input

    ; Call fibonacci
    call    fib

    ; Output result
    write   a0
    halt

; Fibonacci function
; Input: a0 = n
; Output: a0 = fib(n)
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
    mv      a0, t1
    ret

fib_base:
    ret
```

---

## Constraint Costs

| Category | Instructions | Constraints/op |
|----------|--------------|----------------|
| Decode | All | ~43 |
| Arithmetic | ADD, SUB, ADDI | ~3 |
| Multiply | MUL, MULH | ~5 |
| Division | DIV, REM | ~30 |
| Shifts | SLL, SRL, SRA, ROL, ROR | ~40 |
| Bit ops | CLZ, CTZ, CPOP, REV8 | ~94-238 |
| Field ops | FADD, FSUB | ~3-5 |
| Field mul | FMUL | ~5 |
| Field inv | FINV | ~30 |
| Conditional | CMOVZ, CMOVNZ | ~35 |
| Memory | LW, SW | ~40 |
| Branch | BEQ, BNE, etc. | ~10 |
| Hash (Poseidon2) | Syscall | ~300 |
| Hash (SHA-256) | Syscall | ~500 |

**Total per instruction**: ~43 (decode) + operation cost

**30-bit encoding saves ~30% constraints vs 32-bit RISC-V**

---

## v2.2 Highlights

1. **30-bit native architecture** - All values fit in Baby Bear field
2. **Custom encoding** - 30% fewer constraints than RISC-V
3. **Field arithmetic** - FADD, FSUB, FMUL, FNEG, FINV (3-30 constraints)
4. **Poseidon2 support** - 40% faster than Poseidon
5. **Debug support** - DEBUG instruction + EBREAK behavior defined
6. **77 instructions** - Complete ISA with bit manipulation, conditional moves

For detailed encoding specifications, see [docs/CUSTOM_ENCODING_DESIGN.md](docs/CUSTOM_ENCODING_DESIGN.md).

For full instruction details, see [docs/ZKIR_SPEC_V2.2_FINAL.md](docs/ZKIR_SPEC_V2.2_FINAL.md).
