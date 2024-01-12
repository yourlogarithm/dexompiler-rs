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

#[pymodule]
fn dexompiler(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_apk, m)?)?;
    m.add_class::<ApkParseModel>()?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_apk() {
        let apk = ApkParseModel::try_from_path("resources/F-Droid.apk", 0, 4).unwrap();
        println!("{:?}", apk);
    }
}