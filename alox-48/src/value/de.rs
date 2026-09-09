// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use crate::{
    de::{
        ArrayAccess, Deserialize, DeserializeSeed, DeserializerTrait, Error, HashDefaultAccess,
        HashKeyAccess, HashValueAccess, InstanceAccess, IvarAccess, Kind, Result, Visitor,
        VisitorInstance, VisitorOption,
    },
    rb_types::{
        BignumRef, Fixnum, HashVisitor, InstanceVisitor, ObjectVisitor, RbFields, RbString,
        StringVisitor, StructVisitor, Sym, UserdataVisitor,
    },
    Continue, RbHash, Value,
};

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("any ruby value")
    }

    fn visit_nil(self) -> Result<Self::Value> {
        Ok(Value::Nil)
    }

    fn visit_bool(self, v: bool) -> Result<Self::Value> {
        Ok(Value::Bool(v))
    }

    fn visit_fixnum(self, v: Fixnum) -> Result<Self::Value> {
        Ok(Value::Fixnum(v))
    }

    fn visit_bignum(self, v: BignumRef<'de>) -> Result<Self::Value> {
        Ok(Value::Bignum(v.into()))
    }

    fn visit_f64(self, v: f64) -> Result<Self::Value> {
        Ok(Value::Float(v))
    }

    fn visit_hash<A>(self, current: crate::Continue<A, ()>) -> Result<Self::Value>
    where
        A: HashKeyAccess<'de, Finished = ()>,
    {
        HashVisitor.visit_hash(current).map(Value::Hash)
    }

    fn visit_hash_default<A>(self, current: crate::Continue<A, A::Finished>) -> Result<Self::Value>
    where
        A: HashKeyAccess<'de>,
        A::Finished: HashDefaultAccess<'de>,
    {
        HashVisitor.visit_hash_default(current).map(Value::Hash)
    }

    fn visit_array<A>(self, mut access: A) -> Result<Self::Value>
    where
        A: ArrayAccess<'de>,
    {
        let mut array = Vec::with_capacity(access.len());
        while let Some(v) = access.next_element()? {
            array.push(v);
        }
        Ok(Value::Array(array))
    }

    fn visit_string(self, string: &'de [u8]) -> Result<Self::Value> {
        StringVisitor.visit_string(string).map(Value::String)
    }

    fn visit_symbol(self, symbol: &'de Sym) -> Result<Self::Value> {
        Ok(Value::Symbol(symbol.to_symbol()))
    }

    fn visit_regular_expression(self, data: &'de [u8], flags: u8) -> Result<Self::Value> {
        Ok(Value::Regex {
            data: RbString::from(data),
            flags,
        })
    }

    fn visit_object<A>(self, class: &'de Sym, instance_variables: A) -> Result<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        ObjectVisitor
            .visit_object(class, instance_variables)
            .map(Value::Object)
    }

    fn visit_struct<A>(self, name: &'de Sym, members: A) -> Result<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        StructVisitor
            .visit_struct(name, members)
            .map(Value::RbStruct)
    }

    fn visit_class(self, class: &'de Sym) -> Result<Self::Value> {
        Ok(Value::Class(class.to_symbol()))
    }

    fn visit_module(self, module: &'de Sym) -> Result<Self::Value> {
        Ok(Value::Module(module.to_symbol()))
    }

    fn visit_instance<A>(self, instance: A) -> Result<Self::Value>
    where
        A: InstanceAccess<'de>,
    {
        InstanceVisitor(std::marker::PhantomData)
            .visit_instance(instance)
            .map(Value::Instance)
    }

    fn visit_extended<D>(self, module: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let value = deserializer.deserialize(ValueVisitor)?;
        Ok(Value::Extended {
            module: module.to_symbol(),
            value: Box::new(value),
        })
    }

    fn visit_user_class<D>(self, class: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let value = deserializer.deserialize(ValueVisitor)?;
        Ok(Value::UserClass {
            class: class.to_symbol(),
            value: Box::new(value),
        })
    }

    fn visit_user_data(self, class: &'de Sym, data: &'de [u8]) -> Result<Self::Value> {
        UserdataVisitor
            .visit_user_data(class, data)
            .map(Value::Userdata)
    }

    fn visit_user_marshal<D>(self, class: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let value = deserializer.deserialize(ValueVisitor)?;
        Ok(Value::UserMarshal {
            class: class.to_symbol(),
            value: Box::new(value),
        })
    }

    fn visit_data<D>(self, class: &'de Sym, deserializer: D) -> Result<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let value = deserializer.deserialize(ValueVisitor)?;
        Ok(Value::Data {
            class: class.to_symbol(),
            value: Box::new(value),
        })
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self>
    where
        D: DeserializerTrait<'de>,
    {
        deserializer.deserialize(ValueVisitor)
    }
}

