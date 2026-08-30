// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// A type representing a borrowed arbitrary-precision integer outside of the interval
/// $[-2^30, 2^30)$.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BignumRef<'a> {
    /// True if the integer is less than zero, false if the integer is zero or greater than zero.
    is_negative: bool,
    /// Little-endian bytes of the integer with all trailing zero bytes removed.
    le_bytes: &'a [u8],
}

/// A type representing an owned arbitrary-precision integer outside of the interval
/// $[-2^30, 2^30)$.
#[derive(Clone, PartialEq, Eq)]
pub struct Bignum {
    /// True if the integer is less than zero, false if the integer is zero or greater than zero.
    is_negative: bool,
    /// Little-endian bytes of the integer with all trailing zero bytes removed.
    le_bytes: Vec<u8>,
}

impl<'a> From<&'a Bignum> for BignumRef<'a> {
    fn from(value: &'a Bignum) -> Self {
        Self {
            is_negative: value.is_negative,
            le_bytes: &value.le_bytes,
        }
    }
}

impl<'a> From<BignumRef<'a>> for Bignum {
    fn from(value: BignumRef<'a>) -> Self {
        Self {
            is_negative: value.is_negative,
            le_bytes: value.le_bytes.to_vec(),
        }
    }
}

impl<'a> BignumRef<'a> {
    /// Attempts to create a new `BignumRef` from a sign and little-endian bytes.
    /// Will fail if the represented integer is within the interval $[-2^30, 2^30)$.
    pub fn from_le_bytes(is_negative: bool, le_bytes: &'a [u8]) -> Option<Self> {
        let le_bytes = &le_bytes[..super::get_le_bytes_size(le_bytes)];
        let is_negative = if le_bytes.is_empty() {
            false
        } else {
            is_negative
        };
        let value = Self {
            is_negative,
            le_bytes,
        };
        num_traits::ToPrimitive::to_i32(&value)
            .is_none_or(|int| <crate::Fixnum as num_traits::FromPrimitive>::from_i32(int).is_none())
            .then_some(value)
    }

    /// Returns the sign (true if negative, false if nonnegative) and little-endian bytes.
    pub fn as_le_bytes(&self) -> (bool, &[u8]) {
        (self.is_negative, self.le_bytes)
    }
}

impl Bignum {
    /// Attempts to create a new `Bignum` from a sign and little-endian bytes.
    /// Will fail if the represented integer is within the interval $[-2^30, 2^30)$.
    pub fn from_le_bytes(is_negative: bool, mut le_bytes: Vec<u8>) -> Option<Self> {
        le_bytes.truncate(super::get_le_bytes_size(&le_bytes));
        let is_negative = if le_bytes.is_empty() {
            false
        } else {
            is_negative
        };
        let value = Self {
            is_negative,
            le_bytes,
        };
        num_traits::ToPrimitive::to_i32(&value)
            .is_none_or(|int| <crate::Fixnum as num_traits::FromPrimitive>::from_i32(int).is_none())
            .then_some(value)
    }

    /// Borrows this object as a `BignumRef`.
    pub fn as_ref(&self) -> BignumRef<'_> {
        self.into()
    }

    /// Returns the sign (true if negative, false if nonnegative) and little-endian bytes.
    pub fn as_le_bytes(&self) -> (bool, &[u8]) {
        (self.is_negative, &self.le_bytes)
    }

    /// Consumes this object, returning the sign (true if negative, false if nonnegative) and
    /// little-endian bytes.
    pub fn to_le_bytes(self) -> (bool, Vec<u8>) {
        (self.is_negative, self.le_bytes)
    }
}

impl<'a> From<BignumRef<'a>> for num_bigint::BigInt {
    fn from(value: BignumRef<'a>) -> Self {
        num_bigint::BigInt::from_bytes_le(
            if value.is_negative {
                num_bigint::Sign::Minus
            } else {
                num_bigint::Sign::Plus
            },
            value.le_bytes,
        )
    }
}

impl<'a> From<&BignumRef<'a>> for num_bigint::BigInt {
    fn from(value: &BignumRef<'a>) -> Self {
        (*value).into()
    }
}

impl From<&Bignum> for num_bigint::BigInt {
    fn from(value: &Bignum) -> Self {
        value.as_ref().into()
    }
}

impl num_traits::FromPrimitive for Bignum {
    fn from_i64(n: i64) -> Option<Self> {
        let le_bytes = n.wrapping_abs().to_le_bytes();
        crate::Fixnum::from_i64(n).is_none().then(|| Self {
            is_negative: n.is_negative(),
            le_bytes: le_bytes[..super::get_le_bytes_size(&le_bytes)].to_vec(),
        })
    }

    fn from_i128(n: i128) -> Option<Self> {
        let le_bytes = n.wrapping_abs().to_le_bytes();
        crate::Fixnum::from_i128(n).is_none().then(|| Self {
            is_negative: n.is_negative(),
            le_bytes: le_bytes[..super::get_le_bytes_size(&le_bytes)].to_vec(),
        })
    }

