// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use indexmap::{IndexMap, IndexSet};

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque},
    hash::{BuildHasher, Hash},
    marker::PhantomData,
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
    },
};

use super::{
    traits::VisitorOption, ArrayAccess, Deserialize, DeserializeSeed, DeserializerTrait, Error,
    HashAccess, Kind, Result, Unexpected, Visitor,
};
use crate::{Bignum, BignumRef, Fixnum, Sym};

impl<'de, T> DeserializeSeed<'de> for PhantomData<T>
where
    T: Deserialize<'de>,
{
    type Value = T;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        T::deserialize(deserializer)
    }
}

struct IntVisitor<'de>(PhantomData<&'de ()>);

enum IntVisitorValue<'de> {
    Fixnum(Fixnum),
    Bignum(Bignum),
    BignumRef(BignumRef<'de>),
}

impl<'de> Visitor<'de> for IntVisitor<'de> {
    type Value = IntVisitorValue<'de>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an integer")
    }

    fn visit_fixnum(self, v: Fixnum) -> Result<Self::Value> {
        Ok(IntVisitorValue::Fixnum(v))
    }

    fn visit_bignum(self, v: BignumRef<'de>) -> Result<Self::Value> {
        Ok(IntVisitorValue::BignumRef(v))
    }

    fn visit_f64(self, v: f64) -> Result<Self::Value> {
        if let Some(fixnum) = num_traits::FromPrimitive::from_f64(v) {
            self.visit_fixnum(fixnum)
        } else {
            Ok(IntVisitorValue::Bignum(
                num_traits::FromPrimitive::from_f64(v).ok_or(Error {
                    kind: Kind::InvalidFloatToIntConversion,
                })?,
            ))
        }
    }
}

macro_rules! primitive_int_impl {
    ($($primitive:ty => $to_primitive:ident),* $(,)?) => {
        $(impl<'de> Deserialize<'de> for $primitive {
            fn deserialize<D>(deserializer: D) -> Result<Self>
            where
                D: DeserializerTrait<'de>,
            {
                match deserializer.deserialize(IntVisitor(Default::default()))? {
                    IntVisitorValue::Fixnum(v) => num_traits::ToPrimitive::$to_primitive(&v).ok_or(Error { kind: Kind::NumericOverflow }),
                    IntVisitorValue::Bignum(v) => num_traits::ToPrimitive::$to_primitive(&v).ok_or(Error { kind: Kind::NumericOverflow }),
                    IntVisitorValue::BignumRef(v) => num_traits::ToPrimitive::$to_primitive(&v).ok_or(Error { kind: Kind::NumericOverflow }),
                }
            }
        })*
    };
}

struct NonZeroIntVisitor<'de>(PhantomData<&'de ()>);

impl<'de> Visitor<'de> for NonZeroIntVisitor<'de> {
    type Value = IntVisitorValue<'de>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a non-zero integer")
    }

    fn visit_fixnum(self, v: Fixnum) -> Result<Self::Value> {
        if v == 0i16.into() {
            Err(Error::invalid_value(Unexpected::Fixnum(v), &self))
        } else {
            Ok(IntVisitorValue::Fixnum(v))
        }
    }

    fn visit_bignum(self, v: BignumRef<'de>) -> Result<Self::Value> {
        Ok(IntVisitorValue::BignumRef(v))
    }

    fn visit_f64(self, v: f64) -> Result<Self::Value> {
        if let Some(fixnum) = num_traits::FromPrimitive::from_f64(v) {
            self.visit_fixnum(fixnum)
        } else {
            Ok(IntVisitorValue::Bignum(
                num_traits::FromPrimitive::from_f64(v).ok_or(Error {
                    kind: Kind::InvalidFloatToIntConversion,
                })?,
            ))
        }
    }
}

primitive_int_impl!(
    u8 => to_u8,
    u16 => to_u16,
    u32 => to_u32,
    u64 => to_u64,
    u128 => to_u128,
    usize => to_usize,
    i8 => to_i8,
    i16 => to_i16,
    i32 => to_i32,
    i64 => to_i64,
    i128 => to_i128,
    isize => to_isize,
);

