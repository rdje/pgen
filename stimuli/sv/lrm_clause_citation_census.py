#!/usr/bin/env python3
"""LRM clause-citation census (SV-CORPUS-GRAD.13q) — doctrine `LRM-CLAUSE-CITATION`.

Does a cited IEEE clause number actually contain the LRM line it is cited with?

⛔ THE DEFECT THIS CLOSES. A conformance claim in this repository is four measurements wearing one
sentence: the QUOTED TEXT, the LRM FILE, the LINE NUMBER and the CLAUSE NUMBER. The first three are
checkable by anyone with `grep`; the fourth was checkable by nobody. `SV-CORPUS-GRAD.13p` located
the gate-delay cardinality sentence correctly at 1800-2017 `section-28-…md:396` and 1364-2005
`section-7-…md:376`, quoted it correctly, and labelled it §28.16 / §7.14 — a real clause ("Gate and
net delays") whose number was sitting four lines from the located text in §28.3.3's own
cross-reference. Seven cited lines, seven wrong clause labels, every quotation right, shipped to a
release ledger and a downstream contract.
  -> docs/knowledge/a-number-read-out-of-context-carries-a-frame-you-did-not-measure.md

⛔ IT BINDS STRUCTURALLY OR IT DECLINES TO JUDGE. A line reference is judged only when its file is
pinned by one of:

  F1  an explicit `…/section-….md:N` path            (its directory also fixes the edition)
  F2  `§X.Y:N`, or ``§X.Y (`:N`)`` with nothing but punctuation between — the line belongs to the
      clause itself, so the file is that clause's own section file
  F3  a bare `` `:N` `` continuing an F1/F2 inside a bounded span

...and only when it is IN THE CLAUSE'S OWN BRACKET SCOPE (see `in_clause_scope`) and the resolved
file CAN ANSWER FOR THE CITED CLAUSE (see `addressable_for`). Everything else is COUNTED and printed —
`no_line_ref`, `no_edition`, `annex_ref`, `unaddressable_section`, `unresolved_file` — never
silently dropped (docs/decisions/feedback_enumerating_instrument_must_refuse.md). The size of what
it declines to judge is printed beside the verdict on purpose: a census that gets cleaner because
it stopped looking is the failure mode that looks most like success, and this instrument caught its
own author committing it (judged rows fell 35 -> 17 mid-fix while the report stayed green).

⭐ THERE IS NO ALLOWANCE TABLE, AND THAT IS THE DESIGN. Two citation shapes legitimately break the
enclosure premise; the first cut carried them as seven hand-written exceptions keyed by line
numbers that move. Both are recognised STRUCTURALLY instead — `in_clause_scope` for the
defined-here/absent-there sentence, `addressable_for` for a file that carries no heading of the
cited clause's major. A gate whose green depends on a hand-maintained exception list is not a gate,
it is a promise. ⛔ The FILENAME'S NUMBER IS READ NOWHERE in this module (SV-CORPUS-GRAD.13u); an
earlier form tested a file against its own filename and its published count was withdrawn.

Usage:
  python3 stimuli/sv/lrm_clause_citation_census.py              # census
  python3 stimuli/sv/lrm_clause_citation_census.py --self-test  # ground truth + RED controls

Exit: 0 clean · 1 at least one disagreement · 7 ground truth failed (MISCALIBRATED).
"""

import argparse
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(subprocess.run(["git", "rev-parse", "--show-toplevel"],
                           capture_output=True, text=True, check=True).stdout.strip())

# The three vendored LRMs (docs/decisions/feedback_consult_the_lrms_rather_than_asking.md).
EDITION_DIR = {
    "1800-2017": "docs/systemverilog/2017/md",
    "1800-2023": "docs/systemverilog/2023/md",
    "1364-2005": "docs/verilog/2005/md",
}
DIR_EDITION = {v: k for k, v in EDITION_DIR.items()}

# The LRM corpora are the ORACLE, never the subject; the vendored suites are upstream text.
SKIP_PREFIX = ("docs/systemverilog/", "docs/verilog/", "stimuli/sv/subs/", "stimuli/sv/uvm/")
SKIP_INFIX = ("-html/",)

