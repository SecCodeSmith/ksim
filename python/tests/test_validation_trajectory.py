"""Tests for ksim.validation.trajectory: load_simulated."""

import pytest
from ksim.validation.trajectory import load_simulated


class TestLoadSimulated:
    """Test load_simulated function."""

    def test_load_simulated_basic(self, tmp_path):
        """Load simulated trajectory with basic CSV data."""
        csv_path = tmp_path / "sim.csv"
        csv_path.write_text(
            "time_s,service,demand_rps,desired,ready,cpu_usage_millis,latency_p95_ms,error_rate\n"
            "0,api,100.0,10,9,0,50.0,0.01\n"
            "5,api,110.0,12,11,0,52.0,0.02\n"
        )
        traj = load_simulated(csv_path, "api")
        assert traj.tick_s == 5
        assert len(traj.demand_rps) == 2
        assert len(traj.desired) == 2
        assert len(traj.ready) == 2
        assert len(traj.latency_p95_ms) == 2
        assert len(traj.error_rate) == 2

    def test_load_simulated_tick_from_spacing(self, tmp_path):
        """Compute tick_s from time_s spacing."""
        csv_path = tmp_path / "sim.csv"
        csv_path.write_text(
            "time_s,service,demand_rps,desired,ready,cpu_usage_millis,latency_p95_ms,error_rate\n"
            "0,web,50.0,5,5,0,100.0,0.0\n"
            "3,web,55.0,6,6,0,105.0,0.0\n"
            "6,web,60.0,7,7,0,110.0,0.0\n"
        )
        traj = load_simulated(csv_path, "web")
        assert traj.tick_s == 3

    def test_load_simulated_filters_service(self, tmp_path):
        """Filter rows by service name."""
        csv_path = tmp_path / "sim.csv"
        csv_path.write_text(
            "time_s,service,demand_rps,desired,ready,cpu_usage_millis,latency_p95_ms,error_rate\n"
            "0,api,100.0,10,9,0,50.0,0.01\n"
            "0,web,200.0,20,19,0,60.0,0.02\n"
            "5,api,120.0,12,11,0,52.0,0.03\n"
            "5,web,220.0,22,21,0,62.0,0.04\n"
        )
        traj = load_simulated(csv_path, "api")
        assert len(traj.desired) == 2
        assert traj.desired[0] == 10
        assert traj.desired[1] == 12
        assert traj.demand_rps[0] == 100.0
        assert traj.demand_rps[1] == 120.0

    def test_load_simulated_missing_service(self, tmp_path):
        """Raise ValueError when service not found."""
        csv_path = tmp_path / "sim.csv"
        csv_path.write_text(
            "time_s,service,demand_rps,desired,ready,cpu_usage_millis,latency_p95_ms,error_rate\n"
            "0,api,100.0,10,9,0,50.0,0.01\n"
            "5,api,110.0,12,11,0,52.0,0.02\n"
        )
        with pytest.raises(ValueError):
            load_simulated(csv_path, "nonexistent")

    def test_load_simulated_single_row(self, tmp_path):
        """Raise ValueError when service has only one row."""
        csv_path = tmp_path / "sim.csv"
        csv_path.write_text(
            "time_s,service,demand_rps,desired,ready,cpu_usage_millis,latency_p95_ms,error_rate\n"
            "0,api,100.0,10,9,0,50.0,0.01\n"
        )
        with pytest.raises(ValueError):
            load_simulated(csv_path, "api")

    def test_load_simulated_file_not_found(self, tmp_path):
        """Raise ValueError when file does not exist."""
        csv_path = tmp_path / "nonexistent.csv"
        with pytest.raises((ValueError, FileNotFoundError)):
            load_simulated(csv_path, "api")
