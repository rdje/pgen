#!/usr/bin/env bash
# PER-FAMILY STATUS AND DISTANCE TO `Done` — MEASURED, never asserted.
#
# Origin, verbatim, director 2026-08-30 (task-tree leaf `DONE-BAR-VISIBILITY.1`):
#
#   "You just flag parser statuses as Mostly Done, In progress, … I have no idea for each parser
#    the amount of work to reach full completion (Done). I have no idea or clue about (1) the
#    amount of test corpus that are passing (2) a synthesis about what's left to be done to reach
#    the Done status. … there seems to be no end in sight."
#
# A status LABEL without a DISTANCE behind it is not a status. This producer answers three
# questions for one family, every number from a named producer declared in the schema
# `rust/test_data/grammar_quality/family_status_block_v0.json`:
#
#   1. where is it against the three-leg `Done` bar,
#   2. how much of its corpus passes, against a NAMED denominator,
#   3. what remains — and while nothing derives that yet, it says NOT MEASURED and names the leaf
#      that owes it, because a blank reads as "nothing left" and a zero reads as "done".
#
# ⭐ ONE PRODUCER, TWO RENDERINGS (director, same day). The BRIEF form is what the engineer reports
# in chat — the director is NOT the operator of this surface and is never expected to run it; a
# command nobody runs is the GATE-REACHABILITY defect wearing a status report. The THOROUGH form is
# the `<!-- BEGIN DERIVED -->` block published into the family's book. `--check` re-derives and
# DIFFS the published block so the two can never disagree — the `report_grammar_certification.sh`
# shape, including the lesson that cost that script its check: an argument parser that ACCEPTS
# `--check` while nothing implements it is worse than no flag at all (GRAMMAR-CERT-STATUS.1b).
#
# ⛔⛔ WHY THE BOOK HALF IS DERIVED TRACKED-ONLY, AND THE CHAT HALF IS NOT.
# Leg 1 and the green-NOW half of leg 2 rest on gate artifacts under `rust/target/`, which is
# UNTRACKED. The same commit therefore reads "artifact STALE" on the machine that ran the gate and
# "no artifact on this machine" on a fresh clone. Publishing either sentence into a gate-held block
# would make `--check` FAIL for every reader who had not run the gate first — a gate that fails on
# a clean clone of its own repository, which is precisely the class CI-PARITY-GATE-ROT exists for.
# So the BOOK block is derived from a TRACKED-ONLY run of the audit (its own
# `PGEN_DONE_BAR_TARGET_DIR` testability seam, pointed at an empty directory), i.e. what the
# repository proves to any reader who clones it; the CHAT form adds this machine's artifact
# readings on top, clearly labelled. One derivation pass, two renderings, the book half being the
# subset every clone agrees on.
#
# ⛔ NOTHING HERE IS RE-IMPLEMENTED. The leg states come from `scripts/audit_done_bar.sh` (which
# carries 13 ground-truth controls and refuses rather than guessing) and the corpus counts are read
# from the census artifacts that `scripts/check_sv_corpus_denominator.sh` proves FRESH on the same
# run — and the tuple this script derives from those artifacts is CROSS-CHECKED against the tuple
# that gate prints, so an arithmetic drift between the two refuses instead of publishing.
#
# Exit codes follow the repository convention: 2 = COULD NOT EVALUATE, 1 = evaluated and BREACHED.
#
# usage:
#   bash scripts/report_family_status.sh <family>              # brief, for the chat report
#   bash scripts/report_family_status.sh <family> --markdown   # the book's DERIVED block
#   bash scripts/report_family_status.sh <family> --check FILE  # re-derive and diff the published block
#   bash scripts/report_family_status.sh --cross-family        # the whole roster, one screenful
#   bash scripts/report_family_status.sh --cross-family --markdown        # the platform book's block
#   bash scripts/report_family_status.sh --cross-family --check FILE      # …and diff it
#   bash scripts/report_family_status.sh --check-all           # every published block, incl. the cross view
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2

# ⛔ THE CROSS-FAMILY PAGE'S LOCATION IS A SEAM, defaulting to the tracked page so an ordinary run
# reads exactly what ships. It exists so the probe driver can drive this producer's refusals against
# a CONSTRUCTED page instead of mutating a tracked one and hoping the restore runs.
CROSS_PAGE="${PGEN_FAMILY_STATUS_CROSS_PAGE:-docs/book/src/roadmap-and-live-status.md}"

FAMILY=""; MODE=text; CHECK=""; CHECK_ALL=0; CROSS=0
while [ $# -gt 0 ]; do
  case "$1" in
    --markdown) MODE=markdown ;;
    --check) CHECK="${2:-}"; MODE=markdown; shift ;;
    # ⭐ --check-all EXISTS FOR COST, AND THE COST IS WHY THE GATE CAN RUN ON EVERY COMMIT. The two
    # audit runs below are family-INDEPENDENT and are the expensive half (~2.3 s of a 2.4 s run), so
    # ten separate invocations would pay for them ten times. Hoisting them turns 24 s into ~4 s.
    --check-all) CHECK_ALL=1; MODE=markdown ;;
    # ⭐ --cross-family IS THE SAME DERIVATION, ONCE PER FAMILY — never a second reader of the same
    # sources. It answers the question the ten per-family pages cannot: how far is the PROJECT, and
    # how do two families compare without opening two books (DONE-BAR-VISIBILITY.5).
    --cross-family) CROSS=1 ;;
    -*) echo "report_family_status: unknown arg '$1'" >&2; exit 2 ;;
    *) [ -z "$FAMILY" ] || { echo "report_family_status: two families given ('$FAMILY', '$1')" >&2; exit 2; }
       FAMILY="$1" ;;
  esac; shift
done
if [ "$CHECK_ALL" -eq 1 ]; then
  [ -z "$FAMILY" ] || { echo "report_family_status: REFUSED (2) - --check-all takes no family" >&2; exit 2; }
  [ -z "$CHECK" ] || { echo "report_family_status: REFUSED (2) - --check-all and --check are exclusive" >&2; exit 2; }
  [ "$CROSS" -eq 0 ] || { echo "report_family_status: REFUSED (2) - --check-all already covers the cross-family view" >&2; exit 2; }
elif [ "$CROSS" -eq 1 ]; then
  [ -z "$FAMILY" ] || { echo "report_family_status: REFUSED (2) - --cross-family takes no family; it renders the whole register" >&2; exit 2; }
  MODE="cross_${MODE}"
else
  [ -n "$FAMILY" ] || { echo "report_family_status: REFUSED (2) - name the family, e.g. 'systemverilog'" >&2; exit 2; }
fi

# ⛔ VALIDATE `--check`'s SUBJECT BEFORE SPENDING THE DERIVATION — both questions below are
# properties of the PAGE ALONE, so paying for a whole derivation to be told the page is absent or
# carries no block is pure waste. Measured on the sibling producer: driving its no-block refusal as
# a red control cost 128.5 s instead of 64 s for exactly that reason.
if [ -n "$CHECK" ]; then
  [ -f "$CHECK" ] || { echo "report_family_status: REFUSED (2) - no published page at $CHECK" >&2; exit 2; }
  if ! grep -q '<!-- BEGIN DERIVED' "$CHECK" || ! grep -q '<!-- END DERIVED -->' "$CHECK"; then
    echo "report_family_status: REFUSED (2) - $CHECK carries no <!-- BEGIN DERIVED --> ... <!-- END DERIVED -->" >&2
    echo "                      block, so the check cannot inspect its subject. A check that cannot" >&2
    echo "                      run must SAY SO, not return green." >&2
    exit 2
  fi
fi

# Scratch stays on the repository volume, derived from the repo root at runtime (CLAUDE.md §13).
WORK="$ROOT/rust/target/family_status"
rm -rf "$WORK" && mkdir -p "$WORK/empty_target" || {
  echo "report_family_status: REFUSED (2) - cannot create scratch under rust/target/family_status" >&2; exit 2; }

# --- the two audit runs ---------------------------------------------------------------------
# TRACKED-ONLY (book + chat): the audit with its artifact root pointed at an empty directory, so
# every verdict rests on tracked inputs alone and is identical in every clone.
# LOCAL (chat only): the ordinary run, which additionally reads this machine's gate artifacts.
# ⛔ Exit 1 is a VALID audit (it means some `Done` row fails the bar); 2 is a refusal and 3 is
# MISCALIBRATED, and neither may be rendered — an instrument that says it cannot be trusted is not
# a source (docs/decisions/feedback_instrument_needs_ground_truth.md).
run_audit() {  # run_audit <target-dir> <text-out> <json-out>
  PGEN_DONE_BAR_TARGET_DIR="$1" bash "$ROOT/scripts/audit_done_bar.sh" --json "$3" >"$2" 2>"$2.err"
  local rc=$?
  if [ "$rc" -ge 2 ]; then
    echo "report_family_status: REFUSED (2) - scripts/audit_done_bar.sh exited $rc (2=refusal, 3=MISCALIBRATED);" >&2
    echo "                      the bar legs cannot be reported without it. Its own words:" >&2
    sed 's/^/                        /' "$2.err" >&2
    return 2
  fi
  return 0
}
run_audit "$WORK/empty_target" "$WORK/tracked.txt" "$WORK/tracked.json" || exit 2
run_audit "$ROOT/rust/target"  "$WORK/local.txt"   "$WORK/local.json"   || exit 2

