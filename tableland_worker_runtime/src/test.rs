#[cfg(not(feature = "wasi"))]
use crate::spec::bindings::Runtime;
#[cfg(not(feature = "wasi"))]
use crate::spec::types::*;
#[cfg(feature = "wasi")]
use crate::wasi_spec::bindings::Runtime;
#[cfg(feature = "wasi")]
use crate::wasi_spec::types::*;
use anyhow::Result;

#[cfg(not(feature = "wasi"))]
const WASM_BYTES: &'static [u8] =
    include_bytes!("../../example-plugin/target/wasm32-unknown-unknown/debug/example_plugin.wasm");
#[cfg(feature = "wasi")]
const WASM_BYTES: &'static [u8] =
    include_bytes!("../../examples/ssr/target/wasm32-wasi/debug/ssr.wasm");

#[tokio::test]
async fn fetch() -> Result<()> {
    let uri: http::Uri = "https://example.org/hello/world?foo=bar".parse().unwrap();
    let req = Request::new(
        uri.path_and_query().unwrap().to_string(),
        Method::Get,
        Headers::new(),
        None,
    );

    let rt = new_runtime()?;
    let mut res = rt.fetch(req).await??;

    assert_eq!(res.status_code(), 200);

    let body = res.bytes().await?;
    assert_eq!(body.is_empty(), false);

    Ok(())
}

fn new_runtime() -> Result<Runtime> {
    let rt = Runtime::new(WASM_BYTES)?;
    rt.init()?;
    Ok(rt)
}
