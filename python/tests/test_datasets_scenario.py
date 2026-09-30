"""Tests for ksim.datasets.scenario module."""

import tomllib

import numpy as np
import pytest
from ksim.datasets.ksimtrace import ServiceSeries, write
from ksim.datasets.scenario import ScenarioOptions, generate, render


class TestScenarioRender:
    """Test scenario render function."""

    def test_render_constant_series(self, tmp_path):
        """render with constant series produces valid TOML."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        # Create simple trace: 1 service, 10 ticks, constant values
        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 10, dtype=np.float32),
                cpu_millis=np.array([1000.0] * 10, dtype=np.float32),
                memory_bytes=np.array([1024 * 1024 * 1024] * 10, dtype=np.float32),
            )
        ]
        write(trace_path, 15, series)

        options = ScenarioOptions()
        toml_text = render(trace_path, output_path, options)

        # Should be valid TOML
        parsed = tomllib.loads(toml_text)
        assert parsed is not None
        assert isinstance(parsed, dict)

    def test_render_includes_service_sections(self, tmp_path):
        """render includes [[services]] for each series."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 5, dtype=np.float32),
                cpu_millis=np.array([500.0] * 5, dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 5, dtype=np.float32),
            ),
            ServiceSeries(
                name="svc2",
                request_rate=np.array([50.0] * 5, dtype=np.float32),
                cpu_millis=np.array([250.0] * 5, dtype=np.float32),
                memory_bytes=np.array([256 * 1024 * 1024] * 5, dtype=np.float32),
            ),
        ]
        write(trace_path, 15, series)

        options = ScenarioOptions()
        toml_text = render(trace_path, output_path, options)
        parsed = tomllib.loads(toml_text)

        assert "services" in parsed
        services = parsed["services"]
        assert len(services) == 2
        assert services[0]["name"] == "svc1"
        assert services[1]["name"] == "svc2"

    def test_render_service_curve_points_to_trace(self, tmp_path):
        """render sets curve.kind=trace and curve.service to service name."""
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
        write(trace_path, 10, series)

        options = ScenarioOptions()
        toml_text = render(trace_path, output_path, options)
        parsed = tomllib.loads(toml_text)

        service = parsed["services"][0]
        assert service["curve"]["kind"] == "trace"
        assert service["curve"]["service"] == "svc1"

    def test_render_trace_section_tick_s_equals_trace_tick(self, tmp_path):
        """render sets trace.tick_s to the trace file's tick."""
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
        write(trace_path, 12, series)

        options = ScenarioOptions()
        toml_text = render(trace_path, output_path, options)
        parsed = tomllib.loads(toml_text)

        assert parsed["trace"]["tick_s"] == 12

    def test_render_duration_s_defaults_to_ticks_times_tick(self, tmp_path):
        """render duration_s defaults to ticks * tick_s."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 4, dtype=np.float32),
                cpu_millis=np.array([500.0] * 4, dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 4, dtype=np.float32),
            )
        ]
        write(trace_path, 10, series)

        options = ScenarioOptions()
        toml_text = render(trace_path, output_path, options)
        parsed = tomllib.loads(toml_text)

        # 4 ticks * 10 seconds = 40 seconds
        assert parsed["trace"]["duration_s"] == 40

    def test_render_duration_s_honors_option(self, tmp_path):
        """render respects options.duration_s."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 4, dtype=np.float32),
                cpu_millis=np.array([500.0] * 4, dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 4, dtype=np.float32),
            )
        ]
        write(trace_path, 10, series)

        options = ScenarioOptions(duration_s=60)
        toml_text = render(trace_path, output_path, options)
        parsed = tomllib.loads(toml_text)

        assert parsed["trace"]["duration_s"] == 60

    def test_render_name_defaults_to_trace_stem(self, tmp_path):
        """render name defaults to trace file stem."""
        trace_path = tmp_path / "data" / "my_service.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 2, dtype=np.float32),
                cpu_millis=np.array([500.0] * 2, dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 2, dtype=np.float32),
            )
        ]
        write(trace_path, 10, series)

        options = ScenarioOptions()
        toml_text = render(trace_path, output_path, options)
        parsed = tomllib.loads(toml_text)

        assert parsed.get("name") == "my_service"

    def test_render_name_honors_option(self, tmp_path):
        """render respects options.name."""
        trace_path = tmp_path / "data" / "my_service.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 2, dtype=np.float32),
                cpu_millis=np.array([500.0] * 2, dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 2, dtype=np.float32),
            )
        ]
        write(trace_path, 10, series)

        options = ScenarioOptions(name="demo")
        toml_text = render(trace_path, output_path, options)
        parsed = tomllib.loads(toml_text)

        assert parsed.get("name") == "demo"

    def test_render_trace_path_relative_to_output_directory(self, tmp_path):
        """render trace.path is relative to output directory."""
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
        write(trace_path, 10, series)

        options = ScenarioOptions()
        toml_text = render(trace_path, output_path, options)
        parsed = tomllib.loads(toml_text)

        # trace is in tmp_path/data, output is in tmp_path/scenarios
        # relative path should be ../data/test.ksimtrace
        assert parsed["trace"]["path"] == "../data/test.ksimtrace"

    def test_render_trace_wrap_is_true(self, tmp_path):
        """render sets trace.wrap = true."""
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
        write(trace_path, 10, series)

        options = ScenarioOptions()
        toml_text = render(trace_path, output_path, options)
        parsed = tomllib.loads(toml_text)

        assert parsed["trace"]["wrap"] is True

    def test_render_cpu_millis_per_request_is_median_over_positive_rates(self, tmp_path):
        """cpu_millis_per_request is median of cpu/rate over ticks with positive rate."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        # Create series with varying ratios:
        # tick 0: cpu=100, rate=10 → ratio=10
        # tick 1: cpu=200, rate=20 → ratio=10
        # tick 2: cpu=300, rate=30 → ratio=10
        # median = 10
        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([10.0, 20.0, 30.0], dtype=np.float32),
                cpu_millis=np.array([100.0, 200.0, 300.0], dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 3, dtype=np.float32),
            )
        ]
        write(trace_path, 10, series)

        options = ScenarioOptions()
        toml_text = render(trace_path, output_path, options)
        parsed = tomllib.loads(toml_text)

        service = parsed["services"][0]
        # All ratios are 10, so median is 10
        assert service["cpu_millis_per_request"] == pytest.approx(10.0)

    def test_render_cpu_millis_per_request_falls_back_to_option(self, tmp_path):
        """cpu_millis_per_request falls back to option when all rates are zero."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        # All rates are zero
        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([0.0, 0.0, 0.0], dtype=np.float32),
                cpu_millis=np.array([100.0, 200.0, 300.0], dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 3, dtype=np.float32),
            )
        ]
        write(trace_path, 10, series)

        options = ScenarioOptions(cpu_millis_per_request=42.5)
        toml_text = render(trace_path, output_path, options)
        parsed = tomllib.loads(toml_text)

        service = parsed["services"][0]
        assert service["cpu_millis_per_request"] == pytest.approx(42.5)

    def test_render_requests_cpu_millis_from_option(self, tmp_path):
        """render uses options.requests_cpu_millis."""
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
        write(trace_path, 10, series)

        options = ScenarioOptions(requests_cpu_millis=750)
        toml_text = render(trace_path, output_path, options)
        parsed = tomllib.loads(toml_text)

        service = parsed["services"][0]
        assert service["requests_cpu_millis"] == 750

    def test_render_memory_rounded_to_64_mib(self, tmp_path):
        """requests_memory_mib is rounded up to multiple of 64 MiB, at least 64."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        # P95 of memory = constant 1024 MiB = 1024 * 1024 * 1024 bytes
        # With constant series, P95 = the value itself
        # requests_memory_mib = 1024 / 1 (1 initial replica) = 1024
        # This is already a multiple of 64
        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 3, dtype=np.float32),
                cpu_millis=np.array([500.0] * 3, dtype=np.float32),
                memory_bytes=np.array([1024 * 1024 * 1024] * 3, dtype=np.float32),
            )
        ]
        write(trace_path, 10, series)

        options = ScenarioOptions()
        toml_text = render(trace_path, output_path, options)
        parsed = tomllib.loads(toml_text)

        service = parsed["services"][0]
        mem = service["requests_memory_mib"]
        # Should be multiple of 64 and at least 64
        assert mem >= 64
        assert mem % 64 == 0

    def test_render_raises_for_non_positive_requests_cpu_millis(self, tmp_path):
        """render raises ValueError for non-positive requests_cpu_millis."""
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
        write(trace_path, 10, series)

        options = ScenarioOptions(requests_cpu_millis=0)
        with pytest.raises(ValueError):
            render(trace_path, output_path, options)

    def test_render_raises_for_non_positive_hpa_target(self, tmp_path):
        """render raises ValueError for non-positive hpa_target."""
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
        write(trace_path, 10, series)

        options = ScenarioOptions(hpa_target=0.0)
        with pytest.raises(ValueError):
            render(trace_path, output_path, options)

    def test_render_raises_for_non_positive_cpu_millis_per_request(self, tmp_path):
        """render raises ValueError for non-positive cpu_millis_per_request option."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([0.0] * 2, dtype=np.float32),
                cpu_millis=np.array([500.0] * 2, dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 2, dtype=np.float32),
            )
        ]
        write(trace_path, 10, series)

        options = ScenarioOptions(cpu_millis_per_request=0.0)
        with pytest.raises(ValueError):
            render(trace_path, output_path, options)

    def test_render_raises_for_unreadable_trace(self, tmp_path):
        """render raises ValueError for non-existent trace file."""
        trace_path = tmp_path / "nonexistent.ksimtrace"
        output_path = tmp_path / "test.toml"

        options = ScenarioOptions()
        with pytest.raises(ValueError):
            render(trace_path, output_path, options)

    def test_render_raises_for_non_trace_file(self, tmp_path):
        """render raises ValueError for a file that is not a trace."""
        trace_path = tmp_path / "notrace.txt"
        output_path = tmp_path / "test.toml"
        trace_path.write_text("This is not a trace file")

        options = ScenarioOptions()
        with pytest.raises(ValueError):
            render(trace_path, output_path, options)


