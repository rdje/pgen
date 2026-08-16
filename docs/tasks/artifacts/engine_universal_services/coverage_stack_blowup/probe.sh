#!/usr/bin/env bash
# docs/tasks/artifacts/engine_universal_services/coverage_stack_blowup/probe.sh
#
# ENGINE-UNIVERSAL-SERVICES.22 (a) — WHERE the transactional coverage stack blows up, measured as a
# GROWTH LAW rather than as a hang.
#
# Three parses per rung of the else-if ladder, all on the SAME input, differing only in which
# observability surface is attached:
#   BARE     the FUSED cascade_* graph, no counters            (TOOLBOX 1.1)
#   ENTRY    the PROTOCOL graph, counters, NO coverage stack   (TOOLBOX 3.4)
#   OUTCOME  the PROTOCOL graph, counters AND coverage stack   (TOOLBOX 3.5)
#
# ⛔ ENTRY and OUTCOME take the SAME graph, so any divergence between them is the coverage stack and
# nothing else. That is the whole design of this probe: it isolates one variable.
#
# ⭐ THE PRE-FIX RUN IS PRESERVED BESIDE THIS SCRIPT as `probe_before_fix.txt`, because after
# `ENGINE-UNIVERSAL-SERVICES.22` (e) this probe can no longer reproduce the defect it was built
# to measure — the growth it recorded is gone from the engine. A probe whose subject is fixed
# stops being evidence the moment it is re-run, so the measurement is kept as a file rather
# than as a re-runnable command (`GENERATED-LINT-CORRECTNESS.13`, the same reason a scratch-slot
# probe is snapshotted before the slot is restored). Re-running this script now measures the
# FIXED engine, which is what `verify_fix.sh` does deliberately.
#
# Usage:  bash docs/tasks/artifacts/engine_universal_services/coverage_stack_blowup/probe.sh [max_arms] [timeout_s]
# Output: .../coverage_stack_blowup/probe.txt   (pre-fix evidence: probe_before_fix.txt)
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../../.." && pwd)"; cd "$ROOT" || exit 2
OUT="$HERE/probe.txt"
MAX="${1:-10}"
TIMEOUT_S="${2:-25}"

PROBE="rust/target/release/parseability_probe"
WORK="rust/target/e22/ladder"
[ -x "$PROBE" ] || { echo "probe missing: $PROBE" >&2; exit 2; }

python3 "$HERE/gen_ladder.py" "$WORK" "$MAX" >/dev/null || exit 2

exec > >(tee "$OUT") 2>&1

printf '=========================================================================================\n'
printf 'ENGINE-UNIVERSAL-SERVICES.22 (a) — coverage-stack growth law, else-if ladder\n'
printf '=========================================================================================\n'
printf 'probe: %s   per-run timeout: %ss\n\n' "$PROBE" "$TIMEOUT_S"
printf '%5s %8s %10s %10s %12s %12s %10s\n' \
  arms bytes 'bare(s)' 'entry(s)' 'entries' 'committed' 'outcome(s)'
printf '%5s %8s %10s %10s %12s %12s %10s\n' \
  ----- -------- ---------- ---------- ------------ ------------ ----------

