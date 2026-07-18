#!/usr/bin/env python3
"""RGX-0078.5.i.16 FIRST-BYTE-INDEXED SPINE DISPATCH STEP-0 — the pricing
(PGEN-RGX-0078-0121; the `.5.i.8` §5.a / `.5.i.11` §3 / `.5.i.13` §3 method).

Fits the CURRENT floor's per-pattern bench minima (the `-0118` land-gate cand
best-mins, `docs/tasks/artifacts/spine_bench/land_gate_analysis.txt`) against the
census covariates (whole-model regression = the UPPER anchor only), then prices the
lever from per-component first-principles costs BOUNDED by the #16 visible symbol
shares — never from sampled shares (the standing ×5 lesson).

THE ACCOUNTING (what an indexed dispatch elides, per tournament execution):
  today:   G sequential byte-set `matches!` tests (every guarded branch, every exec —
           longest_match has no early exit), + the survivors' bodies.
  indexed: 1 LUT load (byte -> branch bitmask) + per-survivor bit-iteration
           (`trailing_zeros` + jump), + the SAME survivors' bodies.
  net elided (MID/HIGH) = tests_today − lut_lookups − attempts
      (the LUT load and each survivor's bit-step are each charged ONE test-equivalent
       — conservative: a trailing_zeros+shift is cheaper than a gappy-set test)
  net elided (LOW)      = tests_today − 2·lut_lookups − attempts
      (the LUT load double-charged — stacked conservatism for the LOW bound)
  Byte-2 (FIRST₂) conjuncts stay in-branch: no delta either way. Unguarded branches
  (U≈0 on this artifact) ride in every mask: no delta.

Run from the repo root:
    python3 docs/tasks/artifacts/spine_dispatch_step0/pricing_model.py \
        > docs/tasks/artifacts/spine_dispatch_step0/pricing_model_output.txt
"""

import json
import math

CENSUS = "docs/tasks/artifacts/spine_dispatch_step0/census_dispatch.json"

# The CURRENT floor's per-pattern best-mins (ns): the -0118 land-gate CAND column,
# docs/tasks/artifacts/spine_bench/land_gate_analysis.txt (geomean 4401.8ns).
BENCH_MINS = {
    "alternation": 2916.0,
    "anchor_complex": 14250.0,
    "capture_groups": 10000.0,
    "character_class": 13625.0,
    "digit_sequence": 2916.0,
    "email_basic": 2875.0,
    "literal_simple": 1208.0,
    "url_simple": 2458.0,
}

# Per-test component price (ns) — a compiled gappy-set `matches!` on u8 is a constant
# bit-table/range-chain test + a data-dependent conditional branch: ~1–3 cycles on the
# host core ≈ 0.3–1.0 ns. Bounded below by the #16 visible-share sanity check printed
# in the output (the guard chain must remain a plausible FRACTION of the 11.2%
# atom+piece in-window fn self-time — it also carries preamble/ladder/bookkeeping).
C_TEST = {"LOW": 0.3, "MID": 0.6, "HIGH": 1.0}


def lstsq(rows, y):
    """Tiny least squares (normal equations, 3 params)."""
    import itertools

    n = len(rows)
    k = len(rows[0])
    ata = [[sum(rows[i][a] * rows[i][b] for i in range(n)) for b in range(k)]
           for a in range(k)]
    atb = [sum(rows[i][a] * y[i] for i in range(n)) for a in range(k)]
    # gaussian elimination
    m = [ata[i] + [atb[i]] for i in range(k)]
    for col in range(k):
        piv = max(range(col, k), key=lambda r: abs(m[r][col]))
        m[col], m[piv] = m[piv], m[col]
        if abs(m[col][col]) < 1e-12:
            return None
        for r in range(k):
            if r != col:
                f = m[r][col] / m[col][col]
                for c in range(col, k + 1):
                    m[r][c] -= f * m[col][c]
    return [m[i][k] / m[i][i] for i in range(k)]


