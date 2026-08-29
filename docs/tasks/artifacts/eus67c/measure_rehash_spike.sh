#!/usr/bin/env bash
# docs/tasks/artifacts/eus67c/measure_rehash_spike.sh — ENGINE-UNIVERSAL-SERVICES.67(c)(iii).
#
# ⛔ WHY A SECOND DRIVER RATHER THAN A COLUMN ON THE FIRST. `measure_alloc_census.sh`
# emits ONE ROW PER INPUT; the lifetime tape emits ONE ROW PER LARGE ALLOCATION, and
# squeezing a variable-length table into a fixed-width row is how raw data stops being
# raw. This driver writes three tracked files that JOIN on `path`:
#   <out_tape>   one row per recorded allocation: size, alloc ordinal, free ordinal
#   <out_meta>   one row per input: the tape header + the PEAK and SNAPSHOT ordinals
#   <out_class>  one row per size class, re-emitted from the SAME run as the tape
#
# ⭐ THE THIRD FILE IS THE CONTROL AND THAT IS THE ONLY REASON IT IS HERE. The class
# table and the tape are maintained by two independent code paths in the probe, so the
# tape's reconstruction of "live at the snapshot instant" must reproduce the class table
# byte-for-byte in every class at or above the tape threshold. Re-emitting it from the
# same process is what makes that comparison a control rather than a cross-run guess.
#
# ⛔ NA, never 0, for an unmatched field — an unmatched field is a MISSING measurement.
#
# Usage: measure_rehash_spike.sh <filelist> <out_tape.tsv> <out_meta.tsv> <out_class.tsv> [timeout_s]
set -uo pipefail
# FOUR levels up, not two — the `-0124` walk defect, spelled the same way as its
# siblings so a copy of this file cannot repeat it.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"
LIST="$1"; OUTT="$2"; OUTM="$3"; OUTC="$4"; TMO="${5:-900}"
PROBE="${PGEN_EUS67_ALLOC_PROBE:-./rust/target/release/parse_alloc_census_probe}"
# ⛔ THE TRACKED LADDER RUNS AT 16 KiB, AND THE NUMBER IS DERIVED RATHER THAN PICKED.
# It is the LARGEST threshold that still records the SMALLEST structure the claims are about:
# `floor.sv`'s thin-memo table is 18 312 B, and it is what makes "free_seq = NEVER on 11 of 11"
# a claim about eleven inputs rather than ten. A 64 KiB threshold would cut the tracked file
# from ~31 700 rows to ~735 and lose exactly that row. The 8 KiB run is kept for the transient
# TRAFFIC question and is reduced to a histogram (`tape_histogram.py`) rather than tracked raw,
# because at 8 KiB one 36 KB input alone contributes 251 438 rows.
TAPE_MIN="${PGEN_EUS67_TAPE_MIN_BYTES:-8192}"
[ -x "$PROBE" ] || { echo "no alloc-census probe at $PROBE" >&2; exit 2; }
[ -r "$LIST" ]  || { echo "no file list at $LIST" >&2; exit 2; }
W="tmp/eus67c/.rh.$$"; mkdir -p "$W"; trap 'rm -rf "$W"' EXIT

printf 'path\tslot\tsize\talloc_seq\tfree_seq\n' > "$OUTT"
printf 'path\tbytes\tverdict\tpeak_live\tsnap_live\ttape_min_bytes\ttape_capacity\ttape_recorded\ttape_overflow\ttape_unmatched_frees\ttape_probe_max\ttape_complete\tpeak_at_event_seq\tsnap_at_event_seq\tpeak_at_alloc_calls\tpeak_at_free_calls\tsnap_at_alloc_calls\tsnap_at_free_calls\tnever_freed_sentinel\tmemo_bucket_B\tmemo_fail_bucket_B\tmemo_entry_B\tthin_need_bytes\n' > "$OUTM"
printf 'path\tclass\tlo\thi\tlive_bytes\tlive_count\n' > "$OUTC"

# Exact `key=` extraction, never a greedy `.*key=`: the census line carries both
# `alloc_calls=` and `post_peak_alloc_calls=` and a sliding pattern silently matches
# the wrong one (the defect `-0126` recorded, met again in a different file).
kv() {
  printf '%s' "$2" | awk -v want="$1" '{
    for (i = 1; i <= NF; i++) {
      n = index($i, "=")
      if (n > 0 && substr($i, 1, n - 1) == want) { print substr($i, n + 1); exit }
    }
  }'
}

