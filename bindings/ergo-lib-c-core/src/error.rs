#[cfg(feature = "rest")]
use ergo_lib::ergo_rest::{api::node::PeerDiscoveryError, NodeError};

use std::error;
use thiserror::Error;

// pub type Result<T> = result::Result<T, Box<dyn error::Error + 'static>>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error: {0}")]
    Misc(Box<dyn error::Error + 'static>),
    #[error("invalid argument: {0}")]
    InvalidArgument(&'static str),
}

pub type ErrorPtr = *mut Error;

impl Error {
    pub fn misc<E>(details: E) -> Self
    where
        E: error::Error + 'static,
    {
        Error::Misc(Box::new(details))
    }

    pub fn c_api_from(result: Result<(), Error>) -> ErrorPtr {
        match result {
            Ok(()) => std::ptr::null_mut(),
            Err(err) => Box::into_raw(Box::new(err)),
        }
    }
}

#[cfg(feature = "rest")]
impl From<NodeError> for Error {
    fn from(error: NodeError) -> Self {
        Error::Misc(format!("{:?}", error).into())
    }
}

#[cfg(feature = "rest")]
impl From<PeerDiscoveryError> for Error {
    fn from(error: PeerDiscoveryError) -> Self {
        Error::Misc(format!("{:?}", error).into())
    }
}
