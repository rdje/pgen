#!/usr/bin/env bash
# scripts/check_live_document_currency.sh
# LIVE-DOC-CURRENCY (PGEN-LIVE-MEANS-LIVE-0009, leaf LIVE-MEANS-LIVE.2): a live document must be
# currently TRUE, not merely bounded. Register:
#   rust/test_data/grammar_quality/live_document_currency_register_v0.json
# Tree: docs/tasks/LIVE-MEANS-LIVE.md. Exits NONZERO on any breach, 2 on a REFUSAL.
# Called by scripts/check_doctrines.sh via .githooks/pre-commit (E3) and CI (E4).
#
# ⭐ WHY THIS EXISTS — measured, not assumed:
#   LIVE_ACHIEVEMENT_STATUS.md reached 1 547 057 B of which 94.7 % was a dated changelog, while
#   README.md next door was capped on TWO axes and its cap rule routed status overflow INTO that
#   very file. Capping a file is only half a fix; the other half is checking where the overflow
#   lands. ⛔ And a byte cap is the WRONG half — docs/book/src/gate-flow.md is the LARGEST of the
#   three healthy overflow destinations and the healthiest of them. What was actually wanted is
#   "has this surface stopped being a status document", and that is instrument A.
#
# ⭐⭐ THE TWO INSTRUMENTS (ANVIL, adopted in LIVE-MEANS-LIVE.2 over a byte-cap design that was
#   REJECTED for needing three human-chosen numbers and a keyword list):
#     A — count DISTINCT dates in a surface, PAIRED WITH ITS DECLARED CHARTER. The instrument
#         classifies; the charter says which classification is permitted. Without the pairing it
#         fires on CHANGES.md forever, and CHANGES.md is correct.
#     B — a self-declared `Last updated:` that DISAGREES with the newest content date in the same
#         file. Self-refuting: the file's own two halves contradict each other, so it needs no
#         external baseline and no threshold at all.
#
# ⭐⭐⭐ WHY B REFUSES INSTEAD OF ENUMERATING SPELLINGS — this instrument has mis-measured the SAME
#   population THREE times, every miss SILENT IN THE PASSING DIRECTION, every one caught only
#   because a PRIOR PUBLISHED NUMBER existed to disagree with:
#     .4 said 10 — knew one spelling (`Last updated: 2026-05-14`), blind to the ~50 docs/tasks
#        trees written ``- Last updated: `2026-05-31` ``, and carried one phantom row from prose.
#     .5 said 16 — knew two spellings, blind to the docs/contracts CONTINUATION spelling
#        (`- Last updated:` with the date on the FOLLOWING line): 6 files, 2 of them self-refuting,
#        and both of those are PUBLISHED DOWNSTREAM CONTRACTS.
#     .2 says 18 — four shapes, fenced blocks and mid-line prose excluded.
#   ⛔ Enumerating spellings is the thing that failed twice. So this check does NOT count
#   spellings: it requires TOTAL CLASSIFICATION and REFUSES (exit 2) on any anchored
#   `Last updated` line it cannot classify. A fifth spelling appearing tomorrow is a loud refusal
#   rather than an absent row. (docs/decisions/feedback_instrument_needs_ground_truth.md)
#
# ⛔ A cap/ceiling is NEVER raised to land content. A `status` surface over the ceiling is DECLARED
#   as owned debt with a leaf that exists — and the entry is TWO-SIDED, so paying the debt without
#   removing the entry also fails.
#
# Contract compliance (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict; breaches explain on
# stderr with the exact file and both measured dates; DETERMINISTIC (no clock, no network, no
# randomness — every date compared comes out of the tree itself); NON-MUTATING; repo root resolved
# from BASH_SOURCE; reads the WORKING TREE rather than the git index, so it is never vacuous on a
# hosted push. Submodule-blind by construction: `git ls-files` without --recurse-submodules, per
# CI-PARITY-GATE-ROT.20a — submodules are READ-ONLY LINKED REPOS and are not ours to police.
#
# ⚠️ A MISSING TOOL MUST NOT BE A PASS (CI-PARITY-GATE-ROT.20a: an audit whose `rg` was absent
# silently PASSED). python3 is checked explicitly below and its absence REFUSES.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"

if ! command -v python3 >/dev/null 2>&1; then
  printf 'live-doc-currency: REFUSED — python3 not found. A check that cannot inspect its subject\n' >&2
  printf '                   must SAY SO, not return green.\n' >&2
  exit 2
fi

exec python3 - "$ROOT" <<'PYEOF'
import json, os, re, subprocess, sys

ROOT = sys.argv[1]
os.chdir(ROOT)

