mod handlers;
#[cfg(test)]
mod test;
mod wasi_spec;
mod worker;
pub use crate::handlers::*;
use crate::worker::*;
use std::convert::Infallible;
use warp::{http::StatusCode, Filter, Rejection, Reply};

#[tokio::main]
async fn main() {
    let worker = Worker::new();

    let stage_route = warp::path("stage")
        .and(warp::post())
        .and(with_worker(worker.clone()))
        .and(warp::multipart::form().max_length(5_000_000))
        .and_then(stage);
    let invoke_route = warp::path("worker")
        .and(with_worker(worker.clone()))
        .and(warp::method())
        .and(warp::path::full())
        .and(warp::path::param())
        .and(
            warp::query::raw()
                .or(warp::any().map(|| String::default()))
                .unify(),
        )
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .and_then(invoke);

    let router = stage_route
        .or(invoke_route)
        .with(warp::cors().allow_any_origin())
        .recover(handle_rejection);
    println!("Server started at localhost:3030");
    warp::serve(router).run(([127, 0, 0, 1], 3030)).await;
}

fn with_worker(worker: Worker) -> impl Filter<Extract = (Worker,), Error = Infallible> + Clone {
    warp::any().map(move || worker.clone())
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
