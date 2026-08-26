#!/usr/bin/env bash
# table_arity_census.sh — per-table column-arity census for a tracked markdown file.
# Owning leaf: `LIVE-DOC-CONTAINMENT.8` (a tracked markdown table silently drops cells).
#
# ⛔ WHY PER-TABLE, AND WHY THAT IS THE WHOLE POINT. GFM's rule is per TABLE: a row with more cells
#   than ITS OWN header has the excess IGNORED — not rendered, not warned about — and a row with
#   fewer is silently padded. A census that sweeps a whole FILE with one row pattern compares every
#   row against the FIRST table's header and answers about a population that does not exist.
#   MEASURED on `docs/TASK_TREE.md`: the unbounded sweep sees 120 rows across FOUR tables and calls
#   25 of them wrong; the per-table census sees 67 + 18 + 27 + 8 and finds every arity defect in
#   table 1 (14 OVER + 2 UNDER), with tables 2, 3 and 4 clean. The leaf's own earlier note recorded
#   "14 of its 112 tree rows", which is the unbounded denominator — a right observation with a
#   wrong population.
#
# ⛔⛔ AN ESCAPED `\|` IS NOT A SEPARATOR, AND GETTING THAT WRONG INFLATES THE ANSWER BY 3.8x.
#   The first cut of this script split on EVERY `|` and reported **94 of 197** ledger rows over-wide
#   against the leaf's then-recorded **25**. The difference is not drift: **75** ledger rows carry a
#   deliberately escaped `\|`, which GFM renders as a literal pipe INSIDE a cell and never as a
#   column break — so the naive splitter was counting the FIX as the defect, and would have sent an
#   author to re-escape rows that are already correct. Escaped pipes are masked before splitting here.
#   ⚠️ A `|` inside a BACKTICK SPAN is deliberately NOT masked: per the GFM spec, code spans do not
#   protect pipes in tables, so `` `|->` `` really does break the row and really is a defect.
#
# ⚠️ HONEST LIMIT: this is a DETECTOR, not an adjudicator. A reported row still has to be READ to
#   tell an unescaped prose pipe (escape it) from an invented column (the author added a field). The
#   leaf already distinguishes those two causes and says only reading each row separates them.
#
# USAGE:  docs/tasks/artifacts/live_means_live/table_arity_census.sh [file ...]
#         (default: docs/TASK_TREE.md docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md)
#         --rows   also list every offending row with its column count
# Exit 0 always: this is a MEASUREMENT, not a gate. Wiring a gate is the leaf's acceptance (b).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT" || exit 1

SHOW_ROWS=""
files=()
for a in "$@"; do
  case "$a" in
    --rows) SHOW_ROWS=1 ;;
    *) files+=("$a") ;;
  esac
done
[ "${#files[@]}" -gt 0 ] || files=(docs/TASK_TREE.md docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md)

for f in "${files[@]}"; do
  [ -r "$f" ] || { printf 'table-arity: %s is unreadable — REPORTED, not skipped silently\n' "$f"; continue; }
  printf '\n== %s ==\n' "$f"
  awk -v show="$SHOW_ROWS" '
    # ⛔ Mask `\|` FIRST: it is a literal pipe in a cell, not a column break. Masking happens
    # inside the helpers, on a copy, so the reported row text stays verbatim.
    #
    # ⛔⛔ THE LEADING AND TRAILING PIPES ARE **BOTH OPTIONAL** IN GFM, AND DERIVING THE COUNT FROM
    #   awk`s NF GETS THAT WRONG IN THE FALSE-NEGATIVE DIRECTION. `NF-2` assumes a pipe at each end;
    #   a row that omits the trailing one has NF one lower, so an over-wide row is reported at
    #   exactly the declared arity and DISAPPEARS from the census. MEASURED, and it had already
    #   corrupted this leaf`s published figure: `REGEX-0098` in the bug ledger omits its trailing
    #   pipe, renders 14 cells against a 13-column header, LOSES its Notes cell in both renderers,
    #   and the `NF-2` census called it healthy — which is how the leaf came to publish **25 of 197**
    #   for a population that is **26**. The same bug read `README-POLICY` in the resume index as a
    #   4-cell UNDER row when it carries 5, and `SV-CORPUS-GRAD` at 21 when it carries 22.
    function cellcount(s,   arr) {
      gsub(/\\\|/, "\001", s)
      sub(/^[[:space:]]*\|/, "", s); sub(/\|[[:space:]]*$/, "", s)
      return split(s, arr, /\|/)
    }
    function firstcell(s,   arr) {
      gsub(/\\\|/, "\001", s)
      sub(/^[[:space:]]*\|/, "", s); split(s, arr, /\|/)
      return arr[1]
    }
    /^\| *:?-+:? *\|/ { next }                      # the header separator row
    /^\|/ {
      if (prev !~ /^\|/) { tbl++; hdr[tbl]=cellcount($0); start[tbl]=NR; prev=$0; next }
      if (!tbl) { prev=$0; next }
      rows[tbl]++; c=cellcount($0); nm=firstcell($0)
      if (c > hdr[tbl]) { over[tbl]++
        if (c > mx[tbl]) { mx[tbl]=c; mxn[tbl]=nm }
        if (show) printf "   OVER  line %-5d %2d cols (header %d, drops %d): %s\n", NR, c, hdr[tbl], c-hdr[tbl], substr(nm,1,38) }
      else if (c < hdr[tbl]) { under[tbl]++
        if (show) printf "   UNDER line %-5d %2d cols (header %d): %s\n", NR, c, hdr[tbl], substr(nm,1,38) }
    }
    { prev=$0 }
    END {
      for (i=1;i<=tbl;i++) {
        printf "  table %d (line %4d, %2d-col header): %3d data rows | OVER %3d | UNDER %3d",
          i, start[i], hdr[i], rows[i], over[i], under[i]
        if (over[i]) printf " | widest %d %s", mx[i], mxn[i]
        printf "\n"
        R+=rows[i]; O+=over[i]; U+=under[i] }
      printf "  TOTAL: %d data rows in %d table(s) | %d OVER (cells DROPPED) | %d UNDER (cells PADDED)\n", R, tbl, O, U
    }' "$f"
done
