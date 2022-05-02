//! REST API for the services in Ergo ecosystem (node, explorer, etc.)

use crate::reqwest;
use crate::reqwest::{header::CONTENT_TYPE, Client, RequestBuilder};

use crate::NodeConf;

pub mod node;
mod peer_discovery_internals;

fn set_req_headers(rb: RequestBuilder, node: NodeConf) -> RequestBuilder {
    rb.header("accept", "application/json")
        .header("api_key", node.get_node_api_header())
        .header(CONTENT_TYPE, "application/json")
}

fn build_client(node_conf: &NodeConf) -> Result<Client, reqwest::Error> {
    let builder = reqwest::Client::builder();
    if let Some(t) = node_conf.timeout {
        builder.timeout(t).build()
    } else {
        builder.build()
    }
}
