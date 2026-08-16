#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_gate_reachability_string_reader_probes.sh
#
# CI-PARITY-GATE-ROT.34 — RED / GREEN arms for the string-vs-command reader in
# `scripts/check_gate_reachability.sh`.
#
# ⭐ WHAT IS BEING PROVED, AND WHY IT NEEDS A DRIVER AT ALL. `.34`'s defect was invisible in the
# PASSING direction: a make target named only inside a multi-line error STRING was certified
# reachable in the strongest class (`git-hook`), and all eight pre-existing ground-truth controls
# stayed green while it was. The fix therefore ships six SYNTAX_CONTROLS arms — but a control nobody
# has watched fail is a control nobody can trust (`feedback_instrument_needs_ground_truth`). This
# driver makes each of them go RED on demand.
#
# ⛔ THE READER IS MUTATED, NEVER RE-TYPED. `GENERATED-LINT-CORRECTNESS.4` found a probe driver that
# had hand-copied the rule it verified, so correcting the rule left the driver measuring the stale
# one. Here every arm is a surgical `sed` against a COPY of the live gate script, so a future change
# to the reader is what this driver tests — it cannot drift into checking a private second copy.
#
# The arms, each naming the control it must break:
#   BASE   unmutated                      → the six SYNTAX_CONTROLS all pass (only the register
#                                            disposition decides the exit code)
#   RED-1  suppression disabled entirely  → reproduces the PRE-FIX reader: the two RED controls
#                                            (multi-line message, heredoc payload) must both fail
#   RED-2  single-quote state dropped     → the multi-line `'…'` program control must fail
#   RED-3  `$(` re-entry inside "…" dropped → the command-substitution control must fail
#   RED-4  heredoc tracking dropped       → the heredoc control must fail
#   DIFF   HEAD vs working tree           → the full 125-target inventory moves EXACTLY one target
#
# Usage: bash docs/tasks/artifacts/ci_parity_gate_rot/run_gate_reachability_string_reader_probes.sh
# Exit 0 iff every arm reaches its expected verdict.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

GATE="scripts/check_gate_reachability.sh"
[ -f "$GATE" ] || { echo "probes: ✗ gate not found: $GATE" >&2; exit 1; }

WORK="rust/target/ci_parity_gate_rot_probe/gate_reachability_string_reader"
rm -rf "$WORK"; mkdir -p "$WORK"

rc=0 arms=0 red_fired=0
pass() { arms=$((arms + 1)); printf '  ✅ %s\n' "$1"; }
fail() { arms=$((arms + 1)); printf '  ❌ %s\n' "$1" >&2; rc=1; }

# A mutated copy must still resolve ROOT to the repository, not to its own directory.
mutant() {
  local name="$1"; shift
  local out="$WORK/$name.sh"
  sed -e 's|ROOT="$(cd "$(dirname "${BASH_SOURCE\[0\]}")/.." \&\& pwd)"|ROOT="'"$ROOT"'"|' "$GATE" > "$out"
  local expr
  for expr in "$@"; do
    sed -i.bak -e "$expr" "$out" && rm -f "$out.bak"
  done
  printf '%s' "$out"
}

# Assert that running MUTANT reports a control failure whose text contains NEEDLE.
expect_control_failure() {
  local label="$1" script="$2" needle="$3"
  local log="$script.log"
  bash "$script" >/dev/null 2>"$log"
  if ! grep -q "GROUND-TRUTH CONTROLS FAILED" "$log"; then
    fail "$label — the mutated reader did NOT trip the controls (they cannot fire; log: $log)"
    return
  fi
  if ! grep -qF "$needle" "$log"; then
    fail "$label — controls tripped, but not the expected arm '$needle' (log: $log)"
    return
  fi
  red_fired=$((red_fired + 1))
  pass "$label — control fired: $needle"
}

echo "== BASE — the live reader: the six syntax arms must all pass =="
BASE_LOG="$WORK/base.log"
bash "$GATE" >/dev/null 2>"$BASE_LOG"
if grep -q "GROUND-TRUTH CONTROLS FAILED" "$BASE_LOG"; then
  fail "BASE — the live reader fails its own controls (log: $BASE_LOG)"
else
  pass "BASE — no control failure from the unmutated reader"
fi

