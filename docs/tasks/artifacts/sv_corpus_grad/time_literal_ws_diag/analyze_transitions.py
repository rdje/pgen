#!/usr/bin/env python3
r"""SV-CORPUS-GRAD.3.11 — BEFORE/AFTER transition table for run_matrix.sh output.

Column-position parsing (the driver emits fixed-width `printf` rows), because a
token-splitting parser silently DROPS every row whose %q-quoted spelling embeds
an escaped space (`10\ ns`, `4\'d10\ ns`, `10\ n\ s`) — which is most of the
rows this leaf is about. The row-count assertions below exist so that failure
mode can never recur unnoticed.
"""
import re, sys

# driver row forms (FIXED-WIDTH printf — parse by COLUMN, never by token split):
#   row     -> '  %-5s %-5s %-14s %-7s %s\n'  => id[2:7] ctx[8:13] spelling[14:28]
#                                                 verdict[29:36] note[36:]
#   src_row -> '  %-5s %-7s %s\n'             => id[2:7] verdict[8:15] note[15:]
IDENT = re.compile(r"[A-F]\d+\Z")
VERDICTS = ("ACCEPT", "REJECT")

def parse(line):
    if len(line) < 16 or not IDENT.match(line[2:7].strip()):
        return None
    ident = line[2:7].strip()
    if line[8:15].strip() in VERDICTS:                       # src_row
        return ident, ("src", "<multi-line source>", line[8:15].strip(), line[15:].strip())
    if len(line) >= 36 and line[29:36].strip() in VERDICTS:  # row
        return ident, (line[8:13].strip(), line[14:28].strip(),
                       line[29:36].strip(), line[36:].strip())
    return None                                              # header / prose line

def rows(path):
    out = {}
    for line in open(path):
        r = parse(line.rstrip("\n"))
        if r:
            out[r[0]] = r[1]
    return out

EXPECTED = {  # group -> (title, expected transition, expected row count)
    "A": ("MUST ACCEPT (LRM-legal)",          ("ACCEPT", "ACCEPT"), 13),
    "B": ("MUST REJECT (footnote 44)",        ("ACCEPT", "REJECT"),  7),
    "C": ("MUST REJECT (A.8.4 number class)", ("ACCEPT", "REJECT"),  7),
    "D": ("MUST STAY REJECTED (guard)",       ("REJECT", "REJECT"),  5),
    "E": ("REAL-WORLD LEGAL CODE",            ("REJECT", "ACCEPT"),  5),  # E5 is the control
    "F": ("NEIGHBOUR (1step)",                ("ACCEPT", "ACCEPT"),  1),
}
CONTROLS = {"E5"}  # rows inside a group that are deliberately NOT expected to move

b, a = rows(sys.argv[1]), rows(sys.argv[2])
print("SV-CORPUS-GRAD.3.11 — BEFORE -> AFTER TRANSITIONS")
print("=" * 80)
print("Both columns produced by the SAME driver (run_matrix.sh), re-run rather than")
print("re-typed. BEFORE probe 2026-07-25 19:56 (pre-regen); AFTER probe 2026-07-26 00:20.")
print()
missing = set(b) ^ set(a)
if missing:
    sys.exit("FATAL: row sets differ between before/after: %s" % sorted(missing))

failures = []
for g, (title, (eb, ea), n_exp) in EXPECTED.items():
    ids = sorted((k for k in b if k[0] == g), key=lambda s: int(s[1:]))
    if len(ids) != n_exp:
        failures.append("group %s: parsed %d rows, driver emits %d — PARSER DROPPED ROWS"
                        % (g, len(ids), n_exp))
    print("-" * 80)
    print("%s — %s   (expected %s -> %s)" % (g, title, eb, ea))
    print("-" * 80)
    for i in ids:
        ctx, spell, vb, note = b[i]
        va = a[i][2]
        want = (vb, vb) if i in CONTROLS else (eb, ea)
        ok = (vb, va) == want
        if not ok:
            failures.append("row %s: %s -> %s, expected %s -> %s" % (i, vb, va, *want))
        mark = "  <== TRANSITION" if vb != va else "  (unchanged)"
        print("  %-5s %-5s %-16s %-6s -> %-6s%s%s"
              % (i, ctx, spell, vb, va, mark, "" if ok else "   *** UNEXPECTED ***"))
    print()

print("=" * 80)
print("TALLY — computed from the parsed table, never hand-written")
print("=" * 80)
for g, (title, _t, _n) in EXPECTED.items():
    ids = [k for k in b if k[0] == g]
    per = {}
    for i in ids:
        per[(b[i][2], a[i][2])] = per.get((b[i][2], a[i][2]), 0) + 1
    print("  group %s (%2d rows): %s" % (g, len(ids),
          ", ".join("%s -> %s x%d" % (x, y, n) for (x, y), n in sorted(per.items()))))
changed = sum(1 for i in b if b[i][2] != a[i][2])
print()
print("  rows parsed   : %d   (driver emits %d)" % (len(b), sum(n for _t, _x, n in EXPECTED.values())))
print("  rows changed  : %d" % changed)
print("  rows unchanged: %d" % (len(b) - changed))
print()
print("=" * 80)
print("VERDICT: %s" % ("ALL ROWS AS DESIGNED" if not failures else "FAILURES PRESENT"))
print("=" * 80)
for f in failures:
    print("  *** %s" % f)
if not failures:
    print("  * 13/13 MUST-ACCEPT rows untouched            -> no legal spelling lost")
    print("  *  7/7  footnote-44 rows ACCEPT -> REJECT     -> the seam is closed")
    print("  *  7/7  A.8.4 rows      ACCEPT -> REJECT      -> the number class is pinned")
    print("  *  5/5  guard rows still REJECT               -> not over-tightened")
    print("  *  4/4  real-world legal shapes REJECT -> ACCEPT -> the theft is gone")
    print("  *  E5 (same shapes, NON-unit identifier) ACCEPT in both -> flip is unit-specific")
    print("  *  F1 1step unaffected                        -> no collateral damage")
sys.exit(1 if failures else 0)
