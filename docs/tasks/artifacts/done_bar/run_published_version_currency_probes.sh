#!/usr/bin/env bash
# run_published_version_currency_probes.sh — prove the PUBLISHED-VERSION-CURRENCY doctrine check
# (task-tree leaf `DONE-BAR.5a`; scripts/check_published_version_currency.sh) actually bites.
#
# Every arm asserts BOTH the exit code AND a substring of the message (CI-PARITY-GATE-ROT.4's rule:
# pass/fail alone hides arms that reach the right verdict for the wrong reason). The RED-1 arm is
# the HISTORICAL defect replayed verbatim: a guide publishing `1.1.29`/`1.1.31` — the exact stale
# pair this doctrine was created to make impossible to hold silently.
#
#   bash docs/tasks/artifacts/done_bar/run_published_version_currency_probes.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

CHECK="$ROOT/scripts/check_published_version_currency.sh"
WORK="$ROOT/rust/target/done_bar_audit/pvc_probe"
rm -rf "$WORK"; mkdir -p "$WORK"

pass=0
fail=0
arm() {
    local label="$1" want_rc="$2" want_substr="$3"
    shift 3
    local out rc
    out="$(env "$@" bash "$CHECK" 2>&1)"; rc=$?
    if [[ "$rc" == "$want_rc" && "$out" == *"$want_substr"* ]]; then
        printf '  ✅ %-56s exit=%s, message matched\n' "$label" "$rc"
        pass=$((pass + 1))
    else
        printf '  ⛔ %-56s exit=%s (want %s)\n' "$label" "$rc" "$want_rc"
        printf '     wanted substring: %s\n' "$want_substr"
        printf '%s\n' "$out" | tail -n 6 | sed 's/^/     | /'
        fail=$((fail + 1))
    fi
}

echo "=============================================================================="
echo "PUBLISHED-VERSION-CURRENCY probe arms — scripts/check_published_version_currency.sh"
echo "=============================================================================="

# CTRL-1 — the untouched tree passes (the .5a fix landed: guide == contract identity == tracker).
arm "CTRL-1 untouched tree passes" 0 "published-version-currency: OK" --

# RED-1 — THE HISTORICAL DEFECT, verbatim: a guide still publishing the stale 1.1.29/1.1.31 pair.
sed -e 's/`1\.1\.106`/`1.1.29`/' -e 's/`1\.1\.109`/`1.1.31`/' PGEN_USER_GUIDE.md >"$WORK/guide_stale_pair.md"
arm "RED-1 stale published pair fails, named" 1 "publishes parser release '1.1.29'" \
    -- "PGEN_PVC_GUIDE=$WORK/guide_stale_pair.md"

# RED-2 — a stale published STATUS fails: the guide claims `Done` against an `In Progress` row.
sed -e '/^### Regex Parser Flavor/,/^### / s/^  - `In Progress`$/  - `Done`/' PGEN_USER_GUIDE.md >"$WORK/guide_stale_status.md"
arm "RED-2 stale published status fails, named" 1 "publishes regex family status 'Done'" \
    -- "PGEN_PVC_GUIDE=$WORK/guide_stale_status.md"

# RED-3 — an EMPTY extraction fails loudly, never comparing empty strings (the vacuous-green class).
printf '# a guide with no Regex Parser Flavor section\n' >"$WORK/guide_empty.md"
arm "RED-3 empty extraction refuses, never vacuous" 1 "could not extract the guide's published parser release version" \
    -- "PGEN_PVC_GUIDE=$WORK/guide_empty.md"

# RED-4 — the CONTRACT side moving without the guide fails too (the drift direction that actually
# happened: releases advanced ~77 times while the guide stood still).
sed -e '/^## Contract Identity/,/^## / s/`1\.1\.106`/`1.1.107`/' \
    docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md >"$WORK/contract_advanced.md"
arm "RED-4 contract advance without guide fails" 1 "declares '1.1.107'" \
    -- "PGEN_PVC_CONTRACT=$WORK/contract_advanced.md"

