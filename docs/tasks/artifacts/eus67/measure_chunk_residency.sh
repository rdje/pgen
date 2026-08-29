#!/usr/bin/env bash
# docs/tasks/artifacts/eus67/measure_chunk_residency.sh — ENGINE-UNIVERSAL-SERVICES.67(b) leg 3.
#
# THE QUESTION: `ArenaChunkModel` reports what `typed_arena` took from the ALLOCATOR, proven
# byte-exact by `arena_chunk_oracle_probe`. `.67`(b) is attributing PEAK RSS. Those are the same
# number only if allocated-but-never-written chunk pages are resident — and on a lazy-commit OS
# they are not. Getting this wrong attributes the arena's unused slack to the arena, which on this
# repository's own ladder is up to 45 % of the arena's allocated bytes.
#
# THE EXPERIMENT, designed to need no floor model and no calibration: run the same workload at
# `N` and at `N + 1`, where `N` fills the last chunk EXACTLY. One extra item DOUBLES the allocated
# bytes (a fresh chunk of twice the size) and leaves the written bytes unchanged.
#
#   peak RSS(N+1) ~= peak RSS(N)      => slack is NOT resident; `touched` is the RSS-relevant figure
#   peak RSS(N+1) ~= 2 x peak RSS(N)  => slack IS resident; `allocated` is
#
# ⛔ Neither outcome is assumed: the script prints the ratio and names which hypothesis it
# supports, and REFUSES if the two model figures do not have the shape the experiment needs.
#
# Usage: measure_chunk_residency.sh [out.tsv]
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"
OUT="${1:-docs/tasks/artifacts/eus67/chunk_residency.tsv}"
PROBE="${PGEN_EUS67_ORACLE:-./rust/target/release/arena_chunk_oracle_probe}"
[ -x "$PROBE" ] || { echo "no oracle probe at $PROBE" >&2; exit 2; }

# 21 items per 1024-byte initial chunk for a 48-byte item, doubling: 21 * (2^20 - 1) fills
# every chunk exactly. Derived here rather than pasted so the two arms cannot drift apart.
N=$(( 21 * ( (1 << 20) - 1 ) ))
W="tmp/eus67b/.res.$$"; mkdir -p "$W"; trap 'rm -rf "$W"' EXIT

printf 'arm\titems\tpeak_rss_bytes\tallocated_MB\ttouched_MB\tslack_MB\tchunks\n' > "$OUT"
for n in "$N" $(( N + 1 )); do
  arm=full; [ "$n" -eq $(( N + 1 )) ] && arm=one_over
  /usr/bin/time -l "$PROBE" --residency "$n" >"$W/out" 2>"$W/t"
  rc=$?
  [ "$rc" -eq 0 ] || { echo "probe failed rc=$rc at n=$n" >&2; exit 2; }
  rss=$(awk '/maximum resident set size/{print $1; exit}' "$W/t")
  line=$(cat "$W/out")
  alloc=$(printf '%s' "$line" | sed -n 's/.*allocated=\([0-9.]*\) MB.*/\1/p')
  touch=$(printf '%s' "$line" | sed -n 's/.*touched=\([0-9.]*\) MB.*/\1/p')
  slack=$(printf '%s' "$line" | sed -n 's/.*slack=\([0-9.]*\) MB.*/\1/p')
  chunks=$(printf '%s' "$line" | sed -n 's/.*chunks=\([0-9]*\).*/\1/p')
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$arm" "$n" "${rss:-NA}" "${alloc:-NA}" \
      "${touch:-NA}" "${slack:-NA}" "${chunks:-NA}" >> "$OUT"
done

awk -F'\t' 'NR>1{rss[$1]=$3; alloc[$1]=$4; touch[$1]=$5}
END {
  if (rss["full"] == "" || rss["one_over"] == "") { print "REFUSE: a run produced no RSS"; exit 2 }
  ar = alloc["one_over"] / alloc["full"];
  tr = touch["one_over"] / touch["full"];
  rr = rss["one_over"] / rss["full"];
  printf "allocated ratio (one_over/full) = %.4f   [the experiment needs ~2]\n", ar;
  printf "touched   ratio                 = %.6f   [the experiment needs ~1]\n", tr;
  printf "peak RSS  ratio                 = %.4f\n", rr;
  if (ar < 1.8 || tr > 1.01) { print "REFUSE: the two arms do not have the shape this experiment needs"; exit 2 }
  if (rr < 1.2)      print "VERDICT: peak RSS tracks TOUCHED bytes - allocated-but-unwritten chunk slack is NOT resident.";
  else if (rr > 1.8) print "VERDICT: peak RSS tracks ALLOCATED bytes - chunk slack IS resident.";
  else               printf "VERDICT: INCONCLUSIVE - the RSS ratio %.4f sits between the two hypotheses.\n", rr;
}' "$OUT"
