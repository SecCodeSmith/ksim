"""Uncertainty and significance for per-seed benchmark metrics."""

from collections.abc import Sequence
from dataclasses import dataclass


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
    raise NotImplementedError


def paired_wilcoxon(candidate: Sequence[float], baseline: Sequence[float]) -> float:
    """Two-sided p-value of the Wilcoxon signed-rank test on paired per-seed values
    (scipy.stats.wilcoxon, zero_method "wilcox"); 1.0 when every difference is zero.
    Sequences of different length raise ValueError."""
    raise NotImplementedError
