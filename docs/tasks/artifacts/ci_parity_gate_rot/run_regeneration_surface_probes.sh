#!/usr/bin/env bash
# docs/tasks/artifacts/ci_parity_gate_rot/run_regeneration_surface_probes.sh
#
# CI-PARITY-GATE-ROT.4 — RED / GREEN / CONTROL arms for `audit_workflow_regeneration_surface`,
# the audit that stops the "workflow needs generated/ but declares no regeneration step" defect
# from recurring a fourth time.
#
# ⭐ WHAT EACH ARM IS FOR. An audit that only ever runs against a tree that satisfies it proves
# nothing — `.1` found twelve assertions that had been "passing" for 1,371 commits because nothing
# ran them, and `.3` found a filter typo that printed a green while replaying zero workflows. So
# every arm below deliberately BREAKS one invariant and requires the audit to say so.
#
# ⭐⭐ RED-6 IS THE LOAD-BEARING ARM. It adds a brand-new tracked workflow that runs a
# `make -C rust` gate and declares no regeneration step. The audit must fail on a file that did
# not exist when the audit was written — that, and only that, proves the roster is DERIVED from
# `git ls-files` rather than hand-listed. A hand-listed roster is exactly why
# `rtl-const-expr-cert-gate` and `sv-cert-recognized-union-gate` stayed unmeasured through `.3`.
#
# ⛔ REPRODUCTION TRAPS INHERITED FROM run_audit_census.sh, and they still apply:
#   (1) the gate self-locates via `BASH_SOURCE/../..`, so the stripped probe copy MUST sit at the
#       same depth (`rust/scripts/`), or ROOT_DIR resolves one directory off and everything fails;
#   (2) audit names contain digits — harvest with `[a-z0-9_]+`, never `[a-z_]+`.
#
# ⛔ THE INDEX IS NEVER MUTATED. Arms that need a different set of TRACKED files (RED-6, RED-8)
# operate on a COPY of `.git/index` via `GIT_INDEX_FILE`, so the real staging area is untouched
# whatever happens. Working-tree files are backed up and restored by an EXIT trap.
#
# Usage: bash docs/tasks/artifacts/ci_parity_gate_rot/run_regeneration_surface_probes.sh
# Exit 0 iff every arm reaches its expected verdict.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$ROOT"

GATE="rust/scripts/ci_workflow_local_gate.sh"
AUDIT="audit_workflow_regeneration_surface"
CONTROL_AUDIT="audit_workflow_surface"
[ -f "$GATE" ] || { echo "probes: ✗ gate not found: $GATE" >&2; exit 1; }

WORK="rust/target/ci_parity_gate_rot_probe/arms"
BACKUP="$WORK/backup"
PROBE="rust/scripts/.ciparity_regen_probe.sh"
NEW_WORKFLOW=".github/workflows/zz-probe-synthetic-gate.yml"
TMP_INDEX="$WORK/index"

rm -rf "$WORK"; mkdir -p "$BACKUP"

restore_all() {
  local rel
  if [ -d "$ROOT/$BACKUP" ]; then
    (cd "$ROOT/$BACKUP" && find . -type f -print0) 2>/dev/null |
      while IFS= read -r -d '' rel; do
        rel="${rel#./}"
        cp "$ROOT/$BACKUP/$rel" "$ROOT/$rel"
      done
  fi
  rm -f "$ROOT/$PROBE" "$ROOT/$NEW_WORKFLOW"
}
trap 'restore_all' EXIT

backup() { mkdir -p "$ROOT/$BACKUP/$(dirname "$1")"; cp "$ROOT/$1" "$ROOT/$BACKUP/$1"; }
revert() { cp "$ROOT/$BACKUP/$1" "$ROOT/$1"; }

# Trap (1): the stripped copy must sit at the SAME DEPTH as the original.
build_probe() { sed 's/^main "\$@"$//' "$ROOT/$GATE" > "$ROOT/$PROBE"; }

run_audit() {
  local fn="$1"
  ( set +e; . "$ROOT/$PROBE" >/dev/null 2>&1; "$fn" ) 2>&1
  return "${PIPESTATUS[0]}"
}

