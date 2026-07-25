#!/usr/bin/env python3
"""SV-CORPUS-GRAD.3.11 — per-FILE ADJUDICATION-CLASS diff.

The pass-set diff (analyze_corpus_delta.py) proves no regression; this proves
what moved in the GRADUATION accounting. Set-level, not count-level: two rows
swapping classes can leave a count unchanged, so every class is compared as a
SET of relpaths.

Usage: analyze_manifest_delta.py <before.tsv> <after.tsv> [label]
Columns: suite <TAB> relpath <TAB> observed <TAB> expected <TAB> adjudication <TAB> basis
"""
import sys
from collections import defaultdict


def load(path):
    by_class, rows = defaultdict(set), {}
    with open(path) as fh:
        header = next(fh, "")
        for line in fh:
            p = line.rstrip("\n").split("\t")
            if len(p) < 5:
                continue
            key = (p[0], p[1])
            by_class[p[4]].add(key)
            rows[key] = (p[2], p[4])       # observed, adjudication
    return by_class, rows


def main():
    (bc, br), (ac, ar) = load(sys.argv[1]), load(sys.argv[2])
    label = sys.argv[3] if len(sys.argv) > 3 else "lane"
    print("=" * 80)
    print(f"{label} — adjudication-class SET diff (not just counts)")
    print("=" * 80)
    classes = sorted(set(bc) | set(ac))
    print(f"  {'class':<40} {'BEFORE':>7} {'AFTER':>7} {'delta':>6}  set-identical")
    for c in classes:
        b, a = bc.get(c, set()), ac.get(c, set())
        print(f"  {c:<40} {len(b):>7,} {len(a):>7,} {len(a)-len(b):>+6,}  {b == a}")
    print()
    moved = [(k, br[k][1], ar[k][1]) for k in (set(br) & set(ar)) if br[k][1] != ar[k][1]]
    print(f"  rows that CHANGED adjudication class: {len(moved)}")
    for (suite, rel), b, a in sorted(moved):
        print(f"    {b}  ->  {a}")
        print(f"        {suite}/{rel}")
    print()
    for headline in ("divergence:unexplained_rejects_valid",
                     "divergence:unexplained_accepts_invalid"):
        b, a = bc.get(headline, set()), ac.get(headline, set())
        print(f"  {headline}: {len(b)} -> {len(a)}")
        for k in sorted(b - a):
            print(f"      HEALED (left the class):  {k[0]}/{k[1]}")
        for k in sorted(a - b):
            print(f"      ⛔ NEW (entered the class): {k[0]}/{k[1]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
