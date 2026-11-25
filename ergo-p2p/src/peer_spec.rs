//! PeerSpec types
use std::io;

use bounded_vec::BoundedVec;
use ergo_chain_types::PeerAddr;
use sigma_ser::vlq_encode::VlqEncodingError;
use sigma_ser::{ScorexParsingError, ScorexSerializable, ScorexSerializeResult};

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
        Some(PeerAddr::from(
            self.features
                .as_ref()?
                .iter()
                .find_map(PeerFeature::as_local_addr)?
                .to_owned(),
        ))
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

        w.put_option(
            self.declared_addr,
            |w: &mut W, addr: PeerAddr| -> io::Result<()> {
                w.put_u8(addr.ip_size() as u8)?;
                addr.scorex_serialize(w)?;

                Ok(())
            },
        )?;

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
        let declared_addr: Option<PeerAddr> =
            r.get_option(|r: &mut R| -> Result<_, ScorexParsingError> {
                // read the size bytes
                // not used at the moment because PeerAddr is currently ipv4/4 bytes
                r.get_u8()?;
                let addr =
                    PeerAddr::scorex_parse(r).map_err(|_| VlqEncodingError::VlqDecodingFailed)?;

                Ok(addr)
            })?;

        // A feature with no parser here, or one that does not parse, is left out and the rest of the
        // spec is kept, as in the reference node's `PeerSpecSerializer.parse`.
        let feat_len = r.get_u8()?;
        // the reference node reads the count as a signed byte and refuses a negative one
        if feat_len > i8::MAX as u8 {
            return Err(ScorexParsingError::ValueOutOfBounds(format!(
                "{} peer features, more than 127",
                feat_len
            )));
        }
        let mut f: Vec<PeerFeature> = Vec::with_capacity(feat_len as usize);
        for _ in 0..feat_len {
            if let Some(feature) = PeerFeature::scorex_parse_or_skip(r)? {
                f.push(feature);
            }
        }
        let features = if f.is_empty() {
            None
        } else {
            Some(PeerFeatures::from_vec(f)?)
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

/// Arbitrary
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used, clippy::panic)]
pub mod arbitrary {
    use super::*;
    use proptest::prelude::{Arbitrary, BoxedStrategy};
    use proptest::test_runner::TestRunner;
    use proptest::{collection::vec, option, prelude::*};

    impl Arbitrary for PeerSpec {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (
                any::<ProtocolVersion>(),
                option::of(any::<PeerAddr>()),
                option::of(vec(any::<PeerFeature>(), 1..4)),
            )
                .prop_map(|(version, declared_addr, features)| {
                    let feats = features.map(|f| PeerFeatures::from_vec(f).unwrap());

                    PeerSpec::new(
                        "/Ergo-Scala-client:2.0.0(iPad; U; CPU OS 3_2_1)/AndroidBuild:0.8/",
                        version,
                        "Testing node",
                        declared_addr,
                        feats,
                    )
                })
                .boxed()
        }
    }

    impl PeerSpec {
        /// Ensure the PeerSpec has a valid addr
        /// This can happen if declared_addr is none and there is no LocalAddressPeerFeature in features
        pub fn with_ensured_addr(mut self) -> Self {
            if self.addr().is_none() {
                let mut runner = TestRunner::default();
                self.declared_addr =
                    Some(any::<PeerAddr>().new_tree(&mut runner).unwrap().current());
            }
            self
        }
    }
}