struct ValueInstanceAccess<'de> {
    value: &'de Value,
    fields: &'de RbFields,
}

struct ValueIVarAccess<'de> {
    fields: &'de RbFields,
    index: usize,
    state: MapState,
}

enum MapState {
    Key,
    Value,
}

struct ValueArrayAccess<'de> {
    array: &'de [Value],
    index: usize,
}

impl<'de> DeserializerTrait<'de> for &'de Value {
    fn deserialize<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Nil => visitor.visit_nil(),
            Value::Bool(v) => visitor.visit_bool(*v),
            Value::Float(f) => visitor.visit_f64(*f),
            Value::Fixnum(i) => visitor.visit_fixnum(*i),
            Value::Bignum(i) => visitor.visit_bignum(i.as_ref()),
            Value::String(s) => visitor.visit_string(&s.data),
            Value::Symbol(s) => visitor.visit_symbol(s),
            Value::Array(array) => visitor.visit_array(ValueArrayAccess { array, index: 0 }),
            Value::Hash(RbHash { map, default: None }) => {
                visitor.visit_hash(HashAccessImpl::<No>::next_access(map.iter(), ()))
            }
            Value::Hash(RbHash {
                map,
                default: Some(v),
            }) => visitor.visit_hash_default(HashAccessImpl::<Yes>::next_access(
                map.iter(),
                ValueDefaultAccess(v),
            )),
            Value::Userdata(u) => visitor.visit_user_data(&u.class, &u.data),
            Value::Object(o) => visitor.visit_object(
                &o.class,
                ValueIVarAccess {
                    fields: &o.fields,
                    index: 0,
                    state: MapState::Value, // we want to enforce getting a key next so we set the state to value
                },
            ),
            Value::Instance(i) => visitor.visit_instance(ValueInstanceAccess {
                value: &i.value,
                fields: &i.fields,
            }),
            Value::Regex { data, flags } => visitor.visit_regular_expression(&data.data, *flags),
            Value::RbStruct(s) => visitor.visit_struct(
                &s.class,
                ValueIVarAccess {
                    fields: &s.fields,
                    index: 0,
                    state: MapState::Value, // we want to enforce getting a key next so we set the state to value
                },
            ),
            Value::Class(c) => visitor.visit_class(c),
            Value::Module(m) => visitor.visit_module(m),
            Value::Extended { module, value } => visitor.visit_extended(module, value.as_ref()),
            Value::UserClass { class, value } => visitor.visit_user_class(class, value.as_ref()),
            Value::UserMarshal { class, value } => {
                visitor.visit_user_marshal(class, value.as_ref())
            }
            Value::Data { class, value } => visitor.visit_data(class, value.as_ref()),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: VisitorOption<'de>,
    {
        if matches!(self, Value::Nil) {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_instance<V>(self, visitor: V) -> Result<V::Value>
    where
        V: VisitorInstance<'de>,
    {
        if let Value::Instance(i) = self {
            visitor.visit_instance(ValueInstanceAccess {
                value: &i.value,
                fields: &i.fields,
            })
        } else {
            visitor.visit(self)
        }
    }
}

impl<'de> InstanceAccess<'de> for ValueInstanceAccess<'de> {
    type IvarAccess = ValueIVarAccess<'de>;

    fn value_seed<V>(self, seed: V) -> Result<(V::Value, Self::IvarAccess)>
    where
        V: DeserializeSeed<'de>,
    {
        let value = seed.deserialize(self.value)?;
        let access = ValueIVarAccess {
            fields: self.fields,
            index: 0,
            state: MapState::Value, // we want to enforce getting a key next so we set the state to value
        };
        Ok((value, access))
    }
}