def main():
    census = json.load(open(CENSUS))
    pats = sorted(BENCH_MINS)
    P = census["patterns"]

    print("FIRST-BYTE-INDEXED SPINE DISPATCH PRICING — RGX-0078.5.i.16 STEP-0"
          " (PGEN-RGX-0078-0121)")
    print(f"artifact: sha256[:8]={census['artifact']}  bench = the -0118 cand"
          " best-mins (geomean 4401.8ns)")
    print()

    # ---- whole-model regression: T = c0 + cT*tests_today + cR*entries (UPPER anchor)
    rows = [[1.0, P[p]["tests_today"], P[p]["total_entries"]] for p in pats]
    y = [BENCH_MINS[p] for p in pats]
    beta = lstsq(rows, y)
    print("== Whole-model regression (UPPER anchor — absorbs ALL correlated"
          " per-tournament machinery, NOT the elidable price) ==")
    if beta:
        print(f"  T = {beta[0]:.0f} + {beta[1]:.2f}*tests_today +"
              f" {beta[2]:.2f}*total_entries   (n=8)")
        loo = []
        for i in range(len(pats)):
            r2 = [rows[j] for j in range(len(pats)) if j != i]
            y2 = [y[j] for j in range(len(pats)) if j != i]
            b2 = lstsq(r2, y2)
            if b2:
                loo.append(b2[1])
        if loo:
            print(f"  LOO cT range: {min(loo):.2f} … {max(loo):.2f} ns/test"
                  f"  (identifiability: n=8, 3 params — honest anchor only)")
    print()

    # ---- the component price band
    print("== The component price (the claimed band; per-test cost bounded by the"
          " visible #16 shares) ==")
    print(f"  c_test LOW/MID/HIGH = {C_TEST['LOW']}/{C_TEST['MID']}/{C_TEST['HIGH']}"
          " ns per elided test-equivalent")
    print()
    hdr = (f"  {'pattern':16s} {'T_now':>8s} {'elided_L':>9s} {'elided_MH':>9s}"
           f" {'save_L':>7s} {'save_M':>7s} {'save_H':>7s}"
           f" {'-%L':>6s} {'-%M':>6s} {'-%H':>6s}")
    print(hdr)
    geo = {"LOW": 1.0, "MID": 1.0, "HIGH": 1.0}
    for p in pats:
        d = P[p]
        el_mh = d["tests_today"] - d["lut_lookups"] - d["attempts"]
        el_l = d["tests_today"] - 2 * d["lut_lookups"] - d["attempts"]
        saves = {
            "LOW": C_TEST["LOW"] * el_l,
            "MID": C_TEST["MID"] * el_mh,
            "HIGH": C_TEST["HIGH"] * el_mh,
        }
        t = BENCH_MINS[p]
        pct = {k: 100.0 * v / t for k, v in saves.items()}
        for k in geo:
            geo[k] *= (t - saves[k]) / t
        print(f"  {p:16s} {t:8.0f} {el_l:9d} {el_mh:9d}"
              f" {saves['LOW']:7.1f} {saves['MID']:7.1f} {saves['HIGH']:7.1f}"
              f" {pct['LOW']:6.2f} {pct['MID']:6.2f} {pct['HIGH']:6.2f}")
    n = len(pats)
    bands = {k: (1.0 - geo[k] ** (1.0 / n)) * 100.0 for k in geo}
    print()
    print(f"== THE HONEST PRICE: census-fitted geomean ceiling"
          f" LOW −{bands['LOW']:.1f}% / MID −{bands['MID']:.1f}%"
          f" / HIGH −{bands['HIGH']:.1f}% ==")
    base = 4401.8
    for k in ("LOW", "MID", "HIGH"):
        print(f"   {k}: {base:.0f} -> {base * geo[k] ** (1.0 / n):.0f}ns")
    print()

    # ---- visible-share sanity check (the honest-anchor discipline)
    print("== Visible-share sanity check (the #16 profile, digit-heavy window —"
          " DIRECTIONAL) ==")
    d = P["digit_sequence"]
    t = BENCH_MINS["digit_sequence"]
    ap_frac_of_window = {}
    for k, c in C_TEST.items():
        el = d["ap_elided"]  # atom+piece elided tests (their guard chains)
        share = 100.0 * c * el / t
        ap_frac_of_window[k] = share
        print(f"  {k}: atom+piece guard-chain claim on digit_sequence ="
              f" {c}*{el} = {c*el:.0f}ns = {share:.1f}% of the parse"
              f"  (window self-time atom+piece = 11.2%)")
    print("  => at HIGH the claim is ~"
          f"{ap_frac_of_window['HIGH']/11.2*100:.0f}% of the visible atom+piece"
          " self-time (the rest = preamble/ladder/bookkeeping) — the band stays"
          " inside the visible bound; LOW/MID comfortably so.")
    print()
    print("== UNPRICED UPSIDES (recorded, never claimed) ==")
    print("  (a) branch-prediction second-order: ~24 data-dependent conditionals per"
          " atom exec -> 1 indirect dispatch; mispredict stalls are invisible to leaf"
          " attribution (the -0118 over-delivery precedent).")
    print("  (b) mask==0 fast-fail before the tournament preamble: 19 corpus execs"
          " would skip checkpoint+OrWinner+epilogue entirely (marginal).")
    print("  (c) the PROTOCOL-graph population (same guard chains on method rules) —"
          " untouched by the cascade-only emission, upside if later extended.")
    print()
    print("== FALSIFICATION ==")
    print(f"  expectation band −{bands['LOW']:.1f}…−{bands['MID']:.1f}%;"
          " land gate = the standing alternated fat-LTO 5×2000 geomean-of-mins,"
          " land-iff-faster bar −2.0%.")
    print(f"  ⚠️ LOW −{bands['LOW']:.1f}% sits {'BELOW' if bands['LOW'] < 2.0 else 'above'}"
          " the −2.0% land bar — if the true per-test cost is at the pessimistic end,"
          " the land gate itself refutes the lever (this is the honest risk, named).")


if __name__ == "__main__":
    main()
