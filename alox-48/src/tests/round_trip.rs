
use crate::{from_bytes, to_bytes, FromPrimitive, Instance, RbFields, RbHash, RbStruct, Value};

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
    let original = Value::Bignum(FromPrimitive::from_i64(0x1122_3344_5566_7788).unwrap());

    let bytes = to_bytes(&original).unwrap();

    let new: Value = from_bytes(&bytes).unwrap();

    assert_eq!(original, new);
}

#[test]
fn integer_bignum_negative_even_length() {
    let original = Value::Bignum(FromPrimitive::from_i64(-0x1122_3344_5566_7788).unwrap());

    let bytes = to_bytes(&original).unwrap();

    let new: Value = from_bytes(&bytes).unwrap();

    assert_eq!(original, new);
}

#[test]
fn integer_bignum_positive_odd_length() {
    let original = Value::Bignum(FromPrimitive::from_i64(0x0011_2233_4455_6677).unwrap());

    let bytes = to_bytes(&original).unwrap();

    let new: Value = from_bytes(&bytes).unwrap();

    assert_eq!(original, new);
}

#[test]
fn integer_bignum_negative_odd_length() {
    let original = Value::Bignum(FromPrimitive::from_i64(-0x0011_2233_4455_6677).unwrap());

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
