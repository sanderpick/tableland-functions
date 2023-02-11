#[cfg(test)]
mod test;
mod wasi_spec;
#[cfg(not(feature = "wasi"))]
use crate::spec::bindings::Runtime;
#[cfg(not(feature = "wasi"))]
use crate::spec::types::*;
#[cfg(feature = "wasi")]
use crate::wasi_spec::bindings::Runtime;
#[cfg(feature = "wasi")]
use crate::wasi_spec::types::*;
use bytes::BufMut;
use futures::TryStreamExt;
use http::{header::HeaderName, HeaderMap, HeaderValue};
use sha2::{Digest, Sha256};
use std::convert::Infallible;
use warp::{
    http::Response as WarpResponse,
    http::StatusCode,
    multipart::{FormData, Part},
    path::FullPath,
    Filter, Rejection, Reply,
};

#[tokio::main]
async fn main() {
    let stage_route = warp::path("stage")
        .and(warp::post())
        .and(warp::multipart::form().max_length(5_000_000))
        .and_then(stage);
    let run_route = warp::path("worker")
        .and(warp::get())
        .and(warp::path::full())
        .and(warp::path::param())
        .and(
            warp::query::raw()
                .or(warp::any().map(|| String::default()))
                .unify(),
        )
        .and(warp::header::headers_cloned())
        .and_then(run);

    let router = stage_route.or(run_route).recover(handle_rejection);
    println!("Server started at localhost:3030");
    warp::serve(router).run(([127, 0, 0, 1], 3030)).await;
}

async fn stage(form: FormData) -> Result<impl Reply, Rejection> {
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
            tokio::fs::write(&file_name, value).await.map_err(|e| {
                eprint!("error writing file: {}", e);
                warp::reject::reject()
            })?;
            println!("created file: {:?}", name);
        }
    }

    Ok("success")
}

async fn run(
    full_path: FullPath,
    hash: String,
    query: String,
    headers: HeaderMap,
) -> Result<impl Reply, Rejection> {
    let file_name = format!("./tableland_worker_runtime/workers/{}.wasm", hash);
    let worker = tokio::fs::read(&file_name).await.map_err(|e| {
        eprint!("error reading worker file: {}", e);
        warp::reject::reject()
    })?;

    let rt = Runtime::new(worker).map_err(|e| {
        eprint!("error creating worker runtime: {}", e);
        warp::reject::reject()
    })?;
    rt.init().map_err(|e| {
        eprint!("error initializing worker: {}", e);
        warp::reject::reject()
    })?;

    let mut path = full_path
        .as_str()
        .trim_start_matches(format!("/worker/{}", hash).as_str())
        .to_string();
    if query.len() > 0 {
        path = format!("{}?{}", path, query);
    }
    let req = Request::new(path, Method::Get, from_header_map(headers), None);
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
    parts.headers = to_header_map(res.headers());
    Ok(WarpResponse::from_parts(parts, body))
}

fn from_header_map(h: HeaderMap) -> Headers {
    let mut headers = Headers::new();
    for (k, v) in h.iter() {
        let sv = match v.to_str() {
            Ok(sv) => sv,
            _ => continue,
        };
        headers.insert(k.to_string(), sv.to_string());
    }
    return headers;
}

fn to_header_map(h: &Headers) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (k, v) in h.iter() {
        headers.insert(
            HeaderName::from_bytes(k.as_bytes()).unwrap(),
            HeaderValue::from_str(v.as_str()).unwrap(),
        );
    }
    return headers;
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (StatusCode::BAD_REQUEST, "Payload too large".to_string())
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    Ok(warp::reply::with_status(message, code))
}
