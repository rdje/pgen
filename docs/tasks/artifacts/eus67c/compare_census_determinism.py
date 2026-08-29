#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.67(c)(iii) — the NO-REGRESSION arm for an instrument change.

⭐ WHY THIS IS THE RIGHT ARM, AND WHY IT IS AN EQUALITY AND NOT A BAND. The lifetime
tape is static storage and atomics: it makes NO heap allocation. So if it is truly
non-invasive, every ALLOCATION-side column the probe reports must be **byte-identical**
to the value the pre-change binary recorded — not close, identical. `-0128` established
that allocation-side figures are deterministic across runs (byte-identical on 11 of 11
while peak RSS moved by a quarter under load), which is exactly what makes an equality
test legitimate here.

⛔ AND THE RSS COLUMN IS EXCLUDED BY NAME, NOT BY SILENCE. Peak RSS is not deterministic
and is contended-machine-sensitive; including it would make this arm fail for reasons
that have nothing to do with the change. It is listed in `NON_DETERMINISTIC` so a reader
can see what was excluded and why, instead of discovering the omission later.

Usage: compare_census_determinism.py <baseline.tsv> <candidate.tsv>
"""
import csv
import sys

# Columns whose value is a property of the machine or the moment, not of the parse.
NON_DETERMINISTIC = {
    "peak_rss_bytes",  # kernel page accounting; moves with machine load (-0128)
}


def main():
    if len(sys.argv) != 3:
        print(__doc__)
        return 2
    base = {r["path"]: r for r in csv.DictReader(open(sys.argv[1]), delimiter="\t")}
    cand = {r["path"]: r for r in csv.DictReader(open(sys.argv[2]), delimiter="\t")}

    only_base = sorted(set(base) - set(cand))
    only_cand = sorted(set(cand) - set(base))
    shared = [p for p in base if p in cand]

    cols = [c for c in next(iter(base.values())).keys() if c not in NON_DETERMINISTIC]
    missing_cols = sorted(set(cols) - set(next(iter(cand.values())).keys()))

    bad = 0
    checked = 0
    for p in sorted(shared):
        for c in cols:
            if c in missing_cols:
                continue
            checked += 1
            if base[p][c] != cand[p][c]:
                bad += 1
                print(f"⛔ {p.rsplit('/', 1)[-1]:34s} {c:24s} baseline={base[p][c]} "
                      f"candidate={cand[p][c]}")

    print(f"\n=== ALLOCATION-SIDE DETERMINISM: {checked} (input, column) cells compared "
          f"over {len(shared)} inputs, {bad} differ ===")
    print(f"    excluded as non-deterministic: {sorted(NON_DETERMINISTIC)}")
    if missing_cols:
        print(f"    ⚠️ columns absent from the candidate (schema changed): {missing_cols}")
    if only_base:
        print(f"    ⚠️ inputs only in the baseline: {[p.rsplit('/', 1)[-1] for p in only_base]}")
    if only_cand:
        print(f"    ⚠️ inputs only in the candidate: {[p.rsplit('/', 1)[-1] for p in only_cand]}")
    # A comparison over zero cells is a PASS by vacuity and must not read as one.
    if checked == 0:
        print("⛔ NOTHING WAS COMPARED — refusing to report this as agreement.")
        return 1
    return 1 if bad or missing_cols or only_base or only_cand else 0


if __name__ == "__main__":
    sys.exit(main())
