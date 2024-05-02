use core::{
  hash::{Hash, Hasher},
  iter::{Product, Sum},
  ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign},
};
use std::fmt;

use num_bigint::BigUint;
use p3_field::{exp_u64_by_squaring, halve_u32, AbstractField, Field, Packable};
use serde::{Deserialize, Serialize};

const PLUTO_FIELD_PRIME: u32 = 101;
// const MONTY_BITS: u32 = 7;
// const MONTY_MASK: u32 = (1 << MONTY_BITS) - 1;
// const MONTY_MU: u32 = 80;

#[derive(Copy, Clone, Default, Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct PlutoField {
  value: u32,
}

impl PlutoField {
  pub const ORDER_U32: u32 = PLUTO_FIELD_PRIME;

  pub fn new(value: u32) -> Self { Self { value } }
}

impl fmt::Display for PlutoField {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.value) }
}

impl Packable for PlutoField {}

impl Div for PlutoField {
  type Output = Self;

  #[allow(clippy::suspicious_arithmetic_impl)]
  fn div(self, rhs: Self) -> Self { self * rhs.inverse() }
}
impl Field for PlutoField {
  // TODO: Add cfg-guarded Packing for AVX2, NEON, etc.
  type Packing = Self;

  fn is_zero(&self) -> bool { self.value == 0 || self.value == Self::ORDER_U32 }

  #[inline]
  fn exp_u64_generic<AF: AbstractField<F = Self>>(val: AF, power: u64) -> AF {
    exp_u64_by_squaring(val, power)
  }

  fn try_inverse(&self) -> Option<Self> {
    if self.is_zero() {
      return None;
    }
    let exponent = PLUTO_FIELD_PRIME - 2; // p - 2
    let mut result = 1;
    let mut base = self.value;
    let mut power = exponent;

    while power > 0 {
      if power & 1 == 1 {
        result = result * base % PLUTO_FIELD_PRIME;
      }
      base = base * base % PLUTO_FIELD_PRIME;
      power >>= 1;
    }
    Some(Self { value: result })
  }

  #[inline]
  fn halve(&self) -> Self { PlutoField::new(halve_u32::<PLUTO_FIELD_PRIME>(self.value)) }

  #[inline]
  fn order() -> BigUint { PLUTO_FIELD_PRIME.into() }
}

impl AbstractField for PlutoField {
  type F = Self;

  fn zero() -> Self { Self::new(0) }

  fn one() -> Self { Self::new(1) }

  fn two() -> Self { Self::new(2) }

  fn neg_one() -> Self { Self::new(Self::ORDER_U32 - 1) }

  #[inline]
  fn from_f(f: Self::F) -> Self { f }

  fn from_bool(b: bool) -> Self { Self::new(u32::from(b)) }

  fn from_canonical_u8(n: u8) -> Self { Self::new(u32::from(n)) }

  fn from_canonical_u16(n: u16) -> Self { Self::new(u32::from(n)) }

  fn from_canonical_u32(n: u32) -> Self { Self::new(n) }

  #[inline]
  fn from_canonical_u64(n: u64) -> Self {
    debug_assert!(n < PLUTO_FIELD_PRIME as u64);
    Self::from_canonical_u32(n as u32)
  }

  #[inline]
  fn from_canonical_usize(n: usize) -> Self {
    debug_assert!(n < PLUTO_FIELD_PRIME as usize);
    Self::from_canonical_u32(n as u32)
  }

  #[inline]
  fn from_wrapped_u32(n: u32) -> Self { Self { value: n % PLUTO_FIELD_PRIME } }

  #[inline]
  fn from_wrapped_u64(n: u64) -> Self { Self { value: (n % PLUTO_FIELD_PRIME as u64) as u32 } }

  // generator for multiplicative subgroup of the field
  fn generator() -> Self { Self::new(2) }
}

impl Mul for PlutoField {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self { Self { value: (self.value * rhs.value) % 101 } }
}

impl Product for PlutoField {
  fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
    iter.reduce(|x, y| x * y).unwrap_or(Self::one())
  }
}

impl SubAssign for PlutoField {
  fn sub_assign(&mut self, rhs: Self) { *self = *self - rhs; }
}

impl AddAssign for PlutoField {
  fn add_assign(&mut self, rhs: Self) { *self = *self + rhs; }
}

