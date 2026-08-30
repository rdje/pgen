#!/usr/bin/env bash
# run_family_status_gate_probes.sh — prove the FAMILY-STATUS-CURRENCY doctrine
# (`scripts/check_family_status.sh`, task-tree leaf `DONE-BAR-VISIBILITY.4`) actually BITES.
#
# ⛔ A GREEN GATE PROVES NOTHING UNTIL ITS REFUSALS FIRE. Every arm asserts BOTH the exit code AND a
# substring of the message, because pass/fail alone hides an arm reaching the right verdict for the
# wrong reason. Tier A's states are CONSTRUCTED under a scratch pages-root through the checker's own
# `PGEN_FAMILY_STATUS_PAGES_ROOT` seam, so no tracked page is ever mutated: a probe that edits a
# tracked file and restores it on exit leaves a dirty tree the moment it is interrupted.
#
#   bash docs/tasks/artifacts/done_bar_visibility/run_family_status_gate_probes.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

CHECK="$ROOT/scripts/check_family_status.sh"
WORK="$ROOT/rust/target/family_status_gate_probe"
rm -rf "$WORK"; mkdir -p "$WORK"

pass=0
fail=0
arm() {  # arm <label> <want-rc> <want-substring> <pages-root|-->
    local label="$1" want_rc="$2" want_substr="$3" root="$4"
    local out rc
    if [ "$root" = "--" ]; then out="$(bash "$CHECK" 2>&1)"; rc=$?
    else out="$(PGEN_FAMILY_STATUS_PAGES_ROOT="$root" bash "$CHECK" 2>&1)"; rc=$?; fi
    if [[ "$rc" == "$want_rc" && "$out" == *"$want_substr"* ]]; then
        printf '  ✅ %-58s exit=%s, message matched\n' "$label" "$rc"; pass=$((pass + 1))
    else
        printf '  ⛔ %-58s exit=%s (want %s)\n' "$label" "$rc" "$want_rc"
        printf '     wanted substring: %s\n' "$want_substr"
        printf '%s\n' "$out" | tail -n 5 | sed 's/^/     | /'; fail=$((fail + 1))
    fi
}

# Build a scratch mirror of the ten published pages: real content, mutable copies.
mirror() {  # mirror <dest>
    local dest="$1" fam
    rm -rf "$dest"; mkdir -p "$dest/docs"
    for fam in $(python3 -c "import json;print(' '.join(sorted(json.load(open('rust/test_data/grammar_quality/done_bar_family_register_v0.json'))['families'])))"); do
        mkdir -p "$dest/docs/${fam}_parser_book/src"
        cp "docs/${fam}_parser_book/src/status-and-distance.md" "$dest/docs/${fam}_parser_book/src/"
    done
}

echo "=============================================================================="
echo "FAMILY-STATUS-CURRENCY probe arms — scripts/check_family_status.sh"
echo "=============================================================================="

# CTRL-1 — the real tree passes, all three tiers. Without this the RED arms prove nothing.
arm "CTRL-1 the real tree passes all three tiers" 0 \
    "family-status: OK (all published family status blocks equal a fresh derivation)" --

# CTRL-2 — the scratch MIRROR of the real pages also passes, which is what makes the mutations below
# meaningful: they differ from a passing state by exactly one thing.
mirror "$WORK/clean"
arm "CTRL-2 an unmutated mirror passes tier A" 0 "tier A OK (10 pages" "$WORK/clean"

# RED-1 — a registered family with NO published page. The population is closed: a family added to the
# register and never published must FAIL, not be skipped.
mirror "$WORK/missing"; rm -rf "$WORK/missing/docs/vhdl_parser_book"
arm "RED-1 a registered family with no published page" 1 \
    "vhdl is a registered family with NO published status page" "$WORK/missing"

# RED-2 — a page belonging to no registered family. The other direction of the same closure.
mirror "$WORK/ghost"; mkdir -p "$WORK/ghost/docs/notafamily_parser_book/src"
cp docs/vhdl_parser_book/src/status-and-distance.md "$WORK/ghost/docs/notafamily_parser_book/src/"
arm "RED-2 a published page for no registered family" 1 \
    "is not a family in the done-bar register" "$WORK/ghost"

# RED-3 — ANOTHER family's block pasted into a page. Tier A catches this statically, for free,
# without paying for a derivation — the BEGIN marker names the family it was derived for.
mirror "$WORK/swapped"
cp docs/regex_parser_book/src/status-and-distance.md "$WORK/swapped/docs/vhdl_parser_book/src/status-and-distance.md"
arm "RED-3 another family's block in this family's book" 1 \
    "that is another family's block in vhdl's book" "$WORK/swapped"

# RED-4 — a page with the DERIVED markers duplicated. A check that cannot locate exactly one subject
# must SAY SO rather than compare the first one it finds.
mirror "$WORK/dup"
cat docs/vhdl_parser_book/src/status-and-distance.md docs/vhdl_parser_book/src/status-and-distance.md \
    > "$WORK/dup/docs/vhdl_parser_book/src/status-and-distance.md"
arm "RED-4 a page carrying two DERIVED blocks" 1 \
    "carries 2 BEGIN and 2 END DERIVED marker(s)" "$WORK/dup"

# RED-5 — a page with its markers stripped: the block cannot be located at all.
mirror "$WORK/noblock"
grep -v 'DERIVED' docs/vhdl_parser_book/src/status-and-distance.md \
    > "$WORK/noblock/docs/vhdl_parser_book/src/status-and-distance.md"
arm "RED-5 a page carrying no DERIVED block" 1 \
    "carries 0 BEGIN and 0 END DERIVED marker(s)" "$WORK/noblock"

echo "------------------------------------------------------------------------------"
printf 'family-status gate probes: %d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ] || exit 1
