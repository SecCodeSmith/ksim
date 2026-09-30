"""Tests for ksim.validation.__main__: main CLI entry point."""

from ksim.validation.__main__ import main


class TestMainK6Script:
    """Test k6-script subcommand."""

    def test_k6_script_succeeds(self, tmp_path):
        """k6-script writes script and returns 0."""
        sim_csv = tmp_path / "sim.csv"
        sim_csv.write_text(
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
            "5,api,demand_rps,rate,120.0\n"
        )
        out_file = tmp_path / "script.js"

        result = main(
            [
                "k6-script",
                "--trajectory",
                str(sim_csv),
                "--service",
                "api",
                "--url",
                "https://api.example.com",
                "--out",
                str(out_file),
            ]
        )
        assert result == 0
        assert out_file.exists()
        assert "ramping-arrival-rate" in out_file.read_text()

    def test_k6_script_with_rate_scale(self, tmp_path):
        """k6-script accepts rate-scale parameter."""
        sim_csv = tmp_path / "sim.csv"
        sim_csv.write_text(
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
            "5,api,demand_rps,rate,120.0\n"
        )
        out_file = tmp_path / "script.js"

        result = main(
            [
                "k6-script",
                "--trajectory",
                str(sim_csv),
                "--service",
                "api",
                "--url",
                "https://api.example.com",
                "--out",
                str(out_file),
                "--rate-scale",
                "0.5",
            ]
        )
        assert result == 0
        assert out_file.exists()

    def test_k6_script_with_max_vus(self, tmp_path):
        """k6-script accepts max-vus parameter."""
        sim_csv = tmp_path / "sim.csv"
        sim_csv.write_text(
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
            "5,api,demand_rps,rate,120.0\n"
        )
        out_file = tmp_path / "script.js"

        result = main(
            [
                "k6-script",
                "--trajectory",
                str(sim_csv),
                "--service",
                "api",
                "--url",
                "https://api.example.com",
                "--out",
                str(out_file),
                "--max-vus",
                "300",
            ]
        )
        assert result == 0
        assert out_file.exists()

    def test_k6_script_missing_file_error(self, tmp_path):
        """k6-script returns 2 for missing trajectory file."""
        out_file = tmp_path / "script.js"
        result = main(
            [
                "k6-script",
                "--trajectory",
                str(tmp_path / "nonexistent.csv"),
                "--service",
                "api",
                "--url",
                "https://api.example.com",
                "--out",
                str(out_file),
            ]
        )
        assert result == 2

    def test_k6_script_error_stderr(self, tmp_path, capsys):
        """k6-script error includes 'error:' on stderr."""
        out_file = tmp_path / "script.js"
        result = main(
            [
                "k6-script",
                "--trajectory",
                str(tmp_path / "nonexistent.csv"),
                "--service",
                "api",
                "--url",
                "https://api.example.com",
                "--out",
                str(out_file),
            ]
        )
        assert result == 2
        _, stderr = capsys.readouterr()
        assert "error:" in stderr


class TestMainCalibrate:
    """Test calibrate subcommand."""

    def test_calibrate_succeeds(self, tmp_path):
        """calibrate writes scenario and returns 0."""
        in_file = tmp_path / "scenario.toml"
        in_file.write_text("cpu_millis_per_request = 50.0\nother = 123\n")
        out_file = tmp_path / "scenario_out.toml"

        result = main(
            [
                "calibrate",
                "--scenario",
                str(in_file),
                "--cpu-millicores",
                "500.0",
                "--rps",
                "100.0",
                "--out",
                str(out_file),
            ]
        )
        assert result == 0
        assert out_file.exists()

    def test_calibrate_prints_value(self, tmp_path, capsys):
        """calibrate prints the calculated value."""
        in_file = tmp_path / "scenario.toml"
        in_file.write_text("cpu_millis_per_request = 50.0\n")
        out_file = tmp_path / "scenario_out.toml"

        main(
            [
                "calibrate",
                "--scenario",
                str(in_file),
                "--cpu-millicores",
                "500.0",
                "--rps",
                "100.0",
                "--out",
                str(out_file),
            ]
        )
        stdout, _ = capsys.readouterr()
        assert "5.0" in stdout

    def test_calibrate_missing_file_error(self, tmp_path):
        """calibrate returns 2 for missing scenario file."""
        out_file = tmp_path / "scenario_out.toml"
        result = main(
            [
                "calibrate",
                "--scenario",
                str(tmp_path / "nonexistent.toml"),
                "--cpu-millicores",
                "500.0",
                "--rps",
                "100.0",
                "--out",
                str(out_file),
            ]
        )
        assert result == 2

    def test_calibrate_error_stderr(self, tmp_path, capsys):
        """calibrate error includes 'error:' on stderr."""
        out_file = tmp_path / "scenario_out.toml"
        result = main(
            [
                "calibrate",
                "--scenario",
                str(tmp_path / "nonexistent.toml"),
                "--cpu-millicores",
                "500.0",
                "--rps",
                "100.0",
                "--out",
                str(out_file),
            ]
        )
        assert result == 2
        _, stderr = capsys.readouterr()
        assert "error:" in stderr


