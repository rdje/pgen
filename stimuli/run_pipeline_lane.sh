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
#   <the dump>.provenance   (SV-CORPUS-GRAD.38; ignored like the dump) — WHICH RUN wrote the dump, as
#     key<TAB>value rows: profile, probe, probe_sha256, timeout_s, jobs, files, dump_sha256, and the
#     library pass's two dumps with their digests. Written only after the dump is in place, so a
#     sidecar whose dump_sha256 does not match the dump beside it names a different run — the census
#     of two readings (pipeline_sv2023_lane/census.py) refuses on exactly that.
#
# Environment:
#   PGEN_PARSE_PROBE_BIN   the probe (default rust/target/release/parseability_probe; a debug probe
#                          is accepted and RECORDED in the report's identity block)
#   PGEN_PIPELINE_TIMEOUT_S  per-file deadline (default 60, the raw lane's)
#   PGEN_PIPELINE_JOBS       parallelism (default 8, the raw lane's)
#   PGEN_PIPELINE_OUT        the results path (default above; set it to measure without touching
#                            the canonical dump)
#   PGEN_PIPELINE_MAX_FILES  cap for a smoke run (0 = all)
#   PGEN_PIPELINE_PROFILE    the edition every row and every package file is read under
#                            (SV-CORPUS-GRAD.38): `sv_2017` (default — the PRIMARY reading, the one the
#                            verdict-coverage tuple is defined over), `sv_2023` (the second reading:
#                            its dump, library and membership table carry the `_sv2023` suffix, so the
#                            readings never share a path and no run can overwrite another) or
#                            `sv_2023_tolerant` (SV-CORPUS-GRAD.43: the opt-in tolerance profile's
#                            reading, suffix `_sv2023_tolerant`)
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
PROFILE="${PGEN_PIPELINE_PROFILE:-sv_2017}"
case "$PROFILE" in
  sv_2017) PROFILE_SUFFIX="" ;;
  sv_2023) PROFILE_SUFFIX="_sv2023" ;;
  sv_2023_tolerant) PROFILE_SUFFIX="_sv2023_tolerant" ;;
  *) echo "pipeline-lane: unknown profile '$PROFILE' (sv_2017 | sv_2023 | sv_2023_tolerant)" >&2; exit 2 ;;
esac
OUT="${PGEN_PIPELINE_OUT:-stimuli/sv/characterization/pipeline_results${PROFILE_SUFFIX}.tsv}"
MAX_FILES="${PGEN_PIPELINE_MAX_FILES:-0}"
# the per-file context → probe flags lives in ONE sourceable place (SVPP-EXPANSION.6d.1), shared with the library builder:
# the four derived tables (.4 suite rows, .4b per-file includes, .4c.1 per-file defines, .4c.2 per-file unit predecessors)
# shellcheck source=stimuli/sv/pipeline_context_args.sh
source stimuli/sv/pipeline_context_args.sh
LIB_DIR="${PGEN_PIPELINE_LIB_DIR:-stimuli/sv/characterization/pipeline_library${PROFILE_SUFFIX}}"   # SVPP-EXPANSION.6d.1: the library the lane builds
LIB_TSV="${PGEN_PIPELINE_LIB_TSV:-stimuli/sv/characterization/pipeline_library${PROFILE_SUFFIX}.tsv}"
LIB_MEMBERS="${PGEN_PIPELINE_LIB_MEMBERS:-stimuli/sv/characterization/pipeline_library_members${PROFILE_SUFFIX}.tsv}"
[ -x "$PROBE" ] || { echo "pipeline-lane: probe not executable: $PROBE" >&2; exit 2; }
[ -f "$CONTEXT_FILES" ] || { echo "pipeline-lane: missing $CONTEXT_FILES — run stimuli/sv/pipeline_context.py --write" >&2; exit 2; }
[ -f "$CONTEXT" ] || { echo "pipeline-lane: missing $CONTEXT — run stimuli/sv/pipeline_context.py --write" >&2; exit 2; }
# the context is derived: refuse to measure against a stale copy
python3 stimuli/sv/pipeline_context.py --check >/dev/null 2>&1 || { echo "pipeline-lane: $CONTEXT does not equal a fresh derivation — run stimuli/sv/pipeline_context.py --write" >&2; exit 3; }

