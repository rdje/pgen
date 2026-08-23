#!/usr/bin/env bash
# `SV-CORPUS-GRAD.13c.2b` — the five-arm A/B that pinned the defect to ONE grammar edge, and now
# holds the FIX in place.
#
# ⭐ RE-BASELINED 2026-08-23 (`PGEN-SV-CORPUS-GRAD-0279`). The defect this matrix was written to
# reproduce is FIXED: `ENGINE-UNIVERSAL-SERVICES.17` slice 9 (`PGEN-ENGINE-UNIVERSAL-SERVICES-0032`)
# flipped the indirect-LR eliminator from "absorb only if starvation-safe" to "absorb if safe OR if
# a call-site follow-restriction guard makes it safe", which absorbs the `casting_type` knot with
# `casting_type_lr_guard1[loop]` at `cast alt#0`. All five arms ACCEPT at HEAD.
#
# ⛔ THE EXPECTATIONS MOVED WITH THE TREE, AND THAT IS THE POINT — a matrix left asserting the
# pre-fix verdicts is RED on a CORRECT tree, which is an instrument that can no longer detect
# anything. Re-baselined, it is a REGRESSION DETECTOR: if the guarded admission is ever backed out,
# arms 4 and 5 go REJECT and this script exits nonzero.
#
#   arm                                        casting_type resolves via        before  at HEAD
#   -----------------------------------------  ------------------------------  ------  -------
#   control_size_cast_in_statement.sv          constant_primary (FIRST entry)  ACCEPT  ACCEPT
#   control_simple_type_cast_in_constant.sv    simple_type (branch 1/5)        ACCEPT  ACCEPT
#   control_param_name_cast_in_constant.sv     simple_type -> ps_type_ident    ACCEPT  ACCEPT
#   defect_constant_size_cast.sv               constant_primary — was CYCLE-BLOCKED REJECT  ACCEPT
#   defect_constant_size_cast_corpus_shape.sv  constant_primary — was CYCLE-BLOCKED REJECT  ACCEPT
#
# ⇒ the diagnosis the matrix established still stands and is what dates the fix: the discriminator
# was never "constant expression" and never "size cast" — it was whether casting_type had a viable
# alternative OTHER than `constant_primary` at the seed position. The three control arms are exactly
# the cases where it did, which is why they parsed BEFORE the fix as well as after. ⛔ Keep them:
# an all-ACCEPT matrix in which every arm flipped for the same reason would prove nothing about
# WHICH edge moved, and these three are the arms that did not move.
#
# Usage:  bash docs/tasks/artifacts/sv_corpus_grad/constant_size_cast/casting_type_edge_matrix.sh
set -uo pipefail

# ⛔ TERMINATION GUARD (`SV-CORPUS-GRAD.13c.2b`, 2026-08-23). Directive 12 forbids a hard-coded
# depth, so the root is walked up from this script at run time. The walk MUST have its own
# terminator: `cd ..` at `/` SUCCEEDS and is a no-op, so a `cd .. || exit` escape can NEVER fire and
# the loop spins forever outside a checkout (measured: 201+ iterations, `pwd=/`, no error). The
# Python sibling `_repo_root.py` never had this — `Path.parents` is finite and it REFUSES by name.
find_repo_root() {
  local d
  d="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)" || return 1
  while [ ! -f "$d/CLAUDE.md" ] || [ ! -d "$d/grammars" ]; do
    [ "$d" = "/" ] && return 1
    d="$(dirname "$d")"
  done
  printf '%s\n' "$d"
}
ROOT="$(find_repo_root)" || { echo "REFUSE: no repository root (CLAUDE.md + grammars/) above ${BASH_SOURCE[0]}" >&2; exit 2; }
PROBE="$ROOT/rust/target/release/parseability_probe"
REPROS="$ROOT/stimuli/sv/adjudication_repros"

if [ ! -x "$PROBE" ]; then
  echo "REFUSE: $PROBE is missing — build it with:" >&2
  echo "  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)" >&2
  exit 2
fi

status=0
run_arm() {
  local file="$1" expect="$2" note="$3" out verdict
  out="$("$PROBE" --parse systemverilog "$REPROS/$file" --profile sv_2017 2>&1 | tail -1)"
  case "$out" in
    parse_full\ passed*) verdict=ACCEPT ;;
    *) verdict="REJECT ${out##*[}" ; verdict="${verdict%]*}" ;;
  esac
  printf '%-44s %-8s expect=%-6s %s\n' "$file" "${verdict%% *}" "$expect" "$note"
  [ "${verdict%% *}" = "$expect" ] || { echo "  ⛔ MISMATCH: $out"; status=1; }
}

run_arm control_size_cast_in_statement.sv         ACCEPT "8'(1) in a STATEMENT expression"
run_arm control_simple_type_cast_in_constant.sv   ACCEPT "int'(1) — casting_type = simple_type"
run_arm control_param_name_cast_in_constant.sv    ACCEPT "W'(1)  — casting_type = ps_type_identifier"
run_arm defect_constant_size_cast.sv              ACCEPT "8'(1) in a CONSTANT expression — was REJECT before ES.17 slice 9"
run_arm defect_constant_size_cast_corpus_shape.sv ACCEPT "512'({…}) — the OpenTitan corpus shape, same flip"

echo
if [ "$status" = 0 ]; then
  echo "VERDICT: matrix holds — 5 ACCEPT, the post-fix baseline (PGEN-ENGINE-UNIVERSAL-SERVICES-0032)."
else
  echo "VERDICT: an arm moved — this is a REGRESSION, not progress. If arms 4/5 went back to REJECT,"
  echo "         the guarded indirect-LR admission has been backed out: check that"
  echo "         'ast_pipeline grammars/systemverilog.ebnf --lint-grammar' still reports"
  echo "         left_recursion_unhandled=0 and that no build passes"
  echo "         --indirect-lr-admit-starvation-safe-only. If a CONTROL arm (1-3) moved instead,"
  echo "         the regression is wider than this knot — those three never depended on it."
fi
exit "$status"
