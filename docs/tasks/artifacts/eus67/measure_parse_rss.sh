#!/usr/bin/env bash
# docs/tasks/artifacts/eus67/measure_parse_rss.sh
#
# ENGINE-UNIVERSAL-SERVICES.67 (a) — peak RSS of ONE parse, per file, as a TSV.
#
# ⛔ WHY A SEPARATE PROCESS PER FILE. Peak RSS is a per-process high-water mark, so a
# batch parse inside one process reports the max over the batch and attributes it to
# nothing. One `fork` per file is the price of a per-file number.
#
# ⛔ THE `time -l` REPORT MUST BE THE ONLY THING ON THE OUTER STDERR. The first version of
# this script redirected the probe's own stderr on the OUTER command — which is `/usr/bin/time`
# itself, so time's report went into that file and every RSS column came out `NA`. The probe's
# streams are therefore redirected INSIDE an `sh -c` child, which `time` cannot see.
#
# ⚠️ THE PROCESS FLOOR IS NOT A CONSTANT AND IS NOT SUBTRACTED HERE. This script reports the
# RAW high-water mark; `parse_rss_distribution.py` does the floor arithmetic and REFUSES to
# publish a ratio for a file whose peak does not dominate the floor. Measured on this host the
# floor spans 15.1-18.9 MB across inputs — a ~19 % band — so a subtraction on a sub-20 KB file
# is mostly a measurement of the loader.
#
# Usage: measure_parse_rss.sh <filelist> <out.tsv> [timeout_s] [profile]
# Env:   PGEN_EUS67_PROBE   parse binary (default ./rust/target/release/parseability_probe)
#
# ⛔ The profile is a MEASUREMENT PARAMETER, not a default to be forgotten: SystemVerilog
# declares no `@default_profile`, so an unnamed profile parses the UNION of every dialect
# (`ENGINE-UNIVERSAL-SERVICES.62`) and its numbers are NOT comparable with `sv_2017` numbers.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"
LIST="$1"; OUT="$2"; TMO="${3:-60}"; PROF="${4:-sv_2017}"
PROBE="${PGEN_EUS67_PROBE:-./rust/target/release/parseability_probe}"
[ -x "$PROBE" ] || { echo "no probe at $PROBE" >&2; exit 2; }
W="$(mktemp -d "${TMPDIR:-tmp}/eus67.XXXXXX")" || exit 2
trap 'rm -rf "$W"' EXIT
printf 'path\tbytes\tverdict\texit\treal_s\tpeak_rss_bytes\n' > "$OUT"
while IFS= read -r f; do
  [ -n "$f" ] || continue
  sz=$(wc -c < "$f" | tr -d ' ')
  /usr/bin/time -l timeout "$TMO" sh -c \
      'exec "$0" --parse systemverilog "$1" --profile "$2" >"$3" 2>"$4"' \
      "$PROBE" "$f" "$PROF" "$W/out" "$W/err" 2> "$W/time"
  rc=$?
  rss=$(awk '/maximum resident set size/{print $1; exit}' "$W/time")
  real=$(awk '/ real /{print $1; exit}' "$W/time")
  if   [ "$rc" -eq 124 ]; then v=TIMEOUT
  elif grep -q 'parse_full passed' "$W/out" 2>/dev/null; then v=ACCEPT
  else v=REJECT
  fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$f" "$sz" "$v" "$rc" "${real:-NA}" "${rss:-NA}" >> "$OUT"
done < "$LIST"
