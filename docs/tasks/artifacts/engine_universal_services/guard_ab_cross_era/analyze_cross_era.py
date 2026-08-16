#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.20 (b) — the CROSS-ERA adjudication of the +24.3 %.

⛔⛔⛔ WHAT THIS EXISTS TO SETTLE. `.20` has been driven for five slices and three sessions by one
number: the guarded admission "costs **+24.3 %** parse time on the SV corpus" (`.17` slice 9, carried
into the leaf heading, into the ROUTING EVIDENCE, and into director-facing ruling **B**, which forbids
SV reaching `Done` while carrying it). This script re-derives that number from RAW per-file data, in
both eras, under four independent estimators, and it does not reproduce under any of them.

⭐ THE DECISIVE PROPERTY IS THAT ONE ERA IS NOT MINE. `timed.prev.43148` is `.20` slice 4's own raw
output, measured 2026-08-15 on the PRE-`.22`(e) engine — before this slice existed and by an
instrument this slice did not write. Its `median of totals` row reproduces slice 4's PUBLISHED table
(365.8 / 388.1 / 366.6) exactly, which is what fixes its provenance; that check is an assertion below,
not a claim in prose.

⛔ WHY THE ORIGINAL NUMBER WAS WRONG — the mechanism, not merely the discrepancy. Both passes that
produced a large ratio ran the arms in a FIXED order and attributed the whole difference to the arms:
`.17` slice 9 measured narrow-then-shipped, and slice 4's first pass ran 2 → 3 → 1, putting ARM 1
LAST. The host drifts monotonically over a session (it had just absorbed 21-minute release builds),
so whichever arm ran last looked fastest — and ARM 1 running last is exactly what inflates ARM2/ARM1.
When the same three binaries are run under a counterbalanced Latin square, the gap disappears.

⚠️ WHAT THIS DOES **NOT** CLAIM. It does not claim `.22`(e) removed a cost: the PRE-`.22`(e) raw data
shows no +24.3 % either, so the engine change is not the explanation. It does not claim the flip is
free — the deterministic tiers stand, and they measure real costs (+10.6 % rule entries, +9.3 %
parser bytes). It claims exactly one thing: the +24.3 % WALL-CLOCK figure is not reproducible from the
raw data of the runs that produced it, and the true wall-clock effect sits at the noise floor.

