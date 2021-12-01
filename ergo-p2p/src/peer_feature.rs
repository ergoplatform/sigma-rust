//! PeerFeature types
use std::convert::TryInto;

use derive_more::{From, Into};

use sigma_ser::vlq_encode::WriteSigmaVlqExt;
use sigma_ser::{ScorexParsingError, ScorexSerializable, ScorexSerializeResult};

use crate::peer_addr::PeerAddr;

/// Peer feature identifier
#[derive(PartialEq, Eq, Debug, Copy, Clone, From, Into)]
pub struct PeerFeatureId(pub u8);

impl ScorexSerializable for PeerFeatureId {
    fn scorex_serialize<W: sigma_ser::vlq_encode::WriteSigmaVlqExt>(
        &self,
        w: &mut W,
    ) -> ScorexSerializeResult {
        w.put_u8(self.0)?;

        Ok(())
    }

    fn scorex_parse<R: sigma_ser::vlq_encode::ReadSigmaVlqExt>(
        r: &mut R,
    ) -> Result<Self, sigma_ser::ScorexParsingError> {
        Ok(Self(r.get_u8()?))
    }
}

/// Peer features
#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum PeerFeature {
    /// Local address peer feature
    LocalAddress(LocalAddressPeerFeature),
}

impl PeerFeature {
    /// Id of the peer feature
    pub fn id(&self) -> PeerFeatureId {
        match self {
            PeerFeature::LocalAddress(_) => PeerFeatureId(2),
        }
    }

    /// Return the feature as a LocalAddressPeerFeature if its of that type
    /// otherwise returns None
    pub fn as_local_addr(&self) -> Option<&LocalAddressPeerFeature> {
        match self {
            PeerFeature::LocalAddress(pf) => Some(pf),
        }
    }
}

impl ScorexSerializable for PeerFeature {
    fn scorex_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> ScorexSerializeResult {
        self.id().scorex_serialize(w)?;

        let bytes = match self {
            PeerFeature::LocalAddress(pf) => pf.scorex_serialize_bytes(),
        }?;

        w.put_u16(bytes.len().try_into()?)?;
        w.write_all(&bytes)?;

        Ok(())
    }

    fn scorex_parse<R: sigma_ser::vlq_encode::ReadSigmaVlqExt>(
        r: &mut R,
    ) -> Result<Self, sigma_ser::ScorexParsingError> {
        let feature_id = PeerFeatureId::scorex_parse(r)?;
        let feature_size = r.get_u16()?;
        let mut feature_buf = vec![0u8; feature_size as usize];
        r.read_exact(&mut feature_buf)?;

        let feature = match feature_id {
            PeerFeatureId(2) => PeerFeature::LocalAddress(
                LocalAddressPeerFeature::scorex_parse_bytes(&feature_buf)?,
            ),
            _ => return Err(ScorexParsingError::Misc("unknown feature id".into())),
        };

        Ok(feature)
    }
}

/// Arbitrary
#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use super::*;
    use proptest::prelude::*;
    use proptest::prelude::{Arbitrary, BoxedStrategy};

    impl Arbitrary for PeerFeature {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            prop_oneof![any::<LocalAddressPeerFeature>().prop_map(PeerFeature::LocalAddress)]
                .boxed()
        }
    }
}

/// LocalAddressPeerFeature
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
#[derive(PartialEq, Eq, Debug, Hash, Clone, From, Into)]
pub struct LocalAddressPeerFeature(PeerAddr);

impl ScorexSerializable for LocalAddressPeerFeature {
    fn scorex_serialize<W: sigma_ser::vlq_encode::WriteSigmaVlqExt>(
        &self,
        w: &mut W,
    ) -> ScorexSerializeResult {
        self.0.scorex_serialize(w)?;

        Ok(())
    }

    fn scorex_parse<R: sigma_ser::vlq_encode::ReadSigmaVlqExt>(
        r: &mut R,
    ) -> Result<Self, sigma_ser::ScorexParsingError> {
        Ok(PeerAddr::scorex_parse(r)?.into())
    }
}

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use sigma_ser::scorex_serialize_roundtrip;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn local_address_feature_ser_roundtrip(v in any::<LocalAddressPeerFeature>()) {
            assert_eq![scorex_serialize_roundtrip(&v), v]
        }
    }
}
