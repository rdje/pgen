#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.17 slice 8 — does the guard PGEN EMITS actually PARSE?
#
# ⛔⛔ THE HOLE THIS BANK CLOSES, stated as `.17` slice 7 left it: *"nothing has generated a parser
# from PGEN's own guarded output and run an input through it."* Slice 6 measured a HAND-WRITTEN `g7`
# and slice 7 asserted PGEN's emission matches it IN THE GENERATED AST. Two artifacts, agreeing in a
# tree, with the claim "the guard works" resting on the gap between them. Every row below is on the
# far side of that gap: grammars that are still left-recursive, eliminated and guarded by the real
# pass, compiled to real parsers, fed real bytes.
#
# TWO GRAMMARS, TWO ARMS EACH, and the second grammar exists because a PLANT proved the first one's
# rows could not check what they appeared to check.
#
#   S1  scratch := kw_k eq outer_cast semi | kw_k eq prim semi
#       two holders — the residual-BEARING one needs the guard, the residual-FREE one must stay
#       unguarded. ⭐ `e7` (`k = n;`) is the CALL-SITE-SCOPING row: a guard on the shared rule
#       rejects it. ⛔ `e1`–`e6` are NOT discriminating for the guard here — the second alternative
#       parses them on its own through the eliminated-but-unguarded `prim`.
#   S2  scratch := kw_k eq outer_cast semi
#       S1 minus that second alternative. Now `e1`–`e6` have exactly ONE route to a parse and each
#       tests a guard POSITION. ⛔ It cannot test `e7` — there is no residual-free holder to reach.
#
# ⇒ NEITHER GRAMMAR ALONE MEASURES THE DESIGN. S1 proves the guard does no damage where it must do
# none; S2 proves it does the work where it must. `.17` slice 4 drew this exact line one ladder down
# — `guard_effectiveness/probe.sh` runs `g4`/`g5` on `e1` and `e7` ONLY, *"because their second
# alternative absorbs e2–e6 on its own, so those rows would pass under BOTH hypotheses and prove
# nothing"* — and S1's first row set did not inherit it.
#
# THE ARMS, and the one-difference is the admission and nothing else:
#
#   A   the SHIPPED admission (make focus_scratch)   `starvation-safe candidates: 0/3` ⇒ nothing is
#                                                     absorbed, the cycle survives, the runtime cycle
#                                                     guard REJECTS the chained casts
#   B   --indirect-lr-admit-guard-feasible            ⇒ `prim` is absorbed AND its surviving
#                                                     starvation site gets the guarded clone chain
#
# ⭐⭐ SIX GEN ROWS FLIP REJECT -> ACCEPT between the arms (three per grammar). A bank where every
# row accepts under both arms would prove only that the grammars are parseable, so the run DERIVES
# the flip count and FAILS if it reaches zero.
#
# ⭐⭐ AND IT EXERCISES THE HOP CLONE, which is `.17` acceptance (d)'s standing obligation. Both
# holders name `ct`, not `prim`, so the transparent chain is TWO rules long and PGEN must emit
# `prim_lr_guard0_ct`. SystemVerilog's own dry run never reaches that rule (all three of its chains
# have `chain:` of length 1), so before this bank the hop-clone half was covered by unit tests on
# synthetics only.
#
# ⛔ GROUND TRUTH, PER ORACLE (`feedback_instrument_needs_ground_truth`). Every parse row declares
# TWO verdicts — one for GEN, one for INTERP — because on the A arms they legitimately DISAGREE: the
# generated parser has a runtime cycle guard and the interpreter has none (`.14`, the tracked
# divergence). A bank that failed on disagreement would be unable to measure the un-eliminated arms
# at all; a bank that declared one verdict for both would have to pick an oracle to be wrong about.
# So the divergence is DECLARED, and its count is DERIVED from the table and printed.
#
# ⛔⛔ AND THE TWO COLUMNS ARE NOT THE SAME KIND OF CLAIM. **GEN carries every design claim** — it is
# authoritative BY CONSTRUCTION, and every verdict in it was written down before the first run and
# measured correct. **INTERP on the A arms is a PIN of a known-divergent oracle**: nothing in the
# design says what a cycle-guardless interpreter does at its depth ceiling, so those cells record
# `.14`'s behaviour rather than assert what is right.
#
# ⛔ Do NOT adjust an expectation to match a new measurement. `.17`'s decision rests on these rows.
# ⭐ ONE set of cells was corrected during authoring, and it is recorded here rather than quietly
# edited: arm `2A`'s INTERP column was first written `ACCEPT`×6 by EXTRAPOLATION from `1A` (where the
# interpreter does accept), and measured `REJECT`×6. The extrapolation was the error — S2 drops the
# entry alternative that rescues S1's interpreter parse — and the corrected cells are a divergence
# pin, not a design claim. ⛔ No GEN cell was ever wrong: all 26 were predicted and measured correct.
#
# HOW TO RUN (from the repository root) — ~15 minutes, most of it four probe rebuilds:
#   bash docs/tasks/artifacts/engine_universal_services/guard_parses/probe.sh
#
# ⛔ There is no `--interp-only` shortcut, deliberately. The sibling `guard_effectiveness` bank has
# one because its grammars are hand-eliminated and the interpreter can read them as-is. Here the
# INTERP column cannot stand alone for a stronger reason than cost: the A arms' whole finding is that
# the two oracles DISAGREE, so an interpreter-only run would report ACCEPT everywhere and conclude
# the shipped admission already parses it all.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../../.." && pwd)"
cd "$ROOT" || exit 2

