//! Ergo P2P network version
use sigma_ser::{ScorexSerializable, ScorexSerializeResult};

/// P2P network protocol version
/// Follows semantic versioning convention
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct ProtocolVersion {
    major: u8,
    minor: u8,
    patch: u8,
}

impl ProtocolVersion {
    /// Create new ProtocolVersion instance
    pub const fn new(major: u8, minor: u8, patch: u8) -> ProtocolVersion {
        ProtocolVersion {
            major,
            minor,
            patch,
        }
    }

    /// Initial protocol version
    pub const INITIAL: Self = ProtocolVersion::new(0, 0, 1);
}

impl ScorexSerializable for ProtocolVersion {
    fn scorex_serialize<W: sigma_ser::vlq_encode::WriteSigmaVlqExt>(
        &self,
        w: &mut W,
    ) -> ScorexSerializeResult {
        w.put_u8(self.major)?;
        w.put_u8(self.minor)?;
        w.put_u8(self.patch)?;

        Ok(())
    }

    fn scorex_parse<R: sigma_ser::vlq_encode::ReadSigmaVlqExt>(
        r: &mut R,
    ) -> Result<Self, sigma_ser::ScorexParsingError> {
        Ok(ProtocolVersion::new(r.get_u8()?, r.get_u8()?, r.get_u8()?))
    }
}

/// Arbitrary
#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use super::*;
    use proptest::prelude::*;
    use proptest::prelude::{Arbitrary, BoxedStrategy};

    impl Arbitrary for ProtocolVersion {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (any::<u8>(), any::<u8>(), any::<u8>())
                .prop_map(|(major, minor, patch)| ProtocolVersion::new(major, minor, patch))
                .boxed()
        }
    }
}

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[cfg(feature = "arbitrary")]
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use sigma_ser::scorex_serialize_roundtrip;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn ser_roundtrip(v in any::<ProtocolVersion>()) {
            assert_eq![scorex_serialize_roundtrip(&v), v]
        }
    }
}
