#!/usr/bin/env bash
# Prints the cluster state that explains a failed validation run: pod phases and restarts,
# Service endpoints, the HPA, recent events and the php-apache container's last state.
set -uo pipefail
kubectl get pods,endpoints,hpa -o wide
kubectl describe pods -l run=php-apache
kubectl get events --sort-by=.lastTimestamp | tail -n 50
