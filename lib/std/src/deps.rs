use crate::traits::Api;

/// Holds all external dependencies of the contract.
/// Designed to allow easy dependency injection at runtime.
/// This cannot be copied or cloned since it would behave differently
/// for mock storages and a bridge storage in the VM.
pub struct OwnedDeps<A: Api> {
    pub api: A,
}

pub struct DepsMut<'a> {
    pub api: &'a dyn Api,
}

#[derive(Clone)]
pub struct Deps<'a> {
    pub api: &'a dyn Api,
}

// Use custom implementation on order to implement Copy in case `C` is not `Copy`.
// See "There is a small difference between the two: the derive strategy will also
// place a Copy bound on type parameters, which isn’t always desired."
// https://doc.rust-lang.org/std/marker/trait.Copy.html
impl<'a> Copy for Deps<'a> {}

impl<A: Api> OwnedDeps<A> {
    pub fn as_ref(&'_ self) -> Deps<'_> {
        Deps { api: &self.api }
    }

    pub fn as_mut(&'_ mut self) -> DepsMut<'_> {
        DepsMut { api: &self.api }
    }
}

impl<'a> DepsMut<'a> {
    pub fn as_ref(&'_ self) -> Deps<'_> {
        Deps { api: self.api }
    }

    pub fn branch(&'_ mut self) -> DepsMut<'_> {
        DepsMut { api: self.api }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{mock_dependencies, MockApi};
    use serde::{Deserialize, Serialize};

    // ensure we can call these many times, eg. as sub-calls
    fn execute(mut deps: DepsMut) {
        execute2(deps.branch());
        query(deps.as_ref());
        execute2(deps.branch());
    }
    fn execute2(_deps: DepsMut) {}

    fn query(deps: Deps) {
        query2(deps);
        query2(deps);
    }
    fn query2(_deps: Deps) {}

    #[test]
    fn ensure_easy_reuse() {
        let mut deps = mock_dependencies();
        execute(deps.as_mut());
        query(deps.as_ref())
    }

    #[test]
    fn deps_implements_copy() {
        #[derive(Clone, Serialize, Deserialize)]
        struct MyQuery;

        // With C: Copy
        let owned = OwnedDeps {
            api: MockApi::default(),
        };
        let deps: Deps = owned.as_ref();
        let _copy1 = deps;
        let _copy2 = deps;

        // Without C: Copy
        let owned = OwnedDeps {
            api: MockApi::default(),
        };
        let deps: Deps = owned.as_ref();
        let _copy1 = deps;
        let _copy2 = deps;
    }
}
