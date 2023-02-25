use fp_bindgen::prelude::Serializable;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Serializable)]
#[fp(rust_module = "client_types")]
pub enum Format {
    Objects,
    Table,
}

impl Display for Format {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        write!(formatter, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Serializable)]
#[fp(rust_module = "client_types")]
pub struct ReadOptions {
    pub format: Format,
    pub extract: bool,
    pub unwrap: bool,
}

impl ReadOptions {
    pub fn format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    pub fn extract(mut self, extract: bool) -> Self {
        self.extract = extract;
        self
    }

    pub fn unwrap(mut self, unwrap: bool) -> Self {
        self.unwrap = unwrap;
        self
    }
}

impl Default for ReadOptions {
    fn default() -> Self {
        ReadOptions {
            format: Format::Objects,
            extract: false,
            unwrap: false,
        }
    }
}
