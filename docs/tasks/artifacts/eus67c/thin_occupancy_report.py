#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.67(c)(i) — report thin-memo OCCUPANCY and score the
pre-registration in `PREDICTION_THIN_OCCUPANCY.md`.

⛔ THE DENOMINATOR IS `need_slots`, NOT `capacity_slots`, AND THAT IS THE WHOLE
POINT OF NAMING THE QUANTITY FIRST. The scratch is recycled across parses and never
shrinks, so a small file run after a large one sees a capacity unrelated to its own
cost; dividing by it would understate occupancy by however much bigger the previous
file was. `need_slots` is what THIS parse asked for and what `-0128` measured at 872
bytes per input byte.

⛔ `NOT_MEASURED` IS CARRIED THROUGH, NEVER COERCED TO ZERO. A disarmed census and a
table nothing wrote are different facts.

Usage: thin_occupancy_report.py <alloc_census.tsv>
"""
import csv
import sys

MB = 1048576.0
# The break-even written down in the pre-registration BEFORE the numbers were read:
# a dense slot is 8 B; a hash entry with its control byte and load factor is ~28 B.
SPARSE_BREAK_EVEN = 8.0 / 28.0


def num(s):
    if s in ("NA", "", None, "NOT_MEASURED"):
        return None
    try:
        return float(s)
    except ValueError:
        return None


def med(v):
    v = sorted(v)
    return v[len(v) // 2] if len(v) % 2 else (v[len(v) // 2 - 1] + v[len(v) // 2]) / 2


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    rows = list(csv.DictReader(open(sys.argv[1]), delimiter="\t"))
    if not rows or "thin_occupancy" not in rows[0]:
        print("this TSV predates the .67(c)(i) columns — re-run measure_alloc_census.sh",
              file=sys.stderr)
        return 2

    print("=== THIN-MEMO OCCUPANCY (denominator = need_slots, this parse's own ask) ===")
    print("input\tfile B\tneed_slots\tneed MB\tdistinct\toccupancy\tstores/distinct"
          "\tclears\texact?\tcap/need")
    occ, ratio, big = [], [], []
    exact_rows = 0
    for r in rows:
        name = r["path"].split("/")[-1]
        need = num(r["thin_need_slots"])
        cap = num(r["thin_capacity_slots"])
        dist = num(r["thin_distinct_slots"])
        st = num(r["thin_store_calls"])
        cl = num(r["thin_clear_calls"])
        o = num(r["thin_occupancy"])
        armed = r["thin_armed"]
        if armed != "true" or need is None:
            print(f"{name}\t{r['bytes']}\tNOT MEASURED (armed={armed})")
            continue
        exact = (cl == 0)
        exact_rows += exact
        nb = need * 8
        print("\t".join([
            name, r["bytes"], f"{int(need)}", f"{nb / MB:.2f}", f"{int(dist)}",
            f"{o:.6f}" if o is not None else "NOT_MEASURED",
            f"{st / dist:.2f}" if dist else "n/a",
            f"{int(cl)}", "exact" if exact else "UPPER BOUND",
            f"{cap / need:.2f}" if need else "n/a",
        ]))
        if o is not None:
            occ.append((name, o))
            if int(r["bytes"]) >= 30000:
                big.append((name, o))
        if dist and st is not None:
            ratio.append(st / dist)

    if not occ:
        print("\nno armed rows — was PGEN_REPORT_PARSE_FOOTPRINT set?", file=sys.stderr)
        return 2

    vals = [v for _, v in occ]
    bigv = [v for _, v in big]
    print()
    print("=== BANDS ===")
    print(f"  occupancy, all inputs      : min {min(vals):.6f} · median {med(vals):.6f} "
          f"· max {max(vals):.6f}  (n={len(vals)})")
    if bigv:
        print(f"  occupancy, inputs >= 30 KB : min {min(bigv):.6f} · median {med(bigv):.6f} "
              f"· max {max(bigv):.6f}  (n={len(bigv)})")
    if ratio:
        print(f"  store_calls / distinct     : min {min(ratio):.2f} · median {med(ratio):.2f} "
              f"· max {max(ratio):.2f}")
    print(f"  rows where distinct is EXACT (clear_calls == 0): {exact_rows} of {len(rows)}")

    print()
    print("=== THE RULING THE PRE-REGISTRATION COMMITTED TO BEFORE THE NUMBERS ===")
    print(f"  sparse break-even ~= {SPARSE_BREAK_EVEN:.4f} (8 B dense slot vs ~28 B hash entry)")
    above = [n for n, v in big if v >= SPARSE_BREAK_EVEN]
    below = [n for n, v in big if v < SPARSE_BREAK_EVEN]
    if not above:
        print(f"  EVERY input >= 30 KB is BELOW break-even ({len(below)} of {len(big)}).")
        print("  => the dense table is not paying for itself in memory terms. But a sparse")
        print("     replacement is a SPEED trade, and peak speed is the non-negotiable, so the")
        print("     candidate that is NOT a trade is narrowing the (generation, index) word.")
        if bigv:
            worst = max(bigv)
            print(f"  Headroom if the word narrowed 8 B -> 4 B: the table halves with identical")
            print(f"  O(1) access. Worst measured occupancy {worst:.6f} leaves"
                  f" {(1 - worst) * 100:.2f} % of slots never written.")
    elif not below:
        print(f"  EVERY input >= 30 KB is AT OR ABOVE break-even ({len(above)} of {len(big)}).")
        print("  => the dense table is the RIGHT design and .67(c) should say so and stop.")
    else:
        print(f"  SPLIT: {len(above)} at/above break-even {above}, {len(below)} below {below}.")
        print("  => no single ruling; the answer is shape-dependent and must be reported as such.")
    print()
    print("  ⚠️ Occupancy alone cannot rule: the table's RESIDENCY is still an allocation")
    print("     figure (-0128 named that gap), and a fix that shrinks allocation without")
    print("     shrinking resident bytes moves nothing. That measurement is .67(c)(iii).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
