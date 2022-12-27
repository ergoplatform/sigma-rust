//! Secret types

#![allow(clippy::todo)]

use derive_more::From;
use ergotree_interpreter::sigma_protocol::private_input::DhTupleProverInput;
use ergotree_interpreter::sigma_protocol::private_input::DlogProverInput;
use ergotree_interpreter::sigma_protocol::private_input::PrivateInput;
use ergotree_ir::chain::address::Address;

// TODO: json serialization for SecretKey

/// Types of secrets
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json", serde(untagged))]
#[derive(PartialEq, Eq, Debug, Clone, From)]
pub enum SecretKey {
    /// Secret exponent of a group element, i.e. secret w such as h = g^^w, where g is group generator, h is a public key.
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

    // pub fn dht_from_bytes(bytes: &[u8; DhTupleProverInput::SIZE_BYTES]) -> Option<SecretKey> {
    //     DhTupleProverInput::from_bytes(bytes).map(SecretKey::DhtSecretKey)
    // }

    /// Address (encoded public image)
    pub fn get_address_from_public_image(&self) -> Address {
        match self {
            SecretKey::DlogSecretKey(dpi) => Address::P2Pk(dpi.public_image()),
            SecretKey::DhtSecretKey(dht) => todo!(),
        }
    }

    /// Encode from a serialized key
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            SecretKey::DlogSecretKey(key) => key.to_bytes().to_vec(),
            SecretKey::DhtSecretKey(dht) => todo!(),
        }
    }
}

impl From<SecretKey> for PrivateInput {
    fn from(s: SecretKey) -> Self {
        match s {
            SecretKey::DlogSecretKey(dpi) => PrivateInput::DlogProverInput(dpi),
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
        let sk_copy =
            SecretKey::dlog_from_bytes(&sk.to_bytes().as_slice().try_into().unwrap()).unwrap();
        assert_eq!(sk, sk_copy);
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_dlog_roundtrip() {
        let sk = SecretKey::random_dlog();
        let sk_json = serde_json::to_string(&sk).unwrap();
        dbg!(&sk_json);
        let sk_copy: SecretKey = serde_json::from_str(&sk_json).unwrap();
        assert_eq!(sk, sk_copy);
    }

    #[test]
    #[cfg(feature = "json")]
    fn json_dht_roundtrip() {
        let sk = SecretKey::random_dht();
        let sk_json = serde_json::to_string(&sk).unwrap();
        dbg!(&sk_json);
        let sk_copy: SecretKey = serde_json::from_str(&sk_json).unwrap();
        assert_eq!(sk, sk_copy);
    }
}