# SVPP-EXPANSION.6d.1 — the LIBRARY PASS: every package file of the design suites parsed in dependency order
# (`import X::` edges from the preprocessed text), each exporting its facts (`--lib-out`) for the packages that
# import it; then EVERY row below runs with `--lib-in` of the result. Rebuilt on every run (derived, like the dump).
if [ "${PGEN_PIPELINE_LIBRARY:-1}" != 0 ]; then
  rm -rf "$LIB_DIR"; mkdir -p "$LIB_DIR"
  python3 stimuli/sv/pipeline_library.py --probe "$PROBE" --profile "$PROFILE" --lib-dir "$LIB_DIR" --out "$LIB_TSV" --out-members "$LIB_MEMBERS" --timeout-s "$TIMEOUT_S" || { echo "pipeline-lane: the library pass failed" >&2; exit 4; }
  export PGEN_PIPELINE_LIB_DIR="$LIB_DIR"
else
  unset PGEN_PIPELINE_LIB_DIR
fi

suite_root() {
  case "$1" in
    uvm-core) printf 'stimuli/sv/uvm\n' ;;
    *) printf 'stimuli/sv/subs/%s\n' "$1" ;;
  esac
}
suite_of() {  # the raw lane's own mapping, so the two lanes join on the suite column
  printf '%s' "$1" | sed -E 's#.*/subs/([^/]+)/.*#\1#; s#.*/stimuli/sv/uvm/.*#uvm-core#'
}
export ROOT PROBE TIMEOUT_S PROFILE
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


parse_one() {
  local f="$1" suite rel rc status t0 t1 ms err fp origin
  suite="$(suite_of "$f")"
  rel="${f#"$ROOT"/}"
  local -a args=()
  while IFS=$'\t' read -r flag value; do [ -n "$flag" ] && args+=("$flag" "$value"); done < <(context_args "$suite" "$rel")
  t0="$(_now_ms)"
  err="$(timeout "$TIMEOUT_S" "$PROBE" --parse systemverilog "$f" --profile "$PROFILE" --preprocess "${args[@]}" 2>&1 >/dev/null)"
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
# SVPP-EXPANSION.5(g)(h), 2026-09-09: the dump is PUBLISHED ATOMICALLY — the rows accumulate in a temporary
# path beside it and move into place only here, at the end. Writing straight to "$OUT" truncated the previous
# dump the moment a run started, so a run the memory guard killed left an EMPTY dump (and a render then
# published a rows=0 report from it), and a run whose dump path was moved aside meanwhile (the doctrine's
# --self-test does that on the real path) wrote its rows into an unlinked inode and finished with nothing.
OUT_TMP="$OUT.tmp.$$"
trap 'rm -f "$LIST" "$OUT_TMP"' EXIT
xargs -P "$JOBS" -I{} bash -c 'parse_one "$@"' _ {} < "$LIST" | sort -t$'\t' -k1,1 -k3,3 > "$OUT_TMP"
mv -f "$OUT_TMP" "$OUT"
t_end=$(date +%s)
# SV-CORPUS-GRAD.38 — the run's provenance, beside the dump it describes (atomic, after the dump)
python3 - "$OUT" "$PROFILE" "$PROBE" "$TIMEOUT_S" "$JOBS" "$total" "${PGEN_PIPELINE_LIBRARY:-1}" "$LIB_TSV" "$LIB_MEMBERS" <<'PY'
import hashlib, os, pathlib, sys
out, profile, probe, timeout_s, jobs, files, library, lib_tsv, lib_members = sys.argv[1:10]
def sha(p):
    return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
rows = [("profile", profile), ("probe", probe), ("probe_sha256", sha(probe)), ("timeout_s", timeout_s),
        ("jobs", jobs), ("files", files), ("dump", out), ("dump_sha256", sha(out))]
if library != "0":
    rows += [("library", lib_tsv), ("library_sha256", sha(lib_tsv)),
             ("library_members", lib_members), ("library_members_sha256", sha(lib_members))]
else:
    rows += [("library", "off")]
side = pathlib.Path(out + ".provenance")
tmp = pathlib.Path(f"{side}.tmp.{os.getpid()}")
tmp.write_text("".join(f"{k}\t{v}\n" for k, v in rows))
tmp.replace(side)
PY
awk -F'\t' -v total="$total" -v secs="$((t_end - t_start))" -v probe="$PROBE" -v timeout="$TIMEOUT_S" -v jobs="$JOBS" -v profile="$PROFILE" '
  {c[$2]++}
  END { printf "PIPELINE-LANE: files=%d pass=%d fail=%d pp_fail=%d timeout=%d crash=%d wall_s=%d probe=%s timeout_s=%s jobs=%s profile=%s\n", total, c["pass"], c["fail"], c["pp_fail"], c["timeout"], c["crash"], secs, probe, timeout, jobs, profile }' "$OUT"
