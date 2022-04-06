This crate exists as a workaround that gives `reqwest` the ability to have request timeouts for the
WASM platform (see https://github.com/seanmonstar/reqwest/issues/1135). Currently timeouts are
only implemented in `reqwest` for non-WASM platforms. However there exists a yet-to-be-merged pull
request (https://github.com/seanmonstar/reqwest/pull/1274) that implements timeouts under WASM.

This crate vendors the WASM related code of `reqwest` and applies the above mentioned PR on top.
This allows WASM-based dependendents of this crate to have timeouts for requests. Non-WASM users
have access to the underlying `reqwest` crate without any modifications.