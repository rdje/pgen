#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_failure_path_probes.sh
#
# CI-PARITY-GATE-ROT.14 — RED/GREEN/CONTROL arms for the aggregate's failure path.
#
# THE INCIDENT. Acceptance run 3 (5 h 05 m) ended with:
#     ==> regex_parser_family_status_gate (required)
#         fail (…/logs/regex_parser_family_status_gate.log)
#     jq: error: Could not open file …/work/regex_parser_family_status_gate/summary.json
# The last error a triager reads is a MISSING FILE, four lines below the real cause, looking
# exactly like the artifact-hand-off class `.7` fixed. This tree has already lost a session to one
# misdirected diagnosis (`-0015`); that one was a human reading, this one is built into the tool.
#
# ⭐ THE ROOT CAUSE IS SHARPER THAN "IT READS THE SUMMARY AFTER A FAILURE": the guard tested a
# DIFFERENT FILE from the one the else-branch reads, and tested EXISTENCE where it needed CONTENT.
# The failed gate left a **0-byte** `summary.txt` and no `summary.json`, so `[[ ! -f summary.txt ]]`
# was FALSE, the else-branch ran, and `jq` died on a file nothing had written.
#
# ⛔ THE GUARDS ARE EXTRACTED FROM THE LIVE SCRIPT (and the retired form from `git show`), so these
# arms cannot test a rule the gate does not apply.
#
# Usage: bash docs/tasks/artifacts/ci_parity_gate_rot/run_failure_path_probes.sh [BEFORE_REV]
# Exit 0 iff every arm reaches its expected verdict.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

GATE="rust/scripts/sota_exit_gate.sh"
# ⛔ PIN THE PRE-FIX SHA, NEVER `HEAD`. The first cut defaulted to HEAD and passed 11/11 — then
# failed the moment the fix was committed, because HEAD had become the FIXED revision and CTRL-4's
# "the sweep must find 6" no longer held. A probe whose baseline moves with the branch is not a
# baseline. (Same trap already fixed in run_doctrine_lane_census.sh; not applied here until it bit.)
BEFORE_REV="${1:-de94af5e}"
WORK="rust/target/ci_parity_gate_rot_probe/failure_path"
rm -rf "$WORK"; mkdir -p "$WORK"

pass=0; fail=0
ok()  { pass=$((pass+1)); printf '✓ %-52s %s\n' "$1" "$2"; }
no()  { fail=$((fail+1)); printf '✗ %-52s %s\n' "$1" "$2"; }

printf '%s\n' "=============================================================================="
printf 'CI-PARITY-GATE-ROT.14 — aggregate failure-path probes\n'
printf '%s\n' "=============================================================================="

# ---------------------------------------------------------------- the guard forms, both extracted
# LIVE form, read out of the gate itself. RETIRED form, read out of git. Neither is retyped here.
LIVE_GUARD="$(grep -m1 -oE '! -s "\$[A-Z0-9_]+_SUMMARY_TXT" \|\| ! -s "\$[A-Z0-9_]+_SUMMARY_JSON"' "$GATE" || true)"
OLD_GUARD="$(git show "$BEFORE_REV:$GATE" 2>/dev/null | grep -m1 -oE '! -f "\$[A-Z0-9_]+_SUMMARY_TXT"' || true)"
if [ -z "$LIVE_GUARD" ]; then
  no "EXTRACT live guard" "not found in $GATE — the probe cannot test a rule it cannot read"
  printf 'arms=1  PASS=0  FAIL=1\n'; exit 1
fi
ok "EXTRACT live guard from the gate" "$LIVE_GUARD"
if [ -z "$OLD_GUARD" ]; then
  ok "EXTRACT retired guard from $BEFORE_REV" "absent (already fixed at that rev) — arms below skip the before side"
else
  ok "EXTRACT retired guard from $BEFORE_REV" "$OLD_GUARD"
fi

# `run_guard <extracted-form> <txt> <json>` — evaluates the REAL guard expression with the two
# artifact paths substituted, and reports which branch the aggregate would take:
#   MISSING-FALLBACK = the guard fires, the <missing> defaults are used, jq is NEVER reached
#   JQ-READ          = the else-branch runs and jq is invoked on <json>
run_guard() {
  local form="$1" txt="$2" json="$3" expr
  expr="$(printf '%s' "$form" | sed -E 's/\$[A-Z0-9_]+_SUMMARY_TXT/$txt/g; s/\$[A-Z0-9_]+_SUMMARY_JSON/$json/g')"
  if eval "[[ $expr ]]"; then printf 'MISSING-FALLBACK\n'; else printf 'JQ-READ\n'; fi
}

