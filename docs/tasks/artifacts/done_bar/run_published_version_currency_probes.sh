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

echo "------------------------------------------------------------------------------"
printf 'published-version-currency probes: %d/%d passed\n' "$pass" "$((pass + fail))"
[[ "$fail" -eq 0 ]] || exit 1
