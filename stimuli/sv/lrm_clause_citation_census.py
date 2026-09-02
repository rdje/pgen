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
file is CLAUSE-ADDRESSABLE (see `clause_addressable`). Everything else is COUNTED and printed —
`no_line_ref`, `no_edition`, `annex_ref`, `unaddressable_section`, `unresolved_file` — never
silently dropped (docs/decisions/feedback_enumerating_instrument_must_refuse.md). The size of what
it declines to judge is printed beside the verdict on purpose: a census that gets cleaner because
it stopped looking is the failure mode that looks most like success, and this instrument caught its
own author committing it (judged rows fell 35 -> 17 mid-fix while the report stayed green).

⭐ THERE IS NO ALLOWANCE TABLE, AND THAT IS THE DESIGN. Two citation shapes legitimately break the
enclosure premise; the first cut carried them as seven hand-written exceptions keyed by line
numbers that move. Both are recognised STRUCTURALLY instead — `in_clause_scope` for the
defined-here/absent-there sentence, `clause_addressable` for the mis-titled corpus file. A gate
whose green depends on a hand-maintained exception list is not a gate, it is a promise.

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


def modal_major(path):
    """The clause major MOST of this file's own numbered headings carry.

    Parameter-free on purpose: a threshold would be a magic number, and the separation is not
    close — `section-0-…md` scores 1 of 39 against its filename while `section-28-…md` scores 34
    of 40. The modal major is what the file SAYS it holds; the filename is what it CLAIMS.
    """
    key = str(path)
    if key not in _modal:
        majors = [c.split(".")[0] for _, c, _ in headings(path)]
        _modal[key] = Counter(majors).most_common(1)[0][0] if majors else None
    return _modal[key]


def clause_addressable(path):
    """Does the file's own content agree that it holds the clause its NAME claims?

    ⛔ SV-CORPUS-GRAD.13q(c) — MEASURED, AND THE ANSWER IS NOT ALWAYS YES. In the vendored
    IEEE 1800-2017 markdown corpus this is FALSE for 14 of 55 numbered section files and for 3 of
    41 in 1800-2023 (0 of 28 in 1364-2005). The PDF->md split titled some from a sentence fragment
    (`section-1024-characters-if-an-identifier-exceeds-…`, `section-65536-216-bits`) and
    mis-numbered others outright: `section-15-…` holds clause 16, `section-16-777-216-224-elements`
    holds clause 7, and `section-41-data-read-api.md` carries no numbered heading at all while
    holding Annex A text that several task leaves cite by line.

    This census resolves a file FROM a clause number for the F2/F3 forms, so trusting the filename
    would manufacture a disagreement for every citation into those files — and did, in the first
    cut. A citation into a non-addressable file is COUNTED (`unaddressable_section`) and not
    judged: the citation may be perfectly correct, and the corpus filename is simply not evidence
    to check it against. Owned as a corpus-integrity defect by `SV-CORPUS-GRAD.13t`.
    """
    m = re.match(r"section-(\d+)-", path.name)
    return bool(m) and modal_major(path) == m.group(1)


