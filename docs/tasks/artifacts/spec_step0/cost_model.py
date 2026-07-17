#!/usr/bin/env python3
"""RGX-0078.5.i.8 STEP-0 — the per-entry protocol cost model at the D2 boundary classes.

Reads the corrected 8-pattern outcome dumps (TOOLBOX 3.5, post-`.5.i.8.t1` probe) and the
CASCADE-PLAN-B boundary classification (census_boundary_extract.json, from
`--report-fusibility-census --fusibility-census-json` on grammars/regex.ebnf), joins them
per pattern, and fits least-squares per-entry cost models against the `-0105` land-gate
per-pattern bench minimums (docs/tasks/artifacts/rep2/geomean_analysis.txt, cand column,
artifact 97545e7d, geomean-of-mins 6110.6 ns).

Everything here is derived from tracked evidence in this directory — re-run with
`python3 cost_model.py` to reproduce cost_model_output.txt byte-for-byte.
"""
import json
import math
import os

HERE = os.path.dirname(os.path.abspath(__file__))

PATTERNS = [
    'alternation', 'anchor_complex', 'capture_groups', 'character_class',
    'digit_sequence', 'email_basic', 'literal_simple', 'url_simple',
]
# per-pattern best-mins (ns) at the shipped ~6.11us floor — rep2 land-gate cand column
BENCH_MIN_NS = {
    'alternation': 3666, 'anchor_complex': 18334, 'capture_groups': 13041,
    'character_class': 19000, 'digit_sequence': 5042, 'email_basic': 4333,
    'literal_simple': 1583, 'url_simple': 3375,
}
# The attackable population (the design's D3 boundary-scanner plan, census-located):
# scanner-shaped PLAN-B sub-roots + residual digit-run rules whose only directives are a
# matched-text @transform or a read-only @predicate (the predicate-preserving tail class).
ATTACK_SUB_ROOTS = {
    'nonzero_digit', 'letter', 'digit', 'capture_name', 'name', 'quant_bound_number_body',
}
ATTACK_RESIDUAL = {
    'quant_bound_number', 'backreference_digits', 'backreference_digits_single',
    'numeric_backreference',
}
ATTACK = ATTACK_SUB_ROOTS | ATTACK_RESIDUAL


def lstsq(rows, y):
    """Ordinary least squares via normal equations + partial-pivot elimination."""
    n, k = len(rows), len(rows[0])
    a = [[sum(rows[i][p] * rows[i][q] for i in range(n)) for q in range(k)] for p in range(k)]
    v = [sum(rows[i][p] * y[i] for i in range(n)) for p in range(k)]
    for col in range(k):
        piv = max(range(col, k), key=lambda r: abs(a[r][col]))
        a[col], a[piv] = a[piv], a[col]
        v[col], v[piv] = v[piv], v[col]
        for r in range(col + 1, k):
            f = a[r][col] / a[col][col]
            for c in range(col, k):
                a[r][c] -= f * a[col][c]
            v[r] -= f * v[col]
    beta = [0.0] * k
    for r in range(k - 1, -1, -1):
        beta[r] = (v[r] - sum(a[r][c] * beta[c] for c in range(r + 1, k))) / a[r][r]
    return beta


def r_squared(rows, y, beta):
    pred = [sum(b * x for b, x in zip(beta, row)) for row in rows]
    ss_res = sum((p - t) ** 2 for p, t in zip(pred, y))
    mean = sum(y) / len(y)
    return 1 - ss_res / sum((t - mean) ** 2 for t in y)


