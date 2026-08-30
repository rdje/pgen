#!/bin/bash
# `ENGINE-UNIVERSAL-SERVICES.79`(a) — the PER-FAMILY / PER-PROFILE / PER-BUILD-MODE
# per-frame stack-cost census.
#
# WHY IT EXISTS. `.79` established that a stack floor cannot be a byte constant
# because the per-frame cost is a per-(family, profile, build-mode) measurement.
# This is the instrument that measures it, so the claim is a table rather than an
# assertion about SystemVerilog generalised to ten families.
#
# HOW. For each family it climbs a doubling ladder of nesting levels until the
# depth probe refuses on STACK grounds — `BudgetExhausted` or `NoFrameAffordable`,
# the two causes that are facts about the stack rather than about what the process
# could not learn (`.69`(d)) — and reports the probe's own `bytes_per_frame`.
#
# ⭐ RUN AT A REDUCED RESERVATION ON PURPOSE. Since `.79`(b)(c) the ceiling is
# proportional to the reservation, and crossing cost is quadratic in source depth
# (`.68`), so the floor reaches every family's ceiling far more cheaply than the
# 256 MiB default does. The measured quantity is reservation-INDEPENDENT: it is
# `used_bytes / depth` at the refusal, and `-0141` measured the same reading at
# 2 048 / 4 096 / 8 192 levels of the same shape.
#
# Usage:  bash <this script> <path-to-parseability_probe> [reservation-bytes]
#   PGEN_CENSUS_CELLS="family:profile[:seed] ..."  run a subset, optionally seeded
#   PGEN_CENSUS_SEED=N                             first rung for every cell (default 64)
#   PGEN_CENSUS_RUNG_CAP_SECONDS=N                 per-rung wall-clock cap (default 600)
#
# ⚠️ Verdicts, causes and frame/byte counts are deterministic. Wall clocks are not
# — check `uptime` before quoting any timing from this (`-0143`).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 1

probe="${1:?usage: per_frame_census.sh <path-to-parseability_probe> [reservation-bytes]}"
reservation="${2:-100663296}"   # 96 MiB, MIN_DEDICATED_PARSE_STACK_BYTES
gen="docs/tasks/artifacts/engine_universal_services/eus79a_per_frame_census/gen_deep_input.py"
out="tmp/eus79a/census"
# Per-rung wall-clock cap. Crossing the ceiling is quadratic in depth, so a family
# whose frames are very cheap needs a deep input and an honest cap; a rung that
# times out is reported as such, never silently skipped.
cap="${PGEN_CENSUS_RUNG_CAP_SECONDS:-600}"

# family:profile  — an empty profile means "the grammar's own default entry".
CELLS=(
  "systemverilog:sv_2017" "systemverilog:sv_2023" "systemverilog:verilog_2005"
  "vhdl:" "json:" "rtl_const_expr:" "rtl_frontend:"
  "return_annotation:" "semantic_annotation:" "systemverilog_preprocessor:"
  "regex:"
)
# `PGEN_CENSUS_CELLS` narrows the run to a space-separated subset, for re-measuring
# one cell without paying for the whole table.
#
# ⭐ A cell may carry an optional THIRD field — `family:profile:seed` — naming the
# ladder's first rung for that cell alone. It exists because the time to reach a
# family's ceiling grows with the SQUARE of the frame depth it takes to get there
# (`.68`: the cycle guard scans the whole live stack on every rule entry), so a
# family whose frames are cheap has a quadratically more expensive cell — the
# RELEASE census is the case that needs it, since release frames cost ≈8× less than
# debug ones and therefore need ≈8× the depth. ⛔ SEEDING CANNOT BIAS THE ANSWER:
# the measured quantity is `used_bytes / depth` at the refusal, which `-0141`
# measured identically at 2 048 / 4 096 / 8 192 levels of one shape and which this
# census reproduced for `json` (2 882 at 16 384 levels, 2 882 at 50 000 by hand). The
# LEVELS column records where the ladder happened to stop and is not a claim.
if [ -n "${PGEN_CENSUS_CELLS:-}" ]; then read -r -a CELLS <<< "$PGEN_CENSUS_CELLS"; fi
SEED="${PGEN_CENSUS_SEED:-64}"   # default first rung — unambiguously under every family's first sample
GROWTH=2
MAX_LEVELS=262144

