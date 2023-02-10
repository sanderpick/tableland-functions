use bindings::*;
use maud::html;
use std::panic;

fn init_panic_hook() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(|info| log(info.to_string())));
    });
}

#[fp_export_impl(bindings)]
async fn fetch(request: Request) -> Result<Response, Error> {
    log(format!("{:?}", request));

    let data = query("select * from politicians_31337_7;".to_string()).await?;
    log(format!("{:?}", data));
    let list = data.as_array();

    let name = "Lyra";
    let markup = html! {
        p { "Hi, " (name) "!" }
    };

    Response::from_html(markup.into_string())
}

#[fp_export_impl(bindings)]
fn init() {
    init_panic_hook();
}
