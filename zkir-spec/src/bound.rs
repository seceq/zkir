//! # Value Bound Tracking for ZKIR v3.4
//!
//! This module provides bound tracking for range check optimization with
//! crypto-aware bound propagation.

use std::fmt;

/// Crypto operation types with adaptive internal widths
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CryptoType {
    /// SHA-256 (32-bit algorithm, 44-bit min internal)
    Sha256,
    /// Keccak-256 (64-bit algorithm, 80-bit min internal)
    Keccak256,
    /// Poseidon2 (31-bit algorithm, 40-bit min internal)
    Poseidon2,
    /// Blake3 (32-bit algorithm, 44-bit min internal)
    Blake3,
}

impl CryptoType {
    /// Algorithm's semantic bit width (what the algorithm natively operates on)
    #[inline]
    pub const fn algorithm_bits(self) -> u32 {
        match self {
            CryptoType::Sha256 => 32,
            CryptoType::Keccak256 => 64,
            CryptoType::Poseidon2 => 31,
            CryptoType::Blake3 => 32,
        }
    }

    /// Minimum internal representation (guarantees zero intermediate range checks)
    #[inline]
    pub const fn min_internal_bits(self) -> u32 {
        match self {
            CryptoType::Sha256 => 44,   // 12-bit headroom for ~320 ops
            CryptoType::Blake3 => 44,   // 12-bit headroom for ~400 ops
            CryptoType::Poseidon2 => 40, // 9-bit headroom for ~200 ops
            CryptoType::Keccak256 => 80, // 16-bit headroom for ~50 ops
        }
    }

    /// Adaptive internal width: max(min_internal, program_bits)
    /// This ensures maximum headroom while guaranteeing zero intermediate checks
    #[inline]
    pub const fn internal_bits(self, program_bits: u32) -> u32 {
        let min = self.min_internal_bits();
        if program_bits > min {
            program_bits
        } else {
            min
        }
    }

    /// Internal headroom during crypto execution
    #[inline]
    pub const fn internal_headroom(self, program_bits: u32) -> u32 {
        self.internal_bits(program_bits) - self.algorithm_bits()
    }

    /// Post-crypto headroom (after output conversion to program representation)
    /// Output is bounded to algorithm_bits, so headroom is program_bits - algorithm_bits
    #[inline]
    pub const fn post_crypto_headroom(self, program_bits: u32) -> u32 {
        if program_bits >= self.algorithm_bits() {
            program_bits - self.algorithm_bits()
        } else {
            0
        }
    }

    /// Check if range check is needed when algorithm_bits > program_bits
    #[inline]
    pub const fn needs_range_check(self, program_bits: u32) -> bool {
        self.algorithm_bits() > program_bits
    }
}

/// Source of a value bound
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoundSource {
    /// Full program width
    ProgramWidth,
    /// Narrower type (i8, i16, i32)
    TypeWidth(u32),
    /// Crypto syscall output
    CryptoOutput(CryptoType),
    /// Computed from operations
    Computed,
    /// Known constant
    Constant(u64),
}

impl BoundSource {
    /// Get the bit width for this source
    /// For crypto outputs, returns algorithm_bits (not internal_bits)
    pub fn bits(&self) -> Option<u32> {
        match self {
            BoundSource::TypeWidth(bits) => Some(*bits),
            BoundSource::CryptoOutput(crypto) => Some(crypto.algorithm_bits()),
            BoundSource::Constant(val) => {
                if *val == 0 {
                    Some(0)
                } else {
                    Some(64 - val.leading_zeros())
                }
            }
            _ => None,
        }
    }
}

/// Value bound for range check optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueBound {
    /// Maximum bits the value can occupy
    pub max_bits: u32,
    /// Source of this bound
    pub source: BoundSource,
}

impl ValueBound {
    /// Create a bound from program width
    #[inline]
    pub const fn from_program_width(data_bits: u32) -> Self {
        Self {
            max_bits: data_bits,
            source: BoundSource::ProgramWidth,
        }
    }

