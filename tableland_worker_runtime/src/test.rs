use crate::spec::bindings::Runtime;
use crate::spec::types::*;
use anyhow::Result;
use http::{HeaderMap, Method};

const WASM_BYTES: &'static [u8] =
    include_bytes!("../../examples/json_api/target/wasm32-unknown-unknown/release/json_api.wasm");

#[tokio::test]
async fn fetch() -> Result<()> {
    let uri: http::Uri = "/worker-version".parse().unwrap();
    let req = Request::new(uri, Method::GET, HeaderMap::new(), None);

    let rt = new_runtime()?;
    let (res, gas) = rt.fetch(req).await;
    let mut res = res??;

    println!("gas used: {}", gas);

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
