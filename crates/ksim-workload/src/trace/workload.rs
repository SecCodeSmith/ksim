use std::sync::Arc;

use ksim_core::{
    ids::ServiceId,
    time::{SimDuration, SimTime},
    workload::{ServiceDemand, WorkloadSource},
};

use super::format::{Column, TraceError, TraceFile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interpolation {
    Step,
    Linear,
}

/// How simulated time maps onto the trace: trace time `= start + now * compress`, sampled at
/// tick index `trace time / tick`. Past the last tick the trace wraps around when `wrap`,
/// otherwise holds the last value. `scale` multiplies request rate and CPU (memory is a
/// footprint and is not scaled). `services` selects and orders services by name; `None`
/// keeps every service in file order.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceOptions {
    pub services: Option<Vec<String>>,
    pub start: SimDuration,
    pub compress: f64,
    pub scale: f64,
    pub interpolation: Interpolation,
    pub wrap: bool,
}

impl Default for TraceOptions {
    fn default() -> Self {
        Self {
            services: None,
            start: SimDuration::ZERO,
            compress: 1.0,
            scale: 1.0,
            interpolation: Interpolation::Step,
            wrap: false,
        }
    }
}

/// Replays a [`TraceFile`] as a pure function of time.
#[derive(Debug)]
pub struct TraceWorkload {
    file: Arc<TraceFile>,
    indices: Vec<usize>,
    options: TraceOptions,
}

/// Two adjacent ticks and the weight of the later one.
struct Sample {
    lower: usize,
    upper: usize,
    upper_weight: f64,
}

impl TraceWorkload {
    /// # Errors
    /// [`TraceError::UnknownService`] for a selected name missing from the file, and
    /// [`TraceError::Malformed`] for a non-positive or non-finite `compress` or `scale`, or a
    /// trace without ticks.
    pub fn new(file: Arc<TraceFile>, options: TraceOptions) -> Result<Self, TraceError> {
        require_positive_finite(options.compress, "compress")?;
        require_positive_finite(options.scale, "scale")?;
        if file.tick_count() == 0 {
            return Err(TraceError::Malformed("trace has no ticks".to_owned()));
        }
        let indices = select_services(&file, options.services.as_deref())?;
        Ok(Self {
            file,
            indices,
            options,
        })
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "the position is non-negative and float-to-int casts saturate"
    )]
    fn sample_at(&self, now: SimTime) -> Sample {
        let ticks = self.file.tick_count();
        let trace_secs =
            self.options.start.as_secs_f64() + now.as_secs_f64() * self.options.compress;
        let position = trace_secs / self.file.tick().as_secs_f64();
        let lower = position.floor() as usize;
        let upper_weight = match self.options.interpolation {
            Interpolation::Step => 0.0,
            Interpolation::Linear => position - position.floor(),
        };
        if self.options.wrap {
            Sample {
                lower: lower % ticks,
                upper: (lower % ticks + 1) % ticks,
                upper_weight,
            }
        } else {
            let last = ticks - 1;
            Sample {
                lower: lower.min(last),
                upper: lower.saturating_add(1).min(last),
                upper_weight: if lower >= last { 0.0 } else { upper_weight },
            }
        }
    }

    fn interpolated(&self, column: Column, service: usize, sample: &Sample) -> f64 {
        let at = |tick| f64::from(self.file.value(column, service, tick).unwrap_or_default());
        let lower = at(sample.lower);
        lower + (at(sample.upper) - lower) * sample.upper_weight
    }
}

fn require_positive_finite(value: f64, name: &str) -> Result<(), TraceError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(TraceError::Malformed(format!(
            "{name} must be positive and finite, got {value}"
        )))
    }
}

fn select_services(file: &TraceFile, names: Option<&[String]>) -> Result<Vec<usize>, TraceError> {
    let Some(names) = names else {
        return Ok((0..file.service_names().len()).collect());
    };
    names
        .iter()
        .map(|name| {
            file.service_names()
                .iter()
                .position(|candidate| candidate == name)
                .ok_or_else(|| TraceError::UnknownService(name.clone()))
        })
        .collect()
}

impl WorkloadSource for TraceWorkload {
    fn services(&self) -> usize {
        self.indices.len()
    }

    fn demand(&self, service: ServiceId, now: SimTime) -> ServiceDemand {
        let Some(&column_service) = self.indices.get(service.index()) else {
            return ServiceDemand::default();
        };
        let sample = self.sample_at(now);
        let read = |column| self.interpolated(column, column_service, &sample);
        ServiceDemand {
            request_rate: read(Column::RequestRate) * self.options.scale,
            cpu_millis: read(Column::CpuMillis) * self.options.scale,
            memory_bytes: read(Column::MemoryBytes),
        }
    }
}
