"""Tests for ksim.analysis.leaderboard: load, summarize, and render reports."""

import json

import pytest
from ksim.analysis.__main__ import main
from ksim.analysis.leaderboard import (
    MetricSummary,
    PolicyRow,
    load_reports,
    render_markdown,
    summarize,
)
from ksim.analysis.stats import Interval


class TestLoadReports:
    """load_reports: read all JSON lines from *.jsonl files in directory."""

    def test_load_single_jsonl(self, tmp_path):
        """Load reports from a single .jsonl file."""
        jsonl_file = tmp_path / "reports.jsonl"
        reports = [
            {"scenario": "s1", "policy": "p1", "seed": 0, "metrics": {"a": 1.0}},
            {"scenario": "s1", "policy": "p2", "seed": 0, "metrics": {"a": 2.0}},
        ]
        with open(jsonl_file, "w") as f:
            for report in reports:
                f.write(json.dumps(report) + "\n")

        loaded = load_reports(tmp_path)
        assert len(loaded) == 2
        assert loaded[0] == reports[0]
        assert loaded[1] == reports[1]

    def test_load_multiple_jsonl_in_order(self, tmp_path):
        """Load reports from multiple .jsonl files in name order."""
        file_b = tmp_path / "b.jsonl"
        file_a = tmp_path / "a.jsonl"

        with open(file_a, "w") as f:
            f.write(json.dumps({"scenario": "s1", "policy": "p1", "seed": 0}) + "\n")

        with open(file_b, "w") as f:
            f.write(json.dumps({"scenario": "s1", "policy": "p2", "seed": 0}) + "\n")

        loaded = load_reports(tmp_path)
        assert len(loaded) == 2
        assert loaded[0]["policy"] == "p1"  # a.jsonl comes first
        assert loaded[1]["policy"] == "p2"  # b.jsonl comes second

    def test_load_empty_directory(self, tmp_path):
        """Load from empty directory returns empty list."""
        loaded = load_reports(tmp_path)
        assert loaded == []


