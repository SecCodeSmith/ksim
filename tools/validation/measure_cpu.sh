#!/usr/bin/env bash
# Usage: measure_cpu.sh SAMPLES INTERVAL_S
# Prints the mean total CPU in millicores of the php-apache pods over SAMPLES metrics-server
# readings taken INTERVAL_S apart.
set -euo pipefail
samples=$1
interval=$2
total=0
for _ in $(seq "$samples"); do
  sleep "$interval"
  millis=$(kubectl top pod -l run=php-apache --no-headers | awk '{ sub("m", "", $2); sum += $2 } END { print sum + 0 }')
  total=$(( total + millis ))
done
echo $(( total / samples ))
