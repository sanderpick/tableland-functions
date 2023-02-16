mod handlers;
mod spec;
#[cfg(test)]
mod test;
mod worker;
use crate::handlers::*;
use crate::worker::*;
use std::{convert::Infallible, net::SocketAddr};
use warp::{http::StatusCode, Filter, Rejection, Reply};
mod config;
use crate::config::*;

#[tokio::main]
async fn main() {
    let config: Config = confy::load("tableland_worker", Some("config")).unwrap();
    let worker = Worker::new(config.clone());

    let add_runtime_route = warp::path("add")
        .and(warp::post())
        .and(with_worker(worker.clone()))
        .and(warp::path::param())
        .and_then(add_runtime);
    let invoke_runtime_route = warp::path("workers")
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
        .and_then(invoke_runtime);

    let router = add_runtime_route
        .or(invoke_runtime_route)
        .with(warp::cors().allow_any_origin())
        .recover(handle_rejection);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let saddr: SocketAddr = addr.parse().expect("Unable to parse server address");
    println!("Server started at {}", addr);
    warp::serve(router).run(saddr).await;
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