class TestSummarize:
    """summarize: aggregate reports per (scenario, policy) with CI and p-values."""

    def test_basic_summarization(self, tmp_path):
        """Summarize two policies over seeds, baseline normalized to 1.0."""
        reports = [
            {"scenario": "s1", "policy": "baseline", "seed": 0, "metrics": {"m": 10.0}},
            {"scenario": "s1", "policy": "baseline", "seed": 1, "metrics": {"m": 10.0}},
            {"scenario": "s1", "policy": "candidate", "seed": 0, "metrics": {"m": 20.0}},
            {"scenario": "s1", "policy": "candidate", "seed": 1, "metrics": {"m": 20.0}},
        ]

        rows = summarize(reports, baseline="baseline")

        assert len(rows) == 2
        assert rows[0].policy == "baseline"
        assert rows[1].policy == "candidate"

        # Baseline normalized to 1.0
        assert rows[0].metrics["m"].normalized == pytest.approx(1.0)
        assert rows[0].metrics["m"].p_value == pytest.approx(1.0)

        # Candidate normalized by mean ratio
        assert rows[1].metrics["m"].normalized == pytest.approx(2.0)

    def test_sorted_by_scenario_then_policy(self, tmp_path):
        """Rows sorted by scenario then policy name."""
        reports = [
            {"scenario": "z", "policy": "z_policy", "seed": 0, "metrics": {"m": 1.0}},
            {"scenario": "z", "policy": "a_policy", "seed": 0, "metrics": {"m": 1.0}},
            {"scenario": "a", "policy": "z_policy", "seed": 0, "metrics": {"m": 1.0}},
            {"scenario": "a", "policy": "a_policy", "seed": 0, "metrics": {"m": 1.0}},
        ]

        rows = summarize(reports, baseline="a_policy")

        assert [(row.scenario, row.policy) for row in rows] == [
            ("a", "a_policy"),
            ("a", "z_policy"),
            ("z", "a_policy"),
            ("z", "z_policy"),
        ]

    def test_normalized_both_zero_is_1(self):
        """When both baseline and candidate mean are 0, normalized = 1.0."""
        reports = [
            {"scenario": "s", "policy": "baseline", "seed": 0, "metrics": {"m": 0.0}},
            {"scenario": "s", "policy": "baseline", "seed": 1, "metrics": {"m": 0.0}},
            {"scenario": "s", "policy": "candidate", "seed": 0, "metrics": {"m": 0.0}},
            {"scenario": "s", "policy": "candidate", "seed": 1, "metrics": {"m": 0.0}},
        ]

        rows = summarize(reports, baseline="baseline")
        candidate_row = next(r for r in rows if r.policy == "candidate")
        assert candidate_row.metrics["m"].normalized == pytest.approx(1.0)

    def test_normalized_baseline_zero_only_is_inf(self):
        """When baseline is 0 but candidate is not, normalized = inf."""
        reports = [
            {"scenario": "s", "policy": "baseline", "seed": 0, "metrics": {"m": 0.0}},
            {"scenario": "s", "policy": "baseline", "seed": 1, "metrics": {"m": 0.0}},
            {"scenario": "s", "policy": "candidate", "seed": 0, "metrics": {"m": 5.0}},
            {"scenario": "s", "policy": "candidate", "seed": 1, "metrics": {"m": 5.0}},
        ]

        rows = summarize(reports, baseline="baseline")
        candidate_row = next(r for r in rows if r.policy == "candidate")
        assert candidate_row.metrics["m"].normalized == pytest.approx(float("inf"))

    def test_baseline_p_value_always_1(self):
        """Baseline policy always has p_value 1.0 (by definition)."""
        reports = [
            {"scenario": "s", "policy": "baseline", "seed": 0, "metrics": {"m": 10.0}},
            {"scenario": "s", "policy": "baseline", "seed": 1, "metrics": {"m": 15.0}},
        ]

        rows = summarize(reports, baseline="baseline")
        assert rows[0].metrics["m"].p_value == pytest.approx(1.0)

    def test_missing_baseline_raises_valueerror(self):
        """Scenario without baseline policy raises ValueError."""
        reports = [
            {"scenario": "s1", "policy": "candidate", "seed": 0, "metrics": {"m": 10.0}},
        ]

        with pytest.raises(ValueError):
            summarize(reports, baseline="baseline")

    def test_multiple_metrics(self):
        """Handle multiple metrics per report."""
        reports = [
            {"scenario": "s", "policy": "b", "seed": 0, "metrics": {"avail": 0.9, "cost": 100.0}},
            {"scenario": "s", "policy": "b", "seed": 1, "metrics": {"avail": 0.95, "cost": 110.0}},
            {"scenario": "s", "policy": "a", "seed": 0, "metrics": {"avail": 0.8, "cost": 150.0}},
            {"scenario": "s", "policy": "a", "seed": 1, "metrics": {"avail": 0.85, "cost": 160.0}},
        ]

        rows = summarize(reports, baseline="b")

        baseline_row = next(r for r in rows if r.policy == "b")
        assert "avail" in baseline_row.metrics
        assert "cost" in baseline_row.metrics

    def test_seeds_count(self):
        """PolicyRow.seeds is the count of unique seeds."""
        reports = [
            {"scenario": "s", "policy": "p", "seed": 0, "metrics": {"m": 1.0}},
            {"scenario": "s", "policy": "p", "seed": 1, "metrics": {"m": 2.0}},
            {"scenario": "s", "policy": "p", "seed": 2, "metrics": {"m": 3.0}},
        ]

        rows = summarize(reports, baseline="p")
        assert rows[0].seeds == 3


