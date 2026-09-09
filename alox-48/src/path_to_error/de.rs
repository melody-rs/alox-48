// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::{add_context, Context, Trace};
use crate::{
    de::{DeserializeSeed, DeserializerTrait, HashDefaultAccess, HashKeyAccess, HashValueAccess},
    ArrayAccess, BignumRef, Continue, DeResult, Fixnum, InstanceAccess, IvarAccess, Sym, Symbol,
    Visitor, VisitorInstance, VisitorOption,
};

/// A deserializer that tracks where errors occur.
#[derive(Debug)]
pub struct Deserializer<'trace, T> {
    deserializer: T,
    trace: &'trace mut Trace,
}

#[derive(Debug)]
struct Wrapped<'trace, X> {
    inner: X,
    trace: &'trace mut Trace,
}

impl<'de, 'trace, T> Deserializer<'trace, T>
where
    T: DeserializerTrait<'de>,
{
    /// Create a new deserializer.
    pub fn new(deserializer: T, track: &'trace mut Trace) -> Self {
        Self {
            deserializer,
            trace: track,
        }
    }
}

impl<'de, T> DeserializerTrait<'de> for Deserializer<'_, T>
where
    T: DeserializerTrait<'de>,
{
    fn deserialize<V>(self, visitor: V) -> DeResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserializer.deserialize(Wrapped {
            inner: visitor,
            trace: self.trace,
        })
    }

    fn deserialize_option<V>(self, visitor: V) -> DeResult<V::Value>
    where
        V: VisitorOption<'de>,
    {
        self.deserializer.deserialize_option(Wrapped {
            inner: visitor,
            trace: self.trace,
        })
    }

    fn deserialize_instance<V>(self, visitor: V) -> DeResult<V::Value>
    where
        V: VisitorInstance<'de>,
    {
        self.deserializer.deserialize_instance(Wrapped {
            inner: visitor,
            trace: self.trace,
        })
    }
}

