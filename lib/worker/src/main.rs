mod backend;
mod config;
mod handlers;
mod instance;
mod store;
#[cfg(test)]
mod test;

use std::{convert::Infallible, net::SocketAddr};
use warp::{http::StatusCode, Filter, Rejection, Reply};

use crate::config::Config;
use crate::handlers::{add_runtime, invoke_runtime};
use crate::store::Store;

#[tokio::main]
async fn main() {
    let config: Config = confy::load("tableland_worker", Some("config")).unwrap();
    let store = Store::new(config.clone());

    let add_runtime_route = warp::path!("v1" / "add" / String)
        .and(warp::post())
        .and(with_store(store.clone()))
        .and_then(add_runtime);
    let invoke_runtime_route = warp::path!("v1" / "functions" / String / ..)
        .and(with_store(store.clone()))
        .and(warp::method())
        .and(warp::path::full())
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

fn with_store(store: Store) -> impl Filter<Extract = (Store,), Error = Infallible> + Clone {
    warp::any().map(move || store.clone())
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (StatusCode::BAD_REQUEST, "Payload too large".to_string())
    } else if err.find::<warp::reject::LengthRequired>().is_some() {
        (StatusCode::LENGTH_REQUIRED, "Length required".to_string())
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };
    Ok(warp::reply::with_status(message, code))
}
