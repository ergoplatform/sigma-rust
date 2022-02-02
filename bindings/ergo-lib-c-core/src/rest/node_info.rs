use crate::util::const_ptr_as_ref;

#[derive(derive_more::From, derive_more::Into)]
pub struct NodeInfo(pub(crate) ergo_lib::ergo_rest::NodeInfo);
pub type NodeInfoPtr = *mut NodeInfo;

/// Node's name
pub unsafe fn node_info_get_name(node_info_ptr: NodeInfoPtr) -> String {
    let node_info = const_ptr_as_ref(node_info_ptr, "node_info_ptr").unwrap();
    node_info.0.name.clone()
}
