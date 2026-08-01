#!/usr/bin/env bash
# LESSON-RETRIEVAL.4 — GROUND TRUTH for scripts/check_lesson_promotion.sh.
#
# A gate that has never been seen to FAIL is decoration. This drives the REAL shipping script
# end-to-end against a scratch git repo (repo-volume, derived from the repository root — never
# /tmp), staging real index states so `git diff --cached` is genuinely exercised.
#
# Cases:
#   blocked_no_decision   exit 1  a new dated lesson with neither promotion nor decline
#   promoted_knowledge    exit 0  ... plus a docs/knowledge/ card                [PROMOTE path a]
#   promoted_decision     exit 0  ... plus `answers:` added to a decision record [PROMOTE path a']
#   declined_token        exit 0  ... plus `promotion: declined (…)` in a leaf   [DECLINE path b]
#   edit_not_new_lesson   exit 0  editing an EXISTING entry is not a new lesson  [no false positive]
#   unrelated_commit      exit 0  a commit that never touches DEVELOPMENT_NOTES  [no false positive]
#
# Usage: bash docs/tasks/artifacts/lesson_retrieval/run_lesson_promotion_probes.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
CHECKER="$ROOT/scripts/check_lesson_promotion.sh"
LAB="$ROOT/rust/target/lesson_promotion_probe"

rm -rf "$LAB"; mkdir -p "$LAB/scripts" "$LAB/docs/knowledge" "$LAB/docs/tasks" "$LAB/docs/decisions"
cp "$CHECKER" "$LAB/scripts/"
cd "$LAB" || exit 2
git init -q . && git config user.email p@p && git config user.name p

# A baseline commit so the "edit an existing entry" case has something to edit.
printf '# DEVELOPMENT_NOTES.md\n\n## 2026-01-01 - PGEN-OLD-0001 — an older lesson\n\nbody text\n' >DEVELOPMENT_NOTES.md
printf -- '---\nname: d\n---\nbody\n' >docs/decisions/rec.md
printf '# T\n' >docs/tasks/TREE.md
git add -A >/dev/null && git commit -qm base

failures=0
run_case() {
    local name="$1" want="$2"; shift 2
    git reset -q; git checkout -q -- . 2>/dev/null || true
    "$@"                      # stage the case's index state
    local got=0
    bash scripts/check_lesson_promotion.sh >"$LAB/$name.log" 2>&1 || got=$?
    local verdict="PASS"
    if [ "$got" != "$want" ]; then verdict="FAIL"; failures=$((failures + 1)); fi
    printf '[lesson-promotion-probe] %-21s exit=%s (expected %s)  %s\n' "$name" "$got" "$want" "$verdict"
}

add_lesson() {
    printf '\n## 2026-08-01 - PGEN-PROBE-0001 — a brand new lesson\n\nbody\n' >>DEVELOPMENT_NOTES.md
    git add DEVELOPMENT_NOTES.md
}

run_case blocked_no_decision 1 add_lesson

run_case promoted_knowledge 0 bash -c '
    printf "\n## 2026-08-01 - PGEN-PROBE-0001 — a brand new lesson\n\nbody\n" >>DEVELOPMENT_NOTES.md
    printf -- "---\nid: x\ntitle: t\ndate: 2026-08-01\nanswers:\n  - \"q\"\nreverify: true\n---\nbody\n" >docs/knowledge/x.md
    git add DEVELOPMENT_NOTES.md docs/knowledge/x.md'

run_case promoted_decision 0 bash -c '
    printf "\n## 2026-08-01 - PGEN-PROBE-0001 — a brand new lesson\n\nbody\n" >>DEVELOPMENT_NOTES.md
    printf -- "---\nname: d\nanswers:\n  - \"q\"\n---\nbody\n" >docs/decisions/rec.md
    git add DEVELOPMENT_NOTES.md docs/decisions/rec.md'

run_case declined_token 0 bash -c '
    printf "\n## 2026-08-01 - PGEN-PROBE-0001 — a brand new lesson\n\nbody\n" >>DEVELOPMENT_NOTES.md
    printf "# T\npromotion: declined (per-slice history, not general)\n" >docs/tasks/TREE.md
    git add DEVELOPMENT_NOTES.md docs/tasks/TREE.md'

# ⛔ REGRESSION CONTROL, added after the gate's FIRST REAL RUN passed on exactly this: a leaf that
# merely DOCUMENTS the token in prose is a MENTION, not a decision. A gate its own documentation can
# satisfy is decoration.
stage_documentation_mention() {
    # A shell FUNCTION, not `bash -c '…'`: the prose contains backticks, and inside the double
    # quotes of a nested `bash -c` they become command substitution — which silently ate the token
    # and made this very case pass while testing nothing. Quoting is kept under local control.
    printf '\n## 2026-08-01 - PGEN-PROBE-0001 — a brand new lesson\n\nbody\n' >>DEVELOPMENT_NOTES.md
    printf '%s\n' '# T' 'write `promotion: declined (<reason>)` in the owning leaf to decline.' >docs/tasks/TREE.md
    grep -q 'promotion: declined (<reason>)' docs/tasks/TREE.md \
      || { echo "probe setup FAILED: the mention was not written" >&2; exit 2; }
    git add DEVELOPMENT_NOTES.md docs/tasks/TREE.md
}
run_case documentation_mention 1 stage_documentation_mention

run_case edit_not_new_lesson 0 bash -c '
    printf "# DEVELOPMENT_NOTES.md\n\n## 2026-01-01 - PGEN-OLD-0001 — an older lesson\n\nEDITED body text\n" >DEVELOPMENT_NOTES.md
    git add DEVELOPMENT_NOTES.md'

run_case unrelated_commit 0 bash -c '
    printf "# T\nunrelated edit\n" >docs/tasks/TREE.md
    git add docs/tasks/TREE.md'

echo
if [ "$failures" -gt 0 ]; then
    echo "[lesson-promotion-probe] $failures case(s) FAILED — the gate lacks ground truth"
    exit 1
fi
echo "[lesson-promotion-probe] all 7 cases PASSED — the gate blocks a silent omission, accepts both"
echo "                         promotion paths and an explicit decline, and does not false-positive on"
echo "                         an edited entry or an unrelated commit"