impl<'de, X> Visitor<'de> for Wrapped<'_, X>
where
    X: Visitor<'de>,
{
    type Value = X::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.expecting(formatter)
    }

    fn visit_nil(self) -> DeResult<Self::Value> {
        add_context!(self.inner.visit_nil(), self.trace.push(Context::Nil))
    }

    fn visit_bool(self, v: bool) -> DeResult<Self::Value> {
        add_context!(self.inner.visit_bool(v), self.trace.push(Context::Bool(v)))
    }

    fn visit_fixnum(self, v: Fixnum) -> DeResult<Self::Value> {
        add_context!(
            self.inner.visit_fixnum(v),
            self.trace.push(Context::Fixnum(v))
        )
    }

    fn visit_bignum(self, v: BignumRef<'de>) -> DeResult<Self::Value> {
        add_context!(
            self.inner.visit_bignum(v),
            self.trace.push(Context::Bignum(v.into()))
        )
    }

    fn visit_f64(self, v: f64) -> DeResult<Self::Value> {
        add_context!(self.inner.visit_f64(v), self.trace.push(Context::Float(v)))
    }

    fn visit_hash<A>(self, current: Continue<A, ()>) -> DeResult<Self::Value>
    where
        A: HashKeyAccess<'de, Finished = ()>,
    {
        let wrapped = WrappedHashAccess::<_, No>::next_access(self.trace, current);
        let len = wrapped.len();
        add_context!(
            self.inner.visit_hash(wrapped),
            self.trace.push(Context::Hash(len))
        )
    }

    fn visit_hash_default<A>(self, current: Continue<A, A::Finished>) -> DeResult<Self::Value>
    where
        A: HashKeyAccess<'de>,
        A::Finished: HashDefaultAccess<'de>,
    {
        let wrapped = WrappedHashAccess::<_, Yes>::next_access(self.trace, current);
        let len = wrapped.len();
        add_context!(
            self.inner.visit_hash_default(wrapped),
            self.trace.push(Context::Hash(len))
        )
    }

    fn visit_array<A>(self, array: A) -> DeResult<Self::Value>
    where
        A: ArrayAccess<'de>,
    {
        let wrapped = Wrapped {
            inner: array,
            trace: self.trace,
        };
        let len = wrapped.len();
        add_context!(
            self.inner.visit_array(wrapped),
            self.trace.push(Context::Array(len))
        )
    }

    fn visit_string(self, string: &'de [u8]) -> DeResult<Self::Value> {
        add_context!(
            self.inner.visit_string(string),
            self.trace.push(Context::String(
                String::from_utf8_lossy(string).into_owned()
            ))
        )
    }

    fn visit_symbol(self, symbol: &'de Sym) -> DeResult<Self::Value> {
        add_context!(
            self.inner.visit_symbol(symbol),
            self.trace.push(Context::Symbol(symbol.to_symbol()))
        )
    }

    fn visit_regular_expression(self, regex: &'de [u8], flags: u8) -> DeResult<Self::Value> {
        add_context!(
            self.inner.visit_regular_expression(regex, flags),
            self.trace.push(Context::Regex(
                String::from_utf8_lossy(regex).into_owned(),
                flags
            ))
        )
    }

    fn visit_object<A>(self, class: &'de Sym, instance_variables: A) -> DeResult<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        let wrapped = WrappedIvarAccess {
            inner: instance_variables,
            trace: self.trace,
            current_field: None,
        };
        let len = wrapped.len();
        add_context!(
            self.inner.visit_object(class, wrapped),
            self.trace.push(Context::Object(class.to_symbol(), len))
        )
    }

    fn visit_struct<A>(self, name: &'de Sym, members: A) -> DeResult<Self::Value>
    where
        A: IvarAccess<'de>,
    {
        let wrapped = WrappedIvarAccess {
            inner: members,
            trace: self.trace,
            current_field: None,
        };
        let len = wrapped.len();
        add_context!(
            self.inner.visit_struct(name, wrapped),
            self.trace.push(Context::Struct(name.to_symbol(), len))
        )
    }

    fn visit_class(self, class: &'de Sym) -> DeResult<Self::Value> {
        add_context!(
            self.inner.visit_class(class),
            self.trace.push(Context::Class(class.to_symbol()))
        )
    }

    fn visit_module(self, module: &'de Sym) -> DeResult<Self::Value> {
        add_context!(
            self.inner.visit_module(module),
            self.trace.push(Context::Module(module.to_symbol()))
        )
    }

    fn visit_instance<A>(self, instance: A) -> DeResult<Self::Value>
    where
        A: InstanceAccess<'de>,
    {
        let wrapped = Wrapped {
            inner: instance,
            trace: self.trace,
        };
        add_context!(
            self.inner.visit_instance(wrapped),
            self.trace.push(Context::Instance)
        )
    }

    fn visit_extended<D>(self, module: &'de Sym, deserializer: D) -> DeResult<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let wrapped = Deserializer::new(deserializer, self.trace);
        add_context!(
            self.inner.visit_extended(module, wrapped),
            self.trace.push(Context::Extended(module.to_symbol()))
        )
    }

    fn visit_user_class<D>(self, class: &'de Sym, deserializer: D) -> DeResult<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let wrapped = Deserializer::new(deserializer, self.trace);
        add_context!(
            self.inner.visit_user_class(class, wrapped),
            self.trace.push(Context::UserClass(class.to_symbol()))
        )
    }

    fn visit_user_data(self, class: &'de Sym, data: &'de [u8]) -> DeResult<Self::Value> {
        add_context!(
            self.inner.visit_user_data(class, data),
            self.trace.push(Context::UserData(class.to_symbol()))
        )
    }

    fn visit_user_marshal<D>(self, class: &'de Sym, deserializer: D) -> DeResult<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let wrapped = Deserializer::new(deserializer, self.trace);
        add_context!(
            self.inner.visit_user_marshal(class, wrapped),
            self.trace.push(Context::UserMarshal(class.to_symbol()))
        )
    }

    fn visit_data<D>(self, class: &'de Sym, deserializer: D) -> DeResult<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        let wrapped = Deserializer::new(deserializer, self.trace);
        add_context!(
            self.inner.visit_data(class, wrapped),
            self.trace.push(Context::Data(class.to_symbol()))
        )
    }
}

