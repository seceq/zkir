//! Field element type for ZK IR.

use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign, SubAssign, MulAssign, DivAssign};
use std::cmp::Ordering;

/// BabyBear prime: 2^31 - 2^27 + 1
pub const BABYBEAR_PRIME: u64 = 2013265921;

/// A field element in the BabyBear prime field.
///
/// BabyBear is a 31-bit prime field with p = 2^31 - 2^27 + 1 = 2013265921.
/// It's commonly used in STARK proofs due to efficient arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct FieldElement(u64);

impl FieldElement {
    /// The prime modulus
    pub const MODULUS: u64 = BABYBEAR_PRIME;

    /// Create a new field element from a value (will be reduced mod p)
    pub fn new(value: u64) -> Self {
        FieldElement(value % Self::MODULUS)
    }

    /// Create the zero element
    pub fn zero() -> Self {
        FieldElement(0)
    }

    /// Create the one element
    pub fn one() -> Self {
        FieldElement(1)
    }

    /// Get the inner value
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Check if this is zero
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Compute multiplicative inverse using Fermat's little theorem
    /// a^(-1) = a^(p-2) mod p
    pub fn inverse(&self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }
        Some(self.pow(Self::MODULUS - 2))
    }

    /// Compute self^exp mod p using binary exponentiation
    pub fn pow(&self, mut exp: u64) -> Self {
        let mut base = *self;
        let mut result = FieldElement::one();

        while exp > 0 {
            if exp & 1 == 1 {
                result = result * base;
            }
            base = base * base;
            exp >>= 1;
        }

        result
    }

    /// Convert to bytes (little-endian)
    pub fn to_bytes(&self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    /// Create from bytes (little-endian)
    pub fn from_bytes(bytes: [u8; 8]) -> Self {
        FieldElement::new(u64::from_le_bytes(bytes))
    }

    /// Convert to i64 (for signed interpretation)
    pub fn to_signed(&self) -> i64 {
        let half = Self::MODULUS / 2;
        if self.0 > half {
            self.0 as i64 - Self::MODULUS as i64
        } else {
            self.0 as i64
        }
    }

    /// Create from signed value
    pub fn from_signed(value: i64) -> Self {
        if value >= 0 {
            FieldElement::new(value as u64)
        } else {
            FieldElement::new((Self::MODULUS as i64 + value) as u64)
        }
    }
}

impl From<u64> for FieldElement {
    fn from(value: u64) -> Self {
        FieldElement::new(value)
    }
}

impl From<u32> for FieldElement {
    fn from(value: u32) -> Self {
        FieldElement::new(value as u64)
    }
}

impl From<i32> for FieldElement {
    fn from(value: i32) -> Self {
        FieldElement::from_signed(value as i64)
    }
}

impl From<i64> for FieldElement {
    fn from(value: i64) -> Self {
        FieldElement::from_signed(value)
    }
}

impl Add for FieldElement {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        FieldElement::new(self.0 + rhs.0)
    }
}

impl Sub for FieldElement {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        if self.0 >= rhs.0 {
            FieldElement(self.0 - rhs.0)
        } else {
            FieldElement(Self::MODULUS - rhs.0 + self.0)
        }
    }
}

impl Mul for FieldElement {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        // Use u128 for intermediate result to avoid overflow
        let result = (self.0 as u128 * rhs.0 as u128) % (Self::MODULUS as u128);
        FieldElement(result as u64)
    }
}

impl Neg for FieldElement {
    type Output = Self;

    fn neg(self) -> Self {
        if self.0 == 0 {
            self
        } else {
            FieldElement(Self::MODULUS - self.0)
        }
    }
}

impl Div for FieldElement {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        self * rhs.inverse().expect("division by zero")
    }
}

impl AddAssign for FieldElement {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for FieldElement {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign for FieldElement {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl DivAssign for FieldElement {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl PartialOrd for FieldElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FieldElement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl std::fmt::Display for FieldElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let a = FieldElement::new(10);
        let b = FieldElement::new(20);
        assert_eq!((a + b).value(), 30);
    }

    #[test]
    fn test_add_overflow() {
        let a = FieldElement::new(BABYBEAR_PRIME - 1);
        let b = FieldElement::new(2);
        assert_eq!((a + b).value(), 1);
    }

    #[test]
    fn test_sub() {
        let a = FieldElement::new(20);
        let b = FieldElement::new(10);
        assert_eq!((a - b).value(), 10);
    }

