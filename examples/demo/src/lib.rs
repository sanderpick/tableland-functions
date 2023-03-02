use tableland_std::{entry_point, CtxMut, Request, Response, Result, Router};

const VERSION: &str = "0.1.0";

#[entry_point]
pub fn fetch(req: Request, ctx: CtxMut) -> Result<Response> {
    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get("/", |_, _, _| Response::ok("Hello from Workers!"))
        .get("/:type", |_, ctx, rctx| {
            if let Some(t) = rctx.param("type") {
                // let opts = ReadOptions::default().unwrap(true);
                let data = ctx.tableland.read(
                    format!("select * from pets_31337_4 where type = '{}'", t).as_str(),
                    // opts,
                )?;
                return Response::from_json(&data);
            }
            Response::error("Bad Request", 400)
        })
        .get("/worker-version", |_, _, _| Response::ok(VERSION))
        .run(req, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tableland_std::testing::{mock_dependencies, mock_get_request, MockApi};
    use tableland_std::OwnedCtx;

    #[test]
    fn call_fetch_works() {
        let mut ctx = mock_dependencies();
        let res = fetch(mock_get_request(), ctx.as_mut()).unwrap();
        assert_eq!(res.status_code(), 200);
    }
}
