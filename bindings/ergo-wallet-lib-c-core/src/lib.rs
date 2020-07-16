//! C compatible functions to use in C and JNI bindings

// Coding conventions
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
// #![deny(missing_docs)]
#![allow(clippy::missing_safety_doc)]

mod error;
pub use error::*;

use chain::{AddressEncoder, NetworkPrefix};
use sigma_tree::chain;

pub struct Address(chain::Address);
pub type AddressPtr = *mut Address;

pub unsafe fn address_from_testnet(
    address_str: &str,
    address_out: *mut AddressPtr,
) -> Result<(), Error> {
    let address_out: &mut AddressPtr = if let Some(address_out) = address_out.as_mut() {
        address_out
    } else {
        return Err(Error::InvalidArgument("address_out"));
    };

    let encoder = AddressEncoder::new(NetworkPrefix::Testnet);
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
