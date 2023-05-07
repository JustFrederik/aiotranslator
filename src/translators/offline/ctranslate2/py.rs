use pyo3::prelude::PyModule;
use pyo3::{PyResult, Python};
use std::path::PathBuf;

//TEMPORARY CODE - WILL BE REMOVED BY CPP VERSION
pub fn transalte_with_py(
    trans_path: PathBuf,
    tokens: Vec<Vec<String>>,
    device: String,
) -> PyResult<Vec<Vec<String>>> {
    Python::with_gil(|py| {
        let ctt2 = PyModule::from_code(py, include_str!("translator.py"), "ctt2.py", "ctt2")?;
        let translator =
            ctt2.call_method1("translate_init", (trans_path.to_str().unwrap(), device))?;
        let translate = ctt2.call_method1("translate", (translator, tokens))?;
        ctt2.call_method1("output_to_str_arr", (translate,))?
            .extract::<Vec<Vec<String>>>()
    })
}
