#!/usr/bin/env python3
"""RGX-0078.5.j.3 — floor validation for a freshly built regex_perf_probe.

Parses N banked 8-pattern bench round outputs (regex_perf_probe default mode),
computes the geomean-of-best-mins across rounds, and compares it against the
recorded `-0129` floor (3888.1 ns, `docs/tasks/artifacts/dv_bench/land_gate_analysis.txt`)
— the preserved-probe-with-floor-validation method (the `-0118` precedent).
A fresh probe within the tolerance band is a valid corpus-measurement vehicle.

Usage: floor_validate.py ROUND.txt [ROUND.txt ...] [--recorded-ns 3888.1] [--tolerance-pct 2.0]
"""

import math
import sys


def parse_round(path):
    """Extract {pattern: min_ns} from one bench round output."""
    mins = {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            parts = line.split()
            # Data rows: name + 5 integer stat columns + samples count.
            if len(parts) == 7 and not line.startswith("#") and parts[1].isdigit():
                mins[parts[0]] = int(parts[1])
    if not mins:
        sys.exit(f"no bench rows parsed from {path}")
    return mins


def main():
    paths = []
    recorded_ns = 3888.1
    tolerance_pct = 2.0
    args = sys.argv[1:]
    i = 0
    while i < len(args):
        if args[i] == "--recorded-ns":
            i += 1
            recorded_ns = float(args[i])
        elif args[i] == "--tolerance-pct":
            i += 1
            tolerance_pct = float(args[i])
        else:
            paths.append(args[i])
        i += 1
    if not paths:
        sys.exit(__doc__)

    rounds = [parse_round(p) for p in paths]
    names = sorted(rounds[0])
    for r in rounds:
        if sorted(r) != names:
            sys.exit("round outputs disagree on the pattern set")

    best = {n: min(r[n] for r in rounds) for n in names}
    geomean = math.exp(sum(math.log(v) for v in best.values()) / len(best))
    delta_pct = 100.0 * (geomean - recorded_ns) / recorded_ns

    print("=== floor validation (geomean-of-best-mins vs the recorded -0129 floor) ===")
    print(f"rounds={len(rounds)} patterns={len(names)}")
    for n in names:
        print(f"  {n:<18} best-min {best[n]:>8} ns")
    print(f"fresh geomean-of-best-mins = {geomean:.1f} ns")
    print(f"recorded floor             = {recorded_ns:.1f} ns")
    print(f"delta = {delta_pct:+.2f}% (tolerance ±{tolerance_pct:.1f}%)")
    verdict = "VALIDATED" if abs(delta_pct) <= tolerance_pct else "OUT OF BAND — investigate before trusting corpus numbers"
    print(f"verdict: {verdict}")
    sys.exit(0 if abs(delta_pct) <= tolerance_pct else 1)


if __name__ == "__main__":
    main()
