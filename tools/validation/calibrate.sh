#!/usr/bin/env bash
# Usage: calibrate.sh URL RATE OUT.toml
# Measures php-apache CPU per request at a constant RATE with the HPA absent (2 fixed
# replicas, so one pod's CPU limit cannot cap the measurement), then writes the calibrated
# validation scenario.
set -euo pipefail
url=$1
rate=$2
out=$3
kubectl scale deployment php-apache --replicas=2
kubectl rollout status deployment/php-apache --timeout=180s
k6 run --quiet -e URL="$url" -e RATE="$rate" -e DURATION=210s tools/validation/constant.js &
load=$!
sleep 90
millis=$(tools/validation/measure_cpu.sh 6 15)
wait "$load"
echo "measured ${millis}m CPU at ${rate} requests/s"
mkdir -p "$(dirname "$out")"
uv run --no-sync python -m ksim.validation calibrate \
  --scenario scenarios/validation_php_apache.toml \
  --cpu-millicores "$millis" --rps "$rate" --out "$out"
kubectl scale deployment php-apache --replicas=1
kubectl rollout status deployment/php-apache --timeout=180s
