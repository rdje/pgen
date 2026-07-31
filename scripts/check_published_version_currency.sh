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
#   2. the user guide's published `family status:` equals the register's `regex` claim;
#   3. LIVE-MEANS-LIVE.1c1 — the BOOK's published per-family snapshot table equals the register's
#      `claimed_status` for EVERY family, in both directions.
#
# WHY THE BOOK IS HELD TO THE SAME BAR (LIVE-MEANS-LIVE.1c1). The director reviews the book, not
# the code — so the book IS the published state for the reader who matters most, and this doctrine
# already exists to hold a published state true. When the status claim moved out of
# LIVE_ACHIEVEMENT_STATUS.md into the register (.1a), the human at-a-glance view moved into
# docs/book/src/roadmap-and-live-status.md; an unheld copy of a status table is exactly the drift
# that produced the ~77-release-stale guide block above.
#
# ⛔ THIS IS NOT THE TWO-ARM CHECK AND MUST NOT BE MISTAKEN FOR IT. It compares a PRESENTATION
# against the claim it presents. The independent arms are elsewhere and unchanged: a human authors
# `claimed_status`, the three *_parser_family_status_gate.sh gates COMPUTE the status from proof
# surfaces, and they fail on disagreement. Deriving the book table from a GATE would collapse that
# check; deriving it from the CLAIM — which is what this does — cannot, because the claim is itself
# hand-authored.
#
# REFUSAL POLARITY: an extraction that comes back EMPTY fails loudly rather than comparing empty
# strings — two empty strings are never evidence of agreement (the demotion probe's own measured
# first-cut defect, and CI-PARITY-GATE-ROT.3's vacuous-green class). For the book table this is
# load-bearing twice over: a MISSING marker pair, a table of ZERO rows, and a family present in the
# register but ABSENT from the table each FAIL. A snapshot that silently stops listing a family
# would otherwise publish "no such family" as agreement.
#
# TESTABILITY SEAMS (probe driver: docs/tasks/artifacts/done_bar/run_published_version_currency_probes.sh):
#   PGEN_PVC_GUIDE / PGEN_PVC_CONTRACT / PGEN_PVC_TRACKER — override the three inputs one at a time;
#   PGEN_PVC_BOOK_PAGE — override the book snapshot page.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

GUIDE="${PGEN_PVC_GUIDE:-$ROOT/PGEN_USER_GUIDE.md}"
CONTRACT="${PGEN_PVC_CONTRACT:-$ROOT/docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md}"
# LIVE-MEANS-LIVE.1a — the family-status CLAIM moved out of LIVE_ACHIEVEMENT_STATUS.md and into
# the DONE-BAR register. The seam name is kept so the probe driver keeps working.
TRACKER="${PGEN_PVC_TRACKER:-$ROOT/rust/test_data/grammar_quality/done_bar_family_register_v0.json}"
BOOK_PAGE="${PGEN_PVC_BOOK_PAGE:-$ROOT/docs/book/src/roadmap-and-live-status.md}"

fail=0
note() { printf 'published-version-currency: %s\n' "$1" >&2; fail=1; }

for f in "$GUIDE" "$CONTRACT" "$TRACKER" "$BOOK_PAGE"; do
    if [[ ! -s "$f" ]]; then
        note "required input '$f' is missing or empty"
    fi
done
[[ "$fail" -eq 0 ]] || exit 1

# The single home of the family-status claim reader (DONE-BAR.2a, LIVE-MEANS-LIVE.1a).
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
tracker_status="$(PGEN_FAMILY_STATUS_DONE_BAR_REGISTER="$TRACKER" claimed_status_for_family "regex")"

[[ -n "$guide_release" ]]    || note "could not extract the guide's published parser release version (### Regex Parser Flavor block)"
[[ -n "$guide_contract" ]]   || note "could not extract the guide's published integration contract version"
[[ -n "$guide_status" ]]     || note "could not extract the guide's published family status"
[[ -n "$contract_release" ]] || note "could not extract the contract's Parser release version (## Contract Identity block)"
[[ -n "$contract_version" ]] || note "could not extract the contract's Contract version"
[[ -n "$tracker_status" ]]   || note "could not read the regex family's claimed_status from the DONE-BAR register"
[[ "$fail" -eq 0 ]] || exit 1

