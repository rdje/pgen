#!/usr/bin/env python3
"""RGX-0078.5.i.13 THIN-MEMO SPINE STEP-0 — the pricing fit (PGEN-RGX-0078-0116).

Method: the .5.i.8 §5.a / .5.i.11 §3 bench-minima discipline — least-squares of the
CURRENT floor's per-pattern bench minima (the -0115 land-gate cand column, artifact
4b0d8242, docs/tasks/artifacts/termlit_bench/geomean_analysis.txt) against per-pattern
entry-class covariates from the fresh spine census; identifiability recorded honestly
(n=8, leave-one-out); the #15 profile arithmetic as the independent cross-check; the
saving band built from the per-entry MACHINERY inventory x census counts, never from
sampled shares (the x5 lesson).

Run from the repo root:
    python3 docs/tasks/artifacts/spine_step0/pricing_model.py \
        > docs/tasks/artifacts/spine_step0/pricing_model_output.txt
"""

import json

# The -0115 land-gate cand best-mins (ns) — the CURRENT floor, artifact 4b0d8242.
BENCH_MINS = {
    "literal_simple": 1291,
    "digit_sequence": 3416,
    "character_class": 14000,
    "alternation": 3333,
    "capture_groups": 11625,
    "anchor_complex": 17167,
    "email_basic": 3416,
    "url_simple": 3000,
}

CENSUS = json.load(open("docs/tasks/artifacts/spine_step0/census_spine.json"))
PATTERNS = sorted(BENCH_MINS)


def lstsq(rows, y):
    """Tiny dependency-free least squares via normal equations (n is tiny)."""
    k = len(rows[0])
    ata = [[sum(r[i] * r[j] for r in rows) for j in range(k)] for i in range(k)]
    aty = [sum(r[i] * yy for r, yy in zip(rows, y)) for i in range(k)]
    # Gaussian elimination
    m = [row[:] + [b] for row, b in zip(ata, aty)]
    for col in range(k):
        piv = max(range(col, k), key=lambda r: abs(m[r][col]))
        if abs(m[piv][col]) < 1e-12:
            return None
        m[col], m[piv] = m[piv], m[col]
        for r in range(k):
            if r != col:
                f = m[r][col] / m[col][col]
                m[r] = [a - f * b for a, b in zip(m[r], m[col])]
    return [m[i][k] / m[i][i] for i in range(k)]


def fit(patterns):
    rows, y = [], []
    for p in patterns:
        pp = CENSUS["per_pattern"][p]
        spine = pp["spine"]
        rest = pp["total"] - spine
        rows.append([1.0, float(spine), float(rest)])
        y.append(float(BENCH_MINS[p]))
    return lstsq(rows, y)


