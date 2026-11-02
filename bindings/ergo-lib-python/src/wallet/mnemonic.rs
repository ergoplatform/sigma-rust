use std::str::FromStr;

use ergo_lib::wallet::{self, mnemonic::MnemonicSeed, mnemonic_generator::Language};
use pyo3::{pyclass, pyfunction, pymethods, PyResult};

use crate::to_value_error;
/// Create a new MnemonicGenerator. Allowed languages are "english", "chinese_simplified", "chinese_traditional", "french", "italian", "japanese", "korean" and "spanish"
/// Strength must be atleast 128 bits, allowed values are [128, 160, 192, 224, 256]
#[pyclass(frozen, skip_from_py_object)]
pub(crate) struct MnemonicGenerator(wallet::mnemonic_generator::MnemonicGenerator);

#[pymethods]
impl MnemonicGenerator {
    #[new]
    fn new(language: &str, strength: u32) -> PyResult<Self> {
        wallet::mnemonic_generator::MnemonicGenerator::new(
            Language::from_str(language).map_err(to_value_error)?,
            strength,
        )
        .map(Self)
        .map_err(to_value_error)
    }

    fn generate(&self) -> String {
        self.0.generate()
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_entropy(&self, entropy: Vec<u8>) -> PyResult<String> {
        self.0.from_entropy(entropy).map_err(to_value_error)
    }
}

/// Create new MnemonicSeed from seed phrase and optional password
/// Can be turned into ExtSecretKey using ExtSecretKey.derive_master(to_seed(seed_phrase, password))
#[pyfunction]
#[pyo3(signature = (mnemonic_phrase, password = ""))]
pub(crate) fn to_seed(mnemonic_phrase: &str, password: &str) -> MnemonicSeed {
    wallet::mnemonic::Mnemonic::to_seed(mnemonic_phrase, password)
}
