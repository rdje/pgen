#!/usr/bin/env bash
# run_audit_stale_failure_probes.sh — prove `DONE-BAR.1a`: the audit's failed-gate branch now applies
# the SAME staleness rule its artifact branch always applied, and — the arm that matters — a FRESH
# failure still reads UNMET, so the audit has not gone blind to real current failures.
#
# ⭐ The BEFORE arm runs the RETIRED script straight out of `git show HEAD:` — the before→after is
# REPLAYED, not described (the `.5e` / `CI-PARITY-GATE-ROT.14` convention).
#
# ⚠️ This driver MUTATES a log's mtime to build its fixtures and RESTORES it afterwards; the restore
# runs from a trap, so an interrupted run cannot leave the audit reading a fabricated vintage.
#
#   bash docs/tasks/artifacts/done_bar/run_audit_stale_failure_probes.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

AUDIT="scripts/audit_done_bar.sh"
LOG="rust/target/sota_exit_gate/logs/regex_parser_family_status_gate.log"
WORK="rust/target/done_bar_audit/stale_failure_probe"
rm -rf "$WORK"; mkdir -p "$WORK"

pass=0; fail=0; unjudged=0

# ⚠️ MEASURED 2026-07-30: `audit_done_bar.sh` REFUSES ("check_gate_reachability.sh did not succeed")
# transiently while a `make -C rust` aggregate runs concurrently — the reachability check shells out
# to make, and the refusal does not reproduce standalone (the log it points at reads OK). An arm that
# cannot see a verdict must SAY SO rather than pass over a 171-byte refusal, which is exactly what the
# first cut of BEFORE-1 did. Routed as a finding to `DONE-BAR.1a`.
run_audit() {
    local script="$1" out
    for _ in 1 2 3; do
        out="$(bash "$script" 2>&1)"
        [[ "$out" == *"REFUSED"* ]] || { printf '%s' "$out"; return 0; }
    done
    printf '%s' "$out"
    return 1
}
arm() {
    local label="$1" want="$2" haystack="$3"
    if [[ "$haystack" == *"$want"* ]]; then
        printf '  ✅ %-60s found %q\n' "$label" "$want"; pass=$((pass + 1))
    else
        printf '  ⛔ %-60s MISSING %q\n' "$label" "$want"; fail=$((fail + 1))
    fi
}
arm_absent() {
    local label="$1" needle="$2" haystack="$3"
    if [[ "$haystack" != *"$needle"* ]]; then
        printf '  ✅ %-60s absent %q\n' "$label" "$needle"; pass=$((pass + 1))
    else
        printf '  ⛔ %-60s UNEXPECTEDLY PRESENT %q\n' "$label" "$needle"; fail=$((fail + 1))
    fi
}

if [[ ! -f "$LOG" ]]; then
    echo "REFUSED: fixture log $LOG is absent — this probe needs a failed-gate log to reason about." >&2
    echo "         (It is produced by a sota_exit_gate run; a skip is never a pass.)" >&2
    exit 2
fi

# Preserve and restore the fixture's real vintage no matter how this script ends.
ORIG_EPOCH="$(python3 -c 'import os,sys; print(int(os.path.getmtime(sys.argv[1])))' "$LOG")"
restore() { touch -t "$(python3 -c 'import time,sys; print(time.strftime("%Y%m%d%H%M.%S", time.localtime(int(sys.argv[1]))))' "$ORIG_EPOCH")" "$LOG"; }
trap restore EXIT

echo "=============================================================================="
echo "DONE-BAR.1a — the audit's failed-gate branch, staleness-aware"
echo "=============================================================================="
echo "fixture: $LOG (real mtime epoch $ORIG_EPOCH)"
echo

# --- RED-1: the log AS IT REALLY IS (older than the tracker edit that superseded it) ------------
restore
red1="$(run_audit "$AUDIT")" || { echo "  ⚠️  RED-1 UNJUDGEABLE: audit REFUSED 3x (concurrent make?)"; unjudged=$((unjudged+1)); }
arm       "RED-1 stale failure is LABELLED stale"      "PREDATES its own inputs"                 "$red1"
arm       "RED-1 the superseded error is still QUOTED" "regex tracker alignment mismatch"        "$red1"
arm       "RED-1 leg 1 downgrades to UNPROVEN"         "UNPROVEN, not UNMET"                     "$red1"
printf '%s\n' "$red1" > "$WORK/red1_stale.txt"

