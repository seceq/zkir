//! # Configuration System for ZKIR v3.4
//!
//! This module provides the configuration system for variable limb architecture.
//! Programs can configure their data width and address space based on requirements.

use std::fmt;

/// Program configuration for variable limb architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Config {
    /// Limb size in bits (16-30, must be even)
    pub limb_bits: u8,
    /// Number of limbs for data values (1-4)
    pub data_limbs: u8,
    /// Number of limbs for addresses (1-2)
    pub addr_limbs: u8,
}

impl Config {
    /// Default configuration: 20-bit × 2 limbs = 40-bit values/addresses
    /// - Limb bits: 20
    /// - Data limbs: 2 (40-bit values)
    /// - Address limbs: 2 (40-bit addresses)
    /// - Chunk bits: 10 (limb_bits / 2)
    /// - Table size: 1024 (4 KB)
    /// - Headroom: 8 bits (256 deferred adds)
    pub const DEFAULT: Self = Self {
        limb_bits: 20,
        data_limbs: 2,
        addr_limbs: 2,
    };

    /// Create a new configuration with validation
    pub const fn new(limb_bits: u8, data_limbs: u8, addr_limbs: u8) -> Result<Self, ConfigError> {
        let config = Self {
            limb_bits,
            data_limbs,
            addr_limbs,
        };

        // Validate at compile time
        if limb_bits < 16 || limb_bits > 30 {
            return Err(ConfigError::InvalidLimbBits);
        }
        if limb_bits % 2 != 0 {
            return Err(ConfigError::OddLimbBits);
        }
        if data_limbs < 1 || data_limbs > 4 {
            return Err(ConfigError::InvalidDataLimbs);
        }
        if addr_limbs < 1 || addr_limbs > 2 {
            return Err(ConfigError::InvalidAddrLimbs);
        }

        Ok(config)
    }

    /// Total data width in bits
    #[inline]
    pub const fn data_bits(&self) -> u32 {
        self.limb_bits as u32 * self.data_limbs as u32
    }

    /// Total address width in bits
    #[inline]
    pub const fn addr_bits(&self) -> u32 {
        self.limb_bits as u32 * self.addr_limbs as u32
    }

    /// Chunk size in bits (always limb_bits / 2)
    #[inline]
    pub const fn chunk_bits(&self) -> u32 {
        self.limb_bits as u32 / 2
    }

    /// Range check table size (2^chunk_bits)
    #[inline]
    pub const fn table_size(&self) -> usize {
        1 << self.chunk_bits()
    }

    /// Range check table memory in bytes
    #[inline]
    pub const fn table_bytes(&self) -> usize {
        self.table_size() * 4
    }

    /// Limb mask (all ones for limb_bits)
    #[inline]
    pub const fn limb_mask(&self) -> u32 {
        (1u32 << self.limb_bits) - 1
    }

    /// Chunk mask (all ones for chunk_bits)
    #[inline]
    pub const fn chunk_mask(&self) -> u32 {
        (1u32 << self.chunk_bits()) - 1
    }

    /// Headroom bits for i32 operations (data_bits - 32)
    /// Returns 0 if data_bits < 32
    #[inline]
    pub const fn headroom(&self) -> u32 {
        let data_bits = self.data_bits();
        if data_bits >= 32 {
            data_bits - 32
        } else {
            0
        }
    }

    /// Maximum number of additions that can be deferred before range check
    #[inline]
    pub const fn max_deferred_adds(&self) -> usize {
        let headroom = self.headroom();
        if headroom == 0 {
            1
        } else {
            1 << headroom
        }
    }

    /// Maximum number of multiplications that can be deferred before range check
    /// Formula: floor((headroom - 1) / 2)
    #[inline]
    pub const fn max_deferred_muls(&self) -> usize {
        let headroom = self.headroom();
        if headroom <= 1 {
            0
        } else {
            ((headroom - 1) / 2) as usize
        }
    }

    /// Chunks per limb (always 2 for our decomposition strategy)
    #[inline]
    pub const fn chunks_per_limb(&self) -> usize {
        2
    }

    /// Total chunks per data value
    #[inline]
    pub const fn chunks_per_value(&self) -> usize {
        self.data_limbs as usize * 2
    }

    /// Total chunks per address
    #[inline]
    pub const fn chunks_per_addr(&self) -> usize {
        self.addr_limbs as usize * 2
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Limb bits must be even and in range [16, 30]
        if self.limb_bits < 16 || self.limb_bits > 30 {
            return Err(ConfigError::InvalidLimbBits);
        }
        if self.limb_bits % 2 != 0 {
            return Err(ConfigError::OddLimbBits);
        }

        // Data limbs must be in range [1, 4]
        if self.data_limbs < 1 || self.data_limbs > 4 {
            return Err(ConfigError::InvalidDataLimbs);
        }

        // Address limbs must be in range [1, 2]
        if self.addr_limbs < 1 || self.addr_limbs > 2 {
            return Err(ConfigError::InvalidAddrLimbs);
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Config {{ limb_bits: {}, data: {}×{}={} bits, addr: {}×{}={} bits, chunks: {}-bit, table: {} ({} KB) }}",
            self.limb_bits,
            self.data_limbs,
            self.limb_bits,
            self.data_bits(),
            self.addr_limbs,
            self.limb_bits,
            self.addr_bits(),
            self.chunk_bits(),
            self.table_size(),
            self.table_bytes() / 1024,
        )
    }
}