macro_rules! nonzero_int_impl {
    ($($primitive:ty => $to_primitive:ident),* $(,)?) => {
        $(impl<'de> Deserialize<'de> for $primitive {
            fn deserialize<D>(deserializer: D) -> Result<Self>
            where
                D: DeserializerTrait<'de>,
            {
                let i = match deserializer.deserialize(NonZeroIntVisitor(Default::default()))? {
                    IntVisitorValue::Fixnum(v) => num_traits::ToPrimitive::$to_primitive(&v).ok_or(Error { kind: Kind::NumericOverflow }),
                    IntVisitorValue::Bignum(v) => num_traits::ToPrimitive::$to_primitive(&v).ok_or(Error { kind: Kind::NumericOverflow }),
                    IntVisitorValue::BignumRef(v) => num_traits::ToPrimitive::$to_primitive(&v).ok_or(Error { kind: Kind::NumericOverflow }),
                }?;
                // we've already asserted that it's non-zero simply by the fact that NonZeroIntVisitor returns a nonzero integer.
                // so this new_unchecked is safe
                Ok(unsafe { <$primitive>::new_unchecked(i) })
            }
        })*
    };
}

nonzero_int_impl!(
    NonZeroU8 => to_u8,
    NonZeroU16 => to_u16,
    NonZeroU32 => to_u32,
    NonZeroU64 => to_u64,
    NonZeroU128 => to_u128,
    NonZeroUsize => to_usize,
    NonZeroI8 => to_i8,
    NonZeroI16 => to_i16,
    NonZeroI32 => to_i32,
    NonZeroI64 => to_i64,
    NonZeroI128 => to_i128,
    NonZeroIsize => to_isize,
);

struct UnitVisitor;

impl Visitor<'_> for UnitVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("unit")
    }

    fn visit_nil(self) -> Result<Self::Value> {
        Ok(())
    }
}

impl<'de> Deserialize<'de> for () {
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(UnitVisitor)
    }
}

struct BoolVisitor;

impl Visitor<'_> for BoolVisitor {
    type Value = bool;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("bool")
    }

    fn visit_bool(self, v: bool) -> Result<Self::Value> {
        Ok(v)
    }
}

impl<'de> Deserialize<'de> for bool {
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(BoolVisitor)
    }
}

struct FloatVisitor;

impl Visitor<'_> for FloatVisitor {
    type Value = f64;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a float")
    }

    fn visit_fixnum(self, v: Fixnum) -> Result<Self::Value> {
        num_traits::ToPrimitive::to_f64(&v).ok_or(Error {
            kind: Kind::NumericOverflow,
        })
    }

    fn visit_bignum(self, v: BignumRef<'_>) -> Result<Self::Value> {
        num_traits::ToPrimitive::to_f64(&v).ok_or(Error {
            kind: Kind::NumericOverflow,
        })
    }

    fn visit_f64(self, v: f64) -> Result<Self::Value> {
        Ok(v)
    }
}

impl<'de> Deserialize<'de> for f32 {
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: DeserializerTrait<'de>,
    {
        let v = deserializer.deserialize(FloatVisitor)?;
        Ok(v as f32)
    }
}

impl<'de> Deserialize<'de> for f64 {
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(FloatVisitor)
    }
}

struct StrVisitor;

impl<'de> Visitor<'de> for StrVisitor {
    type Value = &'de str;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a utf8 string")
    }

    fn visit_string(self, string: &'de [u8]) -> Result<Self::Value> {
        std::str::from_utf8(string)
            .map_err(|_| Error::invalid_value(super::error::Unexpected::String(string), &self))
    }

    fn visit_symbol(self, symbol: &'de Sym) -> Result<Self::Value> {
        Ok(symbol.as_str())
    }
}

impl<'de> Deserialize<'de> for &'de str {
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(StrVisitor)
    }
}

impl<'de> Deserialize<'de> for String {
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(StrVisitor).map(ToOwned::to_owned)
    }
}

struct BytesVisitor;

