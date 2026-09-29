# Benchmark protocol

Status: **Planned** (Milestone 4 for the harness, Milestone 5 for dataset scenarios).

## Scenarios

Per dataset a fixed suite defined in `scenarios/*.toml`: steady, diurnal, burst / flash crowd, mixed batch + online, node failure. Evaluation uses held-out test windows only (see [Datasets](Datasets)).

## Seeds and statistics

- At least 10 seeds per scenario and policy; the seed list is fixed in the scenario file.
- Report the mean with a 95 % percentile bootstrap confidence interval: resample the per-seed scores with replacement B times (B >= 10,000), take the 2.5th and 97.5th percentiles of the resampled means.
- Compare a policy to the default HPA+VPA+CA baseline with the paired Wilcoxon signed-rank test over per-seed (or per-scenario) score pairs, two-sided, alpha = 0.05, with multiple-comparison correction when several policies or metrics are tested (Holm).
- Normalized score: metric value divided by the baseline's value (baseline = 1.0), direction-aware so that higher is always better.

## Outputs

JSON and CSV per run, leaderboard as Markdown and plots (`python/ksim/analysis`, `scipy`, `matplotlib`).

## Running

```sh
ksim-bench run scenarios/alibaba2018_diurnal.toml --policy hpa
ksim-bench run scenarios/alibaba2018_diurnal.toml --policy python:module:Policy
```

## Submitting a result

Open a pull request adding the result JSON, the exact commit hash of ksim, the policy code or a link, and the seed list. Results must be reproducible from those inputs.
