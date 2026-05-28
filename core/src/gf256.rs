use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};
use zeroize::Zeroize;

/// Element in the Galois Field GF(2^8).
/// Polynomial: x^8 + x^4 + x^3 + x + 1 (0x11B)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Zeroize)]
pub struct Gf256(pub u8);

impl Gf256 {
    pub const ZERO: Gf256 = Gf256(0);
    pub const ONE: Gf256 = Gf256(1);

    /// Computes the multiplicative inverse of `self`.
    /// Panics if `self` is zero.
    pub fn inverse(self) -> Self {
        assert_ne!(self.0, 0, "Zero has no inverse in GF(2^8)");
        // a^254 = a^-1 in GF(2^8)
        let mut res = Gf256::ONE;
        let mut base = self;
        let mut exp = 254u8;
        while exp > 0 {
            if exp & 1 == 1 {
                res = res * base;
            }
            base = base * base;
            exp >>= 1;
        }
        res
    }

    /// Evaluates a polynomial (where `coeffs[0]` is the constant term) at `x = self`.
    pub fn evaluate_polynomial(coeffs: &[Gf256], x: Gf256) -> Gf256 {
        // Horner's method
        let mut result = Gf256::ZERO;
        for &coeff in coeffs.iter().rev() {
            result = (result * x) + coeff;
        }
        result
    }
}

// ── Trait Implementations for Ergonomics ────────────────────────────────────────

impl Add for Gf256 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Gf256(self.0 ^ rhs.0)
    }
}

impl AddAssign for Gf256 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Sub for Gf256 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self + rhs
    }
}

impl SubAssign for Gf256 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for Gf256 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut a = self.0;
        let mut b = rhs.0;
        let mut p = 0u8;

        for _ in 0..8 {
            if b & 1 == 1 {
                p ^= a;
            }
            let hi_bit_set = a & 0x80;
            a <<= 1;
            if hi_bit_set == 0x80 {
                a ^= 0x1b; // Reduction polynomial
            }
            b >>= 1;
        }
        Gf256(p)
    }
}

impl MulAssign for Gf256 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
