#!/usr/bin/env python3
"""table_pipe_repair.py — escape the pipes that a markdown table is silently eating.

Owning leaf: ``LIVE-DOC-CONTAINMENT.8`` (a tracked markdown table silently drops cells).

WHAT THIS FIXES, AND WHAT IT REFUSES TO FIX
-------------------------------------------
GFM splits a table row on unescaped ``|`` BEFORE it parses inline content, so a ``|`` written
inside a backtick span still breaks the row — and the surplus cells are then DISCARDED with no
warning.  The leaf's census names two causes and says only reading each row separates them:

  (1) an unescaped ``|`` inside PROSE/code (a BNF alternation, a shell pipe, a type union)
      -> the repair is ``|`` -> ``\\|`` and nothing else;
  (2) a genuine extra CELL (the author invented a column)
      -> no mechanical repair exists; the row must be read.

This tool implements ONLY cause (1), and it proves it stayed inside that box three ways:

  * CODE-SPAN SCOPED.  It escapes a pipe only where CommonMark's own code-span rule (a backtick
    run of length N closed by the next run of exactly N) says the author had opened a span.  A
    pipe in open prose is left alone, because escaping it would be a guess about intent.
  * ARITY-ENDORSED.  A repaired row is accepted only if it lands EXACTLY on the arity its own
    header declares.  Landing short or long means the pairing did not reconstruct the author's
    intent, so the row is reported for reading instead of written.
  * SEMANTICALLY ENDORSED, by a check that knows nothing about arity.  Every table gets a
    column predicate derived from its healthy rows (the ledger's column 6 is a date in 171 of
    171; the index's column 5 is a link into ``tasks/``).  A merge that satisfies a predicate
    unrelated to counting is right for a reason other than hitting the target number.

CONTENT-PRESERVATION INVARIANT (hard, aborts the run)
-----------------------------------------------------
``fixed`` and ``original`` must be byte-identical once every ``\\|`` is normalised back to
``|``.  The only permitted edit is the insertion of a backslash.  This invariant is not
decorative: it ABORTED the first run of this repair on ``SV-0066``, whose ``\\\\|`` renders at
the declared 13 columns and is NOT defective.  A mechanical fixer that had merely counted to 13
would have corrupted a healthy row in a contract this project SHIPS to a downstream consumer.

USAGE
-----
  table_pipe_repair.py [file ...]          # dry run: report per row, write nothing
  table_pipe_repair.py --apply [file ...]  # write the endorsed rows in place
  table_pipe_repair.py --residual [file..] # print the full text of every row needing a read
Default files: docs/TASK_TREE.md docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md
Exit 0 on a clean dry run or a clean apply; 3 if the invariant tripped (nothing written).
"""
import os
import re
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "..", ".."))

# Per-table semantic predicates, keyed by (file, 1-based table index).  Each maps a 0-based
# column to a regex the cell must satisfy.  These are derived from the HEALTHY rows and are
# deliberately independent of column COUNTING -- that is what makes them an endorsement rather
# than a restatement of the arity test.
PREDICATES = {
    ("docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md", 1): {
        5: (r"^\s*`?\d{4}-\d{2}-\d{2}`?\s*$", "column 6 `First Reported` is a bare ISO date"),
    },
    ("docs/TASK_TREE.md", 1): {
        4: (r"\]\(tasks/", "column 5 `File` is a markdown link into tasks/"),
    },
}

SEP_ROW = re.compile(r"^\| *:?-+:? *\|")

