use bindings::*;
use maud::{html, DOCTYPE};
use serde::{Deserialize, Serialize};
use tableland_worker_utils::*;

const VERSION: &str = "0.1.0";

#[derive(Serialize, Deserialize)]
struct Pet {
    name: String,
    r#type: String,
    owner_name: String,
}

const QUERY: &str = "select * from pets_31337_4;";

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
        .get_async("/pets", |_, _| async move {
            let data = query(QUERY.to_string(), ReadOptions::default()).await?;
            let pets: Vec<Pet> = serde_json::from_value(data)?;
            let markup = html! {
                (DOCTYPE)
                html {
                    head {
                        title { "Example SSR Worker" }
                        meta description="Example showing how to render HTML in a Worker"
                        meta charset="utf-8";
                    }
                    body {
                        p { "We have some pets:" }
                        ul {
                            @for pet in &pets {
                                @let text = format!("{} is a {} owned by {}", pet.name, pet.r#type, pet.owner_name);
                                li { (text) }
                            }
                        }
                    }
                }
            };

            Response::from_html(markup.into_string())
        })
        .get("/worker-version", |_, _| Response::ok(VERSION))
        .run(req)
        .await
}

#[fp_export_impl(bindings)]
fn init() {
    init_panic_hook();
}
