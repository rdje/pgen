#!/usr/bin/env bash
#
# run_json_corpus.sh — EXTERNAL-CORPUS.2 (PGEN-EXTERNAL-CORPUS-0002)
#
# Characterize PGEN's generated `json` parser against the recognized external
# JSONTestSuite corpus (Nicolas Seriot, "Parsing JSON is a Minefield", MIT).
# This is the EXTERNAL oracle half of the [[project_external_corpus_doctrine]]
# directive (the internal half is the stimuli generator / certificate-coverage).
#
# The corpus's OWN filename prefixes are the fix-independent oracle:
#   y_*  MUST be accepted by a conforming JSON parser
#   n_*  MUST be rejected
#   i_*  implementation-defined (either outcome is acceptable)
#
# This script does NOT assert a pass/fail conformance bar — `grammars/json.ebnf`
# is a deliberately SIMPLIFIED subset of JSON, so divergences are expected and
# are a CHARACTERIZATION, not a gate (see results/characterization.md and the
# EXTERNAL-CORPUS tree). It prints per-file outcomes (TSV) + a summary.
#
# Usage:
#   PGEN_PARSEABILITY_PROBE=/path/to/parseability_probe \
#     json_corpus_bundle/scripts/run_json_corpus.sh [--tsv OUT.tsv]
#
# Build the probe first (json compiled in, heavy parsers skipped):
#   cd rust && PGEN_SYSTEMVERILOG_PARSER_PATH=/nonexistent \
#     PGEN_VHDL_PARSER_PATH=/nonexistent \
#     PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH=/nonexistent \
#     PGEN_RTL_CONST_EXPR_PARSER_PATH=/nonexistent \
#     PGEN_RTL_FRONTEND_PARSER_PATH=/nonexistent \
#     PGEN_REGEX_PARSER_PATH=/nonexistent \
#     cargo build --features generated_parsers,ebnf_dual_run --bin parseability_probe
#   (the json parser must exist: make -C rust SHELL=/bin/bash focus_json)
set -euo pipefail

BUNDLE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORPUS_DIR="$BUNDLE_DIR/third_party/upstream/JSONTestSuite/test_parsing"
PROBE="${PGEN_PARSEABILITY_PROBE:-$BUNDLE_DIR/../rust/target/debug/parseability_probe}"
PER_FILE_TIMEOUT_S="${PGEN_JSON_CORPUS_TIMEOUT_S:-10}"
TSV=""
if [[ "${1:-}" == "--tsv" ]]; then TSV="${2:?--tsv needs a path}"; fi

[[ -x "$PROBE" ]] || { echo "error: parseability_probe not found/executable at '$PROBE' (set PGEN_PARSEABILITY_PROBE)" >&2; exit 2; }
[[ -d "$CORPUS_DIR" ]] || { echo "error: corpus dir missing: $CORPUS_DIR" >&2; exit 2; }

# Prefer GNU timeout; fall back to gtimeout; else run without a bound.
if command -v timeout >/dev/null 2>&1; then TO=(timeout "$PER_FILE_TIMEOUT_S")
elif command -v gtimeout >/dev/null 2>&1; then TO=(gtimeout "$PER_FILE_TIMEOUT_S")
else TO=(); echo "warning: no timeout binary; pathological deep-nesting files may hang" >&2; fi

declare -A n=()  # n[class_outcome]
total=0
rows=""
for f in "$CORPUS_DIR"/*.json; do
  base="$(basename "$f")"; cls="${base%%_*}"
  set +e
  "${TO[@]}" "$PROBE" --parse json "$f" >/dev/null 2>&1
  rc=$?
  set -e
  case "$rc" in
    0)                 outcome=accept ;;
    124)               outcome=timeout ;;
    13[0-9]|14[0-9]|1[5-9][0-9]) outcome=crash ;;
    *)                 outcome=reject ;;
  esac
  rows+="${base}	${cls}	${outcome}	${rc}"$'\n'
  n["${cls}_${outcome}"]=$(( ${n["${cls}_${outcome}"]:-0} + 1 ))
  total=$(( total + 1 ))
done

if [[ -n "$TSV" ]]; then printf '%s' "$rows" > "$TSV"; echo "wrote per-file TSV: $TSV"; fi

g(){ echo "${n["$1"]:-0}"; }
y_total=$(( $(g y_accept) + $(g y_reject) + $(g y_timeout) + $(g y_crash) ))
n_total=$(( $(g n_accept) + $(g n_reject) + $(g n_timeout) + $(g n_crash) ))
i_total=$(( $(g i_accept) + $(g i_reject) + $(g i_timeout) + $(g i_crash) ))

echo "=== PGEN json parser vs JSONTestSuite (total=$total files) ==="
printf 'y_ (MUST accept,  %3d): accept=%d reject=%d timeout=%d crash=%d\n' "$y_total" "$(g y_accept)" "$(g y_reject)" "$(g y_timeout)" "$(g y_crash)"
printf 'n_ (MUST reject,  %3d): accept=%d reject=%d timeout=%d crash=%d\n' "$n_total" "$(g n_accept)" "$(g n_reject)" "$(g n_timeout)" "$(g n_crash)"
printf 'i_ (impl-defined, %3d): accept=%d reject=%d timeout=%d crash=%d\n' "$i_total" "$(g i_accept)" "$(g i_reject)" "$(g i_timeout)" "$(g i_crash)"
echo "--- conformance vs recognized labels ---"
echo "  y_ correctly accepted: $(g y_accept) / $y_total"
echo "  n_ correctly rejected: $(g n_reject) / $n_total"
echo "  crashes (robustness):  $(( $(g y_crash) + $(g n_crash) + $(g i_crash) ))"
