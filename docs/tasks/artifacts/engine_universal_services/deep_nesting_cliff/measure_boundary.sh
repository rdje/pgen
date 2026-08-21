#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.43 — the ACCEPT-BOUNDARY + ceiling-contact control.
#
# For each depth it prints the verdict, the wall clock, and — from
# `PGEN_REPORT_MEMO_STATS` — how many times that parse crossed the engine's
# whole-stack recursion ceiling.
#
# The point is the last column. The depth-gated memo this leaf adds is reachable
# ONLY after a ceiling rejection, so any parse reporting `ceiling=0` runs code
# that is byte-for-byte the pre-fix path. A sweep in which every ACCEPTED depth
# reports `ceiling=0` therefore proves the accept set below the ceiling is
# untouched — which a timing comparison alone could never show.
set -u
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
PROBE="$ROOT/rust/target/release/parseability_probe"
CAP="${1:-60}"; shift || true
DEPTHS=("$@"); [ ${#DEPTHS[@]} -eq 0 ] && DEPTHS=($(seq 300 1 325))
TMP="$(mktemp -d "$ROOT/docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/.boundary.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT
[ -x "$PROBE" ] || { echo "missing $PROBE"; exit 2; }
printf '%6s %10s  %-10s %s\n' depth wall verdict 'ceiling rejections / depth-gated entries'
for n in "${DEPTHS[@]}"; do
  python3 -c "import sys;n=$n;sys.stdout.write('module m; assign x = %s1%s; endmodule' % ('('*n, ')'*n))" > "$TMP/in.txt"
  start=$(python3 -c 'import time;print(time.monotonic())')
  PGEN_REPORT_MEMO_STATS=1 timeout "$CAP" "$PROBE" --parse systemverilog "$TMP/in.txt" \
      --profile sv_2017 >"$TMP/out.txt" 2>"$TMP/err.txt"
  rc=$?
  el=$(python3 -c "import time;print(f'{time.monotonic()-$start:.2f}')")
  case $rc in 0) v=accepted ;; 124) v="NO-RESULT" ;; *) v="rejected" ;; esac
  stats=$(grep -o '[0-9]* depth-gated' "$TMP/err.txt" | tail -1)
  ceil=$(grep -o '[0-9]* depth-ceiling rejections' "$TMP/err.txt" | tail -1)
  printf '%6s %9ss  %-10s ceiling=%-28s %s\n' "$n" "$el" "$v" "${ceil:-<no stats line>}" "${stats:-}"
done
