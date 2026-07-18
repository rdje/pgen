#!/usr/bin/env python3
"""RGX-0078.5.j.3 — corpus-max distribution baseline analyzer.

Reads the per-case JSONL emitted by `regex_perf_probe --corpus-jsonl … --out-jsonl …`
and prints the distribution report for the REDEFINED closure bar (PGEN-RGX-0078-0127):

  MAX observable parse time < 1µs on the PCRE2 external corpus, read per the
  recorded recommendation as: absolute < 1µs for every cell of ≤ p99 pattern
  size, and the same implied throughput (≤ ~8 ns/byte) for the > p99 tail.

The per-cell statistic is min_ns (the noise-free best observed parse time — the
campaign's standing steering statistic); p50_ns is reported alongside as the
steady-state-typical view. Both the strict-literal reading (corpus-wide absolute
max) and the recommendation reading are reported honestly.

Usage: analyze_corpus_times.py TIMES.jsonl [--bar-ns 1000] [--tail-ns-per-byte 8.0]
"""

import json
import math
import sys

TRACKED_CELLS = (
    "pcre2:testdata/testinput2:line_878",
    "pcre2:testdata/testinput2:line_881",
    "pcre2:testdata/testinput2:line_4674",
)


def percentile(sorted_vals, p):
    """Nearest-rank percentile, same convention as the probe (round((n-1)*p))."""
    if not sorted_vals:
        return 0
    idx = round((len(sorted_vals) - 1) * p)
    return sorted_vals[min(idx, len(sorted_vals) - 1)]