#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use sigma_ser::scorex_serialize_roundtrip;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn ser_roundtrip(v in any::<PeerSpec>()) {
            assert_eq![scorex_serialize_roundtrip(&v), v]
        }
    }

    fn from_hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    /// The peer spec of a handshake a mainnet seed node (agent `ergoref`, 6.0.2) sent on 2026-11-25, up to
    /// its feature count: the handshake's bytes after its leading time field. Declared address
    /// 159.89.116.15:9030.
    const MAINNET_SPEC_HEAD: &str =
        "076572676f726566060002196d61696e6e65742d736565642d6e6f64652d746f726f6e746f01089f59740fc646";
    /// The two features that handshake carried, as sent: mode (id 16) and session id (id 3).
    const MODE_FEATURE: &str = "100400010001";
    const SESSION_FEATURE: &str = "030d01000204c4b2fbef8fa1a2a92b";
    /// A local address feature (id 2) for 127.0.0.1:9030.
    const LOCAL_ADDR_FEATURE: &str = "02067f000001c646";

    fn spec_bytes(features: &[&str]) -> Vec<u8> {
        let mut bytes = from_hex(MAINNET_SPEC_HEAD);
        bytes.push(features.len() as u8);
        features.iter().for_each(|f| bytes.extend(from_hex(f)));
        bytes
    }

    fn addr(s: &str) -> PeerAddr {
        PeerAddr(s.parse().unwrap())
    }

    #[test]
    fn parses_spec_a_mainnet_node_sent() {
        // exactly the bytes received
        let spec =
            PeerSpec::scorex_parse_bytes(&spec_bytes(&[MODE_FEATURE, SESSION_FEATURE])).unwrap();
        assert_eq!(spec.agent_name, "ergoref");
        assert_eq!(spec.node_name, "mainnet-seed-node-toronto");
        assert_eq!(spec.declared_addr, Some(addr("159.89.116.15:9030")));
        assert_eq!(spec.features, None);
    }

    #[test]
    fn parses_spec_a_second_mainnet_node_sent() {
        // a seed node declaring 6.0.8, the same day: exactly the bytes received after the time field
        let spec = PeerSpec::scorex_parse_bytes(&from_hex(
            "076572676f7265660600081b6d61696e6e65742d736565642d6e6f64652d73696e6761706f7265\
             01089f410b37c64602100400010001030d010002049b8ae8fea6dcf3dc4e",
        ))
        .unwrap();
        assert_eq!(spec.node_name, "mainnet-seed-node-singapore");
        assert_eq!(spec.protocol_version, ProtocolVersion::new(6, 0, 8));
        assert_eq!(spec.declared_addr, Some(addr("159.65.11.55:9030")));
        assert_eq!(spec.features, None);
    }

    #[test]
    fn keeps_the_feature_it_knows_among_those_it_does_not() {
        let spec = PeerSpec::scorex_parse_bytes(&spec_bytes(&[
            MODE_FEATURE,
            LOCAL_ADDR_FEATURE,
            SESSION_FEATURE,
        ]))
        .unwrap();
        assert_eq!(spec.features.as_ref().map(|f| f.len()), Some(1));
        assert_eq!(spec.local_addr(), Some(addr("127.0.0.1:9030")));
    }

    #[test]
    fn skips_a_known_feature_whose_body_does_not_parse() {
        // a local address feature with two bytes where an address and a port belong
        let spec =
            PeerSpec::scorex_parse_bytes(&spec_bytes(&["02027f00", SESSION_FEATURE])).unwrap();
        assert_eq!(spec.features, None);
        assert_eq!(spec.declared_addr, Some(addr("159.89.116.15:9030")));
    }

    /// A feature with an id this crate does not know (99) and a body of `size` zero bytes.
    fn unknown_feature(size: u16) -> Vec<u8> {
        let mut bytes = vec![
            99,
            (size & 0x7f) as u8 | 0x80,
            (size >> 7 & 0x7f) as u8 | 0x80,
            (size >> 14) as u8,
        ];
        bytes.extend(vec![0u8; size as usize]);
        bytes
    }

    fn spec_with(count: u8, features: &[u8]) -> Vec<u8> {
        let mut bytes = from_hex(MAINNET_SPEC_HEAD);
        bytes.push(count);
        bytes.extend_from_slice(features);
        bytes
    }

    #[test]
    fn takes_127_features_and_rejects_128_as_the_reference_node_does() {
        // each feature: id 99, size 0
        let features = [99u8, 0].repeat(128);
        let spec = PeerSpec::scorex_parse_bytes(&spec_with(127, &features[..254])).unwrap();
        assert_eq!(spec.features, None);
        assert!(PeerSpec::scorex_parse_bytes(&spec_with(128, &features)).is_err());
    }

    #[test]
    fn takes_a_feature_of_32767_bytes_and_rejects_32768_as_the_reference_node_does() {
        let spec = PeerSpec::scorex_parse_bytes(&spec_with(1, &unknown_feature(32767))).unwrap();
        assert_eq!(spec.features, None);
        assert!(PeerSpec::scorex_parse_bytes(&spec_with(1, &unknown_feature(32768))).is_err());
    }

    #[test]
    fn rejects_a_feature_longer_than_the_bytes_received() {
        // the session id feature says 13 bytes and 3 follow
        assert!(PeerSpec::scorex_parse_bytes(&spec_bytes(&["030d010002"])).is_err());
    }
}
