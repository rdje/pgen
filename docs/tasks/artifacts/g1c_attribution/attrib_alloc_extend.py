#!/usr/bin/env python3
"""G1-C ATTRIBUTION AUDIT (PGEN-RGX-0078-0171).

The `-0168` G1-C pricing took the `selfcum_tables.txt` row
`typed_arena::ArenaT::alloc_extend` (8.6/6.7/5.3% self) as addressable mass for
a PER-ATOM `ParseNode` change. But `alloc_extend` is GENERIC: the raw `sample`
call trees carry SEVEN distinct monomorphizations (distinct `::h<hash>`
suffixes), and the summary table collapsed them into one `ArenaT` row.

`NodeArena` owns FOUR arenas (mod.rs:755) -- nodes / shaped_values /
shaped_pairs / rendered_strings -- and `alloc_extend` is reached from:
  * `alloc_shaped_values`  (build/value pass)   -> Arena<PgenValue>
  * `alloc_shaped_pairs`   (build/value pass)   -> Arena<(&str, PgenValue)>
  * `Arena::alloc`'s SLOW path only (chunk full) -> any arena, amortized-rare
Only the node-arena share is addressable by G1-C.

This script attributes every alloc_extend sample to its IMMEDIATE CALLER by
parsing the macOS `sample` call tree (indent depth = call depth), so the
node-arena share can be separated from the value-arena share.

Read-only over banked artifacts: no build, no measurement, no twin to flip
(the `-0163` STANDING INSTRUMENT RULE holds by construction).
"""

import re
import sys
from collections import defaultdict
from pathlib import Path

ART = Path("docs/tasks/artifacts/geomean_reprofile")
BANDS = ["sub1us", "1to2p5", "2p5to20"]

# A frame line looks like:  "<tree prefix> <count> <symbol>  (in <image>) + <off> [<addr>]"
# The tree prefix is made of the sample(1) tree glyphs; the sample COUNT is the
# first bare integer token, and indent depth is that token's column.
FRAME = re.compile(r"^(?P<prefix>[\s+!:|]*)(?P<count>\d+) (?P<sym>.+?)  \(in ")


def parse(path):
    """Yield (depth, count, symbol) for every frame line, in file order."""
    out = []
    for line in path.read_text(errors="replace").splitlines():
        m = FRAME.match(line)
        if m:
            out.append((len(m.group("prefix")), int(m.group("count")), m.group("sym")))
    return out


def short(sym):
    """Drop the trailing ::h<hash> for readability, keep it for alloc_extend."""
    return re.sub(r"::h[0-9a-f]{16}$", "", sym)


def attribute(frames):
    """For each alloc_extend frame, find its parent = nearest preceding frame
    with strictly smaller indent depth. Returns caller -> {hash: samples}."""
    by_caller = defaultdict(lambda: defaultdict(int))
    by_hash = defaultdict(int)
    for i, (depth, count, sym) in enumerate(frames):
        if "alloc_extend" not in sym:
            continue
        h = sym.split("::")[-1]
        by_hash[h] += count
        caller = "<root>"
        for j in range(i - 1, -1, -1):
            if frames[j][0] < depth:
                caller = short(frames[j][2])
                break
        by_caller[caller][h] += count
    return by_caller, by_hash


def main():
    grand_caller = defaultdict(int)
    for band in BANDS:
        path = ART / f"sample_band_{band}.txt"
        frames = parse(path)
        by_caller, by_hash = attribute(frames)
        total = sum(by_hash.values())
        print(f"\n{'='*78}\nBAND {band}   ({path.name})   alloc_extend samples = {total}")
        print(f"{'-'*78}")
        print("  per-instantiation:")
        for h, n in sorted(by_hash.items(), key=lambda kv: -kv[1]):
            print(f"    {h:22s} {n:6d}  ({100.0*n/total:5.1f}%)")
        print("  per immediate CALLER:")
        rows = sorted(by_caller.items(), key=lambda kv: -sum(kv[1].values()))
        for caller, hashes in rows:
            n = sum(hashes.values())
            grand_caller[caller] += n
            hs = ",".join(sorted(hashes))
            print(f"    {n:6d} ({100.0*n/total:5.1f}%)  {caller}")
            print(f"             [{hs}]")

    print(f"\n{'='*78}\nALL BANDS POOLED -- alloc_extend samples by immediate caller")
    print(f"{'-'*78}")
    tot = sum(grand_caller.values())
    for caller, n in sorted(grand_caller.items(), key=lambda kv: -kv[1]):
        print(f"  {n:6d} ({100.0*n/tot:5.1f}%)  {caller}")
    print(f"  {tot:6d} TOTAL")


if __name__ == "__main__":
    if not ART.is_dir():
        sys.exit(f"run from repo root; missing {ART}")
    main()