# ⛔ `^#{1,6} +N(.N)*[ .]`, never `^### ` — a heading census is only as good as its heading grammar
# (docs/knowledge/a-heading-census-is-only-as-good-as-the-heading-grammar.md).
HEADING = re.compile(r"^#{1,6} +(\d+(?:\.\d+)*)[ .]")
ANCHOR = re.compile(r"§(\d+(?:\.\d+)+)")
EDITION = re.compile(r"1800-2017|1800-2023|1364-2005")
F2 = re.compile(r"""[\s(\[,-]*`?:(\d{1,6})`?\b""")
F1 = re.compile(
    r"(?:(docs/(?:systemverilog/(?:2017|2023)|verilog/2005)/md)/)?"
    r"(section-[A-Za-z0-9_.-]+\.md):(\d{1,6})"
)
F3 = re.compile(r"`:(\d{1,6})`")

BACK_EDITION = 140   # chars behind an anchor searched for the edition token
SPAN_MAX = 220       # chars ahead of an anchor that can still belong to this citation
CHAIN_MAX = 140      # chars an F3 continuation may sit behind its F1/F2 anchor

_idx: dict = {}
_head: dict = {}
_modal: dict = {}


def section_index(edition):
    if edition not in _idx:
        by_major = {}
        for p in sorted((ROOT / EDITION_DIR[edition]).glob("section-*.md")):
            m = re.match(r"section-(\d+)-", p.name)
            if m:
                by_major.setdefault(int(m.group(1)), []).append(p)
        _idx[edition] = by_major
    return _idx[edition]


def headings(path):
    key = str(path)
    if key not in _head:
        _head[key] = [
            (i, m.group(1), t.strip())
            for i, t in enumerate(path.read_text(errors="replace").splitlines(), 1)
            if (m := HEADING.match(t))
        ]
    return _head[key]


def majors_in(path):
    """The set of clause majors this file actually carries headings for."""
    key = str(path)
    if key not in _modal:
        _modal[key] = {c.split(".")[0] for _, c, _ in headings(path)}
    return _modal[key]


def addressable_for(path, major):
    """Can a citation of clause `major` be CHECKED against this file at all?

    ⛔ SV-CORPUS-GRAD.13u — THE FIRST CUT ASKED THE WRONG QUESTION AND ITS ANSWER WAS PUBLISHED.
    `.13q`(c) tested a FILE against its own filename via the modal heading major, and reported
    *"17 of 124 vendored section files carry a filename their own headings refute."* That claim is
    FALSE and is WITHDRAWN. The modal major measures *which clause has the most headings*, not
    *whether the filename lies*, and the two differ for a file that legitimately SPANS clauses:
    `section-15-interprocess-synchronization-and-communication.md` opens at **15.1**, runs to
    **16.18**, and carries **26** clause-15 headings — it is CORRECTLY NAMED for the clause it
    opens, and the real defect is a MISSING SPLIT. `section-29-user-defined-primitives.md` is
    29.1 -> 29.10 with 16 own headings and is not defective at all; only stray noise headings
    dragged its modal to `0`. The population was never re-derived member by member
    (docs/knowledge/an-x-is-checked-by-nothing-claim-is-a-census-claim.md, the SET failure).

    ⭐ THE QUESTION THIS CENSUS ACTUALLY NEEDS is per-(file, CITED CLAUSE), never per-file: *if I
    resolve clause X to this file, is there anything here to check against?* A file carrying at
    least one heading of X's major can answer; one carrying none cannot, and its citations are
    COUNTED (`unaddressable_section`) rather than judged. That is parameter-free, needs no
    filename claim, and is strictly more precise — measured, it both REMOVES false positives and
    STOPS HIDING real ones the file-level test bucketed away unexamined.
    """
    return major in majors_in(path)


def enclosing(path, line, major):
    """Innermost heading at or before `line` WITHIN THE CITED CLAUSE'S OWN MAJOR.

    ⛔ A heading of a different major cannot be a subheading of the cited clause, so it is not a
    candidate. `docs/verilog/2005/md/section-7-gate-and-switch-level-modeling.md` carries four
    strays — `## 2 delays` and `## 3 delays` at `:2325`/`:2327`/`:2435`/`:2437`, fragments of
    §7.14's own delay-cardinality prose that the PDF->md split promoted to headings — and reading
    one as the enclosing clause reports every citation past it as wrong: line 2328 reads clause
    `3` unfiltered and §7.14 filtered.

    ⛔ SV-CORPUS-GRAD.13u: the filter keys on the CITED major, never on the file's modal major.
    The modal form was latently wrong for a file that spans clauses — in `section-15-…md` (modal
    16) it would have skipped all 26 clause-15 headings and judged a §15.x citation against 16.x.
    That was masked only because the file-level addressability test bucketed the file out first,
    which is a mask, not a defence.
    """
    best = None
    for i, clause, text in headings(path):
        if i > line:
            break
        if clause.split(".")[0] != major:
            continue
        best = (clause, i, text)
    return best