impl<'de> IvarAccess<'de> for ValueIVarAccess<'de> {
    fn next_ivar(&mut self) -> Result<Option<&'de Sym>> {
        let Some((field, _)) = self.fields.get_index(self.index) else {
            return Ok(None);
        };

        match self.state {
            MapState::Key => {
                return Err(Error {
                    kind: Kind::KeyAfterKey,
                })
            }
            MapState::Value => self.state = MapState::Key,
        }

        Ok(Some(field))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let (_, value) = self.fields.get_index(self.index).ok_or(Error {
            kind: Kind::ValueAfterValue,
        })?;
        self.state = MapState::Value;
        self.index += 1;

        seed.deserialize(value)
    }

    fn len(&self) -> usize {
        self.fields.len()
    }

    fn index(&self) -> usize {
        self.index
    }
}

impl<'de> ArrayAccess<'de> for ValueArrayAccess<'de> {
    fn next_element_seed<V>(&mut self, seed: V) -> Result<Option<V::Value>>
    where
        V: DeserializeSeed<'de>,
    {
        let Some(value) = self.array.get(self.index) else {
            return Ok(None);
        };
        self.index += 1;
        seed.deserialize(value).map(Some)
    }

    fn len(&self) -> usize {
        self.array.len()
    }

    fn index(&self) -> usize {
        self.index
    }
}

type Iter<'de> = indexmap::map::Iter<'de, Value, Value>;

pub struct HashAccessImpl<'de, T>
where
    T: HasDefault<'de>,
{
    iter: Iter<'de>,
    next_key: &'de Value,
    next_value: &'de Value,
    finished: T::Finished,
}

pub struct HashValueAccessImpl<'de, T>
where
    T: HasDefault<'de>,
{
    iter: Iter<'de>,
    next_value: &'de Value,
    finished: T::Finished,
}

pub struct ValueDefaultAccess<'de>(&'de Value);

pub struct Yes;

pub struct No;

pub trait HasDefault<'de> {
    type Finished;
}

impl<'de> HasDefault<'de> for Yes {
    type Finished = ValueDefaultAccess<'de>;
}

impl HasDefault<'_> for No {
    type Finished = ();
}

impl<'de, T> HashAccessImpl<'de, T>
where
    T: HasDefault<'de>,
{
    fn next_access(mut iter: Iter<'de>, finished: T::Finished) -> Continue<Self, T::Finished> {
        match iter.next() {
            Some((next_key, next_value)) => Continue::Next(Self {
                iter,
                next_key,
                next_value,
                finished,
            }),
            None => Continue::Finished(finished),
        }
    }

    fn into_value_access(self) -> (&'de Value, HashValueAccessImpl<'de, T>) {
        (
            self.next_key,
            HashValueAccessImpl {
                iter: self.iter,
                next_value: self.next_value,
                finished: self.finished,
            },
        )
    }
}

impl<'de, T> HashValueAccessImpl<'de, T>
where
    T: HasDefault<'de>,
{
    fn into_next_access(self) -> (&'de Value, Continue<HashAccessImpl<'de, T>, T::Finished>) {
        (
            self.next_value,
            HashAccessImpl::next_access(self.iter, self.finished),
        )
    }
}

impl<'de, T> HashKeyAccess<'de> for HashAccessImpl<'de, T>
where
    T: HasDefault<'de>,
{
    type ValueAccess = HashValueAccessImpl<'de, T>;
    type Finished = T::Finished;

    fn next_key_seed<K>(self, seed: K) -> Result<(K::Value, Self::ValueAccess)>
    where
        K: DeserializeSeed<'de>,
    {
        let (next_key, access) = self.into_value_access();
        seed.deserialize(next_key).map(|v| (v, access))
    }

    fn len(&self) -> usize {
        self.iter.len()
    }

    fn index(&self) -> usize {
        todo!()
    }
}

impl<'de, T> HashValueAccess<'de> for HashValueAccessImpl<'de, T>
where
    T: HasDefault<'de>,
{
    type KeyAccess = HashAccessImpl<'de, T>;
    type Finished = T::Finished;

    fn next_value_seed<V>(
        self,
        seed: V,
    ) -> Result<(V::Value, Continue<Self::KeyAccess, Self::Finished>)>
    where
        V: DeserializeSeed<'de>,
    {
        let (next_value, access) = self.into_next_access();
        seed.deserialize(next_value).map(|v| (v, access))
    }

    fn len(&self) -> usize {
        self.iter.len()
    }

    fn index(&self) -> usize {
        todo!()
    }
}

impl<'de> HashDefaultAccess<'de> for ValueDefaultAccess<'de> {
    fn deserialize_default_seed<V>(self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(self.0)
    }
}
