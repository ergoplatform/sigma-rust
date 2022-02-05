use std::str::FromStr;

use ergo_lib::ergo_chain_types::PeerAddr;

use crate::util::mut_ptr_as_mut;
use crate::Error;

#[derive(derive_more::From, derive_more::Into)]
pub struct NodeConf(pub(crate) ergo_lib::ergo_rest::NodeConf);
pub type NodeConfPtr = *mut NodeConf;

// TODO: switch NodeConf to builder pattern (like ErgoBoxCandidateBuilder)

/// Parse IP address and port from string
pub unsafe fn node_conf_from_addr(addr: &str, ptr_out: *mut NodeConfPtr) -> Result<(), Error> {
    let ptr_out = mut_ptr_as_mut(ptr_out, "ptr_out")?;
    let peer_addr = PeerAddr::from_str(addr).map_err(Error::misc)?;
    let node_conf = ergo_lib::ergo_rest::NodeConf {
        addr: peer_addr,
        api_key: None,
        timeout: None,
    };
    *ptr_out = Box::into_raw(Box::new(node_conf.into()));
    Ok(())
}

// pub unsafe fn node_conf_builder_new(addr: &str, builder_out: *mut )