impl<'de> Visitor<'de> for BytesVisitor {
    type Value = &'de [u8];

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a ruby string")
    }

    fn visit_string(self, string: &'de [u8]) -> Result<Self::Value> {
        Ok(string)
    }
}

impl<'de> Deserialize<'de> for &'de [u8] {
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(BytesVisitor)
    }
}

struct OptionVisitor<T> {
    marker: PhantomData<T>,
}

impl<'de, T> VisitorOption<'de> for OptionVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Option<T>;

    fn visit_none(self) -> Result<Self::Value> {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        T::deserialize(deserializer).map(Some)
    }
}

impl<'de, T> Deserialize<'de> for Option<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize_option(OptionVisitor {
            marker: PhantomData,
        })
    }
}

macro_rules! seq_impl {
    (
        $(#[$attr:meta])*
        $ty:ident <T $(: $tbound1:ident $(+ $tbound2:ident)*)* $(, $typaram:ident : $bound1:ident $(+ $bound2:ident)*)*>,
        $access:ident,
        $with_capacity:expr,
        $insert:expr
    ) => {
        $(#[$attr])*
        impl<'de, T $(, $typaram)*> Deserialize<'de> for $ty<T $(, $typaram)*>
        where
            T: Deserialize<'de> $(+ $tbound1 $(+ $tbound2)*)*,
            $($typaram: $bound1 $(+ $bound2)*,)*
        {
            fn deserialize<D>(deserializer: D) -> Result<Self>
            where
                D: DeserializerTrait<'de>,
            {
                struct SeqVisitor<T $(, $typaram)*> {
                    marker: PhantomData<$ty<T $(, $typaram)*>>,
                }

                impl<'de, T $(, $typaram)*> Visitor<'de> for SeqVisitor<T $(, $typaram)*>
                where
                    T: Deserialize<'de> $(+ $tbound1 $(+ $tbound2)*)*,
                    $($typaram: $bound1 $(+ $bound2)*,)*
                {
                    type Value = $ty<T $(, $typaram)*>;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        formatter.write_str("an array")
                    }

                    #[inline]
                    fn visit_array<A>(self, mut $access: A) -> Result<Self::Value>
                    where
                        A: ArrayAccess<'de>,
                    {
                        let mut values = $with_capacity;

                        while let Some(value) = $access.next_element()? {
                            $insert(&mut values, value);
                        }

                        Ok(values)
                    }
                }

                let visitor = SeqVisitor { marker: PhantomData };
                deserializer.deserialize(visitor)
            }
        }
    }
}

seq_impl!(Vec<T>, array, Vec::with_capacity(array.len()), Vec::push);

seq_impl!(
    BTreeSet<T: Eq + Ord>,
    array,
    BTreeSet::new(),
    BTreeSet::insert
);

seq_impl!(
    LinkedList<T>,
    array,
    LinkedList::new(),
    LinkedList::push_back
);

seq_impl!(
    HashSet<T: Hash + Eq, H: BuildHasher + Default>,
    array,
    HashSet::with_capacity_and_hasher(array.len(), H::default()),
    HashSet::insert
);

seq_impl!(
    VecDeque<T: Hash + Eq>,
    array,
    VecDeque::with_capacity(array.len()),
    VecDeque::push_back
);

seq_impl!(
    IndexSet<T: Hash + Eq, H: BuildHasher + Default>,
    array,
    IndexSet::with_capacity_and_hasher(array.len(), H::default()),
    IndexSet::insert
);

struct ArrayVisitor<T, const SIZE: usize> {
    marker: PhantomData<[T; SIZE]>,
}

// not happy about this. maybe there's a better way?
impl<'de, T, const SIZE: usize> Visitor<'de> for ArrayVisitor<T, SIZE>
where
    T: Deserialize<'de>,
{
    type Value = [T; SIZE];

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_fmt(format_args!("an array of length {SIZE}",))
    }

    fn visit_array<A>(self, mut array: A) -> Result<Self::Value>
    where
        A: ArrayAccess<'de>,
    {
        // try_from_fn is not yet stabilized, so we need to use MaybeUninit instead. Oh well.

        // this is what the unstable uninit_array does.
        // this is safe because the types we are claiming to have initialized here are MaybeUninits which do not need initialization.
        let mut uninit_array: [std::mem::MaybeUninit<T>; SIZE] =
            unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        let mut index = 0;
        loop {
            match array.next_element() {
                Ok(Some(value)) => {
                    // (error case) if we filled up with too many elements, drop the elements we did fill up
                    if index == SIZE {
                        for elem in &mut uninit_array[0..index] {
                            unsafe { elem.assume_init_drop() }
                        }
                        break Err(Error::invalid_length(index, &self));
                    }
                    uninit_array[index].write(value);
                    index += 1;
                }
                Ok(None) => {
                    // (error case) if we didn't fill up with enough elements, drop the elements we did fill up
                    break if index < SIZE {
                        for elem in &mut uninit_array[0..index] {
                            unsafe { elem.assume_init_drop() }
                        }
                        Err(Error::invalid_length(index, &self))
                    } else {
                        // what i don't know can't hurt me :)
                        let array =
                            uninit_array.map(|v| unsafe { std::mem::MaybeUninit::assume_init(v) });
                        Ok(array)
                    };
                }
                Err(e) => {
                    // if we ran into an error, drop the elements we did fill up
                    for elem in &mut uninit_array[0..index] {
                        unsafe { elem.assume_init_drop() }
                    }
                    break Err(e);
                }
            }
        }
    }
}

