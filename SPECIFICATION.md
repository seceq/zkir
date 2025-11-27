# ZK IR Specification v0.1.0

## Project Overview

**ZK IR** (Zero-Knowledge Intermediate Representation) is a bytecode format designed specifically for efficient zero-knowledge proof generation. It serves as the target for compilation from LLVM IR and executes on a custom zkVM runtime.

### Goals

1. **ZK-Friendly**: Minimal constraints per instruction
2. **LLVM Compatible**: Easy translation from LLVM IR
3. **Simple**: Small instruction set (~30-40 instructions)
4. **Deterministic**: Same inputs always produce same execution
5. **Provable**: Every execution can generate a valid proof

### Non-Goals

1. Not a general-purpose VM (no I/O, syscalls, threads)
2. Not hardware-optimized (no registers for CPU performance)
3. Not backward-compatible with any existing ISA

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         ZK IR VM                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │ Program     │  │ Memory      │  │ Registers               │ │
│  │ Counter     │  │             │  │                         │ │
│  │ (PC)        │  │ 2^32 cells  │  │ r0-r31 (general)        │ │
│  │             │  │ (field      │  │ f0-f15 (field elements) │ │
│  │ 32-bit      │  │  elements)  │  │                         │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Call Stack                                               │   │
│  │ (return addresses, frame pointers)                       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ I/O Buffers                                              │   │
│  │ - Input buffer (read-only)                               │   │
│  │ - Output buffer (write-only)                             │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Field Element

All computation operates over a prime field.

### Primary Field: BabyBear

```
Field: BabyBear
Prime: p = 2^31 - 2^27 + 1 = 2013265921
Size: 31 bits

Properties:
- Fast modular arithmetic
- Efficient for STARKs
- Used by Plonky3 and SP1
```

### Alternative: Goldilocks (for 64-bit operations)

```
Field: Goldilocks  
Prime: p = 2^64 - 2^32 + 1
Size: 64 bits

Use when:
- 64-bit integer operations needed
- Higher precision required
```

---

## Register Set

### General Purpose Registers (r0-r31)

```
r0:  Zero register (always 0, writes ignored)
r1:  Return value / Accumulator
r2:  Stack pointer (SP)
r3:  Frame pointer (FP)
r4-r7:   Function arguments
r8-r15:  Caller-saved temporaries
r16-r23: Callee-saved
r24-r31: Reserved / Temporaries
```

### Field Registers (f0-f15)

```
f0-f15: 256-bit field elements for cryptographic operations
        Used for native field arithmetic, hashing, etc.
```

### Special Registers

```
PC:  Program counter (32-bit)
```

---

## Instruction Set

### Instruction Encoding

```
Fixed 64-bit instruction format:

┌────────┬────────┬────────┬────────┬────────────────────────────┐
│ Opcode │  Dst   │  Src1  │  Src2  │       Immediate            │
│ 8 bits │ 8 bits │ 8 bits │ 8 bits │        32 bits             │
└────────┴────────┴────────┴────────┴────────────────────────────┘

Alternative formats:

Format R (Register):     opcode | dst | src1 | src2 | unused
Format I (Immediate):    opcode | dst | src1 | immediate (32-bit)
Format J (Jump):         opcode | unused | target (48-bit)
Format M (Memory):       opcode | dst | base | offset (32-bit)
```

### Opcode Table

