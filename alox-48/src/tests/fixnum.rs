use super::int_to_le_bytes;
use crate::{Fixnum, FromPrimitive, ToPrimitive};

#[test]
fn i64_positive() {
    let int: i64 = 0x3fff_ffff;

    let fixnum = Fixnum::from_i64(int).unwrap();

    assert_eq!(int, fixnum.into());
}

#[test]
fn i64_negative() {
    let int: i64 = -0x4000_0000;

    let fixnum = Fixnum::from_i64(int).unwrap();

    assert_eq!(int, fixnum.into());
}

#[test]
fn u64() {
    let int: u64 = 0x3fff_ffff;

    let fixnum = Fixnum::from_u64(int).unwrap();

    assert_eq!(Some(int), fixnum.to_u64());
}

#[test]
fn from_le_bytes_positive() {
    let int: i64 = 0x3fff_ffff;
    let (is_negative, le_bytes) = int_to_le_bytes(int);

    let fixnum = Fixnum::from_le_bytes(is_negative, &le_bytes).unwrap();

    assert_eq!(int, fixnum.into());
}

#[test]
fn from_le_bytes_negative() {
    let int: i64 = -0x4000_0000;
    let (is_negative, le_bytes) = int_to_le_bytes(int);

    let fixnum = Fixnum::from_le_bytes(is_negative, &le_bytes).unwrap();

    assert_eq!(int, fixnum.into());
}

#[test]
fn i64_positive_out_of_range() {
    let int: i64 = 0x4000_0000;

    assert!(Fixnum::from_i64(int).is_none());
}

#[test]
fn i64_negative_out_of_range() {
    let int: i64 = -0x4000_0001;

    assert!(Fixnum::from_i64(int).is_none());
}

#[test]
fn u64_out_of_range() {
    let int: u64 = 0x4000_0000;

    assert!(Fixnum::from_u64(int).is_none());
}

#[test]
fn from_le_bytes_positive_out_of_range() {
    let int: i64 = 0x4000_0000;
    let (is_negative, le_bytes) = int_to_le_bytes(int);

    assert!(Fixnum::from_le_bytes(is_negative, &le_bytes).is_none());
}

#[test]
fn from_le_bytes_negative_out_of_range() {
    let int: i64 = -0x4000_0001;
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
