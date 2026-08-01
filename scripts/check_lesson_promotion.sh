#!/usr/bin/env bash
# LESSON-PROMOTION — a durable lesson written to DEVELOPMENT_NOTES.md must be either
# PROMOTED into the retrievable layer or EXPLICITLY DECLINED, never silently dropped.
#
# Provenance: director directive 2026-08-01 (session #230), LESSON-RETRIEVAL.4. Reviewing
# PGEN-SV-EXH-PROOF-0166 the director asked whether the sharp conclusions are "stored, saved,
# accessible somewhere (book? task-tree? KM?)" and then that "you made a lot of those
# conclusions, these are lesson-learned material."
#
# THE MEASURED DEFECT this exists to stop (LESSON-RETRIEVAL.1):
#   DEVELOPMENT_NOTES.md carried 1 592 dated lesson entries across 62 191 lines and is NOT a
#   Knowledge Map scan dir, so not one of them was reachable by question. docs/decisions/ IS a
#   scan dir and 0 of its 142 records carried `answers:`. The promotion mechanism existed, was
#   wired, and was skipped 1 592 times -- silently, because no gate asked. KNOWLEDGE-MAP checks
#   the map is in sync with its SOURCES; it never asks whether a lesson REACHED a source.
#   ⇒ The director chose a DOCTRINE CHECK over a COMMIT.md reminder for exactly that reason:
#   a reminder has already lost 1 592 times.
#
# THE RULE
#   If a commit stages a NEW dated lesson heading in DEVELOPMENT_NOTES.md (`## <date> - <slice>`),
#   it must ALSO stage EITHER
#     (a) a change under docs/knowledge/ or a docs/decisions/ record gaining `answers:`  [PROMOTED]
#     (b) a `promotion: declined (<reason>)` token in a staged docs/tasks/*.md leaf       [DECLINED]
#   Otherwise: block, and name the entry.
#
# ⛔ DECLINING IS A FIRST-CLASS, EXPECTED OUTCOME -- most lessons are per-slice HISTORY and
# belong exactly where they are. This gate does not demand promotion; it demands a DECISION.
# The promotion criterion (durable + general + re-verifiable + question-shaped) lives in
# docs/tasks/LESSON-RETRIEVAL.md.
#
# ARCHETYPE: evidence (DOCTRINE_ENFORCEMENT.md §3).
#   HONEST LIMIT -- stated rather than hidden, as every evidence check must: this verifies a
#   DECISION WAS RECORDED, not that the decision was correct. A lazy `promotion: declined (n/a)`
#   passes. What it makes impossible is the SILENT omission, which is the measured failure.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict; explains on stderr;
# deterministic; read-only; staged-scope-aware; path-agnostic; fast.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

NOTES="DEVELOPMENT_NOTES.md"
DECLINE_TOKEN="promotion: declined"

# --- the decision, as a pure function so it can carry ground truth (see the controls below) ---
# $1 = count of newly added dated lesson headings
# $2 = 1 if the commit stages a promotion, else 0
# $3 = 1 if the commit stages an explicit decline, else 0
# echoes: ok | needs-decision
lesson_promotion_verdict() {
    local added="$1" promoted="$2" declined="$3"
    if [ "$added" -eq 0 ]; then printf 'ok\n'; return 0; fi
    if [ "$promoted" -eq 1 ] || [ "$declined" -eq 1 ]; then printf 'ok\n'; return 0; fi
    printf 'needs-decision\n'
}

# GROUND TRUTH, run on every invocation (microseconds): an instrument with no ground truth is a
# confident guess (docs/decisions/feedback_instrument_needs_ground_truth.md). Both the positive
# control AND the negatives are pinned; a miss REFUSES rather than reporting a clean tree.
lesson_promotion_self_check() {
    local spec want got misses=0
    for spec in "0:0:0:ok" "0:1:0:ok" "3:1:0:ok" "3:0:1:ok" "3:1:1:ok" \
                "1:0:0:needs-decision" "9:0:0:needs-decision"; do
        want="${spec##*:}"
        got="$(lesson_promotion_verdict "$(echo "$spec" | cut -d: -f1)" \
                                        "$(echo "$spec" | cut -d: -f2)" \
                                        "$(echo "$spec" | cut -d: -f3)")"
        if [ "$got" != "$want" ]; then
            printf 'lesson-promotion: CONTROL MISSED: %s expected=%s got=%s\n' "$spec" "$want" "$got" >&2
            misses=$((misses + 1))
        fi
    done
    if [ "$misses" -gt 0 ]; then
        printf 'lesson-promotion: the gate does not discriminate (%d control(s) missed); refusing\n' "$misses" >&2
        exit 2
    fi
}
lesson_promotion_self_check

