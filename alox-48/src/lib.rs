#![warn(rust_2018_idioms, clippy::all, clippy::pedantic)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::panicking_unwrap,
    clippy::all
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
//! However, alox-48 does NOT support object links. Object links are marshal's way of saving space,
//! if an object was serialized already a "link" indicating when it was serialized is serialized instead.
//!
//! ```rb
//! class MyClass
//!  def initialize
//!    @var = 1
//!    @string = "hiya!"
//!  end
//! end
//!
//! a = MyClass.new
//! Marshal.dump([a, a, a])
//! # The array here has 3 indices all "pointing" to the same object.
//! # Instead of serializing MyClass 3 times, Marshal will serialize it once and replace the other 2 occurences with object links.
//! # When deserializing, Marshal will preserve object links and all 3 elements in the array will point to the same object.
//! # In alox-48, this is not the case. Each index will be a "unique" ""object"".
//! ```
//!
//! This behavior could be simulated with `Rc` and/or `Arc` like `thurgood`, however for the sake of ergonomics (and memory cycles)
//! alox-48 deserializes object links as copies instead. alox-48 does not serialize object links at all.
//!
//! Some common terminology:
//! - ivar: Instance variable. These are variables that are attached to an object.
//! - instance: Not to be confused with a class instance. This is a value that is not an object with attached ivars.
//! - userdata: A special type of object that is serialized by the `_dump` method.
//! - userclass: A subclass of a ruby object like `Hash` or `Array`.
//! - object: A generic ruby object. Can be anything from a string to an instance of a class.

// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// A convenience module for getting exact details about where an error occurred.
pub mod path_to_error;

pub(crate) mod tag;

/// Marshal Deserialization framework and Deserializer.
pub mod de;
/// Marshal Serialization framework and Serializer.
pub mod ser;

mod value;
pub use value::{from_value, to_value, Serializer as ValueSerializer, Value};

mod rb_types;
#[doc(inline)]
pub use rb_types::{
    Bignum, BignumRef, Fixnum, Instance, Object, RbArray, RbFields, RbHash, RbString, RbStruct,
    Sym, Symbol, Userdata,
};

