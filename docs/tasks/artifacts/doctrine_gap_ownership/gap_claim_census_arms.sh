#!/usr/bin/env bash
# gap_claim_census_arms.sh — the END-TO-END two-arm control for `GAP-CLAIM-CENSUS`
# (`scripts/check_gap_claims.sh`, leaf `DOCTRINE-GAP-OWNERSHIP.15`).
#
# ⭐ WHY A SECOND CONTROL EXISTS BESIDE `--self-test`. Arms 1-7 of the self-test call the classifier
#   DIRECTLY, so they prove the predicate and the discharge — but they bypass the git plumbing the
#   real check runs on: reading the INDEX copy of the file, and resolving a `-U0` hunk header to
#   NEW-file line numbers. Arm 8 tests that arithmetic in isolation. Nothing tested the two TOGETHER
#   on a genuinely staged file, and *a control that cannot fail is not a control*
#   ([[a-control-that-cannot-fail-is-not-a-control]]).
#
# ⛔ IT NEVER TOUCHES THE CALLER'S INDEX. A control that stages into the real index can leave residue
#   in a tree an operator is mid-commit on. Everything here runs against a THROWAWAY `GIT_INDEX_FILE`
#   under `rust/target/` (repo volume, CLAUDE.md §13); the caller's `git status` is unchanged, which
#   this script ASSERTS rather than assumes before it exits.
#
# ARMS
#   RED   — a staged leaf asserting an absence, with no census in its section  -> reported, rc 1
#   GREEN — the SAME file with one census line appended                        -> clean,    rc 0
#   The GREEN arm is what makes the RED arm mean something: without it, RED could be red for the
#   fixture rather than for the missing census.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT" || exit 1

WORK="rust/target/gap_claim_census/arms"
FIXTURE="docs/tasks/ZZ-GAP-CLAIM-ARMS-FIXTURE.md"
mkdir -p "$WORK"
SCRATCH_INDEX="$ROOT/$WORK/index"
export GIT_INDEX_FILE="$SCRATCH_INDEX"

# ⛔ cleanup must NOT reference GIT_INDEX_FILE: the hygiene arm below UNSETS it, and under `set -u`
# an EXIT trap reading an unset variable dies with "unbound variable" AFTER the arms have reported —
# which is exactly the shape where a control leaves residue while looking like it ran.
cleanup() { rm -f "$ROOT/$FIXTURE" "$SCRATCH_INDEX"; }
trap cleanup EXIT

# Seed the throwaway index from HEAD so the only staged change is the fixture.
git read-tree HEAD || { echo "arms: could not seed a throwaway index" >&2; exit 2; }

arms=0; passed=0

cat > "$FIXTURE" <<'EOF'
# fixture (temporary — this file is deleted by the arms script)

### `.1` — a leaf that asserts an absence and proves nothing

- **THE GAP** — nothing reads the `@sample` annotation, so the samples rot unnoticed.
EOF
git add -- "$FIXTURE"

arms=$((arms + 1))
red_out="$(scripts/check_gap_claims.sh 2>&1)"; red_rc=$?
if [ "$red_rc" = 1 ] && printf '%s' "$red_out" | grep -q "$FIXTURE"; then
  passed=$((passed + 1))
  printf 'GAP-CLAIM-CENSUS-ARMS: RED   arm — an unbacked staged claim is REPORTED (rc %s) and the line is quoted\n' "$red_rc"
else
  printf 'GAP-CLAIM-CENSUS-ARMS: RED   arm FAILED — rc %s, expected 1 with the fixture line quoted\n' "$red_rc" >&2
  printf '%s\n' "$red_out" >&2
fi

printf -- '- **CENSUS** — `git grep -c "@sample" -- rust/src scripts` returns 0 over both paths.\n' >> "$FIXTURE"
git add -- "$FIXTURE"

arms=$((arms + 1))
green_out="$(scripts/check_gap_claims.sh 2>&1)"; green_rc=$?
if [ "$green_rc" = 0 ]; then
  passed=$((passed + 1))
  printf 'GAP-CLAIM-CENSUS-ARMS: GREEN arm — one pasted census discharges the SAME claim (rc 0)\n'
else
  printf 'GAP-CLAIM-CENSUS-ARMS: GREEN arm FAILED — rc %s; a censused claim must pass\n' "$green_rc" >&2
  printf '%s\n' "$green_out" >&2
fi

# ⛔ The caller's REAL index must be untouched. Asserted, not assumed: this is the property that makes
# the control safe to run in a dirty tree, and an assertion is cheaper than an operator discovering
# it. The worktree fixture is removed FIRST, so the arm judges what the control LEAVES, not what it
# is mid-way through using.
arms=$((arms + 1))
rm -f "$ROOT/$FIXTURE"
unset GIT_INDEX_FILE
if ! git ls-files --cached --error-unmatch -- "$FIXTURE" >/dev/null 2>&1 \
   && [ -z "$(git status --porcelain -- "$FIXTURE")" ]; then
  passed=$((passed + 1))
  printf "GAP-CLAIM-CENSUS-ARMS: HYGIENE arm — the caller's real index and worktree are untouched\n"
else
  printf 'GAP-CLAIM-CENSUS-ARMS: HYGIENE arm FAILED — the fixture leaked into the real index/tree\n' >&2
fi

printf 'GAP-CLAIM-CENSUS-ARMS: arms=%s/%s\n' "$passed" "$arms"
[ "$passed" = "$arms" ] || exit 1
exit 0
