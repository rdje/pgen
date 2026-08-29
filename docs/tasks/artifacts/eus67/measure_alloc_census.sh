#!/usr/bin/env bash
# docs/tasks/artifacts/eus67/measure_alloc_census.sh — the ALLOCATOR-SIDE leg of the
# parse-footprint attribution. ENGINE-UNIVERSAL-SERVICES.67(b) LEG 4.
#
# ⭐ THE JOIN IS INSIDE ONE PROCESS, WHICH IS THE POINT. `parse_alloc_census_probe`
# calls the same `parse_sample_detail_with_options` entry point `parseability_probe
# --parse` calls, so with `PGEN_REPORT_PARSE_FOOTPRINT=1` the engine's arena +
# owned-heap census (legs 1-3, stderr) and this allocator census (leg 4, stdout)
# describe THE SAME PARSE. Nothing here pairs two runs and hopes they agree.
#
# ⛔ NA, never 0, for an unmatched field — an unmatched field is a MISSING
# measurement and writing it as 0 is the confusion the instrument's own `Option`
# guard exists to prevent (`-0121`).
#
# Usage: measure_alloc_census.sh <filelist> <out.tsv> <out_classes.tsv> [timeout_s]
set -uo pipefail
# FOUR levels up, not two — the `-0124` walk defect, spelled the same way as its
# siblings so a copy of this file cannot repeat it.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"
LIST="$1"; OUT="$2"; OUTC="$3"; TMO="${4:-600}"
PROBE="${PGEN_EUS67_ALLOC_PROBE:-./rust/target/release/parse_alloc_census_probe}"
[ -x "$PROBE" ] || { echo "no alloc-census probe at $PROBE" >&2; exit 2; }
[ -r "$LIST" ]  || { echo "no file list at $LIST" >&2; exit 2; }
W="tmp/eus67b/.ac.$$"; mkdir -p "$W"; trap 'rm -rf "$W"' EXIT

printf 'path\tbytes\tverdict\tpeak_rss_bytes\tpeak_live\tsnap_live\tsnap_gap\tsnapshots\tend_live' >  "$OUT"
printf '\tlive_before_parse\talloc_calls\talloc_bytes\tfree_calls\tfree_bytes\tparse_alloc_calls\tchurn_ratio' >> "$OUT"
printf '\tarm_verified_calls\tpeak_at_alloc_calls\tpeak_at_free_calls\tpeak_at_alloc_bytes' >> "$OUT"
printf '\tpost_peak_alloc_calls\tpost_peak_alloc_bytes\tpost_peak_free_calls' >> "$OUT"
printf '\tmemo_entry_B\tmemo_bucket_B\tmemo_fail_bucket_B\ttape_word_B' >> "$OUT"
printf '\tarena_allocated_MB\tarena_touched_MB\tarena_ast_touched_MB\tarena_ast_allocated_MB\towned_MB' >> "$OUT"
printf '\tchunk_nodes_cap\tchunk_sv_cap\tchunk_sp_cap\tchunk_rs_cap' >> "$OUT"
printf '\tthin_armed\tthin_need_slots\tthin_capacity_slots\tthin_distinct_slots' >> "$OUT"
printf '\tthin_store_calls\tthin_clear_calls\tthin_need_bytes\tthin_occupancy\tthin_distinct_exact\n' >> "$OUT"
printf 'path\tclass\tlo\thi\tlive_bytes\tlive_count\tshare\n' > "$OUTC"

