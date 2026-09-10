// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
#![allow(clippy::cast_possible_wrap)]

use indexmap::IndexSet;

use super::{traits::HashDefaultContinue, Error, Kind, Result};
use crate::{tag::Tag, BignumRef, Continue, Fixnum, FromPrimitive, Serialize, Sym, Symbol};

/// The `alox_48` serializer.
#[derive(Debug, Clone)]
pub struct Serializer {
    /// The underlying output of the serializer.
    pub output: Vec<u8>,
    symlink: IndexSet<Symbol>,
}

#[derive(Debug)]
pub struct SerializeIvars<'a> {
    serializer: &'a mut Serializer,
    len: usize,
    index: usize,
    state: MapState,
}

#[derive(Debug)]
pub struct SerializeArray<'a> {
    serializer: &'a mut Serializer,
    len: usize,
    index: usize,
}

impl Default for Serializer {
    fn default() -> Self {
        Self {
            output: vec![4, 8],
            symlink: IndexSet::new(),
        }
    }
}

#[derive(Debug)]
enum MapState {
    Key,
    Value,
}

impl Serializer {
    /// Creates a new deserializer.
    ///
    /// Same as [`Default::default`].
    #[must_use]
    pub fn new() -> Self {
        Serializer::default()
    }

    // Does not emit a type byte.
    fn write_fixnum(&mut self, v: Fixnum) {
        let v: i32 = v.into();
        match v {
            0 => self.write(0),
            1..=122 => self.write(v as u8 + 5),
            -122..=0 => self.write((256 + v - 5) as u8),
            mut v => {
                let mut res = vec![];

                for _ in 0..4 {
                    let b = v & 255;
                    res.push(b as _);

                    v >>= 8;

                    if v == 0 || v == -1 {
                        break;
                    }
                }

                let l_byte = if v < 0 {
                    (256 - res.len()) as u8
                } else {
                    res.len() as _
                };

                self.write(l_byte);
                self.write_bytes(res);
            }
        }
    }

    fn write_usize(&mut self, v: usize) -> Result<()> {
        self.write_fixnum(FromPrimitive::from_usize(v).ok_or(Error {
            kind: Kind::LenOverflow(v),
        })?);
        Ok(())
    }

    fn write(&mut self, b: impl Into<u8>) {
        self.output.push(b.into());
    }

    fn write_symbol(&mut self, symbol: &Sym) -> Result<()> {
        if let Some(idx) = self.symlink.get_index_of(symbol) {
            self.write(Tag::Symlink);
            self.write_usize(idx)?;
        } else {
            self.symlink.insert(symbol.to_symbol());

            self.write(Tag::Symbol);
            self.write_usize(symbol.len())?;

            self.write_bytes(symbol);
        }

        Ok(())
    }

    fn write_bytes(&mut self, bytes: impl AsRef<[u8]>) {
        for &b in bytes.as_ref() {
            self.write(b);
        }
    }

    fn write_bytes_len(&mut self, bytes: impl AsRef<[u8]>) -> Result<()> {
        let bytes = bytes.as_ref();

        self.write_usize(bytes.len())?;
        self.write_bytes(bytes);

        Ok(())
    }
}

impl<'a> super::SerializerTrait for &'a mut Serializer {
    type Ok = ();

    type SerializeIvars = SerializeIvars<'a>;
    type SerializeArray = SerializeArray<'a>;

    type SerializeHash = SerializeHashImpl<'a, No>;
    type SerializeHashDefault = SerializeHashImpl<'a, Yes>;

    fn serialize_nil(self) -> Result<Self::Ok> {
        self.write(Tag::Nil);

        Ok(())
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.write(if v { Tag::True } else { Tag::False });

        Ok(())
    }

    fn serialize_fixnum(self, v: Fixnum) -> Result<Self::Ok> {
        self.write(Tag::Fixnum);
        self.write_fixnum(v);

        Ok(())
    }

    fn serialize_bignum(self, v: BignumRef<'_>) -> Result<Self::Ok> {
        self.write(Tag::Bignum);

        let (is_negative, le_bytes) = v.as_le_bytes();

        self.write(if is_negative { b'-' } else { b'+' });

        self.write_usize(le_bytes.len().div_ceil(2))?;
        self.write_bytes(le_bytes);
        if le_bytes.len() % 2 != 0 {
            self.write(0u8);
        }

        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.write(Tag::Float);

        let str = v.to_string();
        self.write_bytes_len(str)?;

        Ok(())
    }

