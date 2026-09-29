use std::io::Write;

use ksim_metrics::report::EpisodeReport;

/// One JSON object per line.
///
/// # Errors
/// On write or serialisation failure.
pub fn write_json_lines(reports: &[EpisodeReport], writer: impl Write) -> std::io::Result<()> {
    let _ = (reports, writer);
    todo!()
}

/// Header `scenario,policy,seed,<metric names in BTreeMap order>`, one row per report; every
/// report must have the same metric names.
///
/// # Errors
/// On write failure or when reports disagree on metric names.
pub fn write_csv(reports: &[EpisodeReport], writer: impl Write) -> std::io::Result<()> {
    let _ = (reports, writer);
    todo!()
}
