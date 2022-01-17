use crate::rest::node_conf::NodeConfPtr;
use crate::rest::node_info::NodeInfoPtr;
use crate::util::const_ptr_as_ref;
use crate::Error;

use super::runtime::RestApiRuntimePtr;

pub unsafe fn rest_api_node_get_info(
    runtime_ptr: RestApiRuntimePtr,
    node_conf_ptr: NodeConfPtr,
    node_info_out: *mut NodeInfoPtr,
) -> Result<(), Error> {
    let runtime = const_ptr_as_ref(runtime_ptr, "runtime_ptr")?;
    let node_conf = const_ptr_as_ref(node_conf_ptr, "node_conf_ptr")?.0;
    let node_info = runtime
        .0
        .block_on({
            ergo_lib::ergo_rest::api::node::get_info(node_conf)
            // .unwrap()
            // .map_err(|e| Error::Misc(format!("{:?}", e).into()))
        })
        .unwrap();
    *node_info_out = Box::into_raw(Box::new(node_info.into()));
    Ok(())
}