# ---------------------------------------------------------------- the states, built not described
mk() { local d="$WORK/$1"; rm -rf "$d"; mkdir -p "$d"; }

# STATE A — the observed incident: a 0-byte summary.txt, no summary.json.
mk died_midrun; : > "$WORK/died_midrun/summary.txt"
A_TXT="$WORK/died_midrun/summary.txt"; A_JSON="$WORK/died_midrun/summary.json"

# STATE B — a healthy completed sub-gate: both artifacts present and non-empty.
mk healthy; printf 'state_dir: x\n' > "$WORK/healthy/summary.txt"; printf '{"gate":"x"}\n' > "$WORK/healthy/summary.json"
B_TXT="$WORK/healthy/summary.txt"; B_JSON="$WORK/healthy/summary.json"

# STATE C — nothing ran at all: neither artifact exists.
mk absent
C_TXT="$WORK/absent/summary.txt"; C_JSON="$WORK/absent/summary.json"

# RED-1 — the verbatim incident under the RETIRED guard: it reaches jq, which is the crash.
if [ -n "$OLD_GUARD" ]; then
  got="$(run_guard "$OLD_GUARD" "$A_TXT" "$A_JSON")"
  [ "$got" = "JQ-READ" ] && ok "RED-1  retired guard on the observed state" "reaches jq ⇒ the crash" \
                         || no "RED-1  retired guard on the observed state" "expected JQ-READ, got $got"
fi

# GREEN-1 — the same state under the LIVE guard: the <missing> fallback, jq never reached.
got="$(run_guard "$LIVE_GUARD" "$A_TXT" "$A_JSON")"
[ "$got" = "MISSING-FALLBACK" ] && ok "GREEN-1 live guard on the observed state" "fallback ⇒ jq never reached" \
                                || no "GREEN-1 live guard on the observed state" "expected MISSING-FALLBACK, got $got"

# CTRL-1 — ⛔ THE ARM THAT MATTERS: a HEALTHY sub-gate must behave IDENTICALLY under both forms.
# A guard that also changed the healthy path would be a silent behaviour change across 5 sites in a
# 5-hour aggregate — far worse than the crash it fixes.
live_h="$(run_guard "$LIVE_GUARD" "$B_TXT" "$B_JSON")"
if [ -n "$OLD_GUARD" ]; then
  old_h="$(run_guard "$OLD_GUARD" "$B_TXT" "$B_JSON")"
  [ "$live_h" = "JQ-READ" ] && [ "$old_h" = "JQ-READ" ] \
    && ok "CTRL-1 healthy state identical under both forms" "both JQ-READ ⇒ no behaviour change" \
    || no "CTRL-1 healthy state identical under both forms" "live=$live_h old=$old_h"
else
  [ "$live_h" = "JQ-READ" ] && ok "CTRL-1 healthy state still reads the summary" "JQ-READ" \
                            || no "CTRL-1 healthy state still reads the summary" "got $live_h"
fi

# CTRL-2 — nothing ran: both forms must take the fallback. The retired guard was not wrong HERE,
# which is exactly why the defect survived: it handled the obvious case and missed the real one.
live_a="$(run_guard "$LIVE_GUARD" "$C_TXT" "$C_JSON")"
[ "$live_a" = "MISSING-FALLBACK" ] && ok "CTRL-2 nothing-ran state takes the fallback" "MISSING-FALLBACK" \
                                   || no "CTRL-2 nothing-ran state takes the fallback" "got $live_a"

# RED-2 — the inverse half-written state: summary.json written, summary.txt empty. The retired form
# would ALSO pass this to the else-branch; the live form refuses. Included because a half-written
# pair can occur either way round and a fix for one direction only is half a fix.
mk inverse; : > "$WORK/inverse/summary.txt"; printf '{"gate":"x"}\n' > "$WORK/inverse/summary.json"
got="$(run_guard "$LIVE_GUARD" "$WORK/inverse/summary.txt" "$WORK/inverse/summary.json")"
[ "$got" = "MISSING-FALLBACK" ] && ok "RED-2  half-written pair (empty txt, present json)" "MISSING-FALLBACK" \
                               || no "RED-2  half-written pair (empty txt, present json)" "expected MISSING-FALLBACK, got $got"

