#!/usr/bin/env bash
# run_family_status_probes.sh — prove `scripts/report_family_status.sh --check` actually BITES
# (task-tree leaf `DONE-BAR-VISIBILITY.1`).
#
# ⛔⛔ WHY THIS DRIVER EXISTS AT ALL. The sibling producer `scripts/report_grammar_certification.sh`
# shipped a `--check` flag whose implementation had been deleted by a rewrite: the flag went on
# being ACCEPTED and the script went on exiting 0 for a whole commit — on the real page, on a
# corrupted page, on a page with no block, and on a path that does not exist, all four mutually
# indistinguishable (GRAMMAR-CERT-STATUS.1b). A check that cannot go RED is not a check, and the
# only way to know it can is to drive it red on purpose. Every arm asserts BOTH the exit code AND a
# substring of the message, because pass/fail alone hides an arm reaching the right verdict for the
# wrong reason.
#
#   bash docs/tasks/artifacts/done_bar_visibility/run_family_status_probes.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

PRODUCER="$ROOT/scripts/report_family_status.sh"
PAGE="docs/systemverilog_parser_book/src/status-and-distance.md"
WORK="$ROOT/rust/target/family_status_probe"
rm -rf "$WORK"; mkdir -p "$WORK"

pass=0
fail=0
arm() {  # arm <label> <want-rc> <want-substring> -- <args...>
    local label="$1" want_rc="$2" want_substr="$3"; shift 4
    local out rc
    out="$(bash "$PRODUCER" "$@" 2>&1)"; rc=$?
    if [[ "$rc" == "$want_rc" && "$out" == *"$want_substr"* ]]; then
        printf '  ✅ %-58s exit=%s, message matched\n' "$label" "$rc"
        pass=$((pass + 1))
    else
        printf '  ⛔ %-58s exit=%s (want %s)\n' "$label" "$rc" "$want_rc"
        printf '     wanted substring: %s\n' "$want_substr"
        printf '%s\n' "$out" | tail -n 6 | sed 's/^/     | /'
        fail=$((fail + 1))
    fi
}

echo "=============================================================================="
echo "FAMILY-STATUS producer probe arms — scripts/report_family_status.sh"
echo "=============================================================================="

# --- the GREEN control, and it must be green for the RED arms to mean anything ------------------
arm "CTRL-1 the published page equals a fresh derivation" 0 \
    "report_family_status: OK (published block equals a fresh derivation" -- \
    systemverilog --check "$PAGE"

# --- RED-1..4: the four arms that were mutually indistinguishable in the sibling's defect --------
# RED-1 — a MUTATED number inside the published block. This is the rot the doctrine exists for: the
# book says one thing and the tree says another, and today the reader cannot tell which is lying.
sed 's/| \*\*Passing\*\* — the parser.s verdict matches the answer key | 5 845 |/| **Passing** — the parser'"'"'s verdict matches the answer key | 9 999 |/' \
    "$PAGE" > "$WORK/mutated.md"
if ! grep -q '9 999' "$WORK/mutated.md"; then
    echo "  ⛔ SETUP FAILED: the RED-1 mutation did not apply — the arm would pass vacuously" >&2
    fail=$((fail + 1))
fi
arm "RED-1 a mutated pass count in the published block is caught" 1 \
    "the published status block is OUT OF SYNC" -- systemverilog --check "$WORK/mutated.md"

# RED-2 — a page whose DERIVED markers were stripped. The check must SAY it cannot inspect its
# subject, never return green (docs/decisions/feedback_a_check_that_cannot_run_must_say_so.md).
grep -v 'DERIVED' "$PAGE" > "$WORK/noblock.md"
arm "RED-2 a page carrying no DERIVED block is REFUSED" 2 \
    "carries no <!-- BEGIN DERIVED" -- systemverilog --check "$WORK/noblock.md"

# RED-3 — a path that does not exist.
arm "RED-3 a nonexistent page is REFUSED" 2 \
    "no published page at" -- systemverilog --check "$WORK/does_not_exist.md"

# RED-4 — ANOTHER family's block published on this page. The block names its own family in the
# BEGIN marker, so a page that swapped families must fail rather than compare loosely.
bash "$PRODUCER" vhdl --markdown > "$WORK/vhdl_block.txt"
python3 - "$PAGE" "$WORK/vhdl_block.txt" "$WORK/wrong_family.md" <<'PY'
import pathlib, re, sys
page, block, out = (pathlib.Path(p) for p in sys.argv[1:4])
text = re.sub(r"<!-- BEGIN DERIVED.*?<!-- END DERIVED -->", block.read_text().strip(),
              page.read_text(), flags=re.S)
out.write_text(text)
PY
arm "RED-4 another family's block on this page is caught" 1 \
    "the published status block is OUT OF SYNC" -- systemverilog --check "$WORK/wrong_family.md"

# --- RED-5..6: the producer's own refusals -------------------------------------------------------
# RED-5 — a family that does not exist. The roster is DERIVED from the register, so the refusal
# prints the real roster rather than a guess.
arm "RED-5 an unknown family is REFUSED with the derived roster" 2 \
    "is not a parser family" -- nosuchfamily

# RED-6 — no family named at all.
arm "RED-6 naming no family is REFUSED" 2 "name the family" --

