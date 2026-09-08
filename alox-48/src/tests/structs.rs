use crate::tests::marshal_output_for;

#[test]
fn deserialize_borrowed_class() {
    #[derive(alox_48_derive::Deserialize, alox_48_derive::Serialize, PartialEq, Debug)]
    #[marshal(alox_crate_path = "crate")]
    #[marshal(class = "Test")]
    struct Test<'d> {
        field1: bool,
        field2: &'d str,
    }

    let bytes = marshal_output_for(
        "
    class Test
        def initialize
            @field1 = true
            @field2 = 'hello there'
        end
    end
    print Marshal.dump(Test.new)
    ",
    );

    let obj: Test<'_> = crate::from_bytes(&bytes).unwrap();

    assert_eq!(
        obj,
        Test {
            field1: true,
            field2: "hello there"
        }
    );
}

#[test]
fn deserialize_borrowed_struct() {
    #[derive(alox_48_derive::Deserialize, alox_48_derive::Serialize, PartialEq, Debug)]
    #[marshal(alox_crate_path = "crate")]
    #[marshal(class = "Struct::Test", is_struct)]
    struct Test<'d> {
        field1: bool,
        field2: &'d str,
    }

    let bytes = marshal_output_for(
        "
    Test = Struct.new('Test', :field1, :field2)
    print Marshal.dump(Test.new(true, 'hello there'))
    ",
    );

    let obj: Test<'_> = crate::from_bytes(&bytes).unwrap();

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

    let bytes = marshal_output_for(
        "
    class MyUserData
        def _dump(limit)
            'abcd'
        end
    end
    print Marshal.dump(MyUserData.new)
    ",
    );

    let data: MyUserData = crate::from_bytes(&bytes).unwrap();

    assert_eq!(
        data,
        MyUserData {
            field: ['a', 'b', 'c', 'd']
        }
    );
}