REGISTER = "rust/test_data/grammar_quality/live_document_currency_register_v0.json"
README_ENFORCER = "scripts/check_readme_stability.sh"
COMMIT_DOC = "COMMIT.md"

fail = 0
def note(msg):
    global fail
    sys.stderr.write("live-doc-currency: %s\n" % msg)
    fail = 1

def refuse(msg):
    sys.stderr.write("live-doc-currency: REFUSED — %s\n" % msg)
    sys.exit(2)

# --------------------------------------------------------------------------- the extractor
# One implementation, used for BOTH the real scan and the ground-truth controls. If the controls
# and the scan could diverge, the controls would prove nothing about the scan.
DATE = re.compile(r"20\d{2}-\d{2}-\d{2}")
FENCE = re.compile(r"^[ \t]*(?:```|~~~)")
# ⛔ COLUMN 0, deliberately — NOT `^[ \t]*`. Measured over all 61 declaring files: every real
# declaration sits at column 0, bare or behind a column-0 bullet. Allowing leading whitespace made
# the check REFUSE on its OWN task leaf, where a wrapped quotation of its OK line put
# `Last updated:, 18 self-refuting, ...` at the start of an indented continuation line. Indented
# text is quoted material, not the document's own declaration — the same reason fenced blocks are
# excluded. Pinned by the `excluded-indented-quotation` control below.
ANCHOR = re.compile(r"^(?:[-*][ \t]+)?(?:\*\*)?[Ll]ast[ \t]+updated\b")
SHAPE_BACKTICK = re.compile(r"[Ll]ast[ \t]+updated:?[ \t]*\*{0,2}[ \t]*`(20\d{2}-\d{2}-\d{2})`")
SHAPE_BARE = re.compile(r"[Ll]ast[ \t]+updated:?[ \t]*\*{0,2}[ \t]*(20\d{2}-\d{2}-\d{2})")
SHAPE_TEMPLATE = re.compile(r"[Ll]ast[ \t]+updated:?[ \t]*`?YYYY-MM-DD`?")
SHAPE_OPEN = re.compile(r"[Ll]ast[ \t]+updated:[ \t]*$")
CONTINUATION = re.compile(r"^[ \t]*[-*][ \t]*`?(20\d{2}-\d{2}-\d{2})`?[ \t]*$")

def scan(lines):
    """-> (declared_date|None, shape|None, newest_content_date|None, [(lineno, text), ...])

    Shapes are tried in a fixed order; anything anchored that matches none is returned as
    UNCLASSIFIED so the caller can REFUSE. Fenced blocks are excluded from BOTH the declaration
    scan (a fenced `Last updated:` is sample OUTPUT — measured at rust/docs/CLI_REFERENCE.md:161)
    and the content scan (a fenced sample date is illustrative, not content the file claims
    currency over). ⚠️ Instrument A deliberately does NOT share that exclusion — see below.
    """
    decl = shape = newest = None
    unclassified = []
    fenced = False
    awaiting = None            # (lineno, text) of an open `- Last updated:` continuation
    for i, raw in enumerate(lines, start=1):
        line = raw.rstrip("\n")
        if FENCE.match(line):
            fenced = not fenced
            continue
        if awaiting is not None:
            m = CONTINUATION.match(line)
            if m:
                if decl is None:
                    decl, shape = m.group(1), "continuation"
                awaiting = None
                continue
            unclassified.append(awaiting)
            awaiting = None
        if fenced:
            continue
        for m in DATE.finditer(line):
            d = m.group(0)
            if newest is None or d > newest:
                newest = d
        if not ANCHOR.match(line):
            continue
        m = SHAPE_BACKTICK.search(line)
        if m:
            if decl is None:
                decl, shape = m.group(1), "backtick"
            continue
        m = SHAPE_BARE.search(line)
        if m:
            if decl is None:
                decl, shape = m.group(1), "bare"
            continue
        if SHAPE_TEMPLATE.search(line):
            if decl is None:
                decl, shape = None, "template"
            continue
        if SHAPE_OPEN.search(line):
            awaiting = (i, line)
            continue
        unclassified.append((i, line))
    if awaiting is not None:
        unclassified.append(awaiting)
    return decl, shape, newest, unclassified

