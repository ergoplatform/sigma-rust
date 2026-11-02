use derive_more::{From, Into};
use ergo_lib::wallet::secret_key;
use pyo3::{exceptions::PyValueError, prelude::*, types::PyType};

use crate::{errors::JsonError, from_json, sigma_protocol::ProveDlog, to_value_error};

/// Secret Key
#[pyclass(eq, frozen, str = "{0:?}", from_py_object)]
#[derive(PartialEq, Eq, Clone, From, Into)]
pub(crate) struct SecretKey(secret_key::SecretKey);

#[pymethods]
impl SecretKey {
    #[classmethod]
    fn random_dlog(_: &Bound<'_, PyType>) -> Self {
        Self(secret_key::SecretKey::random_dlog())
    }
    #[classmethod]
    fn random_dht(_: &Bound<'_, PyType>) -> Self {
        Self(secret_key::SecretKey::random_dht())
    }
    #[classmethod]
    fn from_json(_: &Bound<'_, PyType>, s: Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Self(from_json(s)?))
    }
    #[classmethod]
    fn from_bytes(_: &Bound<'_, PyType>, b: &[u8]) -> PyResult<SecretKey> {
        secret_key::SecretKey::from_bytes(b)
            .map(Self)
            .map_err(to_value_error)
    }
    fn public_image(&self) -> PyResult<ProveDlog> {
        match &self.0 {
            secret_key::SecretKey::DlogSecretKey(dlog_prover_input) => {
                Ok(dlog_prover_input.public_image().into())
            }
            secret_key::SecretKey::DhtSecretKey(dh_tuple_prover_input) => Err(
                PyValueError::new_err("public_image is not supported for DHTuple secret keys"),
            ),
        }
    }
    fn json(&self) -> PyResult<String> {
        serde_json::to_string(&self.0)
            .map_err(JsonError::from)
            .map_err(Into::into)
    }
    fn __bytes__(&self) -> Vec<u8> {
        self.0.to_bytes()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}
