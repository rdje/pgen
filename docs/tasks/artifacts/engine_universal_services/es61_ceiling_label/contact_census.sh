#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.61(b) — does ANY real SystemVerilog file CONTACT the
# 4096-frame recursion ceiling, on the ACCEPT path as well as the reject path?
#
# ⛔ `.61`(a)'s label answers only half of this: it is composed on the failure path, so
# "0 of 16 336 rejections labelled" does NOT bound contact on accepts — depth 322 ACCEPTS
# while recording 847 refusals. The guard's own counter does bound it, and reading it needs
# `PGEN_REPORT_MEMO_STATS`.
#
# ⚠️ HONEST BOUND, stated because `.56`(b) established it: that env var is a disjunct of the
# `bare_parse` selector, so this runs the PROTOCOL graph, not the fused one that ships. The
# two are different graphs and their refusal counts differ (measured on sv_323.sv: fused 953,
# protocol 1013). This census therefore bounds contact ON THE PROTOCOL GRAPH.
#
# ⛔ Under the UNNAMED profile this same census timed out on 762 of the first 2 179 files
# (`.62`); under `sv_2017` the whole corpus runs in ~8 minutes.
set -uo pipefail
PROBE="${PGEN_PARSE_PROBE_BIN:-./rust/target/release/parseability_probe}"
LIST="${1:?usage: <file-list> <out.tsv>}"; OUT="${2:?usage: <file-list> <out.tsv>}"
: > "$OUT"
while IFS= read -r f; do
  [ -n "$f" ] || continue
  out=$(gtimeout 60s env PGEN_REPORT_MEMO_STATS=1 "$PROBE" --parse systemverilog "$f" --profile sv_2017 2>&1)
  rc=$?
  if [ "$rc" -eq 124 ] || [ "$rc" -eq 137 ]; then printf 'TIMEOUT\tNA\t%s\n' "$f" >> "$OUT"; continue; fi
  if printf '%s' "$out" | grep -q '^parse_full passed'; then v=ACCEPT; else v=REJECT; fi
  d=$(printf '%s' "$out" | grep -o '[0-9]* depth-ceiling rejections' | grep -o '^[0-9]*')
  printf '%s\t%s\t%s\n' "$v" "${d:-NA}" "$f" >> "$OUT"
done < "$LIST"
