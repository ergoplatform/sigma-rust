//! Contract, for easier ErgoTree generation

use ergo_lib::{chain, ergoscript_compiler::script_env::ScriptEnv};

use crate::{
    address::ConstAddressPtr,
    ergo_tree::{ConstErgoTreePtr, ErgoTree, ErgoTreePtr},
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// Defines the contract(script) that will be guarding box contents
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Contract(pub(crate) chain::contract::Contract);
pub type ContractPtr = *mut Contract;
pub type ConstContractPtr = *const Contract;

pub unsafe fn contract_new(
    ergo_tree_ptr: ConstErgoTreePtr,
    contract_out: *mut ContractPtr,
) -> Result<(), Error> {
    let ergo_tree = const_ptr_as_ref(ergo_tree_ptr, "ergo_tree_ptr")?;
    let contract_out = mut_ptr_as_mut(contract_out, "contract_out")?;
    *contract_out = Box::into_raw(Box::new(Contract(chain::contract::Contract::new(
        ergo_tree.0.clone(),
    ))));
    Ok(())
}

pub unsafe fn contract_pay_to_address(
    address_ptr: ConstAddressPtr,
    contract_out: *mut ContractPtr,
) -> Result<(), Error> {
    let address = const_ptr_as_ref(address_ptr, "address_ptr")?;
    let contract_out = mut_ptr_as_mut(contract_out, "contract_out")?;
    let inner = chain::contract::Contract::pay_to_address(&address.0).map_err(|_| {
        Error::Misc("Contract.pay_to_address: can't parse recipient address".into())
    })?;
    *contract_out = Box::into_raw(Box::new(Contract(inner)));
    Ok(())
}

pub unsafe fn contract_compile(source: &str, contract_out: *mut ContractPtr) -> Result<(), Error> {
    let contract_out = mut_ptr_as_mut(contract_out, "contract_out")?;
    let inner = chain::contract::Contract::compile(source, ScriptEnv::new()).map_err(|_| {
        Error::Misc("Contract.pay_to_address: can't parse recipient address".into())
    })?;
    *contract_out = Box::into_raw(Box::new(Contract(inner)));
    Ok(())
}

pub unsafe fn contract_ergo_tree(
    contract_ptr: ConstContractPtr,
    ergo_tree_out: *mut ErgoTreePtr,
) -> Result<(), Error> {
    let contract = const_ptr_as_ref(contract_ptr, "contract_ptr")?;
    let ergo_tree_out = mut_ptr_as_mut(ergo_tree_out, "ergo_tree_out")?;
    *ergo_tree_out = Box::into_raw(Box::new(ErgoTree(contract.0.ergo_tree())));
    Ok(())
}