pass=0; fail=0
arm() {
  # arm <label> <expected: PASS|FAIL> <audit-fn> [expected-substring]
  local label="$1" want="$2" fn="$3" needle="${4:-}"
  local out rc got
  build_probe
  out="$( ( set +e; . "$ROOT/$PROBE" >/dev/null 2>&1; "$fn" ) 2>&1 )"; rc=$?
  [ "$rc" -eq 0 ] && got=PASS || got=FAIL
  if [ "$got" != "$want" ]; then
    fail=$((fail + 1))
    printf '✗ %-34s expected %s, got %s\n' "$label" "$want" "$got"
    printf '%s\n' "$out" | tail -6 | sed 's/^/      /'
    return 0
  fi
  if [ -n "$needle" ] && ! printf '%s\n' "$out" | grep -qF -- "$needle"; then
    fail=$((fail + 1))
    printf '✗ %-34s %s as expected, but the message never mentions %s\n' "$label" "$got" "$needle"
    printf '%s\n' "$out" | tail -6 | sed 's/^/      /'
    return 0
  fi
  pass=$((pass + 1))
  printf '✓ %-34s %s\n' "$label" "$got"
}

printf '%s\n' "=============================================================================="
printf 'CI-PARITY-GATE-ROT.4 — %s probes\n' "$AUDIT"
printf '%s\n' "=============================================================================="

backup .github/workflows/performance-gate.yml
backup .github/workflows/branch-protection-contract-gate.yml
backup .github/workflows/rtl-const-expr-cert-gate.yml
backup rust/Makefile
backup "$GATE"

# ---------------------------------------------------------------- GREEN (baseline)
arm "GREEN  untouched tree" PASS "$AUDIT"

# ---------------------------------------------------------------- RED-1: step removed
grep -v 'uses: ./.github/actions/regenerate-parsers' \
  "$ROOT/$BACKUP/.github/workflows/performance-gate.yml" \
  > "$ROOT/.github/workflows/performance-gate.yml"
arm "RED-1  wired workflow loses step" FAIL "$AUDIT" "declares no regeneration step"
revert .github/workflows/performance-gate.yml

# ---------------------------------------------------------------- RED-2: an exempt workflow pays
#  `.4`'s scope is explicit that "the 3 that pass must NOT pay for it" — a 258 s regeneration
#  bolted onto a 1 s shell check is a cost regression, so exemption is asserted in BOTH directions.
printf '      - name: Regenerate the generated parsers\n        uses: ./.github/actions/regenerate-parsers\n' \
  >> "$ROOT/.github/workflows/branch-protection-contract-gate.yml"
arm "RED-2  exempt workflow pays for it" FAIL "$AUDIT" "unexpected workflow content"
revert .github/workflows/branch-protection-contract-gate.yml

# ---------------------------------------------------------------- RED-3: timeout below the floor
python3 - "$ROOT/.github/workflows/rtl-const-expr-cert-gate.yml" <<'PY'
import re, sys
p = sys.argv[1]
s = open(p).read()
open(p, "w").write(re.sub(r'timeout-minutes: \d+', 'timeout-minutes: 10', s, count=1))
PY
arm "RED-3  timeout below the 30m floor" FAIL "$AUDIT" "below the 30-minute floor"
revert .github/workflows/rtl-const-expr-cert-gate.yml

# ---------------------------------------------------------------- RED-4: the recipe gets a 2nd home
printf '      - name: Inline regeneration\n        run: make -C rust regex_parser_bootstrap\n' \
  >> "$ROOT/.github/workflows/performance-gate.yml"
arm "RED-4  recipe re-inlined in workflow" FAIL "$AUDIT" "regex_parser_bootstrap"
revert .github/workflows/performance-gate.yml

# ---------------------------------------------------------------- RED-5: the single home is renamed
python3 - "$ROOT/rust/Makefile" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
open(p, "w").write(s.replace("regenerate_generated_parsers:", "renamed_away_target:", 1))
PY
arm "RED-5  make target renamed away" FAIL "$AUDIT" "rust/Makefile"
revert rust/Makefile

# ---------------------------------------------------------------- RED-6: a workflow the audit never saw
# ⭐ THE ARM THAT PROVES THE ROSTER IS DERIVED. Tracked via a COPY of the index, so the real
# staging area is never touched.
cat > "$ROOT/$NEW_WORKFLOW" <<'YML'
name: zz-probe-synthetic-gate
on:
  workflow_dispatch:
jobs:
  zz-probe-synthetic-gate:
    runs-on: ubuntu-latest
    timeout-minutes: 60
    steps:
      - name: Checkout
        uses: actions/checkout@v5
      - name: Run a gate that compiles the crate
        run: make -C rust SHELL=/bin/bash sota_exit_gate
YML
cp "$ROOT/.git/index" "$ROOT/$TMP_INDEX"
GIT_INDEX_FILE="$ROOT/$TMP_INDEX" git add -- "$NEW_WORKFLOW" >/dev/null 2>&1
GIT_INDEX_FILE="$ROOT/$TMP_INDEX" arm "RED-6  NEW tracked workflow, no step" FAIL "$AUDIT" "zz-probe-synthetic-gate"

