#[cfg(not(feature = "wasi"))]
use crate::spec::bindings::Runtime;
#[cfg(not(feature = "wasi"))]
use crate::spec::types::*;
#[cfg(feature = "wasi")]
use crate::wasi_spec::bindings::Runtime;
#[cfg(feature = "wasi")]
use crate::wasi_spec::types::*;
use anyhow::Result;
use std::str;

#[cfg(not(feature = "wasi"))]
const WASM_BYTES: &'static [u8] =
    include_bytes!("../../example-plugin/target/wasm32-unknown-unknown/debug/example_plugin.wasm");
#[cfg(feature = "wasi")]
const WASM_BYTES: &'static [u8] =
    include_bytes!("../../examples/ssr/target/wasm32-wasi/debug/ssr.wasm");

#[tokio::test]
async fn fetch() -> Result<()> {
    let uri: http::Uri = "https://example.org:80/hello/world?foo=bar"
        .parse()
        .unwrap();
    let req = Request::new(uri.path_and_query().unwrap().as_str(), Method::Get, None);

    let rt = new_runtime()?;
    let mut res = rt.fetch(req).await??;
    let body = res.bytes().await?;
    println!("{:?}", str::from_utf8(&body));

    // assert_eq!(v.status_code, 201);
    Ok(())
}

fn new_runtime() -> Result<Runtime> {
    let rt = Runtime::new(WASM_BYTES)?;
    rt.init()?;
    Ok(rt)
}
