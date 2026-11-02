use derive_more::{From, Into};
use ergo_lib::ergotree_ir::chain::address::{self, AddressEncoder};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyType};

use crate::{ergo_tree::ErgoTree, sigma_protocol::ProveDlog, to_value_error};
#[pyclass(eq, eq_int, frozen, from_py_object)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NetworkPrefix {
    Mainnet = 0x00,
    Testnet = 0x10,
}

impl From<NetworkPrefix> for address::NetworkPrefix {
    fn from(value: NetworkPrefix) -> Self {
        #[allow(clippy::unwrap_used)] // All variants of python NetworkPrefix are valid
        address::NetworkPrefix::try_from(value as u8).unwrap()
    }
}

#[pyclass(eq, frozen, from_py_object)]
#[derive(From, Into, Clone, PartialEq, Eq)]
pub(crate) struct Address(pub(crate) address::Address);

#[pymethods]
impl Address {
    #[new]
    #[pyo3(signature = (arg, network_prefix=None))]
    fn new(arg: &Bound<'_, PyAny>, network_prefix: Option<NetworkPrefix>) -> PyResult<Self> {
        let encoder =
            network_prefix.map(|prefix| AddressEncoder::new(address::NetworkPrefix::from(prefix)));

        match arg.extract::<&str>() {
            Ok(s) => {
                if let Some(encoder) = encoder {
                    encoder
                        .parse_address_from_str(s)
                        .map_err(to_value_error)
                        .map(Self)
                } else {
                    AddressEncoder::unchecked_parse_address_from_str(s)
                        .map_err(to_value_error)
                        .map(Self)
                }
            }
            Err(e) => match arg.extract::<&[u8]>() {
                Ok(bytes) => AddressEncoder::unchecked_parse_address_from_bytes(bytes)
                    .map_err(to_value_error)
                    .map(Self),
                Err(e) => Err(PyValueError::new_err("expected str or bytes argument")),
            },
        }
    }

    #[classmethod]
    fn p2pk(_: &Bound<'_, PyType>, prove_dlog: ProveDlog) -> Address {
        address::Address::P2Pk(prove_dlog.into()).into()
    }

    #[classmethod]
    fn recreate_from_ergo_tree(_: &Bound<'_, PyType>, tree: &ErgoTree) -> PyResult<Self> {
        address::Address::recreate_from_ergo_tree(&tree.0)
            .map(Self)
            .map_err(to_value_error)
    }

    pub(crate) fn ergo_tree(&self) -> PyResult<ErgoTree> {
        self.0.script().map(Into::into).map_err(to_value_error)
    }

    fn to_str(&self, network_prefix: NetworkPrefix) -> String {
        AddressEncoder::new(network_prefix.into()).address_to_str(&self.0)
    }
}

#[cfg(test)]
mod test {
    use super::NetworkPrefix;
    #[test]
    fn eq_network_prefix() {
        assert_eq!(
            NetworkPrefix::Testnet as u8,
            ergo_lib::ergotree_ir::chain::address::NetworkPrefix::Testnet as u8
        );
        assert_eq!(
            NetworkPrefix::Mainnet as u8,
            ergo_lib::ergotree_ir::chain::address::NetworkPrefix::Mainnet as u8
        );
    }
}
