//! PeerSpec types
use std::io;

use bounded_vec::BoundedVec;
use sigma_ser::vlq_encode::VlqEncodingError;
use sigma_ser::{ScorexParsingError, ScorexSerializable, ScorexSerializeResult};

use crate::peer_addr::PeerAddr;
use crate::{peer_feature::PeerFeature, protocol_version::ProtocolVersion};

type PeerFeatures = BoundedVec<PeerFeature, 1, { u8::MAX as usize }>;

/// PeerSpec
#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct PeerSpec {
    agent_name: String,
    protocol_version: ProtocolVersion,
    node_name: String,
    declared_addr: Option<PeerAddr>,
    features: Option<PeerFeatures>,
}

impl PeerSpec {
    /// Create new PeerSpec instance
    pub fn new(
        agent_name: &str,
        protocol_version: ProtocolVersion,
        node_name: &str,
        declared_addr: Option<PeerAddr>,
        features: Option<PeerFeatures>,
    ) -> Self {
        Self {
            agent_name: agent_name.into(),
            protocol_version,
            node_name: node_name.into(),
            declared_addr,
            features,
        }
    }

    /// Local address of the peer if the peer is using the LocalAddress feature
    pub fn local_addr(&self) -> Option<PeerAddr> {
        Some(
            self.features
                .as_ref()?
                .iter()
                .find_map(PeerFeature::as_local_addr)?
                .0,
        )
    }

    /// Returns true if the peer is reachable
    pub fn reachable_peer(&self) -> bool {
        self.addr().is_some()
    }

    /// The address of the peer
    /// Returns either the declared address or local address if either are valid
    pub fn addr(&self) -> Option<PeerAddr> {
        self.declared_addr.or_else(|| self.local_addr())
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

        w.put_option(self.declared_addr, &|w: &mut W,
                                           addr: PeerAddr|
         -> io::Result<()> {
            w.put_u8(addr.ip_size() as u8)?;
            addr.scorex_serialize(w)?;

            Ok(())
        })?;

        if let Some(feats) = &self.features {
            w.put_u8(feats.len() as u8)?;
            feats.iter().try_for_each(|f| f.scorex_serialize(w))?;
        } else {
            w.put_u8(0)?;
        }

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
        let declared_addr: Option<PeerAddr> = r.get_option(&|r: &mut R| {
            // read the size bytes
            // not used at the moment becuase PeerAddr is currently ipv4/4 bytes
            r.get_u8()?;
            let addr =
                PeerAddr::scorex_parse(r).map_err(|_| VlqEncodingError::VlqDecodingFailed)?;

            Ok(addr)
        });

        let feat_len = r.get_u8()?;
        let features = match feat_len {
            0 => None,
            n => {
                let mut f: Vec<PeerFeature> = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    f.push(PeerFeature::scorex_parse(r)?);
                }
                Some(BoundedVec::from_vec(f)?)
            }
        };

        Ok(PeerSpec::new(
            &agent_name,
            version,
            &node_name,
            declared_addr,
            features,
        ))
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddr};

    use super::*;
    use sigma_ser::scorex_serialize_roundtrip;

    #[test]
    fn peer_spec_basic_ser_roundtrip() {
        let obj = PeerSpec::new(
            "/Ergo-Scala-client:2.0.0(iPad; U; CPU OS 3_2_1)/AndroidBuild:0.8/",
            ProtocolVersion(2, 0, 0),
            "Tester",
            None,
            None,
        );
        assert_eq![scorex_serialize_roundtrip(&obj), obj]
    }

    #[test]
    fn peer_spec_declared_addr_ser_roundtrip() {
        let obj = PeerSpec::new(
            "/Ergo-Scala-client:2.0.0(iPad; U; CPU OS 3_2_1)/AndroidBuild:0.8/",
            ProtocolVersion(2, 0, 0),
            "Tester",
            Some(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080).into()),
            None,
        );
        assert_eq![scorex_serialize_roundtrip(&obj), obj]
    }

    #[test]
    fn peer_spec_features_ser_roundtrip() {
        let peer_addr: PeerAddr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 8080).into();
        let local_addr_feature = PeerFeature::LocalAddress(peer_addr.into());
        let features = BoundedVec::from_vec(vec![local_addr_feature]).unwrap();
        let obj = PeerSpec::new(
            "/Ergo-Scala-client:2.0.0(iPad; U; CPU OS 3_2_1)/AndroidBuild:0.8/",
            ProtocolVersion(2, 0, 0),
            "Tester",
            Some(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080).into()),
            Some(features),
        );
        assert_eq![scorex_serialize_roundtrip(&obj), obj]
    }
}