# ---------------------------------------------------------------- the class sweep, re-derived
# ⚠️⚠️ THE FIRST VERSION OF THIS SWEEP WAS BLIND AND REPORTED A CONFIDENT WRONG NUMBER — 5 sites when
# there are 6. It matched the `else` by INDENTATION, so a nested `if … else … fi` at the same indent
# inside the then-block silently re-bound `else_at` to the INNER else and the scan then searched the
# wrong block. The site it missed is `REGEX_PARSER_FAMILY_STATUS_CONTRACT` — the sub-gate that runs
# IMMEDIATELY AFTER the one that crashed acceptance run 3, i.e. fixing "all 5" would have reproduced
# the identical crash one sub-gate later. ⇒ track NESTING DEPTH, never indentation; shell blocks are
# not indentation-delimited, and an instrument that assumes they are will under-report and read as
# proof. (This tree's own rule: an instrument with no ground truth is a confident guess.)
sweep_count() {  # <file>  -> number of -f-guarded blocks whose else-branch jq-reads a *_SUMMARY_JSON
  python3 - "$1" <<'PY'
import re, sys
lines = open(sys.argv[1]).read().splitlines()
bad = 0
for i, l in enumerate(lines):
    if not re.match(r'\s*if \[\[ ! -f "\$[A-Z0-9_]+_SUMMARY_TXT" \]\]; then', l):
        continue
    depth = 0; j = i + 1; else_at = None
    while j < len(lines):
        s = lines[j].strip()
        if re.match(r'(if|while|until|for)\b', s) or s.startswith('case '):
            depth += 1
        elif re.match(r'(fi|esac|done)\b', s):
            if depth == 0:
                break
            depth -= 1
        elif s == 'else' and depth == 0:
            else_at = j
        j += 1
    if else_at is not None and re.search(r'jq [^\n]*"\$[A-Z0-9_]+_SUMMARY_JSON"', "\n".join(lines[else_at:j])):
        bad += 1
print(bad)
PY
}
mismatch="$(sweep_count "$GATE")"
[ "$mismatch" = "0" ] && ok "SWEEP  guards testing a different file than they read" "0 survivors" \
                      || no "SWEEP  guards testing a different file than they read" "$mismatch still present"

# ⭐ GROUND TRUTH FOR THE SWEEP ITSELF. `0 survivors` is only evidence if the same instrument finds
# the defect where it is KNOWN to exist. Re-run it against the pre-fix revision: it must report 6.
if git cat-file -e "$BEFORE_REV:$GATE" 2>/dev/null; then
  git show "$BEFORE_REV:$GATE" > "$WORK/gate_before.sh"
  before_n="$(sweep_count "$WORK/gate_before.sh")"
  [ "$before_n" = "6" ] \
    && ok "CTRL-4 sweep reproduces the known before-count" "6 at $BEFORE_REV, 0 now" \
    || no "CTRL-4 sweep reproduces the known before-count" "expected 6 at $BEFORE_REV, got $before_n — MISCALIBRATED"
fi

# ---------------------------------------------------------------- the terminal message names causes
# The count alone sent a triager hunting the CSV. Assert the failure path enumerates the checks.
if grep -q 'report_failed_checks required' "$GATE" && grep -q 'read these, not any error printed after them' "$GATE"; then
  ok "RED-3  terminal message names the failing checks" "enumerates + points at each log"
else
  no "RED-3  terminal message names the failing checks" "still a bare count"
fi

# CTRL-3 — the enumerator run for real. ⛔ The function is EXTRACTED FROM THE LIVE GATE, not
# re-typed here: a probe that re-implements the thing it tests proves only that two copies agree.
CSV="$WORK/summary.csv"
{ printf 'a_gate,required,pass,"notes",/logs/a.log\n'
  printf 'b_gate,required,fail,"notes",/logs/b.log\n'
  printf 'c_gate,informational,fail,"notes",/logs/c.log\n'; } > "$CSV"
sed -n '/^report_failed_checks() {$/,/^}$/p' "$GATE" > "$WORK/enumerator.sh"
if [ ! -s "$WORK/enumerator.sh" ]; then
  no "CTRL-3 enumerator selects failed REQUIRED rows only" "could not extract report_failed_checks from $GATE"
else
  # shellcheck disable=SC1090
  out="$( SUMMARY_CSV="$CSV"; . "$WORK/enumerator.sh"; report_failed_checks required 2>&1 )"
  if printf '%s' "$out" | grep -q 'b_gate' && printf '%s' "$out" | grep -q '/logs/b.log' \
     && ! printf '%s' "$out" | grep -q 'c_gate' && ! printf '%s' "$out" | grep -q 'a_gate'; then
    ok "CTRL-3 live enumerator selects failed REQUIRED rows" "b_gate + its log only"
  else
    no "CTRL-3 live enumerator selects failed REQUIRED rows" "got: $(printf '%s' "$out" | tr '\n' ' ')"
  fi
fi

printf '%s\n' "------------------------------------------------------------------------------"
printf 'arms=%d  PASS=%d  FAIL=%d\n' "$((pass + fail))" "$pass" "$fail"
[ "$fail" -eq 0 ]