def in_clause_scope(span, offset):
    """Is `offset` still inside the bracket group the clause itself sits in?

    ⛔ SV-CORPUS-GRAD.13q(c) — THE SHAPE THAT BREAKS THE ENCLOSURE PREMISE. A sentence like
    *"`interface_class_declaration` is defined (§8.26.1 / A.1.2) but NOT referenced as an
    alternative in `class_item` (`section-8-classes.md:95`)"* names a clause and a line that are
    DELIBERATELY different places: the clause says where the production is DEFINED, the line says
    where it is ABSENT. Both parentheticals are correct and enclosure is not the question they ask.

    The signal is structural, never semantic: the clause's own group CLOSES before the ref's group
    opens, so a ref reached only after LEAVING the clause's scope belongs to a different citation.
    A relative depth going negative is that departure. Measured: this unbinds 5 rows of that shape
    and costs ZERO reach — `agree` is identical with and without it.
    """
    depth = 0
    for ch in span[:offset]:
        if ch in "([":
            depth += 1
        elif ch in ")]":
            depth -= 1
            if depth < 0:
                return False
    return True


def judge(path, cited, line):
    """`agree` when the innermost heading enclosing `line` IS `cited` or a descendant of it."""
    enc = enclosing(path, line, cited.split(".")[0])
    if enc is None:
        return "no_heading", None
    clause = enc[0]
    return ("agree" if clause == cited or clause.startswith(cited + ".") else "disagree"), enc


# ── ground truth, re-run on every invocation ──────────────────────────────────────────────────
# An instrument with no ground truth is a confident guess
# (docs/decisions/feedback_instrument_needs_ground_truth.md). Every rule this census depends on
# has a pair here, INCLUDING the defect that founded it, so a regression that stops seeing any of
# them exits 7 rather than reporting a clean tree.
GT_ENCLOSURE = [
    ("1800-2017", "28.4", 396, "agree"),      # the delay sentence's real clause
    ("1800-2017", "28.16", 396, "disagree"),  # SV-CORPUS-GRAD.13q, the founding defect
    ("1800-2017", "28.3", 243, "agree"),      # an ANCESTOR clause is a correct citation
    ("1364-2005", "7.2", 376, "agree"),
    ("1364-2005", "7.14", 376, "disagree"),
]
GT_ADDRESSABLE = [
    ("1800-2017", "section-28-gate-level-and-switch-level-modeling.md", "28", True),
    ("1800-2017", "section-8-classes.md", "8", True),
    ("1364-2005", "section-7-gate-and-switch-level-modeling.md", "7", True),
    # ⭐ SV-CORPUS-GRAD.13u — the two the file-level test got WRONG, pinned so it cannot come back.
    # `section-15-…` SPANS 15.1 -> 16.18 and carries 26 clause-15 headings: correctly named, and
    # answerable for BOTH majors. `section-29-…` is 29.1 -> 29.10 and is not defective at all.
    ("1800-2017", "section-15-interprocess-synchronization-and-communication.md", "15", True),
    ("1800-2017", "section-15-interprocess-synchronization-and-communication.md", "16", True),
    ("1800-2017", "section-29-user-defined-primitives.md", "29", True),
    # genuinely unanswerable: no heading of that major exists in the file at all
    ("1800-2017", "section-41-data-read-api.md", "41", False),
    ("1800-2017", "section-2-normative-references.md", "2", False),
    ("1800-2017", "section-28-gate-level-and-switch-level-modeling.md", "11", False),
]
# The NOISE-heading pair: v2005 `section-7` line 2328 sits under a stray `## 3 delays` (:2327) —
# a fragment of §7.14's OWN delay-cardinality prose that the PDF->md split promoted to a heading.
# The real enclosing clause is §7.14 (:2299). ⛔ The naive leg is asserted too: if an unfiltered
# read stops giving `3`, this pair has gone stale and proves nothing, so it REFUSES rather than
# passing. It caught its own author writing `2` (the FIRST stray, not the innermost one).
GT_NOISE = ("1364-2005", "section-7-gate-and-switch-level-modeling.md", 2328, "7.14", "3")
GT_SCOPE = [
    (" (`:396`, `:454`)", True),        # the clause's own parenthetical
    (":396 zero, one, or two", True),   # glued, no bracket at all
    (" / A.1.2) but NOT referenced in `class_item` (`section-8-classes.md:95`)", False),
]