def dist_line(label, vals):
    s = sorted(vals)
    return (
        f"{label:<28} n={len(s):<5} p50={percentile(s, 0.50):<8} p90={percentile(s, 0.90):<8} "
        f"p99={percentile(s, 0.99):<8} p999={percentile(s, 0.999):<8} max={s[-1]:<10} "
        f"geomean={math.exp(sum(math.log(max(v, 1)) for v in s) / len(s)):.1f}"
    )


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    path = sys.argv[1]
    bar_ns = 1000
    tail_ns_per_byte = 8.0
    args = sys.argv[2:]
    i = 0
    while i < len(args):
        if args[i] == "--bar-ns":
            i += 1
            bar_ns = int(args[i])
        elif args[i] == "--tail-ns-per-byte":
            i += 1
            tail_ns_per_byte = float(args[i])
        else:
            sys.exit(f"unknown arg: {args[i]}")
        i += 1

    rows = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    if not rows:
        sys.exit(f"no rows in {path}")

    lengths = sorted(r["pattern_bytes"] for r in rows)
    p99_len = percentile(lengths, 0.99)

    print("=== RGX-0078.5.j.3 CORPUS-MAX DISTRIBUTION BASELINE ===")
    print(f"rows={len(rows)} source={path}")
    print("per-cell statistic: min_ns (best observed parse; p50_ns reported alongside)")
    print()
    print("--- corpus length profile (bytes) ---")
    print(
        f"p50={percentile(lengths, 0.50)} p90={percentile(lengths, 0.90)} "
        f"p99={p99_len} max={lengths[-1]}"
    )
    print()

    print("--- parse-time distribution (ns) ---")
    print(dist_line("all cells (min_ns)", [r["min_ns"] for r in rows]))
    print(dist_line("all cells (p50_ns)", [r["p50_ns"] for r in rows]))
    accepts = [r for r in rows if r["actual_parse"] == "ok"]
    rejects = [r for r in rows if r["actual_parse"] == "fail"]
    print(dist_line("accepts (min_ns)", [r["min_ns"] for r in accepts]))
    print(dist_line("rejects (min_ns)", [r["min_ns"] for r in rejects]))
    print()

    print("--- length-band table (min_ns) ---")
    bands = [(0, 10), (11, 32), (33, p99_len), (p99_len + 1, None)]
    for lo, hi in bands:
        sel = [
            r["min_ns"]
            for r in rows
            if r["pattern_bytes"] >= lo and (hi is None or r["pattern_bytes"] <= hi)
        ]
        label = f"{lo}..{hi if hi is not None else 'max'} bytes"
        if sel:
            print(dist_line(label, sel))
        else:
            print(f"{label:<28} n=0")
    print()

    print(f"--- BAR COMPLIANCE (bar={bar_ns}ns, p99-size={p99_len}B, tail={tail_ns_per_byte}ns/B) ---")
    small = [r for r in rows if r["pattern_bytes"] <= p99_len]
    tail = [r for r in rows if r["pattern_bytes"] > p99_len]
    small_viol = sorted(
        (r for r in small if r["min_ns"] >= bar_ns), key=lambda r: -r["min_ns"]
    )
    tail_viol = sorted(
        (r for r in tail if r["min_ns"] > tail_ns_per_byte * r["pattern_bytes"]),
        key=lambda r: -(r["min_ns"] / r["pattern_bytes"]),
    )
    small_max = max(small, key=lambda r: r["min_ns"])
    print(
        f"<=p99-size population: n={len(small)} violations(min_ns>={bar_ns})={len(small_viol)} "
        f"({100.0 * len(small_viol) / len(small):.1f}%) "
        f"max={small_max['min_ns']}ns ({small_max['pattern_bytes']}B, {small_max['id']})"
    )
    if tail:
        tail_worst = max(tail, key=lambda r: r["min_ns"] / r["pattern_bytes"])
        print(
            f">p99-size tail:        n={len(tail)} violations(min_ns>{tail_ns_per_byte}ns/B)={len(tail_viol)} "
            f"worst-throughput={tail_worst['min_ns'] / tail_worst['pattern_bytes']:.1f}ns/B "
            f"({tail_worst['pattern_bytes']}B, {tail_worst['id']})"
        )
    strict_max = max(rows, key=lambda r: r["min_ns"])
    print(
        f"strict-literal reading: corpus-wide max(min_ns)={strict_max['min_ns']}ns "
        f"({strict_max['pattern_bytes']}B, {strict_max['id']}) — "
        + ("COMPLIANT (<1µs)" if strict_max["min_ns"] < bar_ns else "NOT YET (bar work remains)")
    )
    print()

    print("--- top 20 cells by min_ns ---")
    print(f"{'min_ns':>10} {'p50_ns':>10} {'bytes':>6} {'ns/B':>8}  {'verdict':<5} {'mode':<12} id")
    for r in sorted(rows, key=lambda r: -r["min_ns"])[:20]:
        print(
            f"{r['min_ns']:>10} {r['p50_ns']:>10} {r['pattern_bytes']:>6} "
            f"{r['min_ns'] / max(r['pattern_bytes'], 1):>8.1f}  {r['actual_parse']:<5} "
            f"{r['sampling_mode']:<12} {r['id']}"
        )
    print()

    print("--- the 3 formerly-'hang' tracked cells (RGX-0078 878/881/1340) ---")
    by_id = {r["id"]: r for r in rows}
    for cell in TRACKED_CELLS:
        r = by_id.get(cell)
        if r is None:
            print(f"{cell}: NOT PRESENT in the sweep (unexpected — investigate)")
        else:
            print(
                f"{cell}: min={r['min_ns']}ns p50={r['p50_ns']}ns max={r['max_ns']}ns "
                f"bytes={r['pattern_bytes']} verdict={r['actual_parse']} mode={r['sampling_mode']}"
            )
    print()

    modes = {}
    for r in rows:
        modes[r["sampling_mode"]] = modes.get(r["sampling_mode"], 0) + 1
    print(f"--- sampling modes (no silent caps) --- {modes}")
    mism = [r for r in rows if r["expected_parse"] != "unknown" and r["expected_parse"] != r["actual_parse"]]
    print(f"--- oracle cross-check --- accepts={len(accepts)} rejects={len(rejects)} expectation_mismatches={len(mism)}")


if __name__ == "__main__":
    main()
