//! # Value Types for ZKIR v3.4
//!
//! This module provides the abstraction for variable-width values.
//! Values are represented as multiple limbs, with configurable limb size.
//!
//! ## Generic Value Type
//!
//! The `GenericValue<const LIMB_BITS: u32, const NUM_LIMBS: usize>` type provides
//! a flexible representation for values with configurable limb sizes:
//!
//! ```ignore
//! // 40-bit value with 2 × 20-bit limbs (default)
//! type Value40 = GenericValue<20, 2>;
//!
//! // 60-bit value with 3 × 20-bit limbs
//! type Value60 = GenericValue<20, 3>;
//!
//! // 64-bit value with 2 × 32-bit limbs
//! type Value64 = GenericValue<32, 2>;
//! ```

use std::fmt;

/// Trait for value types with variable limb widths
pub trait Value: Copy + Clone + fmt::Debug + fmt::Display + Eq + PartialEq + Default + Sized {
    /// Number of limbs in this value type
    const NUM_LIMBS: usize;

    /// Limb size in bits
    const LIMB_BITS: u32;

    /// Total bits
    const TOTAL_BITS: u32 = Self::LIMB_BITS * Self::NUM_LIMBS as u32;

    /// Create a zero value
    fn zero() -> Self {
        Self::default()
    }

    /// Create a value from a u64
    fn from_u64(val: u64) -> Self;

    /// Convert to u64 (truncates if wider than 64 bits)
    fn to_u64(&self) -> u64;

    /// Create a value from a u32
    fn from_u32(val: u32) -> Self {
        Self::from_u64(val as u64)
    }

    /// Convert to u32 (truncates if wider than 32 bits)
    fn to_u32(&self) -> u32 {
        self.to_u64() as u32
    }

    /// Create a value from limbs
    fn from_limbs(limbs: &[u32]) -> Self;

    /// Get limbs as a slice
    fn limbs(&self) -> &[u32];

    /// Get mutable limbs
    fn limbs_mut(&mut self) -> &mut [u32];

    /// Wrapping addition
    fn wrapping_add(self, rhs: Self) -> Self;

    /// Wrapping subtraction
    fn wrapping_sub(self, rhs: Self) -> Self;

    /// Wrapping multiplication
    fn wrapping_mul(self, rhs: Self) -> Self;

    /// Bitwise AND
    fn bitwise_and(self, rhs: Self) -> Self;

    /// Bitwise OR
    fn bitwise_or(self, rhs: Self) -> Self;

    /// Bitwise XOR
    fn bitwise_xor(self, rhs: Self) -> Self;

    /// Bitwise NOT
    fn bitwise_not(self) -> Self;

    /// Left shift
    fn left_shift(self, shift: u32) -> Self;

    /// Logical right shift
    fn right_shift(self, shift: u32) -> Self;

    /// Arithmetic right shift (sign-extending)
    fn arithmetic_right_shift(self, shift: u32, data_bits: u32) -> Self;

    /// Unsigned less than
    fn unsigned_lt(self, rhs: Self) -> bool;

    /// Unsigned less than or equal
    fn unsigned_le(self, rhs: Self) -> bool;

    /// Signed less than (with specified bit width for sign)
    fn signed_lt(self, rhs: Self, data_bits: u32) -> bool;

    /// Check equality
    fn equals(self, rhs: Self) -> bool;

    /// Get the sign bit at the specified position
    fn sign_bit(&self, data_bits: u32) -> bool;

    /// Sign-extend from specified bit width
    fn sign_extend(&self, from_bits: u32, to_bits: u32) -> Self;

    /// Zero-extend from specified bit width
    fn zero_extend(&self, from_bits: u32) -> Self;

    /// Truncate to specified bit width
    fn truncate(&self, to_bits: u32) -> Self;

    /// Check if value is zero
    fn is_zero(&self) -> bool;

    /// Check if all limbs fit in specified bit width
    fn fits_in(&self, bits: u32) -> bool;
}

// ============================================================================
// Generic Value Type with Const Generics
// ============================================================================

/// Generic value type with configurable limb size and count.
///
/// # Type Parameters
/// - `LIMB_BITS`: Number of bits per limb (typically 16-30)
/// - `NUM_LIMBS`: Number of limbs (typically 2-4)
///
/// # Examples
/// ```ignore
/// // 40-bit value: 2 limbs × 20 bits = 40 bits
/// type Value40 = GenericValue<20, 2>;
///
/// // 60-bit value: 3 limbs × 20 bits = 60 bits
/// type Value60 = GenericValue<20, 3>;
/// ```
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct GenericValue<const LIMB_BITS: u32, const NUM_LIMBS: usize> {
    /// Limbs stored in u32 (only lower LIMB_BITS are used)
    limbs: [u32; NUM_LIMBS],
}

