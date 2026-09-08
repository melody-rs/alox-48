use crate::tests::marshal_output_for;

#[test]
fn deserialize() {
    let bytes = marshal_output_for("print Marshal.dump(20)");

    let int: u8 = crate::from_bytes(&bytes).unwrap();

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
    let int: i64 = 0x1122_3344_5566_7788;

    let bytes = crate::to_bytes(int).unwrap();

    let int2 = crate::from_bytes(&bytes).unwrap();

    assert_eq!(int, int2);
}

#[test]
fn round_trip_bignum_negative_even_length() {
    let int: i64 = -0x1122_3344_5566_7788;

    let bytes = crate::to_bytes(int).unwrap();

    let int2 = crate::from_bytes(&bytes).unwrap();

    assert_eq!(int, int2);
}

#[test]
fn round_trip_bignum_positive_odd_length() {
    let int: i64 = 0x0011_2233_4455_6677;

    let bytes = crate::to_bytes(int).unwrap();

    let int2 = crate::from_bytes(&bytes).unwrap();

    assert_eq!(int, int2);
}

#[test]
fn round_trip_bignum_negative_odd_length() {
    let int: i64 = -0x0011_2233_4455_6677;

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
    let bytes = marshal_output_for("print Marshal.dump(-200_675)");

    let int: i32 = crate::from_bytes(&bytes).unwrap();

    assert_eq!(int, -200_675);
}
