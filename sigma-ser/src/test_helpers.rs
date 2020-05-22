//! Helper function for testing purposes
use std::io::Cursor;

/// serialization roundtrip
pub fn sigma_serialize_roundtrip<T: crate::serializer::SigmaSerializable>(v: &T) -> T {
    let mut data = Vec::new();
    v.sigma_serialize(&mut data).expect("serialization failed");
    let mut bytes = data.clone();
    let mut cursor = Cursor::new(&mut bytes[..]);
    T::sigma_parse(&mut cursor).expect("parse failed")
}
