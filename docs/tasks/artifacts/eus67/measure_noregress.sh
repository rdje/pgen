#!/usr/bin/env bash
# docs/tasks/artifacts/eus67/measure_noregress.sh — ENGINE-UNIVERSAL-SERVICES.67(b).
#
# The owned-heap accumulation sits on the ALLOCATION hot path (`NodeArena::alloc`), which
# `-0121`'s instrument did not, so this arm measures WALL CLOCK as well as peak RSS. Time
# is the axis the project refuses to trade (peak speed is a non-negotiable), and an RSS-only
# no-regression arm would be blind to exactly the cost this change could introduce.
#
# Usage: measure_noregress.sh <old_probe> <new_probe> <out.tsv> [repeats]
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"
OLD="$1"; NEW="$2"; OUT="$3"; REPS="${4:-3}"
[ -x "$OLD" ] || { echo "no baseline probe at $OLD" >&2; exit 2; }
[ -x "$NEW" ] || { echo "no candidate probe at $NEW" >&2; exit 2; }
W="tmp/eus67b/.nr.$$"; mkdir -p "$W"; trap 'rm -rf "$W"' EXIT
# ENGINE-UNIVERSAL-SERVICES.67(b) leg 3 — the input set is OVERRIDABLE, because `-0124`'s
# second (long-input) wall-clock arm was run by hand against a hard-coded list and therefore
# was not re-runnable from the tracked producer. That is the same defect this artifact
# directory already recorded once ("durability is the COMMITTED COPY RUNNING"), one level up:
# the script ran, but the ARM that decided the wall-clock question did not live in it.
# Space-separated, repo-root-relative.
if [ -n "${PGEN_EUS67_NOREGRESS_INPUTS:-}" ]; then
  # shellcheck disable=SC2206  # word splitting is the documented interface here
  INPUTS=($PGEN_EUS67_NOREGRESS_INPUTS)
else
  INPUTS=(
    "stimuli/sv/subs/iverilog/ivtest/ivltests/br_gh330.v"
    "stimuli/sv/subs/iverilog/ivtest/ivltests/comp1000.v"
    "stimuli/sv/subs/verilator/test_regress/t/t_math_synmul_mul.v"
    "stimuli/sv/subs/opentitan/hw/ip/spi_device/rtl/spi_device_reg_pkg.sv"
  )
fi
for f in "${INPUTS[@]}"; do
  # A missing input must REFUSE, not silently produce fewer rows than the caller asked for.
  [ -r "$f" ] || { echo "no input at $f" >&2; exit 2; }
done
printf 'arm\tpath\trep\tpeak_rss_bytes\twall_s\n' > "$OUT"
for rep in $(seq 1 "$REPS"); do
  for f in "${INPUTS[@]}"; do
    for arm in OLD NEW; do
      probe="$OLD"; [ "$arm" = NEW ] && probe="$NEW"
      # ⛔ The instrument is OFF here on purpose: this arm prices what a PRODUCTION parse
      # pays for the change, not what a measurement run pays.
      env -u PGEN_REPORT_PARSE_FOOTPRINT /usr/bin/time -l "$probe" \
          --parse systemverilog "$f" --profile sv_2017 >/dev/null 2>"$W/t"
      rss=$(awk '/maximum resident set size/{print $1; exit}' "$W/t")
      wall=$(awk '/real[ \t]/{print $1; exit}' "$W/t")
      printf '%s\t%s\t%s\t%s\t%s\n' "$arm" "$f" "$rep" "${rss:-NA}" "${wall:-NA}" >> "$OUT"
    done
  done
done
