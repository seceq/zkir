//! Program representation for ZK IR.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use crate::instruction::Instruction;
use crate::field::FieldElement;
use crate::error::ZkIrError;
use crate::{MAGIC, VERSION};

/// Flags for program features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProgramFlags(u32);

impl ProgramFlags {
    /// Contains debug symbols
    pub const DEBUG_INFO: u32 = 0x0001;
    /// Optimizations were applied
    pub const OPTIMIZED: u32 = 0x0002;
    /// Uses 64-bit field (Goldilocks) instead of BabyBear
    pub const FIELD_64: u32 = 0x0004;
    /// Uses field registers
    pub const HAS_FIELD_OPS: u32 = 0x0008;

    /// Create new empty flags
    pub fn new() -> Self {
        ProgramFlags(0)
    }

    /// Check if a flag is set
    pub fn has(&self, flag: u32) -> bool {
        (self.0 & flag) != 0
    }

    /// Set a flag
    pub fn set(&mut self, flag: u32) {
        self.0 |= flag;
    }

    /// Clear a flag
    pub fn clear(&mut self, flag: u32) {
        self.0 &= !flag;
    }

    /// Get raw value
    pub fn raw(&self) -> u32 {
        self.0
    }
}

/// Program header containing metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgramHeader {
    /// Entry point (instruction index)
    pub entry_point: u32,
    /// Number of expected inputs
    pub num_inputs: u32,
    /// Number of expected outputs
    pub num_outputs: u32,
    /// Stack size in field elements
    pub stack_size: u32,
    /// Initial heap size in field elements
    pub heap_size: u32,
    /// Program flags
    pub flags: ProgramFlags,
    /// SHA256 checksum of code section (computed during serialization)
    #[serde(default)]
    pub checksum: [u8; 32],
}

impl Default for ProgramHeader {
    fn default() -> Self {
        ProgramHeader {
            entry_point: 0,
            num_inputs: 0,
            num_outputs: 0,
            stack_size: crate::DEFAULT_STACK_SIZE,
            heap_size: crate::DEFAULT_HEAP_SIZE,
            flags: ProgramFlags::new(),
            checksum: [0u8; 32],
        }
    }
}

/// A complete ZK IR program
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program {
    /// Program header/metadata
    pub header: ProgramHeader,
    /// Instructions
    pub instructions: Vec<Instruction>,
    /// Initial data section (loaded into memory at program start)
    pub data: Vec<FieldElement>,
    /// Symbol table (label name -> instruction index)
    #[serde(default)]
    pub symbols: std::collections::HashMap<String, u32>,
}

