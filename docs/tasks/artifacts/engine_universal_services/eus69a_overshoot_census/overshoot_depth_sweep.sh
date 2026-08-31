#!/bin/bash
# `ENGINE-UNIVERSAL-SERVICES.69`(a) — the overshoot as a function of INPUT DEPTH at a FIXED
# reservation.
#
# WHY. The first cut varied the RESERVATION and read the margin, and the four points came
# back 1.10x / 3.26x / 1.13x / 3.31x -- non-monotone, refuting both pre-registered
# hypotheses. The reason is a CONFOUND IN THE INSTRUMENT: the census climbs a doubling
# ladder and stops at the first rung that crosses, so a different reservation is measured
# on a DIFFERENT INPUT. This sweep holds the reservation fixed and moves the depth, which
# is the variable that was moving underneath the earlier comparison.
#
# ⛔⛔ WHAT THIS INSTRUMENT CANNOT SEE, AND IT IS THE WHOLE POINT. The refusal fires when
# `used >= budget`, and the reading is printed by the process AFTERWARDS. A parse that
# overshoots past the RESERVATION does not report a margin below 1 -- it hits the guard
# page and dies. So the census is CENSORED AT 1.0 by construction: sub-1 cases appear as
# `rc 134` (SIGABRT), never as a small margin. That is why this sweep records rc, and why
# a measured minimum of 1.10x must NOT be read as a bound.
#
# Usage:  bash <this script> <path-to-parseability_probe> <shape> [reservation-bytes]
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 1
probe="${1:?usage: overshoot_depth_sweep.sh <probe> <shape> [reservation-bytes]}"
shape="${2:?shape required}"
reservation="${3:-100663296}"
gen="docs/tasks/artifacts/engine_universal_services/eus69a_overshoot_census/gen_shapes.py"
out="tmp/eus69a/sweep"
cap="${PGEN_OVERSHOOT_RUNG_CAP_SECONDS:-600}"
read -r -a LEVELS <<< "${PGEN_OVERSHOOT_LEVELS:-192 224 256 288 320 352 384 448 512 640 768 1024}"

printf '# shape=%s reservation=%s\n' "$shape" "$reservation"
printf '%-8s %-4s %-10s %-20s %-12s %-11s %-9s %s\n' LEVELS RC DEPTH CAUSE USED_BYTES OVERSHOOT B_FRAME MARGIN
for n in "${LEVELS[@]}"; do
  src="$(python3 "$gen" "$shape" "$n" --out-dir "$out")" || continue
  txt="$(timeout "$cap" env PGEN_REPORT_STACK_PROBE=1 PGEN_PARSE_STACK_BYTES="$reservation" \
          "$probe" --parse systemverilog "$src" --profile sv_2017 2>&1)"; rc=$?
  cause="$(printf '%s\n' "$txt" | grep -o 'cause=[A-Za-z]*'          | head -1 | cut -d= -f2)"
  depth="$(printf '%s\n' "$txt" | grep -o 'depth=[0-9]*'             | head -1 | cut -d= -f2)"
  used="$( printf '%s\n' "$txt" | grep -o 'used_bytes=[0-9]*'        | head -1 | cut -d= -f2)"
  budget="$(printf '%s\n' "$txt" | grep -o 'budget_bytes=[0-9]*'     | head -1 | cut -d= -f2)"
  bpf="$(  printf '%s\n' "$txt" | grep -o 'bytes_per_frame=[0-9]*'   | head -1 | cut -d= -f2)"
  if [ "$rc" -eq 134 ]; then
    printf '%-8s %-4s %-10s %-20s %-12s %-11s %-9s %s\n' "$n" 134 - SIGABRT_PAST_RESERVATION - - - "BELOW_1_CENSORED"
    continue
  fi
  if [ -z "${cause:-}" ] || [ -z "${used:-}" ] || [ "$used" = not_measured ]; then
    printf '%-8s %-4s %-10s %-20s %-12s %-11s %-9s %s\n' "$n" "$rc" "${depth:--}" "${cause:-no_refusal}" - - - -
    continue
  fi
  python3 - "$n" "$rc" "$depth" "$cause" "$used" "$budget" "$bpf" "$reservation" <<'PY'
import sys
n, rc, depth, cause, used, budget, bpf, reservation = sys.argv[1:9]
used, budget, reservation = int(used), int(budget), int(reservation)
over = used - budget
reserve = reservation - budget
m = "no_overshoot" if over <= 0 else f"{reserve/over:.2f}x"
print(f"{n:<8} {rc:<4} {depth:<10} {cause:<20} {used:<12} {over:<11} {bpf:<9} {m}")
PY
done
