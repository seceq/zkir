//! Mersenne 31 field arithmetic for ZKIR v3.4
//!
//! p = 2^31 - 1 = 2,147,483,647
//!
//! Properties:
//! - 31-bit prime
//! - All limb values (up to 30 bits) are valid (< p)
//! - Supports configurable limb sizes: 16-30 bits (default: 20 bits)
//! - Efficient modular reduction

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// Mersenne 31 prime: p = 2^31 - 1
pub const MERSENNE31_PRIME: u32 = (1u32 << 31) - 1;

/// Mersenne 31 field element
///
/// Values are stored in canonical form: 0 ≤ value < p
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Mersenne31(u32);

impl Mersenne31 {
    pub const PRIME: u32 = MERSENNE31_PRIME;
    pub const ZERO: Self = Mersenne31(0);
    pub const ONE: Self = Mersenne31(1);

    /// Create a new field element from a u32 (reduces modulo p)
    #[inline]
    pub const fn new(value: u32) -> Self {
        Mersenne31(Self::reduce(value))
    }

    /// Create from a u32 that is already in canonical form (< p)
    ///
    /// # Safety
    /// Caller must ensure value < MERSENNE31_PRIME
    #[inline]
    pub const unsafe fn new_unchecked(value: u32) -> Self {
        debug_assert!(value < MERSENNE31_PRIME);
        Mersenne31(value)
    }

    /// Get the canonical value
    #[inline]
    pub const fn value(self) -> u32 {
        self.0
    }

    /// Reduce a u32 modulo p = 2^31 - 1
    ///
    /// For Mersenne primes: x mod (2^n - 1) = (x & (2^n - 1)) + (x >> n)
    /// This may need one more reduction if result ≥ p
    #[inline]
    const fn reduce(x: u32) -> u32 {
        let low = x & MERSENNE31_PRIME;
        let high = x >> 31;
        let sum = low + high;

        // If sum >= p, subtract p (which is equivalent to clearing bit 31)
        if sum >= MERSENNE31_PRIME {
            sum - MERSENNE31_PRIME
        } else {
            sum
        }
    }

    /// Reduce a u64 modulo p
    #[inline]
    const fn reduce64(x: u64) -> u32 {
        // Split into 31-bit chunks
        let low = (x as u32) & MERSENNE31_PRIME;
        let high = (x >> 31) as u32;

        // Sum and reduce again
        Self::reduce(low + high)
    }

    /// Compute additive inverse: -a mod p
    #[inline]
    pub const fn neg(self) -> Self {
        if self.0 == 0 {
            Self::ZERO
        } else {
            Mersenne31(MERSENNE31_PRIME - self.0)
        }
    }

    /// Compute multiplicative inverse: a^(-1) mod p
    ///
    /// Uses Fermat's little theorem: a^(p-1) = 1 mod p
    /// Therefore: a^(-1) = a^(p-2) mod p
    pub fn inv(self) -> Self {
        if self.0 == 0 {
            panic!("Division by zero in Mersenne31");
        }
        self.pow(MERSENNE31_PRIME - 2)
    }

    /// Compute self^exp mod p using binary exponentiation
    pub fn pow(self, mut exp: u32) -> Self {
        let mut base = self;
        let mut result = Self::ONE;

        while exp > 0 {
            if exp & 1 == 1 {
                result = result * base;
            }
            base = base * base;
            exp >>= 1;
        }

        result
    }

    /// Check if this is zero
    #[inline]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Check if this is one
    #[inline]
    pub const fn is_one(self) -> bool {
        self.0 == 1
    }
}

// Arithmetic implementations

impl Add for Mersenne31 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Mersenne31(Self::reduce(self.0 + rhs.0))
    }
}

impl AddAssign for Mersenne31 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Mersenne31 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        // Add p to avoid underflow, then reduce
        Mersenne31(Self::reduce(self.0 + MERSENNE31_PRIME - rhs.0))
    }
}

impl SubAssign for Mersenne31 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for Mersenne31 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Mersenne31(Self::reduce64((self.0 as u64) * (rhs.0 as u64)))
    }
}

impl MulAssign for Mersenne31 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Neg for Mersenne31 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self::neg(self)
    }
}

// Conversions

impl From<u32> for Mersenne31 {
    #[inline]
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl From<i32> for Mersenne31 {
    #[inline]
    fn from(value: i32) -> Self {
        if value >= 0 {
            Self::new(value as u32)
        } else {
            // Handle negative by computing (-value) mod p, then negate
            Self::new((-value) as u32).neg()
        }
    }
}

impl From<Mersenne31> for u32 {
    #[inline]
    fn from(f: Mersenne31) -> u32 {
        f.0
    }
}

// Display

impl fmt::Display for Mersenne31 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(MERSENNE31_PRIME, 2147483647);
        assert_eq!(Mersenne31::ZERO.value(), 0);
        assert_eq!(Mersenne31::ONE.value(), 1);
    }

    #[test]
    fn test_reduce() {
        assert_eq!(Mersenne31::new(0).value(), 0);
        assert_eq!(Mersenne31::new(1).value(), 1);
        assert_eq!(Mersenne31::new(MERSENNE31_PRIME).value(), 0);
        assert_eq!(Mersenne31::new(MERSENNE31_PRIME + 1).value(), 1);
        assert_eq!(Mersenne31::new(2 * MERSENNE31_PRIME).value(), 0);
    }

    #[test]
    fn test_addition() {
        let a = Mersenne31::new(100);
        let b = Mersenne31::new(200);
        assert_eq!((a + b).value(), 300);

        // Test overflow
        let c = Mersenne31::new(MERSENNE31_PRIME - 1);
        let d = Mersenne31::new(5);
        assert_eq!((c + d).value(), 4);
    }

    #[test]
    fn test_subtraction() {
        let a = Mersenne31::new(200);
        let b = Mersenne31::new(100);
        assert_eq!((a - b).value(), 100);

        // Test underflow
        let c = Mersenne31::new(5);
        let d = Mersenne31::new(10);
        assert_eq!((c - d).value(), MERSENNE31_PRIME - 5);
    }

    #[test]
    fn test_multiplication() {
        let a = Mersenne31::new(100);
        let b = Mersenne31::new(200);
        assert_eq!((a * b).value(), 20000);

        // Test with larger values
        let c = Mersenne31::new(1 << 20);
        let d = Mersenne31::new(1 << 20);
        let result = c * d;
        assert!(result.value() < MERSENNE31_PRIME);
    }

    #[test]
    fn test_negation() {
        let a = Mersenne31::new(100);
        let neg_a = -a;
        assert_eq!((a + neg_a).value(), 0);

        assert_eq!((-Mersenne31::ZERO).value(), 0);
        assert_eq!((-Mersenne31::ONE).value(), MERSENNE31_PRIME - 1);
    }

    #[test]
    fn test_inversion() {
        let a = Mersenne31::new(7);
        let inv_a = a.inv();
        assert_eq!((a * inv_a).value(), 1);

        let b = Mersenne31::new(12345);
        let inv_b = b.inv();
        assert_eq!((b * inv_b).value(), 1);
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_inverse_zero() {
        Mersenne31::ZERO.inv();
    }

    #[test]
    fn test_pow() {
        let a = Mersenne31::new(2);
        assert_eq!(a.pow(0).value(), 1);
        assert_eq!(a.pow(1).value(), 2);
        assert_eq!(a.pow(10).value(), 1024);

        // Fermat's little theorem: a^(p-1) = 1 mod p
        let b = Mersenne31::new(123);
        assert_eq!(b.pow(MERSENNE31_PRIME - 1).value(), 1);
    }
}
