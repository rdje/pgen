#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.65 — per-file verdict census on the SHIPPED (fused) graph.
#
# `.61`(b)'s tracked census sets PGEN_REPORT_MEMO_STATS, which is a disjunct of the
# `bare_parse` selector and therefore routes to the PROTOCOL graph. That is the right
# instrument for reading the guard's refusal counter and the wrong one for asking whether
# the artifact an embedder RUNS changed its mind about a file. This census sets nothing,
# so it measures the fused graph that ships — and it is run on BOTH binaries with the same
# harness, so the two sides differ only in the parser.
#
# Usage: verdict_census.sh <family> <profile-args...> -- <file-list> <out.tsv>
set -uo pipefail
PROBE="${PGEN_PARSE_PROBE_BIN:?set PGEN_PARSE_PROBE_BIN}"
FAM="$1"; shift
EXTRA=()
while [ "$1" != "--" ]; do EXTRA+=("$1"); shift; done
shift
LIST="$1"; OUT="$2"
: > "$OUT"
while IFS= read -r f; do
  [ -n "$f" ] || continue
  out=$(gtimeout 60s "$PROBE" --parse "$FAM" "$f" "${EXTRA[@]}" 2>&1)
  rc=$?
  if [ "$rc" -eq 124 ] || [ "$rc" -eq 137 ]; then printf 'TIMEOUT\t%s\n' "$f" >> "$OUT"; continue; fi
  if printf '%s' "$out" | grep -q '^parse_full passed'; then v=ACCEPT; else v=REJECT; fi
  printf '%s\t%s\n' "$v" "$f" >> "$OUT"
done < "$LIST"
