#!/usr/bin/env bash
# stimuli/run_pipeline_lane.sh <suite> [<suite>...]     — the SystemVerilog PREPROCESS→PARSE lane
#
# SVPP-EXPANSION.4 — the pipeline twin of `stimuli/run_external_corpus.sh sv`. The raw lane parses
# every corpus file ALONE and cannot judge a macro-dependent one; this lane runs each file through
# `parseability_probe --parse systemverilog FILE --profile sv_2017 --preprocess <context>`, where the
# context (include search directories, predefined macros) is the suite's row set in the tracked
# `stimuli/sv/pipeline_context.tsv` (derived by `stimuli/sv/pipeline_context.py`).
#
# Doctrine (characterize, don't game — as the raw lane): raw outcomes only, per file. Five statuses:
#   pass     rc 0    the expanded text parses
#   fail     rc 1    the expanded text is rejected — column 6 = furthest_position, column 7 = origin
#   pp_fail  rc 3    the PREPROCESSOR refused the input (unresolved include, unterminated
#                    conditional, an illegal define …) — the parser never ran; column 7 = the reason
#   timeout  rc 124  the per-file deadline expired
#   crash    rc>=128 signal death
#
# The per-file rows of `stimuli/sv/pipeline_context_files.tsv` (SVPP-EXPANSION.4b) are applied to their
# file FIRST, ahead of the suite's directories, so a name that exists under several `top_*` variants
# resolves to the includer's own top.
#
# Output (repo-root-relative, sorted, deterministic; the dump is .gitignored like the raw lane's —
# the tracked claim is `pipeline_characterization.md`, written by `stimuli/sv/pipeline_vs_raw.py`):
#   stimuli/sv/characterization/pipeline_results.tsv
#     suite<TAB>status<TAB>relpath<TAB>secs<TAB>rc<TAB>furthest<TAB>origin_or_reason
#
# Environment:
#   PGEN_PARSE_PROBE_BIN   the probe (default rust/target/release/parseability_probe; a debug probe
#                          is accepted and RECORDED in the report's identity block)
#   PGEN_PIPELINE_TIMEOUT_S  per-file deadline (default 60, the raw lane's)
#   PGEN_PIPELINE_JOBS       parallelism (default 8, the raw lane's)
#   PGEN_PIPELINE_OUT        the results path (default above; set it to measure without touching
#                            the canonical dump)
#   PGEN_PIPELINE_MAX_FILES  cap for a smoke run (0 = all)
set -uo pipefail
export LC_ALL=C
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
[ $# -ge 1 ] || { echo "usage: stimuli/run_pipeline_lane.sh <suite> [<suite>...] | all   (suites: $(python3 stimuli/sv/pipeline_context.py --list-suites | tr '\n' ' '))" >&2; exit 2; }
# `all` = every registered suite — DERIVED from the tree by the context tool, never from the context
# table (a suite whose includes are all local has no row there and would be skipped).
if [ "$1" = all ]; then set -- $(python3 stimuli/sv/pipeline_context.py --list-suites); fi
PROBE="${PGEN_PARSE_PROBE_BIN:-rust/target/release/parseability_probe}"
TIMEOUT_S="${PGEN_PIPELINE_TIMEOUT_S:-60}"
JOBS="${PGEN_PIPELINE_JOBS:-8}"
OUT="${PGEN_PIPELINE_OUT:-stimuli/sv/characterization/pipeline_results.tsv}"
MAX_FILES="${PGEN_PIPELINE_MAX_FILES:-0}"
CONTEXT=stimuli/sv/pipeline_context.tsv
CONTEXT_FILES=stimuli/sv/pipeline_context_files.tsv   # SVPP-EXPANSION.4b: per-file rows (nearest-candidate includes)
[ -x "$PROBE" ] || { echo "pipeline-lane: probe not executable: $PROBE" >&2; exit 2; }
[ -f "$CONTEXT_FILES" ] || { echo "pipeline-lane: missing $CONTEXT_FILES — run stimuli/sv/pipeline_context.py --write" >&2; exit 2; }
[ -f "$CONTEXT" ] || { echo "pipeline-lane: missing $CONTEXT — run stimuli/sv/pipeline_context.py --write" >&2; exit 2; }
# the context is derived: refuse to measure against a stale copy
python3 stimuli/sv/pipeline_context.py --check >/dev/null 2>&1 || { echo "pipeline-lane: $CONTEXT does not equal a fresh derivation — run stimuli/sv/pipeline_context.py --write" >&2; exit 3; }

suite_root() {
  case "$1" in
    uvm-core) printf 'stimuli/sv/uvm\n' ;;
    *) printf 'stimuli/sv/subs/%s\n' "$1" ;;
  esac
}
suite_of() {  # the raw lane's own mapping, so the two lanes join on the suite column
  printf '%s' "$1" | sed -E 's#.*/subs/([^/]+)/.*#\1#; s#.*/stimuli/sv/uvm/.*#uvm-core#'
}
export ROOT PROBE TIMEOUT_S CONTEXT CONTEXT_FILES
export -f suite_of

