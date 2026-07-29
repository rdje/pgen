#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_target_accounting_probes.sh
#
# CI-PARITY-GATE-ROT.9 — RED/GREEN/CONTROL arms for the two halves of the fix:
#   (A) `parse_target_summary`, which was reading the TARGET-DRIVE summary line and discarding
#       the appended witness pass that runs after it in the same invocation, and
#   (B) the regex family gate's target-accounting assertion, which was a strict equality.
#
# ⭐ CTRL-0 IS THE ARM THAT MATTERS: it extracts the OLD function from the last commit that still
# had it (DERIVED, not pinned to HEAD, so this keeps working after the fix is committed) and the
# NEW one from the working tree, feeds BOTH the same verbatim stage-2 log, and requires them to
# DISAGREE by exactly the deficit the aggregate reported (723 vs 1002 on 1033 targets). The
# before->after is REPLAYED, not described — a driver that only asserted the new behaviour could
# not tell a real fix from a no-op.
#
# ⭐ RED-5 IS THE OTHER ONE: a closed loop that resolved NOTHING (resolved=0, final=initial).
# The retired strict equality PASSED that run; the replacement fails it. That is the dimension
# on which the new consumer check is strictly stronger, and it is proved by running the retired
# form side by side rather than by asserting it in prose.
#
# ⛔ Both forms are EXTRACTED FROM THE GATES (live tree / `git show`), never re-typed, so this
# cannot test a rule the gates do not apply.
#
# Usage: bash docs/tasks/artifacts/ci_parity_gate_rot/run_target_accounting_probes.sh
# Exit 0 iff every arm reaches its expected verdict.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"
W="rust/target/ci_parity_gate_rot_probe/p9"; rm -rf "$W"; mkdir -p "$W/logs"

PRODUCER=rust/scripts/ebnf_stimuli_quality_gate.sh
CONSUMER=rust/scripts/regex_parser_family_contract_gate.sh

# ---- extract the LIVE reader (never re-typed) --------------------------------------------
{ echo '#!/usr/bin/env bash'; echo 'set -uo pipefail'
  sed -n '/^parse_target_summary() {/,/^}/p' "$PRODUCER"; } > "$W/new_reader.sh"
grep -q 'Witness pass: resolved' "$W/new_reader.sh" || { echo "FATAL: new reader did not extract"; exit 1; }

# ---- extract the RETIRED reader from git (never re-typed) --------------------------------
# ⛔ NOT hardcoded to HEAD. Before this leaf is committed the retired form IS at HEAD; after it is
# committed HEAD carries the fix, and a driver pinned to HEAD would compare the fix against itself
# and silently prove nothing. So the baseline is DERIVED: the commit that introduced the
# witness-line read, minus one — falling back to HEAD while that commit does not yet exist.
INTRODUCED="$(git log --format=%H -S'Witness pass: resolved' -- "$PRODUCER" | tail -1)"
if [ -n "$INTRODUCED" ]; then BASE="${INTRODUCED}^"; else BASE="$(git rev-parse --verify HEAD)"; fi
{ echo '#!/usr/bin/env bash'; echo 'set -uo pipefail'
  git show "$BASE:$PRODUCER" | sed -n '/^parse_target_summary() {/,/^}/p'; } > "$W/old_reader.sh"
grep -q '^parse_target_summary() {' "$W/old_reader.sh" || { echo "FATAL: old reader did not extract from $BASE"; exit 1; }
if grep -q 'Witness pass: resolved' "$W/old_reader.sh"; then
  echo "FATAL: baseline $BASE already reads the witness line — the comparison would prove nothing"; exit 1
fi
printf '    retired-reader baseline: %s\n' "$(git rev-parse --short "$BASE")"

# ---- logs: the VERBATIM summary lines measured on the aggregate run that surfaced this ----
mk_log() { # mk_log <file> <drive_resolved> <total> <attempts> [<w_before> <w_after> <w_total>]
  local f="$1" dr="$2" tt="$3" at="$4"
  printf 'Completed target-driven generation: entry=%s\n' "'regex'" >"$f"
  printf 'Target-driven generation: resolved %s/%s targets in %s attempts (generation_successes=3272, generation_errors=1728, depth_exceeded_errors=0, rule_visit_limit_errors=0, target_timeout_errors=0, helper_timeout_errors=0, reach_plan_activations=0)\n' "$dr" "$tt" "$at" >>"$f"
  if [ "$#" -ge 7 ]; then
    printf 'Witness pass: resolved %s -> %s of %s reachable targets (+279 via 249 witnesses; failures depth_exceeded=0, rule_visit_limit=0, target_timeout=0, helper_timeout=0, other=14, no_entry=0; construct_fell_back_to_search=14)\n' "$5" "$6" "$7" >>"$f"
  fi
}
mk_log "$W/logs/regex.log"       723 1033 5000 723 1002 1033   # the real regex row
mk_log "$W/logs/ebnf.log"        55    68 5000  55   60   68   # the real ebnf row
mk_log "$W/logs/saturated.log"   20    20  131  20   20   20   # real annotation row: drive finished
mk_log "$W/logs/no_witness.log"  723 1033 5000                 # witness line absent
mk_log "$W/logs/mismatch_b.log"  723 1033 5000 700 1002 1033   # baseline disagrees with drive
mk_log "$W/logs/mismatch_t.log"  723 1033 5000 723 1002  999   # total disagrees with drive
mk_log "$W/logs/regressed.log"   723 1033 5000 723  700 1033   # witness pass went backwards