class TestMainCompare:
    """Test compare subcommand."""

    def test_compare_succeeds(self, tmp_path):
        """compare writes report and returns 0."""
        sim_csv = tmp_path / "sim.csv"
        sim_csv.write_text(
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
            "5,api,demand_rps,rate,120.0\n"
        )

        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n5,12,11\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text(
            "metric_name,timestamp,metric_value\nhttp_req_duration,1,50\nhttp_req_duration,6,52\n"
        )

        report_file = tmp_path / "report.md"

        result = main(
            [
                "compare",
                "--simulated",
                str(sim_csv),
                "--service",
                "api",
                "--recorder",
                str(recorder_csv),
                "--k6",
                str(k6_csv),
                "--start-epoch",
                "0",
                "--out",
                str(report_file),
            ]
        )
        assert result == 0
        assert report_file.exists()

    def test_compare_prints_report(self, tmp_path, capsys):
        """compare prints the markdown report."""
        sim_csv = tmp_path / "sim.csv"
        sim_csv.write_text(
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
            "5,api,demand_rps,rate,120.0\n"
        )

        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n5,12,11\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text(
            "metric_name,timestamp,metric_value\nhttp_req_duration,1,50\nhttp_req_duration,6,52\n"
        )

        report_file = tmp_path / "report.md"

        main(
            [
                "compare",
                "--simulated",
                str(sim_csv),
                "--service",
                "api",
                "--recorder",
                str(recorder_csv),
                "--k6",
                str(k6_csv),
                "--start-epoch",
                "0",
                "--out",
                str(report_file),
            ]
        )
        stdout, _ = capsys.readouterr()
        assert "Simulator vs. kind cluster" in stdout

    def test_compare_missing_simulated_error(self, tmp_path):
        """compare returns 2 for missing simulated file."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text("metric_name,timestamp,metric_value\n")

        report_file = tmp_path / "report.md"

        result = main(
            [
                "compare",
                "--simulated",
                str(tmp_path / "nonexistent.csv"),
                "--service",
                "api",
                "--recorder",
                str(recorder_csv),
                "--k6",
                str(k6_csv),
                "--start-epoch",
                "0",
                "--out",
                str(report_file),
            ]
        )
        assert result == 2

    def test_compare_error_stderr(self, tmp_path, capsys):
        """compare error includes 'error:' on stderr."""
        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n")

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text("metric_name,timestamp,metric_value\n")

        report_file = tmp_path / "report.md"

        result = main(
            [
                "compare",
                "--simulated",
                str(tmp_path / "nonexistent.csv"),
                "--service",
                "api",
                "--recorder",
                str(recorder_csv),
                "--k6",
                str(k6_csv),
                "--start-epoch",
                "0",
                "--out",
                str(report_file),
            ]
        )
        assert result == 2
        _, stderr = capsys.readouterr()
        assert "error:" in stderr

    def test_compare_missing_recorder_error(self, tmp_path):
        """compare returns 2 for missing recorder file."""
        sim_csv = tmp_path / "sim.csv"
        sim_csv.write_text(
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
            "5,api,demand_rps,rate,120.0\n"
        )

        k6_csv = tmp_path / "k6.csv"
        k6_csv.write_text("metric_name,timestamp,metric_value\n")

        report_file = tmp_path / "report.md"

        result = main(
            [
                "compare",
                "--simulated",
                str(sim_csv),
                "--service",
                "api",
                "--recorder",
                str(tmp_path / "nonexistent.csv"),
                "--k6",
                str(k6_csv),
                "--start-epoch",
                "0",
                "--out",
                str(report_file),
            ]
        )
        assert result == 2

    def test_compare_missing_k6_error(self, tmp_path):
        """compare returns 2 for missing k6 file."""
        sim_csv = tmp_path / "sim.csv"
        sim_csv.write_text(
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
            "5,api,demand_rps,rate,120.0\n"
        )

        recorder_csv = tmp_path / "recorder.csv"
        recorder_csv.write_text("epoch_s,desired,ready\n0,10,9\n5,12,11\n")

        report_file = tmp_path / "report.md"

        result = main(
            [
                "compare",
                "--simulated",
                str(sim_csv),
                "--service",
                "api",
                "--recorder",
                str(recorder_csv),
                "--k6",
                str(tmp_path / "nonexistent.csv"),
                "--start-epoch",
                "0",
                "--out",
                str(report_file),
            ]
        )
        assert result == 2
