mod env;

use pyo3::prelude::*;

fn pep440_version(version: &str) -> String {
    if let Some(base) = version.strip_suffix("-beta") {
        return format!("{base}b0");
    }
    if let Some((base, prerelease_number)) = version.split_once("-beta.") {
        return format!("{base}b{prerelease_number}");
    }
    version.to_owned()
}

#[pyfunction]
fn native_version() -> String {
    pep440_version(env!("CARGO_PKG_VERSION"))
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(native_version, module)?)?;
    module.add("OBSERVATION_LOW", ksim_env::layout::OBSERVATION_LOW)?;
    module.add("OBSERVATION_HIGH", ksim_env::layout::OBSERVATION_HIGH)?;
    module.add_class::<env::KsimEnv>()?;
    module.add_class::<env::KsimVectorEnv>()
}

#[cfg(test)]
mod tests {
    #[test]
    fn pep440_beta_version_is_normalized() {
        assert_eq!(super::pep440_version("0.1.0-beta"), "0.1.0b0");
    }

    #[test]
    fn stable_version_is_unchanged() {
        assert_eq!(super::pep440_version("0.1.0"), "0.1.0");
    }
}
