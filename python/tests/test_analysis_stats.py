"""Tests for ksim.analysis.stats: bootstrap CI and Wilcoxon test."""

import pytest
from ksim.analysis.stats import Interval, bootstrap_ci, paired_wilcoxon
from scipy import stats


class TestBootstrapCI:
    """bootstrap_ci: percentile bootstrap of the mean (Efron & Tibshirani, 1993)."""

    def test_basic_interval_contains_mean(self):
        """Interval mean equals input mean; low < mean < high within data bounds."""
        values = [1.0, 2.0, 3.0, 4.0, 5.0]
        interval = bootstrap_ci(values, seed=42)
        assert interval.mean == pytest.approx(3.0)
        assert interval.low < interval.mean < interval.high
        assert interval.low >= 1.0
        assert interval.high <= 5.0

    def test_deterministic_with_seed(self):
        """Same seed produces identical interval."""
        values = [1.0, 2.0, 3.0, 4.0, 5.0]
        interval1 = bootstrap_ci(values, seed=42)
        interval2 = bootstrap_ci(values, seed=42)
        assert interval1 == interval2

    def test_different_seeds_may_differ(self):
        """Different seeds may produce different intervals (non-deterministic behavior)."""
        values = [1.0, 2.0, 3.0, 4.0, 5.0]
        interval1 = bootstrap_ci(values, seed=1)
        interval2 = bootstrap_ci(values, seed=2)
        # Intervals may differ but means should be the same
        assert interval1.mean == interval2.mean

    def test_single_value_degenerate_interval(self):
        """Single value yields degenerate interval (7, 7, 7)."""
        interval = bootstrap_ci([7.0])
        assert interval == Interval(mean=7.0, low=7.0, high=7.0)

    def test_empty_sequence_raises_valueerror(self):
        """Empty sequence raises ValueError."""
        with pytest.raises(ValueError):
            bootstrap_ci([])

    def test_confidence_affects_width(self):
        """Higher confidence (e.g., 0.99) gives wider interval than lower (e.g., 0.90)."""
        values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
        interval_90 = bootstrap_ci(values, confidence=0.90, seed=42)
        interval_99 = bootstrap_ci(values, confidence=0.99, seed=42)
        width_90 = interval_90.high - interval_90.low
        width_99 = interval_99.high - interval_99.low
        assert width_99 > width_90

    def test_interval_properties_for_negatives(self):
        """Bootstrap CI works for negative values."""
        values = [-5.0, -3.0, -1.0, 2.0, 4.0]
        interval = bootstrap_ci(values)
        assert interval.mean == pytest.approx(-0.6)
        assert interval.low <= interval.mean <= interval.high

    def test_default_confidence_is_0_95(self):
        """Default confidence parameter is 0.95."""
        values = [1.0, 2.0, 3.0, 4.0, 5.0]
        interval_default = bootstrap_ci(values, seed=42)
        interval_explicit = bootstrap_ci(values, confidence=0.95, seed=42)
        assert interval_default == interval_explicit


class TestPairedWilcoxon:
    """paired_wilcoxon: two-sided p-value of Wilcoxon signed-rank test."""

    def test_equals_scipy_wilcoxon(self):
        """Result matches scipy.stats.wilcoxon with zero_method='wilcox'."""
        candidate = [1.0, 2.0, 3.0, 4.0, 5.0]
        baseline = [1.0, 1.5, 2.5, 3.5, 4.5]
        p_value = paired_wilcoxon(candidate, baseline)
        scipy_result = stats.wilcoxon(candidate, baseline, zero_method="wilcox")
        assert p_value == pytest.approx(scipy_result.pvalue)

    def test_all_equal_returns_1_0(self):
        """When all differences are zero, p-value is 1.0."""
        candidate = [5.0, 5.0, 5.0, 5.0]
        baseline = [5.0, 5.0, 5.0, 5.0]
        p_value = paired_wilcoxon(candidate, baseline)
        assert p_value == pytest.approx(1.0)

    def test_length_mismatch_raises_valueerror(self):
        """Sequences of different length raise ValueError."""
        candidate = [1.0, 2.0, 3.0]
        baseline = [1.0, 2.0]
        with pytest.raises(ValueError):
            paired_wilcoxon(candidate, baseline)

    def test_symmetric_differences(self):
        """Symmetric differences (e.g., ±1) from baseline."""
        candidate = [4.0, 5.0, 6.0, 7.0, 8.0]
        baseline = [5.0, 5.0, 5.0, 5.0, 5.0]
        p_value = paired_wilcoxon(candidate, baseline)
        scipy_result = stats.wilcoxon(candidate, baseline, zero_method="wilcox")
        assert p_value == pytest.approx(scipy_result.pvalue)

    def test_all_positive_differences(self):
        """All differences positive; Wilcoxon is more extreme."""
        candidate = [2.0, 3.0, 4.0, 5.0, 6.0]
        baseline = [1.0, 1.0, 1.0, 1.0, 1.0]
        p_value = paired_wilcoxon(candidate, baseline)
        scipy_result = stats.wilcoxon(candidate, baseline, zero_method="wilcox")
        assert p_value == pytest.approx(scipy_result.pvalue)

    def test_single_pair(self):
        """Single pair of values."""
        p_value = paired_wilcoxon([2.0], [1.0])
        scipy_result = stats.wilcoxon([2.0], [1.0], zero_method="wilcox")
        assert p_value == pytest.approx(scipy_result.pvalue)
