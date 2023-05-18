//! Secret types

use derive_more::From;
use ergo_chain_types::EcPoint;
use ergotree_interpreter::sigma_protocol::private_input::DhTupleProverInput;
use ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
use ergotree_interpreter::sigma_protocol::private_input::PrivateInput;
use ergotree_ir::chain::address::Address;
use ergotree_ir::ergo_tree::ErgoTree;
use ergotree_ir::mir::constant::Constant;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::serialization::SigmaSerializable;
use thiserror::Error;

/// Types of secrets
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json", serde(untagged))]
#[derive(PartialEq, Eq, Debug, Clone, From)]
pub enum SecretKey {
    /// Secret exponent of a group element, i.e. secret w such as h = g^^w, where g is group generator,
    /// h is a public key.
    DlogSecretKey(DlogProverInput),
    /// Diffie-Hellman tuple and secret
    /// Used in a proof that of equality of discrete logarithms (i.e., a proof of a Diffie-Hellman tuple):
    /// given group elements g, h, u, v, the proof convinces a verifier that the prover knows `w` such
    /// that `u = g^w` and `v = h^w`, without revealing `w`
    DhtSecretKey(DhTupleProverInput),
}

impl SecretKey {
    /// Generates random DlogProverInput
    pub fn random_dlog() -> SecretKey {
        SecretKey::DlogSecretKey(DlogProverInput::random())
    }

    /// Generates random DhTupleProverInput
    pub fn random_dht() -> SecretKey {
        SecretKey::DhtSecretKey(DhTupleProverInput::random())
    }

    /// Parse DlogSecretKey from bytes (SEC-1-encoded scalar)
    pub fn dlog_from_bytes(bytes: &[u8; DlogProverInput::SIZE_BYTES]) -> Option<SecretKey> {
        DlogProverInput::from_bytes(bytes).map(SecretKey::DlogSecretKey)
    }

    /// Parse DhtSecretKey  from bytes
    /// expected 32(secret)+33(g)+33(h)+33(u)+33(v)=164 bytes
    /// secret is expected as SEC-1-encoded scalar of 32 bytes,
    /// g,h,u,v are expected as 33-byte compressed points
    pub fn dht_from_bytes(bytes: &[u8; DhTupleProverInput::SIZE_BYTES]) -> Option<SecretKey> {
        DhTupleProverInput::from_bytes(bytes).map(SecretKey::DhtSecretKey)
    }

    /// Parse from bytes
    /// secret is expected as SEC-1-encoded scalar of 32 bytes,
    /// g,h,u,v are expected as 33-byte compressed points
    pub fn dht_from_bytes_fields(
        w_bytes: &[u8; DlogProverInput::SIZE_BYTES],
        g_bytes: &[u8; EcPoint::GROUP_SIZE],
        h_bytes: &[u8; EcPoint::GROUP_SIZE],
        u_bytes: &[u8; EcPoint::GROUP_SIZE],
        v_bytes: &[u8; EcPoint::GROUP_SIZE],
    ) -> Option<SecretKey> {
        DhTupleProverInput::from_bytes_fields(w_bytes, g_bytes, h_bytes, u_bytes, v_bytes)
            .map(SecretKey::DhtSecretKey)
    }

    /// Address from public image:
    /// P2PK address for DlogSecretKey
    /// P2S address with ProveDhTuple ergo tree for DhtSecretKey
    #[allow(clippy::unwrap_used)]
    pub fn get_address_from_public_image(&self) -> Address {
        match self {
            SecretKey::DlogSecretKey(dlog) => Address::P2Pk(dlog.public_image()),
            SecretKey::DhtSecretKey(dht) => {
                let expr: Expr = Constant::from(dht.public_image().clone()).into();
                let ergo_tree: ErgoTree = expr.try_into().unwrap();
                Address::P2S(ergo_tree.sigma_serialize_bytes().unwrap())
            }
        }
    }

    /// Encode from a serialized key
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            SecretKey::DlogSecretKey(dlog) => dlog.to_bytes().to_vec(),
            SecretKey::DhtSecretKey(dht) => dht.to_bytes().to_vec(),
        }
    }

    /// Parse secret key from bytes
    /// expected 32 bytes for Dlog, 32(secret)+33(g)+33(h)+33(u)+33(v)=164 bytes for DHT
    /// secret is expected as SEC-1-encoded scalar of 32 bytes,
    /// g,h,u,v are expected as 33-byte compressed points
    pub fn from_bytes(bytes: &[u8]) -> Result<SecretKey, SecretKeyParsingError> {
        if bytes.len() == DlogProverInput::SIZE_BYTES {
            let mut dlog_bytes = [0u8; DlogProverInput::SIZE_BYTES];
            dlog_bytes.copy_from_slice(bytes);
            Ok(SecretKey::dlog_from_bytes(&dlog_bytes)
                .ok_or(SecretKeyParsingError::DlogParsingError)?)
        } else if bytes.len() == DhTupleProverInput::SIZE_BYTES {
            let mut dht_bytes = [0u8; DhTupleProverInput::SIZE_BYTES];
            dht_bytes.copy_from_slice(bytes);
            Ok(SecretKey::DhtSecretKey(
                DhTupleProverInput::from_bytes(&dht_bytes)
                    .ok_or(SecretKeyParsingError::DhtParsingError)?,
            ))
        } else {
            Err(SecretKeyParsingError::InvalidLength)
        }
    }
}

