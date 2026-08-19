#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2k (a) — the VERDICT of every pinned reproducer under the probe now on disk.
#
# ⛔ USE THE EXIT CODE, NEVER THE MESSAGE. The first cut of this check grepped the probe's stdout
# for /accept/ and reported the three `accepts_invalid_keyword_*` rows as ACCEPT under an arm that
# in fact REJECTS them — because the probe echoes the FILE PATH in its rejection message and the
# path contains the word "accepts". A verdict oracle that can be fooled by a filename is not an
# oracle; `parseability_probe --parse` exits 0 on ACCEPT and 1 on REJECT, and that is what this
# reads.
#
# USAGE  bash …/arm_verdicts.sh [probe-path]
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
[ -d "$ROOT/grammars" ] || { echo "arm_verdicts: not at the repo root (derived $ROOT)" >&2; exit 2; }
cd "$ROOT"
PROBE="${1:-rust/target/debug/parseability_probe}"
[ -x "$PROBE" ] || { echo "arm_verdicts: no probe at $PROBE" >&2; exit 2; }

echo "arm_verdicts: probe=$PROBE parser=$("$PROBE" --parser-fingerprint \
  | python3 -c 'import json,sys;print(json.load(sys.stdin)["parsers"]["systemverilog"])')"
printf '%-56s %-14s %s\n' FILE PROFILE VERDICT
for f in rust/target/tk_pending/*.sv \
         stimuli/sv/adjudication_repros/accepts_invalid_keyword_component_indexed.sv \
         stimuli/sv/adjudication_repros/accepts_invalid_keyword_component_plain.sv \
         stimuli/sv/adjudication_repros/accepts_invalid_keyword_first_component.sv; do
  [ -f "$f" ] || continue
  for prof in sv_2017 sv_2023 verilog_2005; do
    if "$PROBE" --parse systemverilog "$f" --profile "$prof" >/dev/null 2>&1; then v=ACCEPT; else v=REJECT; fi
    printf '%-56s %-14s %s\n' "$(basename "$f")" "$prof" "$v"
  done
done