for n in $(seq -f '%02g' 0 "$MAX"); do
  f="$WORK/arms$n.sv"
  bytes=$(wc -c < "$f" | tr -d ' ')

  t0=$(python3 -c 'import time;print(time.perf_counter())')
  "$PROBE" --parse systemverilog "$f" --profile sv_2017 >/dev/null 2>&1
  bare=$(python3 -c "import time;print(f'{time.perf_counter()-$t0:.3f}')")

  t0=$(python3 -c 'import time;print(time.perf_counter())')
  "$PROBE" --parse systemverilog "$f" --profile sv_2017 \
     --dump-rule-entry-counts-json "$WORK/e.json" >/dev/null 2>&1
  entry=$(python3 -c "import time;print(f'{time.perf_counter()-$t0:.3f}')")
  entries=$(python3 -c "
import json
try: print(json.load(open('$WORK/e.json'))['total_entries'])
except Exception: print('-')")

  rm -f "$WORK/o.json"
  t0=$(python3 -c 'import time;print(time.perf_counter())')
  # ⛔ A hard per-run timeout, because the whole point is that this one does not terminate.
  #    `timeout` is not on the base macOS image, so the wait is done in the shell.
  "$PROBE" --parse systemverilog "$f" --profile sv_2017 \
     --dump-rule-outcome-counts-json "$WORK/o.json" >/dev/null 2>&1 &
  pid=$!
  waited=0
  while kill -0 "$pid" 2>/dev/null && [ "$waited" -lt "$TIMEOUT_S" ]; do
    sleep 1; waited=$((waited + 1))
  done
  if kill -0 "$pid" 2>/dev/null; then
    kill -9 "$pid" 2>/dev/null; wait "$pid" 2>/dev/null
    outcome=">${TIMEOUT_S}"
    committed="KILLED"
  else
    wait "$pid" 2>/dev/null
    outcome=$(python3 -c "import time;print(f'{time.perf_counter()-$t0:.3f}')")
    committed=$(python3 -c "
import json
try:
    d = json.load(open('$WORK/o.json'))
    print(sum(d.get('rule_committed_counts', {}).values()))
except Exception: print('-')")
  fi

  printf '%5s %8s %10s %10s %12s %12s %10s\n' \
    "$n" "$bytes" "$bare" "$entry" "$entries" "$committed" "$outcome"
done

printf '\n⛔ BARE and ENTRY are the control: same input, same parse, no coverage stack.\n'
printf '   Any divergence in the OUTCOME column is the transactional coverage stack alone.\n'
printf '   `committed` is `coverage_stack.len()` folded per rule — so it IS the stack size.\n'
printf '\n'
printf 'GROWTH LAW (derived from the table above, not asserted):\n'
python3 - "$OUT" <<'PYEOF'
import re, sys
rows = []
for line in open(sys.argv[1], encoding="utf-8"):
    m = re.match(r"\s*(\d\d)\s+(\d+)\s+\S+\s+\S+\s+(\d+)\s+(\d+)\s", line)
    if m:
        rows.append((int(m.group(1)), int(m.group(2)), int(m.group(3)), int(m.group(4))))
if len(rows) < 3:
    print("  (too few completed rungs to derive a law)")
    raise SystemExit(0)
print(f"  {'arms':>4} {'d(entries)':>11} {'committed x':>12} {'committed/entries':>18}")
for i, (n, _b, e, c) in enumerate(rows):
    de = f"+{e - rows[i-1][2]:,}" if i else "-"
    mult = f"x{c / rows[i-1][3]:.2f}" if i else "-"
    print(f"  {n:>4} {de:>11} {mult:>12} {c / e:>17.2f}x")
n0, e0, c0 = rows[0][0], rows[0][2], rows[0][3]
n1, e1, c1 = rows[-1][0], rows[-1][2], rows[-1][3]
span = n1 - n0
print(f"\n  entries   {e0:,} -> {e1:,} over {span} arms = LINEAR "
      f"(+{(e1 - e0) // span:,} per arm, constant)")
print(f"  committed {c0:,} -> {c1:,} over {span} arms = EXPONENTIAL "
      f"(x{(c1 / c0) ** (1 / span):.2f} per arm)")
base = (c1 / c0) ** (1 / span)
for extra in (1, 2, 3):
    print(f"  extrapolated at {n1 + extra} arms: {c1 * base ** extra:,.0f} stack entries "
          f"= {c1 * base ** extra * 4 / 1024 ** 3:.1f} GB of u32")
print("\n  ⛔ The real corpus file carries 10 else-if arms.")
PYEOF

exec 1>&- 2>&-
wait
