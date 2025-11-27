//! Program structure for ZK IR

use serde::{Deserialize, Serialize};

/// Program header
#[repr(C)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProgramHeader {
    /// Magic number: "ZKIR" = 0x5A4B4952
    pub magic: u32,

    /// Version
    pub version: u32,

    /// Flags (reserved)
    pub flags: u32,

    /// Entry point address
    pub entry_point: u32,

    /// Code section size in bytes
    pub code_size: u32,

    /// Data section size in bytes
    pub data_size: u32,

    /// BSS (uninitialized data) size in bytes
    pub bss_size: u32,
}

impl ProgramHeader {
    pub const MAGIC: u32 = 0x5A4B4952; // "ZKIR"
    pub const VERSION: u32 = 0x00020001;
    pub const SIZE: usize = 28;
}

/// Complete program
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Program {
    pub header: ProgramHeader,
    pub code: Vec<u32>,
    pub data: Vec<u8>,
}

impl Program {
    /// Create new program
    pub fn new(code: Vec<u32>) -> Self {
        let header = ProgramHeader {
            magic: ProgramHeader::MAGIC,
            version: ProgramHeader::VERSION,
            flags: 0,
            entry_point: crate::CODE_BASE,
            code_size: (code.len() * 4) as u32,
            data_size: 0,
            bss_size: 0,
        };

        Program {
            header,
            code,
            data: Vec::new(),
        }
    }

    /// Compute SHA-256 hash of program
    pub fn hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();

        // Hash header
        hasher.update(&self.header.magic.to_le_bytes());
        hasher.update(&self.header.version.to_le_bytes());
        hasher.update(&self.header.entry_point.to_le_bytes());
        hasher.update(&self.header.code_size.to_le_bytes());

        // Hash code
        for &word in &self.code {
            hasher.update(&word.to_le_bytes());
        }

        // Hash data
        hasher.update(&self.data);

        hasher.finalize().into()
    }
}
