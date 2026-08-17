#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.30 (b) — the four published ratios in `write_report` survive a zero
# denominator, and the two that were REACHABLE are pinned by the fixtures that reached them.
#
# `write_report` published four percentages with unguarded denominators. Two were genuinely reached
# and killed the instrument with a Python traceback and exit 1 — which violates this file's own
# docstring contract that it *"refuses (exit 2) rather than reporting a clean measurement it could
# not take"*. It did not refuse; it crashed.
#
#   / lr_total      REACHED by `module m; endmodule` — the simplest legal SystemVerilog file has no
#                   left-recursion-family entries. An EMPTY file reaches it too.
#   / acc_entries   REACHED by any sample whose every file is REJECTED.
#   / total  (x2)   NOT reached today: `run_measure` refuses an empty `rows`, and every dumping file
#                   contributes >= 1 entry (measured: a 0-byte file yields 392).
#
# ⭐ THE FIX IS `n/a`, NOT A REFUSAL. Both reached cases are LEGITIMATE measurements — the binding
# counters are fully measured and only a derived ratio is undefined. Refusing would make the
# instrument unusable on exactly the small ad-hoc samples this campaign runs.
#
# HOW:  bash docs/tasks/artifacts/engine_universal_services/es30_zero_denominator/probe.sh
# COST: three 1-file measurements, ~15 s.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT"

INSTR="stimuli/sv/corpus_parse_cost.py"
PROBE="rust/target/release/parseability_probe"
WORK="rust/target/es30_zero_denominator"

fails=0; arms=0
pass() { arms=$((arms+1)); printf '  ✓ %s\n' "$1"; }
fail() { arms=$((arms+1)); fails=$((fails+1)); printf '  ✗ %s\n' "$1" >&2; }

[ -x "$PROBE" ] || { printf 'es30-probe: missing %s — build it first:\n  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)\n' "$PROBE" >&2; exit 2; }

rm -rf "$WORK"; mkdir -p "$WORK"
printf 'module m;\nendmodule\n'                              > "$WORK/trivial.sv"   # accepted, 0 LR entries
printf 'module m;\n  this is not systemverilog @@@ ;\nendmodule\n' > "$WORK/bad.sv"  # rejected
: > "$WORK/empty.sv"                                                                 # accepted, 0 LR entries
for n in trivial bad empty; do printf 'hot\t%s/%s.sv\n' "$WORK" "$n" > "$WORK/m_$n.tsv"; done

printf '\nES30-ZERO-DENOMINATOR: a published ratio must degrade to n/a, never to a traceback\n'
printf 'commit : %s\n\n' "$(git log -1 --format=%h)"

# run <fixture> — measure one file; echo "<rc>|<cost.md written?>"
run() {
  local n="$1"
  timeout 300 python3 "$INSTR" --manifest "$WORK/m_$n.tsv" --outdir "$WORK/out_$n" \
    > "$WORK/$n.log" 2>&1
  printf '%s' "$?"
}

printf 'ARM 1  the two REACHED denominators produce a report, not a crash\n'
for n in trivial bad; do
  rc=$(run "$n")
  if [ "$rc" != 0 ]; then
    fail "$n.sv: instrument exited $rc (expected 0)"; tail -3 "$WORK/$n.log" | sed 's/^/      /' >&2
  elif ! [ -f "$WORK/out_$n/cost.md" ]; then
    fail "$n.sv: exited 0 but wrote no cost.md — the report must still be written"
  else
    pass "$n.sv: rc=0 and cost.md written ($(wc -c < "$WORK/out_$n/cost.md" | tr -d ' ') B)"
  fi
done

printf 'ARM 2  no traceback reaches the operator\n'
if grep -qE 'ZeroDivisionError|Traceback' "$WORK"/trivial.log "$WORK"/bad.log 2>/dev/null; then
  fail "a Python traceback survived — the contract is a diagnostic, never a stack trace"
else
  pass "neither run emitted ZeroDivisionError or a Traceback"
fi

printf 'ARM 3  the undefined ratio is DECLARED n/a, not silently omitted and not printed as 0\n'
# ⛔ `0.000 %` would read as a MEASURED result; an absent row is indistinguishable from an
# instrument that stopped measuring its subject. Both are refused here.
if grep -q 'family commit ratio is `n/a`' "$WORK/out_trivial/cost.md"; then
  pass "trivial.sv: the LR-family commit ratio is declared n/a with its reason"
else
  fail "trivial.sv: no n/a declaration for the LR-family ratio"
fi
if grep -q 'NO FILE IN THIS SAMPLE WAS ACCEPTED' "$WORK/out_bad/cost.md"; then
  pass "bad.sv: the accepted-only ratio is declared n/a and the report warns it is not a baseline"
