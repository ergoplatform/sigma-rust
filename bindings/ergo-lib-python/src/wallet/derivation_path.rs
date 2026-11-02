use derive_more::{From, Into};
use ergo_lib::wallet::derivation_path::{
    self, ChildIndexError, ChildIndexHardened, ChildIndexNormal,
};
use pyo3::{prelude::*, types::PyType};

use crate::to_value_error;

/// According to
/// BIP-44 <https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki>
/// and EIP-3 <https://github.com/ergoplatform/eips/blob/master/eip-0003.md>
#[pyclass(frozen, eq, from_py_object)]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub(crate) struct DerivationPath(pub(crate) derivation_path::DerivationPath);

#[pymethods]
impl DerivationPath {
    #[new]
    #[pyo3(signature = (acc=0, address_indices=vec![0]))]
    fn new(acc: u32, address_indices: Vec<u32>) -> PyResult<DerivationPath> {
        let acc = ChildIndexHardened::from_31_bit(acc).map_err(to_value_error)?;
        let address_indices = address_indices
            .iter()
            .map(|i| ChildIndexNormal::normal(*i))
            .collect::<Result<Vec<ChildIndexNormal>, ChildIndexError>>()
            .map_err(to_value_error)?;
        Ok(Self(derivation_path::DerivationPath::new(
            acc,
            address_indices,
        )))
    }

    #[classmethod]
    fn master_path(_: &Bound<'_, PyType>) -> Self {
        Self(derivation_path::DerivationPath::master_path())
    }

    #[classmethod]
    fn from_str(_: &Bound<'_, PyType>, path: &str) -> PyResult<DerivationPath> {
        Ok(Self(
            path.parse::<derivation_path::DerivationPath>()
                .map_err(to_value_error)?,
        ))
    }

    #[getter]
    fn depth(&self) -> usize {
        self.0.depth()
    }

    fn next(&self) -> PyResult<DerivationPath> {
        Ok(Self(self.0.next().map_err(to_value_error)?))
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn ledger_bytes(&self) -> Vec<u8> {
        self.0.ledger_bytes()
    }
}