echo "== RED-1 — suppression disabled (this IS the pre-fix reader) =="
M1="$(mutant red1 's|    data = string_data_lines(text) if shell_syntax else set()|    data = set()|')"
expect_control_failure "RED-1a multi-line error message" "$M1" \
  "RED — a target named only in a multi-line error message is NOT an invocation"
expect_control_failure "RED-1b heredoc payload" "$M1" \
  "RED — a target named only inside a heredoc body is payload, not an invocation"

echo "== RED-2 — single-quote state dropped =="
M2="$(mutant red2 's|^            if c == "'"'"'":$|            if False:|')"
expect_control_failure "RED-2 multi-line single-quoted program" "$M2" \
  "GREEN — a multi-line single-quoted program does not swallow the command after it"

echo "== RED-3 — \$( re-entry inside a double-quoted string dropped =="
# 16-space indent: the one INSIDE the `top == "dq"` branch, not its NORMAL-context twin.
M3="$(mutant red3 's|^                elif c == "\$" and line\[i + 1:i + 2\] == "(":$|                elif False:|')"
expect_control_failure "RED-3 multi-line command substitution" "$M3" \
  "GREEN — the body of a multi-line command substitution still executes"

echo "== RED-4 — heredoc tracking dropped =="
M4="$(mutant red4 's|^            elif line.startswith("<<", i):$|            elif False:|')"
expect_control_failure "RED-4a heredoc body desyncs what follows" "$M4" \
  "GREEN — a heredoc body does not swallow the command after it"
expect_control_failure "RED-4b heredoc payload becomes an invocation" "$M4" \
  "RED — a target named only inside a heredoc body is payload, not an invocation"

echo "== DIFF — the whole inventory moves EXACTLY one target =="
HEADV="$WORK/head_version.sh"
if git show HEAD:"$GATE" > "$HEADV" 2>/dev/null && [ -s "$HEADV" ]; then
  sed -i.bak -e 's|ROOT="$(cd "$(dirname "${BASH_SOURCE\[0\]}")/.." && pwd)"|ROOT="'"$ROOT"'"|' "$HEADV"
  rm -f "$HEADV.bak"
  bash "$HEADV" --report > "$WORK/report_head.txt" 2>/dev/null
  bash "$GATE"   --report > "$WORK/report_live.txt" 2>/dev/null
  moved=$(diff <(grep -E '^  [a-z0-9_-]+ ' "$WORK/report_head.txt" | awk '{print $1, $2}') \
               <(grep -E '^  [a-z0-9_-]+ ' "$WORK/report_live.txt" | awk '{print $1, $2}') \
          | grep -cE '^[<>]')
  if [ "$moved" = "0" ]; then
    pass "DIFF — HEAD already carries the fix (no rows differ); re-run before committing to see the move"
  elif [ "$moved" = "2" ]; then
    pass "DIFF — exactly one target changed classification (one '<' row + one '>' row)"
    diff "$WORK/report_head.txt" "$WORK/report_live.txt" | grep -E '^[<>].*generated_reproducibility' || true
  else
    fail "DIFF — expected exactly one target to move, saw $moved changed rows"
    diff "$WORK/report_head.txt" "$WORK/report_live.txt" | head -20 >&2
  fi
else
  pass "DIFF — skipped: HEAD has no $GATE yet (first landing)"
fi

echo
# ⭐ ONE GREPPABLE VERDICT LINE, in the house style of `CERTIFICATE-COVERAGE:` / `INTERPRET-PARSE:`.
# The token `GATE-REACHABILITY-PROBE:` is registered in `scripts/check_diagnosis_evidence.sh`
# (`DIAGNOSIS_SIG`, ops/build-flow family) and in `TOOLBOX.md`'s family table, in the commit that
# landed this file — an instrument whose token the gate cannot see leaves a leaf root-caused with it
# no honest way to cite it.
printf 'GATE-REACHABILITY-PROBE: arms=%d red_arms_fired=%d verdict=%s\n' \
  "$arms" "$red_fired" "$([ "$rc" = 0 ] && echo ALL-EXPECTED || echo MISMATCH)"
[ "$rc" = 0 ] || echo "gate-reachability string-reader probes: ✗ at least one arm did not" >&2
exit "$rc"
