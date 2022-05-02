//! Thin wrapper around `reqwest` crate for WASM

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

    /// Shortcut method to quickly make a `GET` request.
    ///
    /// See also the methods on the [`reqwest::Response`](./struct.Response.html)
    /// type.
    ///
    /// **NOTE**: This function creates a new internal `Client` on each call,
    /// and so should not be used if making many requests. Create a
    /// [`Client`](./struct.Client.html) instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # async fn run() -> Result<(), reqwest::Error> {
    /// let body = reqwest::get("https://www.rust-lang.org").await?
    ///     .text().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// This function fails if:
    ///
    /// - native TLS backend cannot be initialized
    /// - supplied `Url` cannot be parsed
    /// - there was an error while sending request
    /// - redirect limit was exhausted
    pub async fn get<T: IntoUrl>(url: T) -> crate::reqwest::Result<Response> {
        Client::builder().build()?.get(url).send().await
    }
}
