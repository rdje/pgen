#!/usr/bin/env python3
"""SV-CORPUS-GRAD.3.11 — per-FILE corpus transition analysis (the `.3.4` LAW).

`.3.4` established the law the hard way: NET per-suite counts can hide losses
(that leaf's net-positive run was masking 2 regressions). The only sound
no-regression proof is a per-FILE pass-set diff. This script is that diff.

Usage:
  analyze_corpus_delta.py <before.tsv> <after.tsv> [label]

results.tsv columns: suite <TAB> verdict <TAB> abspath
Paths are normalized to their `/pgen/`-relative form before joining, because a
baseline captured under a different checkout root would otherwise report every
row as changed (the `.3.8` requirement).
"""
import sys
from collections import Counter


def load(path):
    rows = {}
    with open(path) as fh:
        for line in fh:
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 3:
                continue
            suite, verdict, p = parts[0], parts[1], parts[2]
            key = p.split("/pgen/", 1)[1] if "/pgen/" in p else p
            rows[key] = (suite, verdict)
    return rows


def main():
    before, after = load(sys.argv[1]), load(sys.argv[2])
    label = sys.argv[3] if len(sys.argv) > 3 else "lane"

    print(f"================================================================================")
    print(f"{label} — per-FILE transition analysis (the .3.4 LAW: pass-set diff, NOT net counts)")
    print(f"================================================================================")

    bk, ak = set(before), set(after)
    print(f"  key sets identical : {bk == ak} ({len(bk)} / {len(ak)})")
    if bk != ak:
        print(f"    only-in-before   : {len(bk - ak)}  {sorted(bk - ak)[:5]}")
        print(f"    only-in-after    : {len(ak - bk)}  {sorted(ak - bk)[:5]}")
    common = bk & ak
    print(f"  joined rows        : {len(common)}")
    print()

    cb, ca = Counter(v for _, v in before.values()), Counter(v for _, v in after.values())
    print("  totals:")
    print(f"    {'verdict':<10} {'BEFORE':>8} {'AFTER':>8} {'delta':>7}")
    for v in sorted(set(cb) | set(ca)):
        print(f"    {v:<10} {cb.get(v,0):>8,} {ca.get(v,0):>8,} {ca.get(v,0)-cb.get(v,0):>+7,}")
    tot = sum(ca.values())
    print(f"    {'TOTAL':<10} {sum(cb.values()):>8,} {tot:>8,}")
    if tot:
        print(f"    pass-rate  {100*cb.get('pass',0)/max(1,sum(cb.values())):.1f}% -> {100*ca.get('pass',0)/tot:.1f}%")
    print()

    trans = [(k, before[k][1], after[k][1]) for k in common if before[k][1] != after[k][1]]
    print(f"  ALL per-FILE transitions: {len(trans)}")
    tc = Counter((b, a) for _, b, a in trans)
    for (b, a), n in sorted(tc.items()):
        print(f"    {b:>7} -> {a:<7} {n:>5}")
    print()

    # THE regression gates — any non-zero here is a hard stop.
    regressions = [t for t in trans if t[1] == "pass" and t[2] != "pass"]
    print(f"  ⛔ REGRESSIONS (pass -> anything else): {len(regressions)}")
    for k, b, a in regressions:
        print(f"       pass -> {a:<8} {k}")
    print()
    print(f"  ✅ HEALS (anything -> pass): {len([t for t in trans if t[2]=='pass' and t[1]!='pass'])}")
    for k, b, a in trans:
        if a == "pass" and b != "pass":
            print(f"       {b:>7} -> pass   {k}")
    other = [t for t in trans if t[1] != "pass" and t[2] != "pass"]
    if other:
        print()
        print(f"  ↔ NON-PASS reshuffles (fail<->timeout etc., neither a heal nor a regression): {len(other)}")
        for k, b, a in other:
            print(f"       {b:>7} -> {a:<8} {k}")

    print()
    print("  VERDICT: " + ("NO REGRESSION (0 pass->fail, 0 pass->timeout, 0 pass->crash)"
                           if not regressions else f"⛔ {len(regressions)} REGRESSION(S) — DO NOT COMMIT"))
    return 1 if regressions else 0


if __name__ == "__main__":
    sys.exit(main())
