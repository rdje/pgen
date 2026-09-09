#!/usr/bin/env bash
# The ONE place the preprocess→parse lane's per-file context becomes probe flags (SVPP-EXPANSION.4b/.4c/.6d).
# Sourced by stimuli/run_pipeline_lane.sh and by stimuli/sv/pipeline_library.py (`bash -c 'source …; context_args S F'`).
# Every table below is DERIVED from the corpus by stimuli/sv/pipeline_context.py (--write / --check).
#   context_args SUITE REPO_RELATIVE_PATH   → one "FLAG<TAB>VALUE" line per flag, the file's own rows FIRST:
#     --sv-unit-file   the unit predecessors the file's own harness compiles before it (.4c.2; in order)
#     --sv-include-dir the file's nearest-candidate include dirs (.4b), then the suite's include dirs (.4)
#     --sv-define      the defines the file's own driver passes (.4c.1), then the suite's defines
#     --lib-in         the library the lane built (.6d.1) — only when PGEN_PIPELINE_LIB_DIR names an existing dir
CONTEXT="${PGEN_PIPELINE_CONTEXT:-stimuli/sv/pipeline_context.tsv}"
CONTEXT_FILES="${PGEN_PIPELINE_CONTEXT_FILES:-stimuli/sv/pipeline_context_files.tsv}"
CONTEXT_DEFINES="${PGEN_PIPELINE_CONTEXT_DEFINES:-stimuli/sv/pipeline_context_defines.tsv}"
CONTEXT_UNITS="${PGEN_PIPELINE_CONTEXT_UNITS:-stimuli/sv/pipeline_context_units.tsv}"
context_args() {
  [ -f "$CONTEXT_UNITS" ] && awk -F'\t' -v s="$1" -v f="$2" 'NR>1 && $1==s && $2==f { printf "--sv-unit-file\t%s\n", $4 }' "$CONTEXT_UNITS"
  awk -F'\t' -v s="$1" -v f="$2" 'NR>1 && $1==s && $2==f { printf "--sv-include-dir\t%s\n", $3 }' "$CONTEXT_FILES"
  [ -f "$CONTEXT_DEFINES" ] && awk -F'\t' -v s="$1" -v f="$2" 'NR>1 && $1==s && $2==f { printf "--sv-define\t%s\n", $3 }' "$CONTEXT_DEFINES"
  awk -F'\t' -v s="$1" 'NR>1 && $1==s { if ($2=="include_dir") printf "--sv-include-dir\t%s\n", $3; else if ($2=="define") printf "--sv-define\t%s\n", $3 }' "$CONTEXT"
  if [ -n "${PGEN_PIPELINE_LIB_DIR:-}" ] && [ -d "$PGEN_PIPELINE_LIB_DIR" ]; then printf -- '--lib-in\t%s\n' "$PGEN_PIPELINE_LIB_DIR"; fi
  return 0
}
export CONTEXT CONTEXT_FILES CONTEXT_DEFINES CONTEXT_UNITS
export -f context_args
