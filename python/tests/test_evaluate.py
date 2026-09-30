"""Tests for ksim.evaluate module."""

import json
import subprocess
import tomllib
from pathlib import Path

import ksim._native as native
import numpy as np
import pytest
from ksim.analysis.leaderboard import load_reports
from ksim.evaluate import evaluate, write_reports
from ksim.gym import EnvOptions

FIXTURES_DIR = Path(__file__).parent / "fixtures"
TINY_SCENARIO = FIXTURES_DIR / "scenarios" / "tiny.toml"
REPO_ROOT = Path(__file__).parent.parent.parent


class TestEvaluateBasics:
    """Test basic behavior of evaluate."""

    def test_evaluate_returns_list_of_dicts(self):
        """evaluate returns a list of dicts."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        reports = evaluate(TINY_SCENARIO, policy, "test_policy", seeds=[0])
        assert isinstance(reports, list)
        assert len(reports) == 1
        assert isinstance(reports[0], dict)

    def test_evaluate_returns_one_report_per_seed(self):
        """evaluate returns one report per seed."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        seeds = [0, 1, 2]
        reports = evaluate(TINY_SCENARIO, policy, "test_policy", seeds=seeds)
        assert len(reports) == 3

    def test_evaluate_returns_reports_in_seed_order(self):
        """evaluate returns reports in the same order as seeds."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        seeds = [5, 1, 3]
        reports = evaluate(TINY_SCENARIO, policy, "test_policy", seeds=seeds)
        assert [r["seed"] for r in reports] == [5, 1, 3]


class TestEvaluateReportStructure:
    """Test the structure of reports returned by evaluate."""

    def test_report_has_scenario_key(self):
        """Each report has a 'scenario' key."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        reports = evaluate(TINY_SCENARIO, policy, "test_policy", seeds=[0])
        assert "scenario" in reports[0]

    def test_report_has_policy_key(self):
        """Each report has a 'policy' key."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        reports = evaluate(TINY_SCENARIO, policy, "test_policy", seeds=[0])
        assert "policy" in reports[0]

    def test_report_has_seed_key(self):
        """Each report has a 'seed' key."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        reports = evaluate(TINY_SCENARIO, policy, "test_policy", seeds=[0])
        assert "seed" in reports[0]

    def test_report_has_metrics_key(self):
        """Each report has a 'metrics' key."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        reports = evaluate(TINY_SCENARIO, policy, "test_policy", seeds=[0])
        assert "metrics" in reports[0]

    def test_scenario_equals_toml_name(self):
        """scenario field equals the 'name' from TOML."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        reports = evaluate(TINY_SCENARIO, policy, "test_policy", seeds=[0])
        with open(TINY_SCENARIO, "rb") as f:
            config = tomllib.load(f)
        assert reports[0]["scenario"] == config["name"]

    def test_policy_equals_policy_name(self):
        """policy field equals the policy_name parameter."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        policy_name = "my_test_policy"
        reports = evaluate(TINY_SCENARIO, policy, policy_name, seeds=[0])
        assert reports[0]["policy"] == policy_name

    def test_seed_equals_input_seed(self):
        """seed field equals the input seed."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        reports = evaluate(TINY_SCENARIO, policy, "test_policy", seeds=[42])
        assert reports[0]["seed"] == 42

    def test_metrics_is_non_empty_dict(self):
        """metrics is a non-empty dict."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        reports = evaluate(TINY_SCENARIO, policy, "test_policy", seeds=[0])
        metrics = reports[0]["metrics"]
        assert isinstance(metrics, dict)
        assert len(metrics) > 0

    def test_metrics_contains_float_values(self):
        """metrics dict contains float values."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        reports = evaluate(TINY_SCENARIO, policy, "test_policy", seeds=[0])
        metrics = reports[0]["metrics"]
        assert all(isinstance(v, (float, int)) for v in metrics.values())


