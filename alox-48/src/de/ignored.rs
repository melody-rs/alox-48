// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use super::{traits::InstanceAccess, Deserialize, Result, Visitor};
use crate::{
    de::traits::{HashDefaultAccess, HashKeyAccess},
    BignumRef, Continue, DeserializerTrait, Fixnum, IvarAccess, Sym,
};

/// A type that implements deserialize which ignores all values.
#[derive(Clone, Copy, Debug, Default)]
pub struct Ignored;

struct IgnoredVisitor;

impl<'de> Visitor<'de> for IgnoredVisitor {
    type Value = Ignored;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("any value")
    }

    fn visit_nil(self) -> Result<Self::Value> {
        Ok(Ignored)
    }
    fn visit_bool(self, _v: bool) -> Result<Self::Value> {
        Ok(Ignored)
    }
    fn visit_fixnum(self, _v: Fixnum) -> Result<Self::Value> {
        Ok(Ignored)
    }
    fn visit_bignum(self, _v: BignumRef<'de>) -> Result<Self::Value> {
        Ok(Ignored)
    }
    fn visit_f64(self, _v: f64) -> Result<Self::Value> {
        Ok(Ignored)
    }

    fn visit_hash<A>(self, mut current: Continue<A, ()>) -> Result<Self::Value>
    where
        A: HashKeyAccess<'de, Finished = ()>,
    {
        while let Continue::Next(access) = current {
            let (_, _, next) = access.next_entry::<Ignored, Ignored>()?;
            current = next;
        }

        Ok(Ignored)
    }

    fn visit_hash_default<A>(self, mut current: Continue<A, A::Finished>) -> Result<Self::Value>
    where
        A: HashKeyAccess<'de>,
        A::Finished: HashDefaultAccess<'de>,
    {
        loop {
            match current {
                Continue::Next(access) => {
                    let (_, _, next) = access.next_entry::<Ignored, Ignored>()?;
                    current = next;
                }
                Continue::Finished(default) => {
                    break default.deserialize_default::<Ignored>();
                }
            }
        }
    }

    fn visit_array<A>(self, mut array: A) -> Result<Self::Value>
    where
        A: crate::ArrayAccess<'de>,
    {
        while let Some(Ignored) = array.next_element()? {}
        Ok(Ignored)
    }
    fn visit_string(self, _string: &'de [u8]) -> Result<Self::Value> {
        Ok(Ignored)
    }
    fn visit_symbol(self, _symbol: &'de Sym) -> Result<Self::Value> {
        Ok(Ignored)
    }
    fn visit_regular_expression(self, _regex: &'de [u8], _flags: u8) -> Result<Self::Value> {
        Ok(Ignored)
    }

    fn visit_object<A>(self, _class: &'de Sym, mut instance_variables: A) -> Result<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        while let Some((_, Ignored)) = instance_variables.next_entry()? {}
        Ok(Ignored)
    }
    fn visit_struct<A>(self, _name: &'de Sym, mut members: A) -> Result<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        while let Some((_, Ignored)) = members.next_entry()? {}
        Ok(Ignored)
    }

    fn visit_class(self, _class: &'de Sym) -> Result<Self::Value> {
        Ok(Ignored)
    }
    fn visit_module(self, _module: &'de Sym) -> Result<Self::Value> {
        Ok(Ignored)
    }

    fn visit_instance<A>(self, instance: A) -> Result<Self::Value>
    where
        A: InstanceAccess<'de>,
    {
        let (_, mut instance_variables) = instance.value::<Ignored>()?;
        while let Some((_, Ignored)) = instance_variables.next_entry()? {}
        Ok(Ignored)
    }

    fn visit_extended<D>(self, _module: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(IgnoredVisitor)
    }

    fn visit_user_class<D>(self, _class: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(IgnoredVisitor)
    }

    fn visit_user_data(self, _class: &'de Sym, _data: &'de [u8]) -> Result<Self::Value> {
        Ok(Ignored)
    }

    fn visit_user_marshal<D>(self, _class: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(IgnoredVisitor)
    }

    fn visit_data<D>(self, _class: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(IgnoredVisitor)
    }
}

impl<'de> Deserialize<'de> for Ignored {
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(IgnoredVisitor)
    }
}
