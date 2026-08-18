#!/usr/bin/env python3
"""Perturbation helper for `probe.sh` — lower a BASELINE counter so the fresh run reads as a rise.

⛔ TWO THINGS THIS EXISTS TO GET RIGHT, both of which the probe's earlier cuts got wrong:

1. **It edits `entries.tsv`, which is the file the gate SUMS** (`totals_of`), not the prose totals in
   `cost.md`. Perturbing the human-readable report leaves the compared numbers untouched and the
   gate reports HOLDING when it was never challenged.
2. **It subtracts from the LARGEST row and reports the EXACT resulting total.** Subtracting a flat
   delta from the first row clamps at zero when that row is small (the first row carried 697 entries
   against a delta of 1000), so the perturbation is not the size it claims — and an acceptance row
   whose `to` is computed from a guessed delta then matches nothing, which fails for the wrong
   reason.

    python3 perturb.py <entries.tsv> <column> <delta>     # prints "from<TAB>to"

`to` is the total the fresh measurement will produce (i.e. the total BEFORE this perturbation);
`from` is the perturbed total the gate will compare against. An acceptance row must name both.
"""
from __future__ import annotations

import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 4:
        print(__doc__, file=sys.stderr)
        return 2
    path, column, delta = Path(sys.argv[1]), sys.argv[2], int(sys.argv[3])

    lines = path.read_text(encoding="utf-8").splitlines()
    head = lines[0].split("\t")
    if column not in head:
        print(f"perturb: {path} has no column `{column}` (header {head})", file=sys.stderr)
        return 2
    col = head.index(column)

    rows = [l for l in lines[1:] if l.strip()]
    original_total = sum(int(l.split("\t")[col]) for l in rows)

    # the largest row, so `delta` is always subtractable without clamping
    victim = max(range(len(rows)), key=lambda i: int(rows[i].split("\t")[col]))
    fields = rows[victim].split("\t")
    if int(fields[col]) < delta:
        print(f"perturb: the largest `{column}` row holds {fields[col]}, less than delta {delta}",
              file=sys.stderr)
        return 2
    fields[col] = str(int(fields[col]) - delta)
    rows[victim] = "\t".join(fields)

    path.write_text("\n".join([lines[0]] + rows) + "\n", encoding="utf-8")
    print(f"{original_total - delta}\t{original_total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
