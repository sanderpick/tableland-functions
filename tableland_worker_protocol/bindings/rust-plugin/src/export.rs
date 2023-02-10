use crate::types::*;

/// Fetch handler for the plugin.
#[fp_bindgen_support::fp_export_signature]
pub async fn fetch(request: Request) -> Result<Response, Error>;

/// Called on the plugin to give it a chance to initialize.
#[fp_bindgen_support::fp_export_signature]
pub fn init();
