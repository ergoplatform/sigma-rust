//! Ergo P2P network version
use sigma_ser::{ScorexSerializable, ScorexSerializeResult};

/// P2P network protocol version
pub struct ProtocolVersion {
    first_digit: u8,
    second_digit: u8,
    third_digit: u8,
}

impl ProtocolVersion {
    /// Create new ProtocolVersion instance
    pub fn new(first_digit: u8, second_digit: u8, third_digit: u8) -> Self {
        ProtocolVersion {
            first_digit,
            second_digit,
            third_digit,
        }
    }
}

impl ScorexSerializable for ProtocolVersion {
    fn scorex_serialize<W: sigma_ser::vlq_encode::WriteSigmaVlqExt>(
        &self,
        w: &mut W,
    ) -> ScorexSerializeResult {
        w.put_u8(self.first_digit)?;
        w.put_u8(self.second_digit)?;
        w.put_u8(self.third_digit)?;

        Ok(())
    }

    fn scorex_parse<R: sigma_ser::vlq_encode::ReadSigmaVlqExt>(
        r: &mut R,
    ) -> Result<Self, sigma_ser::ScorexParsingError> {
        Ok(ProtocolVersion::new(r.get_u8()?, r.get_u8()?, r.get_u8()?))
    }
}