# milliseconds without a subprocess (the raw lane's helper): bash 5's EPOCHREALTIME, else seconds
_now_ms() {
  if [ -n "${EPOCHREALTIME:-}" ]; then
    local e="${EPOCHREALTIME}"
    printf '%s' "$(( ${e%.*} * 1000 + 10#${e#*.} / 1000 ))"
  else
    printf '%s' "$(( SECONDS * 1000 ))"
  fi
}
export -f _now_ms

context_args() {  # $1 = suite, $2 = repo-relative path → the probe flags: the file's own rows FIRST, then the suite's
  awk -F'\t' -v s="$1" -v f="$2" 'NR>1 && $1==s && $2==f { printf "--sv-include-dir\t%s\n", $3 }' "$CONTEXT_FILES"
  awk -F'\t' -v s="$1" 'NR>1 && $1==s { if ($2=="include_dir") printf "--sv-include-dir\t%s\n", $3; else if ($2=="define") printf "--sv-define\t%s\n", $3 }' "$CONTEXT"
}
export -f context_args

parse_one() {
  local f="$1" suite rel rc status t0 t1 ms err fp origin
  suite="$(suite_of "$f")"
  rel="${f#"$ROOT"/}"
  local -a args=()
  while IFS=$'\t' read -r flag value; do [ -n "$flag" ] && args+=("$flag" "$value"); done < <(context_args "$suite" "$rel")
  t0="$(_now_ms)"
  err="$(timeout "$TIMEOUT_S" "$PROBE" --parse systemverilog "$f" --profile sv_2017 --preprocess "${args[@]}" 2>&1 >/dev/null)"
  rc=$?
  t1="$(_now_ms)"
  ms=$((t1 - t0)); [ "$ms" -ge 0 ] || ms=0
  case "$rc" in
    0) status=pass ;; 1) status=fail ;; 3) status=pp_fail ;; 124) status=timeout ;;
    *) if [ "$rc" -ge 128 ]; then status=crash; else status=fail; fi ;;
  esac
  fp=""; origin=""
  case "$err" in *furthest_position=*) fp="${err##*furthest_position=}"; fp="${fp%%[!0-9]*}" ;; esac
  case "$err" in *'[origin='*) origin="${err##*\[origin=}"; origin="${origin%%]*}" ;; esac
  if [ "$status" = pp_fail ]; then origin="$(printf '%s' "$err" | grep -m1 'preprocess_failed' | sed -E "s/.*failed to preprocess: //; s/.*preprocess_failed[^:]*: //" | tr '\t' ' ' | cut -c1-200)"; fi
  printf '%s\t%s\t%s\t%d.%02d\t%s\t%s\t%s\n' "$suite" "$status" "$rel" "$((ms/1000))" "$(( (ms%1000)/10 ))" "$rc" "$fp" "$origin"
}
export -f parse_one

LIST="$(mktemp "$ROOT/rust/target/pipeline_lane_files.XXXXXX")"
trap 'rm -f "$LIST"' EXIT
: > "$LIST"
for suite in "$@"; do
  root="$(suite_root "$suite")"
  [ -d "$root" ] || { echo "pipeline-lane: no such suite root $root" >&2; exit 2; }
  find "$ROOT/$root" -type f \( -name '*.sv' -o -name '*.svh' -o -name '*.v' \) >> "$LIST"
done
sort -o "$LIST" "$LIST"
if [ "$MAX_FILES" -gt 0 ]; then head -n "$MAX_FILES" "$LIST" > "$LIST.cap" && mv "$LIST.cap" "$LIST"; fi
total=$(wc -l < "$LIST" | tr -d ' ')
mkdir -p "$(dirname "$OUT")"
t_start=$(date +%s)
xargs -P "$JOBS" -I{} bash -c 'parse_one "$@"' _ {} < "$LIST" | sort -t$'\t' -k1,1 -k3,3 > "$OUT"
t_end=$(date +%s)
awk -F'\t' -v total="$total" -v secs="$((t_end - t_start))" -v probe="$PROBE" -v timeout="$TIMEOUT_S" -v jobs="$JOBS" '
  {c[$2]++}
  END { printf "PIPELINE-LANE: files=%d pass=%d fail=%d pp_fail=%d timeout=%d crash=%d wall_s=%d probe=%s timeout_s=%s jobs=%s\n", total, c["pass"], c["fail"], c["pp_fail"], c["timeout"], c["crash"], secs, probe, timeout, jobs }' "$OUT"
