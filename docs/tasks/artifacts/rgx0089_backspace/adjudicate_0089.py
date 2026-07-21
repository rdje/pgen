#!/usr/bin/env python3
"""RGX-0089 perf-guardrail adjudication — the -0212/-0214 convention:

- metric: pooled per-cell MINIMA (min of the two sweeps' min_ns per cell,
  fair to both sides), geomean = log-mean over all corpus cells;
- custody gates: each base sweep within +/-3% of the 1,004.4 ns
  floor-of-record AND base1<->base2 within +/-2.5%;
- verdict flips: per-cell actual_parse must be identical base-vs-candidate
  across all 2,189 cells (0 flips required);
- guardrail: candidate corpus MAX (max per-cell pooled min_ns) <= 425,000 ns;
- the geomean delta is RECORDED (this slice is a correctness fix, not a perf
  claim — the standing law records the delta honestly and re-baselines if it
  exceeds the drift family).
"""
import json
import math
import sys

FLOOR_OF_RECORD_NS = 1004.4
SETTLED_CORPUS_MAX_NS = 425_000


def load(path):
    rows = {}
    for line in open(path):
        if line.strip():
            r = json.loads(line)
            rows[r["id"]] = r
    return rows


def geomean(vals):
    return math.exp(sum(math.log(v) for v in vals) / len(vals))


def single_geomean(rows):
    return geomean([int(r["min_ns"]) for r in rows.values()])


def main():
    out = {}
    b1, c1, b2, c2 = (load(f"corpus_{t}.jsonl") for t in ("base1", "cand1", "base2", "cand2"))
    ids = sorted(b1)
    assert set(b1) == set(b2) == set(c1) == set(c2), "cell-id sets differ"

    g_b1, g_b2 = single_geomean(b1), single_geomean(b2)
    g_c1, g_c2 = single_geomean(c1), single_geomean(c2)
    out["base1_geomean"], out["base2_geomean"] = g_b1, g_b2
    out["cand1_geomean"], out["cand2_geomean"] = g_c1, g_c2

    # Custody gates.
    for tag, g in (("base1", g_b1), ("base2", g_b2)):
        dev = (g / FLOOR_OF_RECORD_NS - 1) * 100
        out[f"{tag}_vs_floor_pct"] = dev
        if abs(dev) > 3.0:
            out["custody"] = f"REFUSE: {tag} {g:.1f} deviates {dev:+.2f}% (> +/-3%) from floor {FLOOR_OF_RECORD_NS}"
    b1b2 = (g_b2 / g_b1 - 1) * 100
    out["base1_vs_base2_pct"] = b1b2
    if abs(b1b2) > 2.5:
        out["custody"] = f"REFUSE: base1<->base2 {b1b2:+.2f}% (> +/-2.5%)"
    out.setdefault("custody", "GREEN")

    # Pooled per-cell minima.
    pb = [min(int(b1[i]["min_ns"]), int(b2[i]["min_ns"])) for i in ids]
    pc = [min(int(c1[i]["min_ns"]), int(c2[i]["min_ns"])) for i in ids]
    gb, gc = geomean(pb), geomean(pc)
    out["pooled_base_geomean"] = gb
    out["pooled_cand_geomean"] = gc
    out["pooled_delta_pct"] = (gc / gb - 1) * 100

    # Verdict flips (base vs candidate, both sweeps must agree per side).
    flips = [
        i for i in ids
        if not (b1[i]["actual_parse"] == b2[i]["actual_parse"]
                and c1[i]["actual_parse"] == c2[i]["actual_parse"]
                and b1[i]["actual_parse"] == c1[i]["actual_parse"])
    ]
    out["verdict_flips"] = len(flips)
    out["flip_ids"] = flips[:20]

    cmax = max(pc)
    out["cand_corpus_max_ns"] = cmax
    out["max_guardrail"] = "PASS" if cmax <= SETTLED_CORPUS_MAX_NS else "BREACH"

    for k, v in out.items():
        print(f"{k} = {v!r}" if not isinstance(v, float) else f"{k} = {v}")

    ok = out["custody"] == "GREEN" and out["verdict_flips"] == 0 and out["max_guardrail"] == "PASS"
    print("ADJUDICATION:", "GREEN" if ok else "REFUSE")
    sys.exit(0 if ok else 2)


if __name__ == "__main__":
    main()
