use crate::{error::Error, util::const_ptr_as_ref};

#[derive(derive_more::From, derive_more::Into)]
pub struct NodeInfo(pub(crate) ergo_lib::ergo_rest::NodeInfo);
pub type NodeInfoPtr = *mut NodeInfo;

/// Node's name
pub unsafe fn node_info_get_name(node_info_ptr: NodeInfoPtr) -> String {
    let node_info = const_ptr_as_ref(node_info_ptr, "node_info_ptr").unwrap();
    node_info.0.name.clone()
}

/// Returns true iff the ergo node is at least v4.0.100. This is important due to the EIP-37
/// hard-fork.
pub unsafe fn node_info_is_at_least_version_4_0_100(
    node_info_ptr: NodeInfoPtr,
) -> Result<bool, Error> {
    let node_info = const_ptr_as_ref(node_info_ptr, "node_info_ptr")?;
    Ok(node_info.0.is_at_least_version_4_0_100())
}
