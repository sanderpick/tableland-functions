use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Format {
    Objects,
    Table,
}

impl std::fmt::Display for Format {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
