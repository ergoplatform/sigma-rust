macro_rules! if_wasm {
    ($($item:item)*) => {$(
        #[cfg(target_arch = "wasm32")]
        $item
    )*}
}

macro_rules! if_hyper {
    ($($item:item)*) => {$(
        #[cfg(not(target_arch = "wasm32"))]
        $item
    )*}
}

pub use http::header;
pub use http::Method;
pub use http::{StatusCode, Version};
pub use url::Url;

if_hyper! {
    pub use reqwest::{
        Body, Client, ClientBuilder, Request, RequestBuilder, Response, Error, Result
    };
}

if_wasm! {
    mod wasm;
    mod util;

    pub use self::wasm::{Body, Client, ClientBuilder, Request, RequestBuilder, Response};
}
