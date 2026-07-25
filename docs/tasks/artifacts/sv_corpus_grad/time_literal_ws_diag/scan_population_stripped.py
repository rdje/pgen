#!/usr/bin/env python3
"""SV-CORPUS-GRAD.3.11 — keyed corpus population, COMMENT/STRING-STRIPPED.

The raw scan over-counts: SV sources discuss time literals in prose. Strip
`//`, `/* */` and double-quoted strings first, then re-key. What survives is the
set of files whose PARSE VERDICT can move under the strict fix.
"""
import os, re, sys, collections

ROOTS = ["stimuli/sv/subs", "stimuli/sv/uvm"]
EXTS = (".sv", ".svh", ".v")
UNIT = r"(?:s|ms|us|ns|ps|fs)"

STRIP = re.compile(r'//[^\n]*|/\*.*?\*/|"(?:[^"\\\n]|\\.)*"', re.S)

def strip(text):
    # keep newlines so line numbers stay meaningful
    return STRIP.sub(lambda m: re.sub(r'[^\n]', ' ', m.group(0)), text)

# (A) footnote 44 — white space at the number<->unit seam, in a time_literal position
FACE_A = re.compile(r"(?:\btimeunit\b|\btimeprecision\b|#{1,2}|[=,(\[]\s*)\s*"
                    r"(\d[\d_]*(?:\.\d[\d_]*)?)([ \t]+|\r?\n[ \t]*)(" + UNIT + r")\b")
# (B1) A.8.4 — exponent real + unit  (illegal: not unsigned_number / fixed_point_number)
FACE_B1 = re.compile(r"\b\d[\d_]*(?:\.\d[\d_]*)?[eE][+-]?\d[\d_]*" + UNIT + r"\b")
# (B2) A.8.4 — based literal + unit
FACE_B2 = re.compile(r"\d*[ \t]*'[sS]?[dDhHoObB][ \t]*[0-9a-fA-FxXzZ?_]+" + UNIT + r"\b")
# control — the tight, LRM-legal form
CTRL = re.compile(r"(?:\btimeunit\b|\btimeprecision\b|#{1,2}|[=,(\[]\s*)\s*"
                  r"\d[\d_]*(?:\.\d[\d_]*)?" + UNIT + r"\b")

hits = collections.defaultdict(list)
nfiles = 0
for root in ROOTS:
    for dirpath, _d, names in os.walk(root):
        for n in names:
            if not n.endswith(EXTS):
                continue
            path = os.path.join(dirpath, n)
            nfiles += 1
            try:
                raw = open(path, encoding="utf-8", errors="replace").read()
            except OSError:
                continue
            text = strip(raw)
            for label, rx in (("A_spaced", FACE_A), ("B1_exponent", FACE_B1),
                              ("B2_based", FACE_B2), ("CTRL_tight", CTRL)):
                for m in rx.finditer(text):
                    line = text.count("\n", 0, m.start()) + 1
                    src = raw.splitlines()[line-1].strip() if line-1 < len(raw.splitlines()) else ""
                    hits[label].append((path, line, src[:90]))

print("files scanned (comment/string-stripped): %d" % nfiles)
for label in ("A_spaced", "B1_exponent", "B2_based", "CTRL_tight"):
    rows = hits[label]
    fs = sorted({r[0] for r in rows})
    print("\n== %s: %d matches in %d files ==" % (label, len(rows), len(fs)))
    if label == "CTRL_tight":
        continue
    for path, line, src in rows:
        print("   %s:%d\n       %s" % (path, line, src))