while IFS= read -r f; do
  [ -n "$f" ] || continue
  [ -r "$f" ] || { echo "skip unreadable: $f" >&2; continue; }
  sz=$(wc -c < "$f" | tr -d ' ')
  PGEN_REPORT_PARSE_FOOTPRINT=1 timeout "$TMO" "$PROBE" systemverilog "$f" \
      --profile sv_2017 --tape-min-bytes "$TAPE_MIN" > "$W/out" 2> "$W/err"
  rc=$?
  cen=$(grep -m1 -E 'PARSE[- ]ALLOC[- ]CENSUS' "$W/out" 2>/dev/null)
  tap=$(grep -m1 -E 'PARSE[- ]ALLOC[- ]TAPE' "$W/out" 2>/dev/null)
  tsz=$(grep -m1 -E 'PARSE[- ]ALLOC[- ]TYPE[- ]SIZES' "$W/out" 2>/dev/null)
  occ=$(grep -m1 'PARSE-THIN-OCCUPANCY' "$W/out" 2>/dev/null)
  if [ -z "$tap" ]; then
    echo "NO TAPE LINE for $f (rc=$rc) — probe too old or the run died" >&2
    printf '%s\t%s\tNO_TAPE\tNA\tNA\tNA\tNA\tNA\tNA\tNA\tNA\tfalse\tNA\tNA\tNA\tNA\tNA\tNA\tNA\tNA\tNA\tNA\tNA\n' "$f" "$sz" >> "$OUTM"
    continue
  fi
  if grep -q 'parse_full passed' "$W/out"; then v=ACCEPT; else v=REJECT; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$f" "$sz" "$v" \
    "$(kv peak_live "$cen")" "$(kv snapshot_live "$cen")" \
    "$(kv min_bytes "$tap")" "$(kv capacity "$tap")" "$(kv recorded "$tap")" \
    "$(kv overflow "$tap")" "$(kv unmatched_frees "$tap")" "$(kv probe_max "$tap")" \
    "$(kv complete "$tap")" \
    "$(kv peak_at_event_seq "$tap")" "$(kv snap_at_event_seq "$tap")" \
    "$(kv peak_at_alloc_calls "$tap")" "$(kv peak_at_free_calls "$tap")" \
    "$(kv snap_at_alloc_calls "$tap")" "$(kv snap_at_free_calls "$tap")" \
    "$(kv never_freed_sentinel "$tap")" \
    "$(kv memo_bucket "$tsz")" "$(kv memo_fail_bucket "$tsz")" "$(kv MemoEntry "$tsz")" \
    "$(kv need_bytes "$occ")" >> "$OUTM"

  awk -v p="$f" '/^  ALLOC-TAPE /{
    slot=size=aord=ford=""
    for (i = 1; i <= NF; i++) {
      n = index($i, "=");  if (n == 0) continue
      k = substr($i, 1, n - 1); v = substr($i, n + 1)
      if (k == "slot") slot = v; else if (k == "size") size = v
      else if (k == "alloc_seq") aord = v; else if (k == "free_seq") ford = v
    }
    if (slot != "") printf "%s\t%s\t%s\t%s\t%s\n", p, slot, size, aord, ford
  }' "$W/out" >> "$OUTT"

  awk -v p="$f" '/^  ALLOC-CLASS /{
    cls=lo=hi=lb=lc=""
    for (i = 1; i <= NF; i++) {
      n = index($i, "=");  if (n == 0) continue
      k = substr($i, 1, n - 1); v = substr($i, n + 1)
      if (k == "class") cls = v; else if (k == "lo") lo = v; else if (k == "hi") hi = v
      else if (k == "live_bytes") lb = v; else if (k == "live_count") lc = v
    }
    if (cls != "") printf "%s\t%s\t%s\t%s\t%s\t%s\n", p, cls, lo, hi, lb, lc
  }' "$W/out" >> "$OUTC"
done < "$LIST"
echo "wrote $OUTT / $OUTM / $OUTC"
