use tableland_client_types::ReadOptions;
use tableland_std::{entry_point, CtxMut, Request, Response, Result, Router};

const VERSION: &str = "0.1.0";

#[entry_point]
pub fn fetch(req: Request, ctx: CtxMut) -> Result<Response> {
    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::default();

    // Add as many routes as your Function needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to get route parameters.
    router
        .get("/", |_, _, _| Response::ok("Hello from Tableland!"))
        .get("/version", |_, _, _| Response::ok(VERSION))
        .get("/:type", |_, ctx, rctx| {
            if let Some(t) = rctx.param("type") {
                let data = ctx.tableland.read(
                    format!("select * from pets_31337_4 as pets join homes_31337_2 as homes on pets.owner_name = homes.owner_name where type = '{}';", t).as_str(),
                    ReadOptions::default(),
                )?;
                return Response::from_json(&data);
            }
            Response::error("Bad Request", 400)
        })
        .run(req, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use tableland_std::testing::{mock_dependencies, mock_get_request};

    const MOCK_QUERY_RESPONSE: &[u8] = include_bytes!("../testdata/response.json");

    #[test]
    fn call_fetch_works() {
        let mut ctx = mock_dependencies(MOCK_QUERY_RESPONSE.to_vec());
        let mut res = fetch(mock_get_request("/dog"), ctx.as_mut()).unwrap();
        assert_eq!(res.status_code(), 200);

        let json = res.json::<Value>().unwrap();
        println!("{}", json);
    }
}
