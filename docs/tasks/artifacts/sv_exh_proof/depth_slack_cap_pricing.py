#!/usr/bin/env python3
"""SV-EXH-PROOF.7.4.6.13 — read a `Depth-slack retry census:` line and print the
cumulative success/attempt curve, so a nesting cap is CHOSEN from the pricing rather
than curve-fitted to one number.

⚠️ HONEST BOUND, stated up front: capping changes generation downstream, so the
attempt counts at levels <= cap will NOT stay what they are here. This curve prices
CANDIDATES; only the A/B run measures the outcome.
"""
import re
import sys

LEVEL_RE = re.compile(r"L(\d+):(\d+)/(\d+)@(\d+)")


def parse(line):
    return [
        (int(l), int(s), int(a), int(b)) for l, s, a, b in LEVEL_RE.findall(line)
    ]


def main(path, label):
    line = None
    for raw in open(path):
        if raw.startswith("Depth-slack retry census:"):
            line = raw.strip()
    if line is None:
        print(f"REFUSED {path}: no census line (the instrument did not fire)")
        return 1
    levels = parse(line)
    total_a = sum(a for _, _, a, _ in levels)
    total_s = sum(s for _, s, _, _ in levels)
    deepest_paying = max((l for l, s, _, _ in levels if s > 0), default=None)
    print(f"== {label}  levels={len(levels)} attempts={total_a} successes={total_s} "
          f"deepest_paying_level={deepest_paying} max_budget={levels[-1][3]}")
    print(f"{'cap':>5} {'budget':>7} {'kept succ':>10} {'%':>6} {'attempts kept':>14} {'%':>6}")
    seen_s = seen_a = 0
    marks = set()
    for lvl, s, a, b in levels:
        seen_s += s
        seen_a += a
        if lvl <= 12 or lvl % 10 == 0 or lvl == deepest_paying or lvl == len(levels):
            marks.add(lvl)
    seen_s = seen_a = 0
    for lvl, s, a, b in levels:
        seen_s += s
        seen_a += a
        if lvl in marks:
            print(f"{lvl:>5} {b:>7} {seen_s:>10} {100.0*seen_s/total_s:>5.1f}% "
                  f"{seen_a:>14} {100.0*seen_a/total_a:>5.1f}%")
    return 0


if __name__ == "__main__":
    rc = 0
    for arg in sys.argv[1:]:
        rc |= main(arg, arg)
        print()
    sys.exit(rc)
