#[cfg(not(feature = "wasi"))]
use crate::spec::bindings::Runtime;
#[cfg(feature = "wasi")]
use crate::wasi_spec::bindings::Runtime;
use fp_bindgen_support::host::errors::{InvocationError, RuntimeError};
use std::fmt::{Debug, Display, Formatter};
use stretto::AsyncCache;

#[derive(Clone)]
pub struct Worker {
    cache: AsyncCache<String, Runtime>,
}

impl Worker {
    pub fn new() -> Self {
        Worker {
            cache: AsyncCache::new(12960, 1e6 as i64, tokio::spawn).unwrap(),
        }
    }

    pub async fn set(&self, name: String, module: Vec<u8>) -> Result<Runtime, WorkerError> {
        let rt = Runtime::new(module).map_err(|e| WorkerError::from(e))?;
        rt.init()?;
        self.cache.insert(name, rt.clone(), 1).await;
        Ok(rt)
    }

    pub fn get(&self, name: String) -> Option<Runtime> {
        match self.cache.get(name.as_str()) {
            Some(v) => Some(v.value().clone()),
            None => None,
        }
    }
}

#[derive(Debug)]
pub enum WorkerError {
    Runtime(RuntimeError),
    Invocation(InvocationError),
}

impl From<RuntimeError> for WorkerError {
    fn from(value: RuntimeError) -> Self {
        WorkerError::Runtime(value)
    }
}

impl From<InvocationError> for WorkerError {
    fn from(value: InvocationError) -> Self {
        WorkerError::Invocation(value)
    }
}

impl Display for WorkerError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match &self {
            WorkerError::Runtime(e) => write!(f, "{}", e.to_string()),
            WorkerError::Invocation(e) => write!(f, "{}", e.to_string()),
        }
    }
}
