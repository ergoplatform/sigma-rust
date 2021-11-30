//! docuemt
use std::convert::TryInto;
use std::io;

use sigma_ser::vlq_encode::VlqEncodingError;
use sigma_ser::{ScorexParsingError, ScorexSerializable, ScorexSerializeResult};

use crate::peer_addr::PeerAddr;
use crate::{peer_feature::PeerFeature, protocol_version::ProtocolVersion};

///
pub struct PeerSpec {
    agent_name: String,
    protocol_version: ProtocolVersion,
    node_name: String,
    declared_addr: Option<PeerAddr>,
    features: Vec<PeerFeature>,
}

impl PeerSpec {
    /// Tester
    pub fn new(
        agent_name: &str,
        protocol_version: ProtocolVersion,
        node_name: &str,
        declared_addr: Option<PeerAddr>,
        features: Vec<PeerFeature>,
    ) -> Self {
        Self {
            agent_name: agent_name.into(),
            protocol_version,
            node_name: node_name.into(),
            declared_addr,
            features,
        }
    }

    /// local_addr
    pub fn local_addr(&self) -> Option<PeerAddr> {
        Some(self.features.iter().find_map(PeerFeature::as_local_addr)?.0)
    }

    /// reachable
    pub fn reachable_peer(&self) -> bool {
        self.addr().is_some()
    }

    /// address
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
            let addr_size: u8 = addr.ip_size()?.try_into().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "failed to parse ip size")
            })?;

            w.put_u8(addr_size)?;
            addr.scorex_serialize(w)?;

            Ok(())
        })?;

        // Can't use Vec<ScorexSerializable> becuase we need the size as u8
        w.put_u8(self.features.len().try_into()?)?;
        self.features
            .iter()
            .try_for_each(|i| i.scorex_serialize(w))?;

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
            Ok(PeerAddr::scorex_parse(r).map_err(|_| VlqEncodingError::VlqDecodingFailed)?)
        });

        // Can't use Vec<ScorexSerializable> becuase we need the size as u8
        let features_count = r.get_u8()?;
        let mut features: Vec<PeerFeature> = Vec::with_capacity(features_count as usize);
        for _ in 0..features_count {
            features.push(PeerFeature::scorex_parse(r)?);
        }

        Ok(PeerSpec::new(
            &agent_name,
            version,
            &node_name,
            declared_addr,
            features,
        ))
    }
}

// TODO: round trip serialization tests