impl<const LIMB_BITS: u32, const NUM_LIMBS: usize> Default for GenericValue<LIMB_BITS, NUM_LIMBS> {
    fn default() -> Self {
        Self {
            limbs: [0; NUM_LIMBS],
        }
    }
}

impl<const LIMB_BITS: u32, const NUM_LIMBS: usize> fmt::Debug for GenericValue<LIMB_BITS, NUM_LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GenericValue<{}, {}>(", LIMB_BITS, NUM_LIMBS)?;
        for (i, limb) in self.limbs.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{:#x}", limb)?;
        }
        write!(f, ")")
    }
}

impl<const LIMB_BITS: u32, const NUM_LIMBS: usize> fmt::Display for GenericValue<LIMB_BITS, NUM_LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.to_u64())
    }
}

impl<const LIMB_BITS: u32, const NUM_LIMBS: usize> GenericValue<LIMB_BITS, NUM_LIMBS> {
    /// Total bits in this value type
    pub const TOTAL_BITS: u32 = LIMB_BITS * NUM_LIMBS as u32;

    /// Limb mask (2^LIMB_BITS - 1)
    /// Note: Uses u64 shift to avoid overflow when LIMB_BITS == 32
    pub const LIMB_MASK: u32 = ((1u64 << LIMB_BITS) - 1) as u32;

    /// Create a new value from limbs (unchecked - assumes limbs are already masked)
    #[inline]
    pub const fn new_unchecked(limbs: [u32; NUM_LIMBS]) -> Self {
        Self { limbs }
    }

    /// Create a new value from limbs (masked to LIMB_BITS)
    #[inline]
    pub fn new(limbs: [u32; NUM_LIMBS]) -> Self {
        let mut result = Self { limbs };
        for limb in result.limbs.iter_mut() {
            *limb &= Self::LIMB_MASK;
        }
        result
    }

    /// Convert to u64 (truncates if wider than 64 bits)
    #[inline]
    pub fn to_u64(&self) -> u64 {
        let mut result = 0u64;
        let mut shift = 0u32;
        for &limb in &self.limbs {
            if shift >= 64 {
                break;
            }
            result |= (limb as u64) << shift;
            shift += LIMB_BITS;
        }
        result
    }

    /// Convert to u128 for wider values
    #[inline]
    pub fn to_u128(&self) -> u128 {
        let mut result = 0u128;
        let mut shift = 0u32;
        for &limb in &self.limbs {
            if shift >= 128 {
                break;
            }
            result |= (limb as u128) << shift;
            shift += LIMB_BITS;
        }
        result
    }

    /// Create from u64
    #[inline]
    pub fn from_u64(val: u64) -> Self {
        let mut limbs = [0u32; NUM_LIMBS];
        let mut remaining = val;
        for limb in limbs.iter_mut() {
            *limb = (remaining & Self::LIMB_MASK as u64) as u32;
            remaining >>= LIMB_BITS;
        }
        Self { limbs }
    }

    /// Create from u128
    #[inline]
    pub fn from_u128(val: u128) -> Self {
        let mut limbs = [0u32; NUM_LIMBS];
        let mut remaining = val;
        for limb in limbs.iter_mut() {
            *limb = (remaining & Self::LIMB_MASK as u128) as u32;
            remaining >>= LIMB_BITS;
        }
        Self { limbs }
    }

    /// Check if value is zero
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.limbs.iter().all(|&l| l == 0)
    }

    /// Get the maximum value for this type
    pub fn max_value() -> Self {
        Self {
            limbs: [Self::LIMB_MASK; NUM_LIMBS],
        }
    }
}

impl<const LIMB_BITS: u32, const NUM_LIMBS: usize> Value for GenericValue<LIMB_BITS, NUM_LIMBS> {
    const NUM_LIMBS: usize = NUM_LIMBS;
    const LIMB_BITS: u32 = LIMB_BITS;

    #[inline]
    fn from_u64(val: u64) -> Self {
        Self::from_u64(val)
    }

    #[inline]
    fn to_u64(&self) -> u64 {
        Self::to_u64(self)
    }

    #[inline]
    fn from_limbs(limbs: &[u32]) -> Self {
        assert!(limbs.len() >= NUM_LIMBS, "Need at least {} limbs", NUM_LIMBS);
        let mut arr = [0u32; NUM_LIMBS];
        for (i, &limb) in limbs.iter().take(NUM_LIMBS).enumerate() {
            arr[i] = limb & Self::LIMB_MASK;
        }
        Self { limbs: arr }
    }

