use crate::rest::node_conf::NodeConfPtr;
use crate::rest::node_info::NodeInfoPtr;
use crate::util::const_ptr_as_ref;
use crate::Error;

pub unsafe fn rest_api_node_get_info(
    node_conf_ptr: NodeConfPtr,
    node_info_out: *mut NodeInfoPtr,
) -> Result<(), Error> {
    // let runtime_inner = tokio::runtime::Runtime::new().unwrap();
    let runtime_inner = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::Misc(format!("failed to create tokio runtime: {:?}", e).into()))?;
    let node_conf = const_ptr_as_ref(node_conf_ptr, "node_conf_ptr")?.0;
    let node_info = runtime_inner
        .block_on({
            ergo_lib::ergo_rest::api::node::get_info(node_conf)
            // .unwrap()
            // .map_err(|e| Error::Misc(format!("{:?}", e).into()))
        })
        .unwrap();
    *node_info_out = Box::into_raw(Box::new(node_info.into()));
    Ok(())
}
