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

impl ServiceSeries {
    fn column(&self, column: Column) -> &[f32] {
        match column {
            Column::RequestRate => &self.request_rate,
            Column::CpuMillis => &self.cpu_millis,
            Column::MemoryBytes => &self.memory_bytes,
        }
    }
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
        Self::from_bytes(std::fs::read(path)?)
    }

    /// # Errors
    /// Bad magic, unsupported version, zero tick, inconsistent offsets, truncated or
    /// oversized data, or names that are not UTF-8.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, TraceError> {
        let header = Header::parse(&bytes)?;
        let data_offset = header.validate_layout(bytes.len())?;
        let services = parse_names(&bytes, &header)?;
        Ok(Self {
            bytes,
            tick_seconds: header.tick_seconds,
            services,
            tick_count: widen(header.tick_count),
            data_offset: widen_u64(data_offset)?,
        })
    }

    #[must_use]
    pub fn tick(&self) -> SimDuration {
        SimDuration::from_secs(u64::from(self.tick_seconds))
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
        if service >= self.services.len() || tick >= self.tick_count {
            return None;
        }
        let row = column as usize * self.services.len() + service;
        let at = self.data_offset + (row * self.tick_count + tick) * F32_LEN;
        read_array(&self.bytes, at).map(f32::from_le_bytes)
    }
}

const F32_LEN: usize = 4;
const COLUMNS: u64 = 3;