printf "%-28s %-14s %-8s %-10s %-22s %s\n" FAMILY PROFILE LEVELS FRAMES CAUSE BYTES_PER_FRAME
for cell in "${CELLS[@]}"; do
  IFS=':' read -r family profile cell_seed <<< "$cell"
  levels="${cell_seed:-$SEED}"
  reported=""
  while [ "$levels" -le "$MAX_LEVELS" ]; do
    src="$(python3 "$gen" "$family" "$levels" --out-dir "$out")" || break
    if [ -n "$profile" ]; then
      out_txt="$(timeout "$cap" env PGEN_REPORT_STACK_PROBE=1 PGEN_PARSE_STACK_BYTES="$reservation" \
                  "$probe" --parse "$family" "$src" --profile "$profile" 2>&1)"; rc=$?
    else
      out_txt="$(timeout "$cap" env PGEN_REPORT_STACK_PROBE=1 PGEN_PARSE_STACK_BYTES="$reservation" \
                  "$probe" --parse "$family" "$src" 2>&1)"; rc=$?
    fi
    if [ "$rc" -eq 124 ]; then
      reported="$(printf '%-8s %-10s %-22s %s' "$levels" "-" "RUNG_TIMEOUT_${cap}s" "not_measured")"
      break
    fi
    if [ "$rc" -eq 134 ]; then
      reported="$(printf '%-8s %-10s %-22s %s' "$levels" "-" "SIGABRT" "not_measured")"
      break
    fi
    cause="$(printf '%s\n' "$out_txt" | grep -o 'cause=[A-Za-z]*' | head -1 | cut -d= -f2)"
    depth="$(printf '%s\n' "$out_txt" | grep -o 'depth=[0-9]*' | head -1 | cut -d= -f2)"
    bpf="$(printf '%s\n' "$out_txt" | grep -o 'bytes_per_frame=[0-9]*' | head -1 | cut -d= -f2)"
    # ⛔⛔ A REJECTION THE INSTRUMENT CANNOT SEE IS NOT "NO CEILING". The rejection
    # DIAGNOSTIC carries its own frame count, and on the regex family it is the only
    # surface that does: that family parses on a worker whose stack-probe reading is
    # stranded (`ENGINE-UNIVERSAL-SERVICES.82`), so `stack_probe:` says `no_refusal`
    # for a parse whose own error says the stack ran out. Classifying that as
    # CEILING_NOT_REACHED would publish "this family has no measurable ceiling" for a
    # family that demonstrably refused — the silent, flattering direction.
    ceiling_frames="$(printf '%s\n' "$out_txt" \
      | grep -o 'ran out at [0-9]* parser frames' | head -1 | grep -o '[0-9]*')"
    # Only the two STACK-grounds causes end the climb (`.78`: taking BudgetExhausted
    # alone makes a ladder miss its own crossing on verilog_2005).
    case "$cause" in
      BudgetExhausted|NoFrameAffordable)
        reported="$(printf '%-8s %-10s %-22s %s' "$levels" "$depth" "$cause" "$bpf")"; break ;;
      ExtentUnknown|DirectionUnknown|FrameCostUnmeasurable)
        reported="$(printf '%-8s %-10s %-22s %s' "$levels" "${depth:--}" "$cause" "not_measurable_by_this_route")"; break ;;
    esac
    if [ "$rc" -ne 0 ] && [ -n "$ceiling_frames" ]; then
      reported="$(printf '%-8s %-10s %-22s %s' \
        "$levels" "$ceiling_frames" "REFUSED_UNOBSERVABLE" "not_observable_see_EUS.82")"
      break
    fi
    levels=$((levels * GROWTH))
  done
  [ -n "$reported" ] || reported="$(printf '%-8s %-10s %-22s %s' ">$MAX_LEVELS" "-" "CEILING_NOT_REACHED" "not_measured")"
  printf "%-28s %-14s %s\n" "$family" "${profile:-<default>}" "$reported"
done
