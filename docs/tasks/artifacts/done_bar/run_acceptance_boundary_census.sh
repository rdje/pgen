#!/usr/bin/env bash
# run_acceptance_boundary_census.sh — measure `DONE-BAR.5d`'s surface before designing its gate.
#
# `.5d`'s charter, verbatim from this tree's leg-4 resolution: *"the family's acceptance boundary is
# documented where a consumer looks."* This census answers, per downstream integration contract:
#   (a) does it carry the repo's standard scope section (`## Scope / Non-Goals`)?
#   (b) does it carry a boundary disclosure naming what the parser gets WRONG (over/under-acceptance)?
#   (c) does ANY gate read its CONTENT, or only assert that the file exists?
#
# ⭐ The roster is DERIVED from a glob, never hand-listed — a hand-list cannot see a contract nobody
# added to it (`CI-PARITY-GATE-ROT.4`).
#
# ⭐⭐ CALIBRATION IS PART OF THE CENSUS. Five facts measured by hand on 2026-07-30 must reproduce, or
# the script prints MISCALIBRATED and exits 2 WITHOUT offering a verdict — because this instrument's
# first cut over-counted by matching `## Release … Highlights` headings as boundary sections (regex
# read 12, systemverilog 68), and a count nobody cross-checks is how six wrong answers shipped in
# `CI-PARITY-GATE-ROT.2`.
#
#   bash docs/tasks/artifacts/done_bar/run_acceptance_boundary_census.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

mapfile -t CONTRACTS < <(ls docs/contracts/PGEN_*_PARSER_INTEGRATION_CONTRACT.md 2>/dev/null | sort)
if [[ "${#CONTRACTS[@]}" -eq 0 ]]; then
    echo "MISCALIBRATED: derived contract roster is EMPTY — an empty roster is never a pass" >&2
    exit 2
fi

family_of() { basename "$1" | sed 's/^PGEN_//; s/_PARSER_INTEGRATION_CONTRACT\.md$//'; }

# A boundary disclosure is a section that names what the parser gets WRONG, not one that merely
# mentions the word "support". Measured spellings in this repo, deliberately narrow:
boundary_sections() {
    grep -cE '^#+ .*(Honest boundary|Known (limitation|gap)|Unsupported|Not supported|Over-accept|Under-accept)' "$1" || true
}
scope_sections() { grep -c '^## Scope / Non-Goals' "$1" || true; }

printf '%-30s %14s %20s\n' family scope_nongoals boundary_disclosure
printf '%-30s %14s %20s\n' "------" "-------------" "-------------------"
scope_total=0; boundary_total=0; scope_missing=()
for f in "${CONTRACTS[@]}"; do
    s=$(scope_sections "$f"); b=$(boundary_sections "$f")
    printf '%-30s %14s %20s\n' "$(family_of "$f")" "$s" "$b"
    [[ "$s" -gt 0 ]] && scope_total=$((scope_total + 1)) || scope_missing+=("$(family_of "$f")")
    [[ "$b" -gt 0 ]] && boundary_total=$((boundary_total + 1))
done
echo
printf 'contracts: %d | with a scope section: %d | with a boundary disclosure: %d\n' \
    "${#CONTRACTS[@]}" "$scope_total" "$boundary_total"
[[ "${#scope_missing[@]}" -gt 0 ]] && printf 'NO scope section: %s\n' "${scope_missing[*]}"

# (c) Does any gate read a contract's BOUNDARY? ⛔ An earlier cut of this census tried to split
# references into "existence-only" vs "content-reading" per LINE and reported 16/32 — which is WRONG:
# `check_published_version_currency.sh` demonstrably reads the contract's Contract Identity block, but
# it does so through a VARIABLE on a later line, so a per-line "is an extractor on this line" test
# cannot see it. The split was dropped rather than re-baselined. What follows is the measurement that
# actually answers `.5d`'s question and can be defended directly.
echo
echo "which scripts reference a contract at all (mode NOT inferred — see the note above):"
grep -rlE 'PGEN_[A-Z_]*_PARSER_INTEGRATION_CONTRACT' rust/scripts/ scripts/ 2>/dev/null \
    | while IFS= read -r f; do
        printf '  %-58s %s refs\n' "$f" "$(grep -cE 'PGEN_[A-Z_]*_PARSER_INTEGRATION_CONTRACT' "$f")"
      done
echo
echo "the SOUND measurement — does any gate name a boundary/scope SECTION?"
boundary_readers=$(grep -rlE 'Honest boundary|Scope / Non-Goals|Unsupported|Known limitation' \
    rust/scripts/ scripts/ 2>/dev/null | wc -l | tr -d ' ')
printf '  scripts naming any boundary/scope heading: %s\n' "$boundary_readers"
echo "  => a contract's acceptance boundary is read by NOTHING; the only contract content any gate"
echo "     reads is the regex Contract Identity pair (PUBLISHED-VERSION-CURRENCY) and the regex"
echo "     oracle-tuple anchors (REGEX-ORACLE-ANCHOR-SYNC) — neither is a boundary."

# ---- calibration: five hand-measured facts (2026-07-30) --------------------------------------
echo
fail=0
cal() {
    local label="$1" got="$2" want="$3"
    if [[ "$got" == "$want" ]]; then printf '  ✅ %-56s %s\n' "$label" "$got"
    else printf '  ⛔ %-56s got=%s want=%s\n' "$label" "$got" "$want"; fail=1; fi
}
echo "CALIBRATION — hand-measured 2026-07-30; a miss means the instrument, not the repo, moved:"
cal "CAL-1 derived contract count"                "${#CONTRACTS[@]}" "9"
cal "CAL-2 contracts with a scope section"        "$scope_total"     "8"
cal "CAL-3 the one WITHOUT is regex"              "${scope_missing[*]:-<none>}" "REGEX"
cal "CAL-4 contracts with a boundary disclosure"  "$boundary_total"  "1"
cal "CAL-5 systemverilog is that one"             "$(boundary_sections docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md)" "1"
cal "CAL-6 scripts naming a boundary/scope heading" "$boundary_readers" "0"

if [[ "$fail" -ne 0 ]]; then
    echo
    echo "MISCALIBRATED — no census verdict offered. Re-adjudicate the pinned facts before trusting"
    echo "any number above; do NOT simply re-baseline them." >&2
    exit 2
fi

echo
echo "=============================================================================="
echo "VERDICT: 8 of 9 contracts document SCOPE; 1 of 9 documents an ACCEPTANCE BOUNDARY;"
echo "         0 of 9 have that boundary read by any gate. => .5d's gap is real."
echo "=============================================================================="
