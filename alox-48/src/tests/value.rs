use crate::tests::marshal_output_for;

#[test]
fn untyped_object() {
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

    let obj: crate::Value = crate::from_bytes(&bytes).unwrap();
    let obj = obj.into_object().unwrap();

    assert_eq!(obj.class, "Test");
    assert_eq!(obj.fields["@field1"], true);
}

#[test]
fn untyped_ivar_string() {
    let bytes = marshal_output_for(
        "
    str = 'hello!'
    str.instance_variable_set(:@random, 123)
    print Marshal.dump(str)
    ",
    );

    let obj: crate::Value = crate::from_bytes(&bytes).unwrap();
    let instance = obj.into_instance().unwrap();

    assert_eq!(instance.value.as_ref(), "hello!");
    assert_eq!(instance.fields["@random"], 123);
}

#[test]
fn untyped_ivar_array() {
    let bytes = marshal_output_for(
        "
    arr = ['test', 123]
    arr.instance_variable_set(:@ivar, 5.0)
    print Marshal.dump(arr)
    ",
    );

    let obj: crate::Value = crate::from_bytes(&bytes).unwrap();
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

    let obj: crate::Value = crate::from_bytes(&bytes).unwrap();

    let test: Test<'_> = crate::Deserialize::deserialize(&obj).unwrap();

    assert_eq!(
        test,
        Test {
            field1: true,
            field2: "hello there"
        }
    );
}
