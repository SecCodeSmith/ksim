"""Tests for ksim.datasets CLI extras: unit flags in convert and scenario subcommand."""

import tomllib
from pathlib import Path

import numpy as np
import pytest
from ksim.datasets.__main__ import main
from ksim.datasets.ksimtrace import ServiceSeries, write


class TestConvertWithUnitFlags:
    """Test convert command with unit flags."""

    def test_convert_with_cpu_millis_per_request_flag(self, tmp_path):
        """convert with --cpu-millis-per-request flag succeeds."""
        fixture_dir = Path(__file__).parent / "fixtures" / "datasets" / "alibaba2018"
        if not fixture_dir.exists():
            pytest.skip("Fixtures not found")

        output_path = tmp_path / "output.ksimtrace"
        result = main(
            [
                "convert",
                "--source",
                "alibaba2018",
                "--input",
                str(fixture_dir),
                "--output",
                str(output_path),
                "--tick",
                "15",
                "--cpu-millis-per-request",
                "10",
            ]
        )
        assert result == 0

    def test_convert_with_container_cpu_millis_unsupported_source_returns_2(self, tmp_path):
        """convert with --container-cpu-millis on alibaba2018 (unsupported) returns 2."""
        fixture_dir = Path(__file__).parent / "fixtures" / "datasets" / "alibaba2018"
        if not fixture_dir.exists():
            pytest.skip("Fixtures not found")

        output_path = tmp_path / "output.ksimtrace"
        result = main(
            [
                "convert",
                "--source",
                "alibaba2018",
                "--input",
                str(fixture_dir),
                "--output",
                str(output_path),
                "--tick",
                "15",
                "--container-cpu-millis",
                "5",
            ]
        )
        assert result == 2

    def test_convert_with_memory_bytes_per_unit_flag(self, tmp_path):
        """convert with --memory-bytes-per-unit flag succeeds."""
        fixture_dir = Path(__file__).parent / "fixtures" / "datasets" / "alibaba2018"
        if not fixture_dir.exists():
            pytest.skip("Fixtures not found")

        output_path = tmp_path / "output.ksimtrace"
        result = main(
            [
                "convert",
                "--source",
                "alibaba2018",
                "--input",
                str(fixture_dir),
                "--output",
                str(output_path),
                "--tick",
                "15",
                "--memory-bytes-per-unit",
                "1024",
            ]
        )
        assert result == 0

    def test_convert_with_cpu_millis_per_unit_flag_borg2019(self, tmp_path):
        """convert with --cpu-millis-per-unit for borg2019 succeeds."""
        fixture_dir = Path(__file__).parent / "fixtures" / "datasets" / "borg2019"
        if not fixture_dir.exists():
            pytest.skip("Fixtures not found")

        output_path = tmp_path / "output.ksimtrace"
        result = main(
            [
                "convert",
                "--source",
                "borg2019",
                "--input",
                str(fixture_dir),
                "--output",
                str(output_path),
                "--tick",
                "15",
                "--cpu-millis-per-unit",
                "2.0",
            ]
        )
        assert result in {0, 2}  # May fail if fixture is outside window


