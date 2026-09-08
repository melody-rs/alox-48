
use super::int_to_le_bytes;
use crate::{Bignum, FromPrimitive, ToPrimitive};

#[test]
fn i64_positive() {
    let int: i64 = 0x4000_0000;

    let bignum = Bignum::from_i64(int).unwrap();

    assert_eq!(Some(int), bignum.to_i64());
}

#[test]
fn i64_negative() {
    let int: i64 = -0x4000_0001;

    let bignum = Bignum::from_i64(int).unwrap();

    assert_eq!(Some(int), bignum.to_i64());
}

#[test]
fn u64() {
    let int: u64 = 0x4000_0000;

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
    let int: i128 = 0x4000_0000;

    let bignum = Bignum::from_i128(int).unwrap();

    assert_eq!(Some(int), bignum.to_i128());
}

#[test]
fn i128_negative() {
    let int: i128 = -0x4000_0001;

    let bignum = Bignum::from_i128(int).unwrap();

    assert_eq!(Some(int), bignum.to_i128());
}

#[test]
fn u128() {
    let int: u128 = 0x4000_0000;

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
    let int: f64 = 0x4000_0000 as _;

    let bignum = Bignum::from_f64(int).unwrap();

    assert_eq!(Some(int), bignum.to_f64());
}

#[test]
fn f64_negative() {
    let int: f64 = -0x4000_0001 as _;

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

    assert_eq!(Some(3.402_823_669_209_385e38), bignum.to_f64());
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
    let int: i128 = 0x4d3c_2b1a;
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
    let int: i128 = -0x4d3c_2b1a;
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
    let int1: i64 = -0xffff_ffff_ffff;
    let int2: i64 = 0x0000_ffff_ffff;

    let bignum1 = Bignum::from_i64(int1);
    let bignum2 = Bignum::from_i64(int2);

    assert!(bignum1 < bignum2);
}

#[test]
fn ord_same_sign_differing_length() {
    let int1: i64 = 0x0000_ffff_ffff;
    let int2: i64 = 0xffff_ffff_ffff;

    let bignum1 = Bignum::from_i64(int1);
    let bignum2 = Bignum::from_i64(int2);

    assert!(bignum1 < bignum2);
}

#[test]
fn ord_same_sign_same_length() {
    let int1: i64 = 0xffff_ffff_fffe;
    let int2: i64 = 0xffff_ffff_ffff;

    let bignum1 = Bignum::from_i64(int1);
    let bignum2 = Bignum::from_i64(int2);

    assert!(bignum1 < bignum2);
}

#[test]
fn equality_differing_length() {
    let int1: i64 = 0x4000_0000;
    let int2: i128 = 0x4000_0000;

    let bignum1 = Bignum::from_i64(int1);
    let bignum2 = Bignum::from_i128(int2);

    assert_eq!(bignum1, bignum2);
}
