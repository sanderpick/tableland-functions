use fp_bindgen::prelude::Serializable;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Iter, HashMap};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Serializable)]
#[fp(rust_module = "tableland_worker_protocol")]
pub struct Headers {
    inner: HashMap<String, String>,
}

impl Headers {
    pub fn new() -> Self {
        Headers {
            inner: HashMap::default(),
        }
    }

    pub fn insert(&mut self, k: String, v: String) -> &mut Self {
        self.inner.insert(k, v);
        self
    }

    pub fn iter(&self) -> Iter<'_, String, String> {
        self.inner.iter()
    }
}