# render <family> <mode> — one derivation pass, printed. Every caller below goes through this, so
# the brief form, the book form and the check can never come from different code.
render() {
python3 - "$1" "$2" "$WORK" <<'PY'
import json, os, re, subprocess, sys

FAMILY, MODE, WORK = sys.argv[1], sys.argv[2], sys.argv[3]
# The cross-family renderings pass `*` as the family and loop the register below, calling the SAME
# derive() the single-family renderings call. That is the whole anti-drift argument — see § 5.
CROSS = MODE.startswith("cross")
ROOT = os.getcwd()
# Testability seam, on the `audit_done_bar.sh` pattern: it defaults to the tracked schema, so an
# ordinary run reads exactly what the repository ships, and exists so the probe driver can prove the
# two-sided schema check below actually FIRES. A guard nobody has seen fail is an untested guard.
SCHEMA_PATH = os.environ.get("PGEN_FAMILY_STATUS_SCHEMA") or "rust/test_data/grammar_quality/family_status_block_v0.json"
REGISTER_PATH = "rust/test_data/grammar_quality/done_bar_family_register_v0.json"


def refuse(msg):
    print(f"report_family_status: REFUSED (2) - {msg}", file=sys.stderr)
    sys.exit(2)


def read(p):
    with open(p, encoding="utf-8") as fh:
        return fh.read()


schema = json.loads(read(SCHEMA_PATH))
register = json.loads(read(REGISTER_PATH))

# ---------------------------------------------------------------------------------------------
# 1. The bar legs — read from the audit, whose phrasing is mapped onto a CLOSED vocabulary.
# ---------------------------------------------------------------------------------------------
# ⛔ WHY A CLOSED VOCABULARY AND NOT THE AUDIT'S OWN SENTENCES. The audit is a diagnostic report;
# its lines legitimately carry machine-local paths and mtimes, and its wording is free to improve.
# A published block must not inherit either. So each leg's detail lines are matched against the
# table below and the block publishes THIS script's sentence — and a leg whose lines match NOTHING
# is a REFUSAL, never a leg published with no reason. That polarity is the whole point: it makes a
# change in the audit's wording LOUD here instead of silently emptying the published block.
LEG_REASONS = [
    ("no *_parser_family_status_gate computes",
     "no family-status gate computes this family's closure at all"),
    ("has produced no non-empty summary.txt",
     "rests on a gate artifact, which this repository never tracks — run the family-status gate to establish it"),
    ("green-NOW is UNPROVEN — no artifact",
     "rests on a gate artifact, which this repository never tracks — run the family-status gate to establish it"),
    ("no family-status gate exists",
     "no family-status gate computes this family's closure at all"),
    # DONE-BAR.3c: the conformance gate's OWN assertion, evaluated over the tracked manifests when no
    # artifact is on this machine (the book's reading) or over the artifact (the chat's). Listed BEFORE
    # the triage sentence: for 35 days SystemVerilog's block published "only a TRIAGE gate" while a
    # conformance gate existed, because the triage line was the first ⛔ this table recognised.
    ("UNMET by the surface's own assertion",
     "the external-corpus CONFORMANCE gate's own assertion is false — `full_pass` needs zero unexplained divergences on "
     "both lanes and every row adjudicated under its declared reading (the preprocess→parse pipeline for a row the raw "
     "lane defers to it; the permanently unadjudicable classes named and bounded, never counted as a gap), and rows "
     "still carry no verdict under that reading (`corpus_conformance_compare.py --assert-tracked` prints the count)"),
    ("MET by the surface's own assertion",
     "the external-corpus conformance gate's own assertion holds — zero unexplained divergences on both lanes and every row adjudicated under its declared reading, the permanently unadjudicable classes named and bounded"),
    ("holds over rust/target/",
     "the external-corpus conformance gate's own assertion holds over its artifact"),
    ("a TRIAGE gate is not a conformance gate",
     "the only gate reading the external corpus is a TRIAGE gate, and a triage gate is not a conformance gate"),
    ("not backed by the family's declared external corpus",
     "the corpus gate reads no declared external corpus root"),
    ("invoked by nothing, so its green",
     "the external-corpus gate is invoked by nothing, so its green is never re-established"),
    ("no external corpus EXISTS",
     "no external corpus exists — the language is authored by PGEN itself, so leg 3 is unreachable by construction"),
    ("NO external corpus is wired",
     "no external corpus is wired, though the language is standardized"),
    ("language ownership UNADJUDICATED",
     "no external corpus is wired and the language ownership is UNADJUDICATED (owed by DONE-BAR.3)"),
    ("corpus assets present", "corpus assets are present but NO gate asserts them"),
    ("no gate in the tracked universe",
     "no gate in the tracked universe is attributable to this family"),
    ("unmet_closure_criteria_count", "the family-status gate records unmet closure criteria"),
    ("no artifact on this machine — result UNPROVEN",
     "the external-corpus gate has produced no artifact in this tree"),
]
COVERING_RE = re.compile(r"(\d+) covering gate targets; (\d+) run, (\d+) run by nothing")
UNRUN_RE = re.compile(r"invoked by nothing: (\S+)")


def self_check():
    """An instrument with no ground truth is a confident guess. Both a positive and a NEGATIVE are
    pinned, and a miss REFUSES rather than reporting a clean read."""
    cases = [
        (["no *_parser_family_status_gate computes this family's closure at all"],
         "no family-status gate computes this family's closure at all"),
        (["  ⛔ a TRIAGE gate is not a conformance gate"],
         "the only gate reading the external corpus is a TRIAGE gate, and a triage gate is not a conformance gate"),
        # DONE-BAR.3c: the conformance gate's assertion wins over a triage sibling's line, whichever
        # order the audit prints them in — and the informational triage line alone matches nothing.
        (["    ℹ️ a TRIAGE gate characterizes; the family's CONFORMANCE gate carries the assertion",
          "    ⛔ pass_query `.full_pass == true` is FALSE over the TRACKED manifests (rows=16336) — leg 3 UNMET by the surface's own assertion (tracked derivation)"],
         "the external-corpus CONFORMANCE gate's own assertion is false — `full_pass` needs zero unexplained divergences on "
         "both lanes and every row adjudicated under its declared reading (the preprocess→parse pipeline for a row the raw "
         "lane defers to it; the permanently unadjudicable classes named and bounded, never counted as a gap), and rows "
         "still carry no verdict under that reading (`corpus_conformance_compare.py --assert-tracked` prints the count)"),
        (["    ⛔ a TRIAGE gate is not a conformance gate",
          "    ✅ pass_query `.full_pass == true` holds over the TRACKED manifests (rows=4) — leg 3 MET by the surface's own assertion (tracked derivation)"],
         "the external-corpus conformance gate's own assertion holds — zero unexplained divergences on both lanes and every row adjudicated under its declared reading, the permanently unadjudicable classes named and bounded"),
        (["    ℹ️ a TRIAGE gate characterizes; the family's CONFORMANCE gate carries the assertion"], None),
        (["some sentence this table has never seen"], None),
    ]
    for lines, want in cases:
        got = reason_for(lines)
        if got != want:
            refuse(f"the leg-reason table does not discriminate: {lines!r} -> {got!r}, wanted {want!r}")


def reason_for(lines):
    for token, sentence in LEG_REASONS:
        for ln in lines:
            if token in ln:
                return sentence
    return None


def parse_audit_text(path):
    """Split the audit report into per-family leg sections. Strict by construction: a family or a
    leg this cannot locate is a refusal, not an empty section."""
    fams, cur, leg = {}, None, None
    head = re.compile(r"^(\S+)   claimed: ")
    legline = re.compile(r"^  leg (\d) [^ ].*?\s{2,}(MET|UNMET|UNPROVEN)\s*$")
    for raw in read(path).split("\n"):
        m = head.match(raw)
        if m:
            cur = {"legs": {}}
            fams[m.group(1)] = cur
            leg = None
            continue
        if cur is None:
            continue
        m = legline.match(raw)
        if m:
            leg = {"verdict": m.group(2), "lines": []}
            cur["legs"][int(m.group(1))] = leg
            continue
        if leg is not None and raw.startswith("      "):
            leg["lines"].append(raw.strip())
        elif raw.startswith("  ⇒"):
            leg = None
    return fams


self_check()
tracked = parse_audit_text(os.path.join(WORK, "tracked.txt"))
local = parse_audit_text(os.path.join(WORK, "local.txt"))

# ---------------------------------------------------------------------------------------------
# 3. The remaining work — the OPEN trees that own work for this family.
# ---------------------------------------------------------------------------------------------
# ⛔⛔ THE ITEM IS A TREE, NOT A LEAF, AND THAT IS A MEASURED LIMIT RATHER THAN A CHOICE.
# A leaf's status is free prose inside a Markdown heading. Censused 2026-08-30 over every tracked
# instrument (`git grep -n -- todo -- 'scripts/*.sh' 'rust/scripts/*.sh' 'rust/scripts/lib/*.sh'
# '.githooks/*' '.github/workflows/*'`): exactly ONE reads a leaf status at all —
# `scripts/audit_done_bar.sh:701`, and only for the single hard-coded leaf `DONE-BAR.5`. So leaf
# status is unchecked as a CLASS, which is not the same as unchecked everywhere, and the difference
# is worth stating. This repository has already measured what the prose costs: two leaves read
# `todo` and `blocked` for 6 and 9 days after their fixes shipped
# (docs/knowledge/a-status-that-is-prose-is-a-status-nothing-checks.md), and THREE leaf-definition
# conventions are in live use, so "is this leaf id defined" is not yet mechanically answerable
# (DOCTRINE-GAP-OWNERSHIP.8). Deriving a per-LEAF list today would inherit both defects and publish
# them as a number. The TREE is the granularity the index actually maintains -- COMMIT.md obliges
# updating its row when the frontier moves -- so that is what is published, with the detail left
# where it is maintained rather than copied (a copy rots: DERIVED_STATE_CONTAINMENT R1/R3).
INDEX = "docs/TASK_TREE.md"
OPEN_WORDS = ("active", "proposed", "pending", "blocked", "paused", "todo")
CLOSED_WORDS = ("done", "complete", "completed", "superseded", "closed")


def polarity(cell):
    """OPEN / CLOSED / UNREADABLE from a prose status cell — the FIRST status word wins.

    ⛔⛔ THE FIRST CUT TOOK *ANY* OPEN WORD IN THE CELL AS OPEN, AND THAT MIS-ATTRIBUTED ITS OWN
    BIAS TO THE TRACKER. The index's convention is `| TREE | status (narration...) |`, so the row's
    own status is the FIRST status token; everything after it is narration ABOUT LEAVES. Measured
    2026-08-30: under any-token-wins, `VERILOG-2005-PROFILE` read OPEN because leaf `.6.3` is
    described as `blocked` deep in its cell — while the row's own first token is `complete` and the
    tree file's Metadata agrees. It was published to the director as one of *four* tracker
    disagreements; hand-checking the four showed three real and that one manufactured HERE.
    ⇒ a safe-polarity default is still a MEASUREMENT, and reporting it as a property of the subject
    is a different claim from reporting it as a property of the reader.
    ⛔ The bias is NOT removed, it is moved to where it belongs: a cell with NO status token at all
    is UNREADABLE, and every caller counts UNREADABLE as OPEN and NAMES the row. Over-reporting
    remaining work stays the safe direction; inventing a disagreement does not.
    ⭐ PRICED BEFORE ADOPTING, across the whole population: of 113 rows exactly ONE verdict moves
    (`VERILOG-2005-PROFILE`, OPEN -> CLOSED), and that one is hand-verified in both arms."""
    for tok in re.findall(r"`([A-Za-z]+)`", cell):
        low = tok.lower()
        if low in OPEN_WORDS:
            return "OPEN"
        if low in CLOSED_WORDS:
            return "CLOSED"
    return "UNREADABLE"


def rw_self_check():
    cases = [
        ("`active` (created 2026-08-30)", "OPEN"),
        ("`done` (all leaves closed)", "CLOSED"),
        # a tree whose cell shouts DONE about a LEAF while the tree itself is active
        ("✅✅ `.13e.10`(c4) RESIDUE CLOSED — tree stays `active`", "OPEN"),
        # ⛔ THE REGRESSION THIS READER SHIPPED ONCE: the row's own status is `complete` and a LEAF
        # deep in the narration is `blocked`. Any-token-wins read OPEN and published a tracker
        # disagreement that did not exist. First-token-wins reads the row.
        ("`complete` (`.1` done `-0002`; `.6.3` **`blocked`** checkpoint)", "CLOSED"),
        # no backticked status word at all: the index's own most-active rows look like this
        ("⛔⛔ **THE GAP IS MEASURED** — see the leaf", "UNREADABLE"),
    ]
    for cell, want in cases:
        got = polarity(cell)
        if got != want:
            refuse(f"the status reader does not discriminate: {cell!r} -> {got}, wanted {want}")


rw_self_check()

INDEX_ROW_RE = re.compile(r"^\| `([^`]+)` \|")


def index_rows():
    """Every row of the three tree tables: (tree, status cell, tree file). Section-scoped so a
    table added elsewhere in the document cannot silently enter the population."""
    lines = read(INDEX).split("\n")
    heads = {l.strip(): i for i, l in enumerate(lines) if l.startswith("## ")}
    wanted = ("## Active Task Trees", "## Proposed Task Trees", "## Completed Task Trees")
    missing = [w for w in wanted if w not in heads]
    if missing:
        refuse(f"{INDEX} has no {missing} section(s); the tree population cannot be read")
    out = []
    for w in wanted:
        i = heads[w]
        end = min([v for v in heads.values() if v > i] or [len(lines)])
        for ln in lines[i:end]:
            m = INDEX_ROW_RE.match(ln)
            if not m:
                continue
            cells = ln.split("|")
            if len(cells) < 6:
                continue
            # ⛔ The tree-file link is searched over the WHOLE ROW, not over cells[5]. A row whose
            # cell count disagrees with its header — 14 of 67 rows of this very index did, and the
            # four widest were the four most active trees (LIVE-DOC-CONTAINMENT.8) — puts the File
            # cell somewhere else, and a fixed index silently reports the most active trees as
            # having no tree file.
            link = re.search(r"\(tasks/([^)]+\.md)\)", ln)
            out.append((m.group(1), cells[2], link.group(1) if link else None))
    return out


def meta_status(fname):
    """The tree file's OWN Metadata Status — the second arm. None when the file does not carry one
    (30 of 113 do not), which is reported rather than assumed to agree."""
    if not fname:
        return None
    path = os.path.join("docs", "tasks", fname)
    if not os.path.isfile(path):
        return None
    m = re.search(r"^#{1,6}[ \t]+Metadata[ \t]*$(.*?)(?=^#{1,6}[ \t]|\Z)", read(path), re.M | re.S | re.I)
    if not m:
        return None
    s = re.search(r"^[-*][ \t]*(?:\*\*)?Status(?:\*\*)?[ \t]*:[ \t]*(.+)$", m.group(1), re.M | re.I)
    if not s:
        return None
    val = s.group(1)
    if "`" in val:
        return polarity(val)
    low = val.lower()
    if any(w in low for w in OPEN_WORDS):
        return "OPEN"
    if any(w in low for w in CLOSED_WORDS):
        return "CLOSED"
    return "UNREADABLE"


attrib = schema.get("tree_attribution", {}).get("trees", {})
rows = index_rows()
row_names = {r[0] for r in rows}
unattributed = sorted(row_names - set(attrib))
unknown = sorted(set(attrib) - row_names)
if unattributed or unknown:
    refuse(f"the tree-attribution population is not closed: {len(unattributed)} tree row(s) carry no "
           f"attribution {unattributed[:6]}, {len(unknown)} attribution(s) name no row {unknown[:6]}. "
           f"Every row of {INDEX} must be adjudicated exactly once in `tree_attribution.trees` of "
           f"{SCHEMA_PATH} — that is the mechanism that stops the roster re-growing silently.")

# ⛔ THE OPEN/CLOSED VERDICT IS A PROPERTY OF THE ROW, NOT OF THE FAMILY ASKING, so it is computed
# ONCE here rather than inside the per-family pass. Two reasons, and the second is the one that
# matters: the cross-family view derives ten families and would otherwise re-read all the tree
# files ten times over, and a per-family recomputation is a per-family opportunity for the ten
# answers to disagree about the same row.
row_state = []
for name, cell, fname in rows:
    a = polarity(cell)
    b = meta_status(fname)
    disputed_row = bool(b) and a != b and "UNREADABLE" not in (a, b)
    unreadable_row = (not disputed_row) and a == "UNREADABLE"
    state = "OPEN" if (disputed_row or unreadable_row) else a
    row_state.append((name, attrib[name], state, a, b, fname, disputed_row, unreadable_row))

# The attribution partition of the OPEN rows. Every row is adjudicated to exactly one bucket — the
# two-sided population refusal above is what makes that a closed statement rather than a hope — so
# these counts sum to the open total, and the cross-family view publishes the sum as such.
open_by_class = {}
for _row in row_state:
    if _row[2] == "OPEN":
        open_by_class[_row[1]] = open_by_class.get(_row[1], 0) + 1
n_open = sum(open_by_class.values())
n_disputed_rows = sum(1 for _row in row_state if _row[6])
n_unreadable_rows = sum(1 for _row in row_state if _row[7])


# ---------------------------------------------------------------------------------------------
# The PER-FAMILY DERIVATION — one pass, called once for a single family's rendering and ten times
# for the cross-family view.
# ---------------------------------------------------------------------------------------------
# ⭐⭐ THIS FUNCTION IS THE WHOLE ANTI-DRIFT ARGUMENT. The brief chat form, the ten published book
# blocks and the cross-family table in the platform book are three RENDERINGS of what it returns,
# never three readers of the same sources. A cross-family table computing its own numbers would be
# an eleventh surface to keep in sync — the very class this tree exists to remove — and gating it
# would only make the disagreement loud. Deriving it here makes the disagreement unrepresentable.
def derive(FAMILY):
    if FAMILY not in register["families"]:
        refuse(f"'{FAMILY}' is not a parser family. The roster is derived, not typed — it is the "
               f"`families` map of {REGISTER_PATH}: {', '.join(sorted(register['families']))}")
    entry = register["families"][FAMILY]
    for label, parsed in (("tracked-only", tracked), ("local", local)):
        if FAMILY not in parsed:
            refuse(f"the {label} audit report has no section for '{FAMILY}' — the report format this "
                   f"reader was built against has changed; fix this reader rather than publishing a "
                   f"block with no legs")
        if sorted(parsed[FAMILY]["legs"]) != [1, 2, 3]:
            refuse(f"the {label} audit report gives {FAMILY} legs "
                   f"{sorted(parsed[FAMILY]['legs'])}, not 1/2/3")

    legs = {}
    for n in (1, 2, 3):
        leg = tracked[FAMILY]["legs"][n]
        reason = reason_for(leg["lines"])
        if reason is None:
            refuse(f"leg {n} of {FAMILY} carries no reason this reader recognises. Its lines were:\n  "
                   + "\n  ".join(leg["lines"])
                   + "\nAdd the sentence to LEG_REASONS in scripts/report_family_status.sh — a leg is "
                     "never published without the reason behind it.")
        legs[n] = {"verdict": leg["verdict"], "reason": reason}

    # Leg 2 additionally carries COUNTS, and they are the distance a reader can act on.
    cov = None
    for ln in tracked[FAMILY]["legs"][2]["lines"]:
        m = COVERING_RE.search(ln)
        if m:
            cov = tuple(int(x) for x in m.groups())
    unrun = [m.group(1) for ln in tracked[FAMILY]["legs"][2]["lines"] for m in [UNRUN_RE.search(ln)] if m]
    if cov is None:
        refuse(f"leg 2 of {FAMILY} publishes no covering-gate count; the audit's line format has moved")

    # ⭐ LEG 2'S REASON IS COMPOSED ONCE, HERE, AND BOTH RENDERINGS USE IT — the counts ARE the reason
    # when a covering gate is invoked by nothing, and they lead the sentence for exactly that reason:
    # an uninvoked gate is what makes leg 2 UNMET, while the artifact half can only leave it UNPROVEN.
    # Composing it in each renderer instead would be two sentences free to drift into disagreement,
    # which is the defect this whole tree exists to remove.
    legs[2]["reason"] = (
        f"{cov[0]} covering gate targets, {cov[1]} invoked, {cov[2]} invoked by nothing"
        + (f" ({', '.join(unrun)})" if unrun else "")
        + f"; {legs[2]['reason']}")

    local_legs = {n: local[FAMILY]["legs"][n]["verdict"] for n in (1, 2, 3)}

    # ---------------------------------------------------------------------------------------------
    # 2. The corpus — counts read from the TRACKED census, freshness proved by the census's own gate.
    # ---------------------------------------------------------------------------------------------
    corpus = None
    census = schema.get("corpus_census", {}).get(FAMILY)
    if census:
        gate = census["freshness_gate"]
        proc = subprocess.run(gate, shell=True, capture_output=True, text=True, cwd=ROOT)
        out = (proc.stdout or "") + (proc.stderr or "")
        if proc.returncode != 0:
            refuse(f"the corpus freshness gate `{gate}` exited {proc.returncode}, so the census "
                   f"artifacts are not proven current and their counts must not be published. "
                   f"Its own words:\n  " + out.strip().replace("\n", "\n  "))
        marker = census["tuple_marker"]
        if marker not in out:
            refuse(f"`{gate}` printed no `{marker}` line, so there is no tuple to cross-check against")
        gate_tuple = out.split(marker, 1)[1].split()[0]

    # ⛔⛔ A CENSUS HAS A SHAPE, AND THE SHAPES ARE NOT INTERCHANGEABLE. SystemVerilog's corpus has
    # an ANSWER KEY, so its census partitions rows by VERDICT and a `known defects` count means
    # something. VHDL's does not: `characterization.md` says in its own opening paragraph that many
    # corpus files are intentionally invalid, so a parse-FAIL is frequently the CORRECT outcome. A
    # raw census rendered through the adjudicated template would publish `known defects: 0`, which
    # reads as *nothing is wrong* when the truth is *nobody has looked*. So `kind` dispatches, and
    # an unknown kind REFUSES rather than falling through to whichever template is first.
    if census and census.get("kind") == "raw_parse":
        # ⭐ THE PRODUCER DERIVES THE TUPLE AGAIN, FROM THE SAME TRACKED REPORT THE GATE READ, and
        # the two are held equal below. That duplication is the design, not an oversight: it is the
        # SystemVerilog arm's shape one family over — one artifact, two independent readers, and a
        # refusal if they disagree. A single reader shared by both would agree with itself.
        report_path = census["report"]
        if not os.path.isfile(report_path):
            refuse(f"the census report {report_path} declared for '{FAMILY}' does not exist")
        with open(report_path, encoding="utf-8") as fh:
            report_text = fh.read()

        def _table(heading, arity):
            body, taking, rows, seen_sep = [], False, [], False
            for line in report_text.splitlines():
                if re.match(r"^#{1,6}\s", line):
                    if taking:
                        break
                    taking = line.startswith("## ") and line[3:].strip() == heading
                    continue
                if taking:
                    body.append(line)
            for line in body:
                if "|" not in line:
                    if rows:
                        break
                    continue
                s = line.strip().strip("|")
                cells = [c.strip() for c in s.split("|")]
                if all(re.fullmatch(r":?-{2,}:?", c) for c in cells):
                    seen_sep = True
                    continue
                if not seen_sep:
                    continue
                if len(cells) != arity:
                    refuse(f"{report_path}: a `{heading}` row carries {len(cells)} cells, not "
                           f"{arity}: {cells}")
                rows.append(cells)
            if not rows:
                refuse(f"{report_path}: no `## {heading}` table — this producer will not publish a "
                       f"count it cannot read out of the artifact that holds it")
            return rows

        t = _table("Totals", 6)
        if len(t) != 1:
            refuse(f"{report_path}: the `## Totals` table must carry exactly one row, found {len(t)}")
        total, passing, rejected, timeout, crash = (int(c) for c in t[0][:5])
        subs = [dict(name=r[0], total=int(r[1]), passing=int(r[2]), rejected=int(r[3]),
                     timeout=int(r[4]), crash=int(r[5]))
                for r in _table("Per sub-corpus", 7)]

        derived_tuple = f"{total}/{passing}/{rejected}/{timeout}/{crash}"
        if derived_tuple != gate_tuple:
            refuse(f"this script derives the census tuple {derived_tuple} from {report_path} while "
                   f"`{gate}` derives {gate_tuple}. Two readings of one census disagree — neither "
                   f"may be published until they are reconciled.")
        if passing + rejected + timeout + crash != total:
            refuse(f"{report_path}: the outcome classes sum to "
                   f"{passing + rejected + timeout + crash}, not the published total {total}")

        # ⛔ `defects` is None, NOT ZERO, and every renderer must carry that through. There is no
        # answer key, so the number of KNOWN DEFECTS in this corpus is not zero — it is unknown,
        # and a zero in that cell is the exact shape of lie
        # docs/decisions/feedback_a_check_that_cannot_run_must_say_so.md exists to forbid.
        corpus = dict(kind="raw_parse", total=total, passing=passing, rejected=rejected,
                      timeout=timeout, crash=crash, subs=subs, defects=None,
                      tuple=derived_tuple, denominator=census["denominator"],
                      instrument=census["instrument"], gate=gate, report=report_path,
                      answer_key=census["answer_key"])
    elif census and census.get("kind") == "oracle_expectation":
        # ⭐ Same two-arm rule as every other kind: this script derives the tuple from the TRACKED
        # artifacts and the gate derives it from a FRESH normalizer run plus the anchor. Equal, or
        # refuse. The gate has already proved the tracked artifacts byte-equal to a fresh run, so
        # reading them here is reading the fresh numbers by a different route.
        with open(census["artifacts"]["summary_json"], encoding="utf-8") as fh:
            summ = json.load(fh)
        specs = summ["pattern_specs_detected_input"]
        cells = summ["cases_emitted"]
        skipped = dict(sorted(summ.get("skip_reason_counts", {}).items()))
        with open(census["verdict_anchor"]["file"], encoding="utf-8") as fh:
            found = sorted(set(re.findall(census["verdict_anchor"]["pattern"], fh.read())))
        if len(found) != 1:
            refuse(f"{census['verdict_anchor']['file']} carries {len(found)} distinct current "
                   f"verdict tuples ({found}); exactly one is publishable")
        a_cells, match, fa, fr = (int(x) for x in found[0].split("/"))
        derived_tuple = f"{a_cells}/{match}/{fa}/{fr}"
        if derived_tuple != gate_tuple:
            refuse(f"this script derives the census tuple {derived_tuple} while `{gate}` derives "
                   f"{gate_tuple}. Two readings of one census disagree — neither may be published "
                   f"until they are reconciled.")
        if a_cells != cells:
            refuse(f"the verdict tuple is over {a_cells} cells while the tracked summary emits "
                   f"{cells}")

        # ⛔ THE CLASS MAP IS THE SAME SEMANTIC AS SystemVerilog's, deliberately, because the two
        # sit side by side in the cross-family table and a column that means different things per
        # row is worse than no column. `passing` = the parser's verdict matches the key.
        # `explained` = divergence WITH a recorded explanation: every false REJECT is a documented
        # strict-default divergence class, recorded case-class by case-class in the v12 record of
        # the tracked ratchet env. `defects` = divergence with NO recorded explanation: the false
        # ACCEPTS, which the north star calls a defect outright — over-acceptance is a defect.
        corpus = dict(kind="oracle_expectation", total=cells, passing=match, explained=fr,
                      defects=fa, specs=specs, skipped=skipped,
                      tuple=derived_tuple, denominator=census["denominator"],
                      instrument=census["instrument"], gate=gate,
                      answer_key=census["answer_key"],
                      verdict_producer=census["verdict_producer"],
                      verdict_held_by=census["verdict_anchor"]["held_by"])
    elif census and census.get("kind") != "verdict_coverage":
        refuse(f"'{FAMILY}' declares census kind {census.get('kind')!r}, which no renderer here "
               f"implements. A census shape with no renderer must refuse: rendering it through "
               f"another shape's template publishes fields that do not mean what they say.")
    elif census:
        rows_by_class, buckets = {}, {}
        with open(census["coverage_tsv"], encoding="utf-8") as fh:
            fh.readline()
            for line in fh:
                if not line.strip():
                    continue
                bucket, cls, rows, _pct = line.rstrip("\n").split("\t")
                rows_by_class[cls] = rows_by_class.get(cls, 0) + int(rows)
                buckets[bucket] = buckets.get(bucket, 0) + int(rows)
        dark = 0
        with open(census["strata_tsv"], encoding="utf-8") as fh:
            fh.readline()
            for line in fh:
                if line.strip():
                    dark += int(line.rstrip("\n").split("\t")[4])

        passing = rows_by_class.get("match", 0)
        explained = sum(v for k, v in rows_by_class.items() if k.startswith("divergence:explained"))
        defects = sum(v for k, v in rows_by_class.items() if k.startswith("divergence:unexplained"))
        routed = buckets.get("ROUTED", 0)
        noverdict = buckets.get("NO VERDICT", 0)
        adjudicated = buckets.get("ADJUDICATED", 0)
        total = sum(buckets.values())

        # ⭐ THE CROSS-CHECK, and it is the reason these counts may be published at all: this script
        # derives the tuple from the artifacts and the GATE derives it independently. Equal, or refuse.
        derived_tuple = f"{adjudicated}/{routed}/{noverdict}/{dark}/{defects}"
        if derived_tuple != gate_tuple:
            refuse(f"this script derives the verdict-coverage tuple {derived_tuple} from "
                   f"{census['coverage_tsv']} while `{gate}` derives {gate_tuple}. Two readings of one "
                   f"census disagree — neither may be published until they are reconciled.")
        if adjudicated != passing + explained + defects:
            refuse(f"the adjudicated bucket ({adjudicated}) is not the sum of its classes "
                   f"({passing} + {explained} + {defects}); the class map has moved")
        corpus = dict(kind="verdict_coverage",
                      total=total, passing=passing, explained=explained, defects=defects,
                      routed=routed, noverdict=noverdict, adjudicated=adjudicated, dark=dark,
                      tuple=derived_tuple, denominator=census["denominator"],
                      instrument=census["instrument"], gate=gate)

    # ⛔⛔ WHY THERE IS NO COUNT IS ITSELF DERIVED, NOT TYPED. A family with no census must say WHY and
    # name the leaf that owes one — a blank reads as nothing-to-see and a zero reads as done. The CLASS
    # is computed here from the done-bar register and then held EQUAL to the class declared in the
    # schema: two arms, independently produced. Declaring it alone would let it drift from the register
    # the moment a corpus is wired, and that drift is the whole disease this tree treats.
    state_decl = schema.get("corpus_state", {}).get("families", {}).get(FAMILY)
    if state_decl is None:
        refuse(f"no `corpus_state` is declared for '{FAMILY}' in {SCHEMA_PATH}. Every family says why it "
               f"has no corpus count, and names the leaf that owes one — the population is closed.")
    owner = entry["language_owner"]
    if census:
        derived_class = "census_wired"
    elif owner == "pgen":
        derived_class = "no_third_party_corpus"
    elif owner == "unadjudicated":
        derived_class = "ownership_unadjudicated"
    elif entry["corpus_roots"]:
        derived_class = "corpus_wired_no_census"
    else:
        derived_class = "no_corpus_wired"
    if derived_class != state_decl["class"]:
        refuse(f"the corpus state for '{FAMILY}' is declared `{state_decl['class']}` but the done-bar "
               f"register derives `{derived_class}` (language_owner={owner}, "
               f"corpus_roots={entry['corpus_roots']}, census wired={bool(census)}). The declared class "
               f"has drifted from the register — fix the declaration, or wire the census it claims.")
    if derived_class != "no_third_party_corpus" and not census and not state_decl.get("owed_by"):
        refuse(f"'{FAMILY}' has no corpus count and names no leaf that owes one. A gap with no owner is "
               f"a gap nobody will close (docs/decisions/"
               f"feedback_every_finding_is_owned_and_scheduled_never_just_logged.md).")
    corpus_why = dict(state_decl, derived_class=derived_class,
                      blurb=schema["corpus_state"]["classes"][derived_class])

    # ---------------------------------------------------------------------------------------------
    # 2b. The reading leg 3 ASSERTS (DONE-BAR.3f / .3g) — the corpus table is the RAW lane; the bar
    # reads a row the raw lane defers to the preprocess→parse pipeline under that pipeline's
    # re-adjudication, and names (never demands) the permanently unadjudicable classes. The counts
    # come from the register's `tracked_assertion_command` — the SAME command the done-bar audit
    # evaluates `pass_query` over, so the book and the audit read one derivation — and the sv_2017
    # lane's reading is CROSS-CHECKED against the tuple under the pipeline the census gate prints:
    # two instruments, one reading, equal or refuse. A family declaring no tracked assertion renders
    # NOT MEASURED with the owing leaf, never a blank.
    # ---------------------------------------------------------------------------------------------
    declared = None
    surface = entry.get("leg3_surface") or {}
    assertion_cmd = surface.get("tracked_assertion_command") or ""
    if assertion_cmd:
        proc = subprocess.run(assertion_cmd, shell=True, capture_output=True, text=True, cwd=ROOT)
        try:
            data = json.loads(proc.stdout)
        except ValueError:
            refuse(f"the tracked leg-3 assertion of '{FAMILY}' (`{assertion_cmd}`) printed no JSON "
                   f"(exit {proc.returncode}); its declared-reading counts must not be published. Its "
                   f"own words:\n  " + ((proc.stdout or "") + (proc.stderr or "")).strip().replace("\n", "\n  "))
        if "refused" in data:
            refuse(f"the tracked leg-3 assertion of '{FAMILY}' REFUSED: {data['refused']} — a refusal is "
                   f"never rendered as a count")
        need = ("rows", "declared_adjudicated", "permanent_no_verdict", "burnable_no_verdict", "unexplained",
                "declared_bar", "readjudication_stale", "full_pass", "full_pass_raw_lane")
        missing = [k for k in need if k not in data]
        if missing:
            refuse(f"the tracked leg-3 assertion of '{FAMILY}' carries no {missing}; the comparator's "
                   f"output shape has moved — fix this reader rather than publishing a partial reading")
        if data["rows"] != data["declared_adjudicated"] + data["permanent_no_verdict"] + data["burnable_no_verdict"]:
            refuse(f"the declared reading of '{FAMILY}' does not partition the corpus: rows={data['rows']} but "
                   f"adjudicated {data['declared_adjudicated']} + permanent {data['permanent_no_verdict']} + "
                   f"burnable {data['burnable_no_verdict']} = "
                   f"{data['declared_adjudicated'] + data['permanent_no_verdict'] + data['burnable_no_verdict']}")
        lanes = []
        for lane_key in ("sv_2017", "verilog_2005"):
            l = data.get(lane_key)
            if not isinstance(l, dict):
                continue
            lanes.append(dict(lane=lane_key, rows=l["rows"],
                              adjudicated=l.get("declared_adjudicated", l["adjudicated"]),
                              permanent=l["permanent_no_verdict"], burnable=l["burnable_no_verdict"],
                              unexplained=l["unexplained"], bar=l.get("declared_bar", l["unexplained"]),
                              stale=l.get("readjudication_stale", 0), full_pass=l["full_pass"],
                              reading=l.get("declared_reading", "")))
        pipe_marker = (census or {}).get("pipeline_tuple_marker")
        if pipe_marker and isinstance(data.get("sv_2017"), dict):
            if pipe_marker not in out:
                refuse(f"`{gate}` printed no `{pipe_marker}` tuple, so the declared reading of "
                       f"'{FAMILY}' cannot be cross-checked and must not be published")
            # the gate prints `… under the pipeline A/R/N/D/B; artifacts fresh: …` — the tuple ends at the
            # first white space, with the sentence's punctuation trimmed
            gate_pipe = out.split(pipe_marker, 1)[1].split()[0].rstrip(";,.)")
            try:
                pa, pr, pn, _pd, pb = (int(x) for x in gate_pipe.split("/"))
            except ValueError:
                refuse(f"`{gate}` printed a tuple under the pipeline this reader cannot parse: {gate_pipe!r}")
            l = data["sv_2017"]
            mine = (l["declared_adjudicated"], l["declared_routed"], l["declared_no_verdict"], l["declared_bar"])
            if mine != (pa, pr, pn, pb):
                refuse(f"the tracked leg-3 assertion reads the sv_2017 lane under the pipeline as "
                       f"{'/'.join(str(x) for x in mine)} (adjudicated/routed/no-verdict/bar) while `{gate}` derives "
                       f"{pa}/{pr}/{pn}/{pb}. Two readings of one lane disagree — neither may be published "
                       f"until they are reconciled.")
        declared = dict(rows=data["rows"], adjudicated=data["declared_adjudicated"],
                        permanent=data["permanent_no_verdict"], burnable=data["burnable_no_verdict"],
                        unexplained=data["unexplained"], bar=data["declared_bar"],
                        stale=data["readjudication_stale"], full_pass=data["full_pass"],
                        full_pass_raw_lane=data["full_pass_raw_lane"], lanes=lanes, command=assertion_cmd,
                        cross_checked_by=(gate if (pipe_marker and census) else None))

    # --- 3. the remaining work: this family's slice of the row states computed once above -------
    family_open, disputed, unreadable_rows, cross_open = [], [], [], 0
    for name, who, state, a, b, fname, disputed_row, unreadable_row in row_state:
        if disputed_row and who in (FAMILY, "cross-cutting"):
            disputed.append((name, a, b))
        elif unreadable_row and who == FAMILY:
            unreadable_rows.append(name)
        if state != "OPEN":
            continue
        if who == FAMILY:
            family_open.append((name, fname, a, b))
        elif who == "cross-cutting":
            cross_open += 1
    family_open.sort()

    return dict(family=FAMILY, entry=entry, legs=legs, local_legs=local_legs, corpus=corpus,
                corpus_why=corpus_why, declared=declared, family_open=family_open, disputed=disputed,
                unreadable_rows=unreadable_rows, cross_open=cross_open,
                unmet=[n for n in (1, 2, 3) if legs[n]["verdict"] != "MET"])


# ---------------------------------------------------------------------------------------------
# 4. Render — and hold the rendered field set EQUAL to the schema's, in both directions.
# ---------------------------------------------------------------------------------------------
def pct(n, d):
    return "—" if not d else f"{100.0 * n / d:.1f} %"


def fmt(n):
    return f"{n:,}".replace(",", " ")


# ⛔ WHICH SURFACE IS BEING PUBLISHED TO, AND WHICH FIELDS THAT SURFACE MAY CARRY. The schema
# declares it and this holds the two equal in BOTH directions: a field rendered with no declared
# producer is the hand-written status this tree exists to remove, and a field declared but rendered
# nowhere is a promise no surface keeps. The cross-family view is a THIRD surface, declared like the
# other two. It deliberately drops `language_owner` — the table it sits directly under already
# publishes that column, and a second copy on the same page is a second thing to keep true — and,
# like the book, it never publishes this machine's untracked gate artifacts.
SURFACE = "cross" if CROSS else ("book" if MODE == "markdown" else "chat")
rendered = {"claimed_status", "bar_leg_1", "bar_leg_2", "bar_leg_3",
            "corpus_rows", "corpus_passing", "known_defects", "remaining_work"}
if SURFACE != "cross":
    rendered.add("language_owner")
    rendered.add("leg3_declared_reading")
if SURFACE == "chat":
    rendered.add("local_gate_state")
declared = {k for k, v in schema["fields"].items() if SURFACE in v["surfaces"]}
if rendered != declared:
    refuse(f"the rendered field set and the schema disagree: only-rendered={sorted(rendered - declared)}, "
           f"only-declared={sorted(declared - rendered)}. Every field is declared in {SCHEMA_PATH} "
           f"with the producer that answers it; a field rendered without a declared producer is the "
           f"hand-written status this tree exists to remove.")

LEGNAME = {1: "stimuli-generator proof", 2: "gates green and invoked", 3: "external corpus passing"}
MARK = {"MET": "✅", "UNMET": "⛔", "UNPROVEN": "⚠️"}


def render_markdown(d):
    FAMILY, entry, legs = d["family"], d["entry"], d["legs"]
    corpus, corpus_why, unmet = d["corpus"], d["corpus_why"], d["unmet"]
    family_open, cross_open = d["family_open"], d["cross_open"]
    out = []
    out.append(f"<!-- BEGIN DERIVED: scripts/report_family_status.sh {FAMILY} --markdown -->")
    out.append("")
    out.append(f"**Claimed status:** `{entry['claimed_status']}` · **language owner:** "
               f"`{entry['language_owner']}` · **standard:** {entry['standard']}")
    out.append("")
    out.append(f"**Distance to `Done`: {len(unmet)} of the 3 legs are not met.**")
    out.append("")
    out.append("| Leg | State | Why, derived from the tracked tree |")
    out.append("|---|---|---|")
    for n in (1, 2, 3):
        out.append(f"| {n} — {LEGNAME[n]} | {MARK[legs[n]['verdict']]} **{legs[n]['verdict']}** | "
                   f"{legs[n]['reason']} |")
    out.append("")
    if corpus and corpus["kind"] == "oracle_expectation":
        skipped = sum(corpus["skipped"].values())
        reasons = ", ".join(f"{fmt(v)} `{k}`" for k, v in corpus["skipped"].items())
        out.append(f"**Corpus.** Denominator: {corpus['denominator']} — **{fmt(corpus['total'])} "
                   f"cells** out of {fmt(corpus['specs'])} pattern specs detected, counted by "
                   f"`{corpus['instrument']}` and held current by `{corpus['gate']}`.")
        out.append("")
        out.append("| Outcome | Cells | Share of the corpus |")
        out.append("|---|---:|---:|")
        out.append(f"| **Matches the oracle** — PGEN's compile verdict equals PCRE2's | {fmt(corpus['passing'])} | {pct(corpus['passing'], corpus['total'])} |")
        out.append(f"| **False accepts** — PGEN accepts a pattern PCRE2 rejects | {fmt(corpus['defects'])} | {pct(corpus['defects'], corpus['total'])} |")
        out.append(f"| False rejects, every one an explained divergence | {fmt(corpus['explained'])} | {pct(corpus['explained'], corpus['total'])} |")
        out.append(f"| Total | {fmt(corpus['total'])} | 100.0 % |")
        out.append("")
        out.append(f"⭐ **Unlike the VHDL census, this pass share IS a conformance share**: "
                   f"{cell(corpus['answer_key'].replace('`', ''))}")
        out.append("")
        out.append(f"**The declined {fmt(skipped)} are named, not dropped** — {reasons}. They are "
                   f"counted in the {fmt(corpus['specs'])}-spec denominator above, because a case "
                   f"the census refused to ask is not a case that passed.")
        out.append("")
        out.append(f"⚠️ **The verdict half is anchor-held, not re-executed on every commit.** "
                   f"Producing it needs a cargo build of the corpus probe and a run over every "
                   f"case — measured 80 s and 5.5 GB peak, via `{corpus['verdict_producer']}` — so "
                   f"`{corpus['gate']}` re-derives the DENOMINATOR from the normalizer on every "
                   f"commit and holds the published tuple's cell count equal to it, while "
                   f"`{corpus['verdict_held_by']}` holds every live anchor equal to each other and "
                   f"to the tracked ratchet bounds.")
    elif corpus and corpus["kind"] == "raw_parse":
        out.append(f"**Corpus.** Denominator: {corpus['denominator']} — **{fmt(corpus['total'])} "
                   f"files**, counted by `{corpus['instrument']}`, published in "
                   f"[`{corpus['report']}`](../../../{corpus['report']}) and held current by "
                   f"`{corpus['gate']}`.")
        out.append("")
        out.append("| Outcome | Files | Share of the corpus |")
        out.append("|---|---:|---:|")
        out.append(f"| **Parses** | {fmt(corpus['passing'])} | {pct(corpus['passing'], corpus['total'])} |")
        out.append(f"| Rejected | {fmt(corpus['rejected'])} | {pct(corpus['rejected'], corpus['total'])} |")
        out.append(f"| Timed out | {fmt(corpus['timeout'])} | {pct(corpus['timeout'], corpus['total'])} |")
        out.append(f"| Crashed | {fmt(corpus['crash'])} | {pct(corpus['crash'], corpus['total'])} |")
        out.append(f"| Total | {fmt(corpus['total'])} | 100.0 % |")
        out.append("")
        out.append(f"⛔ **This is a RAW parse rate and it moves leg 3 not at all.** "
                   f"{cell(corpus['answer_key'].replace('`', ''))} So there is no *known defects* "
                   f"count here, and that blank is not a zero: nobody has looked. The adjudication "
                   f"is owed by `{corpus_why['owed_by']}`. What the number IS good for is drift — "
                   f"it is re-derived on every commit, so a grammar or corpus change that moves it "
                   f"can no longer pass unnoticed.")
        out.append("")
        out.append("| Sub-corpus | Files | Parses | Rejected | Share |")
        out.append("|---|---:|---:|---:|---:|")
        for s in corpus["subs"]:
            out.append(f"| `{cell(s['name'])}` | {fmt(s['total'])} | {fmt(s['passing'])} | "
                       f"{fmt(s['rejected'])} | {pct(s['passing'], s['total'])} |")
    elif corpus:
        out.append(f"**Corpus.** Denominator: {corpus['denominator']} — **{fmt(corpus['total'])} "
                   f"rows**, counted by `{corpus['instrument']}` and held current by "
                   f"`{corpus['gate']}`.")
        out.append("")
        out.append("| Outcome | Rows | Share of the corpus |")
        out.append("|---|---:|---:|")
        out.append(f"| **Passing** — the parser's verdict matches the answer key | {fmt(corpus['passing'])} | {pct(corpus['passing'], corpus['total'])} |")
        out.append(f"| Divergence with a recorded explanation | {fmt(corpus['explained'])} | {pct(corpus['explained'], corpus['total'])} |")
        out.append(f"| **Known defects** — divergence with no explanation | {fmt(corpus['defects'])} | {pct(corpus['defects'], corpus['total'])} |")
        out.append(f"| Routed to another lane's manifest | {fmt(corpus['routed'])} | {pct(corpus['routed'], corpus['total'])} |")
        out.append(f"| **No verdict** — nothing is known about these rows | {fmt(corpus['noverdict'])} | {pct(corpus['noverdict'], corpus['total'])} |")
        out.append(f"| Total | {fmt(corpus['total'])} | 100.0 % |")
        out.append("")
        out.append(f"Only **{fmt(corpus['adjudicated'])} rows ({pct(corpus['adjudicated'], corpus['total'])})** "
                   f"were asked a question they could answer, and **{fmt(corpus['dark'])}** of the "
                   f"no-verdict rows are *dark* — the parse failed and nobody has looked at why. "
                   f"⛔ The pass share above is a share of the WHOLE corpus, not of the adjudicated "
                   f"part: reporting {corpus['passing']}/{corpus['adjudicated']} would quietly "
                   f"drop the {pct(corpus['noverdict'], corpus['total'])} carrying no verdict at "
                   f"all and the {pct(corpus['routed'], corpus['total'])} answered in another "
                   f"lane's manifest.")
    else:
        head = ("**Corpus.** ⛔ **NO THIRD-PARTY CORPUS EXISTS** — "
                if corpus_why["derived_class"] == "no_third_party_corpus"
                else "**Corpus.** ⛔ **NOT MEASURED** — ")
        line = head + corpus_why["blurb"] + "."
        if corpus_why.get("owed_by"):
            line += f" Owed by `{corpus_why['owed_by']}`."
        if corpus_why.get("last_measured_in"):
            line += (f" The last measurement lives in `{corpus_why['last_measured_in']}` and is "
                     f"POINTED AT rather than copied here, because a copied number rots.")
        if corpus_why.get("note"):
            line += " " + corpus_why["note"]
        out.append(line)
    declared = d["declared"]
    out.append("")
    if declared:
        lanes = declared["lanes"]
        out.append(f"**Under the reading leg 3 asserts** (`DONE-BAR.3f`): the table above is the RAW lane; "
                   f"the bar reads a row the raw lane defers to the preprocess→parse pipeline under that "
                   f"pipeline's re-adjudication, and names — never demands — the permanently unadjudicable "
                   f"classes, each held under a recorded ceiling. Derived by the register's tracked assertion "
                   f"(`{cell(declared['command'])}`)"
                   + (f", its sv_2017 reading cross-checked against the tuple under the pipeline that "
                      f"`{declared['cross_checked_by']}` prints" if declared.get("cross_checked_by") else "")
                   + ".")
        out.append("")
        out.append("| Lane | Rows | Adjudicated | Permanently unadjudicable (named, bounded) | **Still awaiting a verdict** | Unexplained | Bar rows | `full_pass` |")
        out.append("|---|---:|---:|---:|---:|---:|---:|---|")
        for l in lanes:
            label = {"sv_2017": "`sv_2017` — the corpus, raw rows read under the pipeline where it re-read them",
                     "verilog_2005": "`verilog_2005` — the routed rows, read raw under IEEE 1364-2005"}.get(l["lane"], f"`{l['lane']}`")
            out.append(f"| {label} | {fmt(l['rows'])} | {fmt(l['adjudicated'])} | {fmt(l['permanent'])} | "
                       f"**{fmt(l['burnable'])}** | {fmt(l['unexplained'])} | {fmt(l['bar'])} | `{str(l['full_pass']).lower()}` |")
        out.append(f"| **Both lanes — the corpus** | {fmt(declared['rows'])} | {fmt(declared['adjudicated'])} | "
                   f"{fmt(declared['permanent'])} | **{fmt(declared['burnable'])}** | {fmt(declared['unexplained'])} | "
                   f"{fmt(declared['bar'])} | `{str(declared['full_pass']).lower()}` |")
        out.append("")
        out.append(f"The three row counts sum to the denominator ({fmt(declared['adjudicated'])} + "
                   f"{fmt(declared['permanent'])} + {fmt(declared['burnable'])} = {fmt(declared['rows'])}). "
                   f"**{fmt(declared['burnable'])} rows still stand between this corpus and leg 3** — every one in a "
                   f"named class with an owning leaf; the permanently unadjudicable rows stay in the raw table above "
                   f"as no-verdict rows and are never relabelled. "
                   + (f"The re-adjudication table is current with the raw classes ({fmt(declared['stale'])} stale rows). "
                      if declared["stale"] == 0 else
                      f"⛔ {fmt(declared['stale'])} re-adjudication rows are STALE against the raw classes and fail the pass. ")
                   + f"The pre-`.3f` bit (`no_verdict == 0` on the raw lane, unreachable by construction) reads "
                   f"`{str(declared['full_pass_raw_lane']).lower()}` and is kept by name as `full_pass_raw_lane`.")
    else:
        owed = schema["fields"]["leg3_declared_reading"].get("owed_by_when_absent", "DONE-BAR.3")
        out.append(f"**Under the reading leg 3 asserts:** ⛔ **NOT MEASURED** — the done-bar register declares no "
                   f"tracked leg-3 assertion for this family (`leg3_surface.tracked_assertion_command`), so there is "
                   f"no declared reading to count under; owed by `{owed}`.")

    out.append("")
    out.append(f"**What remains between here and `Done`.** Derived from the task-tree index: the "
               f"trees still OPEN whose work is attributed to this family, out of the "
               f"{len(rows)} trees the index carries. **{len(family_open)} tree"
               f"{'' if len(family_open) == 1 else 's'} own this family's remaining work**, and "
               f"**{cross_open}** cross-cutting trees can gate any family.")
    out.append("")
    if family_open:
        out.append("| Open tree | Status verdict | Where the detail lives |")
        out.append("|---|---|---|")
        for name, fname, a, b in family_open:
            note = "open"
            if b and a != b and "UNREADABLE" not in (a, b):
                note = f"⚠️ DISPUTED — the index reads {a}, the tree file reads {b}; counted OPEN"
            elif a == "UNREADABLE":
                note = "⚠️ the index row carries no status word; counted OPEN"
            where = f"[`docs/tasks/{fname}`](../../tasks/{fname})" if fname else "*(no tree file linked)*"
            out.append(f"| `{name}` | {note} | {where} |")
    else:
        out.append("No open tree is attributed to this family — every tree that owned its work is "
                   "closed. That is not the same as meeting the bar: the legs above are what decide "
                   "`Done`, and cross-cutting work can still gate this family.")
    out.append("")
    out.append("⚠️ **The item here is a TREE, not a leaf, and that is a measured limit.** A leaf's "
               "status is free prose, and this repository has measured leaves reading `todo` for "
               "days after their fix shipped, so a per-leaf list would publish that defect as a "
               "number. The two prose arms — the index row and the tree file's own metadata — "
               "disagree on some trees, and every disagreement is counted OPEN and shown above: "
               "over-reporting is the only safe direction for a list that says how much is left.")
    out.append("")
    out.append("<!-- END DERIVED -->")
    return out


def render_text(d):
    FAMILY, entry, legs = d["family"], d["entry"], d["legs"]
    corpus, corpus_why, unmet = d["corpus"], d["corpus_why"], d["unmet"]
    family_open, cross_open = d["family_open"], d["cross_open"]
    local_legs, disputed, unreadable_rows = d["local_legs"], d["disputed"], d["unreadable_rows"]
    out = []
    out.append(f"FAMILY STATUS — {FAMILY}   (every figure derived; nothing here is hand-written)")
    out.append(f"  claimed status ......... {entry['claimed_status']}   [done-bar register]")
    out.append(f"  language owner ......... {entry['language_owner']}   [{entry['standard'][:60]}]")
    out.append(f"  distance to `Done` ..... {len(unmet)} of 3 legs not met")
    out.append("")
    for n in (1, 2, 3):
        out.append(f"  leg {n} {LEGNAME[n]:26s} {legs[n]['verdict']:9s} {legs[n]['reason']}")
    out.append(f"  on THIS machine (gate artifacts included, not tracked): "
               f"leg1={local_legs[1]} leg2={local_legs[2]} leg3={local_legs[3]}")
    out.append("")
    if corpus and corpus["kind"] == "oracle_expectation":
        skipped = sum(corpus["skipped"].values())
        out.append(f"  corpus  {fmt(corpus['total'])} cells of {fmt(corpus['specs'])} pattern specs "
                   f"({fmt(skipped)} declined)  [{corpus['denominator'][:52]}]")
        out.append(f"    matches the oracle .................. {fmt(corpus['passing']):>7s}  {pct(corpus['passing'], corpus['total'])}")
        out.append(f"    FALSE ACCEPTS (over-acceptance) ..... {fmt(corpus['defects']):>7s}  {pct(corpus['defects'], corpus['total'])}")
        out.append(f"    false rejects, all explained ........ {fmt(corpus['explained']):>7s}  {pct(corpus['explained'], corpus['total'])}")
        out.append(f"    => a REAL answer key, so this pass share IS a conformance share")
        out.append(f"    => verdict half not re-run here (80 s + cargo); anchor-held by {corpus['verdict_held_by']}")
    elif corpus and corpus["kind"] == "raw_parse":
        out.append(f"  corpus  {fmt(corpus['total'])} files  [{corpus['denominator']}]")
        out.append(f"    parses ⚠️ RAW, no answer key ......... {fmt(corpus['passing']):>7s}  {pct(corpus['passing'], corpus['total'])}")
        out.append(f"    rejected ............................ {fmt(corpus['rejected']):>7s}  {pct(corpus['rejected'], corpus['total'])}")
        out.append(f"    timed out / crashed ................. {fmt(corpus['timeout']):>7s}  {fmt(corpus['crash'])}")
        out.append(f"    KNOWN DEFECTS ....................... NOT ADJUDICATED — no key says which "
                   f"of these SHOULD parse; owed by {corpus_why['owed_by']}")
        out.append(f"    => leg 3 is untouched by this number: a RAW rate is not a conformance rate")
    elif corpus:
        out.append(f"  corpus  {fmt(corpus['total'])} rows  [{corpus['denominator']}]")
        out.append(f"    passing (verdict matches the key) .... {fmt(corpus['passing']):>7s}  {pct(corpus['passing'], corpus['total'])}")
        out.append(f"    divergence, explained ............... {fmt(corpus['explained']):>7s}  {pct(corpus['explained'], corpus['total'])}")
        out.append(f"    KNOWN DEFECTS (unexplained) ......... {fmt(corpus['defects']):>7s}  {pct(corpus['defects'], corpus['total'])}")
        out.append(f"    routed to another lane .............. {fmt(corpus['routed']):>7s}  {pct(corpus['routed'], corpus['total'])}")
        out.append(f"    NO VERDICT (nothing is known) ....... {fmt(corpus['noverdict']):>7s}  {pct(corpus['noverdict'], corpus['total'])}   of which {fmt(corpus['dark'])} dark")
        out.append(f"    => adjudicated {fmt(corpus['adjudicated'])} / {fmt(corpus['total'])} = {pct(corpus['adjudicated'], corpus['total'])}")
    else:
        label = ("NO THIRD-PARTY CORPUS EXISTS"
                 if corpus_why["derived_class"] == "no_third_party_corpus" else "NOT MEASURED")
        out.append(f"  corpus  {label} [{corpus_why['derived_class']}]"
                   + (f" — owed by {corpus_why['owed_by']}" if corpus_why.get("owed_by") else "")
                   + (f"; last measured in {corpus_why['last_measured_in']}"
                      if corpus_why.get("last_measured_in") else ""))
    declared = d["declared"]
    if declared:
        out.append(f"  leg 3 reads (declared) . {fmt(declared['adjudicated'])} adjudicated + {fmt(declared['permanent'])} permanent "
                   f"(bounded) + {fmt(declared['burnable'])} AWAITING A VERDICT = {fmt(declared['rows'])} rows; "
                   f"unexplained {fmt(declared['unexplained'])}, bar {fmt(declared['bar'])}, stale {fmt(declared['stale'])}; "
                   f"full_pass={str(declared['full_pass']).lower()} (raw-lane bit {str(declared['full_pass_raw_lane']).lower()})")
        for l in declared["lanes"]:
            out.append(f"    {l['lane']:13s} {fmt(l['adjudicated']):>7s} + {fmt(l['permanent']):>4s} + {fmt(l['burnable']):>5s} = {fmt(l['rows']):>7s}   "
                       f"unexplained {l['unexplained']}  bar {l['bar']}  full_pass={str(l['full_pass']).lower()}")
    else:
        owed = schema["fields"]["leg3_declared_reading"].get("owed_by_when_absent", "DONE-BAR.3")
        out.append(f"  leg 3 reads (declared) . NOT MEASURED — no tracked leg-3 assertion declared; owed by {owed}")
    names = ", ".join(n for n, _f, _a, _b in family_open) or "(none)"
    out.append(f"  remaining work ......... {len(family_open)} open tree(s) own this family's work: {names}")
    out.append(f"                          + {cross_open} cross-cutting tree(s) that can gate any family")
    if disputed or unreadable_rows:
        out.append(f"                          ⚠️ {len(disputed)} status DISPUTED (index vs tree file), "
                   f"{len(unreadable_rows)} unreadable — all counted OPEN")
    return out


# ---------------------------------------------------------------------------------------------
# 5. The CROSS-FAMILY view — the whole roster on one page, from the same derive() ten times.
# ---------------------------------------------------------------------------------------------
# ⛔ WHY THIS EXISTS (DONE-BAR-VISIBILITY.5). Ten published per-family blocks answer "how far is
# THIS parser"; nothing answered "how far is the project", and a reader who wanted to compare two
# families had to open two books and hold both in their head. That is the director's remark —
# *"there seems to be no end in sight"* — in its cross-family form.
def cell(text):
    """Every cell published into a Markdown table passes through here.

    ⛔ A `|` inside a cell SILENTLY re-cuts the row: GFM discards the excess cells of an over-wide
    row without rendering, warning or leaving a gap, which is how 51 of 1 167 tracked files came to
    carry defective rows unnoticed (TABLE-ARITY-RATCHET, founding measurement LIVE-DOC-CONTAINMENT.8
    — the shipped bug ledger lost the consumer-impact column on 26 of 197 rows). Most cells here
    carry text from the schema and the register, so the day someone writes a pipe into a blurb this
    must REFUSE rather than publish a table that reads fine and says something else."""
    if "|" in text:
        refuse(f"a rendered table cell carries a `|`, which GFM would read as a column boundary and "
               f"the published row would silently lose its rightmost cells: {text!r}")
    return text


def render_cross(ds):
    """ds: one derive() result per family, in register order."""
    # ⭐ TWO ARMS OVER THE SAME PARTITION, and they are produced by different code paths: the
    # per-family lists come from filtering row_state by attribution inside derive(), the totals from
    # counting row_state by attribution once. A cross-family page whose totals disagree with the
    # rows above them is worse than no page, so they are held equal here rather than eyeballed.
    fam_total = sum(len(x["family_open"]) for x in ds)
    from_partition = sum(v for k, v in open_by_class.items() if k in register["families"])
    if fam_total != from_partition:
        refuse(f"the per-family open-tree lists sum to {fam_total} while the attribution partition "
               f"counts {from_partition} open trees attributed to a family. Two readings of one "
               f"index disagree — neither may be published until they are reconciled.")
    cross_cutting = open_by_class.get("cross-cutting", 0)
    if any(x["cross_open"] != cross_cutting for x in ds):
        refuse("the families do not agree on how many cross-cutting trees are open, though the "
               "count is a property of the index and not of any family")
    vocab = schema.get("tree_attribution", {}).get("vocabulary", {})
    for who in sorted(open_by_class):
        key = "<family>" if who in register["families"] else who
        if key not in vocab:
            refuse(f"the attribution bucket '{who}' has no entry in `tree_attribution.vocabulary` of "
                   f"{SCHEMA_PATH}, so its rows would be published with no statement of what the "
                   f"bucket means")

    met_all = [x for x in ds if not x["unmet"]]
    legs_met = sum(3 - len(x["unmet"]) for x in ds)
    with_census = [x for x in ds if x["corpus"]]
    out = []

    if MODE == "cross_markdown":
        out.append("<!-- BEGIN DERIVED: scripts/report_family_status.sh --cross-family --markdown -->")
        out.append("")
        out.append(f"**{len(ds)} families, one page — the distance behind the labels above.** Every "
                   f"figure here is derived by `scripts/report_family_status.sh` in the SAME pass "
                   f"that produces each family's own *Status and Distance to `Done`* page, so the "
                   f"two surfaces cannot disagree about a number; `FAMILY-STATUS-CURRENCY` "
                   f"(`scripts/check_family_status.sh`) re-derives and diffs both on every commit.")
        out.append("")
        out.append(f"**{len(met_all)} of {len(ds)} families meet the three-leg `Done` bar, and "
                   f"{legs_met} of the {3 * len(ds)} legs across the roster are met.**")
        out.append("")
        out.append("| Family | Claimed status | Legs met | 1 | 2 | 3 | Trees owning its remaining work |")
        out.append("|---|---|---:|:-:|:-:|:-:|---:|")
        for x in ds:
            fam = x["family"]
            link = f"[`{fam}`](../../{fam}_parser_book/src/status-and-distance.md)"
            out.append(f"| {link} | {cell(x['entry']['claimed_status'])} | "
                       f"{3 - len(x['unmet'])} of 3 | " +
                       " | ".join(MARK[x["legs"][n]["verdict"]] for n in (1, 2, 3)) +
                       f" | {len(x['family_open'])} |")
        out.append("")
        out.append(f"Leg 1 is the {LEGNAME[1]}, leg 2 is *{LEGNAME[2]}* — every covering gate green "
                   f"AND actually invoked by something — and leg 3 is an officially-recognized "
                   f"{LEGNAME[3]}. {MARK['MET']} met · {MARK['UNPROVEN']} unproven · "
                   f"{MARK['UNMET']} unmet. **Why** a leg reads as it does is on that family's own "
                   f"page, one click away; this table is the comparison, not the explanation.")
        out.append("")
        out.append(f"**Corpus.** {len(with_census)} of the {len(ds)} families {'has' if len(with_census) == 1 else 'have'} a corpus count at "
                   f"all. The rows below are that family's own denominator — there is no "
                   f"roster-wide corpus figure, and the `NOT MEASURED` rows are exactly what is "
                   f"missing, each naming the leaf that owes it.")
        out.append("")
        out.append("| Family | Corpus rows | Passing | Known defects | What the number rests on |")
        out.append("|---|---:|---:|---:|---|")
        for x in ds:
            c, why = x["corpus"], x["corpus_why"]
            if c:
                rests = (f"{c['denominator']}, counted by `{c['instrument']}` and held current by "
                         f"`{c['gate']}`")
                # ⛔ A RAW census has no `known defects` number and must not be given one. Zero in
                # that cell would read as *nothing is wrong* where the truth is *nobody has looked*,
                # and this is the one table where a reader compares families side by side — which is
                # exactly where an incomparable number does the most damage.
                if c["defects"] is None:
                    rests += (f". ⚠️ RAW parse outcomes: no answer key, so the pass share is NOT a "
                              f"conformance share and no defect count exists. Adjudication owed by "
                              f"`{why['owed_by']}`")
                    defect_cell = "⛔ n/a"
                    pass_cell = f"{fmt(c['passing'])} ({pct(c['passing'], c['total'])}) ⚠️ raw"
                else:
                    defect_cell = fmt(c["defects"])
                    pass_cell = f"{fmt(c['passing'])} ({pct(c['passing'], c['total'])})"
                row = (f"| `{x['family']}` | {fmt(c['total'])} | {pass_cell} | {defect_cell} | "
                       f"{cell(rests)} |")
            else:
                rests = why["blurb"] + "."
                if why.get("owed_by"):
                    rests += f" Owed by `{why['owed_by']}`."
                row = (f"| `{x['family']}` | ⛔ NOT MEASURED | ⛔ | ⛔ | {cell(rests)} |")
            out.append(row)
        out.append("")
        out.append("⛔ in a numeric cell means **NOT MEASURED**: there is no count. That is not the "
                   "same as a count of zero, and it is not the same as nothing being wrong — it is "
                   "the absence of an instrument, which is itself the work the named leaf owes.")
        out.append("")
        out.append(f"**What remains, across the whole index.** The task-tree index carries "
                   f"{len(row_state)} trees and **{n_open} are OPEN**. Every row is adjudicated to "
                   f"exactly one bucket — the producer refuses to render at all if a row carries no "
                   f"attribution or an attribution names no row — so the buckets below are a "
                   f"partition, and they sum to that total.")
        out.append("")
        out.append("| Where the open work sits | Open trees | What that bucket is |")
        out.append("|---|---:|---|")
        out.append(f"| Attributed to one family | {fam_total} | {cell(vocab['<family>'])} |")
        for who, n in sorted(((k, v) for k, v in open_by_class.items()
                              if k not in register["families"]), key=lambda kv: (-kv[1], kv[0])):
            out.append(f"| `{who}` | {n} | {cell(vocab[who])} |")
        out.append(f"| **Total** | **{n_open}** | of the {len(row_state)} trees the index carries |")
        out.append("")
        out.append(f"⚠️ **These counts are an UPPER BOUND, and the reason is measured.** A tree's "
                   f"status is prose in two places — the index row and the tree file's own metadata "
                   f"— and they DISAGREE on {n_disputed_rows} of the {len(row_state)} rows while "
                   f"{n_unreadable_rows} index rows carry no status word this reader recognises. "
                   f"Every one of those is counted OPEN, because over-reporting how much is left is "
                   f"the only safe direction for a page that says how much is left. Reconciling "
                   f"them is `DONE-BAR-VISIBILITY.8`; raising the item from a TREE to a LEAF is "
                   f"`DONE-BAR-VISIBILITY.7`.")
        out.append("")
        out.append("<!-- END DERIVED -->")
    else:
        out.append(f"CROSS-FAMILY STATUS — {len(ds)} families, one page   (every figure derived; "
                   f"nothing here is hand-written)")
        out.append(f"  {len(met_all)} of {len(ds)} families meet the three-leg `Done` bar; "
                   f"{legs_met} of {3 * len(ds)} legs met across the roster.")
        out.append("")
        out.append(f"  {'family':30s}{'claimed status':30s}{'corpus rows':>13s}{'passing':>22s}"
                   f"{'defects':>9s}{'trees':>7s}   legs 1/2/3")
        for x in ds:
            c = x["corpus"]
            rows_cell = fmt(c["total"]) if c else "NOT MEASURED"
            raw = bool(c) and c["defects"] is None
            pass_cell = (f"{fmt(c['passing'])} ({pct(c['passing'], c['total'])})"
                         + (" raw" if raw else "")) if c else "—"
            defect_cell = ("n/a" if raw else fmt(c["defects"])) if c else "—"
            out.append(f"  {x['family']:30s}{x['entry']['claimed_status']:30s}{rows_cell:>13s}"
                       f"{pass_cell:>22s}{defect_cell:>9s}{len(x['family_open']):>7d}   "
                       + " ".join(MARK[x["legs"][n]["verdict"]] for n in (1, 2, 3)))
        out.append("")
        out.append(f"  => open trees {n_open} of {len(row_state)}: {fam_total} owned by a family "
                   f"(column `trees`), " + ", ".join(
                       f"{n} {who}" for who, n in sorted(
                           ((k, v) for k, v in open_by_class.items() if k not in register["families"]),
                           key=lambda kv: (-kv[1], kv[0]))))
        out.append(f"  => {len(with_census)} of {len(ds)} families {'has' if len(with_census) == 1 else 'have'} a corpus count at all; each "
                   f"NOT MEASURED row names the leaf that owes one")
        out.append(f"  => an UPPER BOUND: {n_disputed_rows} tracker statuses DISPUTED and "
                   f"{n_unreadable_rows} unreadable, all counted OPEN (DONE-BAR-VISIBILITY.8)")
    return out


# --- dispatch -----------------------------------------------------------------------------------
if CROSS:
    print("\n".join(render_cross([derive(f) for f in sorted(register["families"])])))
elif MODE == "markdown":
    print("\n".join(render_markdown(derive(FAMILY))))
else:
    print("\n".join(render_text(derive(FAMILY))))
PY
}

