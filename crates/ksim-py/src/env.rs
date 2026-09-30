//! Python classes over `ksim_env`; the contract is `python/ksim/_native.pyi`.

use numpy::{PyArray1, PyArray2, PyReadonlyArray1, PyReadonlyArray2};
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass(module = "ksim._native")]
#[derive(Debug)]
pub struct KsimEnv {
    _private: (),
}

#[pymethods]
impl KsimEnv {
    #[new]
    #[pyo3(signature = (scenario_path, *, max_replica_step = 3, control = "replicas", service_slots = None, reward_weights = (1.0, 0.5, 0.01)))]
    fn new(
        scenario_path: &str,
        max_replica_step: u32,
        control: &str,
        service_slots: Option<usize>,
        reward_weights: (f64, f64, f64),
    ) -> PyResult<Self> {
        let _ = (
            scenario_path,
            max_replica_step,
            control,
            service_slots,
            reward_weights,
        );
        todo!()
    }

    #[getter]
    fn observation_len(&self) -> usize {
        todo!()
    }

    #[getter]
    fn action_nvec(&self) -> Vec<u32> {
        todo!()
    }

    #[getter]
    fn episode_steps(&self) -> u64 {
        todo!()
    }

    fn reset<'py>(&mut self, py: Python<'py>, seed: u64) -> PyResult<Bound<'py, PyArray1<f32>>> {
        let _ = (py, seed);
        todo!()
    }

    fn step<'py>(
        &mut self,
        py: Python<'py>,
        action: PyReadonlyArray1<'py, u32>,
    ) -> PyResult<(Bound<'py, PyArray1<f32>>, f64, bool, bool)> {
        let _ = (py, action);
        todo!()
    }

    fn episode_metrics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let _ = py;
        todo!()
    }
}

#[pyclass(module = "ksim._native")]
#[derive(Debug)]
pub struct KsimVectorEnv {
    _private: (),
}

type VectorStep<'py> = (
    Bound<'py, PyArray2<f32>>,
    Bound<'py, PyArray1<f32>>,
    Bound<'py, PyArray1<bool>>,
    Bound<'py, PyArray1<bool>>,
);

#[pymethods]
impl KsimVectorEnv {
    #[new]
    #[pyo3(signature = (scenario_path, num_envs, *, max_replica_step = 3, control = "replicas", service_slots = None, reward_weights = (1.0, 0.5, 0.01)))]
    fn new(
        scenario_path: &str,
        num_envs: usize,
        max_replica_step: u32,
        control: &str,
        service_slots: Option<usize>,
        reward_weights: (f64, f64, f64),
    ) -> PyResult<Self> {
        let _ = (
            scenario_path,
            num_envs,
            max_replica_step,
            control,
            service_slots,
            reward_weights,
        );
        todo!()
    }

    #[getter]
    fn num_envs(&self) -> usize {
        todo!()
    }

    #[getter]
    fn observation_len(&self) -> usize {
        todo!()
    }

    #[getter]
    fn action_nvec(&self) -> Vec<u32> {
        todo!()
    }

    fn reset<'py>(&mut self, py: Python<'py>, seed: u64) -> PyResult<Bound<'py, PyArray2<f32>>> {
        let _ = (py, seed);
        todo!()
    }

    fn step<'py>(
        &mut self,
        py: Python<'py>,
        actions: PyReadonlyArray2<'py, u32>,
    ) -> PyResult<VectorStep<'py>> {
        let _ = (py, actions);
        todo!()
    }
}