# HAND-ADJUDICATED JOINS — the rows the mechanical arm REFUSED, read one by one and recorded as
# DATA so the decision is durable, re-runnable and reviewable rather than a one-off edit.
#
# ⛔ WHY THE MECHANICAL ARM CANNOT DO THESE, MEASURED RATHER THAN ASSUMED. CommonMark pairs a
#   backtick run with the NEXT run of the same length, left to right. That pairing is GLOBAL over
#   the line, so ONE unmatched backtick anywhere shifts every pairing after it — and a shifted
#   pairing swallows REAL cell separators into phantom code spans. `SV-0034` is the worked case:
#   it has exactly ONE pipe that needs escaping (a BNF alternation inside
#   `associative_dimension := …`), but a stray trailing backtick later in the row makes the scan
#   report THREE, so escaping them all lands the row at 11 cells against a 13-column header. The
#   tool refused it — landing SHORT is the refusal — and reading is what separates the one real
#   join from the two false ones.
#
# ⭐ HOW EACH GROUPING WAS DERIVED: from the header's OWN column semantics, not from counting down
#   to 13. The bug ledger's 13 columns have recognisable shapes — column 1 is the id, column 6 a
#   bare ISO date, column 12 a release string, column 13 free prose — so the fragments were read
#   back onto the columns they belong to and the groups are what remains. Each grouping is then
#   checked three ways by the code below exactly like a mechanical repair: exact declared arity,
#   the table's semantic predicate (which knows nothing about counting), and the
#   content-preservation invariant.
#
# Format: (file, table index, row name) -> list of fragment-index groups (0-based) to re-join.
HAND_JOINS = {
    ("docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md", 1, "SV-0034"): [[10, 11]],
    ("docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md", 1, "SV-0035"): [[10, 11, 12]],
    ("docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md", 1, "SV-0048"): [[10, 11]],
    ("docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md", 1, "REGEX-0108"): [[12, 13, 14]],
    # REGEX-0105 carries 37 fragments: the Downstream-Consumer, Reproducer-Bundle, Root-Cause and
    # Fix-Proof cells are each shredded by `|` characters inside regex character classes written
    # in code spans (`[\d-[|z]]`, `[~-||]`, `[|-||]`, `[a-|b]`). Empty fragments are real: `||`
    # is two adjacent separators produced by one two-pipe class.
    # docs/TASK_TREE.md table 1 declares `Tree | Status | Roadmap lane | Current frontier | File`.
    # Same reading method: the Tree cell, the lane sentence and the trailing `](tasks/…)` link are
    # recognisable, so the fragments read back onto their columns and the groups are the remainder.
    # ⚠️ `ENGINE-UNIVERSAL-SERVICES`, `DOCTRINE-GAP-OWNERSHIP`, `CI-PARITY-GATE-ROT` and `DONE-BAR`
    # were ALSO missing cells outright; those were authored in `-0013` before this pass ran, because
    # this applier only ever escapes and must never invent a column.
    ("docs/TASK_TREE.md", 1, "RGX-0078"): [[3, 4, 5]],
    ("docs/TASK_TREE.md", 1, "DOCTRINE-GAP-OWNERSHIP"): [[1, 2, 3]],
    ("docs/TASK_TREE.md", 1, "CI-PARITY-GATE-ROT"): [[1, 2]],
    ("docs/TASK_TREE.md", 1, "GRAMMAR-WELLFORMED"): [list(range(1, 9)), list(range(10, 14)), [14, 15]],
    ("docs/TASK_TREE.md", 1, "ENGINE-UNIVERSAL-SERVICES"): [list(range(1, 5)), list(range(6, 20))],
    ("docs/TASK_TREE.md", 1, "SV-CORPUS-GRAD"): [list(range(1, 7)), list(range(8, 17)), list(range(17, 22))],
    ("docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md", 1, "REGEX-0105"): [
        [4, 5, 6],
        list(range(10, 23)),
        list(range(23, 28)),
        list(range(28, 35)),
    ],
}


def code_spans(line):
    """CommonMark code spans in `line`, as (content_start, content_end) offsets.

    A backtick run of length N opens a span closed by the NEXT run of exactly length N; an
    unmatched run is literal text and scanning resumes after it.  A backtick preceded by an odd
    number of backslashes is escaped and does not participate.
    """
    runs = []
    i, n = 0, len(line)
    while i < n:
        if line[i] == "`":
            back = 0
            k = i - 1
            while k >= 0 and line[k] == "\\":
                back += 1
                k -= 1
            j = i
            while j < n and line[j] == "`":
                j += 1
            if back % 2 == 0:
                runs.append((i, j))
            i = j
        else:
            i += 1
    spans, k = [], 0
    while k < len(runs):
        s, e = runs[k]
        want = e - s
        m = k + 1
        while m < len(runs) and (runs[m][1] - runs[m][0]) != want:
            m += 1
        if m < len(runs):
            spans.append((e, runs[m][0]))
            k = m + 1
        else:
            k += 1
    return spans


def split_cells(line):
    """Split a table row the way GFM does: on unescaped `|` only.

    `\\|` is a literal pipe inside a cell, never a column break.  Ignoring that is the trap the
    leaf measured: it reads the bug ledger at 94 over-wide instead of 25, a 3.8x inflation, by
    counting the FIX as the defect.
    """
    masked = line.replace("\\|", "\x01")
    parts = masked.split("|")
    if parts and parts[0].strip() == "":
        parts = parts[1:]
    if parts and parts[-1].strip() == "":
        parts = parts[:-1]
    return [p.replace("\x01", "\\|") for p in parts]


def normalise(line):
    """Strip the only edit this tool is allowed to make, so the invariant can compare content."""
    return line.replace("\\|", "|")


def repair(line):
    """Escape every unescaped `|` that falls inside a code span. Returns the new line."""
    out, last = [], 0
    for cs, ce in code_spans(line):
        out.append(line[last:cs])
        seg = line[cs:ce]
        fixed_seg, i = [], 0
        while i < len(seg):
            ch = seg[i]
            if ch == "|":
                back = 0
                k = i - 1
                while k >= 0 and seg[k] == "\\":
                    back += 1
                    k -= 1
                fixed_seg.append("|" if back % 2 == 1 else "\\|")
            else:
                fixed_seg.append(ch)
            i += 1
        out.append("".join(fixed_seg))
        last = ce
    out.append(line[last:])
    return "".join(out)


