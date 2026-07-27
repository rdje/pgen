#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_audit_census.sh
#
# CI-PARITY-GATE-ROT.1 — run EVERY `audit_*` function of the local workflow-parity gate
# individually and report PASS/FAIL per audit.
#
# ⭐ WHY THIS EXISTS. `make -C rust ci_workflow_local_gate` runs the audits in sequence and stops
# at the first failure, so it can only ever name ONE broken audit per run — which is precisely how
# a gate with a dozen stale assertions looked like a single-blocker problem for 1,371 commits.
# This driver isolates each audit in its own subshell so the WHOLE census is visible at once.
#
# ⛔⛔ TWO REPRODUCTION TRAPS, both hit while writing this, both of which produce a CONFIDENTLY
# WRONG census. They are recorded because either one silently inflates the failure count:
#
#   (1) THE SCRIPT LOCATES ITSELF. `ci_workflow_local_gate.sh` computes
#       `ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"`. Copy the stripped
#       version anywhere other than `rust/scripts/` and ROOT_DIR silently resolves one directory
#       off, so every `assert_tracked` checks a path that does not exist. Observed: 28 of 31
#       "failing", with `rust/Makefile` reported "missing from git index" and a tell-tale doubled
#       `rust/rust/target/...` in the output. The stripped copy MUST live at the same depth.
#
#   (2) THE AUDIT NAMES CONTAIN DIGITS. Harvesting them with `grep -oE '^  audit_[a-z_]+'` cuts
#       `audit_regex_pcre2_compile_oracle_surface` at the `2`, producing the non-existent function
#       `audit_regex_pcre`, which then "fails" as command-not-found and inflates the count by one.
#       Use `[a-z0-9_]+`.
#
# Both traps are instances of the standing TOOLBOX question — *is the thing I am testing the thing
# being run?* — applied to a shell gate rather than a grammar rule.
#
# Usage: bash docs/tasks/artifacts/ci_parity_gate_rot/run_audit_census.sh [--verbose]
# Exit 0 always (a measuring instrument, not a gate); the census is the output.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

GATE="rust/scripts/ci_workflow_local_gate.sh"
[ -f "$GATE" ] || { echo "census: ✗ gate not found: $GATE" >&2; exit 1; }

VERBOSE=0
[ "${1:-}" = "--verbose" ] && VERBOSE=1

# Trap (1): the stripped copy must sit at the SAME DEPTH as the original.
PROBE="rust/scripts/.ciparity_probe.sh"
sed 's/^main "\$@"$//' "$GATE" > "$PROBE"
trap 'rm -f "$ROOT/$PROBE"' EXIT

pass=0; fail=0
printf '%s\n' "=============================================================================="
printf 'PER-AUDIT CENSUS of %s\n' "$GATE"
printf '%s\n' "=============================================================================="
# Trap (2): [a-z0-9_]+, not [a-z_]+.
for f in $(grep -oE '^  audit_[a-z0-9_]+' "$GATE" | tr -d ' '); do
  out=$( (set +e; . "$PROBE" >/dev/null 2>&1; $f) 2>&1 ); rc=$?
  if [ "$rc" -eq 0 ]; then
    pass=$((pass + 1))
    [ "$VERBOSE" -eq 1 ] && printf 'PASS  %s\n' "$f"
  else
    fail=$((fail + 1))
    printf 'FAIL  %s\n' "$f"
    printf '%s\n' "$out" | grep -iE 'error|drift|unexpected|expected' | head -3 | sed 's/^/        /'
  fi
done
printf '%s\n' "------------------------------------------------------------------------------"
printf 'audits=%d  PASS=%d  FAIL=%d\n' "$((pass + fail))" "$pass" "$fail"
exit 0