def enclosing(path, line):
    """Innermost heading at or before `line`, ignoring PDF-split NOISE headings.

    ⛔ A heading whose major disagrees with the file's own modal major is not a clause heading of
    this clause. `docs/verilog/2005/md/section-7-gate-and-switch-level-modeling.md` carries four
    such strays — `## 2 delays` and `## 3 delays` at `:2325`/`:2327`/`:2435`/`:2437`, which are
    fragments of §7.14's own delay-cardinality prose that the split promoted to headings. Reading
    one as the enclosing clause reports every citation past it as wrong; the first cut did exactly
    that, and line 2328 resolved to clause `2` instead of §7.14.
    """
    own = modal_major(path)
    best = None
    for i, clause, text in headings(path):
        if i > line:
            break
        if clause.split(".")[0] != own:
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
    enc = enclosing(path, line)
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
    ("1800-2017", "section-28-gate-level-and-switch-level-modeling.md", True),
    ("1800-2017", "section-8-classes.md", True),
    ("1364-2005", "section-7-gate-and-switch-level-modeling.md", True),
    # the PDF-split artifacts — their names are sentence fragments, not clause addresses
    ("1800-2017", "section-0-defined-as-false-or-if-the-result-is-ambiguous-the-unknown-"
                  "value-x-the-precedence-of-is-greater.md", False),
    ("1800-2017", "section-41-data-read-api.md", False),
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
    for edition, name, want in GT_ADDRESSABLE:
        p = ROOT / EDITION_DIR[edition] / name
        if not p.is_file():
            _fail(f"ground-truth file missing: {p}")
        if clause_addressable(p) != want:
            _fail(f"clause_addressable({name}) -> {not want}, expected {want}")

    edition, name, line, want_real, want_naive = GT_NOISE
    p = ROOT / EDITION_DIR[edition] / name
    got = enclosing(p, line)
    if got is None or got[0] != want_real:
        _fail(f"enclosing({name}, {line}) -> {got[0] if got else None!r}, expected {want_real!r} "
              f"— the noise-heading filter is not firing (naive answer is {want_naive!r})")
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
                if not clause_addressable(cands[0]):
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
        ("agree", "disagree", "section_mismatch", "no_heading", "no_line_ref", "no_edition",
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
            section = int(re.match(r"section-(\d+)-", path.name).group(1))
            if section != int(cited.split(".")[0]):
                verdict, enc = "section_mismatch", None
            else:
                verdict, enc = judge(path, cited, line)
            buckets[verdict] += 1
            if verdict in ("disagree", "section_mismatch"):
                rows.append((rel, citing_line, edition, cited, path.name, line,
                             enc[0] if enc else "-",
                             enc[2][:56] if enc else "CLAUSE/FILE MISMATCH"))
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


def self_test():
    """Ground truth plus a RED control per structural rule, on the REAL corpus.

    ⭐ Each control MUTATES the rule it names and asserts the census notices, so a rule that
    silently stops working cannot pass as a clean tree. A green control on a rule that cannot
    fail is worth nothing (docs/knowledge/... — SV-CORPUS-GRAD `.87`(e)'s withdrawn premise).
    """
    global in_clause_scope, clause_addressable
    calibrate()
    print("self-test: ground truth OK "
          f"({len(GT_ENCLOSURE)} enclosure + {len(GT_ADDRESSABLE)} addressability "
          f"+ 1 noise + {len(GT_SCOPE)} scope pairs)")
    base, _ = census(verbose=False)
    if base["disagree"] or base["section_mismatch"]:
        print(f"self-test: FAIL — the tree is not clean to start "
              f"(disagree={base['disagree']} section_mismatch={base['section_mismatch']})",
              file=sys.stderr)
        return 1

    arms = 0
    saved_scope, saved_addr = in_clause_scope, clause_addressable
    try:
        in_clause_scope = lambda span, offset: True                      # noqa: E731
        red, _ = census(verbose=False)
        if red["disagree"] + red["section_mismatch"] <= 0:
            print("self-test: FAIL — disabling bracket scoping changes nothing; the rule is inert",
                  file=sys.stderr)
            return 1
        print(f"self-test: RED arm 1 OK — without bracket scoping, "
              f"{red['disagree'] + red['section_mismatch']} row(s) misjudge "
              "(the defined-here/absent-there shape)")
        arms += 1
    finally:
        in_clause_scope = saved_scope

    try:
        clause_addressable = lambda path: True                           # noqa: E731
        _modal.clear()
        red, _ = census(verbose=False)
        if red["disagree"] + red["section_mismatch"] <= 0:
            print("self-test: FAIL — trusting every filename changes nothing; the rule is inert",
                  file=sys.stderr)
            return 1
        print(f"self-test: RED arm 2 OK — trusting every section filename, "
              f"{red['disagree'] + red['section_mismatch']} row(s) misjudge "
              "(citations into mis-titled corpus files)")
        arms += 1
    finally:
        clause_addressable = saved_addr
        _modal.clear()

    clean, _ = census(verbose=False)
    if clean != base:
        print("self-test: FAIL — the tree did not return to its starting census after the RED "
              f"arms ({base} -> {clean})", file=sys.stderr)
        return 1
    print(f"self-test: GREEN control OK — the real tree re-reads identically after {arms} RED arms")
    return 0


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