# --- --check: derive, then DIFF ---------------------------------------------------------------
# ⛔⛔ THE FLAG AND ITS IMPLEMENTATION LIVE OR DIE TOGETHER. The sibling producer kept `--check` in
# its argument parser after a rewrite deleted the diff, and the check then exited 0 on every input
# — including a path that does not exist — for a whole commit, while three surfaces went on
# advertising the capability (GRAMMAR-CERT-STATUS.1b). The RED controls for this block are in the
# owning task leaf, and they are what makes this paragraph a claim rather than a hope.
# check_page <page> <fresh-block> — exit 0 in sync, 1 breached, 2 could not evaluate.
check_page() {
  RFS_FRESH="$2" python3 - "$1" <<'PYCHECK'
import difflib, os, pathlib, re, sys
page = sys.argv[1]
fresh = os.environ["RFS_FRESH"].strip()
# An EMPTY derivation must REFUSE, never compare: an empty string equals an empty block, so a
# broken producer would report the page in sync BY CONSTRUCTION.
if not fresh:
    print("report_family_status: REFUSED (2) - the fresh derivation is EMPTY, so there is nothing "
          "to compare against; comparing would pass by construction.", file=sys.stderr)
    sys.exit(2)
m = re.search(r"<!-- BEGIN DERIVED.*?<!-- END DERIVED -->", pathlib.Path(page).read_text(), re.S)
if not m:
    print(f"report_family_status: REFUSED (2) - {page} carries no DERIVED block", file=sys.stderr)
    sys.exit(2)
published = m.group(0).strip()
if published != fresh:
    print("report_family_status: FAILED (1) - the published status block is OUT OF SYNC with the "
          "tree.\n  Re-derive and republish it:\n"
          "    bash scripts/report_family_status.sh <family> --markdown", file=sys.stderr)
    print("\n".join(difflib.unified_diff(published.split("\n"), fresh.split("\n"),
                                         f"published:{page}", "fresh derivation", lineterm="")),
          file=sys.stderr)
    sys.exit(1)
print(f"report_family_status: OK (published block equals a fresh derivation, {len(fresh)} bytes)")
PYCHECK
}