impl Program {
    /// Create a new empty program
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Program {
            header: ProgramHeader::default(),
            instructions,
            data: Vec::new(),
            symbols: std::collections::HashMap::new(),
        }
    }

    /// Create program with header
    pub fn with_header(header: ProgramHeader, instructions: Vec<Instruction>) -> Self {
        Program {
            header,
            instructions,
            data: Vec::new(),
            symbols: std::collections::HashMap::new(),
        }
    }

    /// Get instruction at index
    pub fn get(&self, index: usize) -> Option<&Instruction> {
        self.instructions.get(index)
    }

    /// Number of instructions
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Check if program is empty
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    /// Add a symbol (label)
    pub fn add_symbol(&mut self, name: String, address: u32) {
        self.symbols.insert(name, address);
    }

    /// Lookup symbol by name
    pub fn lookup_symbol(&self, name: &str) -> Option<u32> {
        self.symbols.get(name).copied()
    }

    /// Serialize program to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // First, encode all instructions to compute checksum
        let mut code_bytes = Vec::with_capacity(self.instructions.len() * 8);
        for instr in &self.instructions {
            code_bytes.extend_from_slice(&instr.encode().to_le_bytes());
        }

        // Compute SHA256 checksum of code section
        let mut hasher = Sha256::new();
        hasher.update(&code_bytes);
        let checksum: [u8; 32] = hasher.finalize().into();

        // Magic number
        bytes.extend_from_slice(&MAGIC.to_le_bytes());

        // Version
        bytes.extend_from_slice(&VERSION.to_le_bytes());

        // Flags
        bytes.extend_from_slice(&self.header.flags.raw().to_le_bytes());

        // Header size: 24 bytes of fields + 32 bytes checksum = 56 bytes
        let header_size: u32 = 56;
        bytes.extend_from_slice(&header_size.to_le_bytes());

        // Program header fields (24 bytes)
        bytes.extend_from_slice(&self.header.entry_point.to_le_bytes());      // 4
        bytes.extend_from_slice(&(self.instructions.len() as u32).to_le_bytes()); // 4
        bytes.extend_from_slice(&(self.data.len() as u32).to_le_bytes());     // 4
        bytes.extend_from_slice(&self.header.stack_size.to_le_bytes());       // 4
        bytes.extend_from_slice(&self.header.num_inputs.to_le_bytes());       // 4
        bytes.extend_from_slice(&self.header.num_outputs.to_le_bytes());      // 4

        // Checksum (32 bytes)
        bytes.extend_from_slice(&checksum);

        // Instructions (already encoded)
        bytes.extend_from_slice(&code_bytes);

        // Data section
        for elem in &self.data {
            bytes.extend_from_slice(&elem.to_bytes());
        }

        bytes
    }

    /// Deserialize program from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ZkIrError> {
        if bytes.len() < 16 {
            return Err(ZkIrError::InvalidFormat("File too small".into()));
        }

        // Check magic
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if magic != MAGIC {
            return Err(ZkIrError::InvalidFormat("Invalid magic number".into()));
        }

        // Check version
        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        if version > VERSION {
            return Err(ZkIrError::InvalidFormat(format!(
                "Unsupported version: {:08x}",
                version
            )));
        }

        // Flags
        let flags = ProgramFlags(u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]));

        // Header size
        let header_size = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;

        if bytes.len() < 16 + header_size {
            return Err(ZkIrError::InvalidFormat("Truncated header".into()));
        }

        // Parse header
        let header_bytes = &bytes[16..16 + header_size];
        let entry_point = u32::from_le_bytes([header_bytes[0], header_bytes[1], header_bytes[2], header_bytes[3]]);
        let code_size = u32::from_le_bytes([header_bytes[4], header_bytes[5], header_bytes[6], header_bytes[7]]) as usize;
        let data_size = u32::from_le_bytes([header_bytes[8], header_bytes[9], header_bytes[10], header_bytes[11]]) as usize;
        let stack_size = u32::from_le_bytes([header_bytes[12], header_bytes[13], header_bytes[14], header_bytes[15]]);
        let num_inputs = u32::from_le_bytes([header_bytes[16], header_bytes[17], header_bytes[18], header_bytes[19]]);
        let num_outputs = u32::from_le_bytes([header_bytes[20], header_bytes[21], header_bytes[22], header_bytes[23]]);

        // Read checksum if present (header_size >= 56 means we have 24 bytes of fields + 32 bytes checksum)
        let mut checksum = [0u8; 32];
        let has_checksum = header_size >= 56;
        if has_checksum {
            checksum.copy_from_slice(&header_bytes[24..56]);
        }

        // Parse instructions
        let code_start = 16 + header_size;
        let code_end = code_start + code_size * 8;

        if bytes.len() < code_end {
            return Err(ZkIrError::InvalidFormat("Truncated code section".into()));
        }

        // Get code bytes for checksum verification
        let code_bytes = &bytes[code_start..code_end];

        // Verify checksum if present
        if has_checksum {
            let mut hasher = Sha256::new();
            hasher.update(code_bytes);
            let computed: [u8; 32] = hasher.finalize().into();
            if computed != checksum {
                return Err(ZkIrError::InvalidFormat("Checksum mismatch".into()));
            }
        }

        let mut instructions = Vec::with_capacity(code_size);
        for i in 0..code_size {
            let offset = code_start + i * 8;
            let instr_bytes: [u8; 8] = bytes[offset..offset + 8].try_into().unwrap();
            let encoded = u64::from_le_bytes(instr_bytes);
            let instr = Instruction::decode(encoded)?;
            instructions.push(instr);
        }

        // Parse data section
        let data_start = code_end;
        let data_end = data_start + data_size * 8;

        if bytes.len() < data_end {
            return Err(ZkIrError::InvalidFormat("Truncated data section".into()));
        }

        let mut data = Vec::with_capacity(data_size);
        for i in 0..data_size {
            let offset = data_start + i * 8;
            let elem_bytes: [u8; 8] = bytes[offset..offset + 8].try_into().unwrap();
            data.push(FieldElement::from_bytes(elem_bytes));
        }

        let header = ProgramHeader {
            entry_point,
            num_inputs,
            num_outputs,
            stack_size,
            heap_size: crate::DEFAULT_HEAP_SIZE,
            flags,
            checksum,
        };

        Ok(Program {
            header,
            instructions,
            data,
            symbols: std::collections::HashMap::new(),
        })
    }

    /// Save program to file
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        std::fs::write(path, self.to_bytes())
    }

    /// Load program from file
    pub fn load(path: &str) -> Result<Self, ZkIrError> {
        let bytes = std::fs::read(path)
            .map_err(|e| ZkIrError::IoError(e.to_string()))?;
        Self::from_bytes(&bytes)
    }

    /// Compute SHA256 hash of the entire program (for verification)
    pub fn hash(&self) -> [u8; 32] {
        let bytes = self.to_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        hasher.finalize().into()
    }

    /// Get the checksum of the code section (computed during serialization)
    pub fn code_checksum(&self) -> [u8; 32] {
        self.header.checksum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::register::Register;

    #[test]
    fn test_program_roundtrip() {
        let instructions = vec![
            Instruction::Li { dst: Register::R1, imm: 42 },
            Instruction::Add {
                dst: Register::R3,
                src1: Register::R1,
                src2: Register::R2,
            },
            Instruction::Halt,
        ];

        let program = Program::new(instructions);
        let bytes = program.to_bytes();
        let loaded = Program::from_bytes(&bytes).unwrap();

        assert_eq!(program.instructions, loaded.instructions);
    }

    #[test]
    fn test_program_with_data() {
        let instructions = vec![Instruction::Halt];
        let mut program = Program::new(instructions);
        program.data = vec![
            FieldElement::from(1u64),
            FieldElement::from(2u64),
            FieldElement::from(3u64),
        ];

        let bytes = program.to_bytes();
        let loaded = Program::from_bytes(&bytes).unwrap();

        assert_eq!(program.data, loaded.data);
    }

    #[test]
    fn test_invalid_magic() {
        let bytes = vec![0x00, 0x00, 0x00, 0x00]; // Wrong magic
        let result = Program::from_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_checksum_verification() {
        let instructions = vec![
            Instruction::Li { dst: Register::R1, imm: 42 },
            Instruction::Halt,
        ];
        let program = Program::new(instructions);
        let mut bytes = program.to_bytes();

        // Verify it loads correctly
        assert!(Program::from_bytes(&bytes).is_ok());

        // Corrupt one byte in the code section (after the header)
        // Header is 16 + 56 = 72 bytes, so code starts at byte 72
        let code_start = 72;
        bytes[code_start] ^= 0xFF; // Flip bits

        // Should fail checksum verification
        let result = Program::from_bytes(&bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Checksum"));
    }

    #[test]
    fn test_checksum_is_computed() {
        let instructions = vec![Instruction::Halt];
        let program = Program::new(instructions);
        let bytes = program.to_bytes();
        let loaded = Program::from_bytes(&bytes).unwrap();

        // Checksum should be non-zero after loading
        assert_ne!(loaded.header.checksum, [0u8; 32]);
    }
}
