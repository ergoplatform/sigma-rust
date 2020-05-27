//! Helper function for testing purposes
use crate::peekable_reader::PeekableReader;
use std::io::Cursor;

/// serialization roundtrip
pub fn sigma_serialize_roundtrip<T: crate::serializer::SigmaSerializable>(v: &T) -> T {
    let mut data = Vec::new();
    v.sigma_serialize(&mut data).expect("serialization failed");
    let cursor = Cursor::new(&mut data[..]);
    let mut reader = PeekableReader::new(cursor);
    T::sigma_parse(&mut reader).expect("parse failed")
}