    fn from_u64(n: u64) -> Option<Self> {
        let le_bytes = n.to_le_bytes();
        crate::Fixnum::from_u64(n).is_none().then(|| Self {
            is_negative: false,
            le_bytes: le_bytes[..super::get_le_bytes_size(&le_bytes)].to_vec(),
        })
    }

    fn from_u128(n: u128) -> Option<Self> {
        let le_bytes = n.to_le_bytes();
        crate::Fixnum::from_u128(n).is_none().then(|| Self {
            is_negative: false,
            le_bytes: le_bytes[..super::get_le_bytes_size(&le_bytes)].to_vec(),
        })
    }

    fn from_f64(n: f64) -> Option<Self> {
        if !n.is_finite() {
            return None;
        }
        let (mantissa, exponent, sign) = num_traits::Float::integer_decode(n);
        let (padding_size, value) = if exponent > 0 {
            (exponent / 8, mantissa << (exponent % 8))
        } else {
            (0, mantissa.unbounded_shr(-exponent as _))
        };
        let padding_size = padding_size as usize;
        let mut le_bytes = Vec::with_capacity(padding_size + size_of::<u64>());
        le_bytes.extend(std::iter::repeat_n(0, padding_size).chain(value.to_le_bytes()));
        Self::from_le_bytes(sign < 0, le_bytes)
    }
}

impl num_traits::ToPrimitive for BignumRef<'_> {
    fn to_i64(&self) -> Option<i64> {
        (self.le_bytes.len() <= size_of::<i64>())
            .then(|| {
                let value = i64::from_le_bytes([
                    self.le_bytes.first().copied().unwrap_or_default(),
                    self.le_bytes.get(1).copied().unwrap_or_default(),
                    self.le_bytes.get(2).copied().unwrap_or_default(),
                    self.le_bytes.get(3).copied().unwrap_or_default(),
                    self.le_bytes.get(4).copied().unwrap_or_default(),
                    self.le_bytes.get(5).copied().unwrap_or_default(),
                    self.le_bytes.get(6).copied().unwrap_or_default(),
                    self.le_bytes.get(7).copied().unwrap_or_default(),
                ]);
                let value = if self.is_negative {
                    value.wrapping_neg()
                } else {
                    value
                };
                (value.is_negative() == self.is_negative).then_some(value)
            })
            .flatten()
    }

    fn to_i128(&self) -> Option<i128> {
        (self.le_bytes.len() <= size_of::<i128>())
            .then(|| {
                let value = i128::from_le_bytes([
                    self.le_bytes.first().copied().unwrap_or_default(),
                    self.le_bytes.get(1).copied().unwrap_or_default(),
                    self.le_bytes.get(2).copied().unwrap_or_default(),
                    self.le_bytes.get(3).copied().unwrap_or_default(),
                    self.le_bytes.get(4).copied().unwrap_or_default(),
                    self.le_bytes.get(5).copied().unwrap_or_default(),
                    self.le_bytes.get(6).copied().unwrap_or_default(),
                    self.le_bytes.get(7).copied().unwrap_or_default(),
                    self.le_bytes.get(8).copied().unwrap_or_default(),
                    self.le_bytes.get(9).copied().unwrap_or_default(),
                    self.le_bytes.get(10).copied().unwrap_or_default(),
                    self.le_bytes.get(11).copied().unwrap_or_default(),
                    self.le_bytes.get(12).copied().unwrap_or_default(),
                    self.le_bytes.get(13).copied().unwrap_or_default(),
                    self.le_bytes.get(14).copied().unwrap_or_default(),
                    self.le_bytes.get(15).copied().unwrap_or_default(),
                ]);
                let value = if self.is_negative {
                    value.wrapping_neg()
                } else {
                    value
                };
                (value.is_negative() == self.is_negative).then_some(value)
            })
            .flatten()
    }

    fn to_u64(&self) -> Option<u64> {
        (!self.is_negative && self.le_bytes.len() <= size_of::<u64>()).then(|| {
            u64::from_le_bytes([
                self.le_bytes.first().copied().unwrap_or_default(),
                self.le_bytes.get(1).copied().unwrap_or_default(),
                self.le_bytes.get(2).copied().unwrap_or_default(),
                self.le_bytes.get(3).copied().unwrap_or_default(),
                self.le_bytes.get(4).copied().unwrap_or_default(),
                self.le_bytes.get(5).copied().unwrap_or_default(),
                self.le_bytes.get(6).copied().unwrap_or_default(),
                self.le_bytes.get(7).copied().unwrap_or_default(),
            ])
        })
    }

    fn to_u128(&self) -> Option<u128> {
        (!self.is_negative && self.le_bytes.len() <= size_of::<u128>()).then(|| {
            u128::from_le_bytes([
                self.le_bytes.first().copied().unwrap_or_default(),
                self.le_bytes.get(1).copied().unwrap_or_default(),
                self.le_bytes.get(2).copied().unwrap_or_default(),
                self.le_bytes.get(3).copied().unwrap_or_default(),
                self.le_bytes.get(4).copied().unwrap_or_default(),
                self.le_bytes.get(5).copied().unwrap_or_default(),
                self.le_bytes.get(6).copied().unwrap_or_default(),
                self.le_bytes.get(7).copied().unwrap_or_default(),
                self.le_bytes.get(8).copied().unwrap_or_default(),
                self.le_bytes.get(9).copied().unwrap_or_default(),
                self.le_bytes.get(10).copied().unwrap_or_default(),
                self.le_bytes.get(11).copied().unwrap_or_default(),
                self.le_bytes.get(12).copied().unwrap_or_default(),
                self.le_bytes.get(13).copied().unwrap_or_default(),
                self.le_bytes.get(14).copied().unwrap_or_default(),
                self.le_bytes.get(15).copied().unwrap_or_default(),
            ])
        })
    }

    fn to_f64(&self) -> Option<f64> {
        let mantissa_le_bytes =
            &self.le_bytes[self.le_bytes.len().saturating_sub(size_of::<u64>())..];
        let mut mantissa_first_nonzero_byte_index = 0;
        for (i, byte) in mantissa_le_bytes.iter().copied().enumerate() {
            if byte != 0 {
                mantissa_first_nonzero_byte_index = i;
                break;
            }
        }
        let mantissa_le_bytes = &mantissa_le_bytes[mantissa_first_nonzero_byte_index..];
        let shift = self.le_bytes.len() - mantissa_le_bytes.len();
        let value_unsigned = if shift > (f64::MAX_EXP / 8).try_into().unwrap() {
            f64::INFINITY
        } else {
            let mantissa = u64::from_le_bytes([
                mantissa_le_bytes.first().copied().unwrap_or_default(),
                mantissa_le_bytes.get(1).copied().unwrap_or_default(),
                mantissa_le_bytes.get(2).copied().unwrap_or_default(),
                mantissa_le_bytes.get(3).copied().unwrap_or_default(),
                mantissa_le_bytes.get(4).copied().unwrap_or_default(),
                mantissa_le_bytes.get(5).copied().unwrap_or_default(),
                mantissa_le_bytes.get(6).copied().unwrap_or_default(),
                mantissa_le_bytes.get(7).copied().unwrap_or_default(),
            ]);
            num_traits::ToPrimitive::to_f64(&mantissa).unwrap() * 2.0f64.powi(8 * shift as i32)
        };
        Some(if self.is_negative {
            -value_unsigned
        } else {
            value_unsigned
        })
    }
}

