#!/usr/bin/env python3
"""LRM clause-citation census (SV-CORPUS-GRAD.13q).

Does a cited IEEE clause number actually enclose the LRM line it is cited with?

⛔ THE DEFECT THIS CLOSES. `SV-CORPUS-GRAD.13p` located the gate-delay cardinality sentence
correctly — the LINE numbers were measured — and labelled it IEEE 1800-2017 §28.16 / IEEE
1364-2005 §7.14, carried out of §28.3.3's cross-reference *"Delays are discussed in more detail
in 28.16"* rather than measured. Not one of the seven cited lines falls inside §28.16 / §7.14;
they sit under §28.4 / §28.5 / §28.6 / §28.3.3 and §7.2 / §7.3 / §7.4. A line number and a clause
number are two DIFFERENT measurements of one citation and only one of them had been made
(docs/knowledge/a-number-read-out-of-context-carries-a-frame-you-did-not-measure.md).

⛔ IT BINDS STRUCTURALLY OR NOT AT ALL. The first cut resolved the LRM file from the clause's own
major number and swept every `:N` between one `§` and the next: 160 "disagreements", most of them
the tool attributing a `furthest_position=`, a source-line reference or a neighbouring citation's
line to the wrong clause — a ledger row is one physical line thousands of characters long, so
"the next §" is not a boundary. A line reference is judged ONLY when its file is pinned by one of:

  F1  an explicit `…/section-….md:N` path            (its directory also fixes the edition;
      an Annex file is counted `annex_ref` — an annex heading carries no clause number, so
      enclosure is not the question its citations ask)
  F2  `§X.Y:N` or ``§X.Y (`:N`)`` — the line belongs to the clause itself, nothing but
      punctuation between them (file = that clause's own section file)
  F3  a bare `` `:N` `` continuing an F1/F2 within CHAIN_MAX chars of it, same span

Everything else is counted in `no_line_ref` / `no_edition` / `unresolved_file` and reported, never
silently dropped (docs/decisions/feedback_enumerating_instrument_must_refuse.md).

⭐ THE ENCLOSURE PREMISE IS A PROPERTY OF A CITATION SHAPE, NOT OF CITATIONS. Two shapes break it
and both are named in REVIEWED_NOT_A_DEFECT below with their reason, rather than suppressed by
path. Read the rows; this is a lead generator, not an oracle that thinks for you.

Usage:
  python3 stimuli/sv/lrm_clause_citation_census.py            # census; rc 1 if any row is unruled
  python3 stimuli/sv/lrm_clause_citation_census.py --all      # also list the reviewed rows

Exit: 0 clean · 1 at least one unruled disagreement · 7 ground truth failed (MISCALIBRATED).
"""

import argparse
import re
import subprocess
import sys
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

# ⛔ `^#{2,6} [^`]*\`` , never `^### \`` — a heading census is only as good as its heading grammar
# (docs/knowledge/a-heading-census-is-only-as-good-as-the-heading-grammar.md). Here the numbered
# form is unambiguous, and the count it yields per edition (1837 / 1680 / 851) is the ground truth
# that a regression in this pattern would move.
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

