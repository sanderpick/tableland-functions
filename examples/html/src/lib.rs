use maud::{html, DOCTYPE};
use serde::{Deserialize, Serialize};
use tableland_client_types::ReadOptions;
use tableland_std::{entry_point, CtxMut, Request, Response, Result, Router};

const VERSION: &str = "0.1.0";

#[derive(Serialize, Deserialize)]
struct Pet {
    name: String,
    r#type: String,
    owner_name: String,
}

#[entry_point]
pub fn fetch(req: Request, ctx: CtxMut) -> Result<Response> {
    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Function needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to get route parameters.
    router
        .get("/", |_, _, _| Response::ok("Hello from Tableland!"))
        .get("/version", |_, _, _| Response::ok(VERSION))
        .get("/:type", |_, ctx, rctx| {
            if let Some(t) = rctx.param("type") {
                let data = ctx.tableland.read(
                    format!("select * from pets_31337_4 where type = '{}'", t).as_str(),
                    ReadOptions::default(),
                )?;
                let pets: Vec<Pet> = serde_json::from_value(data)?;
                let markup = html! {
                    (DOCTYPE)
                    html {
                        head {
                            title { "Example HTML Function" }
                            meta description="Example showing how to render HTML from a Function"
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
                return Response::from_html(markup.into_string());
            }
            Response::error("Bad Request", 400)
        })
        .run(req, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tableland_std::testing::{mock_dependencies, mock_get_request};

    #[test]
    fn call_fetch_works() {
        let mut ctx = mock_dependencies();
        let mut res = fetch(mock_get_request("/dog"), ctx.as_mut()).unwrap();
        assert_eq!(res.status_code(), 200);

        let json = res.text().unwrap();
        println!("{}", json);
    }
}
