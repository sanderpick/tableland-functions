use bindings::*;
use serde::{Deserialize, Serialize};
mod router;
use router::*;
mod utils;

const VERSION: &str = "0.1.0";

#[derive(Serialize, Deserialize)]
struct Politician {
    bioguide_id: String,
    position: String,
    state: String,
    party: String,
    first_name: String,
    last_name: String,
    birth_year: u32,
    service_start: u32,
    service_end: u32,
}

#[fp_export_impl(bindings)]
async fn fetch(req: Request) -> Result<Response, Error> {
    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get("/", |_, _| Response::ok("Hello from Workers!"))
        .get_async("/:id", |_, ctx| async move {
            if let Some(name) = ctx.param("id") {
                let opts = ReadOptions::default().unwrap(true);
                let data = query(
                    format!(
                        "select * from politicians_31337_7 where bioguide_id = '{}'",
                        name
                    ),
                    opts,
                )
                .await?;
                let politicians: Politician = serde_json::from_value(data)?;

                return Response::from_json(&politicians);
            }

            Response::error("Bad Request", 400)
        })
        .get("/worker-version", |_, _| Response::ok(VERSION))
        .run(req)
        .await
}

#[fp_export_impl(bindings)]
fn init() {
    utils::init_panic_hook();
}
