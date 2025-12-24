<p align="center">
  <img src="zkir.svg" alt="zkir Logo" width="400">
</p>

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.17747636.svg)](https://doi.org/10.5281/zenodo.17747636)

> **Paper:** [ZK IR: A Minimalist Instruction Set Architecture for Efficient Zero-Knowledge Proof Generation](https://zenodo.org/records/17747636)
>
> Mamone Tarsha-Kurdi, SECEQ Research

A **30-bit native** register-based instruction set and runtime designed for efficient zero-knowledge proof generation with Plonky3.

## Overview

ZK IR v2.2 is a custom bytecode format optimized for ZK proof generation. It features a **30-bit native architecture** where all values fit within the Baby Bear prime field, eliminating the need for range checks on most operations.

**Key Features:**
- **30-bit native architecture** - All values ≤ 2^30-1 < Baby Bear prime
- Custom 30-bit instruction encoding (stored in 32-bit slots)
- Baby Bear prime field (p = 2^31 - 2^27 + 1 = 2,013,265,921)
- RISC-V calling convention (ra, sp, a0-a7, s2-s11, etc.)
- 77 instructions including field arithmetic (FADD, FSUB, FMUL, FNEG, FINV)
- Minimal constraints per instruction
- Syscalls for Poseidon2, SHA-256, memory operations

```
Rust Source → LLVM IR → ZK IR → Runtime → Plonky3 Proof
```

## Quick Start

### Prerequisites

- Rust 1.75+ (install from https://rustup.rs)
- Cargo

### Build

```bash
# Clone the repository
git clone <your-repo-url>
cd zkir

# Build all crates
cargo build --all

# Run tests
cargo test --all

# Build release version
cargo build --all --release
```

## Architecture (v2.2)

```
┌─────────────────────────────────────────────────────────────────┐
│                ZK IR v2.2 - 30-bit Native                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Registers (32 × 30-bit) - RISC-V Convention:                   │
│  ┌─────┬─────┬─────┬─────┬─────┬─────┬────────┬──────────────┐  │
│  │  r0 │  r1 │  r2 │ r3  │ r4  │ r5-7│ r8-r9  │ r10-r31      │  │
│  │zero │ ra  │ sp  │ gp  │ tp  │t0-t2│ fp, s1 │ a0-a7, s2-s11│  │
│  └─────┴─────┴─────┴─────┴─────┴─────┴────────┴──────────────┘  │
│                                                                 │
│  Field Arithmetic Instructions (FADD, FSUB, FMUL, FNEG, FINV)   │
│  All operations stay within 30-bit (no overflow to check!)      │
│                                                                 │
│  Memory: 30-bit addressable, 30-bit words, Harvard architecture │
│  Field: Baby Bear (p = 2^31 - 2^27 + 1 = 2,013,265,921)        │
│                                                                 │
│  Instructions: 30-bit encoding (bits 31:30 = 0), 4-bit opcodes  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Documentation

- **[SPECIFICATION.md](SPECIFICATION.md)** - Complete ISA specification

## Example: Fibonacci

```asm
.section .text
.global _start

_start:
    read    a0              ; n = input
    jal     ra, fib         ; call fibonacci
    write   rv              ; output result
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
```

## Instruction Set Overview (77 Instructions)

### Categories

- **Arithmetic** (10): add, sub, mul, mulh, mulhu, mulhsu, div, divu, rem, remu
- **Logic** (6): and, andn, or, orn, xor, xnor
- **Shift** (5): sll, srl, sra, rol, ror
- **Compare** (6): slt, sltu, min, max, minu, maxu
- **Bit Manipulation** (4): clz, ctz, cpop, rev8
- **Conditional** (2): cmovz, cmovnz
- **Field Arithmetic** (5): fadd, fsub, fmul, fneg, finv
- **Immediate** (9): addi, slti, sltiu, xori, ori, andi, slli, srli, srai
- **Memory** (8): lw, lh, lb, lhu, lbu, sw, sh, sb
- **Control Flow** (10): beq, bne, blt, bge, bltu, bgeu, jal, jalr, lui, auipc
- **System** (2): ecall, ebreak
- **ZK Operations** (10): read, write, hint, commit, assert_eq, assert_ne, assert_zero, range_check, debug, halt

### Syscalls (via ECALL)

| Number | Name | Description |
|--------|------|-------------|
| 0x01 | SYS_EXIT | Exit with code |
| 0x10 | SYS_READ | Read public input |
| 0x11 | SYS_WRITE | Write public output |
| 0x12 | SYS_POSEIDON2 | Poseidon2 permutation (12-element state) |
| 0x13 | SYS_POSEIDON | Original Poseidon hash |
| 0x20 | SYS_SHA256_INIT | Initialize SHA-256 context |
| 0x21 | SYS_SHA256_UPDATE | Update SHA-256 |
| 0x22 | SYS_SHA256_FINALIZE | Finalize SHA-256 |
| 0x30 | SYS_MEMCPY | Copy memory region |
| 0x31 | SYS_MEMSET | Set memory region |
| 0x32 | SYS_BRK | Adjust heap break |

## Library Usage

### zkir-spec

```rust
use zkir_spec::{Program, Instruction, Register, BabyBear};

// Create a simple program
let code = vec![
    0x0000000F, // ecall (v2.2 encoding)
    0x3E00000E, // halt (v2.2 encoding)
];

let program = Program::new(code);
let hash = program.hash();
```

### zkir-assembler

```rust
use zkir_assembler::assemble;

// Assemble from text
let source = r#"
    .text
    addi a0, zero, 10
    addi a1, zero, 32
    add a2, a0, a1
    write a2
    halt
"#;

let program = assemble(source).unwrap();

// Or encode manually
use zkir_spec::{Instruction, Register};
use zkir_assembler::encode;

let halt = encode(&Instruction::Halt);
let add = encode(&Instruction::Add {
    rd: Register::A0,
    rs1: Register::A1,
    rs2: Register::A2,
});
```

### zkir-disassembler

```rust
use zkir_spec::Program;
use zkir_disassembler::disassemble;

let code = vec![0x0000000F]; // ecall (v2.2)
let program = Program::new(code);
let asm = disassemble(&program)?;
println!("{}", asm);
// Output:
// ; ZK IR Disassembly
// 0x00001000:  0000000F  ecall
```

### zkir-runtime

```rust
use zkir_runtime::{VM, VMConfig};
use zkir_spec::{Program, Instruction, Register};
use zkir_assembler::encoder::encode;

// Build a simple add program
let code = vec![
    encode(&Instruction::Read { rd: Register::A0 }),
    encode(&Instruction::Read { rd: Register::A1 }),
    encode(&Instruction::Add {
        rd: Register::A2,
        rs1: Register::A0,
        rs2: Register::A1,
    }),
    encode(&Instruction::Write { rs1: Register::A2 }),
    encode(&Instruction::Halt),
];

let program = Program::new(code);
let inputs = vec![10, 32];
let config = VMConfig::default();

let vm = VM::new(program, inputs, config);
let result = vm.run()?;
println!("Output: {:?}", result.outputs); // [42]
println!("Cycles: {}", result.cycles);    // 5
```

## Testing

```bash
# Run all tests (93 tests total)
cargo test --all

# Run tests for specific crate
cargo test -p zkir-spec
cargo test -p zkir-assembler    # 61 tests
cargo test -p zkir-disassembler # 13 tests
cargo test -p zkir-runtime      # 17 tests

# Run with output
cargo test -- --nocapture

# Current test status: ✅ All 93 tests passing
```

## Contributing

Contributions welcome! Please:

1. Follow the existing code style
2. Add tests for new functionality
3. Update documentation
4. Run `cargo fmt` and `cargo clippy` before submitting

## Performance Targets

| Metric | Target |
|--------|--------|
| Execution speed | > 50M cycles/sec |
| Trace generation | > 10M steps/sec |
| Constraints per instruction | < 50 avg |

## License

MIT OR Apache-2.0
