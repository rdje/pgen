#!/usr/bin/env bash
# docs/tasks/artifacts/eus67c/measure_thin_residency.sh — ENGINE-UNIVERSAL-SERVICES.67(c).
#
# ⛔ THE QUESTION THIS SETTLES, AND WHY THE AGGREGATE COULD NOT.
# `.67`(c)(i) measured the thin-memo table at 0.07 %-4.6 % OCCUPANCY: the parse writes
# almost none of it. Whether the never-written part is RESIDENT is a different question,
# and it decides whether the cost is real. An attempt to settle it from the ladder's own
# aggregate (`peak_live` vs `peak_rss - floor`, with leg 3's proved arena-slack result
# subtracted) DOES NOT WORK and was discarded: the identity leaves
# `thin_nonresident <= V - 348 MB` with `V` the unmeasured allocator/non-heap overhead,
# so it bounds nothing without a second unknown. It looked decisive (a saturated bound on
# 11 of 11) and was an inference dressed as a measurement.
#
# ⭐ THE ISOLATING SHAPE INSTEAD. The table is sized from input LENGTH alone
# (`THIN_RULE_COUNT * (len + 1)` slots), so a file that is almost entirely COMMENT makes
# it huge while the parse does essentially no work: no AST, no memo growth, no tape. Then
# peak RSS above the floor is dominated by ONE term. Three sizes give a SLOPE, which is
# what decides it:
#   slope ~= 872 B per input byte  => the table's untouched pages ARE resident
#   slope ~= 0                     => they are lazy and the cost is address space only
# The slope is the invariant, not the level — the same reasoning leg 3's `N`/`N+1`
# residency experiment used.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"
OUT="${1:-docs/tasks/artifacts/eus67c/thin_residency.tsv}"
REPS="${2:-3}"
PROBE="${PGEN_EUS67_ALLOC_PROBE:-./rust/target/release/parse_alloc_census_probe}"
[ -x "$PROBE" ] || { echo "no alloc-census probe at $PROBE" >&2; exit 2; }
W="tmp/eus67c/.res.$$"; mkdir -p "$W"; trap 'rm -rf "$W"' EXIT
printf 'path\tbytes\trep\tverdict\tpeak_rss_bytes\tpeak_live\tthin_need_slots\tthin_need_bytes\tthin_distinct\tthin_occupancy\n' > "$OUT"
for rep in $(seq 1 "$REPS"); do
  for f in tmp/eus67c/shapes/comment_*.sv; do
    [ -r "$f" ] || { echo "no input at $f" >&2; exit 2; }
    sz=$(wc -c < "$f" | tr -d ' ')
    PGEN_REPORT_PARSE_FOOTPRINT=1 /usr/bin/time -l sh -c \
        'exec "$0" systemverilog "$1" --profile sv_2017 >"$2" 2>"$3"' \
        "$PROBE" "$f" "$W/out" "$W/err" 2> "$W/time"
    rss=$(awk '/maximum resident set size/{print $1; exit}' "$W/time")
    cen=$(grep -m1 -E 'PARSE[- ]ALLOC[- ]CENSUS' "$W/out")
    occ=$(grep -m1 'PARSE-THIN-OCCUPANCY' "$W/out")
    kv() { printf '%s' "$2" | awk -v want="$1" '{for(i=1;i<=NF;i++){n=index($i,"=");if(n>0&&substr($i,1,n-1)==want){print substr($i,n+1);exit}}}'; }
    if grep -q 'parse_full passed' "$W/out"; then v=ACCEPT; else v=REJECT; fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$f" "$sz" "$rep" "$v" "${rss:-NA}" \
        "$(kv peak_live "$cen")" "$(kv need_slots "$occ")" "$(kv need_bytes "$occ")" \
        "$(kv distinct_slots "$occ")" "$(kv occupancy "$occ")" >> "$OUT"
  done
done