```
Category        | Opcode | Mnemonic    | Description
----------------|--------|-------------|----------------------------------
Arithmetic      | 0x01   | ADD         | dst = src1 + src2
                | 0x02   | SUB         | dst = src1 - src2
                | 0x03   | MUL         | dst = src1 * src2
                | 0x04   | DIV         | dst = src1 / src2 (unsigned)
                | 0x05   | SDIV        | dst = src1 / src2 (signed)
                | 0x06   | MOD         | dst = src1 % src2 (unsigned)
                | 0x07   | SMOD        | dst = src1 % src2 (signed)
                | 0x08   | NEG         | dst = -src1
                
Immediate Arith | 0x10   | ADDI        | dst = src1 + imm
                | 0x11   | SUBI        | dst = src1 - imm
                | 0x12   | MULI        | dst = src1 * imm
                
Logic           | 0x20   | AND         | dst = src1 & src2
                | 0x21   | OR          | dst = src1 | src2
                | 0x22   | XOR         | dst = src1 ^ src2
                | 0x23   | NOT         | dst = ~src1
                | 0x24   | SHL         | dst = src1 << src2
                | 0x25   | SHR         | dst = src1 >> src2 (logical)
                | 0x26   | SAR         | dst = src1 >> src2 (arithmetic)
                
Comparison      | 0x30   | EQ          | dst = (src1 == src2) ? 1 : 0
                | 0x31   | NE          | dst = (src1 != src2) ? 1 : 0
                | 0x32   | LT          | dst = (src1 < src2) ? 1 : 0 (signed)
                | 0x33   | LE          | dst = (src1 <= src2) ? 1 : 0 (signed)
                | 0x34   | GT          | dst = (src1 > src2) ? 1 : 0 (signed)
                | 0x35   | GE          | dst = (src1 >= src2) ? 1 : 0 (signed)
                | 0x36   | LTU         | dst = (src1 < src2) ? 1 : 0 (unsigned)
                | 0x37   | GEU         | dst = (src1 >= src2) ? 1 : 0 (unsigned)
                
Memory          | 0x40   | LOAD        | dst = memory[src1 + offset]
                | 0x41   | STORE       | memory[dst + offset] = src1
                | 0x42   | LOAD8       | dst = memory[src1 + offset] & 0xFF
                | 0x43   | LOAD16      | dst = memory[src1 + offset] & 0xFFFF
                | 0x44   | STORE8      | memory[dst + offset] = src1 & 0xFF
                | 0x45   | STORE16     | memory[dst + offset] = src1 & 0xFFFF
                
Control Flow    | 0x50   | JMP         | pc = target
                | 0x51   | JMPI        | pc = src1 (indirect jump)
                | 0x52   | BEQ         | if (src1 == src2) pc = target
                | 0x53   | BNE         | if (src1 != src2) pc = target
                | 0x54   | BLT         | if (src1 < src2) pc = target (signed)
                | 0x55   | BGE         | if (src1 >= src2) pc = target (signed)
                | 0x56   | BLTU        | if (src1 < src2) pc = target (unsigned)
                | 0x57   | BGEU        | if (src1 >= src2) pc = target (unsigned)
                
Functions       | 0x60   | CALL        | push pc+1, pc = target
                | 0x61   | CALLI       | push pc+1, pc = src1 (indirect)
                | 0x62   | RET         | pc = pop()
                
Constants       | 0x70   | LI          | dst = immediate (32-bit)
                | 0x71   | LUI         | dst = immediate << 32
                | 0x72   | MOV         | dst = src1
                
Field Ops       | 0x80   | FADD        | fdst = fsrc1 + fsrc2 (field)
                | 0x81   | FSUB        | fdst = fsrc1 - fsrc2 (field)
                | 0x82   | FMUL        | fdst = fsrc1 * fsrc2 (field)
                | 0x83   | FINV        | fdst = fsrc1^(-1) (field inverse)
                | 0x84   | FNEG        | fdst = -fsrc1 (field)
                
ZK Primitives   | 0x90   | HASH        | fdst = poseidon(fsrc1, fsrc2)
                | 0x91   | HASH4       | fdst = poseidon(fsrc1..fsrc4)
                | 0x92   | ASSERT_EQ   | assert(src1 == src2)
                | 0x93   | ASSERT_ZERO | assert(src1 == 0)
                | 0x94   | RANGE_CHECK | assert(src1 < 2^imm)
                
I/O             | 0xA0   | READ        | dst = read_input()
                | 0xA1   | WRITE       | write_output(src1)
                | 0xA2   | COMMIT      | commit_public(src1)
                
System          | 0xF0   | NOP         | no operation
                | 0xF1   | HALT        | stop execution
                | 0xFF   | INVALID     | invalid instruction (trap)
```

---

## Instruction Details

### Arithmetic Instructions

#### ADD - Addition

```
Syntax:  ADD dst, src1, src2
Opcode:  0x01
Operation: dst = (src1 + src2) mod p
Flags:   None
Cycles:  1

Constraints (for proving):
  - dst_next = src1_current + src2_current
```

#### MUL - Multiplication

```
Syntax:  MUL dst, src1, src2  
Opcode:  0x03
Operation: dst = (src1 * src2) mod p
Flags:   None
Cycles:  1

Constraints:
  - dst_next = src1_current * src2_current
```

#### DIV - Division

```
Syntax:  DIV dst, src1, src2
Opcode:  0x04
Operation: dst = src1 / src2 (unsigned, truncated)
Flags:   None
Cycles:  1

Constraints:
  - src1 = dst * src2 + remainder
  - remainder < src2
  - (src2 == 0) => TRAP
```

