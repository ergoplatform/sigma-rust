#[derive(derive_more::From, derive_more::Into)]
pub struct NodeConf(pub(crate) ergo_lib::ergo_rest::NodeConf);
pub type NodeConfPtr = *mut NodeConf;
