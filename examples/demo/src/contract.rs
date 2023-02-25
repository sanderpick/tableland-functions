use tableland_std::{
    entry_point,
    // from_slice, to_binary, to_vec, Binary, Deps,
    DepsMut,
    Env,
    Response,
    StdResult,
};

#[entry_point]
pub fn fetch(_deps: DepsMut, _env: Env) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tableland_std::testing::{mock_dependencies, mock_env, MockApi};
    use tableland_std::{from_binary, OwnedDeps};

    /// Instantiates a function
    fn create_contract() -> OwnedDeps<MockApi> {
        let mut deps = mock_dependencies();
        let res = fetch(deps.as_mut(), mock_env()).unwrap();
        assert_eq!(0, res.data.is_none());
        deps
    }

    #[test]
    fn basic_fetch() {
        let (deps, _) = create_contract();
    }
}