# ── rows READ and RULED not a defect ──────────────────────────────────────────────────────────
# Keyed (citing file, cited clause, LRM file, LRM line) — deliberately NOT by the citing file's
# own line number, which moves whenever the file is edited. A key that matches nothing is
# reported as `reviewed_stale`: an allowance that has stopped allowing anything is rot, and the
# silent direction is the dangerous one.
REVIEWED_NOT_A_DEFECT = {
    # SHAPE 1 — "defined in §A, but NOT referenced at file:line". The clause and the line are
    # deliberately different places, so enclosure is the wrong question. GRAMMAR-WELLFORMED.H.12.5.2
    # adjudicated the underlying LRM omission; these sentences state it.
    ("CHANGES.md", "8.26.1", "section-8-classes.md", 95):
        "defined-here/absent-there: §8.26.1 defines interface_class_declaration; :95 is the "
        "class_item alternative list that omits it",
    ("DEVELOPMENT_NOTES.md", "8.26", "section-8-classes.md", 102):
        "defined-here/absent-there: §8.26 is the interface-class prose; :102 is the omitting site",
    ("DEVELOPMENT_NOTES.md", "8.26.1", "section-26-packages.md", 60):
        "defined-here/absent-there, across files: package_or_generate_item_declaration is in "
        "section-26, the defining clause in section-8",
    ("DEVELOPMENT_NOTES.md", "8.26.1", "section-8-classes.md", 95):
        "defined-here/absent-there: as CHANGES.md above",
    ("docs/tasks/GRAMMAR-WELLFORMED.md", "8.26.1", "section-8-classes.md", 95):
        "defined-here/absent-there: the H.12.5.2 adjudication row that ruled the omission "
        "LRM-faithful",
    # SHAPE 2 — the PDF→md split mis-titled the file holding §11.4.12, so its NAME carries no
    # clause number. The citation is right; the corpus filename is what cannot be checked.
    ("KNOWLEDGE_MAP.md", "11.4.12", "section-0-defined-as-false-or-if-the-result-is-ambiguous-"
     "the-unknown-value-x-the-precedence-of-is-greater.md", 520):
        "PDF-split artifact: the §11.4.12 notation sentence lives in a section-0-… file",
    ("docs/knowledge/a-mis-cited-production-reproduces-as-a-success.md",
     "11.4.12", "section-0-defined-as-false-or-if-the-result-is-ambiguous-the-unknown-value-x-"
     "the-precedence-of-is-greater.md", 520):
        "PDF-split artifact: as KNOWLEDGE_MAP.md above",
}

_idx: dict = {}
_head: dict = {}


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


def enclosing(path, line):
    best = None
    for i, clause, text in headings(path):
        if i > line:
            break
        best = (clause, i, text)
    return best


def judge(path, cited, line):
    """`agree` when the innermost heading enclosing `line` IS `cited` or a descendant of it."""
    enc = enclosing(path, line)
    if enc is None:
        return "no_heading", None
    clause = enc[0]
    return ("agree" if clause == cited or clause.startswith(cited + ".") else "disagree"), enc


# ── ground truth, re-run on every invocation ──────────────────────────────────────────────────
# An instrument with no ground truth is a confident guess
# (docs/decisions/feedback_instrument_needs_ground_truth.md). The known-bad pair is this leaf's
# own defect, so a regression that stops seeing it exits 7 rather than reporting a clean tree.
GROUND_TRUTH = [
    ("1800-2017", "28.4", 396, "agree"),      # the sentence's real clause
    ("1800-2017", "28.16", 396, "disagree"),  # SV-CORPUS-GRAD.13q, the defect itself
    ("1800-2017", "28.3", 243, "agree"),      # an ANCESTOR clause is a correct citation
    ("1364-2005", "7.2", 376, "agree"),
    ("1364-2005", "7.14", 376, "disagree"),
]


def calibrate():
    for edition, clause, line, want in GROUND_TRUTH:
        files = section_index(edition).get(int(clause.split(".")[0]), [])
        if len(files) != 1:
            print(f"MISCALIBRATED: {edition} §{clause} does not resolve to one section file",
                  file=sys.stderr)
            sys.exit(7)
        got, _ = judge(files[0], clause, line)
        if got != want:
            print(f"MISCALIBRATED: {edition} §{clause} :{line} -> {got}, expected {want}",
                  file=sys.stderr)
            sys.exit(7)