    #[inline]
    fn limbs(&self) -> &[u32] {
        &self.limbs
    }

    #[inline]
    fn limbs_mut(&mut self) -> &mut [u32] {
        &mut self.limbs
    }

    #[inline]
    fn wrapping_add(self, rhs: Self) -> Self {
        // Use u128 for addition to handle overflow across limbs
        let a = self.to_u128();
        let b = rhs.to_u128();
        let mask = (1u128 << Self::TOTAL_BITS) - 1;
        Self::from_u128((a + b) & mask)
    }

    #[inline]
    fn wrapping_sub(self, rhs: Self) -> Self {
        let a = self.to_u128();
        let b = rhs.to_u128();
        let mask = (1u128 << Self::TOTAL_BITS) - 1;
        Self::from_u128(a.wrapping_sub(b) & mask)
    }

    #[inline]
    fn wrapping_mul(self, rhs: Self) -> Self {
        let a = self.to_u128();
        let b = rhs.to_u128();
        let mask = (1u128 << Self::TOTAL_BITS) - 1;
        Self::from_u128((a * b) & mask)
    }

    #[inline]
    fn bitwise_and(self, rhs: Self) -> Self {
        let mut result = self;
        for (a, b) in result.limbs.iter_mut().zip(rhs.limbs.iter()) {
            *a &= *b;
        }
        result
    }

    #[inline]
    fn bitwise_or(self, rhs: Self) -> Self {
        let mut result = self;
        for (a, b) in result.limbs.iter_mut().zip(rhs.limbs.iter()) {
            *a |= *b;
        }
        result
    }

    #[inline]
    fn bitwise_xor(self, rhs: Self) -> Self {
        let mut result = self;
        for (a, b) in result.limbs.iter_mut().zip(rhs.limbs.iter()) {
            *a ^= *b;
        }
        result
    }

    #[inline]
    fn bitwise_not(self) -> Self {
        let mut result = self;
        for limb in result.limbs.iter_mut() {
            *limb = !*limb & Self::LIMB_MASK;
        }
        result
    }

    #[inline]
    fn left_shift(self, shift: u32) -> Self {
        if shift >= Self::TOTAL_BITS {
            return Self::default();
        }
        let val = self.to_u128() << shift;
        let mask = (1u128 << Self::TOTAL_BITS) - 1;
        Self::from_u128(val & mask)
    }

    #[inline]
    fn right_shift(self, shift: u32) -> Self {
        if shift >= Self::TOTAL_BITS {
            return Self::default();
        }
        Self::from_u128(self.to_u128() >> shift)
    }

    #[inline]
    fn arithmetic_right_shift(self, shift: u32, data_bits: u32) -> Self {
        let val = self.to_u128();
        let sign_bit = 1u128 << (data_bits - 1);
        let is_negative = (val & sign_bit) != 0;

        if shift >= data_bits {
            return if is_negative {
                Self::from_u128((1u128 << data_bits) - 1)
            } else {
                Self::default()
            };
        }

        let shifted = val >> shift;
        if is_negative {
            let mask = ((1u128 << shift) - 1) << (data_bits - shift);
            Self::from_u128(shifted | mask)
        } else {
            Self::from_u128(shifted)
        }
    }

    #[inline]
    fn unsigned_lt(self, rhs: Self) -> bool {
        self.to_u128() < rhs.to_u128()
    }

    #[inline]
    fn unsigned_le(self, rhs: Self) -> bool {
        self.to_u128() <= rhs.to_u128()
    }

    #[inline]
    fn signed_lt(self, rhs: Self, data_bits: u32) -> bool {
        let sign_bit = 1u128 << (data_bits - 1);
        let a = self.to_u128() ^ sign_bit;
        let b = rhs.to_u128() ^ sign_bit;
        a < b
    }

    #[inline]
    fn equals(self, rhs: Self) -> bool {
        self == rhs
    }

    #[inline]
    fn sign_bit(&self, data_bits: u32) -> bool {
        let sign_bit_pos = data_bits - 1;
        let val = self.to_u128();
        (val & (1u128 << sign_bit_pos)) != 0
    }

    #[inline]
    fn sign_extend(&self, from_bits: u32, to_bits: u32) -> Self {
        let val = self.to_u128();
        let sign_bit = 1u128 << (from_bits - 1);
        let is_negative = (val & sign_bit) != 0;

        if is_negative {
            let mask = ((1u128 << to_bits) - 1) ^ ((1u128 << from_bits) - 1);
            Self::from_u128(val | mask)
        } else {
            *self
        }
    }

