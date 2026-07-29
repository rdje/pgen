#!/usr/bin/env bash
# ROUTING-EVIDENCE — a task leaf that routes a finding OUT to a DIFFERENT task tree
# must record what it measured before deciding the finding belongs to that family.
#
# Provenance: CI-PARITY-GATE-ROT.9 (session #221). `-0015` routed
# `stimuli regex target accounting mismatch (723 + 31 != 1033)` to the regex family as
# *"the regex family's stimuli target-accounting MODEL, not gate wiring"*. It was neither:
# the defect was in the SHARED closed-loop gate, the same run got the `ebnf` row wrong the
# same way, and the regex accounting was never wrong. The evidence that settled it —
# cross-grammar arithmetic over the run's OWN summary.csv — was already on disk at routing
# time. The misroute cost a session.
#
# THE RULE
#   If a staged `docs/tasks/A.md` adds a line that routes a finding to a DIFFERENT tree,
#   that file must contain a `ROUTING EVIDENCE` section. Otherwise: block, and quote the line.
#
#   The discriminator is routing OUT OF THE TREE, deliberately. `routed to `.4`` (a leaf of the
#   same tree) is ordinary intra-tree bookkeeping and is the common case — measured 17 such lines
#   in the corpus, of which this predicate flags 0. Flagging those would be noise, and a check
#   that cries wolf gets bypassed.
#
# ⚠️ THE FIRST CUT OF THIS CHECK WOULD NOT HAVE CAUGHT ITS OWN FOUNDING INCIDENT.
#   It required a routing verb and the destination TREE ID on the same line. `-0015` names a
#   FAMILY, not a tree file ("BELONGS TO ANOTHER FAMILY", "Filed against the regex family"), so
#   the conjunction missed it — a check calibrated against an imagined phrasing instead of the
#   real one. Caught by replaying the actual commit before shipping. Calibration as it now
#   stands, measured over the whole tracked corpus: 11 lines flagged across 4 trees, the three
#   real `-0015` routing lines among them, and 0 of the 17 intra-tree routings.
#
# WHAT THE EVIDENCE SHOULD SAY (the question the misroute never asked):
#   does the finding REPRODUCE OUTSIDE the family it is being routed to? A defect that also
#   fires for another grammar/family is not that family's defect.
#
# ⚠️ KNOWN FALSE-POSITIVE CLASS, measured on this check's OWN commit: a leaf that merely
#   DISCUSSES routing — quoting the trigger phrases, as the leaf documenting this doctrine must —
#   fires it. That is the self-referential shape already recorded as
#   [[reference_self_referential_assertion_is_unsound]] ("an assertion about a file cannot live
#   inside that file as a literal"), which this repo PRICED at one site and deliberately did not
#   mechanize around. Accepted here on the same reasoning: the discharge is one section, it errs
#   toward asking rather than staying silent, and narrowing the predicate to exclude quotation
#   would re-introduce exactly the phrasing-guessing that made the first cut miss `-0015`.
#   Stated, not hidden.
#
# ARCHETYPE: evidence (DOCTRINE_ENFORCEMENT.md §3).
#   HONEST LIMIT — this verifies the reasoning was RECORDED, not that the reproduction was
#   attempted or that its conclusion was right. Stated rather than hidden; it is the bound
#   every evidence check carries. It would have caught `-0015`, because `-0015` recorded no
#   cross-family check at all — not because it can tell a good check from a bad one.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict; explains on stderr;
# deterministic; read-only; staged-scope-aware; path-agnostic; fast.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

# Staged task-tree files only. No staged set (e.g. a manual run) => nothing to judge.
staged="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null \
          | grep -E '^docs/tasks/[^/]*\.md$' || true)"
[ -n "$staged" ] || exit 0

# The routing-OUT predicate. Keyed on the SEMANTICS of leaving the tree ("out", "another
# family", "that family's tree"), not on the destination being spelled as a tree ID — that
# spelling assumption is what made the first cut miss its own founding incident.
ROUTE_OUT_RE='routed out|routing out|route(d)? it to (that|another)|routed to (another|that) (family|tree)|belongs to another (family|tree)|filed against the [a-z]+ family'

fail=0
for file in $staged; do
  [ -r "$file" ] || continue

  # ADDED lines of this file's staged diff that assert routing a finding OUT of this tree.
  # `routed in` is excluded: that is the RECEIVING side recording what it was handed, not the
  # decision this doctrine governs.
  offending="$(git diff --cached -U0 -- "$file" 2>/dev/null \
               | grep '^+' | grep -v '^+++' \
               | grep -iE "$ROUTE_OUT_RE" \
               | grep -viE 'routed[- ]in' \
               | sed 's/^+//' | cut -c1-160 | sed 's/^/    /' || true)"
  [ -n "$offending" ] || continue

  # A recorded routing rationale discharges it.
  if grep -qE '^[^[:alnum:]]*ROUTING EVIDENCE|## .*[Rr]outing [Ee]vidence' "$file"; then
    continue
  fi

  fail=1
  {
    echo "ROUTING-EVIDENCE: $file routes a finding to ANOTHER task tree with no recorded routing evidence."
    echo "  routing statement(s):$offending"
    echo
    echo "  Add a 'ROUTING EVIDENCE' section to $file answering the question a misroute never asks:"
    echo "    1. does the finding REPRODUCE OUTSIDE the family you are routing it to?"
    echo "       (a defect that also fires for another grammar/family/parser is not that"
    echo "        family's defect — this is exactly how -0015 sent a shared-gate defect to the"
    echo "        regex tree, and the deciding evidence was already on disk)"
    echo "    2. what did you MEASURE to place it there — not what makes it plausible?"
    echo "    3. what would have to be true for the routing to be WRONG, and did you check it?"
    echo
    echo "  If you routed it on reasoning alone, say so in that section. An honest"
    echo "  'not checked outside this family' is a legal answer and a useful signal;"
    echo "  a silent routing is not."
  } >&2
done

[ "$fail" -eq 0 ] || exit 1
exit 0
