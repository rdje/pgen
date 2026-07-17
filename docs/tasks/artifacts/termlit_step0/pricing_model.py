#!/usr/bin/env python3
"""RGX-0078.5.i.11 STEP-0 — terminal-literal per-attempt pricing + falsifiable ceiling.

Method = the `.5.i.8` §5.a bench-minima fit: least-squares of the CURRENT
floor's per-pattern bench minima (the `-0110` land-gate cand column) against
per-pattern covariates:
  E_p = protocol rule-entry counts (corrected spec_step0 3.4 dumps)
  N_p = terminal-literal attempt counts (attempt_census.json, this scout)
plus the independent profile cross-check (RE-PROFILE #15: the terminal-literal
primitive = 14.5% of parse in a ~94.5%-digit_sequence window).

Run from the repo root:

    python3 docs/tasks/artifacts/termlit_step0/pricing_model.py
"""

import itertools
import json
import math

# The -0110 land-gate cand per-pattern best-mins (d3_bench/geomean_analysis.txt)
# — the CURRENT ≈5.22µs floor's own numbers.
BENCH_MIN_NS = {
    "literal_simple": 1375,
    "digit_sequence": 3875,
    "character_class": 16041,
    "alternation": 3167,
    "capture_groups": 11791,
    "url_simple": 2959,
    "email_basic": 3500,
    "anchor_complex": 16792,
}

PATTERNS = sorted(BENCH_MIN_NS)


def load_covariates():
    attempts = json.load(open("docs/tasks/artifacts/termlit_step0/attempt_census.json"))
    entries = {}
    for p in PATTERNS:
        d = json.load(open(f"docs/tasks/artifacts/spec_step0/{p}.entry.json"))
        entries[p] = d["total_entries"]
    return entries, {p: attempts[p]["attempts_total"] for p in PATTERNS}


def lstsq(rows, y):
    """Tiny normal-equations solver (columns = covariates)."""
    n, k = len(rows), len(rows[0])
    ata = [[sum(rows[i][a] * rows[i][b] for i in range(n)) for b in range(k)] for a in range(k)]
    atb = [sum(rows[i][a] * y[i] for i in range(n)) for a in range(k)]
    # Gaussian elimination
    m = [ata[a] + [atb[a]] for a in range(k)]
    for col in range(k):
        piv = max(range(col, k), key=lambda r: abs(m[r][col]))
        m[col], m[piv] = m[piv], m[col]
        for r in range(k):
            if r != col and m[col][col] != 0:
                f = m[r][col] / m[col][col]
                m[r] = [m[r][c] - f * m[col][c] for c in range(k + 1)]
    return [m[a][k] / m[a][a] if m[a][a] else float("nan") for a in range(k)]


def r2(rows, y, coef):
    pred = [sum(c * x for c, x in zip(coef, r)) for r in rows]
    ss_res = sum((a - b) ** 2 for a, b in zip(y, pred))
    mean = sum(y) / len(y)
    ss_tot = sum((a - mean) ** 2 for a in y)
    return 1 - ss_res / ss_tot


def main():
    entries, attempts = load_covariates()
    print("== RGX-0078.5.i.11 pricing model ==")
    print(f"{'pattern':18s} {'T_min(ns)':>9s} {'E(entries)':>10s} {'N(attempts)':>11s}")
    for p in PATTERNS:
        print(f"{p:18s} {BENCH_MIN_NS[p]:9d} {entries[p]:10d} {attempts[p]:11d}")

    y = [BENCH_MIN_NS[p] for p in PATTERNS]

    # Model A: T = cE*E + cN*N (no intercept)
    rows = [[entries[p], attempts[p]] for p in PATTERNS]
    coef = lstsq(rows, y)
    print(f"\nModel A  T = cE*E + cN*N        : cE={coef[0]:7.1f}  cN={coef[1]:7.1f}  R2={r2(rows, y, coef):.3f}")

    # Model B: with intercept
    rows_b = [[1.0, entries[p], attempts[p]] for p in PATTERNS]
    coef_b = lstsq(rows_b, y)
    print(f"Model B  T = c0 + cE*E + cN*N   : c0={coef_b[0]:7.1f}  cE={coef_b[1]:7.1f}  cN={coef_b[2]:7.1f}  R2={r2(rows_b, y, coef_b):.3f}")

    # LOO band on cN (Model A)
    cns = []
    for drop in range(len(PATTERNS)):
        idx = [i for i in range(len(PATTERNS)) if i != drop]
        c = lstsq([rows[i] for i in idx], [y[i] for i in idx])
        cns.append(c[1])
    print(f"LOO band on cN (Model A): min={min(cns):.1f}  max={max(cns):.1f}  (identifiability record — the band IS the result at n=8)")

    # Independent profile cross-check: RE-PROFILE #15 terminal-literal = 14.5% of
    # parse in a ~94.5% digit_sequence window; digit_sequence N=69 attempts.
    ds_min = BENCH_MIN_NS["digit_sequence"]
    per_attempt = 0.145 * ds_min / attempts["digit_sequence"]
    print(f"\nProfile cross-check: 14.5% x digit_sequence T_min {ds_min}ns / {attempts['digit_sequence']} attempts"
          f" = {per_attempt:.1f} ns/attempt all-in (window digit-heavy => directional; traced-vs-bare count"
          f" deltas fold into this band)")

    # Falsifiable ceiling: per-attempt SAVING band (fast path keeps a 1-byte/uN
    # compare + logger_enabled branch ~1-3ns; elides call+memcmp+2 boundary checks
    # + 1 of 2 trace probes).
    print("\nFalsifiable ceiling (geomean impact of saving c_save per attempt; N from the traced census):")
    for label, c_save in (("LOW", 4.0), ("MID", 6.0), ("HIGH", 8.0)):
        logsum = 0.0
        per_pat = []
        for p in PATTERNS:
            save = c_save * attempts[p]
            ratio = (BENCH_MIN_NS[p] - save) / BENCH_MIN_NS[p]
            logsum += math.log(ratio)
            per_pat.append(f"{p.split('_')[0]}:{100 * (1 - ratio):.1f}%")
        g = math.exp(logsum / len(PATTERNS))
        base_geo = math.exp(sum(math.log(v) for v in y) / len(y))
        print(f"  {label:4s} c_save={c_save:.0f}ns/attempt: geomean {base_geo:.0f} -> {base_geo * g:.0f} ns  ({100 * (g - 1):+.1f}%)   per-pattern: {' '.join(per_pat)}")


if __name__ == "__main__":
    main()
