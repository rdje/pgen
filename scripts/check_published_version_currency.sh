#!/usr/bin/env bash
# check_published_version_currency.sh — the PUBLISHED-VERSION-CURRENCY doctrine (DONE-BAR.5a).
#
# WHY THIS EXISTS. `Provisional` ships (director 2026-07-29): downstream customers decide from the
# PUBLISHED state, so the published state must be TRUE — and it measurably was not. The user guide's
# regex "Public contract identity" block published parser release `1.1.29` / integration contract
# `1.1.31` while the contract's Contract Identity block declared `1.1.106` / `1.1.109` (~77 releases
# stale), and it published `family status: Done` after the tracker row moved to `In Progress` —
# with NO gate reading either document (measured: DOCTRINE-GAP-OWNERSHIP.4, DONE-BAR.1/.5).
# *A disclosure nobody checks is a claim, not a disclosure.*
#
# WHAT IT PROVES (structural, cheap — no cargo, no make, no network):
#   1. the user guide's regex published version pair equals the integration contract's
#      Contract Identity block (the authoritative declaration — the same source the
#      PGEN-RGX-0091 embedding-constants gate is specified against);
#   2. the user guide's published `family status:` equals the live tracker's `regex` row.
#
# REFUSAL POLARITY: an extraction that comes back EMPTY fails loudly rather than comparing empty
# strings — two empty strings are never evidence of agreement (the demotion probe's own measured
# first-cut defect, and CI-PARITY-GATE-ROT.3's vacuous-green class).
#
# TESTABILITY SEAMS (probe driver: docs/tasks/artifacts/done_bar/run_published_version_currency_probes.sh):
#   PGEN_PVC_GUIDE / PGEN_PVC_CONTRACT / PGEN_PVC_TRACKER — override the three inputs one at a time.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

GUIDE="${PGEN_PVC_GUIDE:-$ROOT/PGEN_USER_GUIDE.md}"
CONTRACT="${PGEN_PVC_CONTRACT:-$ROOT/docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md}"
TRACKER="${PGEN_PVC_TRACKER:-$ROOT/LIVE_ACHIEVEMENT_STATUS.md}"

fail=0
note() { printf 'published-version-currency: %s\n' "$1" >&2; fail=1; }

for f in "$GUIDE" "$CONTRACT" "$TRACKER"; do
    if [[ ! -s "$f" ]]; then
        note "required input '$f' is missing or empty"
    fi
done
[[ "$fail" -eq 0 ]] || exit 1

# The single home of the tracker-row reader (DONE-BAR.2a).
# shellcheck source=../rust/scripts/lib/parser_family_status_bar.sh
source "$ROOT/rust/scripts/lib/parser_family_status_bar.sh"

backticked_value_after_key() {
    # first "  - `value`" line following KEY, scanning from SECTION; empty when absent
    local file="$1" section="$2" key="$3"
    awk -v section="$section" -v key="$key" '
        index($0, section) == 1 { s = 1 }
        s && index($0, key) == 1 { getline; if (match($0, /`[^`]*`/)) { print substr($0, RSTART + 1, RLENGTH - 2) }; exit }
    ' "$file"
}

guide_release="$(backticked_value_after_key "$GUIDE" "### Regex Parser Flavor" "- parser release version:")"
guide_contract="$(backticked_value_after_key "$GUIDE" "### Regex Parser Flavor" "- integration contract version:")"
guide_status="$(backticked_value_after_key "$GUIDE" "### Regex Parser Flavor" "- family status:")"
contract_release="$(backticked_value_after_key "$CONTRACT" "## Contract Identity" "- Parser release version:")"
contract_version="$(backticked_value_after_key "$CONTRACT" "## Contract Identity" "- Contract version:")"
tracker_status="$(markdown_table_status_for_row '| `regex` parser family |' "$TRACKER")"

[[ -n "$guide_release" ]]    || note "could not extract the guide's published parser release version (### Regex Parser Flavor block)"
[[ -n "$guide_contract" ]]   || note "could not extract the guide's published integration contract version"
[[ -n "$guide_status" ]]     || note "could not extract the guide's published family status"
[[ -n "$contract_release" ]] || note "could not extract the contract's Parser release version (## Contract Identity block)"
[[ -n "$contract_version" ]] || note "could not extract the contract's Contract version"
[[ -n "$tracker_status" ]]   || note "could not extract the tracker's regex row status"
[[ "$fail" -eq 0 ]] || exit 1

if [[ "$guide_release" != "$contract_release" ]]; then
    note "PGEN_USER_GUIDE.md publishes parser release '$guide_release' but the contract's Contract Identity declares '$contract_release' — update the guide's Regex Parser Flavor block (a customer decides from the published state, so it must be true)"
fi
if [[ "$guide_contract" != "$contract_version" ]]; then
    note "PGEN_USER_GUIDE.md publishes integration contract '$guide_contract' but the contract's Contract Identity declares '$contract_version' — update the guide's Regex Parser Flavor block"
fi
if [[ "$guide_status" != "$tracker_status" ]]; then
    note "PGEN_USER_GUIDE.md publishes regex family status '$guide_status' but LIVE_ACHIEVEMENT_STATUS.md says '$tracker_status' — update the guide's published status"
fi

if [[ "$fail" -eq 0 ]]; then
    echo "published-version-currency: OK (guide ${guide_release}/${guide_contract} == contract identity; published status '${guide_status}' == tracker)"
fi
exit "$fail"
