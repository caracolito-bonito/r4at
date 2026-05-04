#!/usr/bin/env bash
set -u

HOST="${1:-127.0.0.1}"
PORT="${2:-6969}"
CLIENTS="${3:-20}"
MESSAGES="${4:-20}"

echo "Starting local chat load test: $CLIENTS clients, $MESSAGES messages each"

for i in $(seq 1 "$CLIENTS"); do
  (
    for j in $(seq 1 "$MESSAGES"); do
      printf "client=%s message=%s\n" "$i" "$j"
      sleep 0.02
    done
  ) | nc "$HOST" "$PORT" &
done

wait
echo "Done"
