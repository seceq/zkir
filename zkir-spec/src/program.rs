//! # Program Structure for ZKIR v3.4
//!
//! Program header and binary format with variable limb configuration.

use crate::config::{Config, ConfigError};
use crate::error::ZkIrError;
use std::fmt;

/// Magic number for ZKIR files: "ZKIR" = 0x5A4B4952
pub const MAGIC: u32 = 0x5A4B4952;

/// Version: v3.4 = 0x00030004
pub const VERSION: u32 = 0x00030004;

/// Program header for ZKIR v3.4 (32 bytes)
///
/// Binary format:
/// ```text
/// Offset  Size  Field
/// ──────────────────────────────────
/// 0x00    4     magic ("ZKIR")
/// 0x04    4     version (v3.4)
/// 0x08    1     limb_bits (16-30)
/// 0x09    1     data_limbs (1-4)
/// 0x0A    1     addr_limbs (1-2)
/// 0x0B    1     flags (reserved)
/// 0x0C    4     entry_point
/// 0x10    4     code_size
/// 0x14    4     data_size
/// 0x18    4     bss_size
/// 0x1C    4     stack_size
/// ```
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramHeader {
    /// Magic number: "ZKIR" = 0x5A4B4952
    pub magic: u32,

    /// Version: 0x00030004 for v3.4
    pub version: u32,

    /// Limb size in bits (16-30, must be even)
    pub limb_bits: u8,

    /// Number of limbs for data values (1-4)
    pub data_limbs: u8,

    /// Number of limbs for addresses (1-2)
    pub addr_limbs: u8,

    /// Flags (reserved for future use)
    pub flags: u8,

    /// Entry point address
    pub entry_point: u32,

    /// Code section size in bytes
    pub code_size: u32,

    /// Data section size in bytes
    pub data_size: u32,

    /// BSS section size in bytes
    pub bss_size: u32,

    /// Stack size hint in bytes
    pub stack_size: u32,
}

impl ProgramHeader {
    /// Header size in bytes
    pub const SIZE: usize = 32;

    /// Create a new header with default configuration
    pub fn new() -> Self {
        let config = Config::DEFAULT;
        Self {
            magic: MAGIC,
            version: VERSION,
            limb_bits: config.limb_bits,
            data_limbs: config.data_limbs,
            addr_limbs: config.addr_limbs,
            flags: 0,
            entry_point: 0x1000, // CODE_BASE
            code_size: 0,
            data_size: 0,
            bss_size: 0,
            stack_size: 1 << 20, // 1 MB default
        }
    }

    /// Create a header with custom configuration
    pub fn with_config(config: Config) -> Result<Self, ConfigError> {
        config.validate()?;
        Ok(Self {
            magic: MAGIC,
            version: VERSION,
            limb_bits: config.limb_bits,
            data_limbs: config.data_limbs,
            addr_limbs: config.addr_limbs,
            flags: 0,
            entry_point: 0x1000,
            code_size: 0,
            data_size: 0,
            bss_size: 0,
            stack_size: 1 << 20,
        })
    }

    /// Get the configuration from this header
    pub fn config(&self) -> Config {
        Config {
            limb_bits: self.limb_bits,
            data_limbs: self.data_limbs,
            addr_limbs: self.addr_limbs,
        }
    }

    /// Validate the header
    pub fn validate(&self) -> Result<(), ZkIrError> {
        // Check magic
        if self.magic != MAGIC {
            return Err(ZkIrError::InvalidMagic(self.magic));
        }

        // Check version
        if self.version != VERSION {
            return Err(ZkIrError::InvalidVersion {
                expected: VERSION,
                found: self.version,
            });
        }

        // Validate configuration
        self.config()
            .validate()
            .map_err(ZkIrError::InvalidConfig)?;

        Ok(())
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];

        bytes[0..4].copy_from_slice(&self.magic.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.version.to_le_bytes());
        bytes[8] = self.limb_bits;
        bytes[9] = self.data_limbs;
        bytes[10] = self.addr_limbs;
        bytes[11] = self.flags;
        bytes[12..16].copy_from_slice(&self.entry_point.to_le_bytes());
        bytes[16..20].copy_from_slice(&self.code_size.to_le_bytes());
        bytes[20..24].copy_from_slice(&self.data_size.to_le_bytes());
        bytes[24..28].copy_from_slice(&self.bss_size.to_le_bytes());
        bytes[28..32].copy_from_slice(&self.stack_size.to_le_bytes());

        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ZkIrError> {
        if bytes.len() < Self::SIZE {
            return Err(ZkIrError::InvalidHeaderSize {
                expected: Self::SIZE,
                found: bytes.len(),
            });
        }

        let header = Self {
            magic: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            version: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            limb_bits: bytes[8],
            data_limbs: bytes[9],
            addr_limbs: bytes[10],
            flags: bytes[11],
            entry_point: u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
            code_size: u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
            data_size: u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
            bss_size: u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]),
            stack_size: u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]),
        };

        header.validate()?;
        Ok(header)
    }
}

