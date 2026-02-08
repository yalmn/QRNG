#!/usr/bin/env bash

ANALYSER="./analyse_"
DATA_DIR="set_of_4096_bytes_qrng"

sum_B=0
sum_M=0
count=0

for i in $(seq -w 0 99); do
  file="$DATA_DIR/qrng_$i"

  output=$($ANALYSER "$file")

  B=$(echo "$output" | awk '/Burstiness/ {print $3}')
  M=$(echo "$output" | awk '/Memory/ {print $3}')

  sum_B=$(awk -v s="$sum_B" -v b="$B" 'BEGIN {print s + b}')
  sum_M=$(awk -v s="$sum_M" -v m="$M" 'BEGIN {print s + m}')

  count=$((count + 1))
done

avg_B=$(awk -v s="$sum_B" -v c="$count" 'BEGIN {print s / c}')
avg_M=$(awk -v s="$sum_M" -v c="$count" 'BEGIN {print s / c}')

echo "Anzahl Dateien: $count"
echo "Durchschnitt Burstiness (B): $avg_B"
echo "Durchschnitt Memory (M):     $avg_M"
