#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.61(b) — how deep does REAL SystemVerilog actually nest?

⛔ This oracle deliberately does NOT use the parser. `.61`(a)'s label fires only on the
REJECT path, so "0 of 16 336 files labelled" does NOT mean "0 files contacted the ceiling"
— depth 322 ACCEPTS while recording 847 refusals. And reading the guard's counter needs
`PGEN_REPORT_MEMO_STATS`, which `.56`(b) proved routes the parse off the fused graph.

So the nesting depth is measured from the SOURCE TEXT instead: a lexical scan that tracks
() [] {} depth while skipping strings and comments. It is a lower bound on parser frames
(the parser also descends through the expression cascade), which is the direction that
makes it safe: if the source never nests deeply, the parser never needed the frames.
"""
import sys, os

OPEN, CLOSE = "([{", ")]}"
PAIR = {")": "(", "]": "[", "}": "{"}

def max_depth(src: str):
    depth = maxd = 0
    i, n = 0, len(src)
    stack = []
    while i < n:
        c = src[i]
        if c == "/" and i + 1 < n and src[i + 1] == "/":
            j = src.find("\n", i)
            i = n if j < 0 else j + 1
            continue
        if c == "/" and i + 1 < n and src[i + 1] == "*":
            j = src.find("*/", i + 2)
            i = n if j < 0 else j + 2
            continue
        if c == '"':
            i += 1
            while i < n and src[i] != '"':
                i += 2 if src[i] == "\\" else 1
            i += 1
            continue
        if c in OPEN:
            stack.append(c)
            depth += 1
            maxd = max(maxd, depth)
        elif c in CLOSE:
            # Unbalanced input is common in this corpus (intentionally invalid files);
            # never let it drive the depth negative.
            if stack and stack[-1] == PAIR[c]:
                stack.pop()
                depth -= 1
        i += 1
    return maxd

rows = []
for path in (l.strip() for l in open(sys.argv[1]) if l.strip()):
    try:
        src = open(path, encoding="utf-8", errors="replace").read()
    except OSError:
        continue
    rows.append((max_depth(src), os.path.getsize(path), path))

rows.sort(reverse=True)
with open(sys.argv[2], "w") as fh:
    for d, sz, p in rows:
        fh.write(f"{d}\t{sz}\t{p}\n")

depths = [r[0] for r in rows]
depths_sorted = sorted(depths)
def pct(q):
    return depths_sorted[min(len(depths_sorted) - 1, int(q * len(depths_sorted)))]
print(f"files={len(rows)}  max={max(depths)}  p99={pct(0.99)}  p999={pct(0.999)}  "
      f"median={pct(0.5)}  mean={sum(depths)/len(depths):.1f}")
print("deepest 10:")
for d, sz, p in rows[:10]:
    print(f"  depth={d:4d}  {sz:9d} B  {p}")