impl num_traits::ToPrimitive for Bignum {
    fn to_i64(&self) -> Option<i64> {
        self.as_ref().to_i64()
    }

    fn to_i128(&self) -> Option<i128> {
        self.as_ref().to_i128()
    }

    fn to_u64(&self) -> Option<u64> {
        self.as_ref().to_u64()
    }

    fn to_u128(&self) -> Option<u128> {
        self.as_ref().to_u128()
    }

    fn to_f64(&self) -> Option<f64> {
        self.as_ref().to_f64()
    }
}

impl num_bigint::ToBigInt for BignumRef<'_> {
    fn to_bigint(&self) -> Option<num_bigint::BigInt> {
        Some(self.into())
    }
}

impl num_bigint::ToBigInt for Bignum {
    fn to_bigint(&self) -> Option<num_bigint::BigInt> {
        self.as_ref().to_bigint()
    }
}

impl num_bigint::ToBigUint for BignumRef<'_> {
    fn to_biguint(&self) -> Option<num_bigint::BigUint> {
        (!self.is_negative).then(|| num_bigint::BigUint::from_bytes_le(self.le_bytes))
    }
}

impl num_bigint::ToBigUint for Bignum {
    fn to_biguint(&self) -> Option<num_bigint::BigUint> {
        self.as_ref().to_biguint()
    }
}

impl Ord for BignumRef<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let ord = other.is_negative.cmp(&self.is_negative);
        if ord.is_ne() {
            return ord;
        }
        let ord = self.le_bytes.len().cmp(&other.le_bytes.len());
        if ord.is_ne() {
            return ord;
        }
        self.le_bytes.iter().rev().cmp(other.le_bytes.iter().rev())
    }
}

impl Ord for Bignum {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(&other.as_ref())
    }
}

impl PartialOrd for BignumRef<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd for Bignum {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Debug for BignumRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        num_bigint::BigInt::from(self).fmt(f)
    }
}

impl std::fmt::Debug for Bignum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl std::fmt::Display for BignumRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        num_bigint::BigInt::from(self).fmt(f)
    }
}

impl std::fmt::Display for Bignum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}
