#!/usr/bin/env bash
# The ONE place the preprocess→parse lane's per-file context becomes probe flags (SVPP-EXPANSION.4b/.4c/.6d).
# Sourced by stimuli/run_pipeline_lane.sh and by stimuli/sv/pipeline_library.py (`bash -c 'source …; context_args S F'`).
# Every table below is DERIVED from the corpus by stimuli/sv/pipeline_context.py (--write / --check).
#   context_args SUITE REPO_RELATIVE_PATH   → one "FLAG<TAB>VALUE" line per flag, the file's own rows FIRST:
#     --unit-file   the unit predecessors the file's own harness compiles before it (.4c.2; in order),
#                   then the suite's hand-curated unit predecessors (.6d.6; in order)
#     --include-dir the file's nearest-candidate include dirs (.4b), then the suite's include dirs (.4)
#     --symbol      the defines the file's own driver passes (.4c.1), then the suite's defines
#     --lib-in         the library the lane built (.6d.1) — only when PGEN_PIPELINE_LIB_DIR names an existing dir
#   An include FRAGMENT (.4c.6, pipeline_context_fragments.tsv) is read as its includer compiles it: the SUITE's rows
#   first (the flow's unit predecessors), the root includer's own rows, then each includer's directory and its PREFIX
#   (the text before the `include line, materialized under rust/target/pipeline_prefix/, the conditional regions open
#   at the cut closed with `endif) as a unit file, down the chain; then the fragment's own rows.
#   PGEN_PIPELINE_NO_FRAGMENTS=1 → the standalone reading (the lane's fallback when a prefix refuses).
CONTEXT="${PGEN_PIPELINE_CONTEXT:-stimuli/sv/pipeline_context.tsv}"
CONTEXT_FILES="${PGEN_PIPELINE_CONTEXT_FILES:-stimuli/sv/pipeline_context_files.tsv}"
CONTEXT_DEFINES="${PGEN_PIPELINE_CONTEXT_DEFINES:-stimuli/sv/pipeline_context_defines.tsv}"
CONTEXT_UNITS="${PGEN_PIPELINE_CONTEXT_UNITS:-stimuli/sv/pipeline_context_units.tsv}"
CONTEXT_FRAGMENTS="${PGEN_PIPELINE_CONTEXT_FRAGMENTS:-stimuli/sv/pipeline_context_fragments.tsv}"   # .4c.6
PREFIX_DIR="${PGEN_PIPELINE_PREFIX_DIR:-rust/target/pipeline_prefix}"                               # .4c.6: the materialized prefixes (derived, never tracked)
_own_rows() {  # SUITE FILE → the file's OWN per-file rows, in order: unit predecessors, include dirs, defines
  [ -f "$CONTEXT_UNITS" ] && awk -F'\t' -v s="$1" -v f="$2" 'NR>1 && $1==s && $2==f { printf "--unit-file\t%s\n", $4 }' "$CONTEXT_UNITS"
  awk -F'\t' -v s="$1" -v f="$2" 'NR>1 && $1==s && $2==f { printf "--include-dir\t%s\n", $3 }' "$CONTEXT_FILES"
  [ -f "$CONTEXT_DEFINES" ] && awk -F'\t' -v s="$1" -v f="$2" 'NR>1 && $1==s && $2==f { printf "--symbol\t%s\n", $3 }' "$CONTEXT_DEFINES"
  return 0
}
_suite_rows() {  # SUITE → the suite-wide rows
  awk -F'\t' -v s="$1" 'NR>1 && $1==s { if ($2=="include_dir") printf "--include-dir\t%s\n", $3; else if ($2=="define") printf "--symbol\t%s\n", $3; else if ($2=="unit_file") printf "--unit-file\t%s\n", $3 }' "$CONTEXT"
  return 0
}
_prefix_of() {  # INCLUDER LINE CLOSE → the materialized prefix: the includer's text BEFORE the `include line, the CLOSE
  # conditional regions still open there closed with `endif (the probe refuses a unit file left inside one) (.4c.6); atomic, idempotent
  local inc="$1" line="$2" close="${3:-0}" out tmp i
  out="$PREFIX_DIR/$(printf '%s' "$inc" | tr '/' '_').L${line}.svh"
  mkdir -p "$PREFIX_DIR"
  tmp="$out.$$.tmp"
  { head -n "$((line - 1))" "$inc"; i=0; while [ "$i" -lt "$close" ]; do printf '`endif\n'; i=$((i + 1)); done; } > "$tmp" && mv -f "$tmp" "$out"
  printf '%s' "$out"
}
context_args() {
  local s="$1" f="$2" chain="" root="" inc line close
  # SVPP-EXPANSION.4c.6 — an include FRAGMENT is read as its includer compiles it, in the real compile's order: the
  # SUITE's rows first (the flow's unit predecessors — OpenTitan's unit starts with UVM, .6d.6/.6d.7 — define what the
  # includer's text uses before its own includes do; measured: the prefix preprocessed BEFORE them refused
  # `uvm_object_utils` as incomplete), then the root includer's own rows, then down the chain each includer's directory
  # as an include dir and its prefix (the text before the `include line, the open conditionals closed) as a unit file —
  # its definitions carry over, its text is discarded, the fragment's own guard is untouched — then the fragment's own
  # rows. PGEN_PIPELINE_NO_FRAGMENTS=1 reads the row standalone (the lane's fallback when a prefix itself refuses:
  # the includer's own context gap is then recorded beside the dump, and the fragment's text keeps its reading).
  if [ -z "${PGEN_PIPELINE_NO_FRAGMENTS:-}" ] && [ -f "$CONTEXT_FRAGMENTS" ]; then
    chain="$(awk -F'\t' -v s="$s" -v f="$f" 'NR>1 && $1==s && $2==f { printf "%s\t%s\t%s\n", $4, $5, $6 }' "$CONTEXT_FRAGMENTS")"
  fi
  if [ -n "$chain" ]; then
    root="${chain%%$'\t'*}"
    _suite_rows "$s"
    _own_rows "$s" "$root"
    while IFS=$'\t' read -r inc line close; do
      [ -n "$inc" ] || continue
      printf -- '--include-dir\t%s\n' "$(dirname "$inc")"
      printf -- '--unit-file\t%s\n' "$(_prefix_of "$inc" "$line" "$close")"
    done <<< "$chain"
    _own_rows "$s" "$f"
  else
    _own_rows "$s" "$f"
    _suite_rows "$s"
  fi
  if [ -n "${PGEN_PIPELINE_LIB_DIR:-}" ] && [ -d "$PGEN_PIPELINE_LIB_DIR" ]; then printf -- '--lib-in\t%s\n' "$PGEN_PIPELINE_LIB_DIR"; fi
  return 0
}
export CONTEXT CONTEXT_FILES CONTEXT_DEFINES CONTEXT_UNITS CONTEXT_FRAGMENTS PREFIX_DIR
export -f _own_rows _suite_rows _prefix_of context_args