# ------------------------------------------------------------------------------------------------
# LIVE-MEANS-LIVE.1c1 — the BOOK snapshot arm. The at-a-glance family-status view moved into
# docs/book/src/roadmap-and-live-status.md when LIVE_ACHIEVEMENT_STATUS.md was retired, and an
# unheld copy of a status table is precisely the drift RED-1/RED-2 above exist to make impossible.
# Every arm below mutates a COPY via the PGEN_PVC_BOOK_PAGE seam — the tracked page is never touched.
BOOK="docs/book/src/roadmap-and-live-status.md"

mutate_book() {
    # mutate_book OUT_NAME MODE [ARG] — build a mutant of the book page under $WORK.
    python3 - "$ROOT/$BOOK" "$WORK/$1" "$2" "${3:-}" <<'PY'
import sys
src, dst, mode, arg = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
text = open(src, encoding="utf-8").read()
BEGIN, END = "LIVE-STATUS-SNAPSHOT:BEGIN", "LIVE-STATUS-SNAPSHOT:END"

if mode == "drop-markers":
    text = text.replace(BEGIN, "snapshot-marker-removed").replace(END, "snapshot-marker-removed")
else:
    head, rest = text.split(BEGIN, 1)
    block, tail = rest.split(END, 1)
    lines = block.splitlines(keepends=True)

    def is_family_row(line, name=None):
        cells = [c.strip() for c in line.split("|")]
        if len(cells) < 6 or not cells[1].startswith("`"):
            return False
        return name is None or cells[1].strip("`") == name

    if mode == "restate":                        # publish a status the register does not claim
        out = []
        for line in lines:
            if is_family_row(line, arg):
                cells = line.split("|")
                cells[2] = " Done "
                line = "|".join(cells)
            out.append(line)
        lines = out
    elif mode == "drop-row":                     # a family silently stops being published
        lines = [l for l in lines if not is_family_row(l, arg)]
    elif mode == "add-row":                      # publish a family the register does not carry
        lines.append("| `%s` | Done | pgen | *(none declared)* |\n" % arg)
    elif mode == "empty-table":                  # markers present, zero rows
        lines = [l for l in lines if not is_family_row(l)]
    else:
        raise SystemExit("unknown mutate mode %r" % mode)

    text = head + BEGIN + "".join(lines) + END + tail

open(dst, "w", encoding="utf-8").write(text)
PY
}

# RED-5 — the published table claims a status the register does not. This is the whole point of the
# arm: the book is the director's review surface, so a false row here is a false published claim.
mutate_book book_restated.md restate vhdl
arm "RED-5 book restates a status, named both ways" 1 "publishes 'Done' for family 'vhdl'" \
    -- "PGEN_PVC_BOOK_PAGE=$WORK/book_restated.md"

# RED-6 — register→table direction: a family that quietly stops being published. Without this arm a
# snapshot could shrink to one row and still read green (the .1b failure shape: the only guard
# sitting on the side that cannot fail).
mutate_book book_missing_row.md drop-row json
arm "RED-6 family dropped from the table fails" 1 "'json' is in the register but ABSENT" \
    -- "PGEN_PVC_BOOK_PAGE=$WORK/book_missing_row.md"

# RED-7 — table→register direction: a published row for a family the register does not carry.
mutate_book book_extra_row.md add-row phantom_family
arm "RED-7 phantom published family fails" 1 "publishes a row for 'phantom_family'" \
    -- "PGEN_PVC_BOOK_PAGE=$WORK/book_extra_row.md"

# RED-8 — the snapshot cannot be LOCATED. A missing marker pair must refuse; silently finding no
# table and passing is the vacuous-green class this whole doctrine was written against.
mutate_book book_no_markers.md drop-markers
arm "RED-8 missing marker pair refuses" 1 "carries no LIVE-STATUS-SNAPSHOT:BEGIN" \
    -- "PGEN_PVC_BOOK_PAGE=$WORK/book_no_markers.md"