def _fail(msg):
    print(f"MISCALIBRATED: {msg}", file=sys.stderr)
    sys.exit(7)


def calibrate():
    for edition, name, major, want in GT_ADDRESSABLE:
        p = ROOT / EDITION_DIR[edition] / name
        if not p.is_file():
            _fail(f"ground-truth file missing: {p}")
        if addressable_for(p, major) != want:
            _fail(f"addressable_for({name}, {major}) -> {not want}, expected {want}")

    edition, name, line, want_real, want_naive = GT_NOISE
    p = ROOT / EDITION_DIR[edition] / name
    got = enclosing(p, line, want_real.split(".")[0])
    if got is None or got[0] != want_real:
        _fail(f"enclosing({name}, {line}, major {want_real.split('.')[0]}) -> "
              f"{got[0] if got else None!r}, expected {want_real!r} — the cited-major filter is "
              f"not firing (an unfiltered read gives {want_naive!r})")
    naive = None
    for i, clause, _ in headings(p):
        if i > line:
            break
        naive = clause
    if naive != want_naive:
        _fail(f"the noise-heading control has gone stale: an unfiltered read of {name}:{line} "
              f"gives {naive!r}, not the {want_naive!r} this pair was built on")

    for span, want in GT_SCOPE:
        off = span.index(":") if ":" in span else 0
        if ":396" in span or ":95" in span:
            off = span.index(":95") if ":95" in span else span.index(":396")
        if in_clause_scope(span, off) != want:
            _fail(f"in_clause_scope({span!r}) -> {not want}, expected {want}")

    for edition, clause, line, want in GT_ENCLOSURE:
        files = section_index(edition).get(int(clause.split(".")[0]), [])
        if len(files) != 1:
            _fail(f"{edition} §{clause} does not resolve to one section file")
        got, _ = judge(files[0], clause, line)
        if got != want:
            _fail(f"{edition} §{clause} :{line} -> {got}, expected {want}")


def citations(text):
    """Yield (citing_line, edition, clause, lrm_path_or_name, lrm_line, unbound_reason)."""
    lines = text.splitlines()
    for i0, ltext in enumerate(lines):
        window = ltext + ("\n" + lines[i0 + 1] if i0 + 1 < len(lines) else "")
        for m in ANCHOR.finditer(ltext):
            cited = m.group(1)
            nxt = ANCHOR.search(window, m.end())
            span = window[m.end():min(nxt.start() if nxt else len(window), m.end() + SPAN_MAX)]

            refs = []  # (edition_dir_or_None, lrm_file_or_None, line, span_end)
            glued = F2.match(span)
            if glued:
                refs.append((None, None, int(glued.group(1)), glued.end()))
            for fm in F1.finditer(span):
                if not in_clause_scope(span, fm.start()):
                    continue
                refs.append((fm.group(1), fm.group(2), int(fm.group(3)), fm.end()))
            if not refs:
                yield (i0 + 1, None, cited, None, None, "no_line_ref")
                continue

            edition = next((DIR_EDITION[r[0]] for r in refs if r[0]), None)
            if edition is None:
                behind = EDITION.findall(window[max(0, m.start() - BACK_EDITION):m.start()])
                edition = behind[-1] if behind else next(iter(EDITION.findall(span)), None)
            if edition is None:
                yield (i0 + 1, None, cited, None, None, "no_edition")
                continue

            index = section_index(edition)
            bound = []
            for _, fname, line, end in refs:
                if fname is None:
                    cands = index.get(int(cited.split(".")[0]), [])
                else:
                    cands = [q for qs in index.values() for q in qs if q.name == fname]
                if len(cands) != 1:
                    kind = "annex_ref" if fname and "Annex" in fname else "unresolved_file"
                    yield (i0 + 1, edition, cited, fname, line, kind)
                    continue
                if not addressable_for(cands[0], cited.split(".")[0]):
                    yield (i0 + 1, edition, cited, cands[0].name, line, "unaddressable_section")
                    continue
                bound.append((cands[0], line, end))
            for f3 in F3.finditer(span):
                if not in_clause_scope(span, f3.start()):
                    continue
                prior = [b for b in bound if b[2] <= f3.start() <= b[2] + CHAIN_MAX]
                if prior:
                    bound.append((prior[-1][0], int(f3.group(1)), f3.end()))
            for path, line, _ in bound:
                yield (i0 + 1, edition, cited, path, line, None)