impl<'de, T, const SIZE: usize> Deserialize<'de> for [T; SIZE]
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(ArrayVisitor {
            marker: PhantomData,
        })
    }
}

macro_rules! map_impl {
    (
        $(#[$attr:meta])*
        $ty:ident <K $(: $kbound1:ident $(+ $kbound2:ident)*)*, V $(, $typaram:ident : $bound1:ident $(+ $bound2:ident)*)*>,
        $access:ident,
        $with_capacity:expr
    ) => {
        $(#[$attr])*
        impl<'de, K, V $(, $typaram)*> Deserialize<'de> for $ty<K, V $(, $typaram)*>
        where
            K: Deserialize<'de> $(+ $kbound1 $(+ $kbound2)*)*,
            V: Deserialize<'de>,
            $($typaram: $bound1 $(+ $bound2)*),*
        {
            fn deserialize<D>(deserializer: D) -> Result<Self>
            where
                D: DeserializerTrait<'de>,
            {
                struct MapVisitor<K, V $(, $typaram)*> {
                    marker: PhantomData<$ty<K, V $(, $typaram)*>>,
                }

                impl<'de, K, V $(, $typaram)*> Visitor<'de> for MapVisitor<K, V $(, $typaram)*>
                where
                    K: Deserialize<'de> $(+ $kbound1 $(+ $kbound2)*)*,
                    V: Deserialize<'de>,
                    $($typaram: $bound1 $(+ $bound2)*),*
                {
                    type Value = $ty<K, V $(, $typaram)*>;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        formatter.write_str("a map")
                    }

                    #[inline]
                    fn visit_hash<A>(self, mut $access: A) -> Result<Self::Value>
                    where
                        A: HashAccess<'de>,
                    {
                        let mut values = $with_capacity;

                        while let Some((key, value)) = $access.next_entry()? {
                            values.insert(key, value);
                        }

                        Ok(values)
                    }
                }

                let visitor = MapVisitor { marker: PhantomData };
                deserializer.deserialize(visitor)
            }
        }
    }
}

map_impl!(BTreeMap<K: Ord, V>, map, BTreeMap::new());

map_impl!(
    HashMap<K: Eq + Hash, V, H: BuildHasher + Default>,
    map,
    HashMap::with_capacity_and_hasher(map.len(), H::default())
);

map_impl!(
    IndexMap<K: Eq + Hash, V, H: BuildHasher + Default>,
    map,
    IndexMap::with_capacity_and_hasher(map.len(), H::default())
);

impl<'de, T> Deserialize<'de> for Box<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: DeserializerTrait<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Ok(Box::new(value))
    }
}