# RED-9 — markers present, ZERO rows. An empty table agrees with every register and proves nothing.
mutate_book book_empty_table.md empty-table
arm "RED-9 empty snapshot table refuses" 1 "yielded ZERO family rows" \
    -- "PGEN_PVC_BOOK_PAGE=$WORK/book_empty_table.md"

# RED-10 — an unreadable register refuses rather than treating "no claims" as agreement.
printf 'not json at all\n' >"$WORK/register_broken.json"
arm "RED-10 unreadable register refuses" 1 "is unreadable" \
    -- "PGEN_PVC_TRACKER=$WORK/register_broken.json"


# ------------------------------------------------------------------------------------------------
# SV-CORPUS-GRAD.13c.2x.6 — the SV recognized cert-coverage UNION tuple arms (tier 4).
#
# THE FOUNDING DEFECT, RESTATED SO THE ARMS ARE READ AGAINST IT: this book section published the
# tuple as of 2026-07-22, the contract was re-baselined ELEVEN times underneath it, and the section
# still asserted in bold that SystemVerilog is recognized `fully_certified` while the measured union
# UNKNOWN was 53. A stale NUMBER misinforms; a stale VERDICT misdirects. RED-11 is that defect
# replayed verbatim, and it is deliberately the FIRST arm.
SVPAGE="docs/book/src/grammar-wellformedness.md"
SVCONTRACT="rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json"

mutate_sv_page() {
    # mutate_sv_page OUT_NAME MODE [ARG] — build a mutant of the book page under $WORK.
    python3 - "$ROOT/$SVPAGE" "$WORK/$1" "$2" "${3:-}" <<'PY'
import re, sys
src, dst, mode, arg = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
text = open(src, encoding="utf-8").read()
BEGIN, END = "SV-CERT-UNION-TUPLE:BEGIN", "SV-CERT-UNION-TUPLE:END"

if mode == "drop-markers":
    text = text.replace(BEGIN, "tuple-marker-removed").replace(END, "tuple-marker-removed")
else:
    head, rest = text.split(BEGIN, 1)
    block, tail = rest.split(END, 1)
    lines = block.splitlines(keepends=True)

    def key_of(line):
        cells = [c.strip() for c in line.split("|")]
        if len(cells) < 5:
            return None
        m = re.search(r"`([a-z_]+)`", cells[2])
        return m.group(1) if m else None

    def set_value(line, value):
        cells = line.split("|")
        cells[3] = " %s " % value
        return "|".join(cells)

    if mode == "set":                            # publish a value the contract does not declare
        key, value = arg.split("=", 1)
        lines = [set_value(l, value) if key_of(l) == key else l for l in lines]
    elif mode == "drop-row":                     # a contract key stops being published
        lines = [l for l in lines if key_of(l) != arg]
    elif mode == "add-row":                      # publish a key the contract does not hold
        lines.append("| phantom | `%s` | 1 |\n" % arg)
    elif mode == "empty-table":                  # markers present, zero rows
        lines = [l for l in lines if key_of(l) is None]
    else:
        raise SystemExit("unknown mutate mode %r" % mode)

    text = head + BEGIN + "".join(lines) + END + tail

open(dst, "w", encoding="utf-8").write(text)
PY
}

# RED-11 — ⛔ THE HISTORICAL DEFECT, VERBATIM: the published verdict says SV IS recognized
# `fully_certified` while the contract's union UNKNOWN is 1. Every NUMBER in the block is still
# correct in this mutant — which is the point. A checker that held only the six numbers would pass
# this, and the one cell that actually misdirects a reader would stay unwatched.
mutate_sv_page sv_false_verdict.md set "fully_certified_via_union=true"
arm "RED-11 false fully_certified verdict fails" 1 "publishes fully_certified_via_union = 'true' but the contract declares False" \
    -- "PGEN_PVC_SV_CERT_UNION_PAGE=$WORK/sv_false_verdict.md"

