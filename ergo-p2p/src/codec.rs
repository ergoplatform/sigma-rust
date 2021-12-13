use bytes::BytesMut;
use sigma_ser::ScorexSerializationError;
use tokio_util::codec::Encoder;

use crate::message::Request;

/// Encoder/Decoder for network messages from/to bytes
#[derive(Default)]
pub struct Codec {}

impl Encoder<Request> for Codec {
    type Error = ScorexSerializationError;

    fn encode(&mut self, item: Request, dst: &mut BytesMut) -> Result<(), Self::Error> {
        todo!()
    }
}