def census(verbose=True):
    buckets = dict.fromkeys(
        ("agree", "disagree", "no_heading", "no_line_ref", "no_edition",
         "annex_ref", "unaddressable_section", "unresolved_file"), 0)
    rows, scanned = [], 0
    tracked = subprocess.run(["git", "ls-files"], capture_output=True, text=True,
                             cwd=ROOT, check=True).stdout.split()
    for rel in tracked:
        if rel.startswith(SKIP_PREFIX) or any(s in rel for s in SKIP_INFIX):
            continue
        try:
            text = (ROOT / rel).read_text(errors="replace")
        except (OSError, UnicodeDecodeError):
            continue
        if "§" not in text:
            continue
        scanned += 1
        for citing_line, edition, cited, path, line, unbound in citations(text):
            if unbound:
                buckets[unbound] += 1
                continue
            # ⛔ SV-CORPUS-GRAD.13u — THE FILENAME'S NUMBER IS NEVER EVIDENCE, anywhere in this
            # census. The first cut also compared it against the cited major and called a
            # disagreement `section_mismatch`; that is the same filename-trusting mistake in
            # another guise, and it fired on two CORRECT citations of §11.4.12 into
            # `section-0-…md`, a file that carries 35 of its 39 headings in clause 11. Whether a
            # file can answer for a clause is decided by `addressable_for` — its content — and
            # nothing else.
            verdict, enc = judge(path, cited, line)
            buckets[verdict] += 1
            if verdict == "disagree":
                rows.append((rel, citing_line, edition, cited, path.name, line,
                             enc[0], enc[2][:56]))
    if verbose:
        print(f"LRM-CLAUSE-CITATION: files_scanned={scanned} "
              + " ".join(f"{k}={v}" for k, v in buckets.items()))
        if rows:
            print()
            print(f"{'CITING FILE':52} {'LN':>6}  {'EDITION':10} {'CITED':10} "
                  f"{'LRM LINE':>9}  {'ENCLOSING':11} HEADING")
            for r in sorted(set(rows)):
                print(f"{r[0][:52]:52} {r[1]:>6}  {r[2]:10} §{r[3]:9} "
                      f"{r[4][:18]}:{r[5]:<6}  §{r[6]:10} {r[7]}")
            print()
            print("  A clause number is a MEASUREMENT, not a label. Re-derive it: the innermost "
                  "numbered\n  heading at or before the cited line IS the clause that contains it.")
    return buckets, rows


def _classify(text):
    """Bucket counts for one synthetic document, through the REAL citation pipeline."""
    out = {}
    for _, _, cited, path, line, unbound in citations(text):
        key = unbound
        if key is None:
            key = judge(path, cited, line)[0]
        out[key] = out.get(key, 0) + 1
    return out


# ── synthetic probes: one per structural rule, so no control can go vacuous ───────────────────
# ⛔ SV-CORPUS-GRAD.13u. The first cut mutated each rule on the REAL TREE and asserted the census
# noticed. That is a good corroboration and a BAD primary control: when `.13u` corrected the
# addressability rule, no citation in the repository landed on an unaddressable pair any more, so
# the mutation changed nothing and the arm reported itself INERT. A control whose subject the tree
# happens not to contain is not a passing control, it is an absent one
# (SV-CORPUS-GRAD.87(e): a green mutation is a question about the CONFIGURATION before it is a
# verdict about the lock). Each rule now has a probe that carries its own subject.
PROBE_SCOPE = (
    "`interface_class_declaration` is defined (IEEE 1800-2017 \u00a78.26.1 / A.1.2) but NOT "
    "referenced in `class_item` "
    "(`docs/systemverilog/2017/md/section-8-classes.md:95`)"
)
PROBE_ADDRESSABLE = (
    "IEEE 1800-2017 \u00a711.4 "
    "(`docs/systemverilog/2017/md/section-28-gate-level-and-switch-level-modeling.md:396`)"
)
PROBE_CITED_MAJOR = (
    "IEEE 1364-2005 \u00a77.14 "
    "(`docs/verilog/2005/md/section-7-gate-and-switch-level-modeling.md:2328`)"
)