    /// Create a bound from a type width
    #[inline]
    pub const fn from_type_width(bits: u32) -> Self {
        Self {
            max_bits: bits,
            source: BoundSource::TypeWidth(bits),
        }
    }

    /// Create a bound from crypto output
    /// Output is always bounded by algorithm_bits (not internal_bits)
    #[inline]
    pub const fn from_crypto(crypto_type: CryptoType) -> Self {
        Self {
            max_bits: crypto_type.algorithm_bits(),
            source: BoundSource::CryptoOutput(crypto_type),
        }
    }

    /// Create a bound from a constant value
    #[inline]
    pub fn from_constant(val: u64) -> Self {
        let bits = if val == 0 {
            0
        } else {
            64 - val.leading_zeros()
        };
        Self {
            max_bits: bits,
            source: BoundSource::Constant(val),
        }
    }

    /// Create a computed bound
    #[inline]
    pub const fn computed(max_bits: u32) -> Self {
        Self {
            max_bits,
            source: BoundSource::Computed,
        }
    }

    /// Headroom available for deferred operations
    #[inline]
    pub const fn headroom(&self, data_bits: u32) -> u32 {
        if data_bits >= self.max_bits {
            data_bits - self.max_bits
        } else {
            0
        }
    }

    /// Check if range check is needed
    #[inline]
    pub const fn needs_range_check(&self, data_bits: u32) -> bool {
        self.max_bits > data_bits
    }

    /// Check if value fits in target bits without range check
    #[inline]
    pub const fn fits_in(&self, target_bits: u32) -> bool {
        self.max_bits <= target_bits
    }

    // ========== Bound Propagation Rules ==========

    /// Bound after ADD: max(a, b) + 1
    #[inline]
    pub fn after_add(a: &Self, b: &Self) -> Self {
        Self::computed(a.max_bits.max(b.max_bits).saturating_add(1))
    }

    /// Bound after SUB: max(a, b)
    #[inline]
    pub fn after_sub(a: &Self, b: &Self) -> Self {
        Self::computed(a.max_bits.max(b.max_bits))
    }

    /// Bound after MUL: a + b
    #[inline]
    pub fn after_mul(a: &Self, b: &Self) -> Self {
        Self::computed(a.max_bits.saturating_add(b.max_bits))
    }

    /// Bound after DIV: dividend bound (quotient <= dividend)
    #[inline]
    pub fn after_div(dividend: &Self, _divisor: &Self) -> Self {
        Self::computed(dividend.max_bits)
    }

    /// Bound after REM: min(dividend, divisor)
    #[inline]
    pub fn after_rem(dividend: &Self, divisor: &Self) -> Self {
        Self::computed(dividend.max_bits.min(divisor.max_bits))
    }

    /// Bound after AND: min(a, b)
    #[inline]
    pub fn after_and(a: &Self, b: &Self) -> Self {
        Self::computed(a.max_bits.min(b.max_bits))
    }

    /// Bound after OR: max(a, b)
    #[inline]
    pub fn after_or(a: &Self, b: &Self) -> Self {
        Self::computed(a.max_bits.max(b.max_bits))
    }

    /// Bound after XOR: max(a, b)
    #[inline]
    pub fn after_xor(a: &Self, b: &Self) -> Self {
        Self::computed(a.max_bits.max(b.max_bits))
    }

    /// Bound after NOT: same as input (bit inversion doesn't change width)
    #[inline]
    pub fn after_not(_a: &Self, data_bits: u32) -> Self {
        Self::computed(data_bits) // NOT fills to full width
    }

    /// Bound after SHL: bits + shift
    #[inline]
    pub fn after_shl(a: &Self, shift: u32, max_bits: u32) -> Self {
        Self::computed(a.max_bits.saturating_add(shift).min(max_bits))
    }

    /// Bound after SRL (logical right shift): bits - shift
    #[inline]
    pub fn after_srl(a: &Self, shift: u32) -> Self {
        Self::computed(a.max_bits.saturating_sub(shift))
    }