class TestEvaluatePolicyCallCount:
    """Test that the policy is called the correct number of times."""

    def test_policy_called_exact_episode_steps_times(self):
        """Policy is called exactly episode_steps times per seed."""
        call_count = 0

        def counting_policy(obs: np.ndarray) -> np.ndarray:
            nonlocal call_count
            call_count += 1
            return np.full(3, 1, dtype=np.uint32)

        env = native.KsimEnv(str(TINY_SCENARIO))
        episode_steps = env.episode_steps
        evaluate(TINY_SCENARIO, counting_policy, "test_policy", seeds=[0])
        assert call_count == episode_steps

    def test_policy_called_correct_times_multiple_seeds(self):
        """Policy is called episode_steps times per seed."""
        call_count = 0

        def counting_policy(obs: np.ndarray) -> np.ndarray:
            nonlocal call_count
            call_count += 1
            return np.full(3, 1, dtype=np.uint32)

        env = native.KsimEnv(str(TINY_SCENARIO))
        episode_steps = env.episode_steps
        seeds = [0, 1, 2]
        evaluate(TINY_SCENARIO, counting_policy, "test_policy", seeds=seeds)
        assert call_count == episode_steps * len(seeds)

    def test_policy_receives_float32_observations(self):
        """Policy receives float32 arrays."""
        received_dtypes = []

        def dtype_checking_policy(obs: np.ndarray) -> np.ndarray:
            received_dtypes.append(obs.dtype)
            return np.full(3, 1, dtype=np.uint32)

        evaluate(TINY_SCENARIO, dtype_checking_policy, "test_policy", seeds=[0])
        assert all(dtype == np.float32 for dtype in received_dtypes)

    def test_policy_receives_observation_length_arrays(self):
        """Policy receives arrays of observation_len."""
        env = native.KsimEnv(str(TINY_SCENARIO))
        expected_len = env.observation_len
        received_lengths = []

        def length_checking_policy(obs: np.ndarray) -> np.ndarray:
            received_lengths.append(len(obs))
            return np.full(3, 1, dtype=np.uint32)

        evaluate(TINY_SCENARIO, length_checking_policy, "test_policy", seeds=[0])
        assert all(length == expected_len for length in received_lengths)


class TestEvaluateDeterminism:
    """Test determinism of evaluate."""

    def test_same_seeds_produce_identical_reports(self):
        """Same seeds produce identical reports."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        reports1 = evaluate(TINY_SCENARIO, policy, "test_policy", seeds=[42, 43])
        reports2 = evaluate(TINY_SCENARIO, policy, "test_policy", seeds=[42, 43])
        assert reports1 == reports2

    def test_different_seeds_produce_different_metrics(self):
        """Different seeds produce different metrics."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        reports = evaluate(TINY_SCENARIO, policy, "test_policy", seeds=[0, 1])
        # Metrics may differ between seeds
        metrics1 = reports[0]["metrics"]
        metrics2 = reports[1]["metrics"]
        # At least some metrics should differ
        assert not all(metrics1[k] == metrics2[k] for k in metrics1 if k in metrics2)


