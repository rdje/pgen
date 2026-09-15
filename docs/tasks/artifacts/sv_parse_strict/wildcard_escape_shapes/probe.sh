#!/usr/bin/env bash
# SV-PARSE-STRICT.4 — the unresolved-wildcard-import ESCAPE is narrower than the boundary it
# implements. Five declaration shapes, one variable: what follows the type name. `pkg` is declared
# NOWHERE, so under the 2026-06-09 director-confirmed boundary ("the parser rejects PROVABLY-
# undeclared type identifiers and accepts when it cannot prove") every one of them must ACCEPT.
# Four do. Routed by SV-CORPUS-GRAD.17 (PGEN-SV-CORPUS-GRAD-0395); this bank is the reproducer, and
# it FAILS today on purpose — it is the finding, not a gate. shape6 is the POSITIVE control: the same
# `T x = new();` with `T` DECLARED parses, so shape2's refusal is the escape's absence and not `new()`.
# Run: bash docs/tasks/artifacts/sv_parse_strict/wildcard_escape_shapes/probe.sh
set -u
cd "$(dirname "$0")/../../../../.." || exit 2
PROBE=${PGEN_PARSE_PROBE_BIN:-rust/target/release/parseability_probe}
HERE=docs/tasks/artifacts/sv_parse_strict/wildcard_escape_shapes
if [ ! -x "$PROBE" ]; then echo "NOT EVALUATED: $PROBE is not built"; exit 0; fi
for f in "$HERE"/shape*.sv; do
  if "$PROBE" --parse systemverilog "$f" --profile sv_2017 >/dev/null 2>&1; then v=PASS; else v=FAIL; fi
  printf '%-10s %s  %s\n' "$v" "$(basename "$f")" "$(sed -n '3p;4p' "$f" | tr -s ' ' | tr '\n' ' ')"
done
