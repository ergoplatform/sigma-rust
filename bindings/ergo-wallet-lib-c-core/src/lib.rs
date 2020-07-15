//! C compatible functions to use in C and JNI bindings

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
        todo!()
        // return Error::invalid_input("address_out").with(NulPtr).into();
    };

    let encoder = AddressEncoder::new(NetworkPrefix::Testnet);
    let result = encoder.parse_address_from_str(address_str);

    match result {
        Ok(address) => {
            todo!()
            // *address_out = Box::into_raw(Box::new(Address(address)));
            // Result::success()
        }
        Err(err) => todo!(), //  err.into(),
    }
}