def main():
    out = []

    def emit(line=''):
        out.append(line)
        print(line)

    census = json.load(open(os.path.join(HERE, 'census_boundary_extract.json')))
    plan_b = census['cascade_plan_b']
    sub_roots = set(plan_b['sub_roots'])
    thin = set(plan_b['thin_memo'])
    internal = set(plan_b['internal'])
    residual = set(census['cascade_rules_ineligible'])

    emit('=== PLAN-B boundary-class join over the corrected 8-pattern outcome dumps ===')
    emit(f"{'pattern':<16} {'S(sub)':>6} {'Tn(thin)':>8} {'F(fused)':>8} {'R(resid)':>8} "
         f"{'A(attack)':>9} {'bench ns':>8}")
    per_pattern = []
    for pat in PATTERNS:
        o = json.load(open(os.path.join(HERE, f'{pat}.outcome.json')))
        s = tn = f = r = a = 0
        for rule, n in o['rule_entry_counts'].items():
            if rule in ATTACK:
                a += n
            if rule in sub_roots:
                s += n
            elif rule in thin:
                tn += n
            elif rule in internal:
                f += n
            elif rule in residual:
                r += n
        t = BENCH_MIN_NS[pat]
        per_pattern.append((pat, s, tn, f, r, a, t))
        emit(f'{pat:<16} {s:>6} {tn:>8} {f:>8} {r:>8} {a:>9} {t:>8}')
    tots = [sum(row[i] for row in per_pattern) for i in range(1, 6)]
    emit(f"{'TOTAL':<16} {tots[0]:>6} {tots[1]:>8} {tots[2]:>8} {tots[3]:>8} {tots[4]:>9}")
    emit()

    y = [row[6] for row in per_pattern]

    emit('=== M-pooled: T = c0 + cP*(S+Tn+R) + cF*F  (c0 fixed; STABLE across c0) ===')
    for c0 in (0, 150, 300, 450, 600):
        rows = [[row[1] + row[2] + row[4], row[3]] for row in per_pattern]
        yy = [t - c0 for t in y]
        beta = lstsq(rows, yy)
        emit(f'  c0={c0:>3}: cP={beta[0]:6.1f} cF={beta[1]:5.1f}  R2={r_squared(rows, yy, beta):.4f}')
    emit()

    emit('=== M-attack: T = cA*A + cB*(boundary-A) + cF*F  (through origin) ===')
    rows = [[row[5], (row[1] + row[2] + row[4]) - row[5], row[3]] for row in per_pattern]
    beta = lstsq(rows, y)
    emit(f'  full fit: cA={beta[0]:6.1f} cB={beta[1]:6.1f} cF={beta[2]:5.1f}  '
         f'R2={r_squared(rows, y, beta):.4f}')
    emit('  leave-one-out stability (the honest identifiability record):')
    for skip in range(8):
        rr = [r for i, r in enumerate(rows) if i != skip]
        yy = [t for i, t in enumerate(y) if i != skip]
        b = lstsq(rr, yy)
        emit(f'    -{per_pattern[skip][0]:<16} cA={b[0]:6.1f} cB={b[1]:6.1f} cF={b[2]:5.1f}')
    emit()

    emit('=== The falsifiable ceiling on the land-gate metric (geomean-of-mins) ===')
    emit('  savings model: each attackable entry\'s all-in boundary cost collapses to a')
    emit('  direct-coded scan (+preserved directive tail); per-entry elidable cost scenarios:')
    geo0 = math.exp(sum(math.log(t) for t in y) / 8)
    emit(f'  current geomean-of-mins: {geo0:.1f} ns')
    scenarios = [
        ('LOW   (profile-grounded floor: ~25ns frame-machinery - ~5ns scan)', 20),
        ('MID   (profile/regression reconciliation)', 35),
        ('CENTRAL (regression cA - scan residual: 61 - 6)', 55),
        ('HIGH  (LOO upper band)', 80),
    ]
    for label, save in scenarios:
        ts = [t - row[5] * save for row, t in zip(per_pattern, y)]
        geo = math.exp(sum(math.log(t) for t in ts) / 8)
        emit(f'  {label}: {save}ns/e -> geomean {geo:.1f} ns  ({100 * (geo / geo0 - 1):+.1f}%)')
    emit()
    emit('  per-pattern at CENTRAL (55ns/e):')
    for row, t in zip(per_pattern, y):
        t2 = t - row[5] * 55
        emit(f'    {row[0]:<16} {t:>6} -> {t2:>6.0f}  ({100 * (t2 / t - 1):+.1f}%)')

    with open(os.path.join(HERE, 'cost_model_output.txt'), 'w') as fh:
        fh.write('\n'.join(out) + '\n')


if __name__ == '__main__':
    main()
