use bytes::Bytes;
use serde_bytes::ByteBuf;
use tableland_std::Request;
use tableland_vm::{GasReport, VmError};
use warp::{
    http::Response as WarpResponse,
    http::{HeaderMap, HeaderValue, Method, StatusCode, Uri},
    path::FullPath,
    Rejection, Reply,
};

use crate::errors::StoreError;
use crate::store::Store;

const MAX_BODY_LENGTH: usize = 1024 * 1024;

pub async fn add_runtime(cid: String, store: Store) -> Result<impl Reply, Rejection> {
    store.add(cid.clone()).await.map_err(|e| {
        eprint!("error saving {}: {}", cid, e);
        warp::reject::reject()
    })?;

    println!("added {}", cid);

    Ok("success")
}

pub async fn invoke_runtime(
    cid: String,
    store: Store,
    method: Method,
    full_path: FullPath,
    query: String,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl Reply, Rejection> {
    if !body_allowed(method.clone(), body.len()) {
        return Err(warp::reject::reject());
    }

    let mut path = full_path
        .as_str()
        .trim_start_matches(format!("/v1/functions/{}", cid).as_str())
        .to_string();
    if !query.is_empty() {
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

    println!("{} {}{}", req.method(), cid, path);

    let out = store.run(cid.clone(), req).await;
    let report = out.1;
    let mut res = match out.0 {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error fetching {}: {}", cid, e);
            return match e {
                StoreError::Vm(VmError::GasDepletion { .. }) => Ok(build_response(
                    StatusCode::PAYMENT_REQUIRED,
                    HeaderMap::new(),
                    report,
                    Vec::new(),
                )),
                // todo handle other errors
                _ => Err(warp::reject::reject()),
            };
        }
    };

    Ok(build_response(
        StatusCode::from_u16(res.status_code()).unwrap(),
        res.headers().clone(),
        report,
        res.bytes().unwrap(),
    ))
}

fn body_allowed(method: Method, body_length: usize) -> bool {
    match method {
        Method::GET | Method::DELETE | Method::TRACE | Method::OPTIONS | Method::HEAD => {
            body_length == 0
        }
        _ => body_length <= MAX_BODY_LENGTH,
    }
}

fn build_response(
    status: StatusCode,
    mut headers: HeaderMap,
    report: GasReport,
    body: Vec<u8>,
) -> WarpResponse<Vec<u8>> {
    let wres = WarpResponse::builder().status(status).body(body).unwrap();
    let (mut parts, body) = wres.into_parts();

    headers.append("x-gas-limit", HeaderValue::from(report.limit));
    headers.append("x-gas-remaining", HeaderValue::from(report.remaining));
    headers.append("x-gas-external", HeaderValue::from(report.used_externally));
    headers.append("x-gas-internal", HeaderValue::from(report.used_internally));
    parts.headers = headers;

    WarpResponse::from_parts(parts, body)
}
