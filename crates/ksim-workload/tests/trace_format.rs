use ksim_core::time::SimDuration;
use ksim_workload::trace::{Column, ServiceSeries, TraceError, TraceFile, encode};
use proptest::prelude::*;
use std::fs;
use std::io::Write;

fn golden_bytes() -> Vec<u8> {
    // Layout: header(48) + names(5) + padding(3) + data(24) = 80 bytes total
    let mut bytes = Vec::new();

    // Header: MAGIC(8) + version(4) + tick_seconds(4) + services(4) + ticks(4)
    //         + names_offset(8) + names_len(8) + data_offset(8)
    bytes.extend_from_slice(b"KSIMTRC1"); // 8 bytes
    bytes.extend_from_slice(&(1_u32).to_le_bytes()); // version: 1
    bytes.extend_from_slice(&(15_u32).to_le_bytes()); // tick_seconds: 15
    bytes.extend_from_slice(&(1_u32).to_le_bytes()); // service_count: 1
    bytes.extend_from_slice(&(2_u32).to_le_bytes()); // tick_count: 2
    bytes.extend_from_slice(&(48_u64).to_le_bytes()); // names_offset: 48
    bytes.extend_from_slice(&(5_u64).to_le_bytes()); // names_len: 5
    bytes.extend_from_slice(&(56_u64).to_le_bytes()); // data_offset: 56

    // Names section: u16 length + "web"
    bytes.extend_from_slice(&(3_u16).to_le_bytes()); // "web" length: 3
    bytes.extend_from_slice(b"web"); // 3 bytes

    // Padding: 3 zero bytes (5 bytes used -> 56 bytes total)
    bytes.extend_from_slice(&[0, 0, 0]);

    // Data section: RequestRate, CpuMillis, MemoryBytes columns (service-major)
    // Service 0, all ticks for RequestRate
    bytes.extend_from_slice(&(1.0_f32).to_le_bytes());
    bytes.extend_from_slice(&(2.0_f32).to_le_bytes());
    // Service 0, all ticks for CpuMillis
    bytes.extend_from_slice(&(100.0_f32).to_le_bytes());
    bytes.extend_from_slice(&(200.0_f32).to_le_bytes());
    // Service 0, all ticks for MemoryBytes
    bytes.extend_from_slice(&(1024.0_f32).to_le_bytes());
    bytes.extend_from_slice(&(2048.0_f32).to_le_bytes());

    assert_eq!(bytes.len(), 80, "Golden bytes must be exactly 80 bytes");
    bytes
}

#[test]
fn encode_golden_case() {
    let service = ServiceSeries {
        name: "web".to_string(),
        request_rate: vec![1.0, 2.0],
        cpu_millis: vec![100.0, 200.0],
        memory_bytes: vec![1024.0, 2048.0],
    };

    let result = encode(15, &[service]);
    assert!(result.is_ok(), "Encode should succeed for valid input");

    let bytes = result.unwrap();
    let expected = golden_bytes();
    assert_eq!(bytes, expected, "Encoded bytes must match golden case");
}

#[test]
fn from_bytes_golden_case() {
    let bytes = golden_bytes();
    let trace = TraceFile::from_bytes(bytes);
    assert!(trace.is_ok(), "from_bytes should succeed for golden bytes");

    let trace = trace.unwrap();
    assert_eq!(
        trace.tick(),
        SimDuration::from_secs(15),
        "tick() must return 15s"
    );
    assert_eq!(
        trace.service_names(),
        &["web".to_string()],
        "service_names() must contain ['web']"
    );
    assert_eq!(trace.tick_count(), 2, "tick_count() must return 2");
}

#[test]
fn value_cpu_millis_in_range() {
    let bytes = golden_bytes();
    let trace = TraceFile::from_bytes(bytes).expect("from_bytes should succeed");

    let value = trace.value(Column::CpuMillis, 0, 1);
    assert_eq!(
        value,
        Some(200.0),
        "value(CpuMillis, 0, 1) must return 200.0"
    );
}

