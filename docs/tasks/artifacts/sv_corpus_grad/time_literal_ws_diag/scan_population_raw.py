#!/usr/bin/env python3
"""SV-CORPUS-GRAD.3.11 — keyed corpus population for the time_literal tightening.

Counts, over the SAME file universe the external-corpus runner walks, every
source line that would CHANGE verdict under the strict fix:
  Face A  number <white space> time_unit          (IEEE 1800-2017 fn 44)
  Face B1 exponent real  + time_unit              (A.8.4: not unsigned/fixed-point)
  Face B2 based literal  + time_unit              (A.8.4: not unsigned/fixed-point)
Plus the TIGHT control so the zero (if any) is shown to be a real zero.
"""
import os, re, sys, collections

ROOTS = ["stimuli/sv/subs", "stimuli/sv/uvm"]
EXTS = (".sv", ".svh", ".v")
UNIT = r"(?:s|ms|us|ns|ps|fs)"

# Face A — the fn-44 seam, keyed to the three time_literal consumer contexts so a
# bare `10 ns` inside unrelated text is not counted blindly.
FACE_A = re.compile(
    r"(?:\btimeunit\b|\btimeprecision\b|#{1,2}|[=,(\[]\s*)\s*"
    r"(\d[\d_]*(?:\.\d[\d_]*)?)([ \t]+|\r?\n\s*)(" + UNIT + r")\b")
# Face B1 — exponent real immediately followed by a unit.
FACE_B1 = re.compile(r"\b\d[\d_]*(?:\.\d[\d_]*)?[eE][+-]?\d[\d_]*(" + UNIT + r")\b")
# Face B2 — based literal immediately followed by a unit.
FACE_B2 = re.compile(r"\d*[ \t]*'[sS]?[dDhHoObB][ \t]*[0-9a-fA-FxXzZ?_]+(" + UNIT + r")\b")
# Controls — the tight, LRM-legal form (proves the corpus does use time literals).
CTRL = re.compile(r"(?:\btimeunit\b|\btimeprecision\b|#{1,2}|[=,(\[]\s*)\s*"
                  r"\d[\d_]*(?:\.\d[\d_]*)?" + UNIT + r"\b")

def files():
    for root in ROOTS:
        for dirpath, _dirs, names in os.walk(root):
            for n in names:
                if n.endswith(EXTS):
                    yield os.path.join(dirpath, n)

hits = collections.defaultdict(list)
nfiles = 0
for path in files():
    nfiles += 1
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError:
        continue
    for label, rx in (("A_spaced", FACE_A), ("B1_exponent", FACE_B1),
                      ("B2_based", FACE_B2), ("CTRL_tight", CTRL)):
        for m in rx.finditer(text):
            line = text.count("\n", 0, m.start()) + 1
            snippet = text[max(0, m.start() - 20):m.end() + 10].replace("\n", "\\n")
            hits[label].append((path, line, snippet))

print(f"files scanned: {nfiles}")
for label in ("A_spaced", "B1_exponent", "B2_based", "CTRL_tight"):
    rows = hits[label]
    nf = len({r[0] for r in rows})
    print(f"\n== {label}: {len(rows)} matches in {nf} files ==")
    for path, line, snip in rows[:40]:
        print(f"   {path}:{line}  |{snip}|")
    if len(rows) > 40:
        print(f"   ... {len(rows)-40} more")
