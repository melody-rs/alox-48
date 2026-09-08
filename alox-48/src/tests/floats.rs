use crate::tests::marshal_output_for;

#[test]
fn deserialize() {
    let bytes = marshal_output_for("print Marshal.dump(15.0)");

    let float: f64 = crate::from_bytes(&bytes).unwrap();

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
    let bytes = marshal_output_for("print Marshal.dump(Float::NAN)");

    let float: f64 = crate::from_bytes(&bytes).unwrap();

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
