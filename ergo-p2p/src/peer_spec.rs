//! docuemt
use std::convert::TryInto;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};

use sigma_ser::vlq_encode::VlqEncodingError;
use sigma_ser::{
    ScorexParsingError, ScorexSerializable, ScorexSerializationError, ScorexSerializeResult,
};

use crate::peer_feature::{LocalAddressPeerFeature, PeerFeatureId};
use crate::{peer_feature::PeerFeature, protocol_version::ProtocolVersion};

///
pub struct PeerSpec {
    agent_name: String,
    protocol_version: ProtocolVersion,
    node_name: String,
    declared_address: Option<SocketAddr>,
    features: Vec<PeerFeature>,
}

impl PeerSpec {
    /// Tester
    pub fn new(
        agent_name: &str,
        protocol_version: ProtocolVersion,
        node_name: &str,
        declared_address: Option<SocketAddr>,
        features: Vec<PeerFeature>,
    ) -> Self {
        Self {
            agent_name: agent_name.into(),
            protocol_version,
            node_name: node_name.into(),
            declared_address,
            features,
        }
    }

    /// local_addr
    pub fn local_addr(&self) -> Option<SocketAddr> {
        // TODO
        None
    }

    /// reachable
    pub fn reachable_peer(&self) -> bool {
        self.address().is_some()
    }

    /// address
    pub fn address(&self) -> Option<SocketAddr> {
        self.declared_address.or(self.local_addr())
    }
}

impl ScorexSerializable for PeerSpec {
    fn scorex_serialize<W: sigma_ser::vlq_encode::WriteSigmaVlqExt>(
        &self,
        w: &mut W,
    ) -> ScorexSerializeResult {
        w.put_short_string(&self.agent_name)?;
        self.protocol_version.scorex_serialize(w)?;
        w.put_short_string(&self.node_name)?;

        w.put_option(self.declared_address, &|w: &mut W,
                                              addr: SocketAddr|
         -> io::Result<()> {
            let ip = match addr.ip() {
                IpAddr::V4(ip) => ip,
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "ipv6 is not supported",
                    ))
                }
            };
            let port: u32 = match addr.port().try_into().ok() {
                Some(p) => p,
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "failed to convert port to u32",
                    ))
                }
            };

            let addr_size: u8 = ip.octets().len().try_into().map_err(|_| {
                io::Error::new(io::ErrorKind::Unsupported, "failed to parse ip size")
            })?;
            w.put_u8(addr_size)?;
            w.put_u32(ip.into())?;
            w.put_u32(port)?;

            Ok(())
        })?;

        w.put_u8(self.features.len().try_into().map_err(|_| {
            ScorexSerializationError::UnexpectedValue("failed to convert port to u16")
        })?)?;
        self.features.iter().try_for_each(|i| {
            i.id().scorex_serialize(w)?;

            let feature_size: u16 = i.scorex_serialize_bytes()?.len().try_into().map_err(|_| {
                ScorexSerializationError::UnexpectedValue("failed to convert feature size to u8")
            })?;

            w.put_u16(feature_size)?;
            i.scorex_serialize(w)
        })?;

        Ok(())
    }

    fn scorex_parse<R: sigma_ser::vlq_encode::ReadSigmaVlqExt>(
        r: &mut R,
    ) -> Result<Self, ScorexParsingError> {
        let agent_name = r.get_short_string()?;

        if agent_name.is_empty() {
            return Err(ScorexParsingError::Io("agent name cannot be empty".into()));
        }

        let version = ProtocolVersion::scorex_parse(r)?;
        let node_name = r.get_short_string()?;
        let declared_address: Option<SocketAddr> = r.get_option(&|r: &mut R| {
            let addr_size = r.get_u8()?;
            let mut fa: Vec<u8> = Vec::with_capacity(addr_size as usize);
            r.read_exact(&mut fa)?;

            let ip = Ipv4Addr::new(fa[0], fa[1], fa[2], fa[3]);
            let port: u16 = r
                .get_u32()?
                .try_into()
                .map_err(|_| VlqEncodingError::Io("failed to convert port to u16".into()))?;

            Ok(SocketAddr::V4(SocketAddrV4::new(ip, port)))
        });

        let features_count = r.get_u8()?;
        let mut features: Vec<PeerFeature> = Vec::with_capacity(features_count as usize);

        for _ in 0..features_count {
            let feature_id: PeerFeatureId = r.get_u8()?.into();
            let feature_size = r.get_u16()?;
            let mut feature_buf: Vec<u8> = Vec::with_capacity(feature_size as usize);
            r.read_exact(&mut feature_buf)?;

            let feature = match feature_id {
                PeerFeatureId(2) => Some(PeerFeature::LocalAddress(
                    LocalAddressPeerFeature::scorex_parse_bytes(&mut feature_buf)?,
                )),
                _ => None,
            };

            if let Some(f) = feature {
                features.push(f);
            }
        }

        Ok(PeerSpec::new(
            &agent_name,
            version,
            &node_name,
            declared_address,
            features,
        ))
    }
}
