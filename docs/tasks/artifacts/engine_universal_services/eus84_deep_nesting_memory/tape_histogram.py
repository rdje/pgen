#!/usr/bin/env python3
"""`ENGINE-UNIVERSAL-SERVICES.84`(b) — read one `parse_alloc_census_probe` report and say WHERE the
memory is, from the two instruments that can: the size-class table at the near-peak snapshot (what is
LIVE at the peak, per class) and the lifetime tape (every allocation >= the tape threshold: its size,
whether it was ever freed, and the `Vec`-doubling chains a push-built buffer leaves behind).

The question it answers for `.84`: is the peak made of MANY SMALL allocations held together (a
per-frame term, quadratic when each is depth-sized) or of a FEW LARGE ones (a table)? It prints:
  * the census scalars the leaf quotes (`peak_live`, `alloc_calls`, `alloc_bytes`, `parse_alloc_calls`);
  * the live-at-snapshot size classes, largest share first, with their COUNT — a class whose count
    equals the frame depth is a per-frame allocation;
  * the tape's size histogram (top sizes by count, the power-of-two doubling chains), the never-freed
    set, and the lifetime distribution in events.
It REFUSES a tape that reports `complete=false` (TOOLBOX.md §3.3e: a truncated tape is a prefix, not a
sample) unless `--allow-incomplete` is given, and then labels every tape figure as a PREFIX.

Usage: tape_histogram.py <census-report.txt> [--allow-incomplete] [--top N]
"""
import argparse
import collections
import statistics
import sys

NEVER = 18446744073709551615


def kv_line(text, anchor):
    for line in text.splitlines():
        if line.startswith(anchor):
            body = line[len(anchor):]
            out = {}
            for chunk in body.replace(" · ", " ").split():
                if "=" in chunk:
                    k, v = chunk.split("=", 1)
                    out[k] = v
            return out
    return None


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("report")
    ap.add_argument("--allow-incomplete", action="store_true")
    ap.add_argument("--top", type=int, default=8)
    args = ap.parse_args()
    text = open(args.report, encoding="utf-8", errors="replace").read()

    census = kv_line(text, "PARSE-ALLOC-CENSUS:")
    tape = kv_line(text, "PARSE-ALLOC-TAPE:")
    if census is None or tape is None:
        print("TAPE-HISTOGRAM: the report carries no PARSE-ALLOC-CENSUS / PARSE-ALLOC-TAPE line", file=sys.stderr)
        return 2
    print("TAPE-HISTOGRAM: grammar=%s input=%s peak_live=%s alloc_calls=%s alloc_bytes=%s parse_alloc_calls=%s free_calls=%s" % (
        census.get("grammar"), census.get("input"), census.get("peak_live"), census.get("alloc_calls"),
        census.get("alloc_bytes"), census.get("parse_alloc_calls"), census.get("free_calls")))
    complete = tape.get("complete") == "true"
    if not complete and not args.allow_incomplete:
        print("TAPE-HISTOGRAM: REFUSED — the tape reports complete=%s (overflow=%s unmatched_frees=%s); a truncated tape is a PREFIX, not a sample (pass --allow-incomplete to read it as one, labelled)" % (
            tape.get("complete"), tape.get("overflow"), tape.get("unmatched_frees")), file=sys.stderr)
        return 1
    label = "" if complete else " [PREFIX — tape incomplete]"

    # live-at-snapshot classes
    classes = []
    for line in text.splitlines():
        if line.strip().startswith("ALLOC-CLASS"):
            d = dict(kv.split("=", 1) for kv in line.split()[1:])
            classes.append((float(d["share"]), int(d["class"]), int(d["lo"]), int(d["hi"]), int(d["live_bytes"]), int(d["live_count"])))
    classes.sort(reverse=True)
    print("LIVE-AT-SNAPSHOT (share of snapshot_live=%s, largest first):" % census.get("snapshot_live"))
    for share, k, lo, hi, b, c in classes[: args.top]:
        print("  class=%d [%d, %d) live_bytes=%d live_count=%d share=%.4f" % (k, lo, hi, b, c, share))

    rows = []
    for line in text.splitlines():
        if line.strip().startswith("ALLOC-TAPE"):
            d = dict(kv.split("=", 1) for kv in line.split()[1:])
            rows.append((int(d["size"]), int(d["alloc_seq"]), int(d["free_seq"])))
    print("TAPE%s: rows=%d min_bytes=%s distinct_sizes=%d total_MB=%.1f" % (
        label, len(rows), tape.get("min_bytes"), len({r[0] for r in rows}), sum(r[0] for r in rows) / 1048576))
    never = [r for r in rows if r[2] == NEVER]
    print("  never_freed=%d bytes=%d" % (len(never), sum(r[0] for r in never)))
    by = collections.Counter(r[0] for r in rows)
    print("  top sizes by count:")
    for s, c in by.most_common(args.top):
        print("    size=%d count=%d MB=%.1f pow2=%s" % (s, c, s * c / 1048576, "yes" if s & (s - 1) == 0 else "no"))
    life = [r[2] - r[1] for r in rows if r[2] != NEVER]
    if life:
        print("  lifetime_events: median=%d mean=%.1f max=%d (median 1 = a realloc chain: freed by the next event)" % (
            statistics.median(life), statistics.mean(life), max(life)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
