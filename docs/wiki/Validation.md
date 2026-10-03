# Validation

Status: **Implemented** (Milestone 7). The simulated Horizontal Pod Autoscaler is compared against a real Kubernetes cluster serving the same load, by `.github/workflows/validation.yml` (manual dispatch, weekly, and on changes to the validation files).

## Method

1. **Cluster.** A single-node [kind](https://kind.sigs.k8s.io/) cluster with metrics-server runs the upstream HPA walkthrough app `registry.k8s.io/hpa-example` (php-apache, which burns CPU on every request) with the walkthrough settings: requests 200m CPU, limits 500m, HPA `autoscaling/v2` at 50 % CPU utilisation, 1 to 10 replicas (`deploy/validation/`). The simulator scenario `scenarios/validation_php_apache.toml` mirrors those values.
2. **Calibration.** `tools/validation/calibrate.sh` fixes the Deployment at 2 replicas (no HPA), offers a constant 8 requests/s with k6, averages the metrics-server CPU of the pods over the last 90 s, and rewrites the scenario's `cpu_millis_per_request` with CPU / rate (`python -m ksim.validation calibrate`). It is the only fitted parameter.
3. **Simulation.** `ksim-bench trajectory --policy hpa --seed 0` writes the per-tick demand, desired and ready replicas, p95 latency and error rate of the calibrated scenario.
4. **Replay.** `python -m ksim.validation k6-script` turns the simulated demand into a k6 `ramping-arrival-rate` script (one 15 s stage per tick). k6 sends it through a NodePort, so kube-proxy spreads requests across all replicas (`kubectl port-forward` would pin them to one pod). `tools/validation/record.sh` samples the HPA's desired and the Deployment's ready replicas every 15 s.
5. **Comparison.** `python -m ksim.validation compare` aligns both sides per tick and reports ready-replica MAE, RMSE and maximum error, desired-replica MAE, the scale-up lag (observed minus simulated time of the first scale-up), the median relative error of the p95 latency and the error-rate MAE. The report is the job summary; the raw CSVs, the k6 script and the calibrated scenario are uploaded as the `validation` artifact.

The demand is a flash crowd (2 requests/s, a jump to 18 at 180 s, exponential decay with a 420 s time constant, 25 minutes), so one run exercises the scale-up path and the 300 s scale-down stabilisation window.

## Results

The job reports the numbers of every run in its summary and artifact. There are no pass/fail thresholds yet: they will be set from the spread of the first runs, so that the workflow can then gate regressions in the model.

### First complete run (2026-09-30)

[Run 36771845206](https://github.com/SecCodeSmith/ksim/actions/runs/36771845206), commit `8c8a8b2`, 100 ticks of 15 s.

| Metric | Value |
|---|---|
| Ready replicas: MAE / RMSE / max error | 2.55 / 3.49 / 6 |
| Desired replicas: MAE | 1.92 |
| Scale-up lag (real minus simulated) | 165 s |
| p95 latency: median relative error | 0.09 |
| Error rate: MAE | 0.01 |

Calibration measured 516m CPU at 8 requests/s, so `cpu_millis_per_request = 64.5`. An earlier run on another runner measured 387m (48 ms per request), so the calibrated cost varies by about 30 % between shared runners. That is why each run calibrates before it simulates.

What the numbers say:

- **Scale-up timing is the main gap.** The simulated HPA reacts to the flash crowd 165 s before the real one, and most of the replica error comes from that lag rather than from the replica count it settles on (desired MAE 1.92 over a 1 to 10 range).
- **Latency agrees where both sides serve traffic normally.** The median per-tick p95 error is 9 %.
- **The real cluster saturated during the lag.** k6 sent 9,319 requests with none failing, but reached its 200 virtual-user cap at the peak and dropped 198 iterations. Its overall p95 was 16.6 s while requests queued on too few replicas. The observed offered load therefore dipped slightly below the simulated demand at the peak.

Candidate causes of the lag, not yet tested: the HPA's CPU initialization period and the readiness delay of new pods, the metrics-server scrape window together with the HPA sync period (the simulator uses a one-scrape lag on the same 15 s grid), and container start on a loaded node. Isolating them, and setting thresholds from several runs, is follow-up work.

## Known limitations

- **Single app, single node.** One CPU-bound service on one kind node; the Cluster Autoscaler, VPA, scheduling pressure and multi-service interference are not validated here.
- **Shared runner.** k6, the kind node and metrics-server share the runner's CPUs, so measured latency includes contention the simulator does not model.
- **Latency model.** The simulator's queueing model assumes Poisson arrivals and a fitted service time; Apache's worker pool, keep-alive and kernel scheduling are not modelled, so latency is expected to match in trend rather than in absolute value.
- **Metrics pipeline.** Metrics Server scrapes pods every 15 s and the HPA reads the latest window; the simulator uses a one-scrape lag on the same 15 s grid, so replica changes can differ by one tick.
- **Pod startup.** Image pull is excluded after the first pod (the image is cached on the node); the simulator's default startup latency is used as is.
