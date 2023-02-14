#[cfg(not(feature = "wasi"))]
use crate::spec::types::*;
#[cfg(feature = "wasi")]
use crate::wasi_spec::types::*;
use crate::worker::*;
use bytes::Bytes;
use http::{HeaderMap, Method, Uri};
use serde_bytes::ByteBuf;
use warp::{http::Response as WarpResponse, path::FullPath, Rejection, Reply};

pub async fn add_runtime(worker: Worker, cid: String) -> Result<impl Reply, Rejection> {
    worker.add_runtime(cid.clone()).await.map_err(|e| {
        eprint!("error caching wasm runtime: {}", e);
        warp::reject::reject()
    })?;
    println!("added new wasm runtime: {}", cid);

    Ok("success")
}

pub async fn invoke_runtime(
    worker: Worker,
    method: Method,
    full_path: FullPath,
    cid: String,
    query: String,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl Reply, Rejection> {
    let rt = worker.get_runtime(cid.clone()).await.map_err(|e| {
        eprint!("error getting runtime {}: {}", cid, e);
        warp::reject::reject()
    })?;

    let mut path = full_path
        .as_str()
        .trim_start_matches(format!("/workers/{}", cid).as_str())
        .to_string();
    if query.len() > 0 {
        path = format!("{}?{}", path, query);
    }
    if path.is_empty() {
        path = "/".to_string();
    }
    let uri = path.parse::<Uri>().unwrap();
    let bbody = match body.is_empty() {
        false => Some(ByteBuf::from(body.to_vec())),
        true => None,
    };
    let req = Request::new(uri, method, headers, bbody);

    println!("fetch {} {} on worker {}", req.method(), path, cid);

    let mut res = rt
        .fetch(req)
        .await
        .map_err(|e| {
            eprint!("error invoking worker: {}", e);
            warp::reject::reject()
        })?
        .map_err(|e| {
            eprint!("error fetching worker result: {}", e);
            warp::reject::reject()
        })?;

    let body = res.bytes().await.map_err(|e| {
        eprint!("error parsing worker fetch result: {}", e);
        warp::reject::reject()
    })?;

    let wres = WarpResponse::builder()
        .status(res.status_code())
        .body(body)
        .unwrap();
    let (mut parts, body) = wres.into_parts();
    parts.headers = res.headers().clone();
    Ok(WarpResponse::from_parts(parts, body))
}