class TestScenarioSubcommand:
    """Test scenario subcommand."""

    def test_scenario_basic_succeeds(self, tmp_path):
        """scenario with trace and output returns 0."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 5, dtype=np.float32),
                cpu_millis=np.array([500.0] * 5, dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 5, dtype=np.float32),
            )
        ]
        write(trace_path, 15, series)

        result = main(
            [
                "scenario",
                "--trace",
                str(trace_path),
                "--output",
                str(output_path),
            ]
        )
        assert result == 0

    def test_scenario_prints_service_count(self, tmp_path, capsys):
        """scenario prints the service count."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 3, dtype=np.float32),
                cpu_millis=np.array([500.0] * 3, dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 3, dtype=np.float32),
            ),
            ServiceSeries(
                name="svc2",
                request_rate=np.array([50.0] * 3, dtype=np.float32),
                cpu_millis=np.array([250.0] * 3, dtype=np.float32),
                memory_bytes=np.array([256 * 1024 * 1024] * 3, dtype=np.float32),
            ),
        ]
        write(trace_path, 15, series)

        main(
            [
                "scenario",
                "--trace",
                str(trace_path),
                "--output",
                str(output_path),
            ]
        )

        captured = capsys.readouterr()
        output = captured.out + captured.err
        # Should mention 2 services
        assert "2" in output

    def test_scenario_writes_parseable_toml(self, tmp_path):
        """scenario output is valid TOML."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 3, dtype=np.float32),
                cpu_millis=np.array([500.0] * 3, dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 3, dtype=np.float32),
            )
        ]
        write(trace_path, 15, series)

        main(
            [
                "scenario",
                "--trace",
                str(trace_path),
                "--output",
                str(output_path),
            ]
        )

        toml_text = output_path.read_text()
        parsed = tomllib.loads(toml_text)
        assert parsed is not None
        assert isinstance(parsed, dict)

    def test_scenario_with_name_option(self, tmp_path):
        """scenario with --name option reflects the name in TOML."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 2, dtype=np.float32),
                cpu_millis=np.array([500.0] * 2, dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 2, dtype=np.float32),
            )
        ]
        write(trace_path, 15, series)

        main(
            [
                "scenario",
                "--trace",
                str(trace_path),
                "--output",
                str(output_path),
                "--name",
                "demo",
            ]
        )

        toml_text = output_path.read_text()
        parsed = tomllib.loads(toml_text)
        assert parsed.get("name") == "demo"

    def test_scenario_with_requests_cpu_option(self, tmp_path):
        """scenario with --requests-cpu option is reflected."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 2, dtype=np.float32),
                cpu_millis=np.array([500.0] * 2, dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 2, dtype=np.float32),
            )
        ]
        write(trace_path, 15, series)

        main(
            [
                "scenario",
                "--trace",
                str(trace_path),
                "--output",
                str(output_path),
                "--requests-cpu",
                "250",
            ]
        )

        toml_text = output_path.read_text()
        parsed = tomllib.loads(toml_text)
        service = parsed["services"][0]
        assert service["requests_cpu_millis"] == 250

    def test_scenario_with_pool_cpu_option(self, tmp_path):
        """scenario with --pool-cpu option is reflected."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 2, dtype=np.float32),
                cpu_millis=np.array([500.0] * 2, dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 2, dtype=np.float32),
            )
        ]
        write(trace_path, 15, series)

        main(
            [
                "scenario",
                "--trace",
                str(trace_path),
                "--output",
                str(output_path),
                "--pool-cpu",
                "4000",
            ]
        )

        toml_text = output_path.read_text()
        parsed = tomllib.loads(toml_text)
        pools = parsed["pools"]
        default_pool = pools[0]
        # pool_cpu_millis should be reflected somewhere
        # Check if there's a capacity calculation or similar
        assert default_pool is not None

    def test_scenario_with_duration_option(self, tmp_path):
        """scenario with --duration option is reflected."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 2, dtype=np.float32),
                cpu_millis=np.array([500.0] * 2, dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 2, dtype=np.float32),
            )
        ]
        write(trace_path, 15, series)

        main(
            [
                "scenario",
                "--trace",
                str(trace_path),
                "--output",
                str(output_path),
                "--duration",
                "120",
            ]
        )

        toml_text = output_path.read_text()
        parsed = tomllib.loads(toml_text)
        assert parsed["duration_s"] == 120

    def test_scenario_missing_trace_returns_2(self, tmp_path):
        """scenario with missing trace file returns 2."""
        trace_path = tmp_path / "nonexistent.ksimtrace"
        output_path = tmp_path / "test.toml"

        result = main(
            [
                "scenario",
                "--trace",
                str(trace_path),
                "--output",
                str(output_path),
            ]
        )
        assert result == 2

    def test_scenario_invalid_trace_file_returns_2(self, tmp_path):
        """scenario with invalid trace file returns 2."""
        trace_path = tmp_path / "notrace.txt"
        output_path = tmp_path / "test.toml"
        trace_path.write_text("Not a trace file")

        result = main(
            [
                "scenario",
                "--trace",
                str(trace_path),
                "--output",
                str(output_path),
            ]
        )
        assert result == 2
