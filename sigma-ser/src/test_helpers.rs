//! Helper function for testing purposes
/// serialization roundtrip
pub fn sigma_serialize_roundtrip<T: crate::serializer::SigmaSerializable>(v: &T) -> T {
    let mut data = Vec::new();
    v.sigma_serialize(&mut data).expect("serialization failed");
    T::sigma_parse(&data[..]).expect("parse failed")
}
