// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// A type representing an integer within the interval $[-2^30, 2^30)$.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fixnum(i32);

impl Fixnum {
    /// Attempts to create a new `Fixnum` from a sign and little-endian bytes.
    /// Will fail if the represented integer is outside of the interval $[-2^30, 2^30)$.
    pub fn from_le_bytes(is_negative: bool, le_bytes: &[u8]) -> Option<Self> {
        let le_bytes = &le_bytes[..super::get_le_bytes_size(le_bytes)];
        let is_negative = if le_bytes.is_empty() {
            false
        } else {
            is_negative
        };
        (le_bytes.len() <= size_of::<i32>())
            .then(|| {
                let value = i32::from_le_bytes([
                    le_bytes.first().copied().unwrap_or_default(),
                    le_bytes.get(1).copied().unwrap_or_default(),
                    le_bytes.get(2).copied().unwrap_or_default(),
                    le_bytes.get(3).copied().unwrap_or_default(),
                ]);
                let value = if is_negative {
                    value.wrapping_neg()
                } else {
                    value
                };
                (value.is_negative() == is_negative)
                    .then(|| num_traits::FromPrimitive::from_i32(value))
                    .flatten()
            })
            .flatten()
    }
}

impl From<i8> for Fixnum {
    fn from(value: i8) -> Self {
        Self(value.into())
    }
}

impl From<u8> for Fixnum {
    fn from(value: u8) -> Self {
        Self(value.into())
    }
}

impl From<i16> for Fixnum {
    fn from(value: i16) -> Self {
        Self(value.into())
    }
}

impl From<u16> for Fixnum {
    fn from(value: u16) -> Self {
        Self(value.into())
    }
}

impl From<Fixnum> for i32 {
    fn from(value: Fixnum) -> Self {
        value.0
    }
}

impl From<Fixnum> for i64 {
    fn from(value: Fixnum) -> Self {
        value.0.into()
    }
}

impl From<Fixnum> for i128 {
    fn from(value: Fixnum) -> Self {
        value.0.into()
    }
}

impl From<Fixnum> for num_bigint::BigInt {
    fn from(value: Fixnum) -> Self {
        value.0.into()
    }
}

impl num_traits::FromPrimitive for Fixnum {
    fn from_i64(n: i64) -> Option<Self> {
        if (-(1 << 30)..1 << 30).contains(&n) {
            Some(Self(n as i32))
        } else {
            None
        }
    }

    fn from_u64(n: u64) -> Option<Self> {
        <u64 as num_traits::ToPrimitive>::to_i64(&n).and_then(Self::from_i64)
    }
}

impl num_traits::ToPrimitive for Fixnum {
    fn to_i64(&self) -> Option<i64> {
        self.0.to_i64()
    }

    fn to_u64(&self) -> Option<u64> {
        self.0.to_u64()
    }
}

impl num_bigint::ToBigInt for Fixnum {
    fn to_bigint(&self) -> Option<num_bigint::BigInt> {
        self.0.to_bigint()
    }
}

impl num_bigint::ToBigUint for Fixnum {
    fn to_biguint(&self) -> Option<num_bigint::BigUint> {
        self.0.to_biguint()
    }
}

impl std::fmt::Debug for Fixnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for Fixnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
