use bindings::*;
use maud::{html, DOCTYPE};
use serde::{Deserialize, Serialize};
mod utils;

#[derive(Serialize, Deserialize)]
struct Pet {
    name: String,
    r#type: String,
    owner_name: String,
}

const QUERY: &str = "select * from pets_31337_5;";

#[fp_export_impl(bindings)]
async fn fetch(req: Request) -> Result<Response, Error> {
    log(format!("ssr: {:?}", req));

    let data = query(QUERY.to_string(), ReadOptions::default()).await?;
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
    utils::init_panic_hook();
}
