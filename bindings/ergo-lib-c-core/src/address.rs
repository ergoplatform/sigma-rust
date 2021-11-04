use crate::error::*;
use ergo_lib::ergotree_ir::chain::address as addr;

pub struct Address(ergo_lib::ergotree_ir::chain::address::Address);
pub type AddressPtr = *mut Address;

/// Decode (base58) testnet address from string, checking that address is from the testnet
pub unsafe fn address_from_testnet(
    address_str: &str,
    address_out: *mut AddressPtr,
) -> Result<(), Error> {
    let address_out: &mut AddressPtr = if let Some(address_out) = address_out.as_mut() {
        address_out
    } else {
        return Err(Error::InvalidArgument("address_out"));
    };

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
    let address_out: &mut AddressPtr = if let Some(address_out) = address_out.as_mut() {
        address_out
    } else {
        return Err(Error::InvalidArgument("address_out"));
    };

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

pub fn address_delete(address: AddressPtr) {
    if !address.is_null() {
        let boxed = unsafe { Box::from_raw(address) };
        std::mem::drop(boxed);
    }
}