def self_test():
    """Ground truth, then a RED control per structural rule that carries its OWN subject.

    Each arm asserts BOTH directions: the rule live gives one classification, the rule disabled
    gives a different one. A tree-level corroboration follows, and reports NOT EXERCISED — never
    a pass — when this repository happens not to contain the shape.
    """
    global in_clause_scope, addressable_for
    calibrate()
    print("self-test: ground truth OK "
          f"({len(GT_ENCLOSURE)} enclosure + {len(GT_ADDRESSABLE)} addressability "
          f"+ 1 cited-major + {len(GT_SCOPE)} scope pairs)")

    base, _ = census(verbose=False)
    if base["disagree"]:
        print(f"self-test: FAIL — the tree is not clean to start (disagree={base['disagree']})",
              file=sys.stderr)
        return 1

    failures = 0

    # ── arm 1: bracket scope ──────────────────────────────────────────────────────────────────
    live = _classify(PROBE_SCOPE)
    if live.get("no_line_ref") != 1 or live.get("disagree"):
        print(f"self-test: FAIL — the scope probe is not being unbound: {live}", file=sys.stderr)
        failures += 1
    saved = in_clause_scope
    in_clause_scope = lambda span, offset: True                          # noqa: E731
    try:
        red = _classify(PROBE_SCOPE)
    finally:
        in_clause_scope = saved
    if red.get("disagree") != 1:
        print(f"self-test: FAIL — disabling bracket scoping leaves the probe unjudged: {red}",
              file=sys.stderr)
        failures += 1
    else:
        print("self-test: RED arm 1 OK — bracket scope: the defined-here/absent-there probe is "
              "unbound live and MISJUDGED (disagree=1) with the rule disabled")

    # ── arm 2: per-citation addressability ────────────────────────────────────────────────────
    live = _classify(PROBE_ADDRESSABLE)
    if live.get("unaddressable_section") != 1:
        print(f"self-test: FAIL — the addressability probe is not being bucketed: {live}",
              file=sys.stderr)
        failures += 1
    saved = addressable_for
    addressable_for = lambda path, major: True                           # noqa: E731
    try:
        red = _classify(PROBE_ADDRESSABLE)
    finally:
        addressable_for = saved
    if red.get("unaddressable_section"):
        print(f"self-test: FAIL — disabling addressability changes nothing: {red}", file=sys.stderr)
        failures += 1
    else:
        print("self-test: RED arm 2 OK — addressability: a \u00a711.4 citation into the clause-28 "
              f"file is bucketed live and {list(red)} with the rule disabled")

    # ── arm 3: the cited-major heading filter ─────────────────────────────────────────────────
    live = _classify(PROBE_CITED_MAJOR)
    if live.get("agree") != 1:
        print(f"self-test: FAIL — the cited-major probe should AGREE (\u00a77.14 contains "
              f":2328): {live}", file=sys.stderr)
        failures += 1
    else:
        print("self-test: RED arm 3 OK — cited-major filter: \u00a77.14 :2328 AGREES, where an "
              "unfiltered read stops at the stray `## 3 delays` heading (asserted in ground truth)")

    # ── tree corroboration, honest about what this repository does and does not exercise ──────
    saved = in_clause_scope
    in_clause_scope = lambda span, offset: True                          # noqa: E731
    try:
        red, _ = census(verbose=False)
    finally:
        in_clause_scope = saved
    if red["disagree"]:
        print(f"self-test: tree corroboration — bracket scope guards {red['disagree']} live row(s)")
    else:
        print("self-test: tree corroboration — bracket scope NOT EXERCISED by this tree "
              "(no live citation of that shape); the synthetic arm above is the control")

    clean, _ = census(verbose=False)
    if clean != base:
        print(f"self-test: FAIL — the tree did not return to its starting census ({base} -> "
              f"{clean})", file=sys.stderr)
        failures += 1
    else:
        print("self-test: GREEN control OK — the real tree re-reads identically after every arm")
    return 1 if failures else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--self-test", action="store_true",
                    help="ground truth + a RED control per structural rule")
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    calibrate()
    _, rows = census()
    return 1 if rows else 0


if __name__ == "__main__":
    sys.exit(main())