def join_groups(line, groups):
    """Re-join the fragment groups a human adjudicated, by ESCAPING the separators inside them.

    The only edit is `|` -> `\\|`, exactly as in the mechanical path, so the content-preservation
    invariant applies unchanged and a mis-read grouping shows up as the wrong arity rather than as
    lost text.
    """
    masked = line.replace("\\|", "\x01")
    segs = masked.split("|")
    lead = segs.pop(0) if segs and segs[0].strip() == "" else None
    trail = segs.pop() if segs and segs[-1].strip() == "" else None
    inside = set()
    for g in groups:
        for i in g[1:]:
            inside.add(i)
    out = []
    if lead is not None:
        out.append(lead)
        out.append("|")
    for i, cell in enumerate(segs):
        if i > 0:
            out.append("\\|" if i in inside else "|")
        out.append(cell)
    if trail is not None:
        out.append("|")
        out.append(trail)
    return "".join(out).replace("\x01", "\\|")


def tables(lines):
    """Yield (index, header_lineno, header_cols, [data lineno...]) for each GFM table."""
    found, tbl, prev_pipe = [], None, False
    for idx, line in enumerate(lines):
        is_pipe = line.startswith("|")
        if is_pipe and SEP_ROW.match(line):
            prev_pipe = True
            continue
        if is_pipe and not prev_pipe:
            tbl = {"n": len(found) + 1, "hdr_line": idx, "cols": len(split_cells(line)), "rows": []}
            found.append(tbl)
        elif is_pipe and tbl is not None:
            tbl["rows"].append(idx)
        prev_pipe = is_pipe
    return found


def process(path, apply_fix, show_residual):
    rel = os.path.relpath(path, ROOT)
    with open(path, encoding="utf-8") as fh:
        lines = fh.read().split("\n")
    endorsed, residual, aborted = 0, [], []
    for tbl in tables(lines):
        preds = PREDICATES.get((rel, tbl["n"]), {})
        for ln in tbl["rows"]:
            original = lines[ln]
            have = len(split_cells(original))
            if have <= tbl["cols"]:
                continue
            name0 = split_cells(original)[0].strip().strip("` ")
            groups = HAND_JOINS.get((rel, tbl["n"], name0))
            fixed = join_groups(original, groups) if groups else repair(original)
            if normalise(fixed) != normalise(original):
                aborted.append((ln, "CONTENT CHANGED — the edit was not a pure escape"))
                continue
            cells = split_cells(fixed)
            name = name0
            if len(cells) != tbl["cols"]:
                residual.append((ln, name, have, len(cells), tbl["cols"], "arity %d after repair" % len(cells)))
                continue
            bad = [why for col, (pat, why) in preds.items()
                   if col >= len(cells) or not re.search(pat, cells[col])]
            if bad:
                residual.append((ln, name, have, len(cells), tbl["cols"], "predicate FAILED: " + "; ".join(bad)))
                continue
            lines[ln] = fixed
            endorsed += 1
            print("   %-8s line %-5d %-34s %2d -> %d cols" % ("HAND" if groups else "ENDORSED", ln + 1, name, have, len(cells)))
    print("\n== %s ==" % rel)
    print("  endorsed (mechanically repairable, all three checks pass): %d" % endorsed)
    print("  residual (must be READ — cause (2) or an unreconstructable span): %d" % len(residual))
    for ln, name, have, got, want, why in residual:
        print("     line %-5d %-34s %2d cols (header %d) — %s" % (ln + 1, name, have, want, why))
        if show_residual:
            print("        %s" % lines[ln][:400])
    if aborted:
        print("  ⛔ INVARIANT TRIPPED on %d row(s) — NOTHING WRITTEN for this file:" % len(aborted))
        for ln, why in aborted:
            print("     line %d — %s" % (ln + 1, why))
        return 3
    if apply_fix and endorsed:
        with open(path, "w", encoding="utf-8") as fh:
            fh.write("\n".join(lines))
        print("  WRITTEN: %d row(s) repaired in place." % endorsed)
    elif apply_fix:
        print("  nothing to write.")
    return 0


def main(argv):
    apply_fix = "--apply" in argv
    show_residual = "--residual" in argv
    files = [a for a in argv if not a.startswith("--")]
    if not files:
        files = ["docs/TASK_TREE.md", "docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md"]
    rc = 0
    for f in files:
        p = os.path.join(ROOT, f)
        if not os.path.isfile(p):
            print("table-pipe-repair: %s is unreadable — REPORTED, not skipped silently" % f)
            rc = max(rc, 2)
            continue
        rc = max(rc, process(p, apply_fix, show_residual))
    return rc


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