impl MulAssign for PlutoField {
  fn mul_assign(&mut self, rhs: Self) { *self = *self * rhs; }
}

impl Neg for PlutoField {
  type Output = Self;

  fn neg(self) -> Self::Output { Self::new(Self::ORDER_U32 - self.value) }
}

impl Add for PlutoField {
  type Output = Self;

  fn add(self, rhs: Self) -> Self {
    let mut sum = self.value + rhs.value;
    let (corr_sum, over) = sum.overflowing_sub(PLUTO_FIELD_PRIME);
    if !over {
      sum = corr_sum;
    }
    Self { value: sum }
  }
}

impl Sum for PlutoField {
  fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
    iter.reduce(|x, y| x + y).unwrap_or(Self::zero())
  }
}

impl Sub for PlutoField {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self {
    let (mut diff, over) = self.value.overflowing_sub(rhs.value);
    let corr = if over { PLUTO_FIELD_PRIME } else { 0 };
    diff = diff.wrapping_add(corr);
    Self::new(diff)
  }
}

mod tests {
  use super::*;

  type F = PlutoField;

  #[test]
  fn test_overflowing_add() {
    let a = PlutoField::new(100);
    let b = PlutoField::new(20);
    let c = a + b;
    assert_eq!(c.value, 19);
  }

  #[test]
  fn underflow_sub() {
    let a = PlutoField::new(10);
    let b = PlutoField::new(20);
    let c = a - b;
    assert_eq!(c.value, 91);
  }

  #[test]
  fn overflowing_mul() {
    let a = PlutoField::new(10);
    let b = PlutoField::new(20);
    let c = a * b;
    println!("c: {:?}", c);
    assert_eq!(c.value, 99);
  }

  #[test]
  fn zero() {
    let f = F::from_canonical_u32(0);
    assert!(f.is_zero());

    let f = F::from_wrapped_u32(F::ORDER_U32);
    assert!(f.is_zero());
  }

  #[test]
  fn exp_u64_generic() {
    let f = F::from_canonical_u32(2);
    let f = F::exp_u64_generic(f, 3);
    assert_eq!(f, F::from_canonical_u32(8));
  }

  #[test]
  fn addition_subtraction() {
    let a = PlutoField::new(50);
    let b = PlutoField::new(60);
    let c = a + b;
    assert_eq!(c.value, 9); // (50 + 60) % 101 = 9

    let d = c - a;
    assert_eq!(d.value, 60); // (9 - 50) % 101 = 60
  }

  #[test]
  fn multiplicative_inverse() {
    let a = PlutoField::new(10);
    let a_inv = a.inverse();
    let should_be_one = a * a_inv;
    assert_eq!(should_be_one.value, 1);
  }

  #[test]
  fn identity_elements() {
    let a = PlutoField::new(10);
    let zero = PlutoField::new(0);
    let one = PlutoField::new(1);
    assert_eq!((a + zero).value, a.value);
    assert_eq!((a * one).value, a.value);
  }

  #[test]
  fn zero_multiplication() {
    let a = PlutoField::new(10);
    let zero = PlutoField::new(0);
    assert_eq!((a * zero).value, 0);
  }

  #[test]
  fn negation() {
    let a = PlutoField::new(10);
    let neg_a = -a;
    assert_eq!((a + neg_a).value, 0);
  }

  #[test]
  fn inverse_of_inverse() {
    let a = PlutoField::new(10);
    let a_inv = a.inverse();
    let a_inv_inv = a_inv.inverse();
    assert_eq!(a_inv_inv.value, a.value);
  }

  #[test]
  fn distributivity() {
    let a = PlutoField::new(5);
    let b = PlutoField::new(6);
    let c = PlutoField::new(7);
    assert_eq!((a * (b + c)).value, (a * b + a * c).value);
  }

  #[test]
  fn associativity() {
    let a = PlutoField::new(5);
    let b = PlutoField::new(6);
    let c = PlutoField::new(7);
    assert_eq!(((a + b) + c).value, (a + (b + c)).value);
    assert_eq!(((a * b) * c).value, (a * (b * c)).value);
  }

  #[test]
  fn non_zero_element() {
    let a = PlutoField::new(10);
    assert!(!a.is_zero());
  }

  #[test]
  fn power_of_zero() {
    let a = PlutoField::new(0);
    let b = PlutoField::exp_u64_generic(a, 3);
    assert_eq!(b.value, 0);
  }
}