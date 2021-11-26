//! docaAF
use std::convert::{TryFrom, TryInto};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use sigma_ser::vlq_encode::WriteSigmaVlqExt;
use sigma_ser::{ScorexSerializable, ScorexSerializationError, ScorexSerializeResult};

/// Peer feature identifier
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

impl From<u8> for PeerFeatureId {
    fn from(u: u8) -> Self {
        PeerFeatureId(u)
    }
}

/// Peer features
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

    /// Write `self` to the `writer`
    pub fn scorex_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> ScorexSerializeResult {
        match self {
            PeerFeature::LocalAddress(pf) => pf.scorex_serialize(w),
        }
    }

    /// Serialize a ScorexSerializable value into bytes
    pub fn scorex_serialize_bytes(&self) -> Result<Vec<u8>, ScorexSerializationError> {
        let mut w = vec![];
        self.scorex_serialize(&mut w)?;
        Ok(w)
    }
}

/// asfd
pub struct LocalAddressPeerFeature {
    address: SocketAddr,
}

impl LocalAddressPeerFeature {
    /// test
    pub fn new(address: SocketAddr) -> Self {
        Self { address }
    }
}

impl ScorexSerializable for LocalAddressPeerFeature {
    fn scorex_serialize<W: sigma_ser::vlq_encode::WriteSigmaVlqExt>(
        &self,
        w: &mut W,
    ) -> ScorexSerializeResult {
        let ip = match self.address.ip() {
            IpAddr::V4(ip) => ip,
            _ => {
                return Err(ScorexSerializationError::UnexpectedValue(
                    "ipv6 unsupported",
                ))
            }
        };
        let port = match u32::try_from(self.address.port()).ok() {
            Some(p) => p,
            _ => {
                return Err(ScorexSerializationError::UnexpectedValue(
                    "failed to convert port to u32",
                ))
            }
        };

        w.put_u32(ip.into())?;
        w.put_u32(port)?;

        Ok(())
    }

    fn scorex_parse<R: sigma_ser::vlq_encode::ReadSigmaVlqExt>(
        r: &mut R,
    ) -> Result<Self, sigma_ser::ScorexParsingError> {
        let ip: Ipv4Addr = r.get_u32()?.into();
        let port = r.get_u32()?;

        Ok(LocalAddressPeerFeature::new(SocketAddr::new(
            IpAddr::V4(ip),
            port.try_into().map_err(|_| {
                ScorexSerializationError::UnexpectedValue("failed to convert port to u16")
            })?,
        )))
    }
}
