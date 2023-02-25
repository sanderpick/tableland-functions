#![allow(dead_code)]
use fp_bindgen::{prelude::*, types::CargoDependency};
use once_cell::sync::Lazy;
use std::collections::{BTreeMap, BTreeSet};
use tableland_client_types::ReadOptions;
use tableland_worker_protocol::{Error, Request, Response};

fp_import! {
    /// Logs a message to the (development) console.
    fn log(message: String);

    /// Tableland query endpoint for plugins.
    async fn query(statement: String, options: ReadOptions) -> Result<serde_json::Value, Error>;
}

fp_export! {
    /// Fetch handler for the plugin.
    async fn fetch(request: Request) -> Result<Response, Error>;

    /// Called on the plugin to give it a chance to initialize.
    fn init();
}

const VERSION: &str = "0.1.0";
const AUTHORS: &str = r#"["Textile <contact@textile.io>"]"#;
const NAME: &str = "bindings";

static PLUGIN_DEPENDENCIES: Lazy<BTreeMap<&str, CargoDependency>> = Lazy::new(|| {
    BTreeMap::from([
        (
            "tableland_worker_protocol",
            CargoDependency::with_path("../../../tableland_worker_protocol"),
        ),
        (
            "tableland_client_types",
            CargoDependency::with_path("../../../lib/client/types"),
        ),
        (
            "fp-bindgen-support",
            CargoDependency::with_git_and_features(
                "https://github.com/fiberplane/fp-bindgen",
                BTreeSet::from(["async", "guest"]),
            ),
        ),
    ])
});

fn main() {
    for bindings_type in [
        BindingsType::RustPlugin(RustPluginConfig {
            name: NAME,
            authors: AUTHORS,
            version: VERSION,
            dependencies: PLUGIN_DEPENDENCIES.clone(),
        }),
        BindingsType::RustWasmerRuntime,
    ] {
        let output_path = format!("../bindings/{bindings_type}");

        fp_bindgen!(BindingConfig {
            bindings_type,
            path: &output_path,
        });
        println!("Generated bindings written to `{output_path}/`.");
    }
}