#[test]
fn value_out_of_range_service() {
    let bytes = golden_bytes();
    let trace = TraceFile::from_bytes(bytes).expect("from_bytes should succeed");

    let value = trace.value(Column::CpuMillis, 999, 0);
    assert_eq!(value, None, "value out of range service must return None");
}

#[test]
fn value_out_of_range_tick() {
    let bytes = golden_bytes();
    let trace = TraceFile::from_bytes(bytes).expect("from_bytes should succeed");

    let value = trace.value(Column::CpuMillis, 0, 999);
    assert_eq!(value, None, "value out of range tick must return None");
}

#[test]
fn value_all_columns() {
    let bytes = golden_bytes();
    let trace = TraceFile::from_bytes(bytes).expect("from_bytes should succeed");

    assert_eq!(
        trace.value(Column::RequestRate, 0, 0),
        Some(1.0),
        "RequestRate[0,0] must be 1.0"
    );
    assert_eq!(
        trace.value(Column::RequestRate, 0, 1),
        Some(2.0),
        "RequestRate[0,1] must be 2.0"
    );
    assert_eq!(
        trace.value(Column::CpuMillis, 0, 0),
        Some(100.0),
        "CpuMillis[0,0] must be 100.0"
    );
    assert_eq!(
        trace.value(Column::CpuMillis, 0, 1),
        Some(200.0),
        "CpuMillis[0,1] must be 200.0"
    );
    assert_eq!(
        trace.value(Column::MemoryBytes, 0, 0),
        Some(1024.0),
        "MemoryBytes[0,0] must be 1024.0"
    );
    assert_eq!(
        trace.value(Column::MemoryBytes, 0, 1),
        Some(2048.0),
        "MemoryBytes[0,1] must be 2048.0"
    );
}

#[test]
fn roundtrip_two_services() {
    // Test with two services "a" and "bcd"
    // names_len = 2 (length of "a") + 1 (u16 len) + 2 (length of "bcd") + 1 (u16 len) = 6
    // But names_len should just be the byte count of the names: 1 + 3 = 4
    // Actually, let me recalculate:
    // Service 0: u16(1) + "a"(1) = 2 bytes
    // Service 1: u16(3) + "bcd"(3) = 6 bytes
    // Total names: 8 bytes
    // names_offset: 48
    // Next position: 48 + 8 = 56
    // Padding: already at 56 (divisible by 4)
    // data_offset: 56

    let service_a = ServiceSeries {
        name: "a".to_string(),
        request_rate: vec![10.0, 20.0],
        cpu_millis: vec![50.0, 100.0],
        memory_bytes: vec![512.0, 1024.0],
    };

    let service_bcd = ServiceSeries {
        name: "bcd".to_string(),
        request_rate: vec![30.0, 40.0],
        cpu_millis: vec![150.0, 200.0],
        memory_bytes: vec![2048.0, 4096.0],
    };

    let encoded = encode(20, &[service_a.clone(), service_bcd.clone()]);
    assert!(encoded.is_ok(), "encode should succeed for two services");

    let bytes = encoded.unwrap();
    let decoded = TraceFile::from_bytes(bytes);
    assert!(
        decoded.is_ok(),
        "from_bytes should succeed for two-service trace"
    );

    let trace = decoded.unwrap();
    assert_eq!(trace.service_names(), &["a".to_string(), "bcd".to_string()]);
    assert_eq!(trace.tick_count(), 2);
    assert_eq!(trace.tick(), SimDuration::from_secs(20));

    // Verify values for service 0
    assert_eq!(trace.value(Column::RequestRate, 0, 0), Some(10.0));
    assert_eq!(trace.value(Column::RequestRate, 0, 1), Some(20.0));
    assert_eq!(trace.value(Column::CpuMillis, 0, 0), Some(50.0));
    assert_eq!(trace.value(Column::CpuMillis, 0, 1), Some(100.0));
    assert_eq!(trace.value(Column::MemoryBytes, 0, 0), Some(512.0));
    assert_eq!(trace.value(Column::MemoryBytes, 0, 1), Some(1024.0));

    // Verify values for service 1
    assert_eq!(trace.value(Column::RequestRate, 1, 0), Some(30.0));
    assert_eq!(trace.value(Column::RequestRate, 1, 1), Some(40.0));
    assert_eq!(trace.value(Column::CpuMillis, 1, 0), Some(150.0));
    assert_eq!(trace.value(Column::CpuMillis, 1, 1), Some(200.0));
    assert_eq!(trace.value(Column::MemoryBytes, 1, 0), Some(2048.0));
    assert_eq!(trace.value(Column::MemoryBytes, 1, 1), Some(4096.0));
}