    /// Bound after SRA (arithmetic right shift): same as SRL for unsigned
    #[inline]
    pub fn after_sra(a: &Self, shift: u32, data_bits: u32) -> Self {
        // For signed values, arithmetic shift can fill with 1s
        // Conservative: assume full width if negative
        if a.max_bits >= data_bits {
            Self::computed(data_bits)
        } else {
            Self::computed(a.max_bits.saturating_sub(shift))
        }
    }

    /// Bound after comparison: always 1 bit (boolean result)
    #[inline]
    pub fn after_cmp() -> Self {
        Self::computed(1)
    }

    /// Bound after sign extension
    #[inline]
    pub fn after_sign_extend(_a: &Self, to_bits: u32) -> Self {
        Self::computed(to_bits)
    }

    /// Bound after zero extension
    #[inline]
    pub fn after_zero_extend(a: &Self, to_bits: u32) -> Self {
        Self::computed(a.max_bits.min(to_bits))
    }

    /// Bound after truncation
    #[inline]
    pub fn after_truncate(_a: &Self, to_bits: u32) -> Self {
        Self::computed(to_bits)
    }
}

impl fmt::Display for ValueBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.source {
            BoundSource::ProgramWidth => write!(f, "{} bits (program)", self.max_bits),
            BoundSource::TypeWidth(bits) => write!(f, "{} bits (type: {})", self.max_bits, bits),
            BoundSource::CryptoOutput(crypto) => {
                write!(f, "{} bits (crypto: {:?})", self.max_bits, crypto)
            }
            BoundSource::Computed => write!(f, "{} bits (computed)", self.max_bits),
            BoundSource::Constant(val) => write!(f, "{} bits (const: {})", self.max_bits, val),
        }
    }
}

/// Wrapper for values with bound tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedValue<V> {
    pub value: V,
    pub bound: ValueBound,
}

impl<V> BoundedValue<V> {
    /// Create a new bounded value
    #[inline]
    pub const fn new(value: V, bound: ValueBound) -> Self {
        Self { value, bound }
    }

    /// Check if range check is needed for target width
    #[inline]
    pub const fn needs_range_check(&self, data_bits: u32) -> bool {
        self.bound.needs_range_check(data_bits)
    }

