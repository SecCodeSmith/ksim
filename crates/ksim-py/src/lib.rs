mod env;

use pyo3::prelude::*;

#[pyfunction]
fn native_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(native_version, module)?)?;
    module.add("OBSERVATION_LOW", ksim_env::layout::OBSERVATION_LOW)?;
    module.add("OBSERVATION_HIGH", ksim_env::layout::OBSERVATION_HIGH)?;
    module.add_class::<env::KsimEnv>()?;
    module.add_class::<env::KsimVectorEnv>()
}