### Memory Instructions

#### LOAD - Load from Memory

```
Syntax:  LOAD dst, offset(base)
Opcode:  0x40
Operation: dst = memory[base + offset]
Encoding: opcode | dst | base | offset (32-bit)

Constraints:
  - address = base + offset
  - dst = memory_read(address)
  - Memory consistency check via permutation argument
```

#### STORE - Store to Memory

```
Syntax:  STORE offset(base), src
Opcode:  0x41  
Operation: memory[base + offset] = src
Encoding: opcode | src | base | offset (32-bit)

Constraints:
  - address = base + offset
  - memory_write(address, src)
  - Memory consistency check via permutation argument
```

### Control Flow Instructions

#### JMP - Unconditional Jump

```
Syntax:  JMP target
Opcode:  0x50
Operation: pc = target
Encoding: opcode | unused | target (48-bit)

Constraints:
  - pc_next = target
```

#### BEQ - Branch if Equal

```
Syntax:  BEQ src1, src2, target
Opcode:  0x52
Operation: if (src1 == src2) pc = target else pc = pc + 1

Constraints:
  - eq = (src1 == src2) ? 1 : 0
  - pc_next = eq * target + (1 - eq) * (pc + 1)
```

#### CALL - Function Call

```
Syntax:  CALL target
Opcode:  0x60
Operation:
  - push(pc + 1)  ; Save return address
  - pc = target

Constraints:
  - stack[sp] = pc + 1
  - sp_next = sp + 1
  - pc_next = target
```

#### RET - Return from Function

```
Syntax:  RET
Opcode:  0x62
Operation:
  - pc = pop()  ; Restore return address

Constraints:
  - sp_next = sp - 1
  - pc_next = stack[sp - 1]
```

### Field Operations

#### FADD - Field Addition

```
Syntax:  FADD fdst, fsrc1, fsrc2
Opcode:  0x80
Operation: fdst = (fsrc1 + fsrc2) mod p

Native field operation - single constraint.
```

#### FMUL - Field Multiplication

```
Syntax:  FMUL fdst, fsrc1, fsrc2
Opcode:  0x81
Operation: fdst = (fsrc1 * fsrc2) mod p

Native field operation - single constraint.
```

#### FINV - Field Inverse

```
Syntax:  FINV fdst, fsrc1
Opcode:  0x83
Operation: fdst = fsrc1^(-1) mod p

Constraints:
  - fdst * fsrc1 = 1 (mod p)
```

### ZK Primitive Instructions

#### HASH - Poseidon Hash

```
Syntax:  HASH fdst, fsrc1, fsrc2
Opcode:  0x90
Operation: fdst = Poseidon(fsrc1, fsrc2)

Uses built-in Poseidon permutation.
Optimized circuit - ~200 constraints.
```

#### ASSERT_EQ - Assert Equality

```
Syntax:  ASSERT_EQ src1, src2
Opcode:  0x92
Operation: if (src1 != src2) TRAP

Constraints:
  - src1 = src2
  
Note: Failure means no valid proof can be generated.
```

#### RANGE_CHECK - Range Check

```
Syntax:  RANGE_CHECK src1, bits
Opcode:  0x94
Operation: assert(src1 < 2^bits)

Constraints:
  - Decompose src1 into `bits` binary values
  - Each bit is 0 or 1
  - Recomposition equals src1
```

### I/O Instructions

#### READ - Read Input

```
Syntax:  READ dst
Opcode:  0xA0
Operation: dst = input_buffer[input_ptr++]

Reads next value from input buffer.
Input buffer is public input to the proof.
```

#### WRITE - Write Output

```
Syntax:  WRITE src
Opcode:  0xA1
Operation: output_buffer[output_ptr++] = src

Writes value to output buffer.
Output buffer becomes public output.
```

#### COMMIT - Commit Public Value

```
Syntax:  COMMIT src
Opcode:  0xA2
Operation: public_values.push(src)

Explicitly marks a value as public output.
Used for proof verification.
```

---

## Binary File Format

### File Structure

