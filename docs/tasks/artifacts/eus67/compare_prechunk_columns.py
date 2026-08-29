#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.67(b) leg 3 — the NO-REGRESSION control for the report itself.

Leg 3 adds a SECOND stderr line to the parse-footprint instrument and thirteen columns to the
tracked producer. The risk is not that the new columns are wrong — they have their own oracle
(`arena_chunk_oracle_probe`). The risk is that adding them silently MOVED an old one: every
extraction in `measure_parse_footprint.sh` is a greedy `sed 's/.*<token>=…/'`, so a new token
ending in an existing one re-targets its capture and the TSV keeps reading like a measurement.

So this diffs the pre-leg-3 columns of the new run against `-0124`'s published
`attribution_owned.tsv`, cell for cell.

⛔ Two columns are EXPECTED to move and are compared with a tolerance rather than exactly:
`peak_rss_bytes` is a process measurement (`.67`(a) measured 0.03–0.48 % run-to-run) and the
instrument now prints one more line. Everything else — every arena and owned-heap count and byte
figure — is DETERMINISTIC for a deterministic parser and must match EXACTLY. A count that moved is
a behaviour change, which is exactly what this control exists to catch.
"""
import sys

EXACT_SKIP = {"peak_rss_bytes"}
RSS_TOLERANCE_PCT = 2.0


def read(path):
    rows = [l.rstrip("\n").split("\t") for l in open(path)]
    return rows[0], rows[1:]


def main(old_path, new_path):
    old_head, old_body = read(old_path)
    new_head, new_body = read(new_path)
    shared = [c for c in old_head if c in new_head]
    missing = [c for c in old_head if c not in new_head]
    if missing:
        print(f"REFUSE: the new TSV dropped pre-leg-3 column(s): {', '.join(missing)}")
        return 2
    old_by_path = {r[0]: r for r in old_body}
    new_by_path = {r[0]: r for r in new_body}
    only_old = sorted(set(old_by_path) - set(new_by_path))
    only_new = sorted(set(new_by_path) - set(old_by_path))
    if only_old or only_new:
        print(f"REFUSE: the two runs cover different inputs "
              f"(only in old: {only_old}; only in new: {only_new})")
        return 2

    oi = {n: i for i, n in enumerate(old_head)}
    ni = {n: i for i, n in enumerate(new_head)}
    exact_checked = exact_bad = 0
    rss_rows = []
    for path, orow in sorted(old_by_path.items()):
        nrow = new_by_path[path]
        for col in shared:
            o, n = orow[oi[col]], nrow[ni[col]]
            if col in EXACT_SKIP:
                if o not in ("NA", "") and n not in ("NA", ""):
                    rss_rows.append((path, float(o), float(n)))
                continue
            exact_checked += 1
            if o != n:
                exact_bad += 1
                print(f"MOVED  {path.split('/')[-1]:<34} {col:<22} {o!r} -> {n!r}")

    print(f"\nexact columns: {exact_checked - exact_bad}/{exact_checked} identical "
          f"across {len(old_by_path)} inputs and {len(shared) - len(EXACT_SKIP)} columns")
    worst = 0.0
    for path, o, n in rss_rows:
        pct = 100.0 * (n - o) / o
        worst = max(worst, abs(pct))
        print(f"  rss {path.split('/')[-1]:<34} {o:>12.0f} -> {n:>12.0f}  {pct:+6.2f} %")
    print(f"peak RSS worst |delta| = {worst:.2f} % (tolerance {RSS_TOLERANCE_PCT:.1f} %)")
    ok = exact_bad == 0 and worst <= RSS_TOLERANCE_PCT
    print("PRE-LEG-3 COLUMNS REPRODUCE" if ok else "PRE-LEG-3 COLUMNS DID NOT REPRODUCE")
    return 0 if ok else 1


if __name__ == "__main__":
    a = sys.argv[1] if len(sys.argv) > 1 else "docs/tasks/artifacts/eus67/attribution_owned.tsv"
    b = sys.argv[2] if len(sys.argv) > 2 else "docs/tasks/artifacts/eus67/attribution_chunks.tsv"
    sys.exit(main(a, b))
