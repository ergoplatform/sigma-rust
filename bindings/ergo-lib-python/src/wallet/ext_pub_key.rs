use super::derivation_path::DerivationPath;
use crate::chain::address::Address;
use crate::chain::ec_point::EcPoint;
use crate::to_value_error;
use derive_more::From;
use ergo_lib::wallet::ext_pub_key::{self, ChainCode, PubKeyBytes};
use ergo_lib::{ergotree_ir::chain::address, wallet::derivation_path::ChildIndexNormal};
use pyo3::{pyclass, pymethods, PyResult};

#[derive(From, Clone, PartialEq, Eq)]
#[pyclass(frozen, eq, from_py_object)]
pub(crate) struct ExtPubKey(ext_pub_key::ExtPubKey);

#[pymethods]
impl ExtPubKey {
    /// Create new ExtPubKey from SEC-1 encoded compressed public key, chain code and derivation path
    #[new]
    fn new(
        public_key_bytes: &[u8],
        chain_code: &[u8],
        derivation_path: &DerivationPath,
    ) -> PyResult<Self> {
        let public_key_bytes: PubKeyBytes = public_key_bytes.try_into().map_err(to_value_error)?;
        let chain_code: ChainCode = chain_code.try_into().map_err(to_value_error)?;
        Ok(ExtPubKey(
            ext_pub_key::ExtPubKey::new(
                public_key_bytes,
                chain_code,
                derivation_path.clone().into(),
            )
            .map_err(to_value_error)?,
        ))
    }
    fn public_key(&self) -> EcPoint {
        self.0.public_key.into()
    }
    fn address(&self) -> Address {
        address::Address::from(self.0.clone()).into()
    }
    /// Derive child public key with the given index
    fn child(&self, index: u32) -> PyResult<ExtPubKey> {
        let index = ChildIndexNormal::normal(index).map_err(to_value_error)?;
        Ok(self.0.child(index).into())
    }

    /// Derive a new extended pub key from the derivation path
    fn derive(&self, up_path: DerivationPath) -> PyResult<ExtPubKey> {
        Ok(self
            .0
            .derive(up_path.into())
            .map_err(to_value_error)?
            .into())
    }

    /// Chain code of the `ExtPubKey`
    #[getter]
    fn chain_code(&self) -> Vec<u8> {
        self.0.chain_code().into()
    }

    /// Public key bytes of the `ExtPubKey`
    #[getter]
    fn pub_key_bytes(&self) -> Vec<u8> {
        self.0.pub_key_bytes().into()
    }
    #[getter]
    fn derivation_path(&self) -> DerivationPath {
        self.0.derivation_path.clone().into()
    }
}
