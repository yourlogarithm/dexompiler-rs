mod dex_parsing;
mod manifest_parsing;
mod apk;
mod utils;

use crate::apk::ApkParseModel;

use pyo3::prelude::*;


#[pyfunction]
fn parse_apk(path: &str, dex_sequence_cap: usize, num_threads: usize) -> PyResult<ApkParseModel> {
    ApkParseModel::try_from_path(path, dex_sequence_cap, num_threads).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

