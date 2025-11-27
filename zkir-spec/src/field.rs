//! Baby Bear field for ZK IR

use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub, Mul, Neg};

/// Baby Bear prime: 2^31 - 2^27 + 1 = 2013265921
pub const BABYBEAR_PRIME: u32 = 2013265921;

/// Baby Bear field element (31-bit prime)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[repr(transparent)]
pub struct BabyBear(pub u32);

impl BabyBear {
    pub const MODULUS: u32 = BABYBEAR_PRIME;
    pub const ZERO: Self = BabyBear(0);
    pub const ONE: Self = BabyBear(1);

    #[inline]
    pub fn new(value: u32) -> Self {
        BabyBear(value % Self::MODULUS)
    }

    #[inline]
    pub fn value(self) -> u32 {
        self.0
    }

    #[inline]
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn add(self, rhs: Self) -> Self {
        let sum = self.0 as u64 + rhs.0 as u64;
        BabyBear((sum % Self::MODULUS as u64) as u32)
    }

    #[inline]
    pub fn sub(self, rhs: Self) -> Self {
        if self.0 >= rhs.0 {
            BabyBear(self.0 - rhs.0)
        } else {
            BabyBear(Self::MODULUS - rhs.0 + self.0)
        }
    }

    #[inline]
    pub fn mul(self, rhs: Self) -> Self {
        let prod = self.0 as u64 * rhs.0 as u64;
        BabyBear((prod % Self::MODULUS as u64) as u32)
    }

    #[inline]
    pub fn neg(self) -> Self {
        if self.0 == 0 {
            self
        } else {
            BabyBear(Self::MODULUS - self.0)
        }
    }

    pub fn pow(self, mut exp: u32) -> Self {
        let mut base = self;
        let mut result = Self::ONE;
        while exp > 0 {
            if exp & 1 == 1 {
                result = result.mul(base);
            }
            base = base.mul(base);
            exp >>= 1;
        }
        result
    }

    pub fn inverse(self) -> Option<Self> {
        if self.is_zero() {
            None
        } else {
            Some(self.pow(Self::MODULUS - 2))
        }
    }
}

impl From<u32> for BabyBear {
    fn from(value: u32) -> Self {
        BabyBear::new(value)
    }
}

impl Add for BabyBear {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.add(rhs)
    }
}

impl Sub for BabyBear {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.sub(rhs)
    }
}

impl Mul for BabyBear {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.mul(rhs)
    }
}

impl Neg for BabyBear {
    type Output = Self;
    fn neg(self) -> Self {
        self.neg()
    }
}

impl std::fmt::Display for BabyBear {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
