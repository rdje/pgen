#!/usr/bin/env bash
# `ENGINE-UNIVERSAL-SERVICES.87`(d) — BISECT the source-level accept boundary at the
# reservation this host derives, with NO environment variable set.
#
# ⛔ A boundary is bisected, never sampled: a grid gives you (last OK, first FAIL) and
# nothing else. That rule is this leaf's own lesson
# (docs/knowledge/a-number-read-out-of-context-carries-a-frame-you-did-not-measure.md),
# earned by publishing a grid point as a spawn ceiling and being wrong by 2x.
#
#   bash docs/tasks/artifacts/engine_universal_services/eus87d_uncapped_reservation/bisect_boundary.sh 7000 7600
set -uo pipefail
PROBE=${PROBE:-rust/target/release/parseability_probe}
WORK=${WORK:-preserved_probes/eus87d}
mkdir -p "$WORK"
lo=${1:-7000}   # known ACCEPT
hi=${2:-7600}   # known REJECT

verdict() { # -> 0 accept, 1 reject
  local n=$1 f="$WORK/bisect_$1.sv"
  python3 -c "
n=$n
open('$f','w').write(\"module m;\n  assign x = \" + \"int'(\"*n + '1' + ')'*n + ';\nendmodule\n')"
  PGEN_REPORT_STACK_PROBE=1 timeout 900 "$PROBE" --parse systemverilog "$f" --profile sv_2017 \
    >/dev/null 2>"$WORK/bisect_$1.err"
  local rc=$?
  rm -f "$f"
  echo "  n=$n rc=$rc $(grep -o 'cause=[A-Za-z]* depth=[0-9]*' "$WORK/bisect_$1.err" | head -1)" >&2
  return $rc
}

echo "bisecting the accept boundary in ($lo accept, $hi reject]" >&2
while [ $((hi - lo)) -gt 1 ]; do
  mid=$(( (lo + hi) / 2 ))
  if verdict "$mid"; then lo=$mid; else hi=$mid; fi
done
echo
echo "DEEPEST ACCEPTED = $lo    SHALLOWEST REJECTED = $hi"
echo "refusal reading at $hi: $(grep -o 'stack_probe:.*' "$WORK/bisect_$hi.err" | head -1)"