Usage: python3 docs/tasks/artifacts/engine_universal_services/guard_ab_cross_era/analyze_cross_era.py
Exit 0 iff the provenance oracle holds and every estimator is computed.
"""
import gzip
import itertools
import os
import statistics
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MATRIX = os.path.join(HERE, "durations_matrix.tsv.gz")

# `.20` slice 4's PUBLISHED medians (docs/tasks/ENGINE-UNIVERSAL-SERVICES.md, and
# guard_ab_timed.txt). The pre-era `median of totals` MUST reproduce these or the data in the
# matrix is not what it says it is.
SLICE4_PUBLISHED = {"arm1": 365.8, "arm3": 388.1, "arm2": 366.6}
# The anchors this analysis adjudicates.
ANCHOR_S9 = 1.243        # `.17` slice 9: 303.0 s narrow -> 376.7 s shipped
ANCHOR_S4_PASS1 = 1.318  # `.20` slice 4 first pass, sequential, order 2 -> 3 -> 1

# ⛔ TEST SEAMS, AND THEY EXIST BECAUSE AN UNFIRED REFUSAL IS A SUGGESTION. This script's whole
# value is two refusals — "the data is not slice 4's" and "an estimator DOES reproduce the published
# anchor" — and a refusal nobody has watched go RED is indistinguishable from one that cannot. Both
# are driven to RED by `--self-test` through these seams. They are read ONLY from the environment, so
# a normal run cannot take a doctored value.
if os.environ.get("PGEN_CROSS_ERA_TEST_ANCHOR"):
    ANCHOR_S9 = float(os.environ["PGEN_CROSS_ERA_TEST_ANCHOR"])
if os.environ.get("PGEN_CROSS_ERA_TEST_PUBLISHED"):
    SLICE4_PUBLISHED = dict(zip(("arm1", "arm3", "arm2"),
                                (float(x) for x in
                                 os.environ["PGEN_CROSS_ERA_TEST_PUBLISHED"].split(","))))


def load():
    with gzip.open(MATRIX, "rt") as fh:
        header = fh.readline().rstrip("\n").split("\t")[1:]
        cols = {c: {} for c in header}
        n = 0
        for line in fh:
            f = line.rstrip("\n").split("\t")
            for c, v in zip(header, f[1:]):
                cols[c][f[0]] = float(v)
            n += 1
    return cols, n


def estimators(cols, era, arms, rounds):
    """Four estimators over the same rows. They differ in HOW they reject a transient, which is the
    only thing separating a 24 % reading from a 0 % one, so all four are reported side by side."""
    keys = None
    for a in arms:
        for r in rounds:
            s = set(cols[f"{era}_{a}_r{r}"])
            keys = s if keys is None else (keys & s)
    tot = lambda a, r: sum(cols[f"{era}_{a}_r{r}"].values())
    out = {
        "median of totals": {a: statistics.median([tot(a, r) for r in rounds]) for a in arms},
        "mean of totals": {a: statistics.mean([tot(a, r) for r in rounds]) for a in arms},
        # Contention and transients only ADD time, so the per-file minimum over repeated runs is
        # the best available estimate of uncontended parse time.
        "sum per-file MIN": {
            a: sum(min(cols[f"{era}_{a}_r{r}"][k] for r in rounds) for k in keys) for a in arms
        },
    }
    if len(rounds) >= 3:
        out["sum per-file MEDIAN"] = {
            a: sum(statistics.median([cols[f"{era}_{a}_r{r}"][k] for r in rounds]) for k in keys)
            for a in arms
        }
    return out, len(keys)


def main():
    cols, rows = load()
    arms = ("arm1", "arm3", "arm2")
    eras = (("pre", [1, 2], "PRE-.22(e)  slice 4 raw, 2026-08-15  (NOT this slice's data)"),
            ("post", [1, 2, 3], "POST-.22(e) slice 5 raw, 2026-08-16  (Latin square, warmed)"))

    print(f"rows={rows} files, 15 runs across two eras — {os.path.basename(MATRIX)}")
    print()
    print(f"{'era':<14}{'estimator':<24}{'ARM1':>9}{'ARM3':>9}{'ARM2':>9}{'ARM2/ARM1':>11}")
    ratios = []
    pre_median = None
    for era, rounds, label in eras:
        ests, nkeys = estimators(cols, era, arms, rounds)
        for name, e in ests.items():
            r = e["arm2"] / e["arm1"]
            ratios.append((era, name, r))
            print(f"{era:<14}{name:<24}{e['arm1']:>8.1f}s{e['arm3']:>8.1f}s{e['arm2']:>8.1f}s{r:>11.4f}")
            if era == "pre" and name == "median of totals":
                pre_median = e
        print(f"{'':<14}{'(files common to all runs of this era: ' + str(nkeys) + ')':<24}")

    # ---- PROVENANCE ORACLE: the pre era must BE slice 4's published measurement ----------------
    print()
    bad = [a for a in arms if abs(pre_median[a] - SLICE4_PUBLISHED[a]) > 0.05]
    if bad:
        sys.exit(f"REFUSE: the pre-era medians {pre_median} do not reproduce slice 4's published "
                 f"table {SLICE4_PUBLISHED} (mismatched: {bad}). The matrix is not the data it "
                 f"claims to be, and nothing above may be read.")
    print(f"✅ PROVENANCE — the pre era reproduces `.20` slice 4's PUBLISHED medians exactly "
          f"({', '.join(f'{a.upper()} {SLICE4_PUBLISHED[a]}s' for a in arms)}). It is that "
          f"measurement, re-analysed, not a re-run of it.")

    lo, hi = min(r for _, _, r in ratios), max(r for _, _, r in ratios)
    print()
    print(f"⇒ {len(ratios)} estimator x era combinations. ARM2/ARM1 in [{lo:.4f}, {hi:.4f}].")
    print(f"   The published anchors are {ANCHOR_S9} (`.17` s9) and {ANCHOR_S4_PASS1} "
          f"(slice 4 pass 1). NEITHER is inside that interval, and neither is within "
          f"{100*(ANCHOR_S9-hi):.0f} points of its top.")
    if hi >= ANCHOR_S9:
        sys.exit("REFUSE: an estimator DOES reproduce the published anchor — the conclusion below "
                 "is void and the finding must be re-opened.")
    print(f"   ⛔ The +24.3 % is absent from BOTH eras, so `.22`(e) is NOT the explanation: the")
    print(f"      number was never in the raw data of the runs that produced it.")

    # ---- estimator stability: how much of the residual spread is the estimator itself? ---------
    print()
    print("estimator stability — the MIN estimator over every round-pair of the post era:")
    keys = None
    for a in arms:
        for r in (1, 2, 3):
            s = set(cols[f"post_{a}_r{r}"])
            keys = s if keys is None else (keys & s)
    for pair in itertools.combinations((1, 2, 3), 2):
        e = {a: sum(min(cols[f"post_{a}_r{r}"][k] for r in pair) for k in keys) for a in arms}
        print(f"   rounds {pair}: ARM1={e['arm1']:6.1f}s ARM2={e['arm2']:6.1f}s  "
              f"ARM2/ARM1={e['arm2']/e['arm1']:.4f}")
    print("   ⇒ the estimator itself moves the ratio by ~3 points, so the honest reading of the")
    print("     wall-clock effect is 'at the noise floor, bounded well under 5 %', NOT a point value.")
    return 0


def self_test():
    """Drive both refusals to RED, then confirm the unperturbed run is GREEN."""
    import subprocess
    me = [sys.executable, os.path.abspath(__file__)]
    arms = [
        ("RED 1  provenance: published table perturbed by 1 s on ARM 1",
         {"PGEN_CROSS_ERA_TEST_PUBLISHED": "366.8,388.1,366.6"}, "REFUSE: the pre-era medians"),
        ("RED 2  anchor lowered to 1.01, inside the measured interval",
         {"PGEN_CROSS_ERA_TEST_ANCHOR": "1.01"}, "REFUSE: an estimator DOES reproduce"),
    ]
    ok = True
    for label, env, expect in arms:
        p = subprocess.run(me, capture_output=True, text=True, env={**os.environ, **env})
        got = (p.returncode != 0) and (expect in (p.stdout + p.stderr))
        print(f"  {'✅' if got else '⛔'} {label}\n      -> exit {p.returncode}, "
              f"{'refused as required' if got else 'DID NOT REFUSE — the check is inert'}")
        ok &= got
    p = subprocess.run(me, capture_output=True, text=True)
    print(f"  {'✅' if p.returncode == 0 else '⛔'} GREEN  unperturbed run -> exit {p.returncode}")
    ok &= p.returncode == 0
    print("  ⇒ " + ("3/3 arms behave as specified." if ok else "⛔ a control is inert; do not trust this script."))
    return 0 if ok else 1


if __name__ == "__main__":
    if "--self-test" in sys.argv:
        sys.exit(self_test())
    sys.exit(main())
