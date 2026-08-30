#!/bin/bash
# `ENGINE-UNIVERSAL-SERVICES.84` — how a parse's PEAK RSS grows with source nesting depth,
# per family, with SystemVerilog as the control.
#
# WHY IT EXISTS. `.79`(a)'s release census was killed by the memory guard at 12 561 MB while
# measuring `return_annotation`. Chasing that found peak RSS growing QUADRATICALLY in nesting
# depth on that family while the AST arena grows LINEARLY — so the memory is not the AST — and
# `systemverilog` on the same shape at the same reservation grows LINEARLY, which is what makes
# this a family-specific finding rather than an engine-wide one.
#
# ⛔ RSS IS NOT DETERMINISTIC AND READS LOW ON A BUSY MACHINE (`-0128`: −13.6/−27.9/−25.8 % on
# exactly the inputs peaking above ~1.9 GB). Run this on an idle host, quote `uptime`, and treat
# any reading above ~1.9 GB as a LOWER BOUND. The RATIOS between rungs survive an under-read in
# the conservative direction, which is why the published conclusion is a growth EXPONENT and not
# an absolute byte count.
#
# Usage:  bash <this script> <path-to-release-parseability_probe>
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 1
probe="${1:?usage: memory_depth_ladder.sh <path-to-parseability_probe>}"
gen="docs/tasks/artifacts/engine_universal_services/eus79a_per_frame_census/gen_deep_input.py"
out="tmp/eus84/mem"; mkdir -p "$out"

uptime
printf '%-20s %-7s %-9s %-12s %-10s %s\n' FAMILY LEVELS INPUT_B PEAK_RSS_MB RSS_PER_B WALL_S
for spec in "return_annotation:" "systemverilog:sv_2017"; do
  family="${spec%%:*}"; profile="${spec##*:}"
  for n in 1024 2048 4096; do
    src="$(python3 "$gen" "$family" "$n" --out-dir "$out")" || continue
    b=$(wc -c < "$src")
    if [ -n "$profile" ]; then
      o=$( { /usr/bin/time -l timeout 900 "$probe" --parse "$family" "$src" --profile "$profile" >/dev/null; } 2>&1 )
    else
      o=$( { /usr/bin/time -l timeout 900 "$probe" --parse "$family" "$src" >/dev/null; } 2>&1 )
    fi
    r=$(printf '%s\n' "$o" | awk '/maximum resident/{print $1}')
    t=$(printf '%s\n' "$o" | awk '/ real /{print $1}')
    printf '%-20s %-7s %-9s %-12.1f %-10.0f %s\n' \
      "$family" "$n" "$b" "$(echo "${r:-0} 1048576" | awk '{print $1/$2}')" \
      "$(echo "${r:-0} $b" | awk '{print $1/$2}')" "${t:-?}"
  done
done
uptime