/// Configuration error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    /// Limb bits must be in range [16, 30]
    InvalidLimbBits,
    /// Limb bits must be even
    OddLimbBits,
    /// Data limbs must be in range [1, 4]
    InvalidDataLimbs,
    /// Address limbs must be in range [1, 2]
    InvalidAddrLimbs,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidLimbBits => {
                write!(f, "limb_bits must be in range [16, 30]")
            }
            ConfigError::OddLimbBits => {
                write!(f, "limb_bits must be even")
            }
            ConfigError::InvalidDataLimbs => {
                write!(f, "data_limbs must be in range [1, 4]")
            }
            ConfigError::InvalidAddrLimbs => {
                write!(f, "addr_limbs must be in range [1, 2]")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::DEFAULT;
        assert_eq!(config.limb_bits, 20);
        assert_eq!(config.data_limbs, 2);
        assert_eq!(config.addr_limbs, 2);
        assert_eq!(config.data_bits(), 40);
        assert_eq!(config.addr_bits(), 40);
        assert_eq!(config.chunk_bits(), 10);
        assert_eq!(config.table_size(), 1024);
        assert_eq!(config.headroom(), 8);
        assert_eq!(config.max_deferred_adds(), 256);
    }

    #[test]
    fn test_various_configs() {
        // 16-bit × 2 (32-bit, minimal headroom)
        let c = Config::new(16, 2, 2).unwrap();
        assert_eq!(c.data_bits(), 32);
        assert_eq!(c.chunk_bits(), 8);
        assert_eq!(c.table_size(), 256);
        assert_eq!(c.headroom(), 0);

        // 30-bit × 2 (60-bit, v3.3 compat)
        let c = Config::new(30, 2, 2).unwrap();
        assert_eq!(c.data_bits(), 60);
        assert_eq!(c.chunk_bits(), 15);
        assert_eq!(c.table_size(), 32768);
        assert_eq!(c.headroom(), 28);

        // 20-bit × 3 (60-bit, optimal table)
        let c = Config::new(20, 3, 2).unwrap();
        assert_eq!(c.data_bits(), 60);
        assert_eq!(c.chunk_bits(), 10);
        assert_eq!(c.table_size(), 1024);
        assert_eq!(c.headroom(), 28);
    }

    #[test]
    fn test_validation() {
        // Valid configs
        assert!(Config::new(20, 2, 2).is_ok());
        assert!(Config::new(16, 1, 1).is_ok());
        assert!(Config::new(30, 4, 2).is_ok());

        // Invalid limb bits
        assert_eq!(
            Config::new(15, 2, 2).unwrap_err(),
            ConfigError::InvalidLimbBits
        );
        assert_eq!(
            Config::new(32, 2, 2).unwrap_err(),
            ConfigError::InvalidLimbBits
        );

        // Odd limb bits
        assert_eq!(
            Config::new(17, 2, 2).unwrap_err(),
            ConfigError::OddLimbBits
        );

        // Invalid data limbs
        assert_eq!(
            Config::new(20, 0, 2).unwrap_err(),
            ConfigError::InvalidDataLimbs
        );
        assert_eq!(
            Config::new(20, 5, 2).unwrap_err(),
            ConfigError::InvalidDataLimbs
        );

        // Invalid address limbs
        assert_eq!(
            Config::new(20, 2, 0).unwrap_err(),
            ConfigError::InvalidAddrLimbs
        );
        assert_eq!(
            Config::new(20, 2, 3).unwrap_err(),
            ConfigError::InvalidAddrLimbs
        );
    }

    #[test]
    fn test_deferred_operations() {
        // 20-bit × 2 (8-bit headroom)
        let c = Config::DEFAULT;
        assert_eq!(c.max_deferred_adds(), 256);
        assert_eq!(c.max_deferred_muls(), 3);

        // 30-bit × 2 (28-bit headroom)
        let c = Config::new(30, 2, 2).unwrap();
        assert_eq!(c.max_deferred_adds(), 1 << 28);
        assert_eq!(c.max_deferred_muls(), 13);

        // 16-bit × 2 (0-bit headroom)
        let c = Config::new(16, 2, 2).unwrap();
        assert_eq!(c.max_deferred_adds(), 1);
        assert_eq!(c.max_deferred_muls(), 0);
    }
}
