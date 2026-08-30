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
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2

FAMILY=""; MODE=text; CHECK=""
while [ $# -gt 0 ]; do
  case "$1" in
    --markdown) MODE=markdown ;;
    --check) CHECK="${2:-}"; MODE=markdown; shift ;;
    -*) echo "report_family_status: unknown arg '$1'" >&2; exit 2 ;;
    *) [ -z "$FAMILY" ] || { echo "report_family_status: two families given ('$FAMILY', '$1')" >&2; exit 2; }
       FAMILY="$1" ;;
  esac; shift
done
[ -n "$FAMILY" ] || { echo "report_family_status: REFUSED (2) - name the family, e.g. 'systemverilog'" >&2; exit 2; }

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

RENDERED="$(python3 - "$FAMILY" "$MODE" "$WORK" <<'PY'
import json, os, re, subprocess, sys

FAMILY, MODE, WORK = sys.argv[1], sys.argv[2], sys.argv[3]
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
if FAMILY not in register["families"]:
    refuse(f"'{FAMILY}' is not a parser family. The roster is derived, not typed — it is the "
           f"`families` map of {REGISTER_PATH}: {', '.join(sorted(register['families']))}")
entry = register["families"][FAMILY]

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
    corpus = dict(total=total, passing=passing, explained=explained, defects=defects,
                  routed=routed, noverdict=noverdict, adjudicated=adjudicated, dark=dark,
                  tuple=derived_tuple, denominator=census["denominator"],
                  instrument=census["instrument"], gate=gate)

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
    """OPEN / CLOSED / UNREADABLE from a prose status cell, biased OPEN.

    ⛔ THE BIAS IS THE DESIGN. Both arms are prose written for humans, so a cell legitimately says
    `.13e.10` DONE about a LEAF while the TREE is active. Reading any open word as OPEN
    over-reports remaining work; reading it the other way DELETES work from the list. Only one of
    those two errors is safe in a document whose whole purpose is to say how much is left."""
    toks = {t.lower() for t in re.findall(r"`([A-Za-z]+)`", cell)}
    if toks & set(OPEN_WORDS):
        return "OPEN"
    if toks & set(CLOSED_WORDS):
        return "CLOSED"
    return "UNREADABLE"


def rw_self_check():
    cases = [
        ("`active` (created 2026-08-30)", "OPEN"),
        ("`done` (all leaves closed)", "CLOSED"),
        # a tree whose cell shouts DONE about a LEAF while the tree itself is active
        ("✅✅ `.13e.10`(c4) RESIDUE CLOSED — tree stays `active`", "OPEN"),
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

family_open, disputed, unreadable_rows, cross_open = [], [], [], 0
for name, cell, fname in rows:
    who = attrib[name]
    a = polarity(cell)
    b = meta_status(fname)
    state = a
    if b and a != b and "UNREADABLE" not in (a, b):
        state = "OPEN"
        if who in (FAMILY, "cross-cutting"):
            disputed.append((name, a, b))
    elif a == "UNREADABLE":
        state = "OPEN"
        if who == FAMILY:
            unreadable_rows.append(name)
    if state != "OPEN":
        continue
    if who == FAMILY:
        family_open.append((name, fname, a, b))
    elif who == "cross-cutting":
        cross_open += 1
family_open.sort()

# ---------------------------------------------------------------------------------------------
# 4. Render — and hold the rendered field set EQUAL to the schema's, in both directions.
# ---------------------------------------------------------------------------------------------
owners = schema.get("not_measured_owners", {})


def pct(n, d):
    return "—" if not d else f"{100.0 * n / d:.1f} %"


def fmt(n):
    return f"{n:,}".replace(",", " ")


rendered = {"claimed_status", "language_owner", "bar_leg_1", "bar_leg_2", "bar_leg_3",
            "corpus_rows", "corpus_passing", "known_defects", "remaining_work"}
if MODE != "markdown":
    rendered.add("local_gate_state")
declared = {k for k, v in schema["fields"].items()
            if ("book" in v["surfaces"] or MODE != "markdown")}
if rendered != declared:
    refuse(f"the rendered field set and the schema disagree: only-rendered={sorted(rendered - declared)}, "
           f"only-declared={sorted(declared - rendered)}. Every field is declared in {SCHEMA_PATH} "
           f"with the producer that answers it; a field rendered without a declared producer is the "
           f"hand-written status this tree exists to remove.")

LEGNAME = {1: "stimuli-generator proof", 2: "gates green and invoked", 3: "external corpus passing"}
MARK = {"MET": "✅", "UNMET": "⛔", "UNPROVEN": "⚠️"}
unmet = [n for n in (1, 2, 3) if legs[n]["verdict"] != "MET"]
out = []

if MODE == "markdown":
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
    if corpus:
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
        out.append(f"**Corpus.** ⛔ **NOT MEASURED** — no corpus census is wired for this family. "
                   f"Owed by `{owners.get('corpus_rows', 'DONE-BAR-VISIBILITY.3')}`. This is an "
                   f"unmeasured leg, not a zero and not a pass.")
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
else:
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
    if corpus:
        out.append(f"  corpus  {fmt(corpus['total'])} rows  [{corpus['denominator']}]")
        out.append(f"    passing (verdict matches the key) .... {fmt(corpus['passing']):>7s}  {pct(corpus['passing'], corpus['total'])}")
        out.append(f"    divergence, explained ............... {fmt(corpus['explained']):>7s}  {pct(corpus['explained'], corpus['total'])}")
        out.append(f"    KNOWN DEFECTS (unexplained) ......... {fmt(corpus['defects']):>7s}  {pct(corpus['defects'], corpus['total'])}")
        out.append(f"    routed to another lane .............. {fmt(corpus['routed']):>7s}  {pct(corpus['routed'], corpus['total'])}")
        out.append(f"    NO VERDICT (nothing is known) ....... {fmt(corpus['noverdict']):>7s}  {pct(corpus['noverdict'], corpus['total'])}   of which {fmt(corpus['dark'])} dark")
        out.append(f"    => adjudicated {fmt(corpus['adjudicated'])} / {fmt(corpus['total'])} = {pct(corpus['adjudicated'], corpus['total'])}")
    else:
        out.append(f"  corpus  NOT MEASURED — no census is wired; owed by "
                   f"{owners.get('corpus_rows', 'DONE-BAR-VISIBILITY.3')}")
    names = ", ".join(n for n, _f, _a, _b in family_open) or "(none)"
    out.append(f"  remaining work ......... {len(family_open)} open tree(s) own this family's work: {names}")
    out.append(f"                          + {cross_open} cross-cutting tree(s) that can gate any family")
    if disputed or unreadable_rows:
        out.append(f"                          ⚠️ {len(disputed)} status DISPUTED (index vs tree file), "
                   f"{len(unreadable_rows)} unreadable — all counted OPEN")

print("\n".join(out))
PY
)" || exit 2

# --- --check: derive, then DIFF ---------------------------------------------------------------
# ⛔⛔ THE FLAG AND ITS IMPLEMENTATION LIVE OR DIE TOGETHER. The sibling producer kept `--check` in
# its argument parser after a rewrite deleted the diff, and the check then exited 0 on every input
# — including a path that does not exist — for a whole commit, while three surfaces went on
# advertising the capability (GRAMMAR-CERT-STATUS.1b). The RED controls for this block are in the
# owning task leaf, and they are what makes this paragraph a claim rather than a hope.
if [ -n "$CHECK" ]; then
  RFS_FRESH="$RENDERED" python3 - "$CHECK" <<'PYCHECK'
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
  exit $?
fi
printf '%s\n' "$RENDERED"
