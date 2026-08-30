// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use super::Value;
use indexmap::IndexMap;

mod bignum;
mod fixnum;
mod instance;
mod object;
mod rb_string;
mod rb_struct;
mod sym;
mod symbol;
mod userdata;

pub use bignum::{Bignum, BignumRef};
pub use fixnum::Fixnum;
pub use instance::Instance;
pub use object::Object;
pub use rb_string::RbString;
pub use rb_struct::RbStruct;
pub use sym::Sym;
pub use symbol::Symbol;
pub use userdata::Userdata;

/// Shorthand type alias for a ruby array.
pub type RbArray = Vec<Value>;
/// Shorthand type alias for a ruby hash.
pub type RbHash = IndexMap<Value, Value>;

/// A type alias used to represent fields of objects.
/// All objects store a [`Symbol`] to represent the key for instance variable, and we do that here too.
pub type RbFields = IndexMap<Symbol, Value>;

/// Returns `false` if `le_bytes` contains only zero bytes or `is_negative` otherwise, and the size
/// of `le_bytes` excluding trailing zero bytes.
fn get_canonical_le_bytes_info(is_negative: bool, le_bytes: &[u8]) -> (bool, usize) {
    for (i, byte) in le_bytes.iter().copied().enumerate().rev() {
        if byte != 0 {
            return (is_negative, i + 1);
        }
    }
    (false, 0)
}

/// Returns an unambiguous version of the integer represented by the given sign and little-endian
/// bytes.
fn canonicalize_le_bytes_ref(is_negative: bool, le_bytes: &[u8]) -> (bool, &[u8]) {
    let (is_negative, le_bytes_size) = get_canonical_le_bytes_info(is_negative, le_bytes);
    (is_negative, &le_bytes[..le_bytes_size])
}

/// Returns an unambiguous version of the integer represented by the given sign and little-endian
/// bytes.
fn canonicalize_le_bytes_vec(is_negative: bool, mut le_bytes: Vec<u8>) -> (bool, Vec<u8>) {
    let (is_negative, le_bytes_size) = get_canonical_le_bytes_info(is_negative, &le_bytes);
    le_bytes.truncate(le_bytes_size);
    (is_negative, le_bytes)
}
