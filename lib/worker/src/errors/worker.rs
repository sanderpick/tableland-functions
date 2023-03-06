use crate::errors::StoreError;
use tableland_vm::GasReport;

#[derive(Clone, Debug)]
pub struct WorkerError {
    pub(crate) error: StoreError,
    pub(crate) report: Option<GasReport>,
}

impl warp::reject::Reject for WorkerError {}

impl WorkerError {
    pub(crate) fn new(error: StoreError, report: Option<GasReport>) -> Self {
        WorkerError { error, report }
    }
}