# --------------------------------------------------------------------------- ground-truth controls
# ⭐ An instrument with no ground truth is a confident guess, and THIS one has been wrong three
# times. Every run pins fixtures with known answers FIRST and refuses rather than publishing a
# number if any of them misses. The fixtures are in-memory: no temp files, so nothing is written
# outside the repository and the controls cannot be defeated by a dirty filesystem.
CONTROLS = [
    ("positive-bare",
     ["# Doc", "Last updated: 2026-01-01", "", "Body mentioning 2026-06-30."],
     {"decl": "2026-01-01", "shape": "bare", "newest": "2026-06-30", "unclassified": 0}),
    ("negative-clean",
     ["# Doc", "Last updated: 2026-06-30", "", "Body mentioning 2026-01-01."],
     {"decl": "2026-06-30", "shape": "bare", "newest": "2026-06-30", "unclassified": 0}),
    ("shape-backtick",
     ["# Tree", "", "- Last updated: `2026-05-31`", "text 2026-07-04"],
     {"decl": "2026-05-31", "shape": "backtick", "newest": "2026-07-04", "unclassified": 0}),
    ("shape-continuation",
     ["# Contract", "- Last updated:", "  - `2026-07-21`", "- Version 2026-07-25"],
     {"decl": "2026-07-21", "shape": "continuation", "newest": "2026-07-25", "unclassified": 0}),
    ("shape-template",
     ["# Template", "- Last updated: `YYYY-MM-DD`"],
     {"decl": None, "shape": "template", "newest": None, "unclassified": 0}),
    ("excluded-fenced",
     ["# Doc", "```", "Last updated:          2025-09-26 18:25:21", "```", "body 2026-03-03"],
     {"decl": None, "shape": None, "newest": "2026-03-03", "unclassified": 0}),
    ("excluded-prose",
     ["# Doc", "its 3 matches are prose (`Last updated: 2026-05-25`) narrating another file",
      "body 2026-08-08"],
     {"decl": None, "shape": None, "newest": "2026-08-08", "unclassified": 0}),
    ("excluded-indented-quotation",
     ["# Doc", "      the gate printed `61/913 tracked .md declare",
      "      Last updated:, 18 self-refuting` and exited 0", "body 2026-04-04"],
     {"decl": None, "shape": None, "newest": "2026-04-04", "unclassified": 0}),
    ("refusal-path-live",
     ["# Doc", "Last updated: sometime in the spring", "body 2026-04-04"],
     {"decl": None, "shape": None, "newest": "2026-04-04", "unclassified": 1}),
]
for name, fixture, want in CONTROLS:
    decl, shape, newest, unc = scan(fixture)
    got = {"decl": decl, "shape": shape, "newest": newest, "unclassified": len(unc)}
    if got != want:
        refuse("ground-truth control %r MISSED.\n"
               "                   expected %r\n"
               "                   measured %r\n"
               "                   The extractor changed meaning. Fix the extractor or re-pin the\n"
               "                   control DELIBERATELY — never publish a population from an\n"
               "                   instrument that cannot reproduce a known answer."
               % (name, want, got))

# --------------------------------------------------------------------------- load the register
if not os.path.isfile(REGISTER):
    refuse("%s is missing; the charters below would be unreviewable assertions." % REGISTER)
try:
    with open(REGISTER, encoding="utf-8") as fh:
        reg = json.load(fh)
except (OSError, ValueError) as exc:
    refuse("%s is unreadable: %s" % (REGISTER, exc))

try:
    inst_a = reg["instrument_a"]
    CEILING = int(inst_a["status_distinct_date_ceiling"])
    SURFACES = inst_a["surfaces"]
    OUT_OF_CHARTER = {k: v for k, v in inst_a["out_of_charter"].items() if not k.startswith("_")}
    DEBT = reg["instrument_b"]["self_refuting_debt"]["files"]
except (KeyError, TypeError, ValueError) as exc:
    refuse("%s does not carry the expected shape (%s)." % (REGISTER, exc))

def tracked(pattern):
    out = subprocess.run(["git", "ls-files", "--", pattern],
                         capture_output=True, text=True, check=False)
    return [p for p in out.stdout.splitlines() if p]

md_files = tracked("*.md")
if not md_files:
    refuse("`git ls-files -- '*.md'` returned NOTHING. A check that cannot enumerate its subject "
           "must say so, not pass over an empty set.")

# --------------------------------------------------------------------------- instrument B
self_refuting = []
declaring = 0
unclassified_all = []
for path in sorted(md_files):
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            lines = fh.readlines()
    except OSError:
        continue
    decl, _shape, newest, unc = scan(lines)
    unclassified_all.extend((path, ln, txt) for ln, txt in unc)
    if decl is None:
        continue
    declaring += 1
    if newest is not None and newest > decl:
        self_refuting.append((path, decl, newest))