pass=0; fail=0
arm(){ local l="$1" want="$2" needle="$3"; shift 3
  out=$( ( set +e; "$@" ) 2>&1 ); rc=$?
  [ $rc -eq 0 ] && got=PASS || got=FAIL
  if [ "$got" != "$want" ]; then fail=$((fail+1)); printf '✗ %-44s expected %s got %s\n' "$l" "$want" "$got"; echo "$out"|sed 's/^/    /'; return; fi
  if [ -n "$needle" ] && ! echo "$out" | grep -qF -- "$needle"; then fail=$((fail+1)); printf '✗ %-44s %s but no "%s"\n' "$l" "$got" "$needle"; echo "$out"|sed 's/^/    /'; return; fi
  pass=$((pass+1)); printf '✓ %-44s %s\n' "$l" "$got"; }
read_with(){ local which="$1" log="$2"; ( . "$W/${which}_reader.sh"; parse_target_summary "$log" ); }

echo "=== CI-PARITY-GATE-ROT.9 — target-accounting probes ==="
echo "--- (A) the reader: before -> after, replayed ---"
old_regex="$(read_with old "$W/logs/regex.log" 2>/dev/null || echo ERR)"
new_regex="$(read_with new "$W/logs/regex.log" 2>/dev/null || echo ERR)"
printf '    OLD reader on the regex log: %s\n    NEW reader on the regex log: %s\n' "$old_regex" "$new_regex"
arm "CTRL-0a OLD reader under-reports 723"  PASS "" bash -c "[ \"$old_regex\" = '723 1033 5000' ]"
arm "CTRL-0b NEW reader reports 1002"       PASS "" bash -c "[ \"$new_regex\" = '1002 1033 5000' ]"
arm "CTRL-0c they DISAGREE by the deficit"  PASS "" bash -c "[ \$((1002-723)) -eq 279 ] && [ \"$old_regex\" != \"$new_regex\" ]"

echo "--- (A) the reader: GREEN / CONTROL / RED ---"
new_ebnf="$(read_with new "$W/logs/ebnf.log" 2>/dev/null || echo ERR)"
arm "GREEN-1 ebnf row post-witness = 60"    PASS "" bash -c "[ \"$new_ebnf\" = '60 68 5000' ]"
new_sat="$(read_with new "$W/logs/saturated.log" 2>/dev/null || echo ERR)"
old_sat="$(read_with old "$W/logs/saturated.log" 2>/dev/null || echo ERR)"
arm "CTRL-1 drive finished => SAME answer"  PASS "" bash -c "[ \"$new_sat\" = \"$old_sat\" ] && [ \"$new_sat\" = '20 20 131' ]"
arm "RED-1 witness line absent => refuse"   FAIL "unable to locate witness-pass summary" read_with new "$W/logs/no_witness.log"
arm "RED-2 baseline mismatch => refuse"     FAIL "describe different passes"             read_with new "$W/logs/mismatch_b.log"
arm "RED-3 total mismatch => refuse"        FAIL "does not match the target-drive total" read_with new "$W/logs/mismatch_t.log"
arm "RED-4 witness regressed => refuse"     FAIL "purely additive"                       read_with new "$W/logs/regressed.log"

# ---- extract the LIVE + RETIRED consumer assertions ---------------------------------------
sed -n '/^if (( stimuli_regex_resolved_targets + stimuli_regex_final_targets > /,/^fi$/p' "$CONSUMER" >"$W/c_sound.sh"
sed -n '/^if (( stimuli_regex_final_targets >= stimuli_regex_initial_targets/,/^fi$/p'     "$CONSUMER" >"$W/c_progress.sh"
C_INTRO="$(git log --format=%H -S'target accounting unsound' -- "$CONSUMER" | tail -1)"
if [ -n "$C_INTRO" ]; then C_BASE="${C_INTRO}^"; else C_BASE="$(git rev-parse --verify HEAD)"; fi
git show "$C_BASE:$CONSUMER" | sed -n '/^if (( stimuli_regex_resolved_targets + stimuli_regex_final_targets != /,/^fi$/p' >"$W/c_old.sh"
for f in c_sound c_progress c_old; do
  [ -s "$W/$f.sh" ] || { echo "FATAL: could not extract $f"; exit 1; }
done
consumer(){ # consumer <which> <resolved> <final> <initial>
  ( set -uo pipefail
    stimuli_regex_resolved_targets="$2"; stimuli_regex_final_targets="$3"; stimuli_regex_initial_targets="$4"
    . "$W/$1.sh" ); }

echo "--- (B) the consumer assertion: the retired equality vs the replacement ---"
arm "CTRL-2 healthy run passes soundness"   PASS "" consumer c_sound    1002 31 1033
arm "CTRL-3 healthy run passes progress"    PASS "" consumer c_progress 1002 31 1033
arm "RED-5a zero-progress PASSED the OLD form"  PASS "" consumer c_old      0 1033 1033
arm "RED-5b zero-progress FAILS the new form"   FAIL "resolved no targets" consumer c_progress 0 1033 1033
arm "RED-6 unsound sum still refused"       FAIL "unsound" consumer c_sound    1003 31 1033
arm "CTRL-4 stale 723 no longer misjudged"  PASS "" consumer c_sound     723 31 1033

echo "---"; printf 'arms=%d PASS=%d FAIL=%d\n' $((pass+fail)) $pass $fail; [ $fail -eq 0 ]