    #[test]
    fn test_sub_underflow() {
        let a = FieldElement::new(10);
        let b = FieldElement::new(20);
        assert_eq!((a - b).value(), BABYBEAR_PRIME - 10);
    }

    #[test]
    fn test_mul() {
        let a = FieldElement::new(10);
        let b = FieldElement::new(20);
        assert_eq!((a * b).value(), 200);
    }

    #[test]
    fn test_inverse() {
        let a = FieldElement::new(123);
        let a_inv = a.inverse().unwrap();
        assert_eq!((a * a_inv).value(), 1);
    }

    #[test]
    fn test_zero_inverse() {
        let zero = FieldElement::zero();
        assert!(zero.inverse().is_none());
    }

    #[test]
    fn test_neg() {
        let a = FieldElement::new(10);
        let neg_a = -a;
        assert_eq!((a + neg_a).value(), 0);
    }

    #[test]
    fn test_div() {
        let a = FieldElement::new(100);
        let b = FieldElement::new(10);
        let c = a / b;
        assert_eq!((c * b).value(), 100);
    }

    #[test]
    fn test_ord() {
        let a = FieldElement::new(10);
        let b = FieldElement::new(20);
        assert!(a < b);
        assert!(b > a);
        assert!(a <= a);
        assert!(a >= a);
    }

    #[test]
    fn test_assign_ops() {
        let mut a = FieldElement::new(10);
        a += FieldElement::new(5);
        assert_eq!(a.value(), 15);

        a -= FieldElement::new(3);
        assert_eq!(a.value(), 12);

        a *= FieldElement::new(2);
        assert_eq!(a.value(), 24);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_field_element() -> impl Strategy<Value = FieldElement> {
        (0u64..BABYBEAR_PRIME).prop_map(FieldElement::new)
    }

    fn arb_nonzero_field_element() -> impl Strategy<Value = FieldElement> {
        (1u64..BABYBEAR_PRIME).prop_map(FieldElement::new)
    }

    proptest! {
        #[test]
        fn test_add_commutative(a in arb_field_element(), b in arb_field_element()) {
            prop_assert_eq!(a + b, b + a);
        }

        #[test]
        fn test_mul_commutative(a in arb_field_element(), b in arb_field_element()) {
            prop_assert_eq!(a * b, b * a);
        }

        #[test]
        fn test_add_associative(
            a in arb_field_element(),
            b in arb_field_element(),
            c in arb_field_element()
        ) {
            prop_assert_eq!((a + b) + c, a + (b + c));
        }

        #[test]
        fn test_mul_associative(
            a in arb_field_element(),
            b in arb_field_element(),
            c in arb_field_element()
        ) {
            prop_assert_eq!((a * b) * c, a * (b * c));
        }

        #[test]
        fn test_add_identity(a in arb_field_element()) {
            let zero = FieldElement::zero();
            prop_assert_eq!(a + zero, a);
            prop_assert_eq!(zero + a, a);
        }

        #[test]
        fn test_mul_identity(a in arb_field_element()) {
            let one = FieldElement::one();
            prop_assert_eq!(a * one, a);
            prop_assert_eq!(one * a, a);
        }

        #[test]
        fn test_add_inverse(a in arb_field_element()) {
            let neg_a = -a;
            prop_assert_eq!((a + neg_a).value(), 0);
        }

        #[test]
        fn test_mul_inverse(a in arb_nonzero_field_element()) {
            let inv_a = a.inverse().unwrap();
            prop_assert_eq!((a * inv_a).value(), 1);
        }

        #[test]
        fn test_distributive(
            a in arb_field_element(),
            b in arb_field_element(),
            c in arb_field_element()
        ) {
            prop_assert_eq!(a * (b + c), a * b + a * c);
        }

        #[test]
        fn test_bytes_roundtrip(a in arb_field_element()) {
            let bytes = a.to_bytes();
            let recovered = FieldElement::from_bytes(bytes);
            prop_assert_eq!(a, recovered);
        }

        #[test]
        fn test_sub_is_add_neg(a in arb_field_element(), b in arb_field_element()) {
            prop_assert_eq!(a - b, a + (-b));
        }

        #[test]
        fn test_div_is_mul_inv(a in arb_field_element(), b in arb_nonzero_field_element()) {
            prop_assert_eq!(a / b, a * b.inverse().unwrap());
        }
    }
}
