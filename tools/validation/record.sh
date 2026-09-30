#!/usr/bin/env bash
# Usage: record.sh OUT.csv DURATION_S [INTERVAL_S]
# Samples the php-apache HPA's desired replicas and the Deployment's ready replicas.
set -euo pipefail
out=$1
duration=$2
interval=${3:-15}
echo "epoch_s,desired,ready" > "$out"
end=$(( $(date +%s) + duration ))
while [ "$(date +%s)" -lt "$end" ]; do
  desired=$(kubectl get hpa php-apache -o jsonpath='{.status.desiredReplicas}')
  ready=$(kubectl get deployment php-apache -o jsonpath='{.status.readyReplicas}')
  echo "$(date +%s),${desired:-0},${ready:-0}" >> "$out"
  sleep "$interval"
done
