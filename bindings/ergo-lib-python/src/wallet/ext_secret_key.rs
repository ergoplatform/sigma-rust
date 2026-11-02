use derive_more::From;
use ergo_lib::wallet::{derivation_path::ChildIndex, ext_secret_key};
use pyo3::{prelude::*, types::PyType};

use crate::to_value_error;

use super::{
    derivation_path::DerivationPath, ext_pub_key::ExtPubKey, mnemonic::to_seed,
    secret_key::SecretKey,
};

#[pyclass(eq, frozen, skip_from_py_object)]
#[derive(PartialEq, Eq, From)]
pub(crate) struct ExtSecretKey(ext_secret_key::ExtSecretKey);

#[pymethods]
impl ExtSecretKey {
    #[classmethod]
    #[pyo3(signature = (mnemonic_phrase, password=""))]
    fn from_mnemonic(
        _: &Bound<'_, PyType>,
        mnemonic_phrase: &str,
        password: &str,
    ) -> PyResult<Self> {
        ext_secret_key::ExtSecretKey::derive_master(to_seed(mnemonic_phrase, password))
            .map_err(to_value_error)
            .map(Self)
    }
    #[classmethod]
    fn derive_master(_: &Bound<'_, PyType>, seed: &[u8]) -> PyResult<Self> {
        ext_secret_key::ExtSecretKey::derive_master(seed.try_into().map_err(to_value_error)?)
            .map(Self)
            .map_err(to_value_error)
    }
    fn path(&self) -> DerivationPath {
        self.0.path().into()
    }

    fn child(&self, index: &str) -> PyResult<ExtSecretKey> {
        let idx = index.parse::<ChildIndex>().map_err(to_value_error)?;
        Ok(self.0.child(idx).map_err(to_value_error)?.into())
    }
    fn derive(&self, up_path: DerivationPath) -> PyResult<Self> {
        self.0.derive(up_path.0).map(Self).map_err(to_value_error)
    }

    fn secret_key(&self) -> SecretKey {
        self.0.secret_key().into()
    }

    fn public_key(&self) -> PyResult<ExtPubKey> {
        self.0
            .public_key()
            .map(ExtPubKey::from)
            .map_err(to_value_error)
    }
}