AST_PIPELINE="rust/target/debug/ast_pipeline"
PROBE_BIN="rust/target/debug/parseability_probe"
SCRATCH="grammars/scratch/scratch.ebnf"
SCRATCH_JSON="generated/scratch.json"
SCRATCH_PARSER="generated/scratch_parser.rs"
WORK="rust/target/es17_guard_parses"   # repo volume, git-ignored (policy 13)
WIDEN="--indirect-lr-admit-guard-feasible"

# grammar key -> file
declare -A GRAMMAR=(
  [1]="$HERE/s1_guard_source.ebnf"
  [2]="$HERE/s2_holder_only.ebnf"
)

[[ -x "$AST_PIPELINE" ]] || {
  echo "probe: missing $AST_PIPELINE — build it:" >&2
  echo "  (cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline)" >&2
  exit 2
}
# The under-featured-binary trap: an under-featured binary errors on stderr and yields an EMPTY
# verdict on stdout, and two empty result sets diff clean. Refuse up front. ⛔ This bank was written
# by an author who hit exactly that: a plain `cargo build --bin ast_pipeline` silently replaced the
# featured binary, and the next measurement produced no output at all rather than an error.
if ! scripts/require_ast_pipeline_features.sh "$AST_PIPELINE" ebnf_dual_run >/dev/null 2>&1; then
  scripts/require_ast_pipeline_features.sh "$AST_PIPELINE" ebnf_dual_run >&2
  exit 2
fi

mkdir -p "$WORK" || exit 2

# ---- the inputs, byte-identical to the `guard_effectiveness` bank's e1–e7 so the two banks measure
# the SAME language on the hand-written `g7` and on PGEN's emission. `k = … ;` is the enclosing
# statement that makes the holder MANDATORY; without it a shorter parse hides the starvation.
printf 'k = n%s(n);'           "'"     > "$WORK/e1.txt"   # the starvation: the holder needs its residual
printf 'k = n%s(n)%s(n);'      "'" "'" > "$WORK/e2.txt"   # the chain must still take ONE iteration
printf 'k = n%s(n)/*c*/;'      "'"     > "$WORK/e3.txt"   # E1 with a comment AT the guard boundary
printf 'k = n%s(n)/*c*/%s(n);' "'" "'" > "$WORK/e4.txt"   # E2 with a comment AT the guard boundary
printf 'k = t%s(n);'           "'"     > "$WORK/e5.txt"   # the over-long SEED starves the holder
printf 'k = t%s(n)%s(n);'      "'" "'" > "$WORK/e6.txt"   # seeded chain plus one iteration
printf 'k = n;'                        > "$WORK/e7.txt"   # the RESIDUAL-FREE holder — S1 only