impl<'de, X> VisitorOption<'de> for Wrapped<'_, X>
where
    X: VisitorOption<'de>,
{
    type Value = X::Value;

    fn visit_none(self) -> DeResult<Self::Value> {
        self.inner.visit_none()
    }

    fn visit_some<D>(self, deserializer: D) -> DeResult<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        self.inner
            .visit_some(Deserializer::new(deserializer, self.trace))
    }
}

impl<'de, X> VisitorInstance<'de> for Wrapped<'_, X>
where
    X: VisitorInstance<'de>,
{
    type Value = X::Value;

    fn visit<D>(self, deserializer: D) -> DeResult<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        self.inner
            .visit(Deserializer::new(deserializer, self.trace))
    }

    fn visit_instance<A>(self, access: A) -> DeResult<Self::Value>
    where
        A: InstanceAccess<'de>,
    {
        self.inner.visit_instance(Wrapped {
            inner: access,
            trace: self.trace,
        })
    }
}

impl<'de, 'trace, X> InstanceAccess<'de> for Wrapped<'trace, X>
where
    X: InstanceAccess<'de>,
{
    type IvarAccess = WrappedIvarAccess<'trace, X::IvarAccess>;

    fn value_seed<V>(self, seed: V) -> DeResult<(V::Value, Self::IvarAccess)>
    where
        V: DeserializeSeed<'de>,
    {
        let wrapped_seed = Wrapped {
            inner: seed,
            trace: &mut *self.trace,
        };
        let (value, access) = self.inner.value_seed(wrapped_seed)?;
        let wrapped_access = WrappedIvarAccess {
            inner: access,
            trace: self.trace,
            current_field: None,
        };
        Ok((value, wrapped_access))
    }
}

struct WrappedIvarAccess<'trace, X> {
    inner: X,
    trace: &'trace mut Trace,
    current_field: Option<Symbol>,
}

impl<'de, X> IvarAccess<'de> for WrappedIvarAccess<'_, X>
where
    X: IvarAccess<'de>,
{
    fn next_ivar(&mut self) -> DeResult<Option<&'de Sym>> {
        let symbol = add_context!(
            self.inner.next_ivar(),
            self.trace.push(Context::FetchingField(self.index()))
        )?;
        self.current_field = symbol.map(Sym::to_symbol);
        Ok(symbol)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> DeResult<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let wrapped_seed = Wrapped {
            inner: seed,
            trace: self.trace,
        };
        add_context!(
            self.inner.next_value_seed(wrapped_seed),
            self.trace
                .push(Context::Field(self.current_field.clone(), self.index()))
        )
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn index(&self) -> usize {
        self.inner.index()
    }
}

trait HasDefault<'de, X>
where
    X: HashKeyAccess<'de>,
{
    type Finished<'a>;

    fn map_finished(trace: &mut Trace, finished: X::Finished) -> Self::Finished<'_>;
}

struct Yes;

impl<'de, X> HasDefault<'de, X> for Yes
where
    X: HashKeyAccess<'de>,
    X::Finished: HashDefaultAccess<'de>,
{
    type Finished<'a> = WrappedHashDefault<'a, X::Finished>;

    fn map_finished(trace: &mut Trace, finished: X::Finished) -> Self::Finished<'_> {
        WrappedHashDefault {
            trace,
            inner: finished,
        }
    }
}

struct No;

impl<'de, X> HasDefault<'de, X> for No
where
    X: HashKeyAccess<'de, Finished = ()>,
{
    type Finished<'a> = ();

    #[allow(clippy::semicolon_if_nothing_returned)]
    fn map_finished(_: &mut Trace, finished: X::Finished) -> Self::Finished<'_> {
        finished
    }
}

#[derive(Debug)]
struct WrappedHashAccess<'trace, X, T> {
    inner: X,
    trace: &'trace mut Trace,
    marker: std::marker::PhantomData<T>,
}

