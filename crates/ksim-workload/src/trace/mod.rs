//! `.ksimtrace`: per-service demand time series converted from public cluster traces.

mod format;
mod workload;

pub use format::{
    Column, HEADER_LEN, MAGIC, ServiceSeries, TraceError, TraceFile, VERSION, encode,
};
pub use workload::{Interpolation, TraceOptions, TraceWorkload};
