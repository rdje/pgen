#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.13 — the indirect-left-recursion probe driver.
#
# Runs the P1/P2/P3 synthetics (see README.md) over the shared input set through BOTH oracles and
# prints one row per (probe, oracle, input):
#
#   * GEN   — the real generated parser, via the scratch slot (`parseability_probe --parse scratch`).
#             Authoritative BY CONSTRUCTION. Costs a `focus_scratch` + probe relink per probe file.
#   * INTERP— the grammar-AST interpreter, via `ast_pipeline --interpret-parse`. Authoritative BY
#             VERIFICATION only, and this leaf MEASURED it diverging from GEN on exactly this shape,
#             so it is reported beside GEN rather than instead of it.
#
# ⛔ WHY BOTH, ALWAYS. The first draft of this measurement ran the interpreter alone, because it is
# the cheap one, and got a table that was wrong in BOTH directions on the defect probe (P1 `t'(n)`
# INTERP=reject / GEN=accept; P1 `n'(n)` INTERP=accept / GEN=reject). A single-oracle run here does
# not under-report — it MISREPORTS. The divergence itself is tracked as `.14`.
#
# Usage (from the repository root):
#   docs/tasks/artifacts/engine_universal_services/indirect_lr/probe.sh            # both oracles
#   docs/tasks/artifacts/engine_universal_services/indirect_lr/probe.sh --interp-only
#
# `--interp-only` skips the scratch-slot cycle (~3 min per probe file) for a fast re-read of the
# interpreter column; it can never be the basis of a claim about the engine on its own.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../../../../.." && pwd)"
cd "$REPO_ROOT" || exit 2

AST_PIPELINE="rust/target/debug/ast_pipeline"
PROBE_BIN="rust/target/debug/parseability_probe"
SCRATCH="grammars/scratch/scratch.ebnf"
WORK="rust/target/indirect_lr_probe"
INTERP_ONLY=0
[[ "${1:-}" == "--interp-only" ]] && INTERP_ONLY=1

# ⭐ P4 is the slice-5 acceptance probe: P1's shape with the return annotations the real grammars
# carry, so the ENGINE eliminates it instead of a hand-written rewrite. Its row should read like
# P3's (all five accept) with NO hand-eliminated grammar in the loop. P1 stays as the DEFECT
# reproduction — it is unannotated, so the pass refuses it by design (see p4's header).
PROBES=(p1_knot_a_defect p2_eliminated_at_inner_rule p3_eliminated_at_consumer_rule p4_knot_a_annotated)
# The shared input set. `t'(n)` is ONE cast level (seeded by `ct`'s own alternative, no recursion
# needed); `n'(n)` and `t'(n)'(n)` each need the cycle once; `t'(n)'(n)'(n)` needs it twice.
INPUTS=("n" "t'(n)" "n'(n)" "t'(n)'(n)" "t'(n)'(n)'(n)")

mkdir -p "$WORK" || exit 2
[[ -x "$AST_PIPELINE" ]] || {
  echo "missing $AST_PIPELINE — build it: (cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline)" >&2
  exit 2
}

# ⭐ Restore the scratch slot on ANY exit, including a failure or a Ctrl-C. The slot is a tracked
# fixture and an un-restored one is a dirty tree the next task-tree pivot would inherit
# (TOOLBOX §1.3's "preserve, THEN restore" discipline, made unconditional).
#
# ⛔⛔ RESTORING THE FIXTURE IS ONLY HALF THE SLOT, and the first version of this trap did only that
# half. `generated/scratch_parser.rs` is git-ignored, so `git checkout` cannot touch it and
# `git status` cannot report it: the tree looked clean while the generated parser still held the LAST
# probe's grammar. Two tests read the slot as a matched pair and both went RED and stayed RED —
# `parse_harness_equivalence::gate::certified_grammars_are_byte_identical` (`scratch DIVERGE
# samples=3 agree=1 diverge=2`, the interpreter reading the restored greeting fixture against a
# generated parser built from `p3_eliminated_at_consumer_rule.ebnf`) and
# `parser_registry::tests::scratch_slot_parses_the_blessed_fixture_to_the_known_verdict_and_ast`.
# ⇒ the restore must regenerate, not just check out. It costs one `focus_scratch` (~80 s) on a run
# that already paid three of them.
restore_scratch() {
  [[ $INTERP_ONLY -eq 1 ]] && return 0
  git checkout -- "$SCRATCH" 2>/dev/null || return 0
  make -C rust SHELL=/bin/bash focus_scratch >"$WORK/restore_focus.log" 2>&1 || {
    echo "⛔ scratch slot RESTORE-REGENERATE failed — generated/scratch_parser.rs still holds the" >&2
    echo "   last probe's grammar. Re-run by hand before committing anything:" >&2
    echo "     make -C rust SHELL=/bin/bash focus_scratch    # log: $WORK/restore_focus.log" >&2
  }
}
trap restore_scratch EXIT

interp_verdict() {  # $1 = grammar file, $2 = input file
  local line
  line="$("$AST_PIPELINE" "$1" --interpret-parse "$2" 2>/dev/null | grep -m1 '^INTERPRET-PARSE:')"
  [[ -z "$line" ]] && { echo "TOOL-ERROR"; return; }
  if [[ "$line" == *"accepted=true"* ]]; then echo "accept"; else
    echo "reject@$(sed -E 's/.*furthest_position=([0-9]+).*/\1/' <<<"$line")"
  fi
}

gen_verdict() {  # $1 = input file
  local out
  out="$("$PROBE_BIN" --parse scratch "$1" 2>&1)"
  if [[ "$out" == *"parse_full passed"* ]]; then echo "accept"; else
    echo "reject@$(sed -E 's/.*furthest_position=([0-9]+).*/\1/' <<<"$out")"
  fi
}

printf '%-32s %-16s %-10s %-10s %s\n' PROBE INPUT GEN INTERP AGREE
for probe in "${PROBES[@]}"; do
  grammar="$HERE/$probe.ebnf"
  if [[ $INTERP_ONLY -eq 0 ]]; then
    cp "$grammar" "$SCRATCH" || exit 2
    make -C rust SHELL=/bin/bash focus_scratch >"$WORK/$probe.focus.log" 2>&1 ||
      { echo "focus_scratch FAILED for $probe — see $WORK/$probe.focus.log" >&2; exit 2; }
    (cd rust && cargo build --features generated_parsers --bin parseability_probe) \
      >"$WORK/$probe.build.log" 2>&1 ||
      { echo "probe rebuild FAILED for $probe — see $WORK/$probe.build.log" >&2; exit 2; }
  fi
  for input in "${INPUTS[@]}"; do
    printf '%s' "$input" >"$WORK/in.txt"
    interp="$(interp_verdict "$grammar" "$WORK/in.txt")"
    if [[ $INTERP_ONLY -eq 1 ]]; then gen="(skipped)"; agree="-"; else
      gen="$(gen_verdict "$WORK/in.txt")"
      # Compare the VERDICT only: `furthest_position` legitimately differs between the two
      # implementations on a reject, and pinning it here would report noise as divergence.
      [[ "${gen%%@*}" == "${interp%%@*}" ]] && agree="yes" || agree="DIVERGE"
    fi
    printf '%-32s %-16s %-10s %-10s %s\n' "$probe" "$input" "$gen" "$interp" "$agree"
  done
done
