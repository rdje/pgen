#!/usr/bin/env bash
# run_audit_stale_failure_probes.sh — prove `DONE-BAR.1a`: the audit's failed-gate branch applies the
# SAME staleness rule its artifact branch always applied, and — the arm that matters — a FRESH failure
# still reads UNMET, so the audit has not gone blind to real current failures.
#
# ⭐⭐ TWO DEFECTS IN THIS DRIVER'S OWN FIRST CUT, both corrected here and both recorded in the leaf:
#
#   1. It ran the retired script from a COPY under `rust/target/…/stale_failure_probe/`. But
#      `audit_done_bar.sh:44` derives `ROOT="$(dirname "${BASH_SOURCE[0]}")/.."` from its OWN location,
#      so ROOT became `rust/target/done_bar_audit` and the script REFUSED
#      ("check_gate_reachability.sh did not succeed"). ⛔ That refusal was first written up as
#      *"transient contention with a concurrently running `make -C rust`"* — **WRONG**: the identical
#      content run from `scripts/` exits 0 with no aggregate involved. A path-depth bug was mistaken for
#      flakiness, and a 3-retry loop was added to paper over it. The retired script is now staged INSIDE
#      `scripts/` so its ROOT resolves correctly, and there is no retry loop.
#   2. It depended on a REAL failed-gate log as its fixture. `sota_exit_gate` run 4 then made that gate
#      PASS, overwriting the log, and the RED arms lost the failure they existed to detect. ⛔ A probe
#      whose fixture is a transient artifact of the last run is not a probe — the `DONE-BAR.5f` custody
#      lesson. The fixture is now SYNTHETIC and built here, so this driver is reproducible on any tree.
#
# Fixture construction moves two real paths aside and restores them from a trap, so an interrupted run
# cannot leave the audit reading a fabricated state.
#
#   bash docs/tasks/artifacts/done_bar/run_audit_stale_failure_probes.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

AUDIT="scripts/audit_done_bar.sh"
GATE="regex_parser_family_status_gate"
# `gate_ran_and_failed` probes these two paths in order and returns the first carrying an `error:` line.
REAL_LOG="rust/target/sota_exit_gate/logs/$GATE.log"
SYNTH_LOG="rust/target/$GATE/logs/$GATE.log"
# The failed-gate branch is only reached when NO summary artifact exists, so the real one must be hidden.
REAL_SUMMARY_DIR="rust/target/sota_exit_gate/work/$GATE"
WORK="rust/target/done_bar_audit/stale_failure_probe"
STAGED_BEFORE="scripts/_probe_audit_before.sh"   # inside scripts/ so ROOT resolves correctly
rm -rf "$WORK"; mkdir -p "$WORK"

pass=0; fail=0

cleanup() {
    rm -f "$STAGED_BEFORE"
    [[ -e "$WORK/hidden_summary" ]] && { rm -rf "$REAL_SUMMARY_DIR"; mv "$WORK/hidden_summary" "$REAL_SUMMARY_DIR"; }
    [[ -e "$WORK/hidden_log" ]] && { mkdir -p "$(dirname "$REAL_LOG")"; mv "$WORK/hidden_log" "$REAL_LOG"; }
    rm -rf "rust/target/$GATE/logs"
    return 0
}
trap cleanup EXIT

arm() {
    local label="$1" want="$2" haystack="$3"
    if [[ "$haystack" == *"$want"* ]]; then
        printf '  ✅ %-58s found %q\n' "$label" "$want"; pass=$((pass + 1))
    else
        printf '  ⛔ %-58s MISSING %q\n' "$label" "$want"; fail=$((fail + 1))
    fi
}
arm_absent() {
    local label="$1" needle="$2" haystack="$3"
    if [[ "$haystack" != *"$needle"* ]]; then
        printf '  ✅ %-58s absent %q\n' "$label" "$needle"; pass=$((pass + 1))
    else
        printf '  ⛔ %-58s UNEXPECTEDLY PRESENT %q\n' "$label" "$needle"; fail=$((fail + 1))
    fi
}
# Every arm must produce a real verdict. A run that judged nothing is UNJUDGEABLE, never a pass —
# the first cut of the BEFORE block passed VACUOUSLY over a 171-byte REFUSED message.
require_verdict() {
    local label="$1" out="$2"
    if [[ "$out" != *"leg 1 stimuli-generator proof"* ]]; then
        printf '  ⛔ %-58s UNJUDGEABLE — no leg output; first line: %s\n' "$label" "$(printf '%s' "$out" | head -1)"
        fail=$((fail + 1)); return 1
    fi
    return 0
}

echo "=============================================================================="
echo "DONE-BAR.1a — the audit's failed-gate branch, staleness-aware"
echo "=============================================================================="

