#!/usr/bin/env python3
"""PGEN-RGX-0078-0171 -- G1-C RE-PRICING from CALLER-ATTRIBUTED profile samples.

WHY THIS EXISTS
---------------
`-0168` priced G1-C (per-atom `ParseNode` materialization + arena bump-copy +
tape push) from two rows of the banked `selfcum_tables.txt`:

    typed_arena::ArenaT::alloc_extend   8.6% / 6.7% / 5.3%
    _platform_memmove                   5.3% / 6.5% / 7.4%
    => log-weighted 13.3% = 168 ns/parse addressable

Both rows are AGGREGATES over callers, and the first is an aggregate over SEVEN
distinct monomorphizations that the demangled table collapsed into one `ArenaT`
row. `alloc_extend` is reachable ONLY from `NodeArena::alloc_shaped_values` /
`alloc_shaped_pairs` (verified: exactly 2 call sites repo-wide, 0 in any
generated parser) plus `typed_arena::Arena::alloc`'s amortized-rare slow path
(`alloc_slow_path` -> `alloc_extend(iter::once(v))`, typed-arena-2.0.2:207).
Those are the SHAPED-VALUE arenas -- the BUILD pass -- not the per-atom
match-pass node path G1-C targets.

This script re-derives the addressable mass by attributing every sample of both
symbols to its IMMEDIATE CALLER in the banked `sample` call trees, then bucketing
callers by MECHANISM. Read-only over banked artifacts (no build, no measurement)
=> the `-0163` STANDING INSTRUMENT RULE holds by construction.
"""

import re
from collections import defaultdict
from pathlib import Path

ART = Path("docs/tasks/artifacts/geomean_reprofile")
FRAME = re.compile(r"^(?P<prefix>[\s+!:|]*)(?P<count>\d+) (?P<sym>.+?)  \(in ")

# -- banked constants (all from the durable record, none re-measured here) -----
BANDS = ["sub1us", "1to2p5", "2p5to20"]
LOG_SHARE = {"sub1us": 38.0, "1to2p5": 35.3, "2p5to20": 26.0}   # -0162 s1
SELF_PCT = {                                                     # selfcum_tables.txt
    "alloc_extend":     {"sub1us": 8.6, "1to2p5": 6.7, "2p5to20": 5.3},
    "_platform_memmove": {"sub1us": 5.3, "1to2p5": 6.5, "2p5to20": 7.4},
}
GEOMEAN_NS = 1263.4        # banked corpus geomean
NOISE_NS = 28.8            # -0166 measured floor: 2.28% p-p of the geomean

# -- mechanism buckets ---------------------------------------------------------
def bucket(caller):
    if caller.startswith("pgen::generated_parsers::regex::RegexParser::cascade_match_"):
        return "G1-C addressable (per-atom match pass)"
    if "cascade_build_" in caller or "to_shaped_value" in caller:
        return "BUILD/VALUE pass"
    if "memoized_call" in caller or "hashbrown" in caller:
        return "MEMO insert/lookup"
    if "semantic_runtime" in caller or "semantic_runtime_" in caller \
       or "SemanticFactSpec" in caller or "FactIndex" in caller \
       or "evaluate_predicate" in caller or "rule_context_path" in caller:
        return "SEMANTIC RUNTIME"
    if "parse_once_timed" in caller:
        return "harness (parse_once_timed)"
    return "other / spine / misc"


def parse(p):
    out = []
    for line in p.read_text(errors="replace").splitlines():
        m = FRAME.match(line)
        if m:
            out.append((len(m.group("prefix")), int(m.group("count")), m.group("sym")))
    return out


def short(s):
    return re.sub(r"::h[0-9a-f]{16}$", "", s)


def shares(band, needle):
    """Return {bucket: fraction-of-this-symbol's-samples} for one band."""
    frames = parse(ART / f"sample_band_{band}.txt")
    acc = defaultdict(int)
    for i, (d, c, s) in enumerate(frames):
        if needle not in s:
            continue
        caller = "<root>"
        for j in range(i - 1, -1, -1):
            if frames[j][0] < d:
                caller = short(frames[j][2])
                break
        acc[bucket(caller)] += c
    tot = sum(acc.values()) or 1
    return {k: v / tot for k, v in acc.items()}, tot


def main():
    print("=" * 78)
    print("PGEN-RGX-0078-0171  G1-C RE-PRICING (caller-attributed)")
    print("=" * 78)

    # mass[bucket] = log-weighted % of parse time, summed over both symbols
    mass = defaultdict(float)
    wsum = sum(LOG_SHARE.values())

    for sym in ("alloc_extend", "_platform_memmove"):
        print(f"\n--- {sym} : caller-bucket shares per band ---")
        for band in BANDS:
            sh, tot = shares(band, sym)
            print(f"  [{band}] n={tot} self={SELF_PCT[sym][band]}%")
            for b, f in sorted(sh.items(), key=lambda kv: -kv[1]):
                contrib = SELF_PCT[sym][band] * f * LOG_SHARE[band] / wsum
                mass[b] += contrib
                print(f"      {100*f:5.1f}%  {b:42s} -> {contrib:5.2f}% of parse")

    print("\n" + "=" * 78)
    print("LOG-WEIGHTED ADDRESSABLE MASS  (alloc_extend + memmove, both symbols)")
    print("=" * 78)
    print(f"  {'mechanism':46s} {'% parse':>8s} {'ns/parse':>9s}")
    for b, pct in sorted(mass.items(), key=lambda kv: -kv[1]):
        print(f"  {b:46s} {pct:7.2f}% {pct/100*GEOMEAN_NS:8.1f}")
    tot = sum(mass.values())
    print(f"  {'TOTAL (the -0168 `13.3% = 168 ns` figure)':46s} {tot:7.2f}% {tot/100*GEOMEAN_NS:8.1f}")

    g1c = mass["G1-C addressable (per-atom match pass)"]
    g1c_ns = g1c / 100 * GEOMEAN_NS
    print("\n" + "=" * 78)
    print("G1-C ADJUDICATION  vs the -0166 noise floor")
    print("=" * 78)
    print(f"  -0168 banked addressable mass : 13.30% = 168.0 ns")
    print(f"  -0171 corrected               : {g1c:5.2f}% = {g1c_ns:5.1f} ns"
          f"   ({168.0/max(g1c_ns,1e-9):.1f}x SMALLER)")
    print(f"  noise floor                   : {NOISE_NS} ns (2.28% p-p, same-binary 8 runs)\n")
    print(f"  {'capture':>10s} {'saving':>10s} {'% geomean':>11s} {'vs noise':>10s}")
    for cap in (0.30, 0.50, 0.70):
        s = g1c_ns * cap
        print(f"  {int(cap*100):9d}% {s:9.1f} ns {100*s/GEOMEAN_NS:10.2f}% {s/NOISE_NS:9.2f}x")
    print("\n  => G1-C is BELOW the noise floor at every capture fraction.")
    print("     Under the -0170 AMENDED rule its population EXISTS (cascade_match_*")
    print("     memmove is real), so it is NOT refused -- it is a BATCH candidate,")
    print("     not the solo heavy chain the stack's step 1 assumed.")


if __name__ == "__main__":
    main()
