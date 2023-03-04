//! Router support for a Function.
//! This is a modified version of https://github.com/cloudflare/workers-rs/blob/main/worker/src/router.rs.
use http::Method;
use matchit::{Match, Node, Params};
use std::collections::HashMap;

use crate::CtxMut;

use super::{Error, Request, Response};

type HandlerFn<D> = fn(Request, CtxMut, RouteContext<D>) -> Result<Response, Error>;

/// Represents the URL parameters parsed from the path, e.g. a route with "/user/:id" pattern would
/// contain a single "id" key.
pub struct RouteParams(HashMap<String, String>);

impl RouteParams {
    fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
}

enum Handler<D> {
    Sync(HandlerFn<D>),
}

impl<D> Clone for Handler<D> {
    fn clone(&self) -> Self {
        match self {
            Self::Sync(func) => Self::Sync(*func),
        }
    }
}

/// A path-based HTTP router supporting exact-match or wildcard placeholders and shared data.
pub struct Router<D> {
    handlers: HashMap<Method, Node<Handler<D>>>,
    or_else_any_method: Node<Handler<D>>,
    data: D,
}

/// Container for a route's parsed parameters, data, and environment bindings from the Runtime (such
/// as KV Stores, Durable Objects, Variables, and Secrets).
pub struct RouteContext<D> {
    pub data: D,
    params: RouteParams,
}

impl<D> RouteContext<D> {
    /// Get a reference to the generic associated data provided to the `Router`.
    #[deprecated(since = "0.0.8", note = "please use the `data` field directly")]
    pub fn data(&self) -> &D {
        &self.data
    }

    /// Get a URL parameter parsed by the router, by the name of its match or wildecard placeholder.
    pub fn param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }
}

impl Router<()> {
    /// Construct a new `Router`. Or, call `Router::with_data(D)` to add arbitrary data that will be
    /// available to your various routes.
    pub fn new() -> Self {
        Self::with_data(())
    }
}

impl<D> Router<D> {
    /// Construct a new `Router` with arbitrary data that will be available to your various routes.
    pub fn with_data(data: D) -> Self {
        Self {
            handlers: HashMap::new(),
            or_else_any_method: Node::new(),
            data,
        }
    }

    /// Register an HTTP handler that will exclusively respond to HEAD requests.
    pub fn head(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::HEAD]);
        self
    }

    /// Register an HTTP handler that will exclusively respond to GET requests.
    pub fn get(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::GET]);
        self
    }

    /// Register an HTTP handler that will exclusively respond to POST requests.
    pub fn post(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::POST]);
        self
    }

    /// Register an HTTP handler that will exclusively respond to PUT requests.
    pub fn put(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::PUT]);
        self
    }

    /// Register an HTTP handler that will exclusively respond to PATCH requests.
    pub fn patch(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::PATCH]);
        self
    }

    /// Register an HTTP handler that will exclusively respond to DELETE requests.
    pub fn delete(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::DELETE]);
        self
    }

    /// Register an HTTP handler that will exclusively respond to OPTIONS requests.
    pub fn options(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), vec![Method::OPTIONS]);
        self
    }

    /// Register an HTTP handler that will respond to any requests.
    pub fn on(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.add_handler(pattern, Handler::Sync(func), all());
        self
    }

    /// Register an HTTP handler that will respond to all methods that are not handled explicitly by
    /// other handlers.
    pub fn or_else_any_method(mut self, pattern: &str, func: HandlerFn<D>) -> Self {
        self.or_else_any_method
            .insert(pattern, Handler::Sync(func))
            .unwrap_or_else(|e| panic!("failed to register route for {} pattern: {}", pattern, e));
        self
    }

    fn add_handler(&mut self, pattern: &str, func: Handler<D>, methods: Vec<Method>) {
        for method in methods {
            self.handlers
                .entry(method.clone())
                .or_insert_with(Node::new)
                .insert(pattern, func.clone())
                .unwrap_or_else(|e| {
                    panic!(
                        "failed to register {:?} route for {} pattern: {}",
                        method, pattern, e
                    )
                });
        }
    }

    /// Handle the request provided to the `Router`.
    pub fn run(self, req: Request, ctx: CtxMut) -> Result<Response, Error> {
        let (handlers, data, or_else_any_method_handler) = self.split();

        if let Some(handlers) = handlers.get(&req.method()) {
            if let Ok(Match { value, params }) = handlers.at(&req.uri().path()) {
                let route_info = RouteContext {
                    data,
                    params: params.into(),
                };
                return match value {
                    Handler::Sync(func) => (func)(req, ctx, route_info),
                };
            }
        }

        for method in all() {
            if method == Method::HEAD || method == Method::OPTIONS || method == Method::TRACE {
                continue;
            }
            if let Some(handlers) = handlers.get(&method) {
                if let Ok(Match { .. }) = handlers.at(&req.uri().path()) {
                    return Response::error("Method Not Allowed", 405);
                }
            }
        }

        if let Ok(Match { value, params }) = or_else_any_method_handler.at(&req.uri().path()) {
            let route_info = RouteContext {
                data,
                params: params.into(),
            };
            return match value {
                Handler::Sync(func) => (func)(req, ctx, route_info),
            };
        }

        Response::error("Not Found", 404)
    }
}

type NodeWithHandlers<D> = Node<Handler<D>>;

impl<D> Router<D> {
    fn split(self) -> (HashMap<Method, NodeWithHandlers<D>>, D, NodeWithHandlers<D>) {
        (self.handlers, self.data, self.or_else_any_method)
    }
}

impl From<Params<'_, '_>> for RouteParams {
    fn from(p: Params) -> Self {
        let mut route_params = RouteParams(HashMap::new());
        for (ident, value) in p.iter() {
            route_params.0.insert(ident.into(), value.into());
        }

        route_params
    }
}

fn all() -> Vec<Method> {
    vec![
        Method::HEAD,
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::PATCH,
        Method::DELETE,
        Method::OPTIONS,
        Method::CONNECT,
        Method::TRACE,
    ]
}
