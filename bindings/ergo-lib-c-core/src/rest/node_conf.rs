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
        // default request timeout so a stalled node does not block fallbacks
        timeout: Some(std::time::Duration::from_secs(30)),
    };
    *ptr_out = Box::into_raw(Box::new(node_conf.into()));
    Ok(())
}

// pub unsafe fn node_conf_builder_new(addr: &str, builder_out: *mut )

#[cfg(test)]
mod tests {
    use std::ptr;
    use std::time::Duration;

    use super::*;

    #[test]
    fn node_conf_from_addr_sets_default_timeout() {
        let mut ptr_out: NodeConfPtr = ptr::null_mut();

        unsafe {
            node_conf_from_addr("127.0.0.1:9053", &mut ptr_out).unwrap();
            let node_conf = Box::from_raw(ptr_out);

            assert_eq!(node_conf.0.timeout, Some(Duration::from_secs(30)));
        }
    }
}
