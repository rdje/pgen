#!/bin/bash
# `ENGINE-UNIVERSAL-SERVICES.69`(a) — the OVERSHOOT distribution over SHAPES.
#
# WHY IT EXISTS. `.69`(d) closed by measuring the spend at a refusal, and left (a) an
# acceptance criterion in its own words: the overshoot past the spendable budget is
# SHAPE-driven (6 800 B to 597 280 B, an 88x spread at ONE reservation), so the 6.25 %
# headroom reserve must be sized against the WORST shape -- and NOTHING MEASURES THAT
# DISTRIBUTION. A default derived from host RAM (which is what (a) owes) is only as safe
# as the reserve that absorbs the overshoot, so this is the measurement (a) rests on.
#
# WHAT IT MEASURES. For each shape it climbs a doubling ladder until the depth probe
# refuses on STACK grounds, then reports, from the probe's own reading:
#
#   overshoot = used_bytes - budget_bytes        (how far past the budget the parse got)
#   reserve   = reservation - budget             (what the divisor withholds to absorb it)
#   margin    = reserve / overshoot              (< 1 means the reserve did NOT cover it)
#
# ⭐ RUN IN DEBUG, ON PURPOSE. `.79`(a) slice 2b measured that the DEBUG build is the
# binding build mode by 7.83x, and per-frame cost is ~8x larger there -- so the overshoot,
# which is frames-past-budget x bytes-per-frame, is larger and the MARGIN is smaller. The
# binding mode is the honest one to size a reserve against.
#
# ⛔ EVERY SHAPE IS VALIDATED TO ACCEPT AT A SHALLOW DEPTH FIRST (`validate_shapes.sh`).
# A shape the shipped profile rejects would measure the rejection path, not the stack.
#
# Usage:  bash <this script> <path-to-parseability_probe> [reservation-bytes]
#   PGEN_OVERSHOOT_SHAPES="a b c"        run a subset
#   PGEN_OVERSHOOT_RUNG_CAP_SECONDS=N    per-rung wall-clock cap (default 600)
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 1

probe="${1:?usage: overshoot_census.sh <path-to-parseability_probe> [reservation-bytes]}"
reservation="${2:-100663296}"
gen="docs/tasks/artifacts/engine_universal_services/eus69a_overshoot_census/gen_shapes.py"
out="tmp/eus69a/shapes"
cap="${PGEN_OVERSHOOT_RUNG_CAP_SECONDS:-600}"

read -r -a SHAPES <<< "${PGEN_OVERSHOOT_SHAPES:-$(python3 "$gen" --list)}"
SEED=32
MAX_LEVELS=65536

printf "%-12s %-8s %-10s %-20s %-12s %-12s %-11s %-9s %s\n" \
  SHAPE LEVELS DEPTH CAUSE USED_BYTES BUDGET OVERSHOOT B_FRAME MARGIN
for shape in "${SHAPES[@]}"; do
  levels=$SEED; row=""
  while [ "$levels" -le "$MAX_LEVELS" ]; do
    src="$(python3 "$gen" "$shape" "$levels" --out-dir "$out")" || break
    txt="$(timeout "$cap" env PGEN_REPORT_STACK_PROBE=1 PGEN_PARSE_STACK_BYTES="$reservation" \
            "$probe" --parse systemverilog "$src" --profile sv_2017 2>&1)"; rc=$?
    if [ "$rc" -eq 124 ]; then
      row="$(printf '%-8s %-10s %-20s %-12s %-12s %-11s %-9s %s' \
             "$levels" "-" "RUNG_TIMEOUT_${cap}s" - - - - -)"; break
    fi
    cause="$(printf '%s\n' "$txt" | grep -o 'cause=[A-Za-z]*' | head -1 | cut -d= -f2)"
    case "$cause" in
      BudgetExhausted|NoFrameAffordable)
        depth="$( printf '%s\n' "$txt" | grep -o 'depth=[0-9]*'            | head -1 | cut -d= -f2)"
        used="$(  printf '%s\n' "$txt" | grep -o 'used_bytes=[0-9]*'       | head -1 | cut -d= -f2)"
        budget="$(printf '%s\n' "$txt" | grep -o 'budget_bytes=[0-9]*'     | head -1 | cut -d= -f2)"
        bpf="$(   printf '%s\n' "$txt" | grep -o 'bytes_per_frame=[0-9]*'  | head -1 | cut -d= -f2)"
        # `used < budget` is legal: `NoFrameAffordable` fires with headroom left that
        # will not pay for one more frame. Report the SIGNED overshoot rather than
        # clamping it -- a negative one is a fact about the cause, not an error.
        row="$(python3 - "$levels" "$depth" "$cause" "$used" "$budget" "$bpf" "$reservation" <<'PY'
import sys
levels, depth, cause, used, budget, bpf, reservation = sys.argv[1:8]
used, budget, reservation = int(used), int(budget), int(reservation)
overshoot = used - budget
reserve = reservation - budget
margin = (reserve / overshoot) if overshoot > 0 else float("inf")
m = "no_overshoot" if overshoot <= 0 else f"{margin:.2f}x"
print(f"{levels:<8} {depth:<10} {cause:<20} {used:<12} {budget:<12} {overshoot:<11} {bpf:<9} {m}")
PY
)"; break ;;
      ExtentUnknown|DirectionUnknown|FrameCostUnmeasurable)
        row="$(printf '%-8s %-10s %-20s %-12s %-12s %-11s %-9s %s' \
               "$levels" - "$cause" - - - - not_measurable_by_this_route)"; break ;;
    esac
    levels=$((levels * 2))
  done
  [ -n "$row" ] || row="$(printf '%-8s %-10s %-20s %-12s %-12s %-11s %-9s %s' \
                          ">$MAX_LEVELS" - CEILING_NOT_REACHED - - - - -)"
  printf "%-12s %s\n" "$shape" "$row"
done
