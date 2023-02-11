use bindings::*;
use maud::{html, DOCTYPE};
use serde::{Deserialize, Serialize};
use std::panic;

fn init_panic_hook() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(|info| log(info.to_string())));
    });
}

#[derive(Serialize, Deserialize)]
struct Pet {
    name: String,
    r#type: String,
    owner_name: String,
}

const QUERY: &str = "select * from pets_31337_5;";

#[fp_export_impl(bindings)]
async fn fetch(request: Request) -> Result<Response, Error> {
    log(format!("{:?}", request));

    let data = query(QUERY.to_string()).await?;
    let pets: Vec<Pet> = serde_json::from_value(data)?;
    let markup = html! {
        (DOCTYPE)
        html {
            head {
                title { "Example SSR Worker" }
                meta description="Example showing how to render HTML in a Worker"
                meta charset="utf-8";
            }
            body {
                p { "We have some pets:" }
                ul {
                    @for pet in &pets {
                        @let text = format!("{} is a {} owned by {}", pet.name, pet.r#type, pet.owner_name);
                        li { (text) }
                    }
                }
            }
        }
    };

    Response::from_html(markup.into_string())
}

#[fp_export_impl(bindings)]
fn init() {
    init_panic_hook();
}
