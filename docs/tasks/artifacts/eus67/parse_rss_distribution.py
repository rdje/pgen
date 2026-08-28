#!/usr/bin/env python3
"""docs/tasks/artifacts/eus67/parse_rss_distribution.py

ENGINE-UNIVERSAL-SERVICES.67 (a) — the parse-memory ratio's DISTRIBUTION, and the
population it is and is not measurable over.

⛔ WHY THIS REFUSES TO PUBLISH A RATIO FOR MOST OF THE CORPUS. `peak RSS / input bytes`
is only a parse cost once the process FLOOR is removed, and the floor on this host is a
BAND, not a constant: measured 15.08-18.86 MB across real inputs, a ~19 % spread. The
median SV corpus file is 826 bytes. At even 2 400x that file's whole parse cost is ~2 MB
— a quarter of the floor's own uncertainty. So a subtraction there measures the loader.

⛔⛔ AND THE FIRST VERSION OF THIS ANALYSIS GOT THAT WRONG SILENTLY. It computed
`(rss - floor) / bytes` and then dropped non-positive values with `if x > 0`. On the
sub-20 KB body that silently discarded 10.7 % of rows — every one of them a row whose
peak RSS came in BELOW the floor file's own — and biased the surviving distribution
upward. A filter that removes the inconvenient tail of a distribution and says nothing
is not a cleanup; it is the defect. Rows are now EXCLUDED BY A NAMED RULE and the
exclusion is COUNTED in the report.

THE RULE: a row is admissible only when its peak RSS is at least `--dominance` times the
floor (default 2.0), i.e. the parse's own footprint is at least as large as the floor it
is being separated from. Everything else is reported as NOT MEASURABLE, never as a ratio.

FLOOR SENSITIVITY: the report is printed at BOTH ends of the measured floor band, so a
conclusion that depends on which floor was chosen is visible as a disagreement rather
than hidden inside one number.

Usage:
  parse_rss_distribution.py --floor-lo BYTES --floor-hi BYTES [--dominance F] FILE.tsv...
"""
from __future__ import annotations

import argparse
import statistics
import sys


def load(path):
    rows = []
    with open(path) as fh:
        fh.readline()
        for line in fh:
            p = line.rstrip("\n").split("\t")
            if len(p) < 6 or p[5] == "NA":
                continue
            n = int(p[1])
            if n == 0:                       # a zero-byte input has no ratio to report
                continue
            rows.append({"path": p[0], "bytes": n, "verdict": p[2],
                         "real_s": None if p[4] == "NA" else float(p[4]),
                         "rss": int(p[5])})
    return rows


def pct(xs, q):
    xs = sorted(xs)
    if not xs:
        return float("nan")
    i = min(len(xs) - 1, max(0, int(round(q * (len(xs) - 1)))))
    return xs[i]


def report(label, rows, floor, dominance):
    admissible = [r for r in rows if r["rss"] >= dominance * floor]
    excluded = len(rows) - len(admissible)
    below = sum(1 for r in rows if r["rss"] <= floor)
    print(f"\n== {label} ==")
    print(f"  rows                      {len(rows)}")
    print(f"  NOT MEASURABLE            {excluded} "
          f"({100 * excluded / len(rows):.1f}%)  [peak RSS < {dominance:g}x floor]")
    print(f"    of which BELOW the floor{below:>5} "
          f"({100 * below / len(rows):.1f}%)  [a ratio here would be negative]")
    print(f"  admissible                {len(admissible)} "
          f"({100 * len(admissible) / len(rows):.1f}%)")
    if not admissible:
        print("  -> no ratio published for this population.")
        return None
    ratios = [(r["rss"] - floor) / r["bytes"] for r in admissible]
    b = [r["bytes"] for r in admissible]
    print(f"  input bytes   min={min(b)}  median={int(statistics.median(b))}  max={max(b)}")
    print("  ratio  " + "  ".join(
        f"{lab}={pct(ratios, q):.0f}x" for q, lab in
        ((0.0, "min"), (0.25, "p25"), (0.50, "p50"), (0.75, "p75"), (0.90, "p90"), (1.0, "max"))))
    return ratios


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--floor-lo", type=int, required=True)
    ap.add_argument("--floor-hi", type=int, required=True)
    ap.add_argument("--dominance", type=float, default=2.0)
    ap.add_argument("tsv", nargs="+")
    a = ap.parse_args()

    rows = []
    for p in a.tsv:
        rows += load(p)
    if not rows:
        print("no rows", file=sys.stderr)
        return 2

    for floor in (a.floor_lo, a.floor_hi):
        print(f"\n######## FLOOR = {floor / 2**20:.2f} MB "
              f"(dominance bar {a.dominance:g}x) ########")
        report("ALL verdicts", rows, floor, a.dominance)
        for v in ("ACCEPT", "REJECT", "TIMEOUT"):
            sel = [r for r in rows if r["verdict"] == v]
            if sel:
                report(f"{v} only", sel, floor, a.dominance)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
