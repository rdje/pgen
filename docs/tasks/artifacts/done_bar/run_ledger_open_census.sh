#!/usr/bin/env bash
# run_ledger_open_census.sh — resolve the released-parser bug ledger to REAL per-family open-defect
# counts (task-tree leaf `DONE-BAR.1`; the tree recorded a bare token tally as a HYPOTHESIS and
# required `.1` to resolve it).
#
# ⛔ THE HYPOTHESIS THIS REPLACES. `DONE-BAR.md` recorded "24 occurrences of the token `open`" and
# said, correctly, that a token tally is NOT a count of open defects — `open` occurs in prose. This
# census reads the ledger's own STATE column instead, against the state vocabulary the ledger itself
# defines in its "State Meanings" section, so the classification is the ledger's, not this script's.
#
# A state is OPEN iff the defect is not closed out: Reported / Reproduced / Root Caused /
# Fix In Progress / Fixed Pending Release / Deferred. `Released` and `Rejected` are closed.
#
#   bash docs/tasks/artifacts/done_bar/run_ledger_open_census.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

LEDGER="docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md"

python3 - "$LEDGER" <<'PY'
import re, sys, collections

path = sys.argv[1]
text = open(path, encoding="utf-8").read()

# The vocabulary is DERIVED from the ledger's own "State Meanings" section, so a state added there
# tomorrow is picked up rather than silently falling through as "unknown".
sec = text.split("## State Meanings", 1)[1].split("## Closure Rule", 1)[0]
states = re.findall(r"^- `([^`]+)`", sec, re.MULTILINE)
CLOSED = {"Released", "Rejected"}
open_states = [s for s in states if s not in CLOSED]

print(f"state vocabulary derived from the ledger: {len(states)} states")
print(f"  closed: {', '.join(sorted(CLOSED))}")
print(f"  open:   {', '.join(open_states)}")
print()

# Ledger rows: | `ID` | `family` / `profile` | ... | `State` | ...
rows = []
for line in text.splitlines():
    if not line.startswith("| `"):
        continue
    cells = [c.strip().strip("`") for c in line.split("|")]
    if len(cells) < 9:
        continue
    ident, family_cell = cells[1], cells[2]
    state = next((c for c in cells if c in states), None)
    if state is None:
        continue
    family = family_cell.split("/")[0].strip().strip("`")
    rows.append((ident, family, state))

print(f"ledger rows parsed: {len(rows)}")
bare_tally = len(re.findall(r"\bopen\b", text, re.IGNORECASE))
print(f"bare `open` token tally (the refuted hypothesis): {bare_tally}")
print()

by_family = collections.defaultdict(lambda: collections.Counter())
for _, family, state in rows:
    by_family[family][state] += 1

print(f"{'family':<32} {'rows':>5} {'OPEN':>5}   open rows")
print("-" * 78)
total_open = 0
for family in sorted(by_family):
    counts = by_family[family]
    n = sum(counts.values())
    op = sum(v for k, v in counts.items() if k not in CLOSED)
    total_open += op
    ids = ", ".join(i for i, f, s in rows if f == family and s not in CLOSED) or "-"
    print(f"{family:<32} {n:>5} {op:>5}   {ids}")
print("-" * 78)
print(f"{'TOTAL':<32} {len(rows):>5} {total_open:>5}")
print()
if total_open == 0:
    print("⇒ ZERO open ledger entries. The `24 occurrences of open` figure was PROSE, exactly as the")
    print("  tree suspected. Leg 4's third consumer-facing check (`zero open ledger entries naming")
    print("  the family`) is SATISFIED today for every family — but by NO GATE: nothing reads this")
    print("  file, so the fact is true and unguarded. That is DONE-BAR.5's third gate, not a pass.")
else:
    print(f"⇒ {total_open} genuinely open ledger entries; each names a family whose leg-4 disclosure")
    print("  check would fail. Route them to DONE-BAR.5.")
PY