    fn serialize_hash(self, len: usize) -> Result<Continue<Self::SerializeHash, Self::Ok>> {
        self.write(Tag::Hash);
        self.write_usize(len)?;

        Ok(Self::SerializeHash::new(self, len).next_access())
    }

    fn serialize_hash_default(self, len: usize) -> Result<HashDefaultContinue<Self>> {
        self.write(Tag::HashDefault);
        self.write_usize(len)?;

        Ok(Self::SerializeHashDefault::new(self, len).next_access())
    }

    fn serialize_array(self, len: usize) -> Result<Self::SerializeArray> {
        self.write(Tag::Array);
        self.write_usize(len)?;

        Ok(SerializeArray {
            serializer: self,
            len,
            index: 0,
        })
    }

    fn serialize_string(self, data: &[u8]) -> Result<Self::Ok> {
        self.write(Tag::String);
        self.write_bytes_len(data)?;

        Ok(())
    }

    fn serialize_symbol(self, sym: &Sym) -> Result<Self::Ok> {
        self.write_symbol(sym)?;

        Ok(())
    }

    fn serialize_regular_expression(self, regex: &[u8], flags: u8) -> Result<Self::Ok> {
        self.write(Tag::RawRegexp);
        self.write_bytes_len(regex)?;
        self.write(flags);

        Ok(())
    }

    fn serialize_object(self, class: &Sym, len: usize) -> Result<Self::SerializeIvars> {
        self.write(Tag::Object);
        self.write_symbol(class)?;
        self.write_usize(len)?;

        Ok(SerializeIvars {
            serializer: self,
            len,
            index: 0,
            state: MapState::Value, // we want to enforce getting a key next so we set the state to value
        })
    }

    fn serialize_struct(self, name: &Sym, len: usize) -> Result<Self::SerializeIvars> {
        self.write(Tag::Struct);
        self.write_symbol(name)?;
        self.write_usize(len)?;

        Ok(SerializeIvars {
            serializer: self,
            len,
            index: 0,
            state: MapState::Value, // we want to enforce getting a key next so we set the state to value
        })
    }

    fn serialize_class(self, class: &Sym) -> Result<Self::Ok> {
        self.write(Tag::ClassRef);
        // Apparently, this isn't a symbol. How strange!
        self.write_bytes_len(class)?;

        Ok(())
    }

    fn serialize_module(self, module: &Sym) -> Result<Self::Ok> {
        self.write(Tag::ModuleRef);
        self.write_bytes_len(module)?;

        Ok(())
    }

    fn serialize_instance<V>(self, value: &V, len: usize) -> Result<Self::SerializeIvars>
    where
        V: crate::Serialize + ?Sized,
    {
        self.write(Tag::Instance);
        value.serialize(&mut *self)?;
        self.write_usize(len)?;

        Ok(SerializeIvars {
            serializer: self,
            len,
            index: 0,
            state: MapState::Value, // we want to enforce getting a key next so we set the state to value
        })
    }

