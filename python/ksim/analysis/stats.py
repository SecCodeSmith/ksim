"""Uncertainty and significance for per-seed benchmark metrics."""

from collections.abc import Sequence
from dataclasses import dataclass

import numpy as np
from scipy import stats


@dataclass(frozen=True)
class Interval:
    mean: float
    low: float
    high: float


def bootstrap_ci(
    values: Sequence[float],
    confidence: float = 0.95,
    resamples: int = 10_000,
    seed: int = 0,
) -> Interval:
    """Percentile bootstrap of the mean (Efron & Tibshirani, 1993) with a seeded NumPy
    generator. A single value yields a degenerate interval; an empty sequence raises
    ValueError."""
    if len(values) == 0:
        raise ValueError("bootstrap_ci needs at least one value")
    data = np.asarray(values, dtype=float)
    mean = float(data.mean())
    if len(data) == 1:
        return Interval(mean=mean, low=mean, high=mean)
    rng = np.random.default_rng(seed)
    picks = rng.integers(0, len(data), size=(resamples, len(data)))
    means = data[picks].mean(axis=1)
    tail = (1.0 - confidence) / 2.0 * 100.0
    low, high = np.percentile(means, [tail, 100.0 - tail])
    return Interval(mean=mean, low=float(low), high=float(high))


def paired_wilcoxon(candidate: Sequence[float], baseline: Sequence[float]) -> float:
    """Two-sided p-value of the Wilcoxon signed-rank test on paired per-seed values
    (scipy.stats.wilcoxon, zero_method "wilcox"); 1.0 when every difference is zero.
    Sequences of different length raise ValueError."""
    if len(candidate) != len(baseline):
        raise ValueError("paired samples must have the same length")
    differences = np.asarray(candidate, dtype=float) - np.asarray(baseline, dtype=float)
    if not differences.any():
        return 1.0
    return float(stats.wilcoxon(differences, zero_method="wilcox").pvalue)
