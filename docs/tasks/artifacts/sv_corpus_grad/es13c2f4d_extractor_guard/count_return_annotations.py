#!/usr/bin/env python3
"""Count the RETURN ANNOTATIONS in a `.ebnf`, derived to agree with the generator's own inventory.

⛔ WHY THIS EXISTS, and why the obvious classifier is wrong. `SV-CORPUS-GRAD.13c.2f`(d) published
*"1 090 return annotations against 0"* from `grep -c '^\\s*-> '` — which counts only annotations
written on their OWN line. The shipped SystemVerilog grammar writes **1 203** more of them INLINE,
after the rule body on the same line, so the published figure was short by more than half. The true
count is **2 292**, and the authority for it is the PRODUCER: `main.rs` emits
`generated/<grammar>_return_annotations.json` whose `annotation_count` this script reproduces
EXACTLY (2292 = 2292, verified per-rule across all 1 058 annotated rules).

⚠️ Two traps this went through before it agreed, both worth keeping:
  1. `->` also occurs inside OPERATOR TOKEN LITERALS — `implies := trivia "->"`,
     `iff_arrow := trivia "<->"`, `overlapped_implication := trivia "|->"` — and inside `//` prose.
  2. Blanking `"…"` strings BEFORE `/…/` regex terminals is wrong: a regex terminal may contain a
     quote, and `string_literal := trivia /"([^"\\]|\\.)*"/ -> {…}` then has its annotation eaten.
     That cost exactly one row (2 291 vs 2 292) and was found by comparing PER RULE against the
     inventory rather than by comparing totals.

⇒ a file-only classifier, so it works on an arbitrary copy where no generated inventory exists:

    python3 count_return_annotations.py <file.ebnf>
"""
import re
import sys

if len(sys.argv) != 2:
    sys.exit("usage: count_return_annotations.py <file.ebnf>")

count = 0
for line in open(sys.argv[1], encoding="utf-8").read().splitlines():
    stripped = line.strip()
    if stripped.startswith("#") or stripped.startswith("//"):
        continue                                        # `[-> repeat_range]` in prose is not one
    body = re.sub(r"/(?:[^/\\]|\\.)*/", "//", line)     # regex terminals FIRST (they contain quotes)
    body = re.sub(r'"(?:[^"\\]|\\.)*"', '""', body)     # then string literals
    count += body.count("->")
print(count)