#[test]
fn open_file_from_temp_dir() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("ksim_trace_test.ksimtrace");

    // Encode and write to file
    let service = ServiceSeries {
        name: "test".to_string(),
        request_rate: vec![1.0, 2.0],
        cpu_millis: vec![10.0, 20.0],
        memory_bytes: vec![100.0, 200.0],
    };

    let encoded = encode(5, &[service]).expect("encode should succeed");
    let mut file = fs::File::create(&temp_file).expect("create temp file should succeed");
    file.write_all(&encoded)
        .expect("write to temp file should succeed");

    // Open the file
    let trace = TraceFile::open(&temp_file);
    assert!(trace.is_ok(), "open should succeed for valid file");

    let trace = trace.unwrap();
    assert_eq!(trace.tick(), SimDuration::from_secs(5));
    assert_eq!(trace.service_names(), &["test".to_string()]);

    // Cleanup
    let _ = fs::remove_file(&temp_file);
}

#[test]
fn open_missing_file_returns_io_error() {
    let result = TraceFile::open("/nonexistent/path/to/file.ksimtrace");
    assert!(
        matches!(result, Err(TraceError::Io(_))),
        "open should return Io error for missing file"
    );
}

#[test]
fn bad_magic_error() {
    let mut bytes = golden_bytes();
    bytes[0] = b'X'; // Corrupt magic

    let result = TraceFile::from_bytes(bytes);
    assert!(
        matches!(result, Err(TraceError::BadMagic)),
        "bad magic must return BadMagic error"
    );
}

#[test]
fn unsupported_version_error() {
    let mut bytes = golden_bytes();
    // Overwrite version (offset 8-11) with 2
    bytes[8..12].copy_from_slice(&(2_u32).to_le_bytes());

    let result = TraceFile::from_bytes(bytes);
    assert!(
        matches!(result, Err(TraceError::UnsupportedVersion(2))),
        "version 2 must return UnsupportedVersion error"
    );
}

#[test]
fn tick_zero_malformed_error() {
    let mut bytes = golden_bytes();
    // Overwrite tick_seconds (offset 12-15) with 0
    bytes[12..16].copy_from_slice(&(0_u32).to_le_bytes());

    let result = TraceFile::from_bytes(bytes);
    assert!(
        matches!(result, Err(TraceError::Malformed(_))),
        "tick 0 must return Malformed error"
    );
}

#[test]
fn truncated_data_error() {
    let mut bytes = golden_bytes();
    bytes.pop(); // Remove last byte

    let result = TraceFile::from_bytes(bytes);
    assert!(
        matches!(result, Err(TraceError::Truncated { .. })),
        "truncated data must return Truncated error"
    );
}

#[test]
fn extra_trailing_byte_error() {
    let mut bytes = golden_bytes();
    bytes.push(0); // Add extra byte

    let result = TraceFile::from_bytes(bytes);
    // Must be an error (either Truncated or Malformed)
    assert!(result.is_err(), "extra trailing byte must return an error");
}