def main():
    print("THIN-MEMO SPINE PRICING — RGX-0078.5.i.13 STEP-0 (PGEN-RGX-0078-0116)")
    print(f"census artifact: {CENSUS['artifact_sha8']} (must be 4b0d8242)")
    print()
    print("inputs (pattern / cand-min ns / spine_e / rest_e):")
    for p in PATTERNS:
        pp = CENSUS["per_pattern"][p]
        print(
            f"  {p:16s} {BENCH_MINS[p]:6d} {pp['spine']:5d} {pp['total']-pp['spine']:5d}"
        )
    print()

    print("== Model A: T = c0 + cS*spine_e + cR*rest_e (n=8) ==")
    full = fit(PATTERNS)
    print(f"  full fit: c0={full[0]:.1f} cS={full[1]:.1f} cR={full[2]:.1f} (ns)")
    print("  leave-one-out (identifiability record):")
    cs_vals, cr_vals = [], []
    for drop in PATTERNS:
        sub = [p for p in PATTERNS if p != drop]
        c = fit(sub)
        if c is None:
            print(f"    drop {drop:16s} SINGULAR")
            continue
        cs_vals.append(c[1])
        cr_vals.append(c[2])
        print(f"    drop {drop:16s} c0={c[0]:8.1f} cS={c[1]:7.1f} cR={c[2]:6.1f}")
    print(
        f"  LOO bands: cS [{min(cs_vals):.1f}, {max(cs_vals):.1f}]"
        f"  cR [{min(cr_vals):.1f}, {max(cr_vals):.1f}]"
    )
    print(
        "  READING: cS bundles the ENTIRE spine fn cost (tournament dispatch + guard +"
    )
    print(
        "  memo machinery + child-call overhead) minus what rest_e absorbs — an upper"
    )
    print(
        "  anchor for 'cost at a spine entry', NOT the elidable machinery price."
    )
    print()

    print("== The machinery inventory price (the band's real basis) ==")
    print("per-entry elidable components — each coefficient BOUNDED by its visible #15")
    print("symbol share (the honest-anchor discipline; the 9.1% cascade fn-self time is")
    print("tournament/dispatch code and is NOT claimed by this slice):")
    print("  C1 guard scan (cE, per SPINE ENTRY): check_cycle = O(stack-depth) &str-")
    print("     compare scan per entry; #15 symbol 163/15074 = 1.1% in-window =>")
    print("     ~42ns per digit parse / 16 spine entries ~= 2.7ns/entry visible +")
    print("     invisible push/pop + iterator overhead => cE band 2/3/5 ns/entry;")
    print("     the own-rule-scan redesign retains ~0.5ns (equivalence argument:")
    print("     check_cycle only ever compares SAME-rule frames).")
    print("  C2 insert/rehash amortization (cP, per TOTAL ENTRY — both maps, spine")
    print("     thin-memo AND residual protocol memo, pre-sized at construction):")
    print("     reserve_rehash 1.6% + growth share of HashMap::insert (~0.5 of 1.7%)")
    print("     in-window, spread over ~125 total entries/digit parse => cP band")
    print("     0.3/0.5/0.7 ns/entry.")
    print("  C3 success-segment malloc pair (cM, per COMMITTED SPINE SUCCESS): 2x")
    print("     to_vec (DerivEvent is 16B POD => memcpy) per success; bounded by the")
    print("     ALLOC/FREE bucket share attributable to ~20 of ~75 real mallocs per")
    print("     digit parse (~2.9% of parse) => cM band 5/8/11 ns/success.")
    print()

    corpus = CENSUS["corpus"]
    spine_total = corpus["spine"]
    body_exec = spine_total - corpus["spine_h"]
    succ = corpus["spine_c"]  # committed ~ successes proxy on accepted parses
    print(f"corpus counts: spine entries {spine_total}, body executions {body_exec},")
    print(f"               committed successes {succ}, hits {corpus['spine_h']}")
    print()

    print("== The census-fitted ceiling (geomean over the 8 cand mins) ==")
    print("   save_p = cE*spine_e + cP*total_e + cM*spine_committed")
    for name, (c_entry, c_per_total, c_succ) in {
        "LOW": (2.0, 0.3, 5.0),
        "MID": (3.0, 0.5, 8.0),
        "HIGH": (5.0, 0.7, 11.0),
    }.items():
        geo_base, geo_cand = 1.0, 1.0
        percents = {}
        for p in PATTERNS:
            pp = CENSUS["per_pattern"][p]
            save = (
                c_entry * pp["spine"]
                + c_per_total * pp["total"]
                + c_succ * pp["spine_c"]
            )
            t = BENCH_MINS[p]
            geo_base *= t
            geo_cand *= max(t - save, 1.0)
            percents[p] = 100.0 * save / t
        n = len(PATTERNS)
        gb = geo_base ** (1.0 / n)
        gc = geo_cand ** (1.0 / n)
        print(
            f"  {name:4s} (cE={c_entry:.0f} cP={c_per_total:.1f} cM={c_succ:.0f} ns):"
            f" geomean {gb:.0f} -> {gc:.0f} ns = {100.0*(gc-gb)/gb:+.1f}%"
        )
        tops = sorted(percents.items(), key=lambda kv: -kv[1])[:3]
        print(
            "        top per-pattern: "
            + ", ".join(f"{p} -{v:.1f}%" for p, v in tops)
        )
    print()
    print("cross-check (#15 in-window arithmetic, digit-heavy window, MID):")
    print("  MID digit save = 3*16 + 0.5*125 + 8*10 ~= 190ns / 3416 = 5.6%; the")
    print("  visible elidable symbol budget = check_cycle 1.1 + reserve_rehash 1.6 +")
    print("  insert-growth ~0.5 + to_vec alloc share ~2.9 = ~6.1% => MID is inside the")
    print("  profile bound; LOW (3.5%) comfortably inside; HIGH (8.1%) leans on the")
    print("  invisible inlined shares (push/pop, growth branches) and is the optimistic")
    print("  edge, NOT the expectation. Expectation band = LOW..MID; the fat first-cut")
    print("  bands (cE=5..14/cM=15..35 => -9..-24%) were REJECTED for exceeding the")
    print("  whole measured compound — recorded here per the honest-anchor discipline.")


if __name__ == "__main__":
    main()