# ---- the parse table: <arm> <input> <WANT_GEN> <WANT_INTERP> <note>
# `<arm>` is <grammar><admission>: 1A 1B 2A 2B.
CASES=(
  "1A e1 ACCEPT ACCEPT  one cast: the un-eliminated grammar reaches it without recursing"
  "1A e2 REJECT ACCEPT  ⛔⛔ THE DEFECT .13 OPENED: the runtime cycle guard refuses the chained cast"
  "1A e3 ACCEPT ACCEPT  a comment does not change the one-cast case"
  "1A e4 REJECT ACCEPT  ⛔ nor the chained one"
  "1A e5 ACCEPT ACCEPT  the seeded single cast is reachable"
  "1A e6 REJECT ACCEPT  ⛔ the seeded chain is not"
  "1A e7 ACCEPT ACCEPT  the residual-free holder, unaffected either way"

  "1B e1 ACCEPT ACCEPT  ~ not guard-discriminating on S1: alt#1 can parse it unguarded"
  "1B e2 ACCEPT ACCEPT  ⭐ FLIP vs 1A — but of the ADMISSION, not the guard (see S2)"
  "1B e3 ACCEPT ACCEPT  ~ not guard-discriminating on S1"
  "1B e4 ACCEPT ACCEPT  ⭐ FLIP vs 1A — admission"
  "1B e5 ACCEPT ACCEPT  ~ not guard-discriminating on S1 — PLANT-PROVEN, see the README"
  "1B e6 ACCEPT ACCEPT  ⭐ FLIP vs 1A — admission"
  "1B e7 ACCEPT ACCEPT  ⭐⭐ THE CALL-SITE-SCOPING ROW: the residual-free holder is UNHARMED (g4 REJECTS it)"

  # ⛔⛔ THE INTERP COLUMN HERE RUNS THE OTHER WAY, AND THAT IS .14's TITLE ("in BOTH directions").
  # On S1 the interpreter ACCEPTs what the generated parser refuses; on S2 it REJECTs what the
  # generated parser accepts. The one difference between the grammars is S1's second entry
  # alternative: the interpreter has no cycle guard, only a whole-stack DEPTH ceiling
  # (`parse_harness_interpreter.rs:746-749`), and under the longest_match default it must evaluate
  # the cyclic alternative to compare it — on S1 the surviving second alternative rescues the parse,
  # on S2 there is nothing to rescue it. ⛔ These six verdicts are a PIN OF A KNOWN-DIVERGENT ORACLE,
  # not a design claim: nothing in the design says what a cycle-guardless interpreter does at its
  # ceiling. Every design claim in this bank lives in the GEN column.
  "2A e1 ACCEPT REJECT  ⛔⛔ .14 IN THE OTHER DIRECTION: GEN accepts, the interpreter hits its depth ceiling"
  "2A e2 REJECT REJECT  the cycle guard again — here the two oracles happen to AGREE, for different reasons"
  "2A e3 ACCEPT REJECT  ⛔ a comment does not change either verdict"
  "2A e4 REJECT REJECT  and the chained case agrees too"
  "2A e5 ACCEPT REJECT  ⛔ the seeded single cast: GEN reaches it, the interpreter does not"
  "2A e6 REJECT REJECT  the seeded chain fails on both"

  "2B e1 ACCEPT ACCEPT  ⭐⭐ THE LOOP-GUARD ROW: the star must stop at zero iterations or the holder starves"
  "2B e2 ACCEPT ACCEPT  ⭐⭐ FLIP vs 2A, and the loop must still take ONE iteration"
  "2B e3 ACCEPT ACCEPT  ⭐ and a comment at the boundary does not slip the structural guard"
  "2B e4 ACCEPT ACCEPT  ⭐⭐ FLIP vs 2A, with a comment mid-chain"
  "2B e5 ACCEPT ACCEPT  ⭐⭐ THE TRAILING-GUARD ROW: the over-long SEED is refused, so kw wins the tournament"
  "2B e6 ACCEPT ACCEPT  ⭐⭐ FLIP vs 2A, seeded chain plus one iteration"
)

