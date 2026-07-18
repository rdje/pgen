#!/usr/bin/env python3
"""RGX-0078.5.j.4 root-cause scout: depth-ladder corpus for the superlinearity mechanism.

Series:
  flat_N       : "x"*N                      — linear control (per-byte baseline)
  nest_D       : "("*D + "x" + ")"*D        — the line_4674 pure nesting mechanism
  nestbr_D     : nest_D + "\\D"             — nesting + backreference (the exact 4674 shape class)
  look_D       : "(?=" *D + "x" + ")"*D     — nested lookahead (the line_6538 direction)
  lookstar_D   : "(?=.*"*D + "x" + ")"*D    — nested lookahead with .* (6538's inner shape)
  orig_4674    : the corpus cell verbatim   — checkpoint vs the banked 2.14 ms
  orig_6538    : the corpus cell verbatim   — checkpoint vs the banked 91.9 us
  6538_noquant : 6538 without the trailing {26} — is the counted quantifier relevant?
"""
import json, sys

rows = []

def add(rid, pattern):
    rows.append({"id": rid, "pattern": pattern, "expected": {"parse": "ok"}})

for n in (8, 16, 32, 64, 128, 160):
    add(f"flat_{n}", "x" * n)

for d in (1, 2, 4, 8, 10, 16, 20, 32, 40, 56, 64, 80):
    add(f"nest_{d}", "(" * d + "x" + ")" * d)

for d in (10, 20, 40, 80):
    add(f"nestbr_{d}", "(" * d + "x" + ")" * d + "\\" + str(d))

for d in (1, 2, 3, 4, 5, 6, 8):
    add(f"look_{d}", "(?=" * d + "x" + ")" * d)
    add(f"lookstar_{d}", "(?=.*" * d + "x" + ")" * d)

add("orig_4674", "(" * 80 + "x" + ")" * 80 + "\\80")
add("orig_6538", "^(?=.*(?=(([A-Z]).*(?(1)\\1)))(?!.+\\2)){26}")
add("6538_noquant", "^(?=.*(?=(([A-Z]).*(?(1)\\1)))(?!.+\\2))")

out = sys.argv[1]
with open(out, "w") as f:
    for r in rows:
        f.write(json.dumps(r) + "\n")
print(f"wrote {len(rows)} rows to {out}")