/// Error type for SecretKey parsing
#[allow(missing_docs)]
#[derive(Error, Eq, PartialEq, Debug, Clone)]
pub enum SecretKeyParsingError {
    #[error("Dlog parsing error")]
    DlogParsingError,
    #[error("DHT secret parsing error")]
    DhtParsingError,
    #[error("Invalid length, expected either 32(Dlog) or 164(DHT) bytes")]
    InvalidLength,
}

impl From<SecretKey> for PrivateInput {
    fn from(s: SecretKey) -> Self {
        match s {
            SecretKey::DlogSecretKey(dlog) => PrivateInput::DlogProverInput(dlog),
            SecretKey::DhtSecretKey(dht) => PrivateInput::DhTupleProverInput(dht),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn dlog_roundtrip() {
        let sk = SecretKey::random_dlog();
        let sk_copy1 =
            SecretKey::dlog_from_bytes(&sk.to_bytes().as_slice().try_into().unwrap()).unwrap();
        let sk_copy2 = SecretKey::from_bytes(sk.to_bytes().as_slice()).unwrap();
        assert_eq!(sk, sk_copy1);
        assert_eq!(sk, sk_copy2);
    }

    #[test]
    fn dht_roundtrip() {
        let sk = SecretKey::random_dht();
        let sk_copy1 =
            SecretKey::dht_from_bytes(&sk.to_bytes().as_slice().try_into().unwrap()).unwrap();
        let sk_copy2 = SecretKey::from_bytes(sk.to_bytes().as_slice()).unwrap();
        assert_eq!(sk, sk_copy1);
        assert_eq!(sk, sk_copy2);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[cfg(feature = "json")]
mod json_tests {
    use crate::wallet::secret_key::SecretKey;
    use pretty_assertions::assert_eq;

    #[test]
    fn json_dlog_roundtrip() {
        let sk = SecretKey::random_dlog();
        let sk_json = serde_json::to_string(&sk).unwrap();
        //dbg!(&sk_json);
        let sk_copy: SecretKey = serde_json::from_str(&sk_json).unwrap();
        assert_eq!(sk, sk_copy);
    }

    #[test]
    fn json_dht_roundtrip() {
        let sk = SecretKey::random_dht();
        let sk_json = serde_json::to_string(&sk).unwrap();
        //dbg!(&sk_json);
        let sk_copy: SecretKey = serde_json::from_str(&sk_json).unwrap();
        assert_eq!(sk, sk_copy);
    }

    #[test]
    fn json_dht_golden() {
        let sk_json = r#"{
  "secret": "b2a93a9a37b4656c7abf4e259b9c066cd8bf4e02449d5956aaf453a73764bfeb",
  "g": "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
  "h": "0288a812f57b66b4c68fd9e097c79a6e2847013fa4112a43c45cd41c9ba8c79b69",
  "u": "0381fff110959bc06d99c5580c462c0196d8434bc5d470fb10a160019276e648c2",
  "v": "02c6626ad387bb6b2eccc2fdd238c97ee4a81bfded401843bc8bb71f1cc7269924"
}"#;
        let sk: SecretKey = serde_json::from_str(sk_json).unwrap();
        assert!(matches!(sk, SecretKey::DhtSecretKey(_)));
        let sk_json_copy = serde_json::to_string_pretty(&sk).unwrap();
        assert_eq!(sk_json, sk_json_copy);
    }

    #[test]
    fn json_dlog_golden() {
        let sk_json = r#""0cd81ce156fed4017520e561e9c492222027751ed0dd71b5a9b3a61da68b5850""#;
        //dbg!(&sk_json);
        let sk: SecretKey = serde_json::from_str(sk_json).unwrap();
        assert!(matches!(sk, SecretKey::DlogSecretKey(_)));
        let sk_json_copy = serde_json::to_string_pretty(&sk).unwrap();
        assert_eq!(sk_json, sk_json_copy);
    }
}
