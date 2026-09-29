# Extending

Extension is done by implementing a trait; the engine is never edited (open/closed). Every walkthrough is test first. Status: trait set **Planned** (Milestones 2-4); the workflow applies from now on.

## Add an autoscaler

1. Write the failing tests: a behavior test with a hand-computed expectation, and the shared contract suite (`autoscaler_contract_tests!(MyAutoscaler)`). Run them and see them fail.
2. Implement `Autoscaler::act` minimally until green. Keep to the small traits you need (`ReplicaRecommender`, `ResourceRecommender`, `NodeProvisioner`).
3. Refactor with tests green; register in the policy factory of `ksim-bench`.
4. Document parameters and sources on [Baseline Autoscalers](Baseline-Autoscalers) or a new wiki page.

## Add a dataset

1. Tests: a tiny fixture file and a `trace_source_contract_tests!` run for the resulting `.ksimtrace`; a test for the normalizer's output shape and split boundaries.
2. Write the polars normalizer in `python/ksim/datasets` and, only if the format needs it, a `TraceSource` implementation.
3. Add the section to [Datasets](Datasets) (source, files, columns, license, splits).

## Add a metric

1. Test with a hand-computed example from the cited paper.
2. Implement `Metric`; add to the report.
3. Add definition, formula and citation to [Benchmark Metrics](Benchmark-Metrics).

## Add a performance model or reward

Implement `PerformanceModel` or `RewardFn` against the same contract suites, then select it through `SimulationBuilder`.
