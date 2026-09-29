use std::path::Path;

use ksim_core::time::SimDuration;

pub const MAGIC: [u8; 8] = *b"KSIMTRC1";
pub const VERSION: u32 = 1;
pub const HEADER_LEN: usize = 48;

/// Errors reading or writing a `.ksimtrace` file.
#[derive(Debug, thiserror::Error)]
pub enum TraceError {
    #[error("cannot read trace: {0}")]
    Io(#[from] std::io::Error),
    #[error("not a .ksimtrace file")]
    BadMagic,
    #[error("unsupported .ksimtrace version {0}")]
    UnsupportedVersion(u32),
    #[error("trace is truncated: expected {expected} bytes, found {actual}")]
    Truncated { expected: u64, actual: u64 },
    #[error("trace header is inconsistent: {0}")]
    Malformed(String),
    #[error("service name {0} is not valid UTF-8")]
    InvalidName(usize),
    #[error("unknown service {0:?}")]
    UnknownService(String),
}

/// The three demand columns, stored in this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    RequestRate,
    CpuMillis,
    MemoryBytes,
}

/// One service's series; all three vectors have the trace's tick count.
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceSeries {
    pub name: String,
    pub request_rate: Vec<f32>,
    pub cpu_millis: Vec<f32>,
    pub memory_bytes: Vec<f32>,
}

/// A validated `.ksimtrace` held in memory.
///
/// Layout, all integers and floats little-endian:
/// - header (48 bytes): magic `KSIMTRC1`, `u32` version, `u32` tick seconds (> 0),
///   `u32` service count, `u32` tick count, `u64` names offset (= 48), `u64` names length,
///   `u64` data offset;
/// - names: per service a `u16` byte length followed by UTF-8 bytes;
/// - zero padding up to the data offset, the smallest multiple of 4 at or after the names;
/// - data: columns in [`Column`] order, each `f32[service][tick]` service-major.
///
/// The file length must equal `data offset + 3 * services * ticks * 4`.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceFile {
    bytes: Vec<u8>,
    tick_seconds: u32,
    services: Vec<String>,
    tick_count: usize,
    data_offset: usize,
}

impl TraceFile {
    /// # Errors
    /// I/O failures and every validation failure of [`TraceFile::from_bytes`].
    pub fn open(path: impl AsRef<Path>) -> Result<Self, TraceError> {
        let _ = path;
        todo!()
    }

    /// # Errors
    /// Bad magic, unsupported version, zero tick, inconsistent offsets, truncated or
    /// oversized data, or names that are not UTF-8.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, TraceError> {
        drop(bytes);
        todo!()
    }

    #[must_use]
    pub fn tick(&self) -> SimDuration {
        let _ = self.tick_seconds;
        todo!()
    }

    #[must_use]
    pub fn service_names(&self) -> &[String] {
        &self.services
    }

    #[must_use]
    pub fn tick_count(&self) -> usize {
        self.tick_count
    }

    /// `None` when `service` or `tick` is out of range.
    #[must_use]
    pub fn value(&self, column: Column, service: usize, tick: usize) -> Option<f32> {
        let _ = (column, service, tick, &self.bytes, self.data_offset);
        todo!()
    }
}

/// Serialises series into the layout documented on [`TraceFile`].
///
/// # Errors
/// [`TraceError::Malformed`] for a zero tick, no services, series of different lengths, or a
/// name longer than `u16::MAX` bytes.
pub fn encode(tick_seconds: u32, services: &[ServiceSeries]) -> Result<Vec<u8>, TraceError> {
    let _ = (tick_seconds, services);
    todo!()
}