# ---- the structural table: what the EMITTED PARSER contains, read off the artifact rather than off
# the plan (`.17` slice 7 RESULT 4 — a report about a thing must be computed from that thing).
# <arm> <probe> <WANT> <note>
STRUCT=(
  "1A guard_rule_count 0  the shipped admission emits no guard rule — 0/3 candidates are starvation-safe"
  "1A eliminated       0  and absorbs nothing, so the cycle survives into the parser"
  "1A banner           0  the opt-in warning does not fire on the shipped path"
  "1B guard_rule_count 3  prim_lr_guard0 + its suffix + the HOP CLONE"
  "1B eliminated       1  'prim' is absorbed — the base rule now has the lr_base/lr_suffix shape"
  "1B hop_clone        1  ⭐⭐ .17 acceptance (d): prim_lr_guard0_ct EXISTS in the emitted parser"
  "1B banner           1  the opt-in warning fires, at DEFAULT verbosity"
  "2A guard_rule_count 0  same on the single-holder grammar"
  "2A eliminated       0  and nothing is absorbed there either"
  "2B guard_rule_count 3  the SAME three rules — dropping a holder changes no guard"
  "2B eliminated       1  'prim' is absorbed"
  "2B hop_clone        1  ⭐⭐ the hop clone again, on the grammar whose rows can falsify it"
)

# ⭐ Restore the scratch slot on ANY exit, including failure or Ctrl-C, and REGENERATE — the fixture
# alone is half the slot (`.13` slice 4b). ⛔ This bank also rebuilds the PROBE BINARY, which the
# sibling banks do not: `parseability_probe` compiles the scratch parser IN, so a restored artifact
# with a stale binary is a third inconsistent state. Costs one build; leaves no trap.
restore_scratch() {
  [[ -n "${ARM_RAN:-}" ]] || return 0
  git checkout -- "$SCRATCH" 2>/dev/null || return 0
  if ! make -C rust SHELL=/bin/bash focus_scratch >"$WORK/restore_focus.log" 2>&1; then
    echo "⛔ scratch slot RESTORE-REGENERATE failed — $SCRATCH_PARSER still holds this probe's" >&2
    echo "   grammar. Re-run by hand before committing anything:" >&2
    echo "     make -C rust SHELL=/bin/bash focus_scratch    # log: $WORK/restore_focus.log" >&2
    return 0
  fi
  (cd rust && cargo build --features generated_parsers --bin parseability_probe) \
    >"$WORK/restore_probe.log" 2>&1 || {
    echo "⛔ probe REBUILD after restore failed — $PROBE_BIN still holds this probe's grammar:" >&2
    echo "     (cd rust && cargo build --features generated_parsers --bin parseability_probe)" >&2
  }
}
trap restore_scratch EXIT

gen_verdict() {  # $1 = input file
  local out
  out="$("$PROBE_BIN" --parse scratch "$1" 2>&1)"
  [[ "$out" == *"parse_full passed"* ]] && echo "ACCEPT" || echo "REJECT"
}

interp_verdict() {  # $1 = grammar file, $2 = input file, $3… = extra ast_pipeline flags
  local grammar="$1" input="$2"; shift 2
  local line
  line="$("$AST_PIPELINE" "$grammar" --interpret-parse "$input" "$@" 2>/dev/null \
          | grep -m1 '^INTERPRET-PARSE:')"
  [[ -z "$line" ]] && { echo "NOVERDICT"; return; }
  [[ "$line" == *"accepted=true"* ]] && echo "ACCEPT" || echo "REJECT"
}