impl<'de, 'trace, X, T> WrappedHashAccess<'trace, X, T>
where
    X: HashKeyAccess<'de>,
    T: HasDefault<'de, X>,
{
    fn new(trace: &'trace mut Trace, access: X) -> Self {
        Self {
            inner: access,
            trace,
            marker: std::marker::PhantomData,
        }
    }

    fn next_access(
        trace: &'trace mut Trace,
        access: Continue<X, X::Finished>,
    ) -> Continue<Self, T::Finished<'trace>> {
        match access {
            Continue::Next(access) => Continue::Next(Self::new(trace, access)),
            Continue::Finished(access) => Continue::Finished(T::map_finished(trace, access)),
        }
    }
}

struct WrappedHashValueAccess<'trace, X, T> {
    inner: X,
    trace: &'trace mut Trace,
    marker: std::marker::PhantomData<T>,
}

impl<'de, 'trace, X, T> WrappedHashValueAccess<'trace, X, T>
where
    X: HashValueAccess<'de>,
    T: HasDefault<'de, X::KeyAccess>,
{
    fn new(trace: &'trace mut Trace, access: X) -> Self {
        Self {
            inner: access,
            trace,
            marker: std::marker::PhantomData,
        }
    }
}

impl<'a, 'de, X, T> HashKeyAccess<'de> for WrappedHashAccess<'a, X, T>
where
    X: HashKeyAccess<'de>,
    T: HasDefault<'de, X>,
{
    type ValueAccess = WrappedHashValueAccess<'a, X::ValueAccess, T>;
    type Finished = T::Finished<'a>;

    fn next_key_seed<K>(self, seed: K) -> DeResult<(K::Value, Self::ValueAccess)>
    where
        K: DeserializeSeed<'de>,
    {
        let index = self.inner.index();
        let trace = self.trace;
        self.inner
            .next_key_seed(Wrapped { inner: seed, trace })
            .inspect_err(|_| trace.push(Context::HashKey(index)))
            .map(|(k, access)| (k, WrappedHashValueAccess::new(trace, access)))
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn index(&self) -> usize {
        self.inner.index()
    }
}

impl<'a, 'de, X, T> HashValueAccess<'de> for WrappedHashValueAccess<'a, X, T>
where
    X: HashValueAccess<'de>,
    T: HasDefault<'de, X::KeyAccess>,
{
    type KeyAccess = WrappedHashAccess<'a, X::KeyAccess, T>;
    type Finished = T::Finished<'a>;

    fn next_value_seed<V>(
        self,
        seed: V,
    ) -> DeResult<(V::Value, Continue<Self::KeyAccess, Self::Finished>)>
    where
        V: DeserializeSeed<'de>,
    {
        let index = self.inner.index();
        let trace = self.trace;
        self.inner
            .next_value_seed(Wrapped { inner: seed, trace })
            .inspect_err(|_| trace.push(Context::HashValue(index)))
            .map(|(k, a)| (k, WrappedHashAccess::next_access(trace, a)))
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn index(&self) -> usize {
        self.inner.index()
    }
}

#[derive(Debug)]
struct WrappedHashDefault<'trace, X> {
    inner: X,
    trace: &'trace mut Trace,
}

impl<'de, X> HashDefaultAccess<'de> for WrappedHashDefault<'_, X>
where
    X: HashDefaultAccess<'de>,
{
    fn deserialize_default_seed<V>(self, seed: V) -> DeResult<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        add_context!(
            self.inner.deserialize_default_seed(seed),
            self.trace.push(Context::HashDefault)
        )
    }
}

impl<'de, X> ArrayAccess<'de> for Wrapped<'_, X>
where
    X: ArrayAccess<'de>,
{
    fn next_element_seed<T>(&mut self, seed: T) -> DeResult<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        add_context!(
            self.inner.next_element_seed(Wrapped {
                inner: seed,
                trace: self.trace,
            }),
            self.trace.push(Context::ArrayIndex(self.index()))
        )
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn index(&self) -> usize {
        self.inner.index()
    }
}

impl<'de, X> DeserializeSeed<'de> for Wrapped<'_, X>
where
    X: DeserializeSeed<'de>,
{
    type Value = X::Value;

    fn deserialize<D>(self, deserializer: D) -> DeResult<Self::Value>
    where
        D: DeserializerTrait<'de>,
    {
        self.inner
            .deserialize(Deserializer::new(deserializer, self.trace))
    }
}
