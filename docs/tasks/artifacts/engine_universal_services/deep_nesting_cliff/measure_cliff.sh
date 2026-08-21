#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.43 — re-runnable cliff measurement.
#
# Times the SHIPPED release probe on N-deep parenthesised expressions for
# SystemVerilog and VHDL, printing one row per (family, depth):
#     family depth wall_clock verdict
# `verdict` is `accepted`, `rejected`, or `NO-RESULT(killed at ${CAP}s)`.
#
# Usage:  bash docs/tasks/.../measure_cliff.sh [cap_seconds] [depths...]
set -u
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
PROBE="$ROOT/rust/target/release/parseability_probe"
CAP="${1:-60}"; shift || true
DEPTHS=("$@"); [ ${#DEPTHS[@]} -eq 0 ] && DEPTHS=(250 300 315 320 350 400 800 2000)
TMP="$(mktemp -d "$ROOT/docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/.measure.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT
[ -x "$PROBE" ] || { echo "missing $PROBE — build it with:"; \
  echo "  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)"; exit 2; }
echo "# parser-fingerprint: $("$PROBE" --parser-fingerprint 2>&1 | tr '\n' ' ' | cut -c1-200)"
printf '%-16s %6s %10s  %s\n' family depth wall verdict
for spec in "systemverilog:module m; assign x = %s1%s; endmodule:sv_2017" \
            "vhdl:entity e is end entity; architecture a of e is begin s <= %s1%s; end architecture;:vhdl_1076_2019"; do
  fam="${spec%%:*}"; rest="${spec#*:}"; tmpl="${rest%:*}"; prof="${rest##*:}"
  for n in "${DEPTHS[@]}"; do
    python3 -c "import sys;n=$n;sys.stdout.write('''$tmpl''' % ('('*n, ')'*n))" > "$TMP/in.txt"
    start=$(python3 -c 'import time;print(time.monotonic())')
    timeout "$CAP" "$PROBE" --parse "$fam" "$TMP/in.txt" --profile "$prof" >/dev/null 2>&1
    rc=$?
    el=$(python3 -c "import time;print(f'{time.monotonic()-$start:.2f}')")
    case $rc in
      0)   v=accepted ;;
      124) v="NO-RESULT(killed at ${CAP}s)" ;;
      *)   v="rejected(rc=$rc)" ;;
    esac
    printf '%-16s %6s %9ss  %s\n' "$fam" "$n" "$el" "$v"
  done
done