while IFS= read -r f; do
  [ -n "$f" ] || continue
  [ -r "$f" ] || { echo "skip unreadable: $f" >&2; continue; }
  sz=$(wc -c < "$f" | tr -d ' ')
  PGEN_REPORT_PARSE_FOOTPRINT=1 /usr/bin/time -l timeout "$TMO" sh -c \
      'exec "$0" systemverilog "$1" --profile sv_2017 >"$2" 2>"$3"' \
      "$PROBE" "$f" "$W/out" "$W/err" 2> "$W/time"
  rc=$?
  rss=$(awk '/maximum resident set size/{print $1; exit}' "$W/time")
  cen=$(grep -m1 -E 'PARSE[- ]ALLOC[- ]CENSUS' "$W/out" 2>/dev/null)
  tsz=$(grep -m1 -E 'PARSE[- ]ALLOC[- ]TYPE[- ]SIZES' "$W/out" 2>/dev/null)
  # ENGINE-UNIVERSAL-SERVICES.67(c)(i) — the thin-memo OCCUPANCY line. Absent on a
  # probe built before that slice, in which case every column below stays NA.
  occ=$(grep -m1 'PARSE-THIN-OCCUPANCY' "$W/out" 2>/dev/null)
  # The engine-side lines, from the SAME process.
  fline=$(grep -m1 'PARSE FOOTPRINT' "$W/err" 2>/dev/null)
  cline=$(grep -m1 'chunk capacity' "$W/err" 2>/dev/null)

  # ⛔⛔ EXACT KEY MATCH, NOT A GREEDY `.*key=`. The census line carries both
  # `alloc_calls=` and `post_peak_alloc_calls=`; a greedy `.*alloc_calls=` matches
  # the LAST one, so the "alloc_calls" column would silently have held the
  # post-peak figure. That is the same class of defect `-0126` recorded for the
  # second footprint line, met again in a different file, so the extraction is a
  # whitespace split with an exact `key=` test rather than a pattern that can
  # slide. `kv()` returns NA when the key is absent — never 0.
  kv() {
    printf '%s' "$2" | awk -v want="$1" '{
      for (i = 1; i <= NF; i++) {
        n = index($i, "=")
        if (n > 0 && substr($i, 1, n - 1) == want) { print substr($i, n + 1); exit }
      }
    }'
  }

  pl=$(kv peak_live "$cen");           sl=$(kv snapshot_live "$cen")
  sg=$(kv snapshot_gap "$cen");        ns=$(kv snapshots "$cen")
  el=$(kv end_live "$cen");            lb=$(kv live_before_parse "$cen")
  ac=$(kv alloc_calls "$cen");         ab=$(kv alloc_bytes "$cen")
  fc=$(kv free_calls "$cen");          fb=$(kv free_bytes "$cen")
  pac=$(kv parse_alloc_calls "$cen");  cr=$(kv churn_ratio "$cen")
  arm=$(kv arm_verified_calls "$cen")
  pkac=$(kv peak_at_alloc_calls "$cen"); pkfc=$(kv peak_at_free_calls "$cen")
  pkab=$(kv peak_at_alloc_bytes "$cen")
  ppac=$(kv post_peak_alloc_calls "$cen"); ppab=$(kv post_peak_alloc_bytes "$cen")
  ppfc=$(kv post_peak_free_calls "$cen")
  # The candidate structures' item sizes, MEASURED by the probe. `MemoEntry`'s layout
  # is not hand-derivable (four `Option`s whose niches the compiler may pack), so a
  # hand-computed bucket size would be an inference dressed as a measurement.
  tsz_memo=$(kv MemoEntry "$tsz");        tsz_bucket=$(kv memo_bucket "$tsz")
  tsz_failb=$(kv memo_fail_bucket "$tsz"); tsz_tape=$(kv TapeWord "$tsz")
  # ⛔ `occupancy` may read NOT_MEASURED rather than a number — a disarmed census
  # and an untouched table are different facts, and the probe refuses to conflate
  # them. It is carried through verbatim, never coerced to 0.
  th_arm=$(kv armed "$occ");            th_need=$(kv need_slots "$occ")
  th_cap=$(kv capacity_slots "$occ");   th_dist=$(kv distinct_slots "$occ")
  th_st=$(kv store_calls "$occ");       th_cl=$(kv clear_calls "$occ")
  th_nb=$(kv need_bytes "$occ");        th_occ=$(kv occupancy "$occ")
  th_ex=$(kv distinct_exact "$occ")

  # The engine-side lines use their own anchors and are extracted with the same
  # `-0126` discipline: `grep -m1` selects the line, and each pattern names a token
  # that appears once on it.
  aalloc=$(printf '%s' "$cline" | sed -n 's/.*allocated total \([0-9.]*\) MB.*/\1/p')
  atouch=$(printf '%s' "$cline" | sed -n 's/.*touched total \([0-9.]*\) MB.*/\1/p')
  aast=$(printf '%s'   "$cline" | sed -n 's/.*arena+AST touched \([0-9.]*\) MB.*/\1/p')
  aasa=$(printf '%s'   "$cline" | sed -n 's/.*\.\. allocated \([0-9.]*\) MB.*/\1/p')
  omb=$(printf '%s'    "$fline" | sed -n 's/.*AST-owned heap total \([0-9.]*\) MB.*/\1/p')
  # ⭐ THE PER-ARENA CAPACITIES, so the attribution pass can derive the EXACT multiset
  # of chunk byte sizes instead of allowing any doubling-sequence value. `typed_arena`
  # chunk capacities are `first, 2*first, 4*first, ...` with `first = 1024 / item`, so
  # `cap = first * (2^chunks - 1)` pins both the count and every individual size. Without
  # this the decomposition can label an allocation an "arena chunk" that the arena never
  # made — measured: it over-assigned 369.66 MB to an arena the engine's own
  # oracle-proved model says allocated 317.13 MB.
  ncap=$(printf '%s' "$cline" | sed -n 's/.*nodes cap=\([0-9]*\).*/\1/p')
  svcap=$(printf '%s' "$cline" | sed -n 's/.*shaped_values cap=\([0-9]*\).*/\1/p')
  spcap=$(printf '%s' "$cline" | sed -n 's/.*shaped_pairs cap=\([0-9]*\).*/\1/p')
  rscap=$(printf '%s' "$cline" | sed -n 's/.*rendered_strings cap=\([0-9]*\).*/\1/p')

  if [ "$rc" -eq 124 ]; then v=TIMEOUT
  elif grep -q 'parse_full passed' "$W/out" 2>/dev/null; then v=ACCEPT
  else v=REJECT; fi

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s' "$f" "$sz" "$v" "${rss:-NA}" \
      "${pl:-NA}" "${sl:-NA}" "${sg:-NA}" "${ns:-NA}" "${el:-NA}" >> "$OUT"
  printf '\t%s\t%s\t%s\t%s\t%s\t%s\t%s' "${lb:-NA}" "${ac:-NA}" "${ab:-NA}" \
      "${fc:-NA}" "${fb:-NA}" "${pac:-NA}" "${cr:-NA}" >> "$OUT"
  printf '\t%s\t%s\t%s\t%s' "${arm:-NA}" "${pkac:-NA}" "${pkfc:-NA}" "${pkab:-NA}" >> "$OUT"
  printf '\t%s\t%s\t%s' "${ppac:-NA}" "${ppab:-NA}" "${ppfc:-NA}" >> "$OUT"
  printf '\t%s\t%s\t%s\t%s' "${tsz_memo:-NA}" "${tsz_bucket:-NA}" "${tsz_failb:-NA}" "${tsz_tape:-NA}" >> "$OUT"
  printf '\t%s\t%s\t%s\t%s\t%s' "${aalloc:-NA}" "${atouch:-NA}" \
      "${aast:-NA}" "${aasa:-NA}" "${omb:-NA}" >> "$OUT"
  printf '\t%s\t%s\t%s\t%s' "${ncap:-NA}" "${svcap:-NA}" "${spcap:-NA}" "${rscap:-NA}" >> "$OUT"
  printf '\t%s\t%s\t%s\t%s' "${th_arm:-NA}" "${th_need:-NA}" "${th_cap:-NA}" "${th_dist:-NA}" >> "$OUT"
  printf '\t%s\t%s\t%s\t%s\t%s\n' "${th_st:-NA}" "${th_cl:-NA}" "${th_nb:-NA}" \
      "${th_occ:-NA}" "${th_ex:-NA}" >> "$OUT"

  # The size-class histogram at the near-peak snapshot, one row per non-empty class.
  # ⛔⛔ KEYED ON UNPADDED `key=value` TOKENS, AND THE FIRST CUT OF THIS BLOCK IS WHY.
  # The probe originally emitted a human-aligned table (`live_bytes={:>14}`), which
  # puts WHITESPACE between the key and its number — so a whitespace-split reader
  # took the bare token `live_bytes=` and left the column EMPTY, on every row, at
  # exit 0. The rehearsal ladder caught it; the fix is at the EMISSION site (values
  # unpadded, alignment is this analysis' job), not a cleverer pattern here.
  awk -v p="$f" '
    /^  ALLOC-CLASS / {
      cls=lo=hi=lb2=lc=sh="NA"
      for (i = 1; i <= NF; i++) {
        n = index($i, "=")
        if (n <= 0) continue
        key = substr($i, 1, n - 1); val = substr($i, n + 1)
        if      (key == "class")      cls = val
        else if (key == "lo")         lo  = val
        else if (key == "hi")         hi  = val
        else if (key == "live_bytes") lb2 = val
        else if (key == "live_count") lc  = val
        else if (key == "share")      sh  = val
      }
      printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\n", p, cls, lo, hi, lb2, lc, sh
    }' "$W/out" >> "$OUTC"
done < "$LIST"