# ⛔ Counted off the EMITTED PARSER, by rule-function name. `_lr_guard` cannot appear in a generated
# parser by any route but this emission — the string is allocated by `plan_guard_chains`.
#
# ⛔⛔ THE ANCHOR IS `pub fn`, AND THE FIRST DRAFT OF IT WAS `fn`. Every rule function the codegen
# emits is `pub fn parse_<rule>(`, so an `^[[:space:]]*fn parse_` anchor matched NOTHING and reported
# `guard_rule_count=0` on the arm that has three of them. It was caught in one run because the row
# DECLARES 3 — which is the entire reason this bank carries structural expectations instead of
# printing what it found ([[a-report-scraper-must-anchor-on-structure-not-on-a-substring]], and the
# fourth measured instance in this leaf).
# ⛔ `|| true` and not `|| echo 0`: `grep -c` prints its zero AND exits 1, so the fallback would
# append a SECOND line and every comparison would fail against a two-line value.
count_matching() { grep -c -E "$1" "$SCRATCH_PARSER" 2>/dev/null || true; }
guard_rule_count() { count_matching '^[[:space:]]*pub fn parse_[a-z_0-9]*_lr_guard[a-z_0-9]*\('; }
eliminated_count()  { count_matching '^[[:space:]]*pub fn parse_prim_lr_base\('; }
hop_clone_present() {
  grep -q -E '^[[:space:]]*pub fn parse_prim_lr_guard0_ct\(' "$SCRATCH_PARSER" 2>/dev/null \
    && echo 1 || echo 0
}

# ⛔⛔ THE TABLES ARE DOUBLE-QUOTED SHELL STRINGS, SO A BACKTICK IN A NOTE IS A COMMAND. Measured the
# hard way: the note *"the `*` must stop at zero iterations"* made bash run the command `*`, which
# globbed to the first file in the repository root and reported `AGENTS.md: command not found` — and
# the row's note silently became that filename. It fails LOUDLY here only because the substitution
# happened to be nonsense; `$HOME` or `$(date)` would have substituted quietly and left a table whose
# printed notes are not the notes anyone wrote.
#
# The check reads THIS FILE rather than the arrays, deliberately: by the time an array element
# exists, the substitution has already happened and the evidence is gone.
#
# ⛔ COMMENT lines inside the array are SKIPPED, and that exemption is measured rather than assumed:
# `#` at the start of a word inside an array literal begins a real shell comment, so nothing in it is
# substituted. The first version of this check had no exemption and refused to start over a backtick
# in its own explanatory comment — a self-check that cannot describe itself is one people delete.
rows_with_substitution() {
  awk '/^(CASES|STRUCT)=\(/,/^\)/' "${BASH_SOURCE[0]}" \
    | grep -vE '^[[:space:]]*#' \
    | grep -nE '`|\$\(|\$\{'
}
if [[ -n "$(rows_with_substitution)" ]]; then
  echo "⛔ a CASES/STRUCT row contains a backtick or a \$ expansion — bash will SUBSTITUTE it and" >&2
  echo "   the row's note will not be the note that was written. Remove it:" >&2
  rows_with_substitution >&2
  exit 2
fi

declare -A MEASURED_GEN MEASURED_INTERP MEASURED_STRUCT