impl Default for ProgramHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ProgramHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ZKIR Program Header v3.4")?;
        writeln!(f, "  Magic:       {:#010x}", self.magic)?;
        writeln!(f, "  Version:     {:#010x}", self.version)?;
        writeln!(f, "  Config:      {} × {}-bit limbs", self.data_limbs, self.limb_bits)?;
        writeln!(f, "  Data bits:   {}", self.config().data_bits())?;
        writeln!(f, "  Addr bits:   {}", self.config().addr_bits())?;
        writeln!(f, "  Entry:       {:#010x}", self.entry_point)?;
        writeln!(f, "  Code size:   {} bytes", self.code_size)?;
        writeln!(f, "  Data size:   {} bytes", self.data_size)?;
        writeln!(f, "  BSS size:    {} bytes", self.bss_size)?;
        writeln!(f, "  Stack size:  {} bytes", self.stack_size)?;
        Ok(())
    }
}

/// Complete program structure
#[derive(Clone, Debug)]
pub struct Program {
    /// Program header
    pub header: ProgramHeader,

    /// Code section (instructions as u32)
    pub code: Vec<u32>,

    /// Data section (initialized data)
    pub data: Vec<u8>,
}

impl Program {
    /// Create a new empty program
    pub fn new() -> Self {
        Self {
            header: ProgramHeader::new(),
            code: Vec::new(),
            data: Vec::new(),
        }
    }

    /// Create a program with custom configuration
    pub fn with_config(config: Config) -> Result<Self, ConfigError> {
        Ok(Self {
            header: ProgramHeader::with_config(config)?,
            code: Vec::new(),
            data: Vec::new(),
        })
    }

    /// Get the configuration
    pub fn config(&self) -> Config {
        self.header.config()
    }

    /// Validate the program
    pub fn validate(&self) -> Result<(), ZkIrError> {
        self.header.validate()?;

        // Validate code size
        if self.code.len() * 4 != self.header.code_size as usize {
            return Err(ZkIrError::InvalidCodeSize {
                expected: self.header.code_size as usize,
                found: self.code.len() * 4,
            });
        }

        // Validate data size
        if self.data.len() != self.header.data_size as usize {
            return Err(ZkIrError::InvalidDataSize {
                expected: self.header.data_size as usize,
                found: self.data.len(),
            });
        }

        Ok(())
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Write header
        bytes.extend_from_slice(&self.header.to_bytes());

        // Write code section
        for &instr in &self.code {
            bytes.extend_from_slice(&instr.to_le_bytes());
        }

        // Write data section
        bytes.extend_from_slice(&self.data);

        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ZkIrError> {
        // Parse header
        let header = ProgramHeader::from_bytes(bytes)?;

        let code_start = ProgramHeader::SIZE;
        let code_end = code_start + header.code_size as usize;
        let data_end = code_end + header.data_size as usize;

        if bytes.len() < data_end {
            return Err(ZkIrError::InvalidProgramSize {
                expected: data_end,
                found: bytes.len(),
            });
        }

        // Parse code section
        let code_bytes = &bytes[code_start..code_end];
        let mut code = Vec::with_capacity(header.code_size as usize / 4);
        for chunk in code_bytes.chunks_exact(4) {
            code.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }

        // Parse data section
        let data = bytes[code_end..data_end].to_vec();

        let program = Self { header, code, data };
        program.validate()?;
        Ok(program)
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_default() {
        let header = ProgramHeader::new();
        assert_eq!(header.magic, MAGIC);
        assert_eq!(header.version, VERSION);
        assert_eq!(header.limb_bits, 20);
        assert_eq!(header.data_limbs, 2);
        assert_eq!(header.addr_limbs, 2);
    }

    #[test]
    fn test_header_serialization() {
        let header = ProgramHeader::new();
        let bytes = header.to_bytes();
        let deserialized = ProgramHeader::from_bytes(&bytes).unwrap();
        assert_eq!(header, deserialized);
    }

    #[test]
    fn test_header_validation() {
        let mut header = ProgramHeader::new();
        assert!(header.validate().is_ok());

        // Invalid magic
        header.magic = 0x12345678;
        assert!(header.validate().is_err());
        header.magic = MAGIC;

        // Invalid limb bits
        header.limb_bits = 15;
        assert!(header.validate().is_err());
        header.limb_bits = 20;

        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_program_serialization() {
        let mut program = Program::new();
        program.code = vec![0x12345678, 0xABCDEF01];
        program.data = vec![1, 2, 3, 4];
        program.header.code_size = 8;
        program.header.data_size = 4;

        let bytes = program.to_bytes();
        let deserialized = Program::from_bytes(&bytes).unwrap();

        assert_eq!(program.header, deserialized.header);
        assert_eq!(program.code, deserialized.code);
        assert_eq!(program.data, deserialized.data);
    }
}