# No staged set (e.g. a manual run outside a commit) => nothing to judge.
staged="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null || true)"
[ -n "$staged" ] || exit 0
printf '%s\n' "$staged" | grep -qx "$NOTES" || exit 0

# Newly ADDED dated lesson headings only -- reflowing or editing an existing entry is not a new
# lesson. `-U0` keeps the diff to changed lines; the `+` prefix keeps it to additions.
added_entries="$(git diff --cached -U0 -- "$NOTES" 2>/dev/null \
                 | grep -E '^\+## [0-9]{4}-[0-9]{2}-[0-9]{2}' | sed 's/^\+//' || true)"
added_count="$(printf '%s' "$added_entries" | grep -c . || true)"
[ "$added_count" -eq 0 ] && exit 0

# (a) PROMOTED: a docs/knowledge/ change, or a docs/decisions/ record gaining `answers:`.
promoted=0
printf '%s\n' "$staged" | grep -qE '^docs/knowledge/.*\.md$' && promoted=1
if [ "$promoted" -eq 0 ]; then
    git diff --cached -U0 -- docs/decisions 2>/dev/null | grep -qE '^\+answers:' && promoted=1
fi

# (b) DECLINED: an explicit token in a staged task leaf, carrying a REAL reason.
#
# ⛔ A MENTION IS NOT A DECISION. The first real-world run of this gate passed on a leaf that merely
# DOCUMENTED the token as `promotion: declined (<reason>)` in prose about the gate itself — a gate
# satisfied by its own documentation is decoration. So the placeholder is rejected explicitly, and
# the reason must be non-empty.
declined=0
for f in $(printf '%s\n' "$staged" | grep -E '^docs/tasks/.*\.md$' || true); do
    [ -r "$f" ] || continue
    if grep -E "${DECLINE_TOKEN} \(..*\)" "$f" | grep -vqF "${DECLINE_TOKEN} (<reason>)"; then
        declined=1; break
    fi
done

if [ "$(lesson_promotion_verdict "$added_count" "$promoted" "$declined")" = "ok" ]; then
    exit 0
fi

{
    printf 'lesson-promotion: %d new lesson entr%s in %s with NO promotion and NO explicit decline:\n' \
        "$added_count" "$([ "$added_count" -eq 1 ] && echo 'y' || echo 'ies')" "$NOTES"
    printf '%s\n' "$added_entries" | sed 's/^/    /'
    printf '\n'
    printf 'A durable lesson that is written down but not retrievable is the measured defect this\n'
    printf 'gate exists to stop: 1592 entries accumulated in %s -- which is NOT a\n' "$NOTES"
    printf 'Knowledge Map scan dir -- and 0 of 142 docs/decisions/ records carried `answers:`.\n\n'
    printf 'Do ONE of:\n'
    printf '  PROMOTE  add docs/knowledge/<slug>.md (front matter at LINE 1) with `answers:` +\n'
    printf '           a runnable `reverify:`, or add `answers:` to a docs/decisions/ record,\n'
    printf '           then regenerate: bash knowledge-map/scripts/gen_knowledge_map.sh\n'
    printf '  DECLINE  write `%s (<reason>)` in the owning docs/tasks/*.md leaf.\n' "$DECLINE_TOKEN"
    printf '\n'
    printf 'DECLINING IS EXPECTED AND FINE -- most lessons are per-slice history and belong exactly\n'
    printf 'where they are. Promote only what is DURABLE (still true after the slice lands), GENERAL\n'
    printf '(reusable beyond one grammar), RE-VERIFIABLE (a command re-proves it) and QUESTION-SHAPED.\n'
    printf 'Criterion: docs/tasks/LESSON-RETRIEVAL.md.\n'
} >&2
exit 1
