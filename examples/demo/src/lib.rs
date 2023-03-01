use serde_bytes::ByteBuf;
use serde_json::to_vec;
use tableland_std::{entry_point, CtxMut, Request, Response, StdResult};

#[entry_point]
pub fn fetch(_req: Request, ctx: CtxMut) -> StdResult<Response> {
    let res = ctx.tableland.read("select * from pets_31337_4")?;
    let json = to_vec(&res).unwrap();

    Ok(Response::new().set_data(ByteBuf::from(json)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_slice, to_string, Value};
    use tableland_std::testing::{mock_dependencies, mock_request, MockApi};
    use tableland_std::OwnedCtx;

    fn create_function() -> OwnedCtx<MockApi> {
        mock_dependencies()
    }

    #[test]
    fn call_fetch_works() {
        let mut ctx = create_function();
        let res = fetch(mock_request(), ctx.as_mut()).unwrap();
        assert_eq!(true, res.data.is_some());

        let data = res.data.unwrap().into_vec();
        let json = from_slice::<Value>(data.as_slice()).unwrap();
        println!("{}", to_string(&json).unwrap());
    }
}
