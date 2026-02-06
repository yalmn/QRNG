#!/usr/bin/env bash
set -euo pipefail

BASE_URL="https://lfdr.de/qrng_api/qrng?length=4096&format=BINARY"
OUT_PREFIX="qrng"
COUNT=100
SLEEP_SECONDS=10

command -v curl >/dev/null 2>&1 || { echo "curl fehlt"; exit 1; }
command -v jq   >/dev/null 2>&1 || { echo "jq fehlt"; exit 1; }

for i in $(seq 0 $((COUNT-1))); do
  fn="$(printf "%s_%02d" "$OUT_PREFIX" "$i")"
  echo "[$(date -Is)] Fetch -> $fn"

  json="$(curl -fsS --retry 3 --retry-delay 2 --connect-timeout 10 --max-time 60 "$BASE_URL")"

  bits="$(echo "$json" | jq -r '.qrn' | tr -d '[][:space:]')"

  # Basic sanity check: must be only 0/1 and non-empty
  if [[ -z "$bits" || ! "$bits" =~ ^[01]+$ ]]; then
    echo "Fehler: Ungültige qrn-Ausgabe für $fn"
    echo "Antwort war: $json" >&2
    exit 1
  fi
  
  printf "%s" "$bits" > "$fn"

  if [[ "$i" -lt $((COUNT-1)) ]]; then
    sleep "$SLEEP_SECONDS"
  fi
done

echo "Fertig: $COUNT Dateien geschrieben (${OUT_PREFIX}_00 .. ${OUT_PREFIX}_99)"