```
┌─────────────────────────────────────────────────────────────┐
│                    ZKBC File Format                         │
├─────────────────────────────────────────────────────────────┤
│ Magic Number        │ 4 bytes  │ "ZKBC" (0x5A4B4243)       │
│ Version             │ 4 bytes  │ Format version (0x00010000)│
│ Flags               │ 4 bytes  │ Feature flags              │
│ Header Size         │ 4 bytes  │ Size of header section     │
├─────────────────────────────────────────────────────────────┤
│ Program Header                                              │
│ ├─ Entry Point      │ 4 bytes  │ Starting PC               │
│ ├─ Code Size        │ 4 bytes  │ Number of instructions    │
│ ├─ Data Size        │ 4 bytes  │ Initial memory size       │
│ ├─ Stack Size       │ 4 bytes  │ Stack allocation          │
│ ├─ Num Inputs       │ 4 bytes  │ Expected input count      │
│ ├─ Num Outputs      │ 4 bytes  │ Expected output count     │
│ └─ Checksum         │ 32 bytes │ SHA256 of code section    │
├─────────────────────────────────────────────────────────────┤
│ Code Section                                                │
│ └─ Instructions     │ 8 bytes each │ Encoded instructions  │
├─────────────────────────────────────────────────────────────┤
│ Data Section                                                │
│ └─ Initial Memory   │ variable │ Initial memory contents   │
├─────────────────────────────────────────────────────────────┤
│ Symbol Table (optional)                                     │
│ └─ Debug symbols    │ variable │ Function names, etc.      │
└─────────────────────────────────────────────────────────────┘
```

### Magic Number and Version

```rust
const MAGIC: u32 = 0x5A4B4243;  // "ZKBC"
const VERSION: u32 = 0x00010000; // v1.0.0

struct ZkbcHeader {
    magic: u32,
    version: u32,
    flags: u32,
    header_size: u32,
}
```

### Program Header

```rust
struct ProgramHeader {
    entry_point: u32,
    code_size: u32,
    data_size: u32,
    stack_size: u32,
    num_inputs: u32,
    num_outputs: u32,
    checksum: [u8; 32],
}
```

### Flags

```rust
const FLAG_DEBUG_INFO: u32 = 0x0001;      // Contains debug symbols
const FLAG_OPTIMIZED: u32 = 0x0002;       // Optimizations applied
const FLAG_FIELD_64: u32 = 0x0004;        // Uses 64-bit field
const FLAG_HAS_FIELD_OPS: u32 = 0x0008;   // Uses field registers
```

---

## Memory Model

### Address Space

```
┌─────────────────────────────────────────────────────────────┐
│ 0x00000000 - 0x0000FFFF │ Reserved (64KB)                   │
├─────────────────────────────────────────────────────────────┤
│ 0x00010000 - 0x0FFFFFFF │ Code (256MB)                      │
├─────────────────────────────────────────────────────────────┤
│ 0x10000000 - 0x1FFFFFFF │ Static Data (256MB)               │
├─────────────────────────────────────────────────────────────┤
│ 0x20000000 - 0x7FFFFFFF │ Heap (1.5GB)                      │
├─────────────────────────────────────────────────────────────┤
│ 0x80000000 - 0xFFFFFFFF │ Stack (2GB, grows down)           │
└─────────────────────────────────────────────────────────────┘
```

### Memory Cell

```rust
// Each memory cell holds a field element
type MemoryCell = FieldElement;

// Memory is word-addressed (not byte-addressed)
// Each address holds one field element
```

### Memory Access Rules

```
1. All memory accesses must be aligned
2. Memory is initialized to zero
3. Stack grows downward from 0xFFFFFFFF
4. Heap grows upward from 0x20000000
5. Out-of-bounds access causes TRAP
```

---

## Execution Model

### Execution Cycle

```
1. FETCH:   instruction = code[pc]
2. DECODE:  parse opcode, operands
3. EXECUTE: perform operation
4. TRACE:   record state for proving
5. UPDATE:  pc = next_pc
6. REPEAT:  until HALT or TRAP
```

### Execution State

```rust
struct ExecutionState {
    pc: u32,
    registers: [FieldElement; 32],
    field_registers: [FieldElement; 16],
    memory: HashMap<u32, FieldElement>,
    stack_pointer: u32,
    frame_pointer: u32,
    call_stack: Vec<u32>,
    input_buffer: Vec<FieldElement>,
    input_ptr: usize,
    output_buffer: Vec<FieldElement>,
    halted: bool,
    trapped: bool,
}
```

### Execution Trace

```rust
struct TraceRow {
    cycle: u64,
    pc: u32,
    opcode: u8,
    operands: [u32; 3],
    registers_before: [FieldElement; 32],
    registers_after: [FieldElement; 32],
    memory_addr: Option<u32>,
    memory_value: Option<FieldElement>,
    memory_op: MemoryOp,  // Read, Write, None
}

enum MemoryOp {
    None,
    Read,
    Write,
}
```