if [[ "$guide_release" != "$contract_release" ]]; then
    note "PGEN_USER_GUIDE.md publishes parser release '$guide_release' but the contract's Contract Identity declares '$contract_release' — update the guide's Regex Parser Flavor block (a customer decides from the published state, so it must be true)"
fi
if [[ "$guide_contract" != "$contract_version" ]]; then
    note "PGEN_USER_GUIDE.md publishes integration contract '$guide_contract' but the contract's Contract Identity declares '$contract_version' — update the guide's Regex Parser Flavor block"
fi
if [[ "$guide_status" != "$tracker_status" ]]; then
    note "PGEN_USER_GUIDE.md publishes regex family status '$guide_status' but the DONE-BAR register's claimed_status for regex is '$tracker_status' — update the guide's published status"
fi

# ---------------------------------------------------------------------------------------------
# (3) The BOOK's per-family snapshot table == the register's claimed_status, BOTH directions.
#
# Both directions matter and they catch different defects: register→table catches a family that
# quietly stopped being published; table→register catches a published row for a family the register
# no longer carries. A one-sided check here would be the exact shape .1b had to repair in
# audit_done_bar.sh, where every guard sat on the arm that could not fail.
book_report="$(
    python3 - "$BOOK_PAGE" "$TRACKER" <<'PY'
import json, re, sys

page_path, register_path = sys.argv[1], sys.argv[2]
page = open(page_path, encoding="utf-8").read()

BEGIN, END = "LIVE-STATUS-SNAPSHOT:BEGIN", "LIVE-STATUS-SNAPSHOT:END"
if BEGIN not in page or END not in page:
    print("REFUSE|%s carries no %s/%s marker pair, so the published snapshot cannot be located. "
          "An unlocatable table must fail, never pass by absence." % (page_path, BEGIN, END))
    raise SystemExit(0)

block = page.split(BEGIN, 1)[1].split(END, 1)[0]

# Rows look like:  | `family` | Status | owner | leg-3 |
published = {}
for line in block.splitlines():
    cells = [c.strip() for c in line.split("|")]
    if len(cells) < 6 or not cells[1].startswith("`"):
        continue
    published[cells[1].strip("`")] = cells[2]

if not published:
    print("REFUSE|the snapshot block in %s yielded ZERO family rows — an empty table agrees with "
          "everything and proves nothing" % page_path)
    raise SystemExit(0)

try:
    families = json.load(open(register_path, encoding="utf-8"))["families"]
except Exception as exc:
    print("REFUSE|the done-bar register '%s' is unreadable: %s" % (register_path, exc))
    raise SystemExit(0)

claims = {name: entry.get("claimed_status") for name, entry in families.items()}

problems = []
for name in sorted(claims):
    if name not in published:
        problems.append("family '%s' is in the register but ABSENT from the book snapshot in %s "
                        "(claimed_status '%s') — a family that stops being published is a silent "
                        "disclosure loss" % (name, page_path, claims[name]))
for name in sorted(published):
    if name not in claims:
        problems.append("the book snapshot in %s publishes a row for '%s', which the done-bar "
                        "register does not carry as a family" % (page_path, name))
for name in sorted(set(claims) & set(published)):
    if published[name] != claims[name]:
        problems.append("the book snapshot publishes '%s' for family '%s' but the register's "
                        "claimed_status is '%s' — update the register first, then this table"
                        % (published[name], name, claims[name]))

print("OK|%d" % len(published) if not problems else "FAIL|" + "\n".join(problems))
PY
)" || note "the book snapshot comparison failed to run"

case "$book_report" in
    REFUSE\|*) note "${book_report#REFUSE|}" ;;
    FAIL\|*)   while IFS= read -r line; do note "$line"; done <<<"${book_report#FAIL|}" ;;
    OK\|*)     book_rows="${book_report#OK|}" ;;
    *)         note "the book snapshot comparison produced no verdict — a check that cannot see must refuse, not pass" ;;
esac

if [[ "$fail" -eq 0 ]]; then
    echo "published-version-currency: OK (guide ${guide_release}/${guide_contract} == contract identity; published status '${guide_status}' == tracker; book snapshot ${book_rows}/${book_rows} families == register)"
fi
exit "$fail"
