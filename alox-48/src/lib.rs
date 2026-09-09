#![warn(rust_2018_idioms, clippy::all, clippy::pedantic)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::panicking_unwrap
)]
#![allow(
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_possible_wrap,
    unexpected_cfgs
)]

//! alox-48
//! (short for aluminum oxide 48)
//!
//! alox-48 supports both full serialization and deserialization of Marshal, but generally users of this library will not be using
//! most of Marshal's features. (Classes, Extended types, etc)
//!
//! Some common terminology:
//! - ivar: Instance variable. These are variables that are attached to an object.
//! - object reference: A shared reference to the same object.
//! - instance: This is a builtin ruby object (strings, arrays, hashes, ints) with attached ivars.
//! - userdata: A special type of object that is serialized by the `_dump` method.
//! - userclass: A subclass of a builtin ruby object like `Hash` or `Array`.
//! - object: A generic ruby object. Can be anything from a string to an instance of a class.
//! - bignum: An arbitrary precision integer. Always larger than ±2<sup>30</sup>.
//! - fixnum: An integer smaller than ±2<sup>30</sup>.
//!
//! ### Object links
//!
//! Object links are used for compression and to represent reference cycles.
//! If the same instance of an object shows up twice in Marshal, it'll instead be serialized as a pointer to the object.
//!
//! alox-48 does *not* support serializing object links.
//!
//! alox-48 *does* support deserializing object links when they aren't cyclic, the deserializer will wind back to
//! where the object first appeared and deserialize it again, effectively copying the object.
//! Reference cycles throw a deserializer error as this behavior would blow up the stack. (Rust isn't great at cyclic data structures anyway.)
//!
//! For example:
//! ```rb
//! str = "hello!"
//! marsh = Marshal.dump([str, str, str])
//! puts Marshal.load(marsh)
//! ```
//! Ruby would deserialize this as an array of 3 references to the same `"hello!"`.
//!
//! (`["hello!", ptr->"hello!", ptr->"hello!"]`)
//!
//! alox-48 would deserialize this as array of 3 unique strings.
//!
//! (`[String("hello!"), String("hello!"), String("hello!")]`)
//!
//! For most cases, this behavior is exactly what you want, but if you really need multiple references to the same object,
//! a different crate might be better for you.
//!
//! This limitation may be lifted in the future, though!
//!
//! ### Bignums & Fixnums
//!
//! Marshal serializes any integer smaller than ±2<sup>30</sup> as a [`Fixnum`], which is just a plain `i32`.
//! Larger integers are serialized as a [`Bignum`], which is an arbitrary precision integer.
//! `alox-48` will automatically convert [`Bignum`]s and [`Fixnum`]s to normal Rust integer types, so generally you don't
//! need to interact with them at all.
//!
//! *However*, array lengths, string lengths, and hash sizes are all serialized as [`Fixnum`]s. If you need to serialize an array
//! with more than ±2<sup>30</sup> elements, you're out of luck :(
//!
//! Fortunately, this realistically only matters for byte arrays- any types larger than a `i64` are going to quickly take up more
//! space than available RAM.

#![allow(missing_docs)]

// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub use num_traits::{cast::cast, FromPrimitive, NumCast, ToPrimitive};

/// A convenience module for getting exact details about where an error occurred.
pub mod path_to_error;

pub(crate) mod tag;

/// Marshal Deserialization framework and Deserializer.
pub mod de;
/// Marshal Serialization framework and Serializer.
pub mod ser;

mod value;
pub use value::{from_value, to_value, Serializer as ValueSerializer, Value};

#[derive(Debug)]
pub enum Continue<Next, Finished> {
    Next(Next),
    Finished(Finished),
}

impl<N, F> Continue<N, F> {
    pub fn is_finished(&self) -> bool {
        matches!(self, Continue::Finished(_))
    }

    pub fn map_next<U>(self, map_fn: impl FnOnce(N) -> U) -> Continue<U, F> {
        match self {
            Self::Next(n) => Continue::Next(map_fn(n)),
            Self::Finished(f) => Continue::Finished(f),
        }
    }

    pub fn map_finished<U>(self, map_fn: impl FnOnce(F) -> U) -> Continue<N, U> {
        match self {
            Self::Next(n) => Continue::Next(n),
            Self::Finished(f) => Continue::Finished(map_fn(f)),
        }
    }
}

mod rb_types;
#[doc(inline)]
pub use rb_types::{
    Bignum, BignumRef, Fixnum, Instance, Object, RbArray, RbFields, RbHash, RbString, RbStruct,
    Sym, Symbol, Userdata,
};

#[doc(inline)]
pub use de::{
    ArrayAccess, Deserialize, DeserializeSeed, Deserializer, DeserializerTrait, Error as DeError,
    HashDefaultAccess, HashKeyAccess, HashValueAccess, InstanceAccess, IvarAccess,
    Result as DeResult, Visitor, VisitorInstance, VisitorOption,
};
#[doc(inline)]
pub use ser::{
    ByteString as SerializeByteString, Error as SerError, Result as SerResult, Serialize,
    SerializeArray, SerializeHashDefault, SerializeHashKey, SerializeHashValue, SerializeIvars,
    Serializer, SerializerTrait,
};

#[cfg(feature = "derive")]
#[doc(inline)]
pub use alox_48_derive::{Deserialize, Serialize};

/// Deserialize data from some bytes.
/// It's a convenience function over [`Deserializer::new`] and [`Deserialize::deserialize`].
#[allow(clippy::missing_errors_doc)]
pub fn from_bytes<'de, T>(data: &'de [u8]) -> Result<T, DeError>
where
    T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(data)?;
    T::deserialize(&mut deserializer)
}

/// Serialize the type into bytes.
///
/// # Errors
///
/// Serialization errors are uncommon, and generally result from improper `Serialize` implementations or a bug in alox-48.
pub fn to_bytes<T>(data: T) -> Result<Vec<u8>, SerError>
where
    T: Serialize,
{
    let mut serializer = Serializer::new();
    data.serialize(&mut serializer)?;
    Ok(serializer.output)
}

#[cfg(test)]
mod tests;
