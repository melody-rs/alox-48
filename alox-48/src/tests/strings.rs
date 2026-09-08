use crate::tests::marshal_output_for;

#[test]
fn deserialize() {
    let bytes = marshal_output_for("print Marshal.dump('hello there!')");

    let str: &str = crate::from_bytes(&bytes).unwrap();

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
    let bytes = marshal_output_for("print Marshal.dump('wawa'.encode('Big5'))");

    let str: crate::Instance<crate::RbString> = crate::from_bytes(&bytes).unwrap();

    assert_eq!(
        str.encoding().unwrap().as_string().unwrap().data, // this is a mess lol, i should fix it
        "Big5".as_bytes()
    );
}

#[test]
fn weird_encoding_round_trip() {
    let bytes = marshal_output_for("print Marshal.dump('wawa'.encode('Big5'))");

    let str: crate::Instance<crate::RbString> = crate::from_bytes(&bytes).unwrap();

    let bytes2 = crate::to_bytes(&str).unwrap();

    assert_eq!(bytes, bytes2);
}