# --- CTRL-1: THE ARM THAT MATTERS — a FRESH failure must still read UNMET -----------------------
# If the fix made every recorded failure UNPROVEN, the audit would stop seeing real current
# failures. Make the same log NEWER than every input and require the verdict to flip back.
touch "$LOG"
ctrl1="$(run_audit "$AUDIT")" || { echo "  ⚠️  CTRL-1 UNJUDGEABLE: audit REFUSED 3x (concurrent make?)"; unjudged=$((unjudged+1)); }
arm       "CTRL-1 fresh failure is NOT labelled stale — leg 1"  "RAN AND FAILED (log rust/target/sota_exit_gate/logs/regex_parser_family_status_gate.log, mtime" "$ctrl1"
# Scoped to the FAILED-GATE wording: the bare phrase also occurs legitimately on the ARTIFACT branch
# for other families, so an unscoped absent-check fails for the wrong reason (caught by this arm).
arm_absent "CTRL-1 fresh failure does NOT claim to predate (leg1)" "that failure PREDATES its own inputs" "$ctrl1"
arm_absent "CTRL-1 fresh failure does NOT claim to predate (leg2)" "but that run PREDATES its own inputs" "$ctrl1"
arm       "CTRL-1 the error is reported as CURRENT"             "⛔ regex_parser_family_status_gate RAN AND FAILED" "$ctrl1"
printf '%s\n' "$ctrl1" > "$WORK/ctrl1_fresh.txt"
restore

# --- CTRL-2: the audit's exit code and control calibration survive both states ------------------
bash "$AUDIT" >/dev/null 2>&1; rc_live=$?
arm       "CTRL-2 audit still exits 0 on the live tree"  "0" "$rc_live"
arm_absent "CTRL-2 no MISCALIBRATED refusal"             "MISCALIBRATED" "$red1"

# --- BEFORE-1: the RETIRED script replayed from git -------------------------------------------
PREV="$WORK/audit_before.sh"
if git show "HEAD:$AUDIT" > "$PREV" 2>/dev/null; then
    before="$(run_audit "$PREV")"; before_rc=$?
    printf '%s\n' "$before" > "$WORK/before.txt"
    if [[ "$before_rc" -ne 0 || "$before" != *"leg 1 stimuli-generator proof"* ]]; then
        # ⛔ A skip is never a pass. The first cut of this block asserted an ABSENT string and passed
        # vacuously over a 171-byte REFUSED message — the arm reached the right verdict for the wrong
        # reason, the failure mode CI-PARITY-GATE-ROT.4 exists to catch.
        echo "  ⚠️  BEFORE-1 UNJUDGEABLE: the retired audit produced no verdict (REFUSED / no leg output)"
        echo "      $(printf '%s' "$before" | head -1)"
        unjudged=$((unjudged + 3))
        before=""
    fi
    if [[ -n "$before" ]]; then
    arm_absent "BEFORE-1 the retired audit reported NO vintage on failures" "PREDATES its own inputs ⇒ UNPROVEN, not UNMET" "$before"
    arm       "BEFORE-1 the retired audit quoted the stale error as current" "⛔ regex_parser_family_status_gate RAN AND FAILED (log rust/target/sota_exit_gate/logs/regex_parser_family_status_gate.log): error:" "$before"
    # ⭐ The self-contradiction the leaf is about: `tracker: In Progress` above a ⛔ saying `Done`.
    arm       "BEFORE-1 it printed the tracker as In Progress"  "regex   tracker: In Progress"  "$before"
    arm       "BEFORE-1 …while quoting an error saying 'Done'"  "tracker says 'Done'"           "$before"
    fi
else
    echo "  ⚠️  BEFORE-1 skipped: git show HEAD:$AUDIT unavailable"
fi

echo
echo "=============================================================================="
printf 'audit stale-failure probes: %d passed, %d failed, %d UNJUDGEABLE\n' "$pass" "$fail" "$unjudged"
[[ "$unjudged" -eq 0 ]] || echo "⚠️  UNJUDGEABLE arms are NOT passes — re-run when no aggregate make is executing"
echo "=============================================================================="
[[ "$fail" -eq 0 ]]
