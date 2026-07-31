#!/usr/bin/env bash
# scripts/check_readme_stability.sh
# README-STABILITY (PGEN-README-POLICY-0001): keep README.md a stable LANDING PAGE instead of
# letting it grow into a changelog, roadmap, gate catalog or documentation inventory.
# Policy: docs/reference/PGEN_README_STABILITY_POLICY.md. Tree: docs/tasks/README-POLICY.md.
# Exits NONZERO on any breach. Called by scripts/check_doctrines.sh via .githooks/pre-commit (E3)
# and CI (E4, memory-architecture-gate.yml, which invokes the whole driver on every push).
#
# ⭐ WHY THIS EXISTS — measured, not assumed (2026-07-30, adoption):
#   README.md was 510 lines / 48,811 bytes, and NO instrument watched it. Every guard that
#   touches the file watches something else:
#     - scripts/check_diagnostics_and_docpaths.sh:52  audits doc PATHS written inside it
#     - rust/scripts/ci_workflow_local_gate.sh:277    audits which root markdown files EXIST
#   A README can triple in size with both fully green. 55.6% of the file was an exhaustive file
#   inventory plus an operations manual, and ONE bullet (README.md:115) was 4,369 bytes — 9.0%
#   of the whole file — on a SINGLE LINE.
#
# ⭐⭐ WHY BOTH CAPS, AND WHY A LINE CAP ALONE IS NOT ENOUGH — measured in this repo, today:
#   scripts/check_memory_architecture.sh:18-19 caps layer-A MEMORY.md at 60 LINES and hard-fails
#   past it. Measured at adoption (2026-07-30): MEMORY.md was 60 lines (passing, exactly at the
#   ceiling) and 149,779 BYTES — 2,496 bytes per line, with no byte bound at all. The line cap held
#   the line count and held back nothing. (The exact byte figure moves every session; the invariant
#   is that layer A sits AT its line cap carrying six figures of unbounded bytes.) That bypass is the whole reason this guard shipped with both caps from day one; the
#   same class was found independently at README.md:115. Holding layer A to this standard is
#   tracked as README-POLICY.2.
#
# ⛔ NEVER raise a cap to land new content. Move the detail to its canonical home (the routing
#   table below and in the policy doc). A cap increase requires an explicit reviewed decision,
#   recorded in docs/tasks/README-POLICY.md, that the landing-page contract itself expanded.
#
# Contract compliance (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict; breaches explain on
# stderr with a routing hint; deterministic (no clock, no network, no randomness); NON-MUTATING;
# repo-root resolved from BASH_SOURCE; reads the WORKING TREE rather than the git index, so it is
# never vacuous on a hosted push the way an index-scoped doctrine is.
#
# ⚠️ DO NOT name the git index-diff flags literally anywhere in this file, not even in prose or
# inside a quoted regex. The driver's vacuity classifier (scripts/check_doctrines.sh, the grep
# just above its "vacuous" array) text-matches the enforcer's SOURCE, comments included. TWO
# successive revisions of this header tripped it: the first mentioned the flag while explaining
# that this check does not use it, and the second quoted the classifier's own pattern while
# documenting the first. Both times the driver reported README-STABILITY as having "evaluated
# NOTHING" — a script mislabelled by documenting its own correctness, twice. The driver states
# this limit honestly in its own comments (a mislabel costs an inaccurate NAME in an
# informational note, never a verdict); the classifier's comment-blindness is routed to
# README-POLICY.3 and is deliberately NOT worked around here.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"

# The reviewed caps. Chosen AFTER the adoption trim (178 lines / 8,287 bytes), leaving ~24%
# headroom on each — deliberately proportional, so neither cap is the soft one.
LINE_CAP="${README_LINE_CAP:-220}"
BYTE_CAP="${README_BYTE_CAP:-10240}"
TARGET="README.md"
POLICY="docs/reference/PGEN_README_STABILITY_POLICY.md"

fail=0
note(){ printf 'readme-stability: %s\n' "$1" >&2; fail=1; }

# ------------------------------------------------------------------ refuse rather than skip
# A skip is never a pass: if the landing page or its policy is gone, this check cannot judge
# anything, and returning 0 would report "the doctrine holds" over an absence.
if [ ! -f "$TARGET" ]; then
  printf 'readme-stability: REFUSED — %s is missing; the landing page is the thing this doctrine governs.\n' \
    "$TARGET" >&2
  exit 2