class TestScenarioGenerate:
    """Test scenario generate function."""

    def test_generate_writes_file(self, tmp_path):
        """generate writes output file."""
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
        write(trace_path, 10, series)

        options = ScenarioOptions()
        generate(trace_path, output_path, options)

        assert output_path.exists()
        assert output_path.read_text()

    def test_generate_creates_parent_directories(self, tmp_path):
        """generate creates parent directories."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "a" / "b" / "c" / "scenarios" / "test.toml"

        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 2, dtype=np.float32),
                cpu_millis=np.array([500.0] * 2, dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 2, dtype=np.float32),
            )
        ]
        write(trace_path, 10, series)

        options = ScenarioOptions()
        generate(trace_path, output_path, options)

        assert output_path.parent.exists()
        assert output_path.exists()

    def test_generate_returns_service_count(self, tmp_path):
        """generate returns the number of services."""
        trace_path = tmp_path / "data" / "test.ksimtrace"
        output_path = tmp_path / "scenarios" / "test.toml"

        series = [
            ServiceSeries(
                name="svc1",
                request_rate=np.array([100.0] * 2, dtype=np.float32),
                cpu_millis=np.array([500.0] * 2, dtype=np.float32),
                memory_bytes=np.array([512 * 1024 * 1024] * 2, dtype=np.float32),
            ),
            ServiceSeries(
                name="svc2",
                request_rate=np.array([50.0] * 2, dtype=np.float32),
                cpu_millis=np.array([250.0] * 2, dtype=np.float32),
                memory_bytes=np.array([256 * 1024 * 1024] * 2, dtype=np.float32),
            ),
        ]
        write(trace_path, 10, series)

        options = ScenarioOptions()
        count = generate(trace_path, output_path, options)

        assert count == 2

    def test_generate_output_is_valid_toml(self, tmp_path):
        """generate output is parseable TOML."""
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
        write(trace_path, 10, series)

        options = ScenarioOptions()
        generate(trace_path, output_path, options)

        toml_text = output_path.read_text()
        parsed = tomllib.loads(toml_text)
        assert parsed is not None
