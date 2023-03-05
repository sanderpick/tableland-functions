use crate::traits::Api;

/// Holds all external dependencies of the contract.
/// Designed to allow easy dependency injection at runtime.
/// This cannot be copied or cloned since it would behave differently
/// for mock storages and a bridge storage in the VM.
pub struct OwnedCtx<A: Api> {
    pub tableland: A,
}

pub struct CtxMut<'a> {
    pub tableland: &'a dyn Api,
}

#[derive(Clone)]
pub struct Ctx<'a> {
    pub tableland: &'a dyn Api,
}

// Use custom implementation on order to implement Copy in case `C` is not `Copy`.
// See "There is a small difference between the two: the derive strategy will also
// place a Copy bound on type parameters, which isn’t always desired."
// https://doc.rust-lang.org/std/marker/trait.Copy.html
impl<'a> Copy for Ctx<'a> {}

impl<A: Api> OwnedCtx<A> {
    pub fn as_ref(&'_ self) -> Ctx<'_> {
        Ctx {
            tableland: &self.tableland,
        }
    }

    pub fn as_mut(&'_ mut self) -> CtxMut<'_> {
        CtxMut {
            tableland: &self.tableland,
        }
    }
}

impl<'a> CtxMut<'a> {
    pub fn as_ref(&'_ self) -> Ctx<'_> {
        Ctx {
            tableland: self.tableland,
        }
    }

    pub fn branch(&'_ mut self) -> CtxMut<'_> {
        CtxMut {
            tableland: self.tableland,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{mock_dependencies, MockApi};
    use serde::{Deserialize, Serialize};

    // ensure we can call these many times, eg. as sub-calls
    fn execute(mut ctx: CtxMut) {
        execute2(ctx.branch());
        query(ctx.as_ref());
        execute2(ctx.branch());
    }
    fn execute2(_ctx: CtxMut) {}

    fn query(ctx: Ctx) {
        query2(ctx);
        query2(ctx);
    }
    fn query2(_ctx: Ctx) {}

    #[test]
    fn ensure_easy_reuse() {
        let mut deps = mock_dependencies(Vec::new());
        execute(deps.as_mut());
        query(deps.as_ref())
    }

    #[test]
    fn deps_implements_copy() {
        #[derive(Clone, Serialize, Deserialize)]
        struct MyQuery;

        // With C: Copy
        let owned = OwnedCtx {
            tableland: MockApi::default(),
        };
        let ctx: Ctx = owned.as_ref();
        let _copy1 = ctx;
        let _copy2 = ctx;

        // Without C: Copy
        let owned = OwnedCtx {
            tableland: MockApi::default(),
        };
        let ctx: Ctx = owned.as_ref();
        let _copy1 = ctx;
        let _copy2 = ctx;
    }
}
