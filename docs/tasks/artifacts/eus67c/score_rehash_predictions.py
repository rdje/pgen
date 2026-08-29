#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.67(c)(iii) — score PREDICTION_REHASH_SPIKE.md.

Derived, not typed. The roster is FIXED and every registered row prints a verdict, so
a row cannot be lost between the prediction and the table that reports it (`-0127`),
and no count here is typed beside the run that produced it (`-0130`: run the check on
every COUNT in the artifact, not on the roster of claims).

The per-input analysis is IMPORTED from `rehash_spike_report.py` rather than
reimplemented, so the scorer and the report cannot disagree about what they measured.

Usage: score_rehash_predictions.py <tape.tsv> <meta.tsv> <classes.tsv> <zero_fill.tsv> <alloc_census.tsv>
"""
import csv
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rehash_spike_report import analyse  # noqa: E402

MB = 1048576.0

# ⛔⛔ THE SET IS NAMED, NOT DERIVED BY EXCLUSION. `-0128` attributed EIGHT inputs and
# REFUSED two by name (`syn_case_items_10000.sv`, `syn_concat_items_10000.sv`: their
# arena chunk multiset is not derivable from the reported capacity). The lifetime tape
# has no such dependency, so it covers TEN — and scoring "5 of 8" against a population
# of ten would be a denominator error dressed as a falsification. The predictions that
# quote `-0128`'s count are scored against `-0128`'s population, BY NAME, and the wider
# figure is reported beside it rather than substituted for it (`-0127`: when you publish
# N findings as a group, verify the GROUPING).
ATTRIBUTED_0128 = (
    "br_gh330.v",
    "comp1000.v",
    "t_math_synmul_mul.v",
    "pinmux_reg_top.sv",
    "spi_device_reg_pkg.sv",
    "syn_netlist_instances_2500.sv",
    "syn_netlist_instances_10000.sv",
    "syn_port_list_10000.sv",
)


def med(v):
    v = sorted(v)
    return v[len(v) // 2] if len(v) % 2 else (v[len(v) // 2 - 1] + v[len(v) // 2]) / 2


def verdict(ok):
    return "CONFIRMED" if ok else "FALSIFIED"


def main():
    if len(sys.argv) != 6:
        print(__doc__)
        return 2
    tape_p, meta_p, class_p, zf_p, census_p = sys.argv[1:6]
    rows, refused, control_checked, control_bad = analyse(tape_p, meta_p, class_p)

    meta = list(csv.DictReader(open(meta_p), delimiter="\t"))
    overflowing = [r["path"].split("/")[-1] for r in meta
                   if r.get("tape_overflow") not in ("0", "NA", None)]
    unmatched = [r["path"].split("/")[-1] for r in meta
                 if r.get("tape_unmatched_frees") not in ("0", "NA", None)]

    everything = [r for r in rows if r["name"] != "floor.sv"]
    body = [r for r in rows if r["name"] in ATTRIBUTED_0128]
    absent = [n for n in ATTRIBUTED_0128 if n not in {r["name"] for r in rows}]
    if absent:
        print(f"⛔ REFUSING TO SCORE: `-0128` inputs missing from this run: {absent}\n")
        return 1
    multi = [r for r in body if r["n_memo"] >= 2]
    over = [r for r in body if r["n_superseded"] >= 1]
    flat = [r for r in body if r["n_memo"] == 1]
    wide_multi = [r for r in everything if r["n_memo"] >= 2]
    wide_over = [r for r in everything if r["n_superseded"] >= 1]
    shares = [r["share"] for r in over]
    ratios_15 = [r for r in body if abs(r["ratio"] - 1.5) < 5e-5]
    ratios_10 = [r for r in flat if abs(r["ratio"] - 1.0) < 5e-5]

    # P8 — buckets per input byte across the attributed body.
    per = [(r["buckets"][0] / r["bytes"], r["name"]) for r in body if r["buckets"]]
    spread = (max(per)[0] / min(per)[0]) if per else 0.0

    # P9 — the zero-fill pricing, median of repeats per (scenario, arm).
    # ⛔ P9 IS WORDED ABOUT THE ISOLATING SHAPE and is scored on it. The clustered and
    # scattered scenarios BRACKET what a real parse would save and are REPORTED, never
    # substituted for the row that was predicted.
    zf = defaultdict(list)
    zf_bytes = {}
    if os.path.exists(zf_p):
        for r in csv.DictReader(open(zf_p), delimiter="\t"):
            if r["peak_rss_bytes"] in ("NA", ""):
                continue
            zf[(r["scenario"], r["arm"])].append(int(r["peak_rss_bytes"]))
            zf_bytes[(r["scenario"], r["arm"])] = int(r["table_bytes"])
    arms = ("resize", "zeroed", "reserve_only")
    have_zf = all(("isolating", a) in zf for a in arms)
    bracket = []
    if have_zf:
        rss_resize, rss_zero, rss_reserve = (med(zf[("isolating", a)]) for a in arms)
        cut = 1.0 - rss_zero / rss_resize
        alloc_same = len({zf_bytes[("isolating", a)] for a in ("resize", "zeroed")}) == 1
        # ⛔ THE CONTROL: reserving without initialising must read FAR below `resize`, or the
        # zero-FILL is not what makes the pages resident and this fix direction is refuted.
        control_separates = rss_resize > rss_reserve * 1.5
        for scen in ("clustered", "scattered"):
            if all((scen, a) in zf for a in arms):
                r0, r1 = med(zf[(scen, "resize")]), med(zf[(scen, "zeroed")])
                bracket.append(f"{scen}: resize {r0/MB:.0f} MB · zeroed {r1/MB:.0f} MB "
                               f"⇒ cut {(1 - r1 / r0) * 100:.1f}%")
    else:
        rss_resize = rss_zero = rss_reserve = 0.0
        cut = 0.0
        alloc_same = False
        control_separates = False

    # P10 — the thin table's written-slot index range.
    census = [r for r in csv.DictReader(open(census_p), delimiter="\t")
              if r.get("thin_armed") == "true"]
    max_distinct = max((int(r["thin_distinct_slots"]) for r in census), default=0)

    print("=== PREDICTION SCORECARD — ENGINE-UNIVERSAL-SERVICES.67(c)(iii) ===")
    print(f"    roster of 10, every row printed; inputs analysed {len(rows)}, "
          f"refused {len(refused)}")
    print(f"    scored population = `-0128`'s EIGHT attributed inputs, by name; the tape "
          f"also covers {len(everything)} body inputs in total")
    print(f"    wider figure, reported beside and not substituted for the scored one: "
          f">= 2 memo tables live at the peak on {len(wide_multi)} of {len(everything)}, "
          f"superseded on {len(wide_over)} of {len(everything)}\n")

    # ⛔⛔ P1 NAMED A CAPACITY, AND THE SENTENCE IS SCORED, NOT THE INTENTION.
    # "the tape never overflows … at threshold 8 KiB / capacity 65 536" was FALSIFIED on the
    # first run (`br_gh330.v`, 185 902 overflowed); the instrument was then resized to 2^20 and
    # the ladder re-run complete. Scoring the re-run as CONFIRMED would be narrowing a
    # prediction after seeing the data, which is how a scorecard stops being a check (`-0128`
    # prediction 8). So the capacity is READ from the run and compared with the registered one.
    PREREGISTERED_CAPACITY = 65536
    caps = {r["tape_capacity"] for r in meta if r["tape_capacity"] not in ("NA", "")}
    cap_as_registered = caps == {str(PREREGISTERED_CAPACITY)}
    reg = []
    reg.append(("1", "tape never overflows at threshold 8 KiB AND capacity 65 536",
                (f"{len(overflowing)} overflowing, {len(unmatched)} with unmatched frees, "
                 f"at capacity {sorted(caps)} — the registered capacity was "
                 f"{PREREGISTERED_CAPACITY}, where it DID overflow (br_gh330.v, 185 902); "
                 f"the instrument was resized and the ladder re-run complete"),
                not overflowing and not unmatched and cap_as_registered))
    reg.append(("2", "CONTROL: tape vs class table, 0 mismatching (input, class) pairs",
                f"{control_bad} mismatched of {control_checked} checked",
                control_bad == 0 and control_checked > 0))
    reg.append(("3", ">= 2 memo tables live at the PEAK instant on exactly 5 of 8 body inputs",
                f"{len(multi)} of {len(body)}",
                len(multi) == 5 and len(body) == 8))
    reg.append(("4", "wherever >= 2 are live, one is SUPERSEDED (succession verified): 5 of 5",
                f"{len(over)} of {len(multi)}",
                len(multi) > 0 and len(over) == len(multi) and len(multi) == 5))
    reg.append(("5", "overlap share of peak_live in [5%,20%] each; median in [12%,17%]",
                (f"min {min(shares)*100:.2f}% · median {med(shares)*100:.2f}% · "
                 f"max {max(shares)*100:.2f}%") if shares else "no overlapping inputs",
                bool(shares) and all(0.05 <= s <= 0.20 for s in shares)
                and 0.12 <= med(shares) <= 0.17))
    reg.append(("6", "memo peak/steady is exactly 1.5000 where superseded, 1.0000 where not",
                f"{len(ratios_15)} of {len(over)} at 1.5000 · "
                f"{len(ratios_10)} of {len(flat)} at 1.0000",
                len(ratios_15) == len(over) and len(ratios_10) == len(flat)))
    reg.append(("7", "`the memo's peak is 1.5x steady` is FALSE ladder-wide: holds on 5, not 8",
                f"holds on {len(ratios_15)} of {len(body)}",
                len(ratios_15) == 5 and len(body) == 8))
    reg.append(("8", "final buckets per input byte spans > 3x, so no single length factor pays",
                (f"{min(per)[0]:.4f} ({min(per)[1]}) .. {max(per)[0]:.4f} ({max(per)[1]}) "
                 f"= {spread:.2f}x") if per else "no data",
                spread > 3.0))
    reg.append(("9", "RESIDENCY (peak RSS): `alloc_zeroed` cuts the table's RSS by >= 85%, "
                "allocation identical",
                (f"ISOLATING: resize {rss_resize/MB:.1f} MB · zeroed {rss_zero/MB:.1f} MB · "
                 f"reserve_only {rss_reserve/MB:.1f} MB ⇒ cut {cut*100:.2f}% · "
                 f"alloc identical={alloc_same} · control separates={control_separates}"
                 + ("".join(f"\n                  BRACKET " + b for b in bracket)))
                if have_zf else "NOT MEASURED — no zero-fill data",
                have_zf and cut >= 0.85 and alloc_same and control_separates))
    reg.append(("10", "max written-slot index < 2^24, so a 24-bit index fits THIS ladder",
                f"max thin_distinct_slots = {max_distinct} vs 2^24 = {1 << 24}",
                0 < max_distinct < (1 << 24)))

    width = max(len(d) for _n, d, _m, _o in reg)
    for n, desc, measured, ok in reg:
        print(f"  [{n:>2s}] {desc:<{width}s}")
        print(f"       measured : {measured}")
        print(f"       verdict  : {verdict(ok)}\n")
    confirmed = sum(1 for *_x, ok in reg if ok)
    print(f"=== {confirmed} of {len(reg)} CONFIRMED ===")
    return 0


if __name__ == "__main__":
    sys.exit(main())