class TestEvaluateErrors:
    """Test error handling in evaluate."""

    def test_empty_seeds_raises_value_error(self):
        """Empty seeds list raises ValueError."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        with pytest.raises(ValueError):
            evaluate(TINY_SCENARIO, policy, "test_policy", seeds=[])

    def test_wrong_action_length_raises_value_error(self):
        """Policy returning wrong-length actions raises ValueError."""

        def bad_policy(obs: np.ndarray) -> np.ndarray:
            return np.array([1, 2], dtype=np.uint32)  # Wrong length

        with pytest.raises(ValueError):
            evaluate(TINY_SCENARIO, bad_policy, "test_policy", seeds=[0])


class TestEvaluateWithEnvOptions:
    """Test evaluate with EnvOptions."""

    def test_env_options_max_replica_step_affects_action_size(self):
        """EnvOptions(max_replica_step=1) changes the action size."""
        env_default = native.KsimEnv(str(TINY_SCENARIO))
        env_custom = native.KsimEnv(str(TINY_SCENARIO), max_replica_step=1)
        # The action size should be different
        assert len(env_default.action_nvec) != len(env_custom.action_nvec)

    def test_policy_with_custom_options(self):
        """evaluate works with custom EnvOptions."""
        options = EnvOptions(max_replica_step=1)
        env = native.KsimEnv(str(TINY_SCENARIO), max_replica_step=1)
        action_len = len(env.action_nvec)

        def size_aware_policy(obs: np.ndarray) -> np.ndarray:
            return np.full(action_len, 1, dtype=np.uint32)

        reports = evaluate(
            TINY_SCENARIO, size_aware_policy, "test_policy", seeds=[0], options=options
        )
        assert len(reports) == 1


class TestWriteReports:
    """Test write_reports function."""

    def test_write_reports_returns_path(self, tmp_path):
        """write_reports returns the file path."""
        reports = [
            {
                "scenario": "tiny",
                "policy": "test_policy",
                "seed": 0,
                "metrics": {"cost": 1.0, "slo_violations": 0.5},
            }
        ]
        result = write_reports(reports, tmp_path)
        assert isinstance(result, Path)

    def test_write_reports_creates_file(self, tmp_path):
        """write_reports creates a file."""
        reports = [
            {
                "scenario": "tiny",
                "policy": "test_policy",
                "seed": 0,
                "metrics": {"cost": 1.0, "slo_violations": 0.5},
            }
        ]
        result = write_reports(reports, tmp_path)
        assert result.exists()

    def test_write_reports_filename_format(self, tmp_path):
        """write_reports creates file named <scenario>-<policy>.jsonl."""
        reports = [
            {
                "scenario": "tiny",
                "policy": "test_policy",
                "seed": 0,
                "metrics": {"cost": 1.0},
            }
        ]
        result = write_reports(reports, tmp_path)
        assert result.name == "tiny-test_policy.jsonl"

    def test_write_reports_creates_nested_dir(self, tmp_path):
        """write_reports creates output directory if it does not exist."""
        reports = [
            {
                "scenario": "tiny",
                "policy": "test_policy",
                "seed": 0,
                "metrics": {"cost": 1.0},
            }
        ]
        out_path = tmp_path / "nested" / "output"
        result = write_reports(reports, out_path)
        assert out_path.exists()
        assert result.parent == out_path

    def test_write_reports_one_line_per_report(self, tmp_path):
        """write_reports writes one JSON line per report."""
        reports = [
            {
                "scenario": "tiny",
                "policy": "test_policy",
                "seed": i,
                "metrics": {"cost": float(i), "slo": float(i + 0.5)},
            }
            for i in range(3)
        ]
        result = write_reports(reports, tmp_path)
        lines = result.read_text().splitlines()
        assert len(lines) == 3

    def test_write_reports_json_format(self, tmp_path):
        """write_reports writes valid JSON lines."""
        reports = [
            {
                "scenario": "tiny",
                "policy": "test_policy",
                "seed": 0,
                "metrics": {"cost": 1.5},
            },
            {
                "scenario": "tiny",
                "policy": "test_policy",
                "seed": 1,
                "metrics": {"cost": 2.5},
            },
        ]
        result = write_reports(reports, tmp_path)
        lines = result.read_text().splitlines()
        for line in lines:
            obj = json.loads(line)
            assert "scenario" in obj
            assert "policy" in obj
            assert "seed" in obj
            assert "metrics" in obj

    def test_write_reports_empty_raises_value_error(self, tmp_path):
        """write_reports with empty reports raises ValueError."""
        with pytest.raises(ValueError):
            write_reports([], tmp_path)

    def test_write_reports_mixed_scenarios_raises_value_error(self, tmp_path):
        """write_reports with mixed scenarios raises ValueError."""
        reports = [
            {
                "scenario": "tiny",
                "policy": "test_policy",
                "seed": 0,
                "metrics": {"cost": 1.0},
            },
            {
                "scenario": "other",
                "policy": "test_policy",
                "seed": 1,
                "metrics": {"cost": 2.0},
            },
        ]
        with pytest.raises(ValueError):
            write_reports(reports, tmp_path)

    def test_write_reports_mixed_policies_raises_value_error(self, tmp_path):
        """write_reports with mixed policies raises ValueError."""
        reports = [
            {
                "scenario": "tiny",
                "policy": "test_policy_1",
                "seed": 0,
                "metrics": {"cost": 1.0},
            },
            {
                "scenario": "tiny",
                "policy": "test_policy_2",
                "seed": 1,
                "metrics": {"cost": 2.0},
            },
        ]
        with pytest.raises(ValueError):
            write_reports(reports, tmp_path)

    def test_write_reports_loadable_by_leaderboard(self, tmp_path):
        """Reports can be loaded by ksim.analysis.leaderboard.load_reports."""
        reports = [
            {
                "scenario": "tiny",
                "policy": "test_policy",
                "seed": i,
                "metrics": {"cost": float(i), "slo": float(i + 0.5)},
            }
            for i in range(2)
        ]
        write_reports(reports, tmp_path)
        loaded = load_reports(tmp_path)
        assert len(loaded) == 2
        assert loaded[0]["scenario"] == "tiny"
        assert loaded[0]["policy"] == "test_policy"


class TestEvaluateIntegration:
    """Integration tests for evaluate."""

    def test_evaluate_and_write_roundtrip(self, tmp_path):
        """evaluate -> write_reports -> load_reports roundtrip."""
        policy = lambda obs: np.full(3, 1, dtype=np.uint32)  # noqa: E731
        reports = evaluate(TINY_SCENARIO, policy, "test_policy", seeds=[0, 1])
        write_reports(reports, tmp_path)
        loaded = load_reports(tmp_path)
        assert len(loaded) == 2
        assert all(r["policy"] == "test_policy" for r in loaded)
        assert all(r["scenario"] == "tiny" for r in loaded)


@pytest.mark.skipif(
    not (REPO_ROOT / "target" / "release" / "ksim-bench").exists(),
    reason="ksim-bench not built",
)
class TestEvaluateParity:
    """Test parity with ksim-bench run."""

    def test_parity_with_ksim_bench_ca_policy(self, tmp_path):
        """Zero-delta policy metrics match ksim-bench run --policy ca."""

        # Define a zero-delta policy (keep replicas constant)
        def zero_delta_policy(obs: np.ndarray) -> np.ndarray:
            return np.full(3, 0, dtype=np.uint32)

        # Run evaluate
        reports = evaluate(TINY_SCENARIO, zero_delta_policy, "ca", seeds=[1])

        # Run ksim-bench
        bench_output = tmp_path / "bench_output"
        bench_output.mkdir()
        result = subprocess.run(
            [
                str(REPO_ROOT / "target" / "release" / "ksim-bench"),
                "run",
                "--scenario",
                str(TINY_SCENARIO),
                "--policy",
                "ca",
                "--seeds",
                "1",
                "--first-seed",
                "1",
                "--output",
                str(bench_output),
            ],
            capture_output=True,
            timeout=30,
            check=False,
        )
        assert result.returncode == 0, f"ksim-bench failed: {result.stderr.decode()}"

        # Load bench reports
        bench_reports = load_reports(bench_output)
        assert len(bench_reports) > 0

        # Compare metrics
        our_metrics = reports[0]["metrics"]
        bench_metrics = bench_reports[0]["metrics"]

        # Metrics should match
        for key in our_metrics:
            if key in bench_metrics:
                our_val = our_metrics[key]
                bench_val = bench_metrics[key]
                # Allow for floating point differences, but should be very close
                if not (np.isnan(our_val) and np.isnan(bench_val)):
                    assert our_val == bench_val or np.isclose(our_val, bench_val, rtol=0, atol=0), (
                        f"Metric {key}: {our_val} != {bench_val}"
                    )