# Load one grammar into the scratch slot and generate its parser under one admission.
#  $1 = arm key (e.g. 1A), $2 = grammar file, $3 = "shipped" | "widened"
run_arm() {
  local arm="$1" grammar="$2" mode="$3"
  ARM_RAN=1
  cp "$grammar" "$SCRATCH" || return 1
  # ⛔ `make focus_scratch` FIRST in every arm, even the widened one: it is what regenerates
  # `generated/scratch.json` from the fixture. The widened arm then re-runs ONLY the generator step
  # on that same JSON, so the two arms of a grammar compile the identical front-end output and the
  # admission is provably the only difference.
  make -C rust SHELL=/bin/bash focus_scratch >"$WORK/arm$arm.focus.log" 2>&1 || {
    echo "focus_scratch FAILED for arm $arm — see $WORK/arm$arm.focus.log" >&2; return 1; }
  if [[ "$mode" == widened ]]; then
    # This is `rust/Makefile`'s RUST_GENERATOR line (:93) plus one flag.
    "$AST_PIPELINE" "$SCRATCH_JSON" --generate-parser --debug --trace --eliminate-left-recursion \
      "$WIDEN" -o "$SCRATCH_PARSER" >"$WORK/arm$arm.gen.log" 2>"$WORK/arm$arm.gen.err" || {
      echo "widened generation FAILED for arm $arm — see $WORK/arm$arm.gen.err" >&2; return 1; }
  fi
  (cd rust && cargo build --features generated_parsers --bin parseability_probe) \
    >"$WORK/arm$arm.build.log" 2>&1 || {
    echo "probe rebuild FAILED for arm $arm — see $WORK/arm$arm.build.log" >&2; return 1; }

  # ⛔ Written as two explicit calls rather than an array splat: `"${flags[@]}"` on an EMPTY array
  # under `set -u` is a hard error on bash 3.2, which is what `/bin/bash` still is on macOS.
  local spec case_arm input
  for spec in "${CASES[@]}"; do
    read -r case_arm input _wg _wi _note <<<"$spec"
    [[ "$case_arm" == "$arm" ]] || continue
    MEASURED_GEN[$arm:$input]="$(gen_verdict "$WORK/$input.txt")"
    if [[ "$mode" == widened ]]; then
      MEASURED_INTERP[$arm:$input]="$(interp_verdict "$grammar" "$WORK/$input.txt" "$WIDEN")"
    else
      MEASURED_INTERP[$arm:$input]="$(interp_verdict "$grammar" "$WORK/$input.txt")"
    fi
  done
  MEASURED_STRUCT[$arm:guard_rule_count]="$(guard_rule_count)"
  MEASURED_STRUCT[$arm:eliminated]="$(eliminated_count)"
  MEASURED_STRUCT[$arm:hop_clone]="$(hop_clone_present)"
  # ⛔ The banner probe is what proves the two INTERP columns are DIFFERENT RUNS. Both read ACCEPT on
  # every row, so without it a bank that silently dropped the flag would look identical.
  local banner_stderr
  if [[ "$mode" == widened ]]; then
    banner_stderr="$("$AST_PIPELINE" "$grammar" --interpret-parse "$WORK/e1.txt" "$WIDEN" 2>&1 1>/dev/null)"
  else
    banner_stderr="$("$AST_PIPELINE" "$grammar" --interpret-parse "$WORK/e1.txt" 2>&1 1>/dev/null)"
  fi
  if grep -q "GUARD-FEASIBLE ADMISSION IS ON" <<<"$banner_stderr"; then
    MEASURED_STRUCT[$arm:banner]=1
  else
    MEASURED_STRUCT[$arm:banner]=0
  fi
}

echo "ENGINE-UNIVERSAL-SERVICES.17 slice 8 — does the EMITTED guard parse?"
echo

run_arm 1A "${GRAMMAR[1]}" shipped || exit 2
run_arm 1B "${GRAMMAR[1]}" widened || exit 2
run_arm 2A "${GRAMMAR[2]}" shipped || exit 2
run_arm 2B "${GRAMMAR[2]}" widened || exit 2

# ================= the verdict table =================
fail=0
printf '  %-4s %-4s %-8s %-8s %-8s %-8s %s\n' ARM IN GEN WANT INTERP WANT ''
for spec in "${CASES[@]}"; do
  read -r arm input want_gen want_interp note <<<"$spec"
  gen="${MEASURED_GEN[$arm:$input]:-MISSING}"
  interp="${MEASURED_INTERP[$arm:$input]:-MISSING}"
  mark="✅"
  [[ "$gen" != "$want_gen" ]] && { mark="⛔"; fail=1; }
  [[ "$interp" != "$want_interp" ]] && { mark="⛔"; fail=1; }
  [[ "$want_gen" != "$want_interp" && "$mark" == "✅" ]] && mark="✅ DIVERGE(.14)"
  printf '  %-4s %-4s %-8s %-8s %-8s %-8s %s  %s\n' \
    "$arm" "$input" "$gen" "$want_gen" "$interp" "$want_interp" "$mark" "$note"
done

