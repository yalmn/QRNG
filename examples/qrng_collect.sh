#!/usr/bin/env bash
set -euo pipefail

URL="https://lfdr.de/qrng_api/qrng?length=4096&format=BINARY"
ROUNDS=20
PREFIX="qrng_"

for i in $(seq 1 "$ROUNDS"); do
  out=$(printf "%s%02d.bin" "$PREFIX" "$i")
  echo "Download $i/$ROUNDS -> $out"

  # Download -> nur 0/1 behalten -> Datei schreiben
  curl -s "$URL" | LC_ALL=C tr -cd '01' > "$out"

  # Optional: kurz validieren (nur 0/1)
  # grep -q '[^01]' "$out" && { echo "WARN: Nicht 0/1 in $out"; exit 1; }
done

echo "Fertig: $ROUNDS Dateien (${PREFIX}*.bin)"