    #[inline]
    fn zero_extend(&self, from_bits: u32) -> Self {
        let mask = (1u128 << from_bits) - 1;
        Self::from_u128(self.to_u128() & mask)
    }

    #[inline]
    fn truncate(&self, to_bits: u32) -> Self {
        let mask = (1u128 << to_bits) - 1;
        Self::from_u128(self.to_u128() & mask)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        Self::is_zero(self)
    }

    #[inline]
    fn fits_in(&self, bits: u32) -> bool {
        if bits >= Self::TOTAL_BITS {
            return true;
        }
        let max_val = (1u128 << bits) - 1;
        self.to_u128() <= max_val
    }
}

impl<const LIMB_BITS: u32, const NUM_LIMBS: usize> From<u64> for GenericValue<LIMB_BITS, NUM_LIMBS> {
    fn from(val: u64) -> Self {
        Self::from_u64(val)
    }
}

impl<const LIMB_BITS: u32, const NUM_LIMBS: usize> From<u32> for GenericValue<LIMB_BITS, NUM_LIMBS> {
    fn from(val: u32) -> Self {
        Self::from_u64(val as u64)
    }
}

impl<const LIMB_BITS: u32, const NUM_LIMBS: usize> From<GenericValue<LIMB_BITS, NUM_LIMBS>> for u64 {
    fn from(val: GenericValue<LIMB_BITS, NUM_LIMBS>) -> u64 {
        val.to_u64()
    }
}

// ============================================================================
// Common Type Aliases
// ============================================================================

/// 40-bit value with 2 × 20-bit limbs (default configuration)
pub type Value40Generic = GenericValue<20, 2>;

/// 60-bit value with 3 × 20-bit limbs
pub type Value60 = GenericValue<20, 3>;

/// 80-bit value with 4 × 20-bit limbs
pub type Value80 = GenericValue<20, 4>;

/// 30-bit value with 2 × 15-bit limbs (compact)
pub type Value30 = GenericValue<15, 2>;

/// 64-bit value with 2 × 32-bit limbs
pub type Value64 = GenericValue<32, 2>;

// ============================================================================
// Legacy Value40 Type (for backwards compatibility)
// ============================================================================

/// Default 40-bit value type (2 × 20-bit limbs)
///
/// This is the original implementation kept for backwards compatibility.
/// New code should prefer `GenericValue<20, 2>` or the `Value40Generic` alias.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Value40 {
    /// Two 20-bit limbs stored in u32 (low 20 bits used)
    limbs: [u32; 2],
}

impl Value40 {
    /// Limb size in bits
    pub const LIMB_BITS: u32 = 20;

    /// Number of limbs
    pub const NUM_LIMBS: usize = 2;

    /// Total bits
    pub const TOTAL_BITS: u32 = Self::LIMB_BITS * Self::NUM_LIMBS as u32;

    /// Limb mask (0xFFFFF for 20 bits)
    pub const LIMB_MASK: u32 = (1 << Self::LIMB_BITS) - 1;

    /// Create a new value from limbs (unchecked)
    #[inline]
    pub const fn new_unchecked(limbs: [u32; 2]) -> Self {
        Self { limbs }
    }

    /// Create a new value from limbs (masked)
    #[inline]
    pub const fn new(limbs: [u32; 2]) -> Self {
        Self {
            limbs: [limbs[0] & Self::LIMB_MASK, limbs[1] & Self::LIMB_MASK],
        }
    }

    /// Normalize limbs (apply mask)
    #[inline]
    #[allow(dead_code)]
    fn normalize(&mut self) {
        self.limbs[0] &= Self::LIMB_MASK;
        self.limbs[1] &= Self::LIMB_MASK;
    }

    /// Create from i32 (sign-extended)
    pub fn from_i32(val: i32) -> Self {
        let val_u64 = if val < 0 {
            // Sign-extend: fill upper bits with 1s up to 40 bits
            let abs = val.unsigned_abs() as u64;
            let neg = (1u64 << 32) - abs;
            neg | (0xFFu64 << 32) // Sign-extend to 40 bits
        } else {
            val as u64
        };
        Self::from_u64(val_u64)
    }

    /// Convert to i32 (sign-extended from bit 31)
    pub fn to_i32(&self) -> i32 {
        let val = self.to_u64() as u32;
        val as i32
    }

    /// Maximum unsigned value (all bits set)
    pub const fn max_value() -> Self {
        Self::new_unchecked([Self::LIMB_MASK, Self::LIMB_MASK])
    }
}

impl Value for Value40 {
    const NUM_LIMBS: usize = 2;
    const LIMB_BITS: u32 = 20;

