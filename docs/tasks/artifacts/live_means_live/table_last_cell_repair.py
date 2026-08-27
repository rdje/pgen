#!/usr/bin/env python3
"""table_last_cell_repair.py — put the row boundary back where the header says it goes.

Owning leaf: ``LIVE-DOC-CONTAINMENT.11`` (a count is not a schema).

THE DEFECT THIS REPAIRS, AND WHY NO COUNTER CAN SEE IT
-------------------------------------------------------
``TABLE-ARITY-RATCHET`` compares a row's cell COUNT against its header.  A row can hit that
count exactly and still be wrong, because a count says nothing about WHERE the boundaries
fell.  The shape repaired here is the one the leaf measured in its own founding population:

    | Tree | Status | Roadmap lane | Current frontier | LINK  ...more frontier prose... |
                                                        ^^^^ the File column

A later session appended new frontier text to the END of the row instead of into the
``Current frontier`` cell, so the ``File`` column renders as the link followed by an essay.
The row has exactly 5 cells, so the arity arm is GREEN; the substring predicate
``\\]\\(tasks/`` is satisfied, because the link really is *somewhere* in the cell.

MEASURED 2026-08-27, by the RENDERER rather than by a re-implementation: pulldown-cmark
(mdbook, the renderer this project publishes through) emits **4 of 67** ``File`` cells in
``docs/TASK_TREE.md`` carrying non-anchor residue -- 1 368, 415, 1 217 and **54 378**
characters of prose sitting in a column whose header says ``File``.  The 54 378-character
one is ``SV-CORPUS-GRAD``, the most active tree in the index.

THE ONLY EDIT: ONE SEGMENT MOVES, NOTHING IS WRITTEN OR DELETED
---------------------------------------------------------------
The trailing residue is moved from the last cell to the end of the ``Current frontier``
cell and the link stays alone in ``File``.  Source order inside the frontier is preserved:
what stood earlier in the row still stands earlier in the cell.

CONTENT-PRESERVATION INVARIANT (hard, aborts the row)
------------------------------------------------------
Three checks, each of which can fail independently:

  * STRUCTURAL -- every cell but the last two is byte-identical; the new last cell is
    exactly the link; the new frontier cell is the old one plus exactly the old residue.
  * CHARACTER-MULTISET -- the sorted non-whitespace characters of the whole row are
    unchanged.  A reorder cannot hide a deletion from this.
  * ARITY -- the repaired row still lands on its header's declared count, so the repair
    cannot buy a semantic fix with an arity defect.

USAGE
-----
  table_last_cell_repair.py            # dry run: report per row, write nothing
  table_last_cell_repair.py --apply    # write the endorsed rows in place
Exit 0 on a clean dry run or apply; 3 if an invariant tripped (nothing written for that file).
"""
import importlib.util
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))

_spec = importlib.util.spec_from_file_location(
    "table_pipe_repair", os.path.join(HERE, "table_pipe_repair.py"))
_tpr = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_tpr)
split_cells = _tpr.split_cells
tables = _tpr.tables

# A markdown link occupying the START of the cell, and the residue after it.
LEADING_LINK = re.compile(r"^\s*(\[[^\]]+\]\([^)\s]+\))\s*(.*)$", re.S)

# THE ADJUDICATED SET, recorded as DATA rather than applied as a one-off edit -- the same
# discipline `table_pipe_repair.py` uses for its HAND_JOINS, and for the same reason: the
# decision stays re-runnable, reviewable and replayable against git.
#
# Each row was READ before being listed.  In all four the residue is unambiguously frontier
# narration (a leaf id, a status glyph, a session number, a dated finding) and never a second
# file reference, which is what makes `Current frontier` its column.
ROWS = [
    ("docs/TASK_TREE.md", 1, "GRAMMAR-WELLFORMED"),
    ("docs/TASK_TREE.md", 1, "LANG-CAPABILITY-AUDIT"),
    ("docs/TASK_TREE.md", 1, "SV-REPLAY-DEBT"),
    ("docs/TASK_TREE.md", 1, "SV-CORPUS-GRAD"),
]


def charset(s):
    """The row's content as an order-free multiset, so a reorder cannot hide a deletion."""
    return sorted(c for c in s if not c.isspace())


def rebuild(line):
    """Return (new_line, moved_chars) or (None, why) when this row is not the shape."""
    cells = split_cells(line)
    if len(cells) < 2:
        return None, "fewer than 2 cells"
    m = LEADING_LINK.match(cells[-1])
    if not m:
        return None, "the last cell does not START with a markdown link"
    link, residue = m.group(1), m.group(2).strip()
    if not residue:
        return None, "the last cell is already the link alone — nothing to move"
    new = list(cells)
    new[-2] = cells[-2].rstrip() + " " + residue + " "
    new[-1] = " " + link + " "
    lead = "|" if line.lstrip().startswith("|") else ""
    trail = "|" if line.rstrip().endswith("|") else ""
    return lead + "|".join(new) + trail, len(residue)


def process(path, apply_fix):
    rel = os.path.relpath(path, ROOT)
    with open(path, encoding="utf-8") as fh:
        lines = fh.read().split("\n")
    wanted = {(t, n) for f, t, n in ROWS if f == rel}
    if not wanted:
        return 0
    fixed, aborted, skipped = 0, [], []
    for tbl in tables(lines):
        for ln in tbl["rows"]:
            original = lines[ln]
            name = split_cells(original)[0].strip().strip("` ")
            if (tbl["n"], name) not in wanted:
                continue
            new, info = rebuild(original)
            if new is None:
                skipped.append((ln, name, info))
                continue
            if charset(new) != charset(original):
                aborted.append((ln, name, "CHARACTER MULTISET CHANGED — the edit was not a move"))
                continue
            old_cells, new_cells = split_cells(original), split_cells(new)
            if len(new_cells) != tbl["cols"]:
                aborted.append((ln, name, "arity %d after repair, header %d" % (len(new_cells), tbl["cols"])))
                continue
            if old_cells[:-2] != new_cells[:-2]:
                aborted.append((ln, name, "a cell OTHER than the last two changed"))
                continue
            lines[ln] = new
            fixed += 1
            print("   MOVED    line %-5d %-26s %6d chars of frontier prose out of the File cell"
                  % (ln + 1, name, info))
    print("\n== %s ==" % rel)
    print("  repaired: %d" % fixed)
    for ln, name, why in skipped:
        print("  SKIPPED  line %-5d %-26s %s" % (ln + 1, name, why))
    if aborted:
        print("  ⛔ INVARIANT TRIPPED on %d row(s) — NOTHING WRITTEN for this file:" % len(aborted))
        for ln, name, why in aborted:
            print("     line %-5d %-26s %s" % (ln + 1, name, why))
        return 3
    if apply_fix and fixed:
        with open(path, "w", encoding="utf-8") as fh:
            fh.write("\n".join(lines))
        print("  WRITTEN: %d row(s) repaired in place." % fixed)
    elif apply_fix:
        print("  nothing to write.")
    return 0


def main(argv):
    apply_fix = "--apply" in argv
    rc = 0
    for rel in sorted({f for f, _, _ in ROWS}):
        p = os.path.join(ROOT, rel)
        if not os.path.isfile(p):
            print("last-cell-repair: %s is unreadable — REPORTED, not skipped silently" % rel)
            rc = max(rc, 2)
            continue
        rc = max(rc, process(p, apply_fix))
    return rc


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