# ---- build the SYNTHETIC fixture ------------------------------------------------------------
# Hide the real summary so the failed-gate branch is reachable, and hide the real log so the
# synthetic one is the first candidate carrying an `error:` line.
[[ -d "$REAL_SUMMARY_DIR" ]] && mv "$REAL_SUMMARY_DIR" "$WORK/hidden_summary"
[[ -f "$REAL_LOG" ]] && { mkdir -p "$WORK"; mv "$REAL_LOG" "$WORK/hidden_log"; }
mkdir -p "$(dirname "$SYNTH_LOG")"
cat > "$SYNTH_LOG" <<'FIXTURE'
==> regex_parser_family_status_gate
error: regex tracker alignment mismatch: computed 'In Progress' but tracker says 'Done'
FIXTURE
echo "fixture: synthetic failed-gate log at $SYNTH_LOG"
echo

set_epoch() { touch -t "$(python3 -c 'import time,sys;print(time.strftime("%Y%m%d%H%M.%S", time.localtime(int(sys.argv[1]))))' "$1")" "$SYNTH_LOG"; }

# ---- RED-1: the failure PREDATES its inputs (the measured 2026-07-29 13:49 vintage) ----------
set_epoch 1785325790
red1="$(bash "$AUDIT" 2>&1)"
if require_verdict "RED-1" "$red1"; then
    arm "RED-1 stale failure is LABELLED stale"        "⚠️ STALE — older than its own inputs" "$red1"
    arm "RED-1 the superseded error is still QUOTED"   "regex tracker alignment mismatch"     "$red1"
    arm "RED-1 leg 1 downgrades to UNPROVEN"           "UNPROVEN, not UNMET"                  "$red1"
    arm "RED-1 leg 2 says green-NOW is UNPROVEN"       "but that run PREDATES its own inputs" "$red1"
fi
printf '%s\n' "$red1" > "$WORK/red1_stale.txt"

# ---- CTRL-1: THE ARM THAT MATTERS — a FRESH failure must still read UNMET --------------------
# If the fix made every recorded failure UNPROVEN, the audit would stop seeing real current failures.
touch "$SYNTH_LOG"
ctrl1="$(bash "$AUDIT" 2>&1)"
if require_verdict "CTRL-1" "$ctrl1"; then
    arm        "CTRL-1 the error is reported as CURRENT"           "⛔ $GATE RAN AND FAILED"              "$ctrl1"
    arm_absent "CTRL-1 fresh failure is not called stale (leg 1)"  "that failure PREDATES its own inputs" "$ctrl1"
    arm_absent "CTRL-1 fresh failure is not called stale (leg 2)"  "but that run PREDATES its own inputs" "$ctrl1"
    arm_absent "CTRL-1 no STALE label on the failed-gate branch"   "⚠️ STALE — older than its own inputs" "$ctrl1"
fi
printf '%s\n' "$ctrl1" > "$WORK/ctrl1_fresh.txt"

# ---- BEFORE-1: the RETIRED script, staged at the CORRECT DEPTH so its ROOT resolves ----------
# `.1a` landed in 76455204, so HEAD~1 is the last revision without the fix. Pinning the SHA-relative
# ref rather than a literal keeps this reproducible as history grows.
set_epoch 1785325790
if git show "76455204~1:$AUDIT" > "$STAGED_BEFORE" 2>/dev/null; then
    chmod +x "$STAGED_BEFORE"
    before="$(bash "$STAGED_BEFORE" 2>&1)"
    printf '%s\n' "$before" > "$WORK/before.txt"
    if require_verdict "BEFORE-1" "$before"; then
        arm        "BEFORE-1 retired form quoted the stale error as CURRENT" "⛔ $GATE RAN AND FAILED" "$before"
        arm_absent "BEFORE-1 retired form reported NO vintage"               "PREDATES its own inputs ⇒ UNPROVEN, not UNMET" "$before"
        arm_absent "BEFORE-1 retired form never labelled it STALE"           "⚠️ STALE — older than its own inputs" "$before"
        # ⭐ The self-contradiction the leaf is about: the tracker read In Progress while the quoted
        #    error complained the tracker said Done.
        arm        "BEFORE-1 printed tracker as In Progress…"                "tracker: In Progress"   "$before"
        arm        "BEFORE-1 …above an error claiming it says 'Done'"        "tracker says 'Done'"    "$before"
    fi
else
    printf '  ⛔ %-58s UNJUDGEABLE — git show 76455204~1 unavailable\n' "BEFORE-1"; fail=$((fail + 1))
fi

# ---- CTRL-2: the live tree (real artifacts restored) still audits clean ----------------------
cleanup; trap - EXIT
live="$(bash "$AUDIT" 2>&1)"; rc=$?
arm        "CTRL-2 audit exits 0 on the restored live tree" "0"              "$rc"
arm_absent "CTRL-2 no MISCALIBRATED refusal"                "MISCALIBRATED"  "$live"
arm_absent "CTRL-2 restored tree shows no stale failure"    "⚠️ STALE — older than its own inputs" "$live"

echo
echo "=============================================================================="
printf 'audit stale-failure probes: %d passed, %d failed\n' "$pass" "$fail"
echo "=============================================================================="
[[ "$fail" -eq 0 ]]