---

## Constraint System

### Per-Instruction Constraints

Each instruction type generates specific algebraic constraints:

```
ADD (dst = src1 + src2):
  Constraints: 1
  - registers[dst]' = registers[src1] + registers[src2]

MUL (dst = src1 * src2):
  Constraints: 1
  - registers[dst]' = registers[src1] * registers[src2]

LOAD (dst = mem[addr]):
  Constraints: ~3
  - addr = base + offset
  - registers[dst]' = memory_value
  - Memory permutation check

BEQ (branch if equal):
  Constraints: ~3
  - eq = (src1 == src2)
  - pc' = eq * target + (1-eq) * (pc + 1)
```

### Memory Consistency

```
Memory operations proven via permutation argument:

1. Create sorted list of (address, timestamp, value, is_write)
2. For each consecutive pair with same address:
   - If second is read: value must match previous write
   - If second is write: timestamp must be greater
3. Prove permutation between execution order and sorted order
```

### Estimated Constraints Per Instruction

```
Instruction      | Constraints | Notes
-----------------|-------------|---------------------------
ADD/SUB/MUL      | 1-2         | Simple field operation
DIV/MOD          | 10-15       | Requires inverse + range check
AND/OR/XOR       | 20-30       | Bit decomposition needed
SHL/SHR          | 15-25       | Bit operations
LOAD/STORE       | 3-5         | Memory consistency
BEQ/BNE/etc      | 3-5         | Conditional PC update
CALL/RET         | 5-8         | Stack operations
FADD/FMUL        | 1           | Native field operation
FINV             | 2           | Inverse constraint
HASH             | 200-300     | Poseidon permutation
RANGE_CHECK      | n           | n = number of bits
```

---

## Assembly Syntax

### Basic Syntax

```asm
; Comment (semicolon)
label:              ; Label definition
    ADD r1, r2, r3  ; Instruction

; Register names: r0-r31, f0-f15
; Immediates: decimal (42), hex (0x2A), binary (0b101010)
```

### Example Program

```asm
; Fibonacci sequence
; Input: n (which Fibonacci number to compute)
; Output: fib(n)

.entry main

main:
    READ r4             ; r4 = n (input)
    LI r1, 0            ; r1 = fib(0) = 0
    LI r2, 1            ; r2 = fib(1) = 1
    LI r3, 0            ; r3 = counter
    
loop:
    BEQ r3, r4, done    ; if counter == n, done
    ADD r5, r1, r2      ; r5 = fib(i-1) + fib(i-2)
    MOV r1, r2          ; shift: r1 = old r2
    MOV r2, r5          ; shift: r2 = new fib
    ADDI r3, r3, 1      ; counter++
    JMP loop
    
done:
    WRITE r1            ; output result
    COMMIT r1           ; commit as public output
    HALT
```

### Directives

```asm
.entry <label>      ; Set entry point
.data               ; Start data section
.code               ; Start code section
.word <value>       ; Define word constant
.space <size>       ; Reserve space
.align <n>          ; Align to n bytes
```

---

## Standard Library (Built-in Functions)

### Memory Operations

```
memcpy(dst, src, len)  - Copy memory
memset(dst, val, len)  - Fill memory
```

### Cryptographic Primitives

```
poseidon_hash(a, b)        - 2-to-1 Poseidon hash
poseidon_hash4(a, b, c, d) - 4-to-1 Poseidon hash
merkle_verify(root, leaf, path, index) - Merkle proof verification
```

### Utility Functions

```
assert_eq(a, b)     - Assert equality
range_check(v, bits) - Check value fits in bits
```

---

## Implementation Notes

### Recommended Crates

```toml
[dependencies]
# Field arithmetic
p3-field = { git = "https://github.com/Plonky3/Plonky3" }
p3-baby-bear = { git = "https://github.com/Plonky3/Plonky3" }

# Or use:
ark-ff = "0.4"            # arkworks field arithmetic
goldilocks = "0.2"        # Goldilocks field

# Parsing
nom = "7"                 # Parser combinators
logos = "0.13"            # Lexer generator

# Serialization
bincode = "1.3"           # Binary serialization
serde = { version = "1", features = ["derive"] }

# Utilities
thiserror = "1"           # Error handling
tracing = "0.1"           # Logging
```

### Project Structure

