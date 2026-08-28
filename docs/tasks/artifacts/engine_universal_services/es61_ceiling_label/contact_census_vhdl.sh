#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.61(b) — the same ceiling-contact census on the OTHER released
# family. The 4096-frame ceiling is an ENGINE constant, so a ruling on its location that
# rests on one family's corpus is a ruling about that family, not about the engine.
set -uo pipefail
PROBE="${PGEN_PARSE_PROBE_BIN:-./rust/target/release/parseability_probe}"
LIST="${1:?}"; OUT="${2:?}"
: > "$OUT"
while IFS= read -r f; do
  [ -n "$f" ] || continue
  out=$(gtimeout 60s env PGEN_REPORT_MEMO_STATS=1 "$PROBE" --parse vhdl "$f" 2>&1)
  rc=$?
  if [ "$rc" -eq 124 ] || [ "$rc" -eq 137 ]; then printf 'TIMEOUT\tNA\t%s\n' "$f" >> "$OUT"; continue; fi
  if printf '%s' "$out" | grep -q '^parse_full passed'; then v=ACCEPT; else v=REJECT; fi
  d=$(printf '%s' "$out" | grep -o '[0-9]* depth-ceiling rejections' | grep -o '^[0-9]*')
  printf '%s\t%s\t%s\n' "$v" "${d:-NA}" "$f" >> "$OUT"
done < "$LIST"