    #[inline]
    fn from_u64(val: u64) -> Self {
        let limb0 = (val & Self::LIMB_MASK as u64) as u32;
        let limb1 = ((val >> Self::LIMB_BITS) & Self::LIMB_MASK as u64) as u32;
        Self::new_unchecked([limb0, limb1])
    }

    #[inline]
    fn to_u64(&self) -> u64 {
        (self.limbs[0] as u64) | ((self.limbs[1] as u64) << Self::LIMB_BITS)
    }

    #[inline]
    fn from_limbs(limbs: &[u32]) -> Self {
        assert!(limbs.len() >= 2, "Need at least 2 limbs");
        Self::new([limbs[0], limbs[1]])
    }

    #[inline]
    fn limbs(&self) -> &[u32] {
        &self.limbs
    }

    #[inline]
    fn limbs_mut(&mut self) -> &mut [u32] {
        &mut self.limbs
    }

    #[inline]
    fn wrapping_add(self, rhs: Self) -> Self {
        let val = self.to_u64().wrapping_add(rhs.to_u64());
        Self::from_u64(val)
    }

    #[inline]
    fn wrapping_sub(self, rhs: Self) -> Self {
        let val = self.to_u64().wrapping_sub(rhs.to_u64());
        Self::from_u64(val)
    }

    #[inline]
    fn wrapping_mul(self, rhs: Self) -> Self {
        let val = self.to_u64().wrapping_mul(rhs.to_u64());
        Self::from_u64(val)
    }

    #[inline]
    fn bitwise_and(self, rhs: Self) -> Self {
        Self::new_unchecked([self.limbs[0] & rhs.limbs[0], self.limbs[1] & rhs.limbs[1]])
    }

    #[inline]
    fn bitwise_or(self, rhs: Self) -> Self {
        Self::new_unchecked([self.limbs[0] | rhs.limbs[0], self.limbs[1] | rhs.limbs[1]])
    }

    #[inline]
    fn bitwise_xor(self, rhs: Self) -> Self {
        Self::new_unchecked([self.limbs[0] ^ rhs.limbs[0], self.limbs[1] ^ rhs.limbs[1]])
    }

    #[inline]
    fn bitwise_not(self) -> Self {
        Self::new([!self.limbs[0], !self.limbs[1]])
    }

    #[inline]
    fn left_shift(self, shift: u32) -> Self {
        if shift >= Self::TOTAL_BITS {
            return Self::zero();
        }
        let val = self.to_u64() << shift;
        Self::from_u64(val)
    }

    #[inline]
    fn right_shift(self, shift: u32) -> Self {
        if shift >= Self::TOTAL_BITS {
            return Self::zero();
        }
        let val = self.to_u64() >> shift;
        Self::from_u64(val)
    }

    #[inline]
    fn arithmetic_right_shift(self, shift: u32, data_bits: u32) -> Self {
        let val = self.to_u64();
        let sign_bit = 1u64 << (data_bits - 1);
        let is_negative = (val & sign_bit) != 0;

        if shift >= data_bits {
            return if is_negative {
                Self::from_u64((1u64 << data_bits) - 1)
            } else {
                Self::zero()
            };
        }

        let shifted = val >> shift;
        if is_negative {
            // Fill with ones from the left
            let mask = ((1u64 << shift) - 1) << (data_bits - shift);
            Self::from_u64(shifted | mask)
        } else {
            Self::from_u64(shifted)
        }
    }

    #[inline]
    fn unsigned_lt(self, rhs: Self) -> bool {
        self.to_u64() < rhs.to_u64()
    }

    #[inline]
    fn unsigned_le(self, rhs: Self) -> bool {
        self.to_u64() <= rhs.to_u64()
    }

    #[inline]
    fn signed_lt(self, rhs: Self, data_bits: u32) -> bool {
        let sign_bit = 1u64 << (data_bits - 1);
        // XOR trick: flip sign bit to convert signed to unsigned comparison
        let a = self.to_u64() ^ sign_bit;
        let b = rhs.to_u64() ^ sign_bit;
        a < b
    }

    #[inline]
    fn equals(self, rhs: Self) -> bool {
        self == rhs
    }

    #[inline]
    fn sign_bit(&self, data_bits: u32) -> bool {
        let sign_bit_pos = data_bits - 1;
        let val = self.to_u64();
        (val & (1u64 << sign_bit_pos)) != 0
    }

    #[inline]
    fn sign_extend(&self, from_bits: u32, to_bits: u32) -> Self {
        let val = self.to_u64();
        let sign_bit = 1u64 << (from_bits - 1);
        let is_negative = (val & sign_bit) != 0;

        if is_negative {
            // Extend with ones
            let mask = ((1u64 << to_bits) - 1) ^ ((1u64 << from_bits) - 1);
            Self::from_u64(val | mask)
        } else {
            // Already zero-extended
            *self
        }
    }

