#!/usr/bin/env python3
"""PGEN-RGX-0078-0217 — closure confirmation sweep analysis.

Computes the campaign's canonical unrounded corpus geomean (log-mean of
per-cell min_ns over every corpus cell), the corpus MAX, and the verdict
tuple from a sweep JSONL, and compares against the banked floor family.

Self-check mode: run first against the banked -0212 `rerun_cand2.jsonl`,
which must reproduce the banked floor geomean 1003.3049233222944 EXACTLY
(the -0213 derive_band_inputs.py convention) — proving this script computes
the same metric before any fresh number is read.
"""
import json
import math
import sys

# The banked floor family (single-sweep-comparable readings on carrier48_8d392176).
BANKED_SELFCHECK_GEOMEAN = 1003.3049233222944  # -0212 rerun_cand2 == -0213 floor capture
FLOOR_OF_RECORD_NS = 1004.4                    # single-sweep-comparable floor of record
SETTLED_CORPUS_MAX_NS = 425_000                # the #178 re-settled guardrail
MARGINED_BAR_NS = 950.0                        # (geomean + 50) <= 1000 form


def analyze(path):
    rows = [json.loads(line) for line in open(path) if line.strip()]
    logs = [math.log(int(r["min_ns"])) for r in rows]
    geomean = math.exp(sum(logs) / len(logs))
    max_ns = max(int(r["min_ns"]) for r in rows)
    max_id = max(rows, key=lambda r: int(r["min_ns"]))["id"]
    ok = sum(1 for r in rows if r["actual_parse"] == "ok")
    err = len(rows) - ok
    flips = sum(1 for r in rows if r["actual_parse"] != r["expected_parse"])
    return {
        "cells": len(rows),
        "geomean_ns": geomean,
        "max_min_ns": max_ns,
        "max_cell": max_id,
        "parse_ok": ok,
        "parse_err": err,
        "expected_vs_actual_flips": flips,
    }


def main():
    mode, path = sys.argv[1], sys.argv[2]
    res = analyze(path)
    if mode == "selfcheck":
        if f"{res['geomean_ns']!r}" != f"{BANKED_SELFCHECK_GEOMEAN!r}":
            print(
                f"REFUSE: selfcheck geomean {res['geomean_ns']!r} != "
                f"banked {BANKED_SELFCHECK_GEOMEAN!r}"
            )
            sys.exit(2)
        print(f"selfcheck PASS: geomean {res['geomean_ns']!r} == banked; cells={res['cells']}")
        return
    print(f"cells                 = {res['cells']}")
    print(f"geomean_ns (unrounded)= {res['geomean_ns']!r}")
    print(f"corpus MAX (min_ns)   = {res['max_min_ns']}  [{res['max_cell']}]")
    print(f"MAX guardrail 425000  = {'PASS' if res['max_min_ns'] <= SETTLED_CORPUS_MAX_NS else 'BREACH'}")
    print(f"parse ok/err          = {res['parse_ok']}/{res['parse_err']}")
    print(f"expected-vs-actual flips = {res['expected_vs_actual_flips']}")
    g = res["geomean_ns"]
    print(f"vs floor-of-record 1,004.4        : {100.0 * (g - FLOOR_OF_RECORD_NS) / FLOOR_OF_RECORD_NS:+.4f}%")
    print(f"vs banked selfcheck 1003.3049...  : {100.0 * (g - BANKED_SELFCHECK_GEOMEAN) / BANKED_SELFCHECK_GEOMEAN:+.4f}%")
    print(f"raw sub-1us this sweep            : {'YES' if g < 1000.0 else 'NO'}")
    print(f"margined bar (<=950.0)            : {'MET' if g <= MARGINED_BAR_NS else 'NOT MET (recorded: out of designed reach)'}")


if __name__ == "__main__":
    main()
