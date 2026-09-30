"""Tests for ksim.validation.trajectory: load_simulated."""

import pytest
from ksim.validation.trajectory import load_simulated


class TestLoadSimulated:
    """Test load_simulated function."""

    def test_load_simulated_basic(self, tmp_path):
        """Load simulated trajectory with basic CSV data."""
        csv_path = tmp_path / "sim.csv"
        csv_path.write_text(
            "time_s,service,replica_type,metric,value\n"
            "0,api,desired,replica_count,10\n"
            "0,api,ready,replica_count,9\n"
            "0,api,latency_p95_ms,latency,50.0\n"
            "0,api,error_rate,rate,0.01\n"
            "0,api,demand_rps,rate,100.0\n"
            "5,api,desired,replica_count,12\n"
            "5,api,ready,replica_count,11\n"
            "5,api,latency_p95_ms,latency,52.0\n"
            "5,api,error_rate,rate,0.02\n"
            "5,api,demand_rps,rate,110.0\n"
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
            "time_s,service,replica_type,metric,value\n"
            "0,web,desired,replica_count,5\n"
            "0,web,ready,replica_count,5\n"
            "0,web,latency_p95_ms,latency,100.0\n"
            "0,web,error_rate,rate,0.0\n"
            "0,web,demand_rps,rate,50.0\n"
            "3,web,desired,replica_count,6\n"
            "3,web,ready,replica_count,6\n"
            "3,web,latency_p95_ms,latency,105.0\n"
            "3,web,error_rate,rate,0.0\n"
            "3,web,demand_rps,rate,55.0\n"
            "6,web,desired,replica_count,7\n"
            "6,web,ready,replica_count,7\n"
            "6,web,latency_p95_ms,latency,110.0\n"
            "6,web,error_rate,rate,0.0\n"
            "6,web,demand_rps,rate,60.0\n"
        )
        traj = load_simulated(csv_path, "web")
        assert traj.tick_s == 3

    def test_load_simulated_filters_service(self, tmp_path):
        """Filter rows by service name."""
        csv_path = tmp_path / "sim.csv"
        csv_path.write_text(
            "time_s,service,replica_type,metric,value\n"
            "0,api,desired,replica_count,10\n"
            "0,api,ready,replica_count,9\n"
            "0,api,latency_p95_ms,latency,50.0\n"
            "0,api,error_rate,rate,0.01\n"
            "0,api,demand_rps,rate,100.0\n"
            "0,web,desired,replica_count,20\n"
            "0,web,ready,replica_count,19\n"
            "0,web,latency_p95_ms,latency,60.0\n"
            "0,web,error_rate,rate,0.02\n"
            "0,web,demand_rps,rate,200.0\n"
            "5,api,desired,replica_count,12\n"
            "5,api,ready,replica_count,11\n"
            "5,api,latency_p95_ms,latency,52.0\n"
            "5,api,error_rate,rate,0.03\n"
            "5,api,demand_rps,rate,120.0\n"
            "5,web,desired,replica_count,22\n"
            "5,web,ready,replica_count,21\n"
            "5,web,latency_p95_ms,latency,62.0\n"
            "5,web,error_rate,rate,0.04\n"
            "5,web,demand_rps,rate,220.0\n"
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
            "time_s,service,replica_type,metric,value\n"
            "0,api,desired,replica_count,10\n"
            "0,api,ready,replica_count,9\n"
            "0,api,latency_p95_ms,latency,50.0\n"
            "0,api,error_rate,rate,0.01\n"
            "0,api,demand_rps,rate,100.0\n"
        )
        with pytest.raises(ValueError):
            load_simulated(csv_path, "nonexistent")

    def test_load_simulated_single_row(self, tmp_path):
        """Raise ValueError when service has only one row."""
        csv_path = tmp_path / "sim.csv"
        csv_path.write_text(
            "time_s,service,replica_type,metric,value\n"
            "0,api,desired,replica_count,10\n"
            "0,api,ready,replica_count,9\n"
            "0,api,latency_p95_ms,latency,50.0\n"
            "0,api,error_rate,rate,0.01\n"
            "0,api,demand_rps,rate,100.0\n"
        )
        with pytest.raises(ValueError):
            load_simulated(csv_path, "api")

    def test_load_simulated_file_not_found(self, tmp_path):
        """Raise ValueError when file does not exist."""
        csv_path = tmp_path / "nonexistent.csv"
        with pytest.raises((ValueError, FileNotFoundError)):
            load_simulated(csv_path, "api")
