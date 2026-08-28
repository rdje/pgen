#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.61(a) — how often does the new ceiling label fire on a
# REAL rejection population, measured on the graph that SHIPS?
#
# ⛔ This is the census the pre-change instrument could not take. Reading the counter
# needed `PGEN_REPORT_MEMO_STATS`, and `.56`(b) established that every observation
# channel is a disjunct of `bare_parse` — so observing the parse ROUTES IT OFF the
# fused graph. The label is emitted by the shipped parse itself, so this run needs no
# env var, measures the graph an embedder actually gets, and is ~35x faster (the
# instrumented run timed out on 762 of the first 2 179 files at 20 s each).
#
# ⛔ The probe exits 1 on a REJECTED parse and 0 on an accepted one, so a nonzero
# status is NOT a timeout — only gtimeout's own 124/137 is.
set -uo pipefail
PROBE="${PGEN_PARSE_PROBE_BIN:-./rust/target/release/parseability_probe}"
GRAMMAR="${PGEN_CENSUS_GRAMMAR:-systemverilog}"
LIST="${1:?usage: census_ceiling_label.sh <file-list> <out.tsv>}"
OUT="${2:?usage: census_ceiling_label.sh <file-list> <out.tsv>}"
: > "$OUT"
while IFS= read -r f; do
  [ -n "$f" ] || continue
  out=$(gtimeout 20s "$PROBE" --parse "$GRAMMAR" "$f" 2>&1)
  rc=$?
  if [ "$rc" -eq 124 ] || [ "$rc" -eq 137 ]; then
    printf 'TIMEOUT\tNA\t%s\n' "$f" >> "$OUT"; continue
  fi
  if printf '%s' "$out" | grep -q '^parse_full passed'; then v=ACCEPT; else v=REJECT; fi
  if printf '%s' "$out" | grep -q 'recursion ceiling CONTACTED'; then l=LABELLED; else l=plain; fi
  printf '%s\t%s\t%s\n' "$v" "$l" "$f" >> "$OUT"
done < "$LIST"
