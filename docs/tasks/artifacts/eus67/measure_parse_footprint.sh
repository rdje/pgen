#!/usr/bin/env bash
# docs/tasks/artifacts/eus67/measure_parse_footprint.sh — pair the parse footprint with
# peak RSS, per file. ENGINE-UNIVERSAL-SERVICES.67(b).
#
# The report line was renamed `(arena leg)` -> `(arena + AST-owned heap)` when the
# out-of-arena child-`Vec`/`String` legs became MEASURED rather than declared, so the
# match below anchors on `PARSE FOOTPRINT` alone and is stable across that rename.
# Usage: measure_footprint.sh <filelist> <out.tsv> [timeout_s]
set -uo pipefail
# ⛔ FOUR levels, not two. This producer was authored at `tmp/eus67/` and COPIED here
# without re-deriving the walk, so the tracked copy `cd`-ed to `docs/tasks` and could not
# open its own inputs or output — "tracked and re-runnable" was true of the first word only.
# Its sibling `measure_parse_rss.sh` had it right all along, which is why they are now
# spelled identically.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"
LIST="$1"; OUT="$2"; TMO="${3:-120}"
PROBE="${PGEN_EUS67_PROBE:-./rust/target/release/parseability_probe}"
# A missing probe must REFUSE, not produce a TSV of NA rows that reads like a measurement.
[ -x "$PROBE" ] || { echo "no probe at $PROBE" >&2; exit 2; }
[ -r "$LIST" ]  || { echo "no file list at $LIST" >&2; exit 2; }
W="tmp/eus67b/.fw.$$"; mkdir -p "$W"; trap 'rm -rf "$W"' EXIT
printf 'path\tbytes\tverdict\tpeak_rss_bytes\tarena_nodes\tarena_node_MB\tarena_total_MB'   >  "$OUT"
printf '\tchild_vecs\tchild_slots\tchild_MB\ttransformed_strings\ttransformed_MB\trendered_MB\towned_MB\tarena_owned_MB\n' >> "$OUT"
while IFS= read -r f; do
  [ -n "$f" ] || continue
  sz=$(wc -c < "$f" | tr -d ' ')
  PGEN_REPORT_PARSE_FOOTPRINT=1 /usr/bin/time -l timeout "$TMO" sh -c \
      'exec "$0" --parse systemverilog "$1" --profile sv_2017 >"$2" 2>"$3"' \
      "$PROBE" "$f" "$W/out" "$W/err" 2> "$W/time"
  rc=$?
  rss=$(awk '/maximum resident set size/{print $1; exit}' "$W/time")
  line=$(grep -m1 'PARSE FOOTPRINT' "$W/err" 2>/dev/null)
  nodes=$(printf '%s' "$line" | sed -n 's/.*nodes=\([0-9]*\).*/\1/p')
  nmb=$(printf '%s' "$line" | sed -n 's/.*nodes=[0-9]* (\([0-9.]*\) MB.*/\1/p')
  tmb=$(printf '%s' "$line" | sed -n 's/.*arena item total \([0-9.]*\) MB.*/\1/p')
  # ENGINE-UNIVERSAL-SERVICES.67(b) — the out-of-arena AST-owned legs. Any field that
  # does not match stays NA rather than 0: an unmatched field is a MISSING measurement,
  # and writing it as 0 is the exact confusion the instrument's `Option` guard prevents.
  cvec=$(printf '%s' "$line" | sed -n 's/.*child_vecs=\([0-9]*\).*/\1/p')
  cslot=$(printf '%s' "$line" | sed -n 's/.*child_vecs=[0-9]* (\([0-9]*\) slots.*/\1/p')
  cmb=$(printf '%s' "$line" | sed -n 's/.*child_vecs=[0-9]* ([0-9]* slots, \([0-9.]*\) MB.*/\1/p')
  xstr=$(printf '%s' "$line" | sed -n 's/.*transformed_strings=\([0-9]*\).*/\1/p')
  xmb=$(printf '%s' "$line" | sed -n 's/.*transformed_strings=[0-9]* (\([0-9.]*\) MB.*/\1/p')
  rmb=$(printf '%s' "$line" | sed -n 's/.*rendered_payload \([0-9.]*\) MB.*/\1/p')
  omb=$(printf '%s' "$line" | sed -n 's/.*AST-owned heap total \([0-9.]*\) MB.*/\1/p')
  aomb=$(printf '%s' "$line" | sed -n 's/.*arena+owned total \([0-9.]*\) MB.*/\1/p')
  if [ "$rc" -eq 124 ]; then v=TIMEOUT
  elif grep -q 'parse_full passed' "$W/out" 2>/dev/null; then v=ACCEPT
  else v=REJECT; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s' "$f" "$sz" "$v" "${rss:-NA}" "${nodes:-NA}" "${nmb:-NA}" "${tmb:-NA}" >> "$OUT"
  printf '\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "${cvec:-NA}" "${cslot:-NA}" "${cmb:-NA}" \
      "${xstr:-NA}" "${xmb:-NA}" "${rmb:-NA}" "${omb:-NA}" "${aomb:-NA}" >> "$OUT"
done < "$LIST"
