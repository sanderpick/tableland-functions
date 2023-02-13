#[cfg(not(feature = "wasi"))]
use crate::spec::bindings::Runtime;
#[cfg(not(feature = "wasi"))]
use crate::spec::types::*;
#[cfg(feature = "wasi")]
use crate::wasi_spec::bindings::Runtime;
#[cfg(feature = "wasi")]
use crate::wasi_spec::types::*;
use crate::worker::*;
use bytes::BufMut;
use futures::TryStreamExt;
use http::{HeaderMap, Method, Uri};
use sha2::{Digest, Sha256};
use warp::{
    http::Response as WarpResponse,
    multipart::{FormData, Part},
    path::FullPath,
    Rejection, Reply,
};

pub async fn stage(worker: Worker, form: FormData) -> Result<impl Reply, Rejection> {
    let parts: Vec<Part> = form.try_collect().await.map_err(|e| {
        eprintln!("form error: {}", e);
        warp::reject::reject()
    })?;

    for p in parts {
        if p.name() == "file" {
            let value = p
                .stream()
                .try_fold(Vec::new(), |mut vec, data| {
                    vec.put(data);
                    async move { Ok(vec) }
                })
                .await
                .map_err(|e| {
                    eprintln!("reading file error: {}", e);
                    warp::reject::reject()
                })?;

            let hash = Sha256::new().chain_update(&value).finalize();

            let name = hex::encode(hash);
            let file_name = format!("./tableland_worker_runtime/workers/{}.wasm", name);
            tokio::fs::write(&file_name, &value).await.map_err(|e| {
                eprint!("error writing file: {}", e);
                warp::reject::reject()
            })?;

            worker.set(name.clone(), value).await.map_err(|e| {
                eprint!("error caching wasm runtime: {}", e);
                warp::reject::reject()
            })?;
            println!("cached new wasm runtime: {:?}", name);
        }
    }

    Ok("success")
}

pub async fn run(
    worker: Worker,
    full_path: FullPath,
    hash: String,
    query: String,
    headers: HeaderMap,
) -> Result<impl Reply, Rejection> {
    let rt: Runtime;
    let name = hash.clone();
    match worker.get(hash.clone()) {
        Some(r) => {
            rt = r;
        }
        None => {
            let file_name = format!("./tableland_worker_runtime/workers/{}.wasm", hash);
            let module = tokio::fs::read(&file_name).await.map_err(|e| {
                eprint!("error reading worker file: {}", e);
                warp::reject::reject()
            })?;
            rt = worker.set(hash, module).await.map_err(|e| {
                eprint!("error caching wasm runtime: {}", e);
                warp::reject::reject()
            })?;
        }
    }

    let mut path = full_path
        .as_str()
        .trim_start_matches(format!("/worker/{}", name).as_str())
        .to_string();
    if query.len() > 0 {
        path = format!("{}?{}", path, query);
    }
    if path.is_empty() {
        path = "/".to_string();
    }
    let uri = path.parse::<Uri>().unwrap();
    let req = Request::new(uri, Method::GET, headers, None);

    println!("fetching {} from worker {}", path, name);

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
