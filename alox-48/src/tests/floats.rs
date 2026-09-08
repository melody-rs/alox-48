
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