if unclassified_all:
    sys.stderr.write(
        "live-doc-currency: REFUSED — %d anchored `Last updated` line(s) matched NO pinned shape.\n"
        % len(unclassified_all))
    for path, ln, txt in unclassified_all[:10]:
        sys.stderr.write("                   %s:%d: %s\n" % (path, ln, txt.strip()[:110]))
    sys.stderr.write(
        "                   ⛔ This is the refusal that ends a three-measurement error series: the\n"
        "                   same instrument reported 10, then 16, then 18 for one population, and\n"
        "                   every miss was an ABSENT ROW rather than a reported miss. A new\n"
        "                   declaration spelling must be PINNED in %s (instrument_b.pinned_shapes)\n"
        "                   and given a ground-truth control, not silently skipped.\n" % REGISTER)
    sys.exit(2)

refuting_set = {p for p, _, _ in self_refuting}
debt_set = set(DEBT)
new_rot = sorted(refuting_set - debt_set)
paid = sorted(debt_set - refuting_set)

for path in new_rot:
    decl, newest = next((d, n) for p, d, n in self_refuting if p == path)
    note("%s SELF-REFUTES its own `Last updated:` — declares %s, its own newest content date is %s."
         % (path, decl, newest))
if new_rot:
    sys.stderr.write(
        "                   Fix the file, do not add it to the register: either update the\n"
        "                   declaration BECAUSE the content really moved, or DELETE a\n"
        "                   self-declaration nothing maintains. Registering it is only correct for\n"
        "                   pre-existing debt LIVE-MEANS-LIVE.4 already owns.\n")
for path in paid:
    note("%s is listed as self-refuting debt but no longer self-refutes — remove it from %s "
         "(instrument_b.self_refuting_debt.files). A debt list that keeps paid entries stops "
         "being a ratchet." % (path, REGISTER))

# --------------------------------------------------------------------------- instrument A
# ⚠️ DELIBERATE ASYMMETRY with instrument B, stated rather than discovered: A counts EVERY distinct
# date in the file, fenced blocks included, because A asks "how many dates does this surface
# carry" — a dated sample is still carried. B excludes fenced blocks because it asks whether the
# file's own DECLARATION contradicts content the file claims currency over, and a sample is not
# that. A also reproduces .3's published table exactly, which is its own ground truth.
def distinct_dates(path):
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            return len(set(DATE.findall(fh.read())))
    except OSError:
        return None

VALID_CHARTERS = {"status", "log", "index"}
measured = {}
for path, spec in sorted(SURFACES.items()):
    charter = (spec or {}).get("charter")
    why = ((spec or {}).get("_why") or "").strip()
    if charter not in VALID_CHARTERS:
        note("%s declares charter %r, which is not one of %s."
             % (path, charter, sorted(VALID_CHARTERS)))
        continue
    if not why:
        note("%s declares charter %r with no `_why`. An exemption nobody justified is an "
             "exemption nobody reviewed." % (path, charter))
    if not os.path.isfile(path):
        note("%s is a registered surface but does not exist. Remove the row or restore the file — "
             "a register naming absent surfaces is how a watched destination stops being watched."
             % path)
        continue
    n = distinct_dates(path)
    measured[path] = (charter, n)
    entry = OUT_OF_CHARTER.get(path)
    if charter != "status":
        if entry is not None:
            note("%s is chartered %r yet carries an out_of_charter entry. Only `status` surfaces "
                 "are subject to the ceiling." % (path, charter))
        continue
    over = n > CEILING
    if over and entry is None:
        note("%s carries %d distinct dates (> the `status` ceiling %d) — it has stopped being a "
             "status view. Either route the dated history to its canonical home, or declare it in "
             "%s (instrument_a.out_of_charter) with an owner leaf. ⛔ Do NOT raise the ceiling."
             % (path, n, CEILING, REGISTER))
    elif not over and entry is not None:
        note("%s carries %d distinct dates, back UNDER the ceiling %d, but still holds an "
             "out_of_charter entry. The debt is paid — remove the entry (this is the ratchet's "
             "lower side)." % (path, n, CEILING))
    elif over and entry is not None:
        leaf = (entry or {}).get("owner_leaf", "")
        if "." not in leaf:
            note("%s's out_of_charter entry names owner_leaf %r, which is not a `TREE.leaf` id."
                 % (path, leaf))
        else:
            tree, suffix = leaf.split(".", 1)
            tree_file = os.path.join("docs", "tasks", tree + ".md")
            if not os.path.isfile(tree_file):
                note("%s's out_of_charter entry names owner_leaf %r but %s does not exist — a debt "
                     "with no owner is a permanent exemption." % (path, leaf, tree_file))
            else:
                with open(tree_file, encoding="utf-8", errors="replace") as fh:
                    body = fh.read()
                if not re.search(r"^#{2,4}\s+`\.%s`" % re.escape(suffix), body, re.M):
                    note("%s's out_of_charter entry names owner_leaf %r but %s carries no `.%s` "
                         "leaf heading." % (path, leaf, tree_file, suffix))

