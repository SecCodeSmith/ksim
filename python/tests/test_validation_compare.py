"""Tests for ksim.validation.compare: compare and render_markdown."""

import numpy as np
import pytest
from ksim.validation.compare import ValidationReport, compare, render_markdown
from ksim.validation.trajectory import Trajectory


class TestCompare:
    """Test compare function."""

    def test_compare_basic(self):
        """Compare two trajectories."""
        sim = Trajectory(
            tick_s=5,
            demand_rps=np.array([100.0, 110.0]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.01, 0.02]),
        )
        obs = Trajectory(
            tick_s=5,
            demand_rps=np.array([np.nan, np.nan]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([51.0, 53.0]),
            error_rate=np.array([0.01, 0.02]),
        )
        report = compare(sim, obs)
        assert report.ticks == 2
        assert report.ready_mae == pytest.approx(0.0)
        assert report.desired_mae == pytest.approx(0.0)

    def test_compare_truncate_to_shorter(self):
        """Truncate to shorter trajectory."""
        sim = Trajectory(
            tick_s=5,
            demand_rps=np.array([100.0, 110.0, 120.0]),
            desired=np.array([10.0, 12.0, 14.0]),
            ready=np.array([9.0, 11.0, 13.0]),
            latency_p95_ms=np.array([50.0, 52.0, 54.0]),
            error_rate=np.array([0.01, 0.02, 0.03]),
        )
        obs = Trajectory(
            tick_s=5,
            demand_rps=np.array([np.nan, np.nan]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([51.0, 53.0]),
            error_rate=np.array([0.01, 0.02]),
        )
        report = compare(sim, obs)
        assert report.ticks == 2

    def test_compare_ready_mae(self):
        """Calculate ready_mae."""
        sim = Trajectory(
            tick_s=5,
            demand_rps=np.array([100.0, 110.0]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.0, 0.0]),
        )
        obs = Trajectory(
            tick_s=5,
            demand_rps=np.array([np.nan, np.nan]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([10.0, 12.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.0, 0.0]),
        )
        report = compare(sim, obs)
        assert report.ready_mae == pytest.approx(1.0)

    def test_compare_ready_rmse(self):
        """Calculate ready_rmse."""
        sim = Trajectory(
            tick_s=5,
            demand_rps=np.array([100.0, 110.0]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.0, 0.0]),
        )
        obs = Trajectory(
            tick_s=5,
            demand_rps=np.array([np.nan, np.nan]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([10.0, 12.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.0, 0.0]),
        )
        report = compare(sim, obs)
        expected_rmse = np.sqrt((1.0**2 + 1.0**2) / 2)
        assert report.ready_rmse == pytest.approx(expected_rmse)

    def test_compare_ready_max_error(self):
        """Calculate ready_max_error."""
        sim = Trajectory(
            tick_s=5,
            demand_rps=np.array([100.0, 110.0]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.0, 0.0]),
        )
        obs = Trajectory(
            tick_s=5,
            demand_rps=np.array([np.nan, np.nan]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([10.0, 14.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.0, 0.0]),
        )
        report = compare(sim, obs)
        assert report.ready_max_error == pytest.approx(3.0)

    def test_compare_desired_mae(self):
        """Calculate desired_mae."""
        sim = Trajectory(
            tick_s=5,
            demand_rps=np.array([100.0, 110.0]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.0, 0.0]),
        )
        obs = Trajectory(
            tick_s=5,
            demand_rps=np.array([np.nan, np.nan]),
            desired=np.array([11.0, 13.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.0, 0.0]),
        )
        report = compare(sim, obs)
        assert report.desired_mae == pytest.approx(1.0)

    def test_compare_scale_up_lag_positive(self):
        """Calculate positive scale_up_lag_s."""
        sim = Trajectory(
            tick_s=5,
            demand_rps=np.array([100.0, 110.0, 120.0]),
            desired=np.array([10.0, 12.0, 15.0]),
            ready=np.array([9.0, 11.0, 14.0]),
            latency_p95_ms=np.array([50.0, 52.0, 54.0]),
            error_rate=np.array([0.0, 0.0, 0.0]),
        )
        obs = Trajectory(
            tick_s=5,
            demand_rps=np.array([np.nan, np.nan, np.nan]),
            desired=np.array([10.0, 10.0, 15.0]),
            ready=np.array([9.0, 9.0, 14.0]),
            latency_p95_ms=np.array([50.0, 52.0, 54.0]),
            error_rate=np.array([0.0, 0.0, 0.0]),
        )
        report = compare(sim, obs)
        assert report.scale_up_lag_s == pytest.approx(5.0)

    def test_compare_scale_up_lag_negative(self):
        """Calculate negative scale_up_lag_s."""
        sim = Trajectory(
            tick_s=5,
            demand_rps=np.array([100.0, 110.0, 120.0]),
            desired=np.array([10.0, 10.0, 15.0]),
            ready=np.array([9.0, 9.0, 14.0]),
            latency_p95_ms=np.array([50.0, 52.0, 54.0]),
            error_rate=np.array([0.0, 0.0, 0.0]),
        )
        obs = Trajectory(
            tick_s=5,
            demand_rps=np.array([np.nan, np.nan, np.nan]),
            desired=np.array([10.0, 12.0, 15.0]),
            ready=np.array([9.0, 11.0, 14.0]),
            latency_p95_ms=np.array([50.0, 52.0, 54.0]),
            error_rate=np.array([0.0, 0.0, 0.0]),
        )
        report = compare(sim, obs)
        assert report.scale_up_lag_s == pytest.approx(-5.0)

    def test_compare_scale_up_lag_none(self):
        """scale_up_lag_s is None when no scale-up."""
        sim = Trajectory(
            tick_s=5,
            demand_rps=np.array([100.0, 110.0]),
            desired=np.array([10.0, 10.0]),
            ready=np.array([9.0, 9.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.0, 0.0]),
        )
        obs = Trajectory(
            tick_s=5,
            demand_rps=np.array([np.nan, np.nan]),
            desired=np.array([10.0, 10.0]),
            ready=np.array([9.0, 9.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.0, 0.0]),
        )
        report = compare(sim, obs)
        assert report.scale_up_lag_s is None

    def test_compare_latency_relative_error_median(self):
        """Calculate latency_p95_relative_error as median."""
        sim = Trajectory(
            tick_s=5,
            demand_rps=np.array([100.0, 110.0, 120.0]),
            desired=np.array([10.0, 12.0, 14.0]),
            ready=np.array([9.0, 11.0, 13.0]),
            latency_p95_ms=np.array([100.0, 110.0, 120.0]),
            error_rate=np.array([0.0, 0.0, 0.0]),
        )
        obs = Trajectory(
            tick_s=5,
            demand_rps=np.array([np.nan, np.nan, np.nan]),
            desired=np.array([10.0, 12.0, 14.0]),
            ready=np.array([9.0, 11.0, 13.0]),
            latency_p95_ms=np.array([100.0, 100.0, 100.0]),
            error_rate=np.array([0.0, 0.0, 0.0]),
        )
        report = compare(sim, obs)
        assert report.latency_p95_relative_error == pytest.approx(0.1)

    def test_compare_latency_relative_error_none(self):
        """latency_p95_relative_error is None without valid ticks."""
        sim = Trajectory(
            tick_s=5,
            demand_rps=np.array([100.0, 110.0]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([np.nan, np.nan]),
            error_rate=np.array([0.0, 0.0]),
        )
        obs = Trajectory(
            tick_s=5,
            demand_rps=np.array([np.nan, np.nan]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([np.nan, np.nan]),
            error_rate=np.array([0.0, 0.0]),
        )
        report = compare(sim, obs)
        assert report.latency_p95_relative_error is None

    def test_compare_error_rate_mae(self):
        """Calculate error_rate_mae."""
        sim = Trajectory(
            tick_s=5,
            demand_rps=np.array([100.0, 110.0]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.01, 0.02]),
        )
        obs = Trajectory(
            tick_s=5,
            demand_rps=np.array([np.nan, np.nan]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.02, 0.03]),
        )
        report = compare(sim, obs)
        assert report.error_rate_mae == pytest.approx(0.01)

    def test_compare_skips_nan_values(self):
        """Skip ticks with NaN values per metric."""
        sim = Trajectory(
            tick_s=5,
            demand_rps=np.array([100.0, 110.0]),
            desired=np.array([np.nan, 12.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.0, 0.0]),
        )
        obs = Trajectory(
            tick_s=5,
            demand_rps=np.array([np.nan, np.nan]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.0, 0.0]),
        )
        report = compare(sim, obs)
        assert report.ticks == 2

    def test_compare_differing_tick_s_error(self):
        """Raise ValueError when tick lengths differ."""
        sim = Trajectory(
            tick_s=5,
            demand_rps=np.array([100.0, 110.0]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.0, 0.0]),
        )
        obs = Trajectory(
            tick_s=10,
            demand_rps=np.array([np.nan, np.nan]),
            desired=np.array([10.0, 12.0]),
            ready=np.array([9.0, 11.0]),
            latency_p95_ms=np.array([50.0, 52.0]),
            error_rate=np.array([0.0, 0.0]),
        )
        with pytest.raises(ValueError):
            compare(sim, obs)

    def test_compare_no_common_tick_error(self):
        """Raise ValueError when there is no common tick."""
        sim = Trajectory(
            tick_s=5,
            demand_rps=np.array([]),
            desired=np.array([]),
            ready=np.array([]),
            latency_p95_ms=np.array([]),
            error_rate=np.array([]),
        )
        obs = Trajectory(
            tick_s=5,
            demand_rps=np.array([np.nan]),
            desired=np.array([10.0]),
            ready=np.array([9.0]),
            latency_p95_ms=np.array([50.0]),
            error_rate=np.array([0.0]),
        )
        with pytest.raises(ValueError):
            compare(sim, obs)