    /// Get headroom for deferred operations
    #[inline]
    pub const fn headroom(&self, data_bits: u32) -> u32 {
        self.bound.headroom(data_bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_adaptive_internal() {
        // SHA-256: 32-bit algorithm, 44-bit min internal
        let sha = CryptoType::Sha256;
        assert_eq!(sha.algorithm_bits(), 32);
        assert_eq!(sha.min_internal_bits(), 44);

        // In 40-bit program: uses 44-bit internal (min)
        assert_eq!(sha.internal_bits(40), 44);
        assert_eq!(sha.internal_headroom(40), 12); // 44 - 32

        // In 60-bit program: uses 60-bit internal (adaptive)
        assert_eq!(sha.internal_bits(60), 60);
        assert_eq!(sha.internal_headroom(60), 28); // 60 - 32

        // Keccak-256: 64-bit algorithm, 80-bit min internal
        let keccak = CryptoType::Keccak256;
        assert_eq!(keccak.algorithm_bits(), 64);
        assert_eq!(keccak.min_internal_bits(), 80);
        assert_eq!(keccak.internal_bits(40), 80); // uses min
        assert_eq!(keccak.internal_bits(100), 100); // uses program
    }

    #[test]
    fn test_crypto_output_bounds() {
        // SHA-256 output in 40-bit program
        let sha_output = ValueBound::from_crypto(CryptoType::Sha256);
        assert_eq!(sha_output.max_bits, 32); // Algorithm width, not internal
        assert!(!sha_output.needs_range_check(40)); // 32 <= 40
        assert_eq!(sha_output.headroom(40), 8); // Post-crypto headroom

        // Keccak-256 output in 40-bit program
        let keccak_output = ValueBound::from_crypto(CryptoType::Keccak256);
        assert_eq!(keccak_output.max_bits, 64); // Algorithm width
        assert!(keccak_output.needs_range_check(40)); // 64 > 40

        // Keccak-256 output in 80-bit program
        assert!(!keccak_output.needs_range_check(80)); // 64 <= 80
        assert_eq!(keccak_output.headroom(80), 16); // Post-crypto headroom

        // Poseidon2 output in 40-bit program
        let poseidon_output = ValueBound::from_crypto(CryptoType::Poseidon2);
        assert_eq!(poseidon_output.max_bits, 31);
        assert!(!poseidon_output.needs_range_check(40));
        assert_eq!(poseidon_output.headroom(40), 9);
    }

    #[test]
    fn test_crypto_post_headroom() {
        // SHA-256 in different program configs
        let sha = CryptoType::Sha256;
        assert_eq!(sha.post_crypto_headroom(32), 0);  // 32 - 32
        assert_eq!(sha.post_crypto_headroom(40), 8);  // 40 - 32
        assert_eq!(sha.post_crypto_headroom(60), 28); // 60 - 32

        // Keccak-256 in different program configs
        let keccak = CryptoType::Keccak256;
        assert_eq!(keccak.post_crypto_headroom(40), 0);  // 40 < 64
        assert_eq!(keccak.post_crypto_headroom(64), 0);  // 64 - 64
        assert_eq!(keccak.post_crypto_headroom(80), 16); // 80 - 64
    }

    #[test]
    fn test_crypto_range_check_rules() {
        // Rule: Range check ONLY when algorithm_bits > program_bits

        // SHA-256 (32-bit algorithm)
        let sha = CryptoType::Sha256;
        assert!(!sha.needs_range_check(32)); // 32 <= 32: SKIP
        assert!(!sha.needs_range_check(40)); // 32 <= 40: SKIP
        assert!(!sha.needs_range_check(60)); // 32 <= 60: SKIP
        assert!(sha.needs_range_check(30));  // 32 > 30: REQUIRED

        // Keccak-256 (64-bit algorithm)
        let keccak = CryptoType::Keccak256;
        assert!(keccak.needs_range_check(40));  // 64 > 40: REQUIRED
        assert!(keccak.needs_range_check(60));  // 64 > 60: REQUIRED
        assert!(!keccak.needs_range_check(64)); // 64 <= 64: SKIP
        assert!(!keccak.needs_range_check(80)); // 64 <= 80: SKIP

        // Poseidon2 (31-bit algorithm)
        let poseidon = CryptoType::Poseidon2;
        assert!(poseidon.needs_range_check(30));  // 31 > 30: REQUIRED
        assert!(!poseidon.needs_range_check(31)); // 31 <= 31: SKIP
        assert!(!poseidon.needs_range_check(40)); // 31 <= 40: SKIP
    }

    #[test]
    fn test_bound_propagation_add() {
        let a = ValueBound::from_type_width(30);
        let b = ValueBound::from_type_width(30);
        let result = ValueBound::after_add(&a, &b);
        assert_eq!(result.max_bits, 31); // max + 1
    }

    #[test]
    fn test_bound_propagation_mul() {
        let a = ValueBound::from_type_width(16);
        let b = ValueBound::from_type_width(16);
        let result = ValueBound::after_mul(&a, &b);
        assert_eq!(result.max_bits, 32); // 16 + 16
    }

    #[test]
    fn test_bound_propagation_and() {
        let a = ValueBound::from_type_width(40);
        let b = ValueBound::from_type_width(16);
        let result = ValueBound::after_and(&a, &b);
        assert_eq!(result.max_bits, 16); // min
    }

    #[test]
    fn test_bound_propagation_shift() {
        let a = ValueBound::from_type_width(20);

        // Left shift
        let shl = ValueBound::after_shl(&a, 3, 40);
        assert_eq!(shl.max_bits, 23);

        // Right shift
        let shr = ValueBound::after_srl(&a, 3);
        assert_eq!(shr.max_bits, 17);
    }

    #[test]
    fn test_constant_bound() {
        let b = ValueBound::from_constant(255);
        assert_eq!(b.max_bits, 8);

        let b = ValueBound::from_constant(0x1000);
        assert_eq!(b.max_bits, 13);
    }


    #[test]
    fn test_comparison_bound() {
        let result = ValueBound::after_cmp();
        assert_eq!(result.max_bits, 1); // Boolean result
    }
}