fi
if [ ! -f "$POLICY" ]; then
  printf 'readme-stability: REFUSED — %s is missing; the caps above would be unreviewable numbers.\n' \
    "$POLICY" >&2
  exit 2
fi

lines=$(wc -l < "$TARGET" | tr -d ' ')
bytes=$(wc -c < "$TARGET" | tr -d ' ')

routing_hint() {
  cat >&2 <<'HINT'
                    Route the new detail to its canonical home instead of growing the landing page:
                      gate recipes / make targets ....... docs/book/src/gate-flow.md
                      operational procedure ............. docs/book/src/operations-and-governance.md
                      repository layout / paths ......... docs/book/src/developer-architecture.md
                      per-parser books and their gates .. docs/book/src/parser-families.md
                      family status / Done-bar claims ... rust/test_data/grammar_quality/done_bar_family_register_v0.json
                        (the CLAIM; its published view is docs/book/src/roadmap-and-live-status.md)
                      release history ................... CHANGES.md
                      design rationale .................. docs/decisions/
                      normative spec / contracts ........ docs/reference/, docs/contracts/
                      doc inventories / indexes ......... docs/book/src/documentation-model.md
                      current work and priorities ....... docs/tasks/, docs/TASK_TREE.md
                    Full policy: docs/reference/PGEN_README_STABILITY_POLICY.md
HINT
}

# ------------------------------------------------------------------ E1 — the line cap
if [ "$lines" -gt "$LINE_CAP" ]; then
  note "$TARGET is $lines lines (> cap $LINE_CAP)."
  routing_hint
fi

# ------------------------------------------------------------------ E2 — the byte cap
# Complements E1: wrapped prose cannot bypass the byte budget, and a single very long line
# cannot bypass the line budget. Neither cap is redundant with the other.
if [ "$bytes" -gt "$BYTE_CAP" ]; then
  note "$TARGET is $bytes bytes (> cap $BYTE_CAP)."
  routing_hint
fi

# ------------------------------------------------------------------ E3 — changelog leakage
# ⚠️ HONEST BOUND, stated because this repository has been bitten by cheap textual proxies
# standing in for the real fact: this is NOT a general "is this changelog content?" oracle. It
# detects exactly ONE measured leakage class — the dated historical annotation. At adoption
# README.md carried 6 of them ("demoted to Mostly Done on 2026-07-29 (DONE-BAR.2b)", ...), each
# a CHANGES.md / status-tracker row living on the landing page. A date on a landing page is
# release history; the escape is to move it, not to weaken this check.
#
# ⭐ LIVE-MEANS-LIVE.1c2 — this rule used to route status overflow into LIVE_ACHIEVEMENT_STATUS.md,
# and that redirect is exactly how the rot happened: README.md was capped on two axes while the file
# it overflowed INTO had no instrument at all, and it reached 1 547 057 B of which 94.7 % was a dated
# changelog. Overflow now lands in a SCHEMA-BOUNDED field, which cannot accumulate a changelog.
# ⇒ capping a file is only half a fix; the other half is checking where the overflow lands.
dated=$(grep -cE '20[0-9]{2}-[0-9]{2}-[0-9]{2}' "$TARGET" || true)
if [ "${dated:-0}" -gt 0 ]; then
  note "$TARGET carries $dated date-stamped line(s) — release history belongs in CHANGES.md and family status in the DONE-BAR register (rust/test_data/grammar_quality/done_bar_family_register_v0.json)."
  grep -nE '20[0-9]{2}-[0-9]{2}-[0-9]{2}' "$TARGET" | head -5 | sed 's/^/                      /' >&2
fi

# ------------------------------------------------------------------ E4 — the policy is reachable
# The caps are only defensible if a reader can find the reviewed decision behind them. If the
# README stops naming the policy, the numbers above become folklore.
if ! grep -q "$POLICY" "$TARGET"; then
  note "$TARGET no longer links $POLICY — the caps must stay traceable to the reviewed decision that set them."
fi

if [ "$fail" -eq 0 ]; then
  printf 'readme-stability: OK — %s is %s/%s lines, %s/%s bytes.\n' \
    "$TARGET" "$lines" "$LINE_CAP" "$bytes" "$BYTE_CAP"
fi
exit "$fail"
