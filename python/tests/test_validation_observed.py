"""Tests for ksim.validation.observed: load_observed."""

import math

import numpy as np
import pytest
from ksim.validation.observed import load_observed


class TestLoadObserved:
    """Test load_observed function."""

    def test_load_observed_basic(self, tmp_path):
        """Load observed trajectory from recorder and k6 CSVs."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n5,12,11\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text(
            "metric_name,timestamp,metric_value\n"
            "http_req_duration,1,50\n"
            "http_req_duration,1,52\n"
            "http_req_duration,6,55\n"
            "http_req_duration,6,56\n"
        )

        traj = load_observed(recorder_csv, k6_csv, start_epoch=0, tick_s=5, ticks=2)
        assert traj.tick_s == 5
        assert len(traj.desired) == 2
        assert len(traj.ready) == 2
        assert len(traj.latency_p95_ms) == 2
        assert len(traj.error_rate) == 2
        assert math.isnan(traj.demand_rps[0])
        assert math.isnan(traj.demand_rps[1])

    def test_load_observed_tick_bucketing(self, tmp_path):
        """Bucket measurements into ticks by timestamp."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n3,11,10\n5,12,11\n8,13,12\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text(
            "metric_name,timestamp,metric_value\n"
            "http_req_duration,1,100\n"
            "http_req_duration,2,102\n"
            "http_req_duration,6,110\n"
            "http_req_duration,7,112\n"
        )

        traj = load_observed(recorder_csv, k6_csv, start_epoch=0, tick_s=5, ticks=2)
        assert len(traj.latency_p95_ms) == 2

    def test_load_observed_carry_forward(self, tmp_path):
        """Carry forward last recorder sample when none in tick."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n12,15,14\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text(
            "metric_name,timestamp,metric_value\n"
            "http_req_duration,1,50\n"
            "http_req_duration,6,50\n"
            "http_req_duration,11,50\n"
        )

        traj = load_observed(recorder_csv, k6_csv, start_epoch=0, tick_s=5, ticks=3)
        assert traj.desired[0] == 10
        assert traj.desired[1] == 10
        assert traj.desired[2] == 15
        assert traj.ready[0] == 9
        assert traj.ready[1] == 9
        assert traj.ready[2] == 14

    def test_load_observed_nan_before_first_sample(self, tmp_path):
        """NaN for desired/ready before first recorder sample."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n10,15,14\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text("metric_name,timestamp,metric_value\n")

        traj = load_observed(recorder_csv, k6_csv, start_epoch=0, tick_s=5, ticks=3)
        assert math.isnan(traj.desired[0])
        assert math.isnan(traj.ready[0])
        assert math.isnan(traj.desired[1])
        assert math.isnan(traj.ready[1])
        assert traj.desired[2] == 15
        assert traj.ready[2] == 14

    def test_load_observed_latency_p95(self, tmp_path):
        """Compute p95 latency via numpy.percentile linear."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n5,12,11\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text(
            "metric_name,timestamp,metric_value\n"
            "http_req_duration,1,100\n"
            "http_req_duration,2,110\n"
            "http_req_duration,3,120\n"
            "http_req_duration,4,130\n"
            "http_req_duration,6,200\n"
        )

        traj = load_observed(recorder_csv, k6_csv, start_epoch=0, tick_s=5, ticks=2)
        expected_p95_tick0 = np.percentile([100, 110, 120, 130], 95, method="linear")
        assert traj.latency_p95_ms[0] == pytest.approx(expected_p95_tick0)

    def test_load_observed_latency_nan_without_requests(self, tmp_path):
        """Latency is NaN for ticks without requests."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n5,12,11\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text(
            "metric_name,timestamp,metric_value\n"
            "http_req_duration,1,100\n"
            "http_req_duration,11,200\n"
        )

        traj = load_observed(recorder_csv, k6_csv, start_epoch=0, tick_s=5, ticks=3)
        assert not math.isnan(traj.latency_p95_ms[0])
        assert not math.isnan(traj.latency_p95_ms[2])

    def test_load_observed_error_rate_mean(self, tmp_path):
        """Error rate is mean of http_req_failed values."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n5,12,11\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text(
            "metric_name,timestamp,metric_value\n"
            "http_req_failed,1,0\n"
            "http_req_failed,2,1\n"
            "http_req_failed,3,0\n"
            "http_req_duration,1,50\n"
        )

        traj = load_observed(recorder_csv, k6_csv, start_epoch=0, tick_s=5, ticks=2)
        assert traj.error_rate[0] == pytest.approx(1.0 / 3.0)

    def test_load_observed_error_rate_zero_without_requests(self, tmp_path):
        """Error rate is 0 for ticks without requests."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n5,12,11\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text("metric_name,timestamp,metric_value\n")

        traj = load_observed(recorder_csv, k6_csv, start_epoch=0, tick_s=5, ticks=2)
        assert traj.error_rate[0] == 0.0
        assert traj.error_rate[1] == 0.0

    def test_load_observed_demand_rps_nan(self, tmp_path):
        """demand_rps is always NaN."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n5,12,11\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text("metric_name,timestamp,metric_value\n")

        traj = load_observed(recorder_csv, k6_csv, start_epoch=0, tick_s=5, ticks=2)
        assert all(math.isnan(x) for x in traj.demand_rps)

    def test_load_observed_non_positive_tick_error(self, tmp_path):
        """Raise ValueError for non-positive tick_s."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text("metric_name,timestamp,metric_value\n")

        with pytest.raises(ValueError):
            load_observed(recorder_csv, k6_csv, start_epoch=0, tick_s=0, ticks=2)

    def test_load_observed_non_positive_ticks_error(self, tmp_path):
        """Raise ValueError for non-positive ticks count."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text("metric_name,timestamp,metric_value\n")

        with pytest.raises(ValueError):
            load_observed(recorder_csv, k6_csv, start_epoch=0, tick_s=5, ticks=0)

    def test_load_observed_missing_recorder_columns(self, tmp_path):
        """Raise ValueError when recorder CSV missing required columns."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("time_s,value\n0,10\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text("metric_name,timestamp,metric_value\n")

        with pytest.raises(ValueError):
            load_observed(recorder_csv, k6_csv, start_epoch=0, tick_s=5, ticks=2)

    def test_load_observed_missing_k6_columns(self, tmp_path):
        """Raise ValueError when k6 CSV missing required columns."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text("value\n50\n")

        with pytest.raises(ValueError):
            load_observed(recorder_csv, k6_csv, start_epoch=0, tick_s=5, ticks=2)

    def test_load_observed_ignores_other_k6_metrics(self, tmp_path):
        """Ignore k6 CSV rows for metrics other than http_req_duration and http_req_failed."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n5,12,11\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text(
            "metric_name,timestamp,metric_value\nhttp_reqs,1,1\nhttp_req_duration,1,100\nvus,1,50\n"
        )

        traj = load_observed(recorder_csv, k6_csv, start_epoch=0, tick_s=5, ticks=2)
        assert not math.isnan(traj.latency_p95_ms[0])
