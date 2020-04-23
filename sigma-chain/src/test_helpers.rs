use sigma_ser::serializer::SigmaSerializable;

#[cfg(test)]
pub fn sigma_serialize_roundtrip<T: SigmaSerializable>(v: &T) -> T {
    let mut data = Vec::new();
    v.sigma_serialize(&mut data).expect("serialization failed");
    T::sigma_parse(&data[..]).expect("parse failed")
}
