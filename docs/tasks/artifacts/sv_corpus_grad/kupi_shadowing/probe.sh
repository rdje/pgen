#!/usr/bin/env bash
# probe.sh — SV-CORPUS-GRAD.13c.2x.9
#
# SEVEN carriers for `known_unscoped_property_identifier` — SystemVerilog's LAST union-residual
# `UNKNOWN`. Each one PARSES; not one COMMITS the rule. Re-runnable, so the claim "no carrier tested
# reaches it" is a measurement anyone can repeat rather than a summary of a session.
#
# ⛔ WHAT IT DOES NOT PROVE, stated first: seven negatives are not a proof of unreachability. This
# sizes and pins the behaviour; adjudicating it as PROVABLY shadowed is the owning leaf's work.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT"
PROBE="$ROOT/rust/target/release/parseability_probe"
[[ -x "$PROBE" ]] || { echo "error: build it: make -C rust SHELL=/bin/bash focus_systemverilog && cargo build --release --features generated_parsers --bin parseability_probe" >&2; exit 1; }
HERE="$(dirname "${BASH_SOURCE[0]}")"
OUT="$ROOT/rust/target/kupi_diag"; mkdir -p "$OUT"

printf '%-12s %-7s %s\n' carrier parses commits_known_unscoped_property_identifier
for f in "$HERE"/*.sv; do
    b="$(basename "$f" .sv)"
    p="$("$PROBE" --parse systemverilog "$f" --profile sv_2017 2>&1 | grep -c passed)"
    "$PROBE" --parse-dump-ast systemverilog "$f" "$OUT/$b.json" --profile sv_2017 >/dev/null 2>&1
    c="$(grep -c 'known_unscoped_property_identifier' "$OUT/$b.json" 2>/dev/null)"; c="${c:-0}"
    printf '%-12s %-7s %s\n' "$b" "$p" "$c"
done
