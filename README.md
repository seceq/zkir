# ZK IR - Zero-Knowledge Intermediate Representation

A bytecode format and runtime designed for efficient zero-knowledge proof generation.

## Overview

ZK IR is a platform-independent bytecode format that serves as the compilation target from LLVM IR. It's designed specifically for ZK-friendly execution, with minimal constraints per instruction.

```
Rust Source → LLVM IR → ZK IR → zkVM Runtime → Proof
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
cargo build

# Run tests
cargo test

# Build release version
cargo build --release
```

### Usage

```bash
# Assemble a ZK IR program
cargo run --bin zkir -- assemble examples/fibonacci.zkasm -o fibonacci.zkbc

# Disassemble a bytecode file
cargo run --bin zkir -- disasm fibonacci.zkbc

# Execute a program (when runtime is implemented)
cargo run --bin zkir -- run fibonacci.zkbc --input 10
```

## Documentation

- [ZK IR Specification](SPECIFICATION.md) - Complete specification
- [Architecture](ARCHITECTURE.md) - System architecture (TODO)
- [Contributing](CONTRIBUTING.md) - How to contribute (TODO)

## Examples

### Fibonacci

```asm
.entry main

main:
    READ r4             ; n = input
    LI r1, 0            ; fib(0) = 0
    LI r2, 1            ; fib(1) = 1
    LI r3, 0            ; counter = 0
    
loop:
    BEQ r3, r4, done
    ADD r5, r1, r2
    MOV r1, r2
    MOV r2, r5
    ADDI r3, r3, 1
    JMP loop
    
done:
    WRITE r1
    HALT
```

### Run Example

```bash
# Assemble
cargo run --bin zkir -- assemble examples/fibonacci.zkasm -o fib.zkbc

# Execute with input n=10
cargo run --bin zkir -- run fib.zkbc --input 10
# Output: 55
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        ZK IR VM                             │
├─────────────────────────────────────────────────────────────┤
│  Registers: r0-r31 (general), f0-f15 (field elements)       │
│  Memory: 2^32 addressable cells (field elements)            │
│  Stack: Grows downward from 0xFFFFFFFF                      │
└─────────────────────────────────────────────────────────────┘
```

## License

MIT OR Apache-2.0
