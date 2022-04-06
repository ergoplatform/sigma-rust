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

if_hyper! {
    pub use reqwest::{
        Body, Client, ClientBuilder, Request, RequestBuilder, Response, Error, Result
    };
}

if_wasm! {
    mod wasm;
    mod util;
    #[macro_use]
    mod error;
    mod into_url;
    mod response;
    pub use self::error::{Error, Result};
    pub use self::into_url::IntoUrl;
    pub use self::response::ResponseBuilderExt;
    pub use http::Method;
    pub use http::{StatusCode, Version};
    pub use url::Url;
    pub use self::wasm::{Body, Client, ClientBuilder, Request, RequestBuilder, Response};
}
