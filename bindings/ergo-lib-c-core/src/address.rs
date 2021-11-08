use crate::{
    error::*,
    util::{const_ptr_as_ref, mut_ptr_as_mut},
};
use ergo_lib::ergotree_ir::chain::address as addr;

pub struct Address(ergo_lib::ergotree_ir::chain::address::Address);
pub type AddressPtr = *mut Address;
/// Pointer to const Address (Address that is pointed-to is immutable)
pub type ConstAddressPtr = *const Address;

/// Decode (base58) testnet address from string, checking that address is from the testnet
pub unsafe fn address_from_testnet(
    address_str: &str,
    address_out: *mut AddressPtr,
) -> Result<(), Error> {
    let address_out = mut_ptr_as_mut(address_out, "address_out")?;

    let encoder = addr::AddressEncoder::new(addr::NetworkPrefix::Testnet);
    let result = encoder.parse_address_from_str(address_str);

    match result {
        Ok(address) => {
            *address_out = Box::into_raw(Box::new(Address(address)));
            Ok(())
        }
        Err(err) => Err(Error::misc(err)),
    }
}

/// Decode (base58) mainnet address from string, checking that address is from the mainnet
pub unsafe fn address_from_mainnet(
    address_str: &str,
    address_out: *mut AddressPtr,
) -> Result<(), Error> {
    let address_out = mut_ptr_as_mut(address_out, "address_out")?;

    let encoder = addr::AddressEncoder::new(addr::NetworkPrefix::Mainnet);
    let result = encoder.parse_address_from_str(address_str);

    match result {
        Ok(address) => {
            *address_out = Box::into_raw(Box::new(Address(address)));
            Ok(())
        }
        Err(err) => Err(Error::misc(err)),
    }
}

/// Decode (base58) address from string without checking the network prefix
pub unsafe fn address_from_base58(
    address_str: &str,
    address_out: *mut AddressPtr,
) -> Result<(), Error> {
    let address_out = mut_ptr_as_mut(address_out, "address_out")?;
    let result = addr::AddressEncoder::unchecked_parse_address_from_str(address_str);
    match result {
        Ok(address) => {
            *address_out = Box::into_raw(Box::new(Address(address)));
            Ok(())
        }
        Err(err) => Err(Error::misc(err)),
    }
}

/// Encode address as base58 string
pub unsafe fn address_to_base58(
    address: ConstAddressPtr,
    network_prefix: NetworkPrefix,
) -> Result<String, Error> {
    let address = const_ptr_as_ref(address, "address")?;
    Ok(addr::AddressEncoder::encode_address_as_string(
        addr::NetworkPrefix::from(network_prefix),
        &address.0,
    ))
}

pub unsafe fn address_type_prefix(address: ConstAddressPtr) -> Result<AddressTypePrefix, Error> {
    let address = const_ptr_as_ref(address, "address")?;
    Ok(address.0.address_type_prefix().into())
}

pub fn address_delete(address: AddressPtr) {
    if !address.is_null() {
        let boxed = unsafe { Box::from_raw(address) };
        std::mem::drop(boxed);
    }
}

/// Network type
#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum NetworkPrefix {
    /// Mainnet
    Mainnet = 0,
    /// Testnet
    Testnet = 16,
}

impl From<NetworkPrefix> for addr::NetworkPrefix {
    fn from(v: NetworkPrefix) -> Self {
        use addr::NetworkPrefix::*;
        match v {
            NetworkPrefix::Mainnet => Mainnet,
            NetworkPrefix::Testnet => Testnet,
        }
    }
}

impl From<addr::NetworkPrefix> for NetworkPrefix {
    fn from(v: addr::NetworkPrefix) -> Self {
        use NetworkPrefix::*;
        match v {
            addr::NetworkPrefix::Mainnet => Mainnet,
            addr::NetworkPrefix::Testnet => Testnet,
        }
    }
}

#[repr(u8)]
pub enum AddressTypePrefix {
    /// 0x01 - Pay-to-PublicKey(P2PK) address
    P2Pk = 1,
    /// 0x02 - Pay-to-Script-Hash(P2SH)
    Pay2Sh = 2,
    /// 0x03 - Pay-to-Script(P2S)
    Pay2S = 3,
}

impl From<AddressTypePrefix> for addr::AddressTypePrefix {
    fn from(v: AddressTypePrefix) -> Self {
        use addr::AddressTypePrefix::*;
        match v {
            AddressTypePrefix::P2Pk => P2Pk,
            AddressTypePrefix::Pay2Sh => Pay2Sh,
            AddressTypePrefix::Pay2S => Pay2S,
        }
    }
}

impl From<addr::AddressTypePrefix> for AddressTypePrefix {
    fn from(v: addr::AddressTypePrefix) -> Self {
        use AddressTypePrefix::*;
        match v {
            addr::AddressTypePrefix::P2Pk => P2Pk,
            addr::AddressTypePrefix::Pay2Sh => Pay2Sh,
            addr::AddressTypePrefix::Pay2S => Pay2S,
        }
    }
}