    #[inline]
    fn zero_extend(&self, from_bits: u32) -> Self {
        let mask = (1u64 << from_bits) - 1;
        Self::from_u64(self.to_u64() & mask)
    }

    #[inline]
    fn truncate(&self, to_bits: u32) -> Self {
        let mask = (1u64 << to_bits) - 1;
        Self::from_u64(self.to_u64() & mask)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.limbs[0] == 0 && self.limbs[1] == 0
    }

    #[inline]
    fn fits_in(&self, bits: u32) -> bool {
        if bits >= Self::TOTAL_BITS {
            return true;
        }
        let max_val = (1u64 << bits) - 1;
        self.to_u64() <= max_val
    }
}

impl fmt::Display for Value40 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.to_u64())
    }
}

impl From<u64> for Value40 {
    #[inline]
    fn from(val: u64) -> Self {
        Self::from_u64(val)
    }
}

impl From<u32> for Value40 {
    #[inline]
    fn from(val: u32) -> Self {
        Self::from_u32(val)
    }
}

impl From<Value40> for u64 {
    #[inline]
    fn from(val: Value40) -> u64 {
        val.to_u64()
    }
}

impl From<Value40> for u32 {
    #[inline]
    fn from(val: Value40) -> u32 {
        val.to_u32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value40_basic() {
        let v = Value40::zero();
        assert_eq!(v.to_u64(), 0);

        let v = Value40::from_u64(0x123456789);
        assert_eq!(v.to_u64(), 0x123456789);

        let v = Value40::from_u32(0xDEADBEEF);
        assert_eq!(v.to_u32(), 0xDEADBEEF);
    }

    #[test]
    fn test_value40_arithmetic() {
        let a = Value40::from_u64(100);
        let b = Value40::from_u64(50);

        assert_eq!(a.wrapping_add(b).to_u64(), 150);
        assert_eq!(a.wrapping_sub(b).to_u64(), 50);
        assert_eq!(a.wrapping_mul(b).to_u64(), 5000);
    }

    #[test]
    fn test_value40_bitwise() {
        let a = Value40::from_u64(0b1100);
        let b = Value40::from_u64(0b1010);

        assert_eq!(a.bitwise_and(b).to_u64(), 0b1000);
        assert_eq!(a.bitwise_or(b).to_u64(), 0b1110);
        assert_eq!(a.bitwise_xor(b).to_u64(), 0b0110);
    }

    #[test]
    fn test_value40_shifts() {
        let v = Value40::from_u64(0b1100);

        assert_eq!(v.left_shift(2).to_u64(), 0b110000);
        assert_eq!(v.right_shift(1).to_u64(), 0b110);
    }

    #[test]
    fn test_value40_comparison() {
        let a = Value40::from_u64(100);
        let b = Value40::from_u64(50);

        assert!(b.unsigned_lt(a));
        assert!(!a.unsigned_lt(b));
        assert!(a.equals(a));
        assert!(!a.equals(b));
    }

    #[test]
    fn test_value40_sign_extension() {
        // Positive number
        let v = Value40::from_u32(0x7FFF);
        let ext = v.sign_extend(16, 32);
        assert_eq!(ext.to_u32(), 0x7FFF);

        // Negative number (bit 15 set)
        let v = Value40::from_u32(0x8000);
        let ext = v.sign_extend(16, 32);
        assert_eq!(ext.to_u32(), 0xFFFF8000);
    }

    #[test]
    fn test_value40_truncate() {
        let v = Value40::from_u64(0xDEADBEEF);
        assert_eq!(v.truncate(16).to_u64(), 0xBEEF);
        assert_eq!(v.truncate(8).to_u64(), 0xEF);
    }

    #[test]
    fn test_value40_limbs() {
        // Use a value that spans both limbs (> 20 bits)
        let v = Value40::from_u64(0x12ABCDE); // 24-bit value
        let limbs = v.limbs();
        assert_eq!(limbs.len(), 2);
        // With 20-bit limbs:
        // limb0 = lower 20 bits = 0x12ABCDE & 0xFFFFF = 0xABCDE
        // limb1 = upper 20 bits = (0x12ABCDE >> 20) & 0xFFFFF = 0x12
        assert_eq!(limbs[0], 0xABCDE); // Lower 20 bits
        assert_eq!(limbs[1], 0x12);    // Upper 20 bits
    }

    // ========================================================================
    // GenericValue Tests
    // ========================================================================

    #[test]
    fn test_generic_value_basic() {
        // Test Value40Generic (2 × 20-bit limbs)
        let v: Value40Generic = GenericValue::from_u64(0x123456789);
        assert_eq!(v.to_u64(), 0x123456789);
        assert_eq!(Value40Generic::TOTAL_BITS, 40);
        assert_eq!(Value40Generic::LIMB_MASK, 0xFFFFF);

        // Test zero
        let z = Value40Generic::default();
        assert!(z.is_zero());
        assert_eq!(z.to_u64(), 0);
    }

    #[test]
    fn test_generic_value_60bit() {
        // Test Value60 (3 × 20-bit limbs = 60 bits)
        let v: Value60 = GenericValue::from_u64(0x0FFF_FFFF_FFFF_FFFF);
        assert_eq!(v.to_u64(), 0x0FFF_FFFF_FFFF_FFFF);
        assert_eq!(Value60::TOTAL_BITS, 60);

        // Test u128 for values > 64 bits
        let large: Value60 = GenericValue::from_u128(0x0FFF_FFFF_FFFF_FFFF_u128);
        assert_eq!(large.to_u128(), 0x0FFF_FFFF_FFFF_FFFF);
    }

    #[test]
    fn test_generic_value_80bit() {
        // Test Value80 (4 × 20-bit limbs = 80 bits)
        assert_eq!(Value80::TOTAL_BITS, 80);

        // Test that it can hold values larger than u64
        let max_u64: Value80 = GenericValue::from_u64(u64::MAX);
        assert_eq!(max_u64.to_u64(), u64::MAX);

        // Test u128 values
        let v: Value80 = GenericValue::from_u128(0x1_0000_0000_0000_0000_u128);
        assert_eq!(v.to_u128(), 0x1_0000_0000_0000_0000_u128);
    }

    #[test]
    fn test_generic_value_64bit() {
        // Test Value64 (2 × 32-bit limbs)
        assert_eq!(Value64::TOTAL_BITS, 64);
        assert_eq!(Value64::LIMB_MASK, 0xFFFF_FFFF);

        let v: Value64 = GenericValue::from_u64(0xDEAD_BEEF_CAFE_BABE);
        assert_eq!(v.to_u64(), 0xDEAD_BEEF_CAFE_BABE);

        // Check limbs
        assert_eq!(v.limbs()[0], 0xCAFE_BABE);
        assert_eq!(v.limbs()[1], 0xDEAD_BEEF);
    }

    #[test]
    fn test_generic_value_arithmetic() {
        let a: Value60 = GenericValue::from_u64(0x1_0000_0000);
        let b: Value60 = GenericValue::from_u64(0x2_0000_0000);

        // Addition
        assert_eq!(a.wrapping_add(b).to_u64(), 0x3_0000_0000);

        // Subtraction
        assert_eq!(b.wrapping_sub(a).to_u64(), 0x1_0000_0000);

        // Multiplication
        let c: Value60 = GenericValue::from_u64(1000);
        let d: Value60 = GenericValue::from_u64(2000);
        assert_eq!(c.wrapping_mul(d).to_u64(), 2_000_000);
    }

    #[test]
    fn test_generic_value_wrapping() {
        // Test wrapping on 40-bit boundary
        let max: Value40Generic = GenericValue::max_value();
        let one: Value40Generic = GenericValue::from_u64(1);

        // max + 1 should wrap to 0
        let wrapped = max.wrapping_add(one);
        assert!(wrapped.is_zero());

        // 0 - 1 should wrap to max
        let zero: Value40Generic = GenericValue::default();
        let wrapped = zero.wrapping_sub(one);
        assert_eq!(wrapped.to_u64(), max.to_u64());
    }

    #[test]
    fn test_generic_value_bitwise() {
        let a: Value60 = GenericValue::from_u64(0xFF00_FF00);
        let b: Value60 = GenericValue::from_u64(0xF0F0_F0F0);

        assert_eq!(a.bitwise_and(b).to_u64(), 0xF000_F000);
        assert_eq!(a.bitwise_or(b).to_u64(), 0xFFF0_FFF0);
        assert_eq!(a.bitwise_xor(b).to_u64(), 0x0FF0_0FF0);

        // NOT (should mask to 60 bits)
        let c: Value60 = GenericValue::from_u64(0);
        let not_c = c.bitwise_not();
        assert_eq!(not_c.to_u128(), (1u128 << 60) - 1);
    }

    #[test]
    fn test_generic_value_shifts() {
        let v: Value60 = GenericValue::from_u64(0x1234);

        // Left shift
        assert_eq!(v.left_shift(4).to_u64(), 0x12340);
        assert_eq!(v.left_shift(32).to_u64(), 0x1234_0000_0000);

        // Right shift
        assert_eq!(v.right_shift(4).to_u64(), 0x123);
        assert_eq!(v.right_shift(12).to_u64(), 0x1);
    }

    #[test]
    fn test_generic_value_comparisons() {
        let a: Value60 = GenericValue::from_u64(100);
        let b: Value60 = GenericValue::from_u64(200);

        assert!(a.unsigned_lt(b));
        assert!(a.unsigned_le(b));
        assert!(!b.unsigned_lt(a));
        assert!(a.unsigned_le(a));
        assert!(a.equals(a));
        assert!(!a.equals(b));
    }

    #[test]
    fn test_generic_value_signed_comparison() {
        // With 32-bit signed interpretation
        let pos: Value40Generic = GenericValue::from_u64(100);
        let neg: Value40Generic = GenericValue::from_u64(0xFFFF_FFFF); // -1 in 32-bit

        // Unsigned: neg > pos
        assert!(pos.unsigned_lt(neg));

        // Signed (32-bit): pos > neg
        assert!(neg.signed_lt(pos, 32));
        assert!(!pos.signed_lt(neg, 32));
    }

    #[test]
    fn test_generic_value_sign_extend() {
        // 16-bit negative extended to 40 bits
        let v: Value40Generic = GenericValue::from_u64(0x8000); // -32768 in 16-bit
        let ext = v.sign_extend(16, 40);
        // Should have bits 16-39 set to 1
        assert_eq!(ext.to_u64() & 0xFF_FFFF_0000, 0xFF_FFFF_0000);

        // Positive number should not change
        let pos: Value40Generic = GenericValue::from_u64(0x7FFF);
        let ext_pos = pos.sign_extend(16, 40);
        assert_eq!(ext_pos.to_u64(), 0x7FFF);
    }

    #[test]
    fn test_generic_value_truncate() {
        let v: Value60 = GenericValue::from_u64(0xDEAD_BEEF_CAFE);

        assert_eq!(v.truncate(32).to_u64(), 0xBEEF_CAFE);
        assert_eq!(v.truncate(16).to_u64(), 0xCAFE);
        assert_eq!(v.truncate(8).to_u64(), 0xFE);
    }

    #[test]
    fn test_generic_value_fits_in() {
        let v: Value40Generic = GenericValue::from_u64(0xFF);

        assert!(v.fits_in(8));
        assert!(v.fits_in(16));
        assert!(v.fits_in(40));

        let large: Value40Generic = GenericValue::from_u64(0x1_0000_0000);
        assert!(!large.fits_in(32));
        assert!(large.fits_in(33));
        assert!(large.fits_in(40));
    }

    #[test]
    fn test_generic_value_from_limbs() {
        let limbs = [0x12345, 0x67890, 0xABCDE];
        let v: Value60 = GenericValue::from_limbs(&limbs);

        assert_eq!(v.limbs()[0], 0x12345);
        assert_eq!(v.limbs()[1], 0x67890);
        assert_eq!(v.limbs()[2], 0xABCDE);
    }

    #[test]
    fn test_generic_value_max_value() {
        let max40: Value40Generic = GenericValue::max_value();
        assert_eq!(max40.to_u64(), (1u64 << 40) - 1);

        let max60: Value60 = GenericValue::max_value();
        assert_eq!(max60.to_u128(), (1u128 << 60) - 1);

        let max64: Value64 = GenericValue::max_value();
        assert_eq!(max64.to_u64(), u64::MAX);
    }

    #[test]
    fn test_value40_and_generic_equivalence() {
        // Test that Value40 and Value40Generic produce same results
        let val = 0x12345_ABCDE_u64;

        let v40 = Value40::from_u64(val);
        let v40g: Value40Generic = GenericValue::from_u64(val);

        assert_eq!(v40.to_u64(), v40g.to_u64());
        assert_eq!(v40.limbs()[0], v40g.limbs()[0]);
        assert_eq!(v40.limbs()[1], v40g.limbs()[1]);

        // Test arithmetic equivalence
        let a40 = Value40::from_u64(1000);
        let b40 = Value40::from_u64(500);
        let a40g: Value40Generic = GenericValue::from_u64(1000);
        let b40g: Value40Generic = GenericValue::from_u64(500);

        assert_eq!(a40.wrapping_add(b40).to_u64(), a40g.wrapping_add(b40g).to_u64());
        assert_eq!(a40.wrapping_sub(b40).to_u64(), a40g.wrapping_sub(b40g).to_u64());
        assert_eq!(a40.wrapping_mul(b40).to_u64(), a40g.wrapping_mul(b40g).to_u64());
    }
}
