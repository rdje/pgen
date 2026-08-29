#!/usr/bin/env bash
# docs/tasks/artifacts/eus67c/measure_zero_fill_residency.sh — ENGINE-UNIVERSAL-SERVICES.67(c)(iii).
#
# Prices the ONE candidate fix for the dense thin-memo table that is not a trade:
# stop faulting in what is never written. Three arms over the same table size and the
# same number of written slots:
#   resize        `Vec::resize(n, 0)`  — what `ThinMemoScratch::begin` does today
#   zeroed        `vec![0u64; n]`      — specialises to `alloc_zeroed`
#   reserve_only  `Vec::with_capacity(n)`, nothing written — THE CONTROL
#
# ⛔ THE CONTROL IS THE POINT. If `resize` and `reserve_only` read the same peak RSS,
# the zero-fill is NOT what makes the pages resident and this entire fix direction is
# refuted. A pricing without an arm that can refute it is an advertisement.
#
# ⛔⛔ AND IT MUST RUN ON AN IDLE MACHINE. `-0128` measured peak RSS reading 13.6-27.9 %
# LOW on exactly the inputs peaking above ~1.9 GB while a 10 GB build was compiling —
# silent, plausible, wrong. Nothing in the harness enforces idleness, so this driver
# REFUSES to run when the 1-minute load average is above its ceiling, and says so.
#
# Usage: measure_zero_fill_residency.sh <out.tsv> [reps]
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"
OUT="${1:-docs/tasks/artifacts/eus67c/zero_fill_residency.tsv}"
REPS="${2:-3}"
# 114 295 329 slots = 914 362 632 B: the table `comment_1024k.sv` asks for
# (109 rules x 1 048 581 positions), the exact size `.67`(c)(ii) measured at 906 MB resident.
SLOTS="${PGEN_EUS67_ZF_SLOTS:-114295329}"
LOAD_CEIL="${PGEN_EUS67_LOAD_CEILING:-2.0}"
BIN="tmp/eus67c/zero_fill_residency_probe"
SRC="docs/tasks/artifacts/eus67c/zero_fill_residency_probe.rs"
mkdir -p tmp/eus67c
if [ ! -x "$BIN" ] || [ "$SRC" -nt "$BIN" ]; then
  rustc -O -o "$BIN" "$SRC" || { echo "rustc failed on $SRC" >&2; exit 2; }
fi

load=$(uptime | sed -E 's/.*load averages?: *//' | awk '{print $1}' | tr -d ',')
if awk -v l="$load" -v c="$LOAD_CEIL" 'BEGIN{exit !(l > c)}'; then
  echo "⛔ REFUSING: 1-minute load average ${load} exceeds the ${LOAD_CEIL} ceiling." >&2
  echo "   Peak RSS on a busy machine reads LOW and silently so (-0128). Re-run idle, or set" >&2
  echo "   PGEN_EUS67_LOAD_CEILING deliberately and say so where the number is quoted." >&2
  exit 3
fi

# THREE SCENARIOS, because the saving is a BRACKET and not a number. `written` at 4.58 % is
# the WORST occupancy `.67`(c)(i) measured on any ladder input.
#   isolating  14 writes, page stride   — the shape `.67`(c)(ii) measured at 906 MB resident
#   clustered  4.58 %, stride 1         — contiguous writes; pages touched ~= occupancy
#   scattered  4.58 %, page stride      — one write per page; EVERY page touched, lazy saves 0
OCC=$(awk -v s="$SLOTS" 'BEGIN{printf "%d", s * 0.0458}')
printf 'scenario	arm	slots	written	stride	rep	table_bytes	touched	distinct_pages	table_pages	page_coverage	peak_rss_bytes	load_at_start
' > "$OUT"
kv() { printf '%s' "$2" | awk -v want="$1" '{for(i=1;i<=NF;i++){n=index($i,"=");if(n>0&&substr($i,1,n-1)==want){print substr($i,n+1);exit}}}'; }
for rep in $(seq 1 "$REPS"); do
  for scen in "isolating 14 512" "clustered $OCC 1" "scattered $OCC 512"; do
    set -- $scen; name="$1"; w="$2"; st="$3"
    for arm in resize zeroed reserve_only; do
      aw="$w"; [ "$arm" = "reserve_only" ] && aw=0
      /usr/bin/time -l "$BIN" "$arm" "$SLOTS" "$aw" "$st" > tmp/eus67c/.zf.out 2> tmp/eus67c/.zf.time
      rss=$(awk '/maximum resident set size/{print $1; exit}' tmp/eus67c/.zf.time)
      line=$(grep -m1 'ZERO-FILL-PROBE' tmp/eus67c/.zf.out)
      printf '%s	%s	%s	%s	%s	%s	%s	%s	%s	%s	%s	%s	%s
' \
        "$name" "$arm" "$SLOTS" "$aw" "$st" "$rep" \
        "$(kv table_bytes "$line")" "$(kv touched "$line")" "$(kv distinct_pages "$line")" \
        "$(kv table_pages "$line")" "$(kv page_coverage "$line")" "${rss:-NA}" "$load" >> "$OUT"
    done
  done
done
rm -f tmp/eus67c/.zf.out tmp/eus67c/.zf.time
echo "wrote $OUT"