fn widen(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

fn widen_u64(value: u64) -> Result<usize, TraceError> {
    usize::try_from(value).map_err(|_| malformed("offset does not fit in memory"))
}

fn malformed(reason: &str) -> TraceError {
    TraceError::Malformed(reason.to_owned())
}

fn read_array<const N: usize>(bytes: &[u8], at: usize) -> Option<[u8; N]> {
    bytes.get(at..at.checked_add(N)?)?.try_into().ok()
}

struct Header {
    tick_seconds: u32,
    service_count: u32,
    tick_count: u32,
    names_offset: u64,
    names_len: u64,
    data_offset: u64,
}

impl Header {
    fn parse(bytes: &[u8]) -> Result<Self, TraceError> {
        let truncated = || TraceError::Truncated {
            expected: HEADER_LEN as u64,
            actual: bytes.len() as u64,
        };
        let u32_at = |at| {
            read_array(bytes, at)
                .map(u32::from_le_bytes)
                .ok_or_else(truncated)
        };
        let u64_at = |at| {
            read_array(bytes, at)
                .map(u64::from_le_bytes)
                .ok_or_else(truncated)
        };
        if bytes.get(..MAGIC.len()).ok_or_else(truncated)? != MAGIC {
            return Err(TraceError::BadMagic);
        }
        let version = u32_at(8)?;
        if version != VERSION {
            return Err(TraceError::UnsupportedVersion(version));
        }
        let header = Self {
            tick_seconds: u32_at(12)?,
            service_count: u32_at(16)?,
            tick_count: u32_at(20)?,
            names_offset: u64_at(24)?,
            names_len: u64_at(32)?,
            data_offset: u64_at(40)?,
        };
        if header.tick_seconds == 0 {
            return Err(malformed("tick seconds must be positive"));
        }
        if header.names_offset != HEADER_LEN as u64 {
            return Err(malformed("names must start right after the header"));
        }
        Ok(header)
    }

    /// Returns the validated data offset.
    fn validate_layout(&self, actual_len: usize) -> Result<u64, TraceError> {
        let names_end = self
            .names_offset
            .checked_add(self.names_len)
            .ok_or_else(|| malformed("names length overflows"))?;
        if self.data_offset != names_end.next_multiple_of(4) {
            return Err(malformed("data must follow the names, aligned to 4 bytes"));
        }
        self.validate_length(actual_len)?;
        Ok(self.data_offset)
    }

    fn validate_length(&self, actual: usize) -> Result<(), TraceError> {
        let expected = COLUMNS
            .checked_mul(u64::from(self.service_count))
            .and_then(|cells| cells.checked_mul(u64::from(self.tick_count)))
            .and_then(|cells| cells.checked_mul(F32_LEN as u64))
            .and_then(|data| data.checked_add(self.data_offset))
            .ok_or_else(|| malformed("data size overflows"))?;
        let actual = actual as u64;
        match actual.cmp(&expected) {
            std::cmp::Ordering::Less => Err(TraceError::Truncated { expected, actual }),
            std::cmp::Ordering::Equal => Ok(()),
            std::cmp::Ordering::Greater => Err(malformed("unexpected bytes after the data")),
        }
    }
}

/// Reads exactly the names block; the header layout must already be validated.
fn parse_names(bytes: &[u8], header: &Header) -> Result<Vec<String>, TraceError> {
    let overrun = || malformed("service names do not fit the names block");
    let block_end = widen_u64(header.names_offset + header.names_len)?;
    let block = bytes.get(HEADER_LEN..block_end).ok_or_else(overrun)?;
    let mut names = Vec::new();
    let mut rest = block;
    for index in 0..widen(header.service_count) {
        let len = read_array(rest, 0)
            .map(u16::from_le_bytes)
            .ok_or_else(overrun)?;
        let raw = rest.get(2..2 + usize::from(len)).ok_or_else(overrun)?;
        let name = std::str::from_utf8(raw).map_err(|_| TraceError::InvalidName(index))?;
        names.push(name.to_owned());
        rest = &rest[2 + raw.len()..];
    }
    if rest.is_empty() {
        Ok(names)
    } else {
        Err(malformed("names block has bytes beyond the service names"))
    }
}

/// Serialises series into the layout documented on [`TraceFile`].
///
/// # Errors
/// [`TraceError::Malformed`] for a zero tick, no services, series of different lengths, or a
/// name longer than `u16::MAX` bytes.
pub fn encode(tick_seconds: u32, services: &[ServiceSeries]) -> Result<Vec<u8>, TraceError> {
    if tick_seconds == 0 {
        return Err(malformed("tick seconds must be positive"));
    }
    let tick_count = uniform_tick_count(services)?;
    let names = encode_names(services)?;
    let data_offset = (HEADER_LEN + names.len()).next_multiple_of(4);
    let mut bytes = Vec::with_capacity(data_offset + services.len() * tick_count * 12);
    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(&tick_seconds.to_le_bytes());
    bytes.extend_from_slice(&count_u32(services.len())?.to_le_bytes());
    bytes.extend_from_slice(&count_u32(tick_count)?.to_le_bytes());
    bytes.extend_from_slice(&(HEADER_LEN as u64).to_le_bytes());
    bytes.extend_from_slice(&(names.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&(data_offset as u64).to_le_bytes());
    bytes.extend_from_slice(&names);
    bytes.resize(data_offset, 0);
    for column in [Column::RequestRate, Column::CpuMillis, Column::MemoryBytes] {
        for value in services.iter().flat_map(|s| s.column(column)) {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    Ok(bytes)
}

fn count_u32(count: usize) -> Result<u32, TraceError> {
    u32::try_from(count).map_err(|_| malformed("too many services or ticks"))
}

fn uniform_tick_count(services: &[ServiceSeries]) -> Result<usize, TraceError> {
    let first = services
        .first()
        .ok_or_else(|| malformed("a trace needs at least one service"))?;
    let expected = first.request_rate.len();
    let uniform = services.iter().all(|s| {
        s.request_rate.len() == expected
            && s.cpu_millis.len() == expected
            && s.memory_bytes.len() == expected
    });
    if uniform {
        Ok(expected)
    } else {
        Err(malformed("all series must have the same length"))
    }
}

fn encode_names(services: &[ServiceSeries]) -> Result<Vec<u8>, TraceError> {
    let mut names = Vec::new();
    for service in services {
        let len = u16::try_from(service.name.len())
            .map_err(|_| malformed("service name is longer than 65535 bytes"))?;
        names.extend_from_slice(&len.to_le_bytes());
        names.extend_from_slice(service.name.as_bytes());
    }
    Ok(names)
}