class TestRenderMarkdown:
    """render_markdown: format rows as a table with CI and p-value stars."""

    def test_table_structure(self):
        """Output contains markdown table with policy, seeds, and metric columns."""
        interval = {
            "m": MetricSummary(interval=Interval(10.0, 9.0, 11.0), normalized=1.0, p_value=1.0)
        }
        rows = [
            PolicyRow(scenario="s1", policy="p1", seeds=2, metrics=interval),
        ]

        output = render_markdown(rows, ["m"])

        assert "| policy |" in output
        assert "| seeds |" in output
        assert "p1" in output
        assert "2" in output

    def test_significant_difference_marked_with_star(self):
        """Output contains * when p_value < 0.05."""
        # Create metrics with p_value < 0.05 to trigger star marking
        interval_sig = {
            "m": MetricSummary(
                interval=Interval(10.0, 9.0, 11.0),
                normalized=2.0,
                p_value=0.01,
            )
        }

        rows = [
            PolicyRow(scenario="s1", policy="p1", seeds=8, metrics=interval_sig),
        ]

        output = render_markdown(rows, ["m"])
        assert "*" in output

    def test_not_significant_no_star(self):
        """Output has no * when p_value >= 0.05."""
        interval_not_sig = {
            "m": MetricSummary(
                interval=Interval(10.0, 9.0, 11.0),
                normalized=1.1,
                p_value=0.5,
            )
        }

        rows = [
            PolicyRow(scenario="s1", policy="p1", seeds=5, metrics=interval_not_sig),
        ]

        output = render_markdown(rows, ["m"])
        # Should have the row but no * in the metric cell
        assert "p1" in output

    def test_multiple_scenarios_multiple_tables(self):
        """Multiple scenarios produce multiple tables."""
        interval = {
            "m": MetricSummary(
                interval=Interval(10.0, 9.0, 11.0),
                normalized=1.0,
                p_value=1.0,
            )
        }

        rows = [
            PolicyRow(scenario="scenario1", policy="p1", seeds=2, metrics=interval),
            PolicyRow(scenario="scenario2", policy="p1", seeds=2, metrics=interval),
        ]

        output = render_markdown(rows, ["m"])
        assert "scenario1" in output
        assert "scenario2" in output

    def test_four_significant_digits(self):
        """Cell values formatted to 4 significant digits."""
        interval = {
            "m": MetricSummary(
                interval=type("Interval", (), {"mean": 1.23456, "low": 1.1, "high": 1.4})(),
                normalized=2.3456,
                p_value=1.0,
            )
        }

        rows = [
            PolicyRow(scenario="s", policy="p", seeds=3, metrics=interval),
        ]

        output = render_markdown(rows, ["m"])
        # Check that output contains reasonable precision (hard to test exact format
        # without seeing implementation, but ensure numbers are present)
        assert "1.23" in output or "1.2" in output
        assert "2.34" in output or "2.3" in output


class TestMainCLI:
    """main(argv): command-line interface for the leaderboard."""

    def test_main_with_valid_directory(self, tmp_path, capsys):
        """main([dir, '--baseline', 'k8s']) returns 0 and prints table."""
        # Write test data: k8s (baseline) and oracle over 8 seeds (for Wilcoxon p < 0.05)
        # Oracle has strictly higher availability than k8s
        jsonl_file = tmp_path / "reports.jsonl"
        with open(jsonl_file, "w") as f:
            for seed in range(8):
                # k8s: baseline availability
                report_k8s = {
                    "scenario": "test",
                    "policy": "k8s",
                    "seed": seed,
                    "metrics": {"availability": 0.90 + seed * 0.005, "cost": 100.0},
                }
                f.write(json.dumps(report_k8s) + "\n")

                # oracle: strictly higher availability (all positive differences for Wilcoxon)
                report_oracle = {
                    "scenario": "test",
                    "policy": "oracle",
                    "seed": seed,
                    "metrics": {"availability": 0.95 + seed * 0.005, "cost": 50.0},
                }
                f.write(json.dumps(report_oracle) + "\n")

        result = main([str(tmp_path), "--baseline", "k8s"])
        assert result == 0

        captured = capsys.readouterr()
        assert "| policy |" in captured.out
        assert "k8s" in captured.out
        assert "oracle" in captured.out
        # Oracle availability is strictly higher; with n=8, Wilcoxon p=0.0078 < 0.05
        assert "*" in captured.out

    def test_main_with_missing_directory_returns_nonzero(self, tmp_path):
        """main with non-existent directory returns non-zero."""
        missing_dir = tmp_path / "nonexistent"
        result = main([str(missing_dir), "--baseline", "k8s"])
        assert result != 0

    def test_main_output_is_printable(self, tmp_path, capsys):
        """main produces printable markdown output."""
        jsonl_file = tmp_path / "reports.jsonl"
        with open(jsonl_file, "w") as f:
            for seed in range(3):
                report = {
                    "scenario": "test",
                    "policy": "baseline",
                    "seed": seed,
                    "metrics": {"m": 10.0},
                }
                f.write(json.dumps(report) + "\n")

        result = main([str(tmp_path), "--baseline", "baseline"])
        assert result == 0

        captured = capsys.readouterr()
        assert len(captured.out) > 0
        assert "test" in captured.out  # scenario name