class TestRenderMarkdown:
    """Test render_markdown function."""

    def test_render_markdown_basic(self):
        """Render basic markdown table."""
        report = ValidationReport(
            ticks=10,
            ready_mae=1.5,
            ready_rmse=2.0,
            ready_max_error=5.0,
            desired_mae=0.5,
            scale_up_lag_s=2.5,
            latency_p95_relative_error=0.1,
            error_rate_mae=0.01,
        )
        markdown = render_markdown(report)
        assert "## Simulator vs. kind cluster" in markdown
        assert "| Metric | Value |" in markdown

    def test_render_markdown_table_header(self):
        """Markdown contains table header."""
        report = ValidationReport(
            ticks=10,
            ready_mae=1.5,
            ready_rmse=2.0,
            ready_max_error=5.0,
            desired_mae=0.5,
            scale_up_lag_s=2.5,
            latency_p95_relative_error=0.1,
            error_rate_mae=0.01,
        )
        markdown = render_markdown(report)
        lines = markdown.split("\n")
        assert any("Metric" in line and "Value" in line for line in lines)

    def test_render_markdown_field_order(self):
        """One row per field in declaration order."""
        report = ValidationReport(
            ticks=10,
            ready_mae=1.5,
            ready_rmse=2.0,
            ready_max_error=5.0,
            desired_mae=0.5,
            scale_up_lag_s=2.5,
            latency_p95_relative_error=0.1,
            error_rate_mae=0.01,
        )
        markdown = render_markdown(report)
        assert "ticks" in markdown
        assert "ready_mae" in markdown
        assert "ready_rmse" in markdown
        assert "ready_max_error" in markdown
        assert "desired_mae" in markdown
        assert "scale_up_lag_s" in markdown
        assert "latency_p95_relative_error" in markdown
        assert "error_rate_mae" in markdown

    def test_render_markdown_two_decimal_floats(self):
        """Floats rendered with two decimals."""
        report = ValidationReport(
            ticks=10,
            ready_mae=1.567,
            ready_rmse=2.123,
            ready_max_error=5.999,
            desired_mae=0.444,
            scale_up_lag_s=2.5,
            latency_p95_relative_error=0.1111,
            error_rate_mae=0.01,
        )
        markdown = render_markdown(report)
        assert "1.57" in markdown
        assert "2.12" in markdown
        assert "6.00" in markdown
        assert "0.44" in markdown
        assert "2.50" in markdown
        assert "0.11" in markdown
        assert "0.01" in markdown

    def test_render_markdown_none_as_na(self):
        """None values rendered as 'n/a'."""
        report = ValidationReport(
            ticks=10,
            ready_mae=1.5,
            ready_rmse=2.0,
            ready_max_error=5.0,
            desired_mae=0.5,
            scale_up_lag_s=None,
            latency_p95_relative_error=None,
            error_rate_mae=0.01,
        )
        markdown = render_markdown(report)
        assert "n/a" in markdown
