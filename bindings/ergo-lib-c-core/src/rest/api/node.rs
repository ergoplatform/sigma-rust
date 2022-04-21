//! Async REST API for Ergo node

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::str::FromStr;

use crate::block_header::ConstBlockIdPtr;
use crate::rest::c_string_collection::{CStringCollection, CStringCollectionInner};
use crate::rest::node_conf::NodeConfPtr;
use crate::util::const_ptr_as_ref;
use crate::Error;

use self::abortable::spawn_abortable;

use super::callback::AbortCallback;
use super::callback::CompletionCallback;
use super::request_handle::RequestHandle;
use super::request_handle::RequestHandlePtr;
use super::runtime::RestApiRuntimePtr;

mod abortable;

/// GET on /info endpoint
pub unsafe fn rest_api_node_get_info(
    runtime_ptr: RestApiRuntimePtr,
    node_conf_ptr: NodeConfPtr,
    callback: CompletionCallback,
    request_handle_out: *mut RequestHandlePtr,
) -> Result<(), Error> {
    let runtime = const_ptr_as_ref(runtime_ptr, "runtime_ptr")?;
    let node_conf = const_ptr_as_ref(node_conf_ptr, "node_conf_ptr")?.0;
    let abort_callback: AbortCallback = (&callback).into();
    let abort_handle = spawn_abortable(runtime, async move {
        match ergo_lib::ergo_rest::api::node::get_info(node_conf).await {
            Ok(node_info) => callback.succeeded(node_info),
            Err(e) => callback.failed(e.into()),
        }
    })?;
    let request_handle = RequestHandle::new(abort_handle, abort_callback);
    *request_handle_out = Box::into_raw(Box::new(request_handle));
    Ok(())
}

/// GET on /nipopow/proof/{minChainLength}/{suffixLength}/{headerId} endpoint
pub unsafe fn rest_api_node_get_nipopow_proof_by_header_id(
    runtime_ptr: RestApiRuntimePtr,
    node_conf_ptr: NodeConfPtr,
    callback: CompletionCallback,
    request_handle_out: *mut RequestHandlePtr,
    min_chain_length: u32,
    suffix_len: u32,
    header_id_ptr: ConstBlockIdPtr,
) -> Result<(), Error> {
    let runtime = const_ptr_as_ref(runtime_ptr, "runtime_ptr")?;
    let node_conf = const_ptr_as_ref(node_conf_ptr, "node_conf_ptr")?.0;
    let header_id = const_ptr_as_ref(header_id_ptr, "header_id_ptr")?.0.clone();
    let abort_callback: AbortCallback = (&callback).into();
    let abort_handle = spawn_abortable(runtime, async move {
        match ergo_lib::ergo_rest::api::node::get_nipopow_proof_by_header_id(
            node_conf,
            min_chain_length,
            suffix_len,
            header_id,
        )
        .await
        {
            Ok(proof) => callback.succeeded(crate::nipopow::NipopowProof(proof)),
            Err(e) => callback.failed(e.into()),
        }
    })?;
    let request_handle = RequestHandle::new(abort_handle, abort_callback);
    *request_handle_out = Box::into_raw(Box::new(request_handle));
    Ok(())
}

/// Given a list of seed nodes, recursively determine all known peer nodes.
pub unsafe fn rest_api_node_peer_discovery(
    runtime_ptr: RestApiRuntimePtr,
    callback: CompletionCallback,
    request_handle_out: *mut RequestHandlePtr,
    seeds_ptr: *const *const c_char,
    num_seeds: usize,
    max_parallel_requests: u16,
    timeout_sec: u32,
) -> Result<(), Error> {
    if seeds_ptr.is_null() {
        return Err(Error::Misc("seeds_ptr is null".into()));
    }
    if num_seeds == 0 {
        return Err(Error::Misc("num_seeds must be > 0".into()));
    }
    let mut seeds_vec = Vec::with_capacity(num_seeds);
    for i in 0..num_seeds {
        let ptr = *(seeds_ptr.add(i));
        if !ptr.is_null() {
            let str = CStr::from_ptr(ptr)
                .to_str()
                .map_err(|e| Error::Misc(format!("CStr::from_ptr: {:?}", e).into()))?;
            seeds_vec.push(
                url::Url::from_str(str)
                    .map_err(|e| Error::Misc(format!("Url::from_str: {:?}", e).into()))?,
            );
        } else {
            return Err(Error::Misc("seeds_ptr contains null element".into()));
        }
    }
    #[allow(clippy::unwrap_used)]
    let seeds_bounded_vec =
        bounded_vec::NonEmptyVec::from_vec(seeds_vec).unwrap();

    let max_parallel_requests = bounded_integer::BoundedU16::new(max_parallel_requests)
        .ok_or_else(|| Error::Misc("max_parallel_requests must be > 0".into()))?;
    let runtime = const_ptr_as_ref(runtime_ptr, "runtime_ptr")?;
    let abort_callback: AbortCallback = (&callback).into();
    let abort_handle = spawn_abortable(runtime, async move {
        match ergo_lib::ergo_rest::api::node::peer_discovery(
            seeds_bounded_vec,
            max_parallel_requests,
            std::time::Duration::from_secs(timeout_sec as u64),
        )
        .await
        {
            Ok(peers) => {
                let length = peers.len();
                let mut out = vec![];
                for peer in peers {
                    out.push(CString::new(peer.as_str()).unwrap().into_raw());
                }

                // Very important to shrink, since we assume `out`s length equals its capacity when
                // deallocating its memory.
                out.shrink_to_fit();

                let known_peers = CStringCollection(CStringCollectionInner {
                    ptr: out.as_mut_ptr(),
                    length,
                });
                std::mem::forget(out);
                callback.succeeded(known_peers)
            }
            Err(e) => callback.failed(e.into()),
        }
    })?;
    let request_handle = RequestHandle::new(abort_handle, abort_callback);
    *request_handle_out = Box::into_raw(Box::new(request_handle));
    Ok(())
}
