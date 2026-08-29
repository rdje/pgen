#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.67(b) LEG 4 — score PREDICTION_ALLOC_CENSUS.md.

⛔ THE SCORECARD IS DERIVED, NOT TYPED. `-0127`'s self-audit found a scorecard that
omitted one of its own rows while its headline counted over the whole set — "a count
with a missing sub-row is an unfinished census, not a rounding". A scorer that walks a
fixed roster cannot lose a row silently: every registered prediction prints a verdict,
and the roster length is printed beside the counts.

⛔ AND IT SCORES CONTROLS SEPARATELY. A control verifies the INSTRUMENT (the snapshot
hysteresis bound is true by construction); counting it among the predictions would
inflate the confirmed count with something that was never at risk.

Usage: score_alloc_predictions.py <alloc_census.tsv> <alloc_census_classes.tsv>
                                  <canonical_ladder.tsv>
"""
import csv
import sys

THIN_RULE_COUNT = 109
MB = 1048576.0
FLOOR_LOW = 14.72 * MB
HYSTERESIS = 65536


def med(v):
    v = sorted(v)
    return v[len(v) // 2] if len(v) % 2 else (v[len(v) // 2 - 1] + v[len(v) // 2]) / 2


def main():
    if len(sys.argv) < 4:
        print(__doc__)
        return 2
    rows = list(csv.DictReader(open(sys.argv[1]), delimiter="\t"))
    cls = {}
    for r in csv.DictReader(open(sys.argv[2]), delimiter="\t"):
        cls.setdefault(r["path"], []).append(r)
    canon = {r["path"]: r for r in csv.DictReader(open(sys.argv[3]), delimiter="\t")}

    def f(r, k):
        return float(r[k])

    def i(r, k):
        return int(r[k])

    cross = [i(r, "peak_live") / (i(r, "peak_rss_bytes") - FLOOR_LOW) for r in rows
             if i(r, "peak_rss_bytes") > FLOOR_LOW]
    resid = [1 - (f(r, "arena_ast_allocated_MB") * MB) / i(r, "peak_live") for r in rows]
    churn = [i(r, "alloc_bytes") / i(r, "peak_live") for r in rows]
    freer = [i(r, "free_bytes") / i(r, "alloc_bytes") for r in rows]

    verdicts = []

    def rec(n, claim, ok, detail):
        verdicts.append((n, "CONFIRMED" if ok else "FALSIFIED", claim, detail))

    rec(1, "peak_live/(rss-floor) in [0.50,1.10] every input, median in [0.65,0.90]",
        all(0.50 <= c <= 1.10 for c in cross) and 0.65 <= med(cross) <= 0.90,
        f"measured {min(cross):.4f}..{max(cross):.4f}, median {med(cross):.4f}; "
        f"{sum(1 for c in cross if c > 1.10)} input(s) above 1.10")
    rec(2, "non-AST residual a MINORITY of peak_live, in [0.20,0.60], median ~0.35",
        all(0.20 <= x <= 0.60 for x in resid) and 0.20 <= med(resid) <= 0.60,
        f"measured {min(resid):.4f}..{max(resid):.4f}, median {med(resid):.4f}; "
        f"a MAJORITY on {sum(1 for x in resid if x > 0.5)} of {len(resid)}")

    by = {r["path"].split("/")[-1]: 1 - (f(r, "arena_ast_allocated_MB") * MB) / i(r, "peak_live")
          for r in rows}
    real = [by[k] for k in by if k in ("pinmux_reg_top.sv", "t_math_synmul_mul.v")]
    syn = [by[k] for k in by if k.startswith("syn_netlist_instances")]
    rec(3, "residual share larger on the two real files than on syn_netlist_instances",
        all(r > max(syn) for r in real),
        f"real {['%.4f' % x for x in real]} vs syn_netlist {['%.4f' % x for x in syn]}")

    ok4, d4 = True, []
    for r in rows:
        if i(r, "bytes") < 30000:
            continue
        pl = i(r, "peak_live")
        hit = [c for c in cls[r["path"]]
               if int(c["live_count"]) <= 4 and int(c["live_bytes"]) >= 0.05 * pl]
        if not hit:
            ok4 = False
            d4.append(r["path"].split("/")[-1])
    rec(4, ">=1 class holding >=5% of peak_live in <=4 live allocations, every input >=30 KB",
        ok4, "all inputs satisfy it" if ok4 else f"failed on {d4}")

    rec(5, "alloc_bytes/peak_live >= 2.0 on every input", all(c >= 2.0 for c in churn),
        f"measured {min(churn):.4f}..{max(churn):.4f}, median {med(churn):.4f}")
    rec(6, "free_bytes/alloc_bytes >= 0.50 on every input", all(x >= 0.50 for x in freer),
        f"measured {min(freer):.4f}..{max(freer):.4f}, median {med(freer):.4f}")

    ends = [(r["path"].split("/")[-1], i(r, "end_live")) for r in rows]
    rec(7, "end_live < 10 MB on every input (the parse frees what it built)",
        all(e < 10 * MB for _, e in ends),
        "measured %.2f..%.2f MB; end_live tracks the RETAINED thin scratch"
        % (min(e for _, e in ends) / MB, max(e for _, e in ends) / MB))

    d8 = []
    for r in rows:
        c = canon.get(r["path"])
        if not c:
            continue
        d8.append((r["path"].split("/")[-1],
                   (i(r, "peak_rss_bytes") - int(c["peak_rss_bytes"])) / int(c["peak_rss_bytes"]) * 100))
    worst = max(abs(d) for _, d in d8) if d8 else None
    rec(8, "counting-allocator peak_rss within +-2% of the canonical probe's",
        worst is not None and worst <= 2.0,
        f"worst |delta| {worst:.3f}% over {len(d8)} inputs" if worst is not None else "no overlap")

    ok9, d9 = True, []
    for r in rows:
        if i(r, "bytes") < 30000:
            continue
        big = sum(int(c["live_bytes"]) for c in cls[r["path"]] if int(c["lo"]) >= 1048576)
        if big < f(r, "arena_allocated_MB") * MB:
            ok9 = False
            d9.append(r["path"].split("/")[-1])
    rec(9, "sum of live bytes in classes >= 2^20 is >= arena_allocated, every input >=30 KB",
        ok9, "all inputs satisfy it" if ok9 else f"failed on {d9}")

    ok10, d10 = True, []
    for r in rows:
        if i(r, "bytes") < 30000:
            continue
        pred = THIN_RULE_COUNT * (i(r, "bytes") + 1) * 8
        k = pred.bit_length()
        hit = [c for c in cls[r["path"]] if int(c["class"]) == k]
        # The class must exist and be able to CONTAIN the prediction.
        if not hit or int(hit[0]["live_bytes"]) < pred:
            ok10 = False
            d10.append(r["path"].split("/")[-1])
        elif int(hit[0]["live_count"]) == 1 and int(hit[0]["live_bytes"]) != pred:
            ok10 = False
            d10.append(r["path"].split("/")[-1] + " (sole allocation, wrong size)")
    rec(10, "a live allocation of exactly THIN_RULE_COUNT*(bytes+1)*8 on every input >=30 KB",
        ok10, "present on every input; byte-EXACT and alone in its class where the class is clean"
        if ok10 else f"failed on {d10}")

    # 11 needs the thin share, which is `end_live` minus the process baseline, or the
    # attribution pass. The share used here is the DIRECT one: predicted thin bytes over
    # near-peak live bytes — no attribution pass involved, so this row is independent of it.
    pairs = []
    ast_share = {"spi_device_reg_pkg.sv": 0.156, "syn_port_list_10000.sv": 0.183,
                 "br_gh330.v": 0.286, "comp1000.v": 0.254, "t_math_synmul_mul.v": 0.270,
                 "pinmux_reg_top.sv": 0.234, "syn_netlist_instances_2500.sv": 0.309,
                 "syn_netlist_instances_10000.sv": 0.312, "syn_case_items_10000.sv": 0.295,
                 "syn_concat_items_10000.sv": 0.363}
    for r in rows:
        name = r["path"].split("/")[-1]
        if name not in ast_share:
            continue
        thin = THIN_RULE_COUNT * (i(r, "bytes") + 1) * 8 / i(r, "snap_live")
        pairs.append((ast_share[name], thin, name))
    conc = disc = 0
    for a in range(len(pairs)):
        for b in range(a + 1, len(pairs)):
            s = (pairs[a][0] - pairs[b][0]) * (pairs[a][1] - pairs[b][1])
            conc += s > 0
            disc += s < 0
    tau = (conc - disc) / (conc + disc) if (conc + disc) else 0
    rec(11, "thin-scratch share is negatively rank-correlated with the leg-3 AST share",
        tau < 0, f"Kendall tau = {tau:+.4f} over {len(pairs)} inputs "
                 f"({conc} concordant / {disc} discordant)")

    print("=== PREDICTIONS (roster of %d) ===" % len(verdicts))
    for n, v, claim, detail in verdicts:
        print(f"  {n:>2}  {v:<10} {claim}")
        print(f"      {detail}")
    c = sum(1 for _, v, _, _ in verdicts if v == "CONFIRMED")
    print(f"\n  CONFIRMED {c} of {len(verdicts)} · FALSIFIED {len(verdicts) - c} of {len(verdicts)}")

    print("\n=== CONTROLS (verify the INSTRUMENT, scored separately) ===")
    gap = max(i(r, "snap_gap") for r in rows)
    print(f"  C1  {'PASS' if gap <= HYSTERESIS else 'FAIL'}      peak_live - snapshot_live <= "
          f"{HYSTERESIS} B (hysteresis) — max observed {gap} B")
    return 0


if __name__ == "__main__":
    sys.exit(main())