# --- entry points -------------------------------------------------------------------------------
if [ "$CHECK_ALL" -eq 1 ]; then
  # ⛔ THE POPULATION IS THE REGISTER'S, NOT A LIST TYPED HERE. A family added to the register with
  # no published page must FAIL rather than be skipped — a check whose population is hand-listed
  # stops covering the thing it was written for the first time the roster grows.
  fams="$(python3 -c "import json;print(' '.join(sorted(json.load(open('rust/test_data/grammar_quality/done_bar_family_register_v0.json'))['families'])))")" || exit 2
  rc_all=0; n_ok=0
  for fam in $fams; do
    page="docs/${fam}_parser_book/src/status-and-distance.md"
    if [ ! -f "$page" ]; then
      echo "report_family_status: FAILED (1) - $fam is a registered family with NO published status page at $page" >&2
      rc_all=1; continue
    fi
    fresh="$(render "$fam" markdown)" || { rc_all=2; continue; }
    if out="$(check_page "$page" "$fresh" 2>&1)"; then
      n_ok=$((n_ok + 1))
    else
      rc="$?"; [ "$rc_all" -eq 2 ] || rc_all="$rc"
      printf '%s\n' "$out" >&2
    fi
  done
  # ⛔ AND THE CROSS-FAMILY VIEW IS CHECKED IN THE SAME PASS, not by a second invocation. It is the
  # eleventh published surface derived from these two audit runs; checking it separately would pay
  # for them again and, worse, would leave it possible to run the ten and believe the roster covered.
  if [ ! -f "$CROSS_PAGE" ]; then
    echo "report_family_status: FAILED (1) - no published cross-family view at $CROSS_PAGE" >&2
    rc_all=1
  else
    fresh="$(render '*' cross_markdown)" || rc_all=2
    if [ -n "${fresh:-}" ]; then
      if out="$(check_page "$CROSS_PAGE" "$fresh" 2>&1)"; then
        n_ok=$((n_ok + 1))
      else
        rc="$?"; [ "$rc_all" -eq 2 ] || rc_all="$rc"
        printf '%s\n' "$out" >&2
      fi
    fi
  fi
  [ "$rc_all" -eq 0 ] && echo "report_family_status: OK ($n_ok published blocks — the family pages and the cross-family view — equal a fresh derivation)"
  exit "$rc_all"
fi

RENDERED="$(render "${FAMILY:-*}" "$MODE")" || exit 2
if [ -n "$CHECK" ]; then
  check_page "$CHECK" "$RENDERED"
  exit $?
fi
printf '%s\n' "$RENDERED"
