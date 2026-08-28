#!/usr/bin/env bash
# tmp/eus67/measure_footprint.sh — pair the arena-leg footprint with peak RSS, per file.
# Usage: measure_footprint.sh <filelist> <out.tsv> [timeout_s]
set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LIST="$1"; OUT="$2"; TMO="${3:-120}"
PROBE="${PGEN_EUS67_PROBE:-./rust/target/release/parseability_probe}"
W="tmp/eus67/.fw.$$"; mkdir -p "$W"; trap 'rm -rf "$W"' EXIT
printf 'path\tbytes\tverdict\tpeak_rss_bytes\tarena_nodes\tarena_node_MB\tarena_total_MB\n' > "$OUT"
while IFS= read -r f; do
  [ -n "$f" ] || continue
  sz=$(wc -c < "$f" | tr -d ' ')
  PGEN_REPORT_PARSE_FOOTPRINT=1 /usr/bin/time -l timeout "$TMO" sh -c \
      'exec "$0" --parse systemverilog "$1" --profile sv_2017 >"$2" 2>"$3"' \
      "$PROBE" "$f" "$W/out" "$W/err" 2> "$W/time"
  rc=$?
  rss=$(awk '/maximum resident set size/{print $1; exit}' "$W/time")
  line=$(grep -m1 'PARSE FOOTPRINT (arena leg)' "$W/err" 2>/dev/null)
  nodes=$(printf '%s' "$line" | sed -n 's/.*nodes=\([0-9]*\).*/\1/p')
  nmb=$(printf '%s' "$line" | sed -n 's/.*nodes=[0-9]* (\([0-9.]*\) MB.*/\1/p')
  tmb=$(printf '%s' "$line" | sed -n 's/.*arena item total \([0-9.]*\) MB.*/\1/p')
  if [ "$rc" -eq 124 ]; then v=TIMEOUT
  elif grep -q 'parse_full passed' "$W/out" 2>/dev/null; then v=ACCEPT
  else v=REJECT; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$f" "$sz" "$v" "${rss:-NA}" "${nodes:-NA}" "${nmb:-NA}" "${tmb:-NA}" >> "$OUT"
done < "$LIST"
