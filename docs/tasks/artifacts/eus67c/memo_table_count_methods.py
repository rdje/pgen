#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.67(c)(iii) — how many inputs hold >= 2 live memo tables,
answered THREE DIFFERENT WAYS, because the first two disagree.

⛔ THE POINT IS THE DISAGREEMENT, NOT THE NUMBER. `-0128` published *"two success tables
are live at once on 4 OF 8 INPUTS"*. Re-derived from its own tracked artifacts, the count
depends entirely on the IDENTIFICATION METHOD:

  A. STRICT CLASS  — a size class holding exactly ONE live allocation whose byte count is
     exactly a `T=296` hashbrown table. Unambiguous, and a **LOWER BOUND BY CONSTRUCTION**:
     a table sharing its class with anything else is invisible to it. Reads **4**.
  B. ATTRIBUTION   — the subset-sum search in `attribute_alloc_classes.py`, which can name a
     table inside a shared class. Reads **5**.
  C. LIFETIME TAPE — each allocation is its own row, so nothing has to be inferred from a
     class total at all, and the instant asked about is the PEAK rather than the snapshot.

A and B are not in conflict about the data; A is answering a narrower question and its answer
is a floor. Publishing either number without saying which question it answers is what turns a
sound measurement into a wrong sentence. C settles it because it needs no identification
inference — and it is also the only one of the three that can distinguish CO-RESIDENCE from
SUCCESSION, which is what the word "rehash" actually claims.

Usage: memo_table_count_methods.py <alloc_census_classes.tsv> <alloc_attribution.txt> \
                                   [<rehash_tape.tsv> <rehash_meta.tsv> <rehash_classes.tsv>]
"""
import collections
import csv
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

MEMO_T = 296
GROUP_WIDTH = 8  # determined by `-0128` from the large tables, held fixed (see the analyser)

# `-0128`'s attributed population, BY NAME. The tape covers ten; scoring a count that was
# published over eight against ten is a denominator error dressed as a falsification.
ATTRIBUTED_0128 = (
    "br_gh330.v", "comp1000.v", "t_math_synmul_mul.v", "pinmux_reg_top.sv",
    "spi_device_reg_pkg.sv", "syn_netlist_instances_2500.sv",
    "syn_netlist_instances_10000.sv", "syn_port_list_10000.sv",
)


def memo_sizes():
    return {(1 << k) * MEMO_T + (1 << k) + GROUP_WIDTH: (1 << k) for k in range(0, 32)}


def method_a(classes_tsv):
    """STRICT: classes holding exactly one live allocation of an exact memo-table size."""
    sizes = memo_sizes()
    out = collections.defaultdict(list)
    for r in csv.DictReader(open(classes_tsv), delimiter="\t"):
        if int(r["live_count"]) == 1 and int(r["live_bytes"]) in sizes:
            out[r["path"].rsplit("/", 1)[-1]].append(sizes[int(r["live_bytes"])])
    return out


def method_b(attribution_txt):
    """The attribution report's own naming of `hashbrown[T=296B] N buckets` allocations."""
    out = collections.defaultdict(list)
    cur = None
    for ln in open(attribution_txt):
        m = re.match(r"^== (\S+)", ln)
        if m:
            cur = m.group(1)
            out.setdefault(cur, [])
            continue
        m = re.search(r"hashbrown\[T=%dB\] (\d+) buckets" % MEMO_T, ln)
        if m and cur:
            out[cur].append(int(m.group(1)))
    return out


def main():
    if len(sys.argv) not in (3, 6):
        print(__doc__)
        return 2
    a = method_a(sys.argv[1])
    b = method_b(sys.argv[2])
    c = {}
    if len(sys.argv) == 6:
        from rehash_spike_report import analyse
        rows, _refused, _ck, _bad = analyse(sys.argv[3], sys.argv[4], sys.argv[5])
        c = {r["name"]: r for r in rows}

    print(f"{'input':34s} {'A strict':>26s} {'B attribution':>26s} {'C tape @peak':>26s} {'C sup':>6s}")
    counts = {"A": 0, "B": 0, "C": 0}
    for name in ATTRIBUTED_0128:
        va = sorted(a.get(name, []), reverse=True)
        vb = sorted(b.get(name, []), reverse=True)
        rc = c.get(name)
        vc = rc["buckets"] if rc else None
        counts["A"] += len(va) >= 2
        counts["B"] += len(vb) >= 2
        counts["C"] += bool(rc) and rc["n_memo"] >= 2
        print(f"{name:34s} {str(va):>26s} {str(vb):>26s} "
              f"{str(vc) if vc is not None else 'NOT MEASURED':>26s} "
              f"{(str(rc['n_superseded']) if rc else 'NA'):>6s}")
    n = len(ATTRIBUTED_0128)
    print(f"\n  A STRICT CLASS (a LOWER BOUND)        : {counts['A']} of {n}")
    print(f"  B ATTRIBUTION (subset-sum naming)     : {counts['B']} of {n}")
    if c:
        print(f"  C LIFETIME TAPE at the PEAK instant   : {counts['C']} of {n}")
        missing = [k for k in ATTRIBUTED_0128 if k not in c]
        if missing:
            print(f"  ⚠️ C could not measure (tape refused): {missing}")
    else:
        print("  C LIFETIME TAPE                       : not supplied")
    return 0


if __name__ == "__main__":
    sys.exit(main())