# RED-7 — THE SCHEMA IS TWO-SIDED. A field declared in the schema but not rendered is as much a
# defect as a field rendered with no declared producer, and both must refuse. Driven here by adding
# a declared-but-unrendered field to a COPY of the schema, through the producer's own seam.
python3 - "$WORK/extra_field_schema.json" <<'PYX'
import json, pathlib, sys
d = json.loads(pathlib.Path("rust/test_data/grammar_quality/family_status_block_v0.json").read_text())
d["fields"]["a_field_nothing_renders"] = {"question": "?", "producer": "none",
                                          "state": "derived", "surfaces": ["chat", "book"]}
pathlib.Path(sys.argv[1]).write_text(json.dumps(d, indent=2))
PYX
out="$(PGEN_FAMILY_STATUS_SCHEMA="$WORK/extra_field_schema.json" bash "$PRODUCER" systemverilog 2>&1)"; rc=$?
if [[ "$rc" == 2 && "$out" == *"only-declared=['a_field_nothing_renders']"* ]]; then
    printf '  ✅ %-58s exit=%s, message matched\n' "RED-7 a declared-but-unrendered field is REFUSED" "$rc"
    pass=$((pass + 1))
else
    printf '  ⛔ %-58s exit=%s (want 2)\n' "RED-7 a declared-but-unrendered field is REFUSED" "$rc"
    printf '%s\n' "$out" | tail -n 4 | sed 's/^/     | /'
    fail=$((fail + 1))
fi

# RED-8/RED-9 — THE TREE-ATTRIBUTION POPULATION IS CLOSED IN BOTH DIRECTIONS. A remaining-work list
# is only finite if the roster it is drawn from cannot grow silently: a tree row nobody adjudicated
# would simply be absent from every family's list, which is the failure this list exists to end.
python3 - "$WORK/drop_tree_schema.json" "$WORK/ghost_tree_schema.json" <<'PYX'
import json, pathlib, sys
d = json.loads(pathlib.Path("rust/test_data/grammar_quality/family_status_block_v0.json").read_text())
drop = json.loads(json.dumps(d)); drop["tree_attribution"]["trees"].pop("SV-EXH-PROOF")
pathlib.Path(sys.argv[1]).write_text(json.dumps(drop, indent=2))
ghost = json.loads(json.dumps(d)); ghost["tree_attribution"]["trees"]["A-TREE-THAT-DOES-NOT-EXIST"] = "systemverilog"
pathlib.Path(sys.argv[2]).write_text(json.dumps(ghost, indent=2))
PYX
for spec in "RED-8 a tree row with no attribution is REFUSED|$WORK/drop_tree_schema.json|carry no attribution ['SV-EXH-PROOF']" \
            "RED-9 an attribution naming no tree row is REFUSED|$WORK/ghost_tree_schema.json|name no row ['A-TREE-THAT-DOES-NOT-EXIST']"; do
    IFS='|' read -r label sch want <<<"$spec"
    out="$(PGEN_FAMILY_STATUS_SCHEMA="$sch" bash "$PRODUCER" systemverilog 2>&1)"; rc=$?
    if [[ "$rc" == 2 && "$out" == *"$want"* ]]; then
        printf '  ✅ %-58s exit=%s, message matched\n' "$label" "$rc"; pass=$((pass + 1))
    else
        printf '  ⛔ %-58s exit=%s (want 2)\n' "$label" "$rc"
        printf '     wanted substring: %s\n' "$want"
        printf '%s\n' "$out" | tail -n 4 | sed 's/^/     | /'; fail=$((fail + 1))
    fi
done

# RED-10/RED-11 — WHY A FAMILY HAS NO CORPUS COUNT IS DERIVED FROM THE REGISTER AND ONLY CHECKED
# against the declaration. A declared-only class silently becomes a lie the moment a corpus is
# wired, which is the same rot as a hand-maintained status: the fix is that the two arms must agree.
python3 - "$WORK/drifted_state_schema.json" "$WORK/no_state_schema.json" <<'PYX'
import json, pathlib, sys
d = json.loads(pathlib.Path("rust/test_data/grammar_quality/family_status_block_v0.json").read_text())
drift = json.loads(json.dumps(d))
drift["corpus_state"]["families"]["vhdl"]["class"] = "no_third_party_corpus"
pathlib.Path(sys.argv[1]).write_text(json.dumps(drift, indent=2))
gone = json.loads(json.dumps(d)); gone["corpus_state"]["families"].pop("vhdl")
pathlib.Path(sys.argv[2]).write_text(json.dumps(gone, indent=2))
PYX
for spec in "RED-10 a declared corpus state that drifts from the register|$WORK/drifted_state_schema.json|but the done-bar register derives" \
            "RED-11 a family with no declared corpus state is REFUSED|$WORK/no_state_schema.json|is declared for 'vhdl' in"; do
    IFS='|' read -r label sch want <<<"$spec"
    out="$(PGEN_FAMILY_STATUS_SCHEMA="$sch" bash "$PRODUCER" vhdl 2>&1)"; rc=$?
    if [[ "$rc" == 2 && "$out" == *"$want"* ]]; then
        printf '  ✅ %-58s exit=%s, message matched\n' "$label" "$rc"; pass=$((pass + 1))
    else
        printf '  ⛔ %-58s exit=%s (want 2)\n' "$label" "$rc"
        printf '     wanted substring: %s\n' "$want"
        printf '%s\n' "$out" | tail -n 4 | sed 's/^/     | /'; fail=$((fail + 1))
    fi
done

echo "------------------------------------------------------------------------------"
printf 'family-status probes: %d passed, %d failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ] || exit 1
