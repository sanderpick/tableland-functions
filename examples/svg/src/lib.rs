use serde_json::Value;
use std::collections::HashMap;
use strfmt::Format;
use tableland_client_types::ReadOptions;
use tableland_std::{entry_point, CtxMut, Request, Response, Result, Router};

const VERSION: &str = "0.1.0";

const METADATA_QUERY: &str = r#"select json_object(
    'name','Player #'||id,
    'attributes',json_array(
        json_object(
            'trait_type','name',
            'value',name
        )
        json_object(
            'display_type','number',
            'trait_type','health',
            'value',health
        )
)) from players_31337_7 where id = {id};"#;

const SVG: &str = r##"<svg class="healthbar" xmlns="http://www.w3.org/2000/svg" viewBox="0 -0.5 38 9" shape-rendering="crispEdges">
    <metadata>Made with Pixels to Svg https://codepen.io/shshaw/pen/XbxvNj</metadata>
    <path stroke="#222034" d="M2 0h34M1 1h1M36 1h1M0 2h1M3 2h32M37 2h1M0 3h1M2 3h1M35 3h1M37 3h1M0 4h1M2 4h1M35 4h1M37 4h1M0 5h1M2 5h1M35 5h1M37 5h1M0 6h1M3 6h32M37 6h1M1 7h1M36 7h1M2 8h34" />
    <path stroke="#ffffff" d="M2 1h34" />
    <path stroke="#f2f2f5" d="M1 2h2M35 2h2M1 3h1M36 3h1M1 4h1M36 4h1M1 5h1M36 5h1M1 6h2M35 6h2M2 7h34" />
    <path stroke="#323c39" d="M3 3h32" />
    <path stroke="#494d4c" d="M3 4h32M3 5h32" />
    <!-- Fill container -->
    <svg x="3" y="2.5" width="32" height="3">
        <rect width="{health}%" height="3" fill="#57e705"/>
        <rect width="{health}%" height="1" fill="#6aff03"/>
    </svg>
</svg>"##;

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
        // Renders player as NFT metadata
        .get("/:id", |_, ctx, rctx| {
            if let Some(id) = rctx.param("id") {
                let vars = HashMap::from([("id".to_string(), id.as_str())]);
                let mut data = ctx.tableland.read(
                    METADATA_QUERY.format(&vars).unwrap().as_str(),
                    ReadOptions::default().extract(true).unwrap(true),
                )?;
                let player = data.as_object_mut().unwrap();
                player.insert(
                    "image".to_string(),
                    Value::from(format!("tbl://{}/image", rctx.id())),
                );

                return Response::from_json(player);
            }
            Response::error("Bad Request", 400)
        })
        // Renders player as SVG image
        .get("/:id/image", |_, ctx, rctx| {
            if let Some(id) = rctx.param("id") {
                let data = ctx.tableland.read(
                    format!("select * from players_31337_7 where id = {};", id).as_str(),
                    ReadOptions::default().unwrap(true),
                )?;
                let player = data.as_object().unwrap();

                let vars = HashMap::from([(
                    "health".to_string(),
                    player.get("health").unwrap().as_u64().unwrap(),
                )]);
                let svg = SVG.format(&vars).unwrap();

                return Response::from_svg(svg);
            }
            Response::error("Bad Request", 400)
        })
        .run(req, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tableland_std::testing::{mock_dependencies, mock_get_request};

    const MOCK_QUERY_RESPONSE_METADATA: &[u8] = include_bytes!("../testdata/response1.json");
    const MOCK_QUERY_RESPONSE_IMAGE: &[u8] = include_bytes!("../testdata/response2.json");

    #[test]
    fn call_fetch_metadata_works() {
        let mut ctx = mock_dependencies(MOCK_QUERY_RESPONSE_METADATA.to_vec());
        let mut res = fetch(mock_get_request("/3"), ctx.as_mut()).unwrap();
        assert_eq!(res.status_code(), 200);

        let json = res.json::<Value>().unwrap();
        println!("{}", json);
    }

    #[test]
    fn call_fetch_image_works() {
        let mut ctx = mock_dependencies(MOCK_QUERY_RESPONSE_IMAGE.to_vec());
        let mut res = fetch(mock_get_request("/3/image"), ctx.as_mut()).unwrap();
        assert_eq!(res.status_code(), 200);

        let svg = res.text().unwrap();
        println!("{}", svg);
    }
}