# ---------------------------------------------------------------- RED-7: the local gate stops delegating
cp "$ROOT/.git/index" "$ROOT/$TMP_INDEX"
python3 - "$ROOT/$GATE" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
open(p, "w").write(s.replace(
    'make -C rust SHELL=/bin/bash "$REGENERATION_MAKE_TARGET"',
    'make -C rust SHELL=/bin/bash regex_parser_bootstrap', 1))
PY
arm "RED-7  local gate stops delegating" FAIL "$AUDIT" "ci_workflow_local_gate.sh"
revert "$GATE"

# ---------------------------------------------------------------- RED-9: the PREPARE default erodes
# ⭐ The precedent this guards against is measured, not hypothetical: `PGEN_CLIPPY_GENERATED_STRICT`
# defaulted to `0` and was set by nothing, leaving a 291 → 0 correctness win unguarded from the day
# it landed. ⚠️ The FIRST implementation of this guard was an `assert_file_not_contains` naming the
# forbidden literal — which put that literal into the very file it forbade it from, and 8 of these
# 12 arms failed on the audit tripping over its own source. The guard now compares the extracted
# VALUE instead of matching text.
python3 - "$ROOT/$GATE" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
open(p, "w").write(s.replace(
    'PREPARE_RAW="${PGEN_CI_WORKFLOW_LOCAL_PREPARE:-true}"',
    'PREPARE_RAW="${PGEN_CI_WORKFLOW_LOCAL_PREPARE:-false}"', 1))
PY
arm "RED-9  PREPARE default flipped off" FAIL "$AUDIT" "expected 'true'"
revert "$GATE"

# ---------------------------------------------------------------- RED-8: vacuous roster must REFUSE
# The corollary this whole tree keeps re-deriving: an audit that inspects NOTHING must say so, not
# return green. Simulated by untracking every workflow in the temporary index.
cp "$ROOT/.git/index" "$ROOT/$TMP_INDEX"
GIT_INDEX_FILE="$ROOT/$TMP_INDEX" git rm --cached -q -- '.github/workflows/*.yml' >/dev/null 2>&1
GIT_INDEX_FILE="$ROOT/$TMP_INDEX" arm "RED-8  empty roster must refuse" FAIL "$AUDIT" "matched no workflows at all"

# ---------------------------------------------------------------- CONTROL-1: unrelated edit, no misfire
# A gate that blames the nearest defect it knows about is worse than one that stays quiet.
printf '      - name: Unrelated probe step\n        run: echo hello\n' \
  >> "$ROOT/.github/workflows/performance-gate.yml"
arm "CTRL-1 unrelated workflow edit" PASS "$AUDIT"
revert .github/workflows/performance-gate.yml

# ---------------------------------------------------------------- CONTROL-2: no over-binding
# The pre-existing workflow audit must be unaffected by everything `.4` added.
arm "CTRL-2 pre-existing workflow audit" PASS "$CONTROL_AUDIT"

# ---------------------------------------------------------------- CONTROL-3: identical on the OLD gate
# `.4` must not have changed what `audit_workflow_surface` decides. Replay it against HEAD's gate.
OLD_PROBE="rust/scripts/.ciparity_regen_probe_old.sh"
git show "HEAD:$GATE" | sed 's/^main "\$@"$//' > "$ROOT/$OLD_PROBE"
old_out="$( ( set +e; . "$ROOT/$OLD_PROBE" >/dev/null 2>&1; "$CONTROL_AUDIT" ) 2>&1 )"; old_rc=$?
rm -f "$ROOT/$OLD_PROBE"
build_probe
new_out="$( ( set +e; . "$ROOT/$PROBE" >/dev/null 2>&1; "$CONTROL_AUDIT" ) 2>&1 )"; new_rc=$?
if [ "$old_rc" -eq "$new_rc" ]; then
  pass=$((pass + 1)); printf '✓ %-34s identical verdict (rc=%d) on HEAD gate and working tree\n' \
    "CTRL-3 old-vs-new control audit" "$new_rc"
else
  fail=$((fail + 1)); printf '✗ %-34s HEAD rc=%d, working tree rc=%d\n' \
    "CTRL-3 old-vs-new control audit" "$old_rc" "$new_rc"
fi

printf '%s\n' "------------------------------------------------------------------------------"
printf 'arms=%d  PASS=%d  FAIL=%d\n' "$((pass + fail))" "$pass" "$fail"
[ "$fail" -eq 0 ]