# --------------------------------------------------------------------------- route closure
# ⭐ The edge that carried the rot out of README.md was a HINT STRING inside an error message
# (check_readme_stability.sh:83, retired in .1c2). A hand-authored route registry cannot see such
# an edge, so the edges are DERIVED from the enforcers' own output text.
MD_TOKEN = re.compile(r"[A-Za-z0-9_./*-]+\.md")
edges = {}

def add_edges(source, text):
    for tok in MD_TOKEN.findall(text):
        if "*" in tok:
            for real in tracked(tok):
                edges.setdefault(real, set()).add(source)
            continue
        edges.setdefault(tok, set()).add(source)

if not os.path.isfile(README_ENFORCER):
    refuse("%s is missing; route closure cannot be derived." % README_ENFORCER)
with open(README_ENFORCER, encoding="utf-8") as fh:
    src = fh.read()
hint = re.search(r"^routing_hint\(\).*?^HINT$", src, re.S | re.M)
if hint is None:
    refuse("could not locate %s's routing_hint() heredoc. The route edges are DERIVED from it; an "
           "extraction that silently yields nothing would report full closure over an empty set."
           % README_ENFORCER)
add_edges(README_ENFORCER, hint.group(0))

if not os.path.isfile(COMMIT_DOC):
    refuse("%s is missing; the commit workflow's own live-document list cannot be read." % COMMIT_DOC)
with open(COMMIT_DOC, encoding="utf-8") as fh:
    commit_src = fh.read()
files_involved = re.search(r"^## Files Involved$.*?^## ", commit_src, re.S | re.M)
if files_involved is None:
    refuse("could not locate %s's `## Files Involved` section — same reason as above." % COMMIT_DOC)
add_edges(COMMIT_DOC, files_involved.group(0))

if not edges:
    refuse("route-closure derivation produced ZERO edges. Full closure over an empty set is not a "
           "pass.")

tracked_md = set(md_files)
informational = []
for dest in sorted(edges):
    if dest not in tracked_md:
        informational.append((dest, sorted(edges[dest])))
        continue
    if dest not in SURFACES:
        note("%s names %s as a destination, but it is not a registered surface in %s. A guard that "
             "names a destination is DEFINING A ROUTE, and an unwatched route is exactly how "
             "LIVE_ACHIEVEMENT_STATUS.md reached 1 547 057 B while the file routing INTO it was "
             "capped on two axes."
             % (", ".join(sorted(edges[dest])), dest, REGISTER))

# --------------------------------------------------------------------------- report
if fail == 0:
    status_surfaces = sum(1 for c, _ in measured.values() if c == "status")
    # ⛔ B REPORTS DORMANCY RATHER THAN A VACUOUS PASS (LIVE-MEANS-LIVE.4b, consequence 4).
    # .4a deleted the declaration from every file that carried it, so B's population is now
    # empty by construction. "0/914 declare, 0 self-refuting" read as an ordinary OK line is
    # indistinguishable from an instrument that has silently stopped seeing its subject — which
    # is precisely the failure mode this whole doctrine was built to catch. So say DORMANT, say
    # WHY, and say the tripwire is still armed. The 9 ground-truth controls above ran before
    # this line printed, so the instrument is proven live even with nothing to measure.
    if declaring == 0:
        b_report = ("instrument B: DORMANT — 0/%d tracked .md declare `Last updated:` (deleted "
                    "repo-wide by LIVE-MEANS-LIVE.4a; `git log -1 --date=short` is the "
                    "non-rotting source). Still WIRED as a re-introduction tripwire and its %d "
                    "ground-truth controls ran"
                    % (len(md_files), len(CONTROLS)))
    else:
        b_report = ("instrument B: %d/%d tracked .md declare `Last updated:`, %d self-refuting, "
                    "all owned by LIVE-MEANS-LIVE.4"
                    % (declaring, len(md_files), len(self_refuting)))
    sys.stderr.write(
        "live-doc-currency: OK — %d surfaces chartered (%d `status`, ceiling %d distinct dates); "
        "%s; %d derived route edges, all watched.\n"
        % (len(measured), status_surfaces, CEILING, b_report, len(edges)))
    for dest, srcs in informational:
        sys.stderr.write(
            "                   note: %s names %s, which is not a tracked file (normally prose "
            "about a deleted surface).\n" % (", ".join(srcs), dest))
sys.exit(fail)
PYEOF
