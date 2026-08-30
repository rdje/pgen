#!/bin/bash
# ENGINE-UNIVERSAL-SERVICES.79(b)(c) — the reservation x depth outcome ladder.
#
# Nine nesting depths against three parse-stack reservations. For each cell it
# reports the exit code, the wall clock and the depth probe's own reading
# (`PGEN_REPORT_STACK_PROBE`, TOOLBOX.md 3.3d). The cell that matters is `rc`:
# 134 is an uncatchable SIGABRT (the parse thread overflowed before the guard
# could refuse); 1 is a clean, diagnosed rejection.
#
# Usage:  bash <this script> <path-to-parseability_probe>          # DEBUG build
#
# The scratch sources are written under the repository's own gitignored tmp/ so
# every byte this produces stays on the repository volume (directive 13).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 1

probe="${1:?usage: reservation_depth_ladder.sh <path-to-parseability_probe>}"
work="tmp/eus79/src"
depths=(200 275 300 350 400 500 600 700 800)
# 96 MiB is MIN_DEDICATED_PARSE_STACK_BYTES (the module's documented floor and
# the value every smaller request is clamped up to); 256 MiB is the shipped
# default; 128 MiB is the intermediate point where behaviour could move and
# did not.
reservations=(100663296 134217728 268435456)

mkdir -p "$work"
for n in "${depths[@]}"; do
  f="$work/p_$n.sv"
  [ -f "$f" ] || python3 -c "
open('$f','w').write('module m; wire w; assign w = ' + '('*$n + '1' + ')'*$n + '; endmodule\n')"
done

now() { python3 -c 'import time; print(time.time())'; }

printf "%-12s %-5s %-4s %-9s %s\n" RESERVATION LEVELS RC ELAPSED VERDICT
for res in "${reservations[@]}"; do
  for n in "${depths[@]}"; do
    start="$(now)"
    out="$(PGEN_REPORT_STACK_PROBE=1 PGEN_PARSE_STACK_BYTES="$res" \
             "$probe" --parse systemverilog --profile sv_2017 "$work/p_$n.sv" 2>&1)"; rc=$?
    end="$(now)"
    if [ "$rc" -eq 134 ]; then
      verdict="SIGABRT"
    else
      verdict="$(printf '%s\n' "$out" \
        | grep -o 'stack_probe: [^ ]*\( cause=[A-Za-z]*\)\?\( depth=[0-9]*\)\?' | head -1)"
    fi
    accepted="$(printf '%s\n' "$out" | grep -c 'parse_full passed')"
    printf "%-12s %-5s %-4s %-9s %s accepted=%s\n" \
      "$res" "$n" "$rc" "$(python3 -c "print(f'{$end-$start:.2f}')")" "$verdict" "$accepted"
  done
done
