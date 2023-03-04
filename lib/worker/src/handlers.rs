use bytes::Bytes;
use http::{HeaderMap, HeaderValue, Method, StatusCode, Uri};
use serde_bytes::ByteBuf;
use tableland_std::Request;
use tableland_vm::{GasReport, VmError};
use warp::{http::Response as WarpResponse, path::FullPath, Rejection, Reply};

use crate::worker::{Worker, WorkerError};

pub async fn add_runtime(worker: Worker, cid: String) -> Result<impl Reply, Rejection> {
    worker.add(cid.clone()).await.map_err(|e| {
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

    let out = worker.run(cid.clone(), req).await;
    let report = out.1;
    let mut res = match out.0 {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error calling function {}: {}", cid, e);
            return match e {
                WorkerError::Vm(er) => match er {
                    VmError::GasDepletion { .. } => Ok(build_response(
                        StatusCode::PAYMENT_REQUIRED,
                        HeaderMap::new(),
                        report,
                        Vec::new(),
                    )),
                    // todo handle other errors
                    _ => Err(warp::reject::reject()),
                },
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