# RED-12 — a published NUMBER drifts from the contract. `1362` is not an arbitrary mutant: it is the
# exact stale `expected_total` this section carried through eleven rebaselines.
mutate_sv_page sv_stale_total.md set "expected_total=1362"
arm "RED-12 stale published total fails, named" 1 "publishes expected_total = '1362'" \
    -- "PGEN_PVC_SV_CERT_UNION_PAGE=$WORK/sv_stale_total.md"

# RED-13 — the PRODUCER side moving without the book. This is the drift direction that actually
# happened eleven times, and it is the one a book-only check would never see.
python3 -c "
import json,sys
d=json.load(open('$ROOT/$SVCONTRACT'))
d['expected_union_unknown']=0
json.dump(d,open('$WORK/sv_contract_advanced.json','w'),indent=2)
"
arm "RED-13 contract advance without the book fails" 1 "publishes expected_union_unknown = '1' but the contract declares 0" \
    -- "PGEN_PVC_SV_CERT_UNION_CONTRACT=$WORK/sv_contract_advanced.json"

# RED-14 — contract→book direction: a number that quietly stops being published. Without this arm the
# block could shrink to one row and still read green.
mutate_sv_page sv_missing_row.md drop-row expected_union_residual_rules
arm "RED-14 dropped published row fails" 1 "carries NO row for it" \
    -- "PGEN_PVC_SV_CERT_UNION_PAGE=$WORK/sv_missing_row.md"

# RED-15 — book→contract direction: a published row for a key the contract does not hold.
mutate_sv_page sv_extra_row.md add-row expected_phantom_field
arm "RED-15 phantom published key fails" 1 "carries a row for 'expected_phantom_field'" \
    -- "PGEN_PVC_SV_CERT_UNION_PAGE=$WORK/sv_extra_row.md"

# RED-16 — the block cannot be LOCATED. Passing by absence is the vacuous-green class.
mutate_sv_page sv_no_markers.md drop-markers
arm "RED-16 missing marker pair refuses" 1 "carries no SV-CERT-UNION-TUPLE:BEGIN" \
    -- "PGEN_PVC_SV_CERT_UNION_PAGE=$WORK/sv_no_markers.md"

# RED-17 — markers present, ZERO rows. An empty table agrees with every contract.
mutate_sv_page sv_empty_table.md empty-table
arm "RED-17 empty tuple table refuses" 1 "yielded ZERO published rows" \
    -- "PGEN_PVC_SV_CERT_UNION_PAGE=$WORK/sv_empty_table.md"

# RED-18 — the residual SET is compared as a set, not as a string: a DIFFERENT single name must fail
# even though the row still parses as a one-element array of the right shape.
mutate_sv_page sv_wrong_residual.md set 'expected_union_residual_rules=`["select_expression_lr_suffix"]`'
arm "RED-18 wrong residual rule name fails" 1 "publishes expected_union_residual_rules" \
    -- "PGEN_PVC_SV_CERT_UNION_PAGE=$WORK/sv_wrong_residual.md"

# RED-19 — a published cell that is not the type the contract holds must be named, not coerced.
mutate_sv_page sv_bad_type.md set "expected_union_unknown=none"
arm "RED-19 non-integer published cell fails" 1 "must publish an integer" \
    -- "PGEN_PVC_SV_CERT_UNION_PAGE=$WORK/sv_bad_type.md"

# RED-20 — an unreadable contract refuses rather than treating "no expectations" as agreement.
printf 'not json at all\n' >"$WORK/sv_contract_broken.json"
arm "RED-20 unreadable contract refuses" 1 "is unreadable" \
    -- "PGEN_PVC_SV_CERT_UNION_CONTRACT=$WORK/sv_contract_broken.json"

echo "------------------------------------------------------------------------------"
printf 'published-version-currency probes: %d/%d passed\n' "$pass" "$((pass + fail))"
[[ "$fail" -eq 0 ]] || exit 1
