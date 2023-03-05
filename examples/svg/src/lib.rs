use handlebars::Handlebars;
use indoc::indoc;
use tableland_client_types::ReadOptions;
use tableland_std::{entry_point, CtxMut, Request, Response, Result, Router};

const VERSION: &str = "0.1.0";

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
        .get("/:id", |_, ctx, rctx| {
            if let Some(id) = rctx.param("id") {
                let data = ctx.tableland.read(
                    format!("select * from players_31337_7 where id = '{}'", id).as_str(),
                    ReadOptions::default(),
                )?;
                let player = match data.as_array().unwrap().get(0) {
                    Some(v) => v,
                    None => return Response::error("Not Found", 404),
                };
                let svg = Handlebars::new()
                    .render_template(template().as_str(), player)
                    .unwrap();

                return Response::from_svg(svg);
            }
            Response::error("Bad Request", 400)
        })
        .run(req, ctx)
}

fn template() -> String {
    (indoc! {r##"
        <svg class="healthbar" xmlns="http://www.w3.org/2000/svg" viewBox="0 -0.5 38 9" shape-rendering="crispEdges">
            <metadata>Made with Pixels to Svg https://codepen.io/shshaw/pen/XbxvNj</metadata>
            <path stroke="#222034" d="M2 0h34M1 1h1M36 1h1M0 2h1M3 2h32M37 2h1M0 3h1M2 3h1M35 3h1M37 3h1M0 4h1M2 4h1M35 4h1M37 4h1M0 5h1M2 5h1M35 5h1M37 5h1M0 6h1M3 6h32M37 6h1M1 7h1M36 7h1M2 8h34" />
            <path stroke="#ffffff" d="M2 1h34" />
            <path stroke="#f2f2f5" d="M1 2h2M35 2h2M1 3h1M36 3h1M1 4h1M36 4h1M1 5h1M36 5h1M1 6h2M35 6h2M2 7h34" />
            <path stroke="#323c39" d="M3 3h32" />
            <path stroke="#494d4c" d="M3 4h32M3 5h32" />
            <!-- Fill container -->
            <svg x="3" y="2.5" width="32" height="3">
                <rect width="{{health}}%" height="3" fill="#57e705"/>
                <rect width="{{health}}%" height="1" fill="#6aff03"/>
            </svg>
        </svg>
    "##}).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tableland_std::testing::{mock_dependencies, mock_get_request};

    #[test]
    fn call_fetch_works() {
        let mut ctx = mock_dependencies();
        let mut res = fetch(mock_get_request("/3"), ctx.as_mut()).unwrap();
        assert_eq!(res.status_code(), 200);

        let svg = res.text().unwrap();
        println!("{}", svg);
    }
}
