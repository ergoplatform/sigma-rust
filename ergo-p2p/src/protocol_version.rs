//! Ergo P2P network version
use sigma_ser::{ScorexSerializable, ScorexSerializeResult};

/// P2P network protocol version
pub struct ProtocolVersion(pub u8, pub u8, pub u8);

impl ProtocolVersion {
    /// Create new ProtocolVersion instance
    pub fn new(first_digit: u8, second_digit: u8, third_digit: u8) -> Self {
        ProtocolVersion {
            0: first_digit,
            1: second_digit,
            2: third_digit,
        }
    }
}

impl ScorexSerializable for ProtocolVersion {
    fn scorex_serialize<W: sigma_ser::vlq_encode::WriteSigmaVlqExt>(
        &self,
        w: &mut W,
    ) -> ScorexSerializeResult {
        w.put_u8(self.0)?;
        w.put_u8(self.1)?;
        w.put_u8(self.2)?;

        Ok(())
    }

    fn scorex_parse<R: sigma_ser::vlq_encode::ReadSigmaVlqExt>(
        r: &mut R,
    ) -> Result<Self, sigma_ser::ScorexParsingError> {
        Ok(ProtocolVersion::new(r.get_u8()?, r.get_u8()?, r.get_u8()?))
    }
}