```
zkir/
├── Cargo.toml
├── crates/
│   ├── zkir-spec/           # Types, opcodes, encoding
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── opcode.rs
│   │   │   ├── instruction.rs
│   │   │   ├── program.rs
│   │   │   └── encoding.rs
│   │   └── Cargo.toml
│   │
│   ├── zkir-assembler/      # Assembly parser
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── lexer.rs
│   │   │   ├── parser.rs
│   │   │   └── codegen.rs
│   │   └── Cargo.toml
│   │
│   └── zkir-disassembler/   # Bytecode to assembly
│       ├── src/
│       │   └── lib.rs
│       └── Cargo.toml
│
└── examples/
    ├── fibonacci.zkasm
    ├── sha256.zkasm
    └── merkle.zkasm
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_add_encoding() {
        let instr = Instruction::Add { dst: 1, src1: 2, src2: 3 };
        let encoded = instr.encode();
        let decoded = Instruction::decode(encoded);
        assert_eq!(instr, decoded);
    }
    
    #[test]
    fn test_program_serialization() {
        let program = Program::new(vec![
            Instruction::Li { dst: 1, imm: 42 },
            Instruction::Halt,
        ]);
        let bytes = program.to_bytes();
        let loaded = Program::from_bytes(&bytes);
        assert_eq!(program, loaded);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_fibonacci() {
    let program = assemble_file("examples/fibonacci.zkasm");
    let inputs = vec![FieldElement::from(10)];
    let outputs = execute(&program, inputs);
    assert_eq!(outputs[0], FieldElement::from(55)); // fib(10) = 55
}
```

---

## Version History

| Version | Date       | Changes                     |
|---------|------------|-----------------------------|
| 0.1.0   | 2024-XX-XX | Initial specification       |

---

## References

1. RISC-V Specification - https://riscv.org/specifications/
2. LLVM Language Reference - https://llvm.org/docs/LangRef.html
3. Plonky3 - https://github.com/Plonky3/Plonky3
4. Winterfell - https://github.com/facebook/winterfell
5. SP1 - https://github.com/succinctlabs/sp1
6. Valida - https://github.com/valida-xyz/valida

---

## Appendix A: Opcode Quick Reference

```
0x01 ADD    0x20 AND    0x40 LOAD   0x60 CALL   0x80 FADD   0xA0 READ
0x02 SUB    0x21 OR     0x41 STORE  0x61 CALLI  0x81 FSUB   0xA1 WRITE  
0x03 MUL    0x22 XOR    0x42 LOAD8  0x62 RET    0x82 FMUL   0xA2 COMMIT
0x04 DIV    0x23 NOT    0x43 LOAD16           0x83 FINV
0x05 SDIV   0x24 SHL    0x44 STORE8           0x84 FNEG
0x06 MOD    0x25 SHR    0x45 STORE16
0x07 SMOD   0x26 SAR                          0x90 HASH
0x08 NEG                                       0x91 HASH4
                         0x50 JMP              0x92 ASSERT_EQ
0x10 ADDI   0x30 EQ     0x51 JMPI             0x93 ASSERT_ZERO
0x11 SUBI   0x31 NE     0x52 BEQ              0x94 RANGE_CHECK
0x12 MULI   0x32 LT     0x53 BNE    0x70 LI
            0x33 LE     0x54 BLT    0x71 LUI   0xF0 NOP
            0x34 GT     0x55 BGE    0x72 MOV   0xF1 HALT
            0x35 GE     0x56 BLTU             0xFF INVALID
            0x36 LTU    0x57 BGEU
            0x37 GEU
```

---

## Appendix B: Example Programs

### Example 1: Simple Addition

```asm
.entry main

main:
    READ r1          ; Read first input
    READ r2          ; Read second input
    ADD r3, r1, r2   ; r3 = r1 + r2
    WRITE r3         ; Output result
    HALT
```

### Example 2: Factorial

```asm
.entry main

main:
    READ r1          ; r1 = n
    LI r2, 1         ; r2 = result = 1
    
loop:
    BEQ r1, r0, done ; if n == 0, done
    MUL r2, r2, r1   ; result *= n
    SUBI r1, r1, 1   ; n--
    JMP loop
    
done:
    WRITE r2         ; output result
    HALT
```

### Example 3: Hash Computation

```asm
.entry main

main:
    READ r1              ; Read value to hash
    MOV f0, r1           ; Move to field register
    LI f1, 0             ; Second input = 0
    HASH f2, f0, f1      ; f2 = Poseidon(f0, f1)
    MOV r2, f2           ; Move back to general register
    WRITE r2             ; Output hash
    COMMIT r2            ; Commit as public value
    HALT
```
