#[derive(derive_more::From, derive_more::Into)]
pub struct NodeInfo(pub(crate) ergo_lib::ergo_rest::NodeInfo);
pub type NodeInfoPtr = *mut NodeInfo;