else
  fail "bad.sv: no n/a declaration for the accepted-only ratio"
fi
# ⛔ AND THE CONVERSE, WHICH THIS ARM GOT WRONG ON ITS FIRST RUN: `lr_total / total` is NOT
# undefined when the family is empty — it is a well-defined ratio whose value is genuinely 0.
# Printing `0.000 %` there is CORRECT, and demanding `n/a` for it would have made the instrument
# claim ignorance about something it measured exactly. Only `lr_committed / lr_total` is undefined.
# The arm therefore asserts the opposite of what it first asserted: the share row must be numeric.
if grep -qE '\| family share of all entries in this sample \| 0\.000 % \|' "$WORK/out_trivial/cost.md"; then
  pass "the family SHARE stays numeric (0.000 %) — it is measured, not undefined"
else
  fail "the family share row is not the expected numeric 0.000 % — an exactly-measured 0 must not become n/a"
fi

printf 'ARM 4  an EMPTY file is measurable too, so the two `/ total` sites stay unreachable\n'
rc=$(run empty)
ent=$(grep -oE 'entries=[0-9,]+' "$WORK/empty.log" | head -1)
if [ "$rc" = 0 ] && [ -n "$ent" ]; then
  pass 'empty.sv: rc=0, '"$ent"' — a 0-byte file still enters rules, so `total` is never 0 while a row exists'
else
  fail "empty.sv: rc=$rc ${ent:-(no entry count)}"
fi

printf 'ARM 5  RED CONTROL — the pre-fix instrument must CRASH on the same fixtures\n'
# A control never observed failing is not known to work (`docs/CLAIM_VERIFICATION.md` §3 leg 2).
#
# ⛔⛔ IT USES THE REAL PRE-FIX CODE FROM GIT, NOT A SYNTHETIC MUTANT, AND THE FIRST ATTEMPT PROVED
# WHY. Reverting `pct()`'s guard alone (`if not denominator:` -> `if False:`) exits **0**, because
# the fix has TWO layers: the guard AND the `if lr_total:` branch at the call site. A single-point
# mutant therefore reports "the fixtures do not reach the defect" while the defect is perfectly
# reachable — a control that disarms itself and looks like evidence. Replaying the historical blob
# cannot drift out of step with the fix, however many layers the fix has.
#
# ⛔ THE REPLAY MUST LIVE TWO DIRECTORIES BELOW THE REPO ROOT. The instrument computes
# `ROOT = dirname(__file__)/../..`, so a copy under `rust/target/...` resolves ROOT to `rust/` and
# dies at exit 2 on missing paths — indistinguishable from "not reachable". Measured, same run.
MUT="stimuli/sv/.es30_prefix_replay.py"
trap 'rm -f "$ROOT/stimuli/sv/.es30_prefix_replay.py"' EXIT
NEEDLE='100.0 \* lr_committed / lr_total'
PREFIX_COMMIT=""
for c in $(git rev-list HEAD -- "$INSTR"); do
  if git show "$c:$INSTR" 2>/dev/null | grep -qE "$NEEDLE"; then PREFIX_COMMIT="$c"; break; fi
done
if [ -z "$PREFIX_COMMIT" ]; then
  # LOUDLY, never as a pass: an unreachable history is not evidence that the fix works.
  printf '  ⚠️  NOT EVALUATED — no commit in this history still carries the unguarded expression\n'
  printf '      (shallow clone?). Arm 5 is the only proof the fixtures reach the defect at all.\n'
  fails=$((fails + 1)); arms=$((arms + 1))
else
  git show "$PREFIX_COMMIT:$INSTR" > "$MUT"
  timeout 300 python3 "$MUT" --manifest "$WORK/m_trivial.tsv" --outdir "$WORK/out_replay" \
    > "$WORK/replay.log" 2>&1
  mrc=$?
  if [ "$mrc" != 0 ] && grep -q 'ZeroDivisionError' "$WORK/replay.log"; then
    pass "the PRE-FIX instrument ($(git log -1 --format=%h "$PREFIX_COMMIT")) dies with ZeroDivisionError (rc=$mrc) — these fixtures DO reach the defect"
  else
    fail "the pre-fix replay exited $mrc without ZeroDivisionError — the fixtures do not reach the defect, so arms 1-3 prove nothing"
  fi
fi

printf '\n'
if [ "$fails" -eq 0 ]; then
  printf 'ES30-ZERO-DENOMINATOR: %d/%d arms as declared\n' "$arms" "$arms"
else
  printf 'ES30-ZERO-DENOMINATOR: %d of %d ARMS FAILED\n' "$fails" "$arms" >&2
fi
exit "$fails"
