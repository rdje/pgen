#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.61(a) — the same false-positive census as
# `census_ceiling_label.sh`, run under the EXPLICIT `sv_2017` profile.
#
# ⛔ Why both: SystemVerilog declares NO `@default_profile`, so an unnamed profile selects
# the UNION of every dialect rather than a real one. The tracked characterization was taken
# under `--profile sv_2017`; the first census here was not, and the two disagree on 28 files
# and 6 timeouts for that reason alone (routed as `.62`). The label claim must hold on the
# profile a consumer actually ships, so it is measured on both.
set -uo pipefail
PROBE="${PGEN_PARSE_PROBE_BIN:-./rust/target/release/parseability_probe}"
LIST="${1:?usage: <file-list> <out.tsv>}"
OUT="${2:?usage: <file-list> <out.tsv>}"
: > "$OUT"
while IFS= read -r f; do
  [ -n "$f" ] || continue
  out=$(gtimeout 60s "$PROBE" --parse systemverilog "$f" --profile sv_2017 2>&1)
  rc=$?
  if [ "$rc" -eq 124 ] || [ "$rc" -eq 137 ]; then
    printf 'TIMEOUT\tNA\t%s\n' "$f" >> "$OUT"; continue
  fi
  if printf '%s' "$out" | grep -q '^parse_full passed'; then v=ACCEPT; else v=REJECT; fi
  if printf '%s' "$out" | grep -q 'recursion ceiling CONTACTED'; then l=LABELLED; else l=plain; fi
  printf '%s\t%s\t%s\n' "$v" "$l" "$f" >> "$OUT"
done < "$LIST"
