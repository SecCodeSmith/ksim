//! Python classes over `ksim_env`; the contract is `python/ksim/_native.pyi`.

use ksim_bench::scenario::Scenario;
use ksim_env::{Control, Env, EnvConfig, VectorEnv, WeightedRewardWeights};
use numpy::{
    Element, IntoPyArray, PyArray1, PyArray2, PyArrayMethods, PyReadonlyArray, PyReadonlyArray1,
    PyReadonlyArray2, PyUntypedArrayMethods, ndarray::Dimension,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict};

fn value_error(error: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn parse_control(name: &str) -> PyResult<Control> {
    match name {
        "replicas" => Ok(Control::Replicas),
        "replicas_and_nodes" => Ok(Control::ReplicasAndNodes),
        other => Err(value_error(format!(
            "unknown control {other:?}, expected \"replicas\" or \"replicas_and_nodes\""
        ))),
    }
}

/// The array's elements in logical (row-major) order, copying only when not contiguous.
fn action_values<T: Element + Copy, D: Dimension>(array: &PyReadonlyArray<'_, T, D>) -> Vec<T> {
    match array.as_slice() {
        Ok(slice) => slice.to_vec(),
        Err(_) => array.as_array().iter().copied().collect(),
    }
}

/// A scenario together with the environment configuration it is simulated under.
#[derive(Debug, Clone)]
struct Setup {
    scenario: Scenario,
    config: EnvConfig,
}

impl Setup {
    fn load(
        scenario_path: &str,
        max_replica_step: u32,
        control: &str,
        service_slots: Option<usize>,
        reward_weights: (f64, f64, f64),
    ) -> PyResult<Self> {
        let (slo, cost, actions) = reward_weights;
        Ok(Self {
            scenario: Scenario::from_toml_file(scenario_path).map_err(value_error)?,
            config: EnvConfig {
                max_replica_step,
                control: parse_control(control)?,
                service_slots,
                reward: WeightedRewardWeights { slo, cost, actions },
            },
        })
    }
}

#[pyclass(module = "ksim._native", unsendable)]
#[derive(Debug)]
pub struct KsimEnv {
    env: Env,
}

impl KsimEnv {
    fn from_setup(setup: &Setup) -> PyResult<Self> {
        Ok(Self {
            env: Env::new(&setup.scenario, &setup.config).map_err(value_error)?,
        })
    }

    fn observation_buffer(&self) -> Vec<f32> {
        vec![0.0; self.env.layout().observation_len()]
    }
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
        Self::from_setup(&Setup::load(
            scenario_path,
            max_replica_step,
            control,
            service_slots,
            reward_weights,
        )?)
    }

    #[getter]
    fn observation_len(&self) -> usize {
        self.env.layout().observation_len()
    }

    #[getter]
    fn action_nvec(&self) -> Vec<u32> {
        self.env.layout().action_nvec()
    }

    #[getter]
    fn episode_steps(&self) -> u64 {
        self.env.episode_steps()
    }

    fn reset<'py>(&mut self, py: Python<'py>, seed: u64) -> PyResult<Bound<'py, PyArray1<f32>>> {
        let mut observation = self.observation_buffer();
        py.detach(|| self.env.reset(seed, &mut observation))
            .map_err(value_error)?;
        Ok(observation.into_pyarray(py))
    }

    #[allow(clippy::needless_pass_by_value)] // PyO3 extracts array arguments by value
    fn step<'py>(
        &mut self,
        py: Python<'py>,
        action: PyReadonlyArray1<'py, u32>,
    ) -> PyResult<(Bound<'py, PyArray1<f32>>, f64, bool, bool)> {
        let action = action_values(&action);
        let mut observation = self.observation_buffer();
        let outcome = py
            .detach(|| self.env.step(&action, &mut observation))
            .map_err(value_error)?;
        Ok((
            observation.into_pyarray(py),
            outcome.reward,
            outcome.terminated,
            outcome.truncated,
        ))
    }

    fn episode_metrics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        self.env.episode_metrics().into_py_dict(py)
    }
}

#[pyclass(module = "ksim._native", unsendable)]
#[derive(Debug)]
pub struct KsimVectorEnv {
    envs: VectorEnv,
}

type VectorStep<'py> = (
    Bound<'py, PyArray2<f32>>,
    Bound<'py, PyArray1<f32>>,
    Bound<'py, PyArray1<bool>>,
    Bound<'py, PyArray1<bool>>,
);

impl KsimVectorEnv {
    fn single_observation_len(&self) -> usize {
        self.envs.first().layout().observation_len()
    }

    fn batch_shape(&self, width: usize) -> (usize, usize) {
        (self.envs.num_envs(), width)
    }
}

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
        let setup = Setup::load(
            scenario_path,
            max_replica_step,
            control,
            service_slots,
            reward_weights,
        )?;
        let envs = VectorEnv::new(&setup.scenario, &setup.config, num_envs).map_err(value_error)?;
        Ok(Self { envs })
    }

    #[getter]
    fn num_envs(&self) -> usize {
        self.envs.num_envs()
    }

    #[getter]
    fn observation_len(&self) -> usize {
        self.single_observation_len()
    }

    #[getter]
    fn action_nvec(&self) -> Vec<u32> {
        self.envs.first().layout().action_nvec()
    }

    fn reset<'py>(&mut self, py: Python<'py>, seed: u64) -> PyResult<Bound<'py, PyArray2<f32>>> {
        let shape = self.batch_shape(self.single_observation_len());
        let mut observations = vec![0.0; shape.0 * shape.1];
        py.detach(|| self.envs.reset(seed, &mut observations))
            .map_err(value_error)?;
        observations.into_pyarray(py).reshape(shape)
    }

    #[allow(clippy::needless_pass_by_value)] // PyO3 extracts array arguments by value
    fn step<'py>(
        &mut self,
        py: Python<'py>,
        actions: PyReadonlyArray2<'py, u32>,
    ) -> PyResult<VectorStep<'py>> {
        let action_len = self.envs.first().layout().action_len();
        let expected = self.batch_shape(action_len);
        if actions.shape() != [expected.0, expected.1] {
            return Err(value_error(format!(
                "actions have shape {:?}, expected {expected:?}",
                actions.shape()
            )));
        }
        let actions = action_values(&actions);
        let num_envs = self.envs.num_envs();
        let shape = self.batch_shape(self.single_observation_len());
        let mut observations = vec![0.0; shape.0 * shape.1];
        let mut rewards = vec![0.0; num_envs];
        let mut terminated = vec![false; num_envs];
        let mut truncated = vec![false; num_envs];
        py.detach(|| {
            self.envs.step(
                &actions,
                &mut observations,
                &mut rewards,
                &mut terminated,
                &mut truncated,
            )
        })
        .map_err(value_error)?;
        Ok((
            observations.into_pyarray(py).reshape(shape)?,
            rewards.into_pyarray(py),
            terminated.into_pyarray(py),
            truncated.into_pyarray(py),
        ))
    }
}
