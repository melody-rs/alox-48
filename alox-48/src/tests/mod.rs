#![allow(clippy::panic)]

fn int_to_le_bytes<T>(int: T) -> (bool, <T as num_traits::ToBytes>::Bytes)
where
    T: num_traits::ToBytes + num_traits::Signed + num_traits::WrappingNeg,
{
    let is_negative = int.is_negative();
    let le_bytes = if is_negative { int.wrapping_neg() } else { int }.to_le_bytes();
    (is_negative, le_bytes)
}

fn marshal_output_for(code: &str) -> Vec<u8> {
    let ruby_output = std::process::Command::new("ruby")
        .arg("-e")
        .arg(code)
        .output()
        .unwrap();

    if ruby_output.stdout.get(..2) != Some(&[0x4, 0x8]) {
        let stdout = String::from_utf8_lossy(&ruby_output.stdout);
        let stderr = String::from_utf8_lossy(&ruby_output.stderr);
        panic!(
            "ruby command did not output marshal bytes (did you print it with print?)\n
        stdout: {stdout}\n
        stderr: {stderr}"
        );
    }

    ruby_output.stdout
}

mod fixnum;

mod bignum;

mod ints;

mod strings;

mod floats;
mod arrays {
    #[test]
    fn deserialize() {
        let bytes = &[
            0x04, 0x08, 0x5b, 0x0a, 0x69, 0x00, 0x69, 0x06, 0x69, 0x07, 0x69, 0x08, 0x69, 0x09,
        ];

        let ary: Vec<u8> = crate::from_bytes(bytes).unwrap();

        assert_eq!(ary, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn round_trip() {
        let ary = vec!["hi!", "goodbye!", "pain"];

        let bytes = crate::to_bytes(&ary).unwrap();
        let ary2: Vec<&str> = crate::from_bytes(&bytes).unwrap();

        assert_eq!(ary, ary2);
    }
}

mod structs;
#[cfg(test)]
mod misc {
    #[test]
    fn symbol() {
        let sym = crate::Symbol::from("symbol");

        let bytes = crate::to_bytes(&sym).unwrap();

        let sym2: crate::Symbol = crate::from_bytes(&bytes).unwrap();

        assert_eq!(sym, sym2);
    }

    // Testing for zero copy symlink deserialization
    // ALL symbols should be the same reference
    #[test]
    fn symlink() {
        let bytes = &[
            0x04, 0x08, 0x5b, 0x0a, 0x3a, 0x09, 0x74, 0x65, 0x73, 0x74, 0x3b, 0x00, 0x3b, 0x00,
            0x3b, 0x00, 0x3b, 0x00,
        ];

        let symbols: Vec<&str> = crate::from_bytes(bytes).unwrap();

        for sym in symbols.windows(2) {
            assert_eq!(sym[0].as_ptr(), sym[1].as_ptr());
        }
    }
}

mod value;

mod round_trip;