#[doc(inline)]
pub use de::{
    ArrayAccess, Deserialize, Deserializer, DeserializerTrait, Error as DeError, HashAccess,
    InstanceAccess, IvarAccess, Result as DeResult, Visitor, VisitorInstance, VisitorOption,
};
#[doc(inline)]
pub use ser::{
    ByteString as SerializeByteString, Error as SerError, Result as SerResult, Serialize,
    SerializeArray, SerializeHash, SerializeIvars, Serializer, SerializerTrait,
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
fn int_to_le_bytes<T>(int: T) -> (bool, <T as num_traits::ToBytes>::Bytes)
where
    T: num_traits::ToBytes + num_traits::Signed + num_traits::WrappingNeg,
{
    let is_negative = int.is_negative();
    let le_bytes = if is_negative { int.wrapping_neg() } else { int }.to_le_bytes();
    (is_negative, le_bytes)
}

#[cfg(test)]
mod fixnum {
    use crate::{int_to_le_bytes, Fixnum};
    use num_traits::{FromPrimitive, ToPrimitive};

    #[test]
    fn i64_positive() {
        let int: i64 = 0x3fffffff;

        let fixnum = Fixnum::from_i64(int).unwrap();

        assert_eq!(int, fixnum.into())
    }

    #[test]
    fn i64_negative() {
        let int: i64 = -0x40000000;

        let fixnum = Fixnum::from_i64(int).unwrap();

        assert_eq!(int, fixnum.into())
    }

    #[test]
    fn u64() {
        let int: u64 = 0x3fffffff;

        let fixnum = Fixnum::from_u64(int).unwrap();

        assert_eq!(Some(int), fixnum.to_u64())
    }

    #[test]
    fn from_le_bytes_positive() {
        let int: i64 = 0x3fffffff;
        let (is_negative, le_bytes) = int_to_le_bytes(int);

        let fixnum = Fixnum::from_le_bytes(is_negative, &le_bytes).unwrap();

        assert_eq!(int, fixnum.into());
    }

    #[test]
    fn from_le_bytes_negative() {
        let int: i64 = -0x40000000;
        let (is_negative, le_bytes) = int_to_le_bytes(int);

        let fixnum = Fixnum::from_le_bytes(is_negative, &le_bytes).unwrap();

        assert_eq!(int, fixnum.into());
    }

    #[test]
    fn i64_positive_out_of_range() {
        let int: i64 = 0x40000000;

        assert!(Fixnum::from_i64(int).is_none());
    }

    #[test]
    fn i64_negative_out_of_range() {
        let int: i64 = -0x40000001;

        assert!(Fixnum::from_i64(int).is_none());
    }

    #[test]
    fn u64_out_of_range() {
        let int: u64 = 0x40000000;

        assert!(Fixnum::from_u64(int).is_none());
    }

    #[test]
    fn from_le_bytes_positive_out_of_range() {
        let int: i64 = 0x40000000;
        let (is_negative, le_bytes) = int_to_le_bytes(int);

        assert!(Fixnum::from_le_bytes(is_negative, &le_bytes).is_none());
    }

    #[test]
    fn from_le_bytes_negative_out_of_range() {
        let int: i64 = -0x40000001;
        let (is_negative, le_bytes) = int_to_le_bytes(int);

        assert!(Fixnum::from_le_bytes(is_negative, &le_bytes).is_none());
    }

    #[test]
    fn i64_max() {
        let int = i64::MAX;

        assert!(Fixnum::from_i64(int).is_none());
    }

    #[test]
    fn i64_min() {
        let int = i64::MIN;

        assert!(Fixnum::from_i64(int).is_none());
    }

    #[test]
    fn u64_max() {
        let int = u64::MAX;

        assert!(Fixnum::from_u64(int).is_none());
    }

    #[test]
    fn from_le_bytes_max() {
        let int = i64::MAX;
        let (is_negative, le_bytes) = int_to_le_bytes(int);

        assert!(Fixnum::from_le_bytes(is_negative, &le_bytes).is_none());
    }

    #[test]
    fn from_le_bytes_min() {
        let int = i64::MIN;
        let (is_negative, le_bytes) = int_to_le_bytes(int);

        assert!(Fixnum::from_le_bytes(is_negative, &le_bytes).is_none());
    }
}

#[cfg(test)]
mod bignum {
    use crate::{int_to_le_bytes, Bignum};
    use num_traits::{FromPrimitive, ToPrimitive};

    #[test]
    fn i64_positive() {
        let int: i64 = 0x40000000;

        let bignum = Bignum::from_i64(int).unwrap();

        assert_eq!(Some(int), bignum.to_i64());
    }

    #[test]
    fn i64_negative() {
        let int: i64 = -0x40000001;

        let bignum = Bignum::from_i64(int).unwrap();

        assert_eq!(Some(int), bignum.to_i64());
    }

    #[test]
    fn u64() {
        let int: u64 = 0x40000000;

        let bignum = Bignum::from_u64(int).unwrap();

        assert_eq!(Some(int), bignum.to_u64());
    }

    #[test]
    fn i64_max() {
        let int = i64::MAX;

        let bignum = Bignum::from_i64(int).unwrap();

        assert_eq!(Some(int), bignum.to_i64());
    }

    #[test]
    fn i64_min() {
        let int = i64::MIN;

        let bignum = Bignum::from_i64(int).unwrap();

        assert_eq!(Some(int), bignum.to_i64());
    }

    #[test]
    fn u64_max() {
        let int = u64::MAX;

        let bignum = Bignum::from_u64(int).unwrap();

        assert_eq!(Some(int), bignum.to_u64());
    }

    #[test]
    fn i128_positive() {
        let int: i128 = 0x40000000;

        let bignum = Bignum::from_i128(int).unwrap();

        assert_eq!(Some(int), bignum.to_i128());
    }

    #[test]
    fn i128_negative() {
        let int: i128 = -0x40000001;

        let bignum = Bignum::from_i128(int).unwrap();

        assert_eq!(Some(int), bignum.to_i128());
    }

    #[test]
    fn u128() {
        let int: u128 = 0x40000000;

        let bignum = Bignum::from_u128(int).unwrap();

        assert_eq!(Some(int), bignum.to_u128());
    }

    #[test]
    fn i128_max() {
        let int = i128::MAX;

        let bignum = Bignum::from_i128(int).unwrap();

        assert_eq!(Some(int), bignum.to_i128());
    }

    #[test]
    fn i128_min() {
        let int = i128::MIN;

        let bignum = Bignum::from_i128(int).unwrap();

        assert_eq!(Some(int), bignum.to_i128());
    }

    #[test]
    fn u128_max() {
        let int = u128::MAX;

        let bignum = Bignum::from_u128(int).unwrap();

        assert_eq!(Some(int), bignum.to_u128());
    }

    #[test]
    fn f64_positive() {
        let int: f64 = 0x40000000 as _;

        let bignum = Bignum::from_f64(int).unwrap();

        assert_eq!(Some(int), bignum.to_f64());
    }

    #[test]
    fn f64_negative() {
        let int: f64 = -0x40000001 as _;

        let bignum = Bignum::from_f64(int).unwrap();

        assert_eq!(Some(int), bignum.to_f64());
    }

    #[test]
    fn f64_max() {
        let int = f64::MAX;

        let bignum = Bignum::from_f64(int).unwrap();

        assert_eq!(Some(int), bignum.to_f64());
    }

    #[test]
    fn f64_min() {
        let int = f64::MIN;

        let bignum = Bignum::from_f64(int).unwrap();

        assert_eq!(Some(int), bignum.to_f64());
    }

    #[test]
    fn f64_mantissa_overflow() {
        let int = u128::MAX;

        let bignum = Bignum::from_u128(int).unwrap();

        assert_eq!(Some(3.402823669209385e38), bignum.to_f64());
    }

    #[test]
    fn f64_infinity() {
        let int = f64::INFINITY;

        assert!(Bignum::from_f64(int).is_none());
    }

    #[test]
    fn f64_neg_infinity() {
        let int = f64::NEG_INFINITY;

        assert!(Bignum::from_f64(int).is_none());
    }

    #[test]
    fn f64_nan() {
        let int = f64::NAN;

        assert!(Bignum::from_f64(int).is_none());
    }

    #[test]
    fn as_le_bytes_positive() {
        let int: i128 = 0x4d3c2b1a;
        let (is_negative, le_bytes) = int_to_le_bytes(int);

        let bignum = Bignum::from_le_bytes(is_negative, le_bytes.to_vec()).unwrap();

        let expected_is_negative = false;
        let expected_le_bytes: &[_] = &[0x1a, 0x2b, 0x3c, 0x4d];

        assert_eq!(
            bignum.as_ref().as_le_bytes(),
            (expected_is_negative, expected_le_bytes),
        );
        assert_eq!(
            bignum.as_le_bytes(),
            (expected_is_negative, expected_le_bytes),
        );
        assert_eq!(
            bignum.to_le_bytes(),
            (expected_is_negative, expected_le_bytes.to_vec()),
        );
    }

    #[test]
    fn as_le_bytes_negative() {
        let int: i128 = -0x4d3c2b1a;
        let (is_negative, le_bytes) = int_to_le_bytes(int);

        let bignum = Bignum::from_le_bytes(is_negative, le_bytes.to_vec()).unwrap();

        let expected_is_negative = true;
        let expected_le_bytes: &[_] = &[0x1a, 0x2b, 0x3c, 0x4d];

        assert_eq!(
            bignum.as_ref().as_le_bytes(),
            (expected_is_negative, expected_le_bytes),
        );
        assert_eq!(
            bignum.as_le_bytes(),
            (expected_is_negative, expected_le_bytes),
        );
        assert_eq!(
            bignum.to_le_bytes(),
            (expected_is_negative, expected_le_bytes.to_vec()),
        );
    }

    #[test]
    fn ord_differing_sign() {
        let int1: i64 = -0xffffffffffff;
        let int2: i64 = 0x0000ffffffff;

        let bignum1 = Bignum::from_i64(int1);
        let bignum2 = Bignum::from_i64(int2);

        assert!(bignum1 < bignum2);
    }

    #[test]
    fn ord_same_sign_differing_length() {
        let int1: i64 = 0x0000ffffffff;
        let int2: i64 = 0xffffffffffff;

        let bignum1 = Bignum::from_i64(int1);
        let bignum2 = Bignum::from_i64(int2);

        assert!(bignum1 < bignum2);
    }

    #[test]
    fn ord_same_sign_same_length() {
        let int1: i64 = 0xfffffffffffe;
        let int2: i64 = 0xffffffffffff;

        let bignum1 = Bignum::from_i64(int1);
        let bignum2 = Bignum::from_i64(int2);

        assert!(bignum1 < bignum2);
    }

    #[test]
    fn equality_differing_length() {
        let int1: i64 = 0x40000000;
        let int2: i128 = 0x40000000;

        let bignum1 = Bignum::from_i64(int1);
        let bignum2 = Bignum::from_i128(int2);

        assert_eq!(bignum1, bignum2);
    }
}

#[cfg(test)]
mod ints {
    #[test]
    fn deserialize() {
        let bytes = &[0x04, 0x08, 0x69, 0x19];

        let int: u8 = crate::from_bytes(bytes).unwrap();

        assert_eq!(int, 20);
    }

    #[test]
    fn round_trip() {
        let int = 123;

        let bytes = crate::to_bytes(int).unwrap();

        let int2 = crate::from_bytes(&bytes).unwrap();

        assert_eq!(int, int2);
    }

    #[test]
    fn round_trip_bignum_positive_even_length() {
        let int: i64 = 0x1122334455667788;

        let bytes = crate::to_bytes(int).unwrap();

        let int2 = crate::from_bytes(&bytes).unwrap();

        assert_eq!(int, int2);
    }

    #[test]
    fn round_trip_bignum_negative_even_length() {
        let int: i64 = -0x1122334455667788;

        let bytes = crate::to_bytes(int).unwrap();

        let int2 = crate::from_bytes(&bytes).unwrap();

        assert_eq!(int, int2);
    }

    #[test]
    fn round_trip_bignum_positive_odd_length() {
        let int: i64 = 0x11223344556677;

        let bytes = crate::to_bytes(int).unwrap();

        let int2 = crate::from_bytes(&bytes).unwrap();

        assert_eq!(int, int2);
    }

    #[test]
    fn round_trip_bignum_negative_odd_length() {
        let int: i64 = -0x11223344556677;

        let bytes = crate::to_bytes(int).unwrap();

        let int2 = crate::from_bytes(&bytes).unwrap();

        assert_eq!(int, int2);
    }

    #[test]
    fn round_trip_value() {
        let value = crate::Value::Fixnum(123i16.into());

        let bytes = crate::to_bytes(&value).unwrap();

        let value2: crate::Value = crate::from_bytes(&bytes).unwrap();

        assert_eq!(value, value2);
    }

    #[test]
    fn negatives() {
        let bytes = &[0x04, 0x08, 0x69, 0xfd, 0x1d, 0xf0, 0xfc];

        let int: i32 = crate::from_bytes(bytes).unwrap();

        assert_eq!(int, -200_675);
    }
}

#[cfg(test)]
mod strings {
    #[test]
    fn deserialize() {
        let bytes = &[
            0x04, 0x08, 0x49, 0x22, 0x11, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x74, 0x68, 0x65,
            0x72, 0x65, 0x21, 0x06, 0x3a, 0x06, 0x45, 0x54,
        ];

        let str: &str = crate::from_bytes(bytes).unwrap();

        assert_eq!(str, "hello there!");
    }

    #[test]
    fn round_trip() {
        let str = "round trip!!";

        let bytes = crate::to_bytes(str).unwrap();

        let str2: &str = crate::from_bytes(&bytes).unwrap();

        assert_eq!(str, str2);
    }

    #[test]
    fn weird_encoding() {
        let bytes = &[
            0x04, 0x08, 0x49, 0x22, 0x11, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x74, 0x68, 0x65,
            0x72, 0x65, 0x21, 0x06, 0x3a, 0x0d, 0x65, 0x6e, 0x63, 0x6f, 0x64, 0x69, 0x6e, 0x67,
            0x22, 0x09, 0x42, 0x69, 0x67, 0x35,
        ];

        let str: crate::Instance<crate::RbString> = crate::from_bytes(bytes).unwrap();

        assert_eq!(
            str.encoding().unwrap().as_string().unwrap().data, // this is a mess lol, i should fix it
            "Big5".as_bytes()
        );
    }

    #[test]
    fn weird_encoding_round_trip() {
        let bytes: &[_] = &[
            0x04, 0x08, 0x49, 0x22, 0x11, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x74, 0x68, 0x65,
            0x72, 0x65, 0x21, 0x06, 0x3a, 0x0d, 0x65, 0x6e, 0x63, 0x6f, 0x64, 0x69, 0x6e, 0x67,
            0x22, 0x09, 0x42, 0x69, 0x67, 0x35,
        ];

        let str: crate::Instance<crate::RbString> = crate::from_bytes(bytes).unwrap();

        let bytes2 = crate::to_bytes(&str).unwrap();

        assert_eq!(bytes, bytes2);
    }
}

#[cfg(test)]
mod floats {
    #[test]
    fn deserialize() {
        let bytes = &[0x04, 0x08, 0x66, 0x07, 0x31, 0x35];

        let float: f64 = crate::from_bytes(bytes).unwrap();

        assert!((float - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn round_trip() {
        let float = 20870.15;

        let bytes = crate::to_bytes(float).unwrap();

        let float2: f64 = crate::from_bytes(&bytes).unwrap();

        assert!((float - float2).abs() < f64::EPSILON);
    }

    #[test]
    fn nan() {
        let bytes = &[0x04, 0x08, 0x66, 0x08, 0x6e, 0x61, 0x6e];

        let float: f64 = crate::from_bytes(bytes).unwrap();

        assert!(float.is_nan());
    }

    #[test]
    fn round_trip_nan() {
        let float = f64::NAN;

        let bytes = crate::to_bytes(float).unwrap();

        let float2: f64 = crate::from_bytes(&bytes).unwrap();

        assert!(float.is_nan());
        assert_eq!(
            bytemuck::cast::<_, u64>(float),
            bytemuck::cast::<_, u64>(float2)
        );
    }
}

#[cfg(test)]
mod arrays {
    #[test]
    fn deserialize() {
        let bytes = &[
            0x04, 0x08, 0x5b, 0x0a, 0x69, 0x00, 0x69, 0x06, 0x69, 0x07, 0x69, 0x08, 0x69, 0x09,
        ];

        let ary: Vec<u8> = crate::from_bytes(bytes).unwrap();

        assert_eq!(ary, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn round_trip() {
        let ary = vec!["hi!", "goodbye!", "pain"];

        let bytes = crate::to_bytes(&ary).unwrap();
        let ary2: Vec<&str> = crate::from_bytes(&bytes).unwrap();

        assert_eq!(ary, ary2);
    }
}

mod structs {
    #[test]
    fn deserialize_borrowed() {
        #[derive(alox_48_derive::Deserialize, alox_48_derive::Serialize, PartialEq, Debug)]
        #[marshal(alox_crate_path = "crate")]
        struct Test<'d> {
            field1: bool,
            field2: &'d str,
        }

        let bytes = &[
            0x04, 0x08, 0x6f, 0x3a, 0x09, 0x54, 0x65, 0x73, 0x74, 0x07, 0x3a, 0x0c, 0x40, 0x66,
            0x69, 0x65, 0x6c, 0x64, 0x31, 0x54, 0x3a, 0x0c, 0x40, 0x66, 0x69, 0x65, 0x6c, 0x64,
            0x32, 0x49, 0x22, 0x10, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x06, 0x3a, 0x06, 0x45, 0x54,
        ];

        let obj: Test<'_> = crate::from_bytes(bytes).unwrap();

        assert_eq!(
            obj,
            Test {
                field1: true,
                field2: "hello there"
            }
        );
    }

    #[test]
    fn deserialize_multi_borrowed() {
        #[derive(alox_48_derive::Deserialize, alox_48_derive::Serialize, PartialEq, Debug)]
        #[marshal(alox_crate_path = "crate")]
        struct Test<'a, 'b> {
            field1: bool,
            field2: &'a str,
            #[marshal(byte_string)]
            field3: &'b [u8],
        }

        let initial = Test {
            field1: true,
            field2: "borrowed from the stack",
            field3: b"also borrowed from the stack",
        };

        let bytes = crate::to_bytes(&initial).unwrap();
        let obj: Test<'_, '_> = crate::from_bytes(&bytes).unwrap();

        assert_eq!(obj, initial);
    }

    #[test]
    fn deserialize_multi_bounds() {
        #[derive(alox_48_derive::Deserialize, alox_48_derive::Serialize, PartialEq, Debug)]
        #[marshal(alox_crate_path = "crate")]
        struct Test<'a, 'b: 'a, 'c: 'a + 'b> {
            field1: bool,
            field2: &'a str,
            #[marshal(byte_string)]
            field3: &'b [u8],
            field4: &'c str,
        }

        let initial = Test {
            field1: true,
            field2: "borrowed from the stack",
            field3: b"also borrowed from the stack",
            field4: "multiple bounds",
        };

        let bytes = crate::to_bytes(&initial).unwrap();
        let obj: Test<'_, '_, '_> = crate::from_bytes(&bytes).unwrap();

        assert_eq!(obj, initial);
    }

    #[test]
    fn userdata() {
        #[derive(alox_48_derive::Deserialize, Debug, PartialEq, Eq)]
        #[marshal(alox_crate_path = "crate")]
        #[marshal(from = "crate::Userdata")]
        struct MyUserData {
            field: [char; 4],
        }

        impl From<crate::Userdata> for MyUserData {
            fn from(value: crate::Userdata) -> Self {
                assert_eq!(value.class, "MyUserData");
                let field = std::array::from_fn(|i| value.data[i] as char);
                Self { field }
            }
        }
        let bytes = &[
            0x04, 0x08, 0x75, 0x3a, 0x0f, 0x4d, 0x79, 0x55, 0x73, 0x65, 0x72, 0x44, 0x61, 0x74,
            0x61, 0x09, 0x61, 0x62, 0x63, 0x64,
        ];
        let data: MyUserData = crate::from_bytes(bytes).unwrap();

        assert_eq!(
            data,
            MyUserData {
                field: ['a', 'b', 'c', 'd']
            }
        );
    }
}

#[cfg(test)]
mod misc {
    #[test]
    fn symbol() {
        let sym = crate::Symbol::from("symbol");

        let bytes = crate::to_bytes(&sym).unwrap();

        let sym2: crate::Symbol = crate::from_bytes(&bytes).unwrap();

        assert_eq!(sym, sym2);
    }

    // Testing for zero copy symlink deserialization
    // ALL symbols should be the same reference
    #[test]
    fn symlink() {
        let bytes = &[
            0x04, 0x08, 0x5b, 0x0a, 0x3a, 0x09, 0x74, 0x65, 0x73, 0x74, 0x3b, 0x00, 0x3b, 0x00,
            0x3b, 0x00, 0x3b, 0x00,
        ];

        let symbols: Vec<&str> = crate::from_bytes(bytes).unwrap();

        for sym in symbols.windows(2) {
            assert_eq!(sym[0].as_ptr(), sym[1].as_ptr());
        }
    }
}

#[cfg(test)]
mod value_test {
    #[test]
    fn untyped_object() {
        let bytes = &[
            0x04, 0x08, 0x6f, 0x3a, 0x09, 0x54, 0x65, 0x73, 0x74, 0x07, 0x3a, 0x0c, 0x40, 0x66,
            0x69, 0x65, 0x6c, 0x64, 0x31, 0x54, 0x3a, 0x0c, 0x40, 0x66, 0x69, 0x65, 0x6c, 0x64,
            0x32, 0x49, 0x22, 0x10, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x06, 0x3a, 0x06, 0x45, 0x54,
        ];

        let obj: crate::Value = crate::from_bytes(bytes).unwrap();
        let obj = obj.into_object().unwrap();

        assert_eq!(obj.class, "Test");
        assert_eq!(obj.fields["@field1"], true);
    }

    #[test]
    fn untyped_ivar_string() {
        let bytes = &[
            0x04, 0x08, 0x49, 0x22, 0x0b, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x21, 0x07, 0x3a, 0x06,
            0x45, 0x54, 0x3a, 0x0c, 0x40, 0x72, 0x61, 0x6e, 0x64, 0x6f, 0x6d, 0x69, 0x01, 0x7b,
        ];

        let obj: crate::Value = crate::from_bytes(bytes).unwrap();
        let instance = obj.into_instance().unwrap();

        assert_eq!(instance.value.as_ref(), "hello!");
        assert_eq!(instance.fields["@random"], 123);
    }

    #[test]
    fn untyped_ivar_array() {
        let bytes = &[
            0x04, 0x08, 0x49, 0x5b, 0x07, 0x49, 0x22, 0x09, 0x74, 0x65, 0x73, 0x74, 0x06, 0x3a,
            0x06, 0x45, 0x54, 0x69, 0x01, 0x7b, 0x06, 0x3a, 0x0a, 0x40, 0x69, 0x76, 0x61, 0x72,
            0x66, 0x06, 0x35,
        ];

        let obj: crate::Value = crate::from_bytes(bytes).unwrap();
        let instance = obj.into_instance().unwrap();

        let array = instance.value.as_array().unwrap();
        assert_eq!(&array[0], "test");
        assert_eq!(array[1], 123);
        assert_eq!(instance.fields["@ivar"], 5.0);
    }

    #[test]

    fn untyped_to_borrowed() {
        #[derive(alox_48_derive::Deserialize, alox_48_derive::Serialize, PartialEq, Debug)]
        #[marshal(alox_crate_path = "crate")]
        struct Test<'d> {
            field1: bool,
            field2: &'d str,
        }

        let bytes = &[
            0x04, 0x08, 0x6f, 0x3a, 0x09, 0x54, 0x65, 0x73, 0x74, 0x07, 0x3a, 0x0c, 0x40, 0x66,
            0x69, 0x65, 0x6c, 0x64, 0x31, 0x54, 0x3a, 0x0c, 0x40, 0x66, 0x69, 0x65, 0x6c, 0x64,
            0x32, 0x49, 0x22, 0x10, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x06, 0x3a, 0x06, 0x45, 0x54,
        ];

        let obj: crate::Value = crate::from_bytes(bytes).unwrap();

        let test: Test<'_> = crate::Deserialize::deserialize(&obj).unwrap();

        assert_eq!(
            test,
            Test {
                field1: true,
                field2: "hello there"
            }
        );
    }
}

#[cfg(test)]
mod round_trip {
    use crate::{from_bytes, to_bytes, Instance, RbFields, RbHash, RbStruct, Value};

    #[test]
    fn nil() {
        let original = Value::Nil;

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn bool() {
        let original = Value::Bool(true);

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn float() {
        let original = Value::Float(123.456);

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn integer() {
        let original = Value::Fixnum(123i16.into());

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn integer_bignum_positive_even_length() {
        let original =
            Value::Bignum(num_traits::FromPrimitive::from_i64(0x1122334455667788).unwrap());

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn integer_bignum_negative_even_length() {
        let original =
            Value::Bignum(num_traits::FromPrimitive::from_i64(-0x1122334455667788).unwrap());

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn integer_bignum_positive_odd_length() {
        let original =
            Value::Bignum(num_traits::FromPrimitive::from_i64(0x11223344556677).unwrap());

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn integer_bignum_negative_odd_length() {
        let original =
            Value::Bignum(num_traits::FromPrimitive::from_i64(-0x11223344556677).unwrap());

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn string() {
        let original = Value::String("round trip".into());

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn symbol() {
        let original = Value::Symbol("round_trip".into());

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn array() {
        let original = Value::Array(vec![Value::Fixnum(1i16.into()), Value::Float(256.652)]);

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn hash() {
        let mut hash = RbHash::new();
        hash.insert(Value::Bool(true), Value::Fixnum(1i16.into()));
        hash.insert(Value::Symbol("a_symbol".into()), Value::Float(256.652));
        let original = Value::Hash(hash);

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn userdata() {
        let original = Value::Userdata(crate::Userdata {
            class: "TestUserdata".into(),
            data: vec![97, 98, 99, 100],
        });

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn object() {
        let mut fields = RbFields::new();
        fields.insert("@field1".into(), Value::Bool(true));
        fields.insert(
            "@field2".into(),
            Value::String("i've been round tripped".into()),
        );
        let original = Value::Object(crate::Object {
            class: "Test".into(),
            fields,
        });

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn instance() {
        let inner_value = Box::new(Value::String("I've been round tripped, with ivars!".into()));
        let mut fields = RbFields::new();
        fields.insert("E".into(), Value::Bool(true));
        fields.insert("@round_trip".into(), Value::Fixnum(123i16.into()));
        let original = Value::Instance(Instance {
            value: inner_value,
            fields,
        });

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn regex() {
        let original = Value::Regex {
            data: "/round trip/".into(),
            flags: 0b1010,
        };

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn rb_struct() {
        let mut fields = RbFields::new();
        fields.insert("field1".into(), Value::Bool(true));
        fields.insert("field2".into(), Value::String("round trip".into()));
        let original = Value::RbStruct(RbStruct {
            class: "TestStruct".into(),
            fields,
        });

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn class() {
        let original = Value::Class("TestClass".into());

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn module() {
        let original = Value::Module("TestModule".into());

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn user_class() {
        let inner_value = Box::new(Value::String("I'm a user class".into()));
        let original = Value::UserClass {
            class: "TestUserClass".into(),
            value: inner_value,
        };

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn user_marshal() {
        let inner_value = Box::new(Value::String("I've been serialized as another type".into()));
        let original = Value::UserMarshal {
            class: "TestUserMarshal".into(),
            value: inner_value,
        };

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }

    #[test]
    fn data() {
        let inner_value = Box::new(Value::String("???".into()));
        let original = Value::Data {
            class: "TestData".into(),
            value: inner_value,
        };

        let bytes = to_bytes(&original).unwrap();

        let new: Value = from_bytes(&bytes).unwrap();

        assert_eq!(original, new);
    }
}
