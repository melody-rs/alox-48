
#[test]
fn deserialize_borrowed() {
    #[derive(alox_48_derive::Deserialize, alox_48_derive::Serialize, PartialEq, Debug)]
    #[marshal(alox_crate_path = "crate")]
    struct Test<'d> {
        field1: bool,
        field2: &'d str,
    }

    let bytes = &[
        0x04, 0x08, 0x6f, 0x3a, 0x09, 0x54, 0x65, 0x73, 0x74, 0x07, 0x3a, 0x0c, 0x40, 0x66, 0x69,
        0x65, 0x6c, 0x64, 0x31, 0x54, 0x3a, 0x0c, 0x40, 0x66, 0x69, 0x65, 0x6c, 0x64, 0x32, 0x49,
        0x22, 0x10, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x74, 0x68, 0x65, 0x72, 0x65, 0x06, 0x3a,
        0x06, 0x45, 0x54,
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
        0x04, 0x08, 0x75, 0x3a, 0x0f, 0x4d, 0x79, 0x55, 0x73, 0x65, 0x72, 0x44, 0x61, 0x74, 0x61,
        0x09, 0x61, 0x62, 0x63, 0x64,
    ];
    let data: MyUserData = crate::from_bytes(bytes).unwrap();

    assert_eq!(
        data,
        MyUserData {
            field: ['a', 'b', 'c', 'd']
        }
    );
}
