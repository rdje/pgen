#!/usr/bin/env bash
# docs/tasks/artifacts/eus67/measure_parse_footprint.sh — pair the parse footprint with
# peak RSS, per file. ENGINE-UNIVERSAL-SERVICES.67(b).
#
# The report line was renamed `(arena leg)` -> `(arena + AST-owned heap)` when the
# out-of-arena child-`Vec`/`String` legs became MEASURED rather than declared, so the
# match below anchors on `PARSE FOOTPRINT` alone and is stable across that rename.
#
# ENGINE-UNIVERSAL-SERVICES.67(b) LEG 3: the instrument now prints a SECOND line, the
# `typed_arena` chunk-capacity census. It is a second line and not more fields on the
# first for a mechanical reason — every extraction below is a GREEDY `.*<token>=`, so a
# new token ending in `nodes=` would have silently re-targeted the existing `nodes=`
# capture at it. `grep -m1 'PARSE FOOTPRINT'` still selects the arena line (the
# instrument prints it first) and the chunk line has its own anchor, so the columns this
# producer wrote before leg 3 must re-derive UNCHANGED — which is the control.
# Usage: measure_footprint.sh <filelist> <out.tsv> [timeout_s]
set -uo pipefail
# ⛔ FOUR levels, not two. This producer was authored at `tmp/eus67/` and COPIED here
# without re-deriving the walk, so the tracked copy `cd`-ed to `docs/tasks` and could not
# open its own inputs or output — "tracked and re-runnable" was true of the first word only.
# Its sibling `measure_parse_rss.sh` had it right all along, which is why they are now
# spelled identically.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"
LIST="$1"; OUT="$2"; TMO="${3:-120}"
PROBE="${PGEN_EUS67_PROBE:-./rust/target/release/parseability_probe}"
# A missing probe must REFUSE, not produce a TSV of NA rows that reads like a measurement.
[ -x "$PROBE" ] || { echo "no probe at $PROBE" >&2; exit 2; }
[ -r "$LIST" ]  || { echo "no file list at $LIST" >&2; exit 2; }
W="tmp/eus67b/.fw.$$"; mkdir -p "$W"; trap 'rm -rf "$W"' EXIT
printf 'path\tbytes\tverdict\tpeak_rss_bytes\tarena_nodes\tarena_node_MB\tarena_total_MB'   >  "$OUT"
printf '\tchild_vecs\tchild_slots\tchild_MB\ttransformed_strings\ttransformed_MB\trendered_MB\towned_MB\tarena_owned_MB' >> "$OUT"
printf '\tarena_shaped_values\tarena_shaped_pairs\tarena_rendered_strings' >> "$OUT"
printf '\tchunk_nodes_cap\tchunk_nodes_touched\tchunk_nodes_MB\tchunk_sv_cap\tchunk_sv_touched\tchunk_sv_MB' >> "$OUT"
printf '\tchunk_sp_cap\tchunk_sp_touched\tchunk_sp_MB\tchunk_rs_cap\tchunk_rs_touched\tchunk_rs_MB\tspine_B' >> "$OUT"
printf '\tallocated_total_MB\ttouched_total_MB\tarena_ast_touched_MB\tarena_ast_allocated_MB\titems_agree\textend_regrow\n' >> "$OUT"
while IFS= read -r f; do
  [ -n "$f" ] || continue
  sz=$(wc -c < "$f" | tr -d ' ')
  PGEN_REPORT_PARSE_FOOTPRINT=1 /usr/bin/time -l timeout "$TMO" sh -c \
      'exec "$0" --parse systemverilog "$1" --profile sv_2017 >"$2" 2>"$3"' \
      "$PROBE" "$f" "$W/out" "$W/err" 2> "$W/time"
  rc=$?
  rss=$(awk '/maximum resident set size/{print $1; exit}' "$W/time")
  line=$(grep -m1 'PARSE FOOTPRINT' "$W/err" 2>/dev/null)
  cline=$(grep -m1 'chunk capacity' "$W/err" 2>/dev/null)
  nodes=$(printf '%s' "$line" | sed -n 's/.*nodes=\([0-9]*\).*/\1/p')
  nmb=$(printf '%s' "$line" | sed -n 's/.*nodes=[0-9]* (\([0-9.]*\) MB.*/\1/p')
  tmb=$(printf '%s' "$line" | sed -n 's/.*arena item total \([0-9.]*\) MB.*/\1/p')
  # The three side arenas' ITEM counts, so the written-vs-live check below has a
  # denominator for every arena rather than only for `nodes`.
  asv=$(printf '%s' "$line" | sed -n 's/.*shaped_values=\([0-9]*\).*/\1/p')
  asp=$(printf '%s' "$line" | sed -n 's/.*shaped_pairs=\([0-9]*\).*/\1/p')
  ars=$(printf '%s' "$line" | sed -n 's/.*rendered_strings=\([0-9]*\).*/\1/p')
  # ENGINE-UNIVERSAL-SERVICES.67(b) — the out-of-arena AST-owned legs. Any field that
  # does not match stays NA rather than 0: an unmatched field is a MISSING measurement,
  # and writing it as 0 is the exact confusion the instrument's `Option` guard prevents.
  cvec=$(printf '%s' "$line" | sed -n 's/.*child_vecs=\([0-9]*\).*/\1/p')
  cslot=$(printf '%s' "$line" | sed -n 's/.*child_vecs=[0-9]* (\([0-9]*\) slots.*/\1/p')
  cmb=$(printf '%s' "$line" | sed -n 's/.*child_vecs=[0-9]* ([0-9]* slots, \([0-9.]*\) MB.*/\1/p')
  xstr=$(printf '%s' "$line" | sed -n 's/.*transformed_strings=\([0-9]*\).*/\1/p')
  xmb=$(printf '%s' "$line" | sed -n 's/.*transformed_strings=[0-9]* (\([0-9.]*\) MB.*/\1/p')
  rmb=$(printf '%s' "$line" | sed -n 's/.*rendered_payload \([0-9.]*\) MB.*/\1/p')
  omb=$(printf '%s' "$line" | sed -n 's/.*AST-owned heap total \([0-9.]*\) MB.*/\1/p')
  aomb=$(printf '%s' "$line" | sed -n 's/.*arena+owned total \([0-9.]*\) MB.*/\1/p')
  # ENGINE-UNIVERSAL-SERVICES.67(b) leg 3 — the chunk-capacity line. Same NA-not-0
  # discipline: an unmatched field is a missing measurement.
  ncap=$(printf '%s' "$cline" | sed -n 's/.*nodes cap=\([0-9]*\).*/\1/p')
  ntch=$(printf '%s' "$cline" | sed -n 's/.*nodes cap=[0-9]* items=[0-9]* touched=\([0-9]*\).*/\1/p')
  nchmb=$(printf '%s' "$cline" | sed -n 's/.*nodes cap=[0-9]* items=[0-9]* touched=[0-9]* chunks=[0-9]* (\([0-9.]*\) MB).*/\1/p')
  svcap=$(printf '%s' "$cline" | sed -n 's/.*shaped_values cap=\([0-9]*\).*/\1/p')
  # ⭐ Each arena's `touched` is captured beside its `items` so a reader can CHECK the
  # structural claim "written slots == live items for PGEN's call pattern" per arena
  # rather than take it from the prose. It is exactly this claim that decides whether
  # the pre-leg-3 item-bytes figure was already the RSS-relevant one.
  svtch=$(printf '%s' "$cline" | sed -n 's/.*shaped_values cap=[0-9]* items=[0-9]* touched=\([0-9]*\).*/\1/p')
  sptch=$(printf '%s' "$cline" | sed -n 's/.*shaped_pairs cap=[0-9]* items=[0-9]* touched=\([0-9]*\).*/\1/p')
  rstch=$(printf '%s' "$cline" | sed -n 's/.*rendered_strings cap=[0-9]* items=[0-9]* touched=\([0-9]*\).*/\1/p')
  svmb=$(printf '%s' "$cline" | sed -n 's/.*shaped_values cap=[0-9]* items=[0-9]* touched=[0-9]* chunks=[0-9]* (\([0-9.]*\) MB).*/\1/p')
  spcap=$(printf '%s' "$cline" | sed -n 's/.*shaped_pairs cap=\([0-9]*\).*/\1/p')
  spmb=$(printf '%s' "$cline" | sed -n 's/.*shaped_pairs cap=[0-9]* items=[0-9]* touched=[0-9]* chunks=[0-9]* (\([0-9.]*\) MB).*/\1/p')
  rscap=$(printf '%s' "$cline" | sed -n 's/.*rendered_strings cap=\([0-9]*\).*/\1/p')
  rsmb=$(printf '%s' "$cline" | sed -n 's/.*rendered_strings cap=[0-9]* items=[0-9]* touched=[0-9]* chunks=[0-9]* (\([0-9.]*\) MB).*/\1/p')
  spine=$(printf '%s' "$cline" | sed -n 's/.*spines \([0-9]*\) B.*/\1/p')
  atot=$(printf '%s' "$cline" | sed -n 's/.*allocated total \([0-9.]*\) MB.*/\1/p')
  ttot=$(printf '%s' "$cline" | sed -n 's/.*touched total \([0-9.]*\) MB.*/\1/p')
  # ⛔ The BAND, both ends. Publishing only the allocated end would attribute the
  # arena's unwritten chunk slack to the arena; `measure_chunk_residency.sh`
  # MEASURED that slack at 1 008 MB of allocation costing 32 768 B of peak RSS.
  aast=$(printf '%s' "$cline" | sed -n 's/.*arena+AST touched \([0-9.]*\) MB.*/\1/p')
  aasa=$(printf '%s' "$cline" | sed -n 's/.*\.\. allocated \([0-9.]*\) MB.*/\1/p')
  agree=$(printf '%s' "$cline" | sed -n 's/.*items_agree=\([a-z]*\).*/\1/p')
  regrow=$(printf '%s' "$cline" | sed -n 's/.*extend_regrow=\([0-9]*\).*/\1/p')
  if [ "$rc" -eq 124 ]; then v=TIMEOUT
  elif grep -q 'parse_full passed' "$W/out" 2>/dev/null; then v=ACCEPT
  else v=REJECT; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s' "$f" "$sz" "$v" "${rss:-NA}" "${nodes:-NA}" "${nmb:-NA}" "${tmb:-NA}" >> "$OUT"
  printf '\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s' "${cvec:-NA}" "${cslot:-NA}" "${cmb:-NA}" \
      "${xstr:-NA}" "${xmb:-NA}" "${rmb:-NA}" "${omb:-NA}" "${aomb:-NA}" >> "$OUT"
  printf '\t%s\t%s\t%s' "${asv:-NA}" "${asp:-NA}" "${ars:-NA}" >> "$OUT"
  printf '\t%s\t%s\t%s\t%s\t%s\t%s' "${ncap:-NA}" "${ntch:-NA}" "${nchmb:-NA}" \
      "${svcap:-NA}" "${svtch:-NA}" "${svmb:-NA}" >> "$OUT"
  printf '\t%s\t%s\t%s\t%s\t%s\t%s\t%s' "${spcap:-NA}" "${sptch:-NA}" "${spmb:-NA}" \
      "${rscap:-NA}" "${rstch:-NA}" "${rsmb:-NA}" "${spine:-NA}" >> "$OUT"
  printf '\t%s\t%s\t%s\t%s\t%s\t%s\n' "${atot:-NA}" "${ttot:-NA}" "${aast:-NA}" "${aasa:-NA}" \
      "${agree:-NA}" "${regrow:-NA}" >> "$OUT"
done < "$LIST"
