use std::io::{self, Write};

use ksim_metrics::report::EpisodeReport;

/// One JSON object per line.
///
/// # Errors
/// On write or serialisation failure.
pub fn write_json_lines(reports: &[EpisodeReport], mut writer: impl Write) -> io::Result<()> {
    for report in reports {
        serde_json::to_writer(&mut writer, report)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}

/// Header `scenario,policy,seed,<metric names in BTreeMap order>`, one row per report; every
/// report must have the same metric names.
///
/// # Errors
/// On write failure or when reports disagree on metric names.
pub fn write_csv(reports: &[EpisodeReport], writer: impl Write) -> io::Result<()> {
    let Some(first) = reports.first() else {
        return Ok(());
    };
    let names: Vec<&String> = first.metrics.keys().collect();
    if reports
        .iter()
        .any(|report| !report.metrics.keys().eq(names.iter().copied()))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "reports disagree on metric names",
        ));
    }
    let mut csv_writer = csv::Writer::from_writer(writer);
    csv_writer.write_record(
        ["scenario", "policy", "seed"]
            .into_iter()
            .map(str::to_owned)
            .chain(names.into_iter().cloned()),
    )?;
    for report in reports {
        csv_writer.write_record(
            [
                report.scenario.clone(),
                report.policy.clone(),
                report.seed.to_string(),
            ]
            .into_iter()
            .chain(report.metrics.values().map(f64::to_string)),
        )?;
    }
    csv_writer.flush()
}