#[test]
fn invalid_utf8_name_error() {
    let mut bytes = Vec::new();

    // Header
    bytes.extend_from_slice(b"KSIMTRC1");
    bytes.extend_from_slice(&(1_u32).to_le_bytes()); // version
    bytes.extend_from_slice(&(1_u32).to_le_bytes()); // tick_seconds
    bytes.extend_from_slice(&(1_u32).to_le_bytes()); // service_count
    bytes.extend_from_slice(&(1_u32).to_le_bytes()); // tick_count
    bytes.extend_from_slice(&(48_u64).to_le_bytes()); // names_offset
    bytes.extend_from_slice(&(4_u64).to_le_bytes()); // names_len: u16 + 2 bytes
    bytes.extend_from_slice(&(52_u64).to_le_bytes()); // data_offset: 48 + 4, already aligned

    bytes.extend_from_slice(&(2_u16).to_le_bytes());
    bytes.extend_from_slice(&[0xFF, 0xFE]);

    // 3 columns * 1 service * 1 tick, total length 52 + 12 = 64
    bytes.extend_from_slice(&(1.0_f32).to_le_bytes());
    bytes.extend_from_slice(&(2.0_f32).to_le_bytes());
    bytes.extend_from_slice(&(3.0_f32).to_le_bytes());

    let result = TraceFile::from_bytes(bytes);
    assert!(
        matches!(result, Err(TraceError::InvalidName(0))),
        "invalid UTF-8 name must return InvalidName error"
    );
}

#[test]
fn encode_no_services_malformed() {
    let result = encode(1, &[]);
    assert!(
        matches!(result, Err(TraceError::Malformed(_))),
        "encode with no services must return Malformed error"
    );
}

#[test]
fn encode_zero_tick_malformed() {
    let service = ServiceSeries {
        name: "test".to_string(),
        request_rate: vec![1.0],
        cpu_millis: vec![10.0],
        memory_bytes: vec![100.0],
    };

    let result = encode(0, &[service]);
    assert!(
        matches!(result, Err(TraceError::Malformed(_))),
        "encode with zero tick must return Malformed error"
    );
}

#[test]
fn encode_unequal_lengths_malformed() {
    let service = ServiceSeries {
        name: "test".to_string(),
        request_rate: vec![1.0, 2.0],
        cpu_millis: vec![10.0], // Different length
        memory_bytes: vec![100.0, 200.0],
    };

    let result = encode(1, &[service]);
    assert!(
        matches!(result, Err(TraceError::Malformed(_))),
        "encode with unequal lengths must return Malformed error"
    );
}

proptest! {
    #[test]
    fn proptest_roundtrip_random_traces(
        num_services in 1_usize..4,
        num_ticks in 1_usize..50,
        tick_seconds in 1_u64..3600,
    ) {
        // Generate random f32 values (use fixed finite values for simplicity)
        let fixed_values = [0.1_f32, 1.0_f32, 10.0_f32, 100.0_f32];
        let services: Vec<ServiceSeries> = (0..num_services)
            .map(|i| ServiceSeries {
                name: format!("service_{i}"),
                request_rate: (0..num_ticks)
                    .map(|j| fixed_values[j % fixed_values.len()])
                    .collect(),
                cpu_millis: (0..num_ticks)
                    .map(|j| fixed_values[(j + 1) % fixed_values.len()] * 10.0)
                    .collect(),
                memory_bytes: (0..num_ticks)
                    .map(|j| fixed_values[(j + 2) % fixed_values.len()] * 1024.0)
                    .collect(),
            })
            .collect();

        // Encode
        let encoded = encode(u32::try_from(tick_seconds).unwrap_or(u32::MAX), &services)?;

        // Decode
        let trace = TraceFile::from_bytes(encoded)?;

        // Verify metadata
        prop_assert_eq!(trace.tick(), SimDuration::from_secs(tick_seconds));
        prop_assert_eq!(trace.tick_count(), num_ticks);
        prop_assert_eq!(trace.service_names().len(), num_services);

        // Verify values are bit-equal
        for (svc_idx, svc) in services.iter().enumerate() {
            for (tick_idx, _) in svc.request_rate.iter().enumerate() {
                let encoded_rr = trace.value(Column::RequestRate, svc_idx, tick_idx);
                prop_assert_eq!(encoded_rr, Some(svc.request_rate[tick_idx]));

                let encoded_cpu = trace.value(Column::CpuMillis, svc_idx, tick_idx);
                prop_assert_eq!(encoded_cpu, Some(svc.cpu_millis[tick_idx]));

                let encoded_mem = trace.value(Column::MemoryBytes, svc_idx, tick_idx);
                prop_assert_eq!(encoded_mem, Some(svc.memory_bytes[tick_idx]));
            }
        }
    }
}
