#!/usr/bin/env bash
# Usage: replay.sh URL SCENARIO.toml OUT_DIR
# Simulates the scenario with the HPA policy, replays its demand against the cluster with the
# real HPA, records replicas and request timings, and writes OUT_DIR/report.md.
set -euo pipefail
url=$1
scenario=$2
out=$3
mkdir -p "$out"
./target/release/ksim-bench trajectory --scenario "$scenario" --policy hpa --seed 0 \
  --out "$out/sim.csv"
uv run --no-sync python -m ksim.validation k6-script --trajectory "$out/sim.csv" \
  --service php-apache --url "$url" --out "$out/replay.js"
kubectl apply -f deploy/validation/hpa.yaml
sleep 60
duration=$(( $(tail -n +2 "$out/sim.csv" | wc -l) * 15 + 30 ))
start=$(date +%s)
tools/validation/record.sh "$out/real.csv" "$duration" &
recorder=$!
k6 run --quiet --out "csv=$out/k6.csv" "$out/replay.js"
wait "$recorder"
uv run --no-sync python -m ksim.validation compare --simulated "$out/sim.csv" \
  --service php-apache --recorder "$out/real.csv" --k6 "$out/k6.csv" \
  --start-epoch "$start" --out "$out/report.md"