    fn serialize_extended<V>(self, module: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: crate::Serialize + ?Sized,
    {
        // the ruby docs lie! it is the module which comes before the value.
        self.write(Tag::Extended);
        self.write_symbol(module)?;
        value.serialize(self)
    }

    fn serialize_user_class<V>(self, class: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: crate::Serialize + ?Sized,
    {
        self.write(Tag::UserClass);
        self.write_symbol(class)?;
        value.serialize(self)
    }

    fn serialize_user_data(self, class: &Sym, data: &[u8]) -> Result<Self::Ok> {
        self.write(Tag::UserDef);
        self.write_symbol(class)?;
        self.write_bytes_len(data)?;

        Ok(())
    }

    fn serialize_user_marshal<V>(self, class: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: crate::Serialize + ?Sized,
    {
        self.write(Tag::UserMarshal);
        self.write_symbol(class)?;
        value.serialize(self)
    }

    fn serialize_data<V>(self, class: &Sym, value: &V) -> Result<Self::Ok>
    where
        V: crate::Serialize + ?Sized,
    {
        self.write(Tag::Data);
        self.write_symbol(class)?;
        value.serialize(self)
    }
}

impl super::SerializeIvars for SerializeIvars<'_> {
    type Ok = ();

    fn serialize_field(&mut self, k: &Sym) -> Result<()> {
        self.index += 1;
        if self.index > self.len {
            return Err(Error {
                kind: Kind::OvershotProvidedLen(self.index, self.len),
            });
        }
        match self.state {
            MapState::Key => {
                return Err(Error {
                    kind: Kind::KeyAfterKey,
                })
            }
            MapState::Value => self.state = MapState::Key,
        }

        self.serializer.write_symbol(k)?;

        Ok(())
    }

    fn serialize_value<V>(&mut self, v: &V) -> Result<()>
    where
        V: crate::Serialize + ?Sized,
    {
        match self.state {
            MapState::Value => {
                return Err(Error {
                    kind: Kind::ValueAfterValue,
                })
            }
            MapState::Key => self.state = MapState::Value,
        }
        v.serialize(&mut *self.serializer)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        if self.index < self.len {
            Err(Error {
                kind: Kind::UndershotProvidedLen(self.index, self.len),
            })
        } else {
            Ok(())
        }
    }
}

impl super::SerializeArray for SerializeArray<'_> {
    type Ok = ();

    fn serialize_element<T>(&mut self, v: &T) -> Result<()>
    where
        T: crate::Serialize + ?Sized,
    {
        self.index += 1;
        v.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok> {
        if self.index < self.len {
            Err(Error {
                kind: Kind::UndershotProvidedLen(self.index, self.len),
            })
        } else {
            Ok(())
        }
    }
}

pub trait HasDefault: Sized {
    type Finished<'a>;

    fn next_access(
        serialize: SerializeHashImpl<'_, Self>,
    ) -> Continue<SerializeHashImpl<'_, Self>, Self::Finished<'_>>;
}

#[derive(Debug)]
pub struct SerializeHashImpl<'a, T> {
    serializer: &'a mut Serializer,
    len: usize,
    index: usize,
    marker: std::marker::PhantomData<T>,
}

#[derive(Debug, Clone, Copy)]
pub struct Yes;

#[derive(Debug, Clone, Copy)]
pub struct No;

impl HasDefault for Yes {
    type Finished<'a> = SerializeHashImpl<'a, Self>;

    fn next_access(
        access: SerializeHashImpl<'_, Self>,
    ) -> Continue<SerializeHashImpl<'_, Self>, Self::Finished<'_>> {
        if access.index >= access.len {
            Continue::Finished(access)
        } else {
            Continue::Next(access)
        }
    }
}

impl HasDefault for No {
    type Finished<'a> = ();

    fn next_access(
        access: SerializeHashImpl<'_, Self>,
    ) -> Continue<SerializeHashImpl<'_, Self>, ()> {
        if access.index >= access.len {
            Continue::Finished(())
        } else {
            Continue::Next(access)
        }
    }
}

impl<'a, T> SerializeHashImpl<'a, T>
where
    T: HasDefault,
{
    pub fn new(serializer: &'a mut Serializer, len: usize) -> Self {
        Self {
            serializer,
            index: 0,
            len,
            marker: std::marker::PhantomData,
        }
    }

    pub fn next_access(self) -> Continue<Self, T::Finished<'a>> {
        T::next_access(self)
    }
}

impl<'a, T> crate::SerializeHashKey for SerializeHashImpl<'a, T>
where
    T: HasDefault,
{
    type SerializeValue = Self;
    type Finished = T::Finished<'a>;

    fn serialize_key<K>(mut self, k: &K) -> Result<Self::SerializeValue>
    where
        K: Serialize + ?Sized,
    {
        self.index += 1;
        k.serialize(&mut *self.serializer)?;
        Ok(self)
    }

    fn index(&self) -> usize {
        self.index
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T> crate::SerializeHashValue for SerializeHashImpl<'a, T>
where
    T: HasDefault,
{
    type SerializeKey = Self;
    type Finished = T::Finished<'a>;

    fn serialize_value<V>(self, v: &V) -> Result<Continue<Self::SerializeKey, Self::Finished>>
    where
        V: Serialize + ?Sized,
    {
        v.serialize(&mut *self.serializer)?;
        Ok(self.next_access())
    }

    fn index(&self) -> usize {
        self.index
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl<T> crate::SerializeHashDefault for SerializeHashImpl<'_, T>
where
    T: HasDefault,
{
    type Ok = ();

    fn serialize_default<V>(self, v: &V) -> Result<Self::Ok>
    where
        V: Serialize + ?Sized,
    {
        v.serialize(&mut *self.serializer)
    }
}
