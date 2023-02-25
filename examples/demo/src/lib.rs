use tableland_std::{entry_point, DepsMut, Env, Response, StdResult};

#[entry_point]
pub fn fetch(_deps: DepsMut, _env: Env) -> StdResult<Response> {
    _deps
        .api
        .debug(format!("block info: {:?}", _env.block).as_str());
    let res = _deps.api.hello("foo")?;
    Ok(Response::new().set_data(res))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tableland_std::testing::{mock_dependencies, mock_env, MockApi};
    use tableland_std::OwnedDeps;

    fn create_function() -> OwnedDeps<MockApi> {
        let mut deps = mock_dependencies();
        let res = fetch(deps.as_mut(), mock_env()).unwrap();
        println!("{:?}", res.data);
        assert_eq!(true, res.data.is_some());
        deps
    }

    #[test]
    fn basic_fetch() {
        create_function();
    }
}