def citations(text):
    """Yield (citing_line, edition, clause, lrm_path, lrm_line) for every STRUCTURALLY bound ref."""
    lines = text.splitlines()
    for i0, ltext in enumerate(lines):
        window = ltext + ("\n" + lines[i0 + 1] if i0 + 1 < len(lines) else "")
        for m in ANCHOR.finditer(ltext):
            cited = m.group(1)
            nxt = ANCHOR.search(window, m.end())
            span = window[m.end():min(nxt.start() if nxt else len(window), m.end() + SPAN_MAX)]

            refs = []  # (edition_dir_or_None, lrm_file_or_None, line, span_end)
            # F2 -- the line reference belongs to the clause itself: `§X.Y:N`, or `§X.Y (`:N`)`
            # with nothing but punctuation between. The second spelling is this repo's dominant
            # citation shape and the first cut did not bind it, so splitting one clause into two
            # (`§28.4 (`:396`) / §28.5 (`:454`)`) silently made the second half uncheckable.
            glued = F2.match(span)
            if glued:
                refs.append((None, None, int(glued.group(1)), glued.end()))
            for fm in F1.finditer(span):
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
                    # An Annex file carries no clause NUMBER, so enclosure is not the question
                    # its citations ask; report it as its own bucket rather than as a failure.
                    kind = "annex_ref" if fname and "Annex" in fname else "unresolved_file"
                    yield (i0 + 1, edition, cited, fname, line, kind)
                    continue
                bound.append((cands[0], line, end))
            for f3 in F3.finditer(span):
                prior = [b for b in bound if b[2] <= f3.start() <= b[2] + CHAIN_MAX]
                if prior:
                    bound.append((prior[-1][0], int(f3.group(1)), f3.end()))
            for path, line, _ in bound:
                yield (i0 + 1, edition, cited, path, line, None)


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--all", action="store_true",
                    help="also list the rows ruled not a defect")
    args = ap.parse_args()

    calibrate()

    buckets = dict.fromkeys(
        ("agree", "disagree", "section_mismatch", "no_heading", "no_line_ref",
         "no_edition", "annex_ref", "unresolved_file", "reviewed_not_a_defect"), 0)
    unruled, reviewed_hits, scanned = [], set(), 0

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
            key = (rel, cited, path.name, line)
            section = int(re.match(r"section-(\d+)-", path.name).group(1))
            if section != int(cited.split(".")[0]):
                verdict, enc = "section_mismatch", None
            else:
                verdict, enc = judge(path, cited, line)
            if verdict in ("disagree", "section_mismatch"):
                if key in REVIEWED_NOT_A_DEFECT:
                    buckets["reviewed_not_a_defect"] += 1
                    reviewed_hits.add(key)
                    if args.all:
                        print(f"  ruled: {rel}:{citing_line} §{cited} -> {path.name}:{line} "
                              f"— {REVIEWED_NOT_A_DEFECT[key]}")
                    continue
                unruled.append((rel, citing_line, edition, cited, path.name, line,
                                enc[0] if enc else "-", enc[2][:56] if enc else "CLAUSE/FILE MISMATCH"))
            buckets[verdict] += 1

    stale = sorted(set(REVIEWED_NOT_A_DEFECT) - reviewed_hits)
    print(f"LRM-CLAUSE-CITATION: files_scanned={scanned} "
          + " ".join(f"{k}={v}" for k, v in buckets.items())
          + f" reviewed_stale={len(stale)}")
    for key in stale:
        print(f"  STALE allowance (matches nothing): {key}", file=sys.stderr)
    if unruled:
        print()
        print(f"{'CITING FILE':52} {'LN':>6}  {'EDITION':10} {'CITED':10} "
              f"{'LRM LINE':>9}  {'ENCLOSING':11} HEADING")
        for r in sorted(set(unruled)):
            print(f"{r[0][:52]:52} {r[1]:>6}  {r[2]:10} §{r[3]:9} "
                  f"{r[4][:18]}:{r[5]:<6}  §{r[6]:10} {r[7]}")
    return 1 if (unruled or stale) else 0


if __name__ == "__main__":
    sys.exit(main())