echo
printf '  %-4s %-18s %-6s %-6s %s\n' ARM PROBE GOT WANT ''
for spec in "${STRUCT[@]}"; do
  read -r arm probe want note <<<"$spec"
  got="${MEASURED_STRUCT[$arm:$probe]:-MISSING}"
  mark="✅"
  [[ "$got" != "$want" ]] && { mark="⛔"; fail=1; }
  printf '  %-4s %-18s %-6s %-6s %s  %s\n' "$arm" "$probe" "$got" "$want" "$mark" "$note"
done

echo
# ⛔ DERIVED, never stored (`docs/DERIVED_STATE_CONTAINMENT.md` R1/R3): every count below is read off
# the tables at run time. A prose count of a table is a claim with no gate behind it.
parse_rows=${#CASES[@]}
struct_rows=${#STRUCT[@]}
want_accept=0; want_reject=0; diverge=0; flips=0
declare -A GEN_BY_ROW
for spec in "${CASES[@]}"; do
  read -r arm input want_gen want_interp _note <<<"$spec"
  for w in "$want_gen" "$want_interp"; do
    [[ "$w" == ACCEPT ]] && want_accept=$((want_accept + 1))
    [[ "$w" == REJECT ]] && want_reject=$((want_reject + 1))
  done
  [[ "$want_gen" != "$want_interp" ]] && diverge=$((diverge + 1))
  GEN_BY_ROW[$arm:$input]="$want_gen"
done
for g in 1 2; do
  for i in 1 2 3 4 5 6 7; do
    [[ "${GEN_BY_ROW[${g}A:e$i]:-}" == REJECT && "${GEN_BY_ROW[${g}B:e$i]:-}" == ACCEPT ]] \
      && flips=$((flips + 1))
  done
done
printf 'ground truth, both directions: %d parse rows x2 oracles — %d ACCEPT, %d REJECT; %d structural rows\n' \
  "$parse_rows" "$want_accept" "$want_reject" "$struct_rows"
printf 'GEN flips A REJECT -> B ACCEPT: %d   ·   declared oracle divergences (.14): %d\n' \
  "$flips" "$diverge"

if [[ $((want_accept + want_reject)) -ne $((parse_rows * 2)) ]]; then
  echo "⛔ a CASES row declares a verdict that is neither ACCEPT nor REJECT — the table is malformed" >&2
  fail=1
fi
if [[ $want_accept -eq 0 || $want_reject -eq 0 ]]; then
  echo "⛔ the bank has lost one of its two directions — a one-sided bank proves nothing" >&2
  fail=1
fi
if [[ $flips -eq 0 ]]; then
  echo "⛔ NO ROW FLIPS between the arms — this bank would then be describing a grammar, not" >&2
  echo "   measuring an admission. The flip rows are the finding." >&2
  fail=1
fi
# ⛔ The S2 arm is what makes any row able to falsify a GUARD defect, and a bank that lost it would
# still print a green table full of S1 rows. Refuse to pass without it.
if [[ -z "${MEASURED_GEN[2B:e5]:-}" || -z "${MEASURED_GEN[2B:e1]:-}" ]]; then
  echo "⛔ the SINGLE-HOLDER grammar (S2) did not run — without it e1/e5 are absorbed by S1's" >&2
  echo "   second alternative and no row can fail when a guard position is deleted." >&2
  fail=1
fi

if [[ $fail -eq 0 ]]; then
  echo "GUARD-PARSES: $((parse_rows * 2 + struct_rows))/$((parse_rows * 2 + struct_rows)) as declared — PGEN's OWN emitted guard chain"
  echo "  compiles to a parser that accepts every input on both grammars: the LOOP guard row (2B e1),"
  echo "  the TRAILING guard row (2B e5), and the CALL-SITE-SCOPING row (1B e7, which the shared-rule"
  echo "  shape rejects). The HOP CLONE prim_lr_guard0_ct is in both emitted parsers, and $flips GEN"
  echo "  rows flip REJECT -> ACCEPT against the shipped admission."
else
  echo "GUARD-PARSES: MISMATCH — a row disagrees with the verdict .17 slice 8 recorded." >&2
  echo "  Do NOT adjust the expectations to match; the leaf's decision rests on them." >&2
fi
exit "$fail"
