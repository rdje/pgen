#!/usr/bin/env bash
# table_arity_population_sweep.sh — size the table-arity population across the whole tracked tree,
# BROKEN DOWN BY PROVENANCE. Owning leaf: `LIVE-DOC-CONTAINMENT.8` (step 4, the gate).
#
# ⭐⭐ WHY PROVENANCE AND NOT A TOTAL. The total says the defect matters; only the breakdown says what
#   a BLOCKING rule would do to the people it fires on. Measured 2026-08-27: 54 of 1 164 tracked .md
#   files carry an arity defect, 148 OVER + 51 UNDER — and the two classes have OPPOSITE shapes.
#   The machine-converted LRM files under docs/{systemverilog,verilog,vhdl}/*/md/ hold ~88 % of the
#   UNDER population and nobody in this repository authored them; the project-authored files hold
#   ~74 % of the OVER. A blanket gate over the total would have fired mostly on documents no author
#   here can be asked to fix — the 91 %-false-positive shape GENERATED-LINT-CORRECTNESS.6/.12
#   refused. That is why TABLE-ARITY-RATCHET is a RATCHET against git rather than a gate against 0.
#
# ⛔ Same splitting rule as table_arity_census.sh, in ONE awk pass so the sweep is seconds not minutes:
#   an escaped \| is masked (it is content, never a separator), the leading AND trailing pipes are
#   both OPTIONAL, and every row is judged against ITS OWN table's header.
#
# USAGE: docs/tasks/artifacts/live_means_live/table_arity_population_sweep.sh [--files]
# Exit 0 always: this is a MEASUREMENT. The gate is scripts/check_table_arity.sh.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT" || exit 1
SHOW="${1:-}"

awk -v show="$SHOW" '
  function cellcount(s,   arr) {
    gsub(/\\\|/, "\001", s)
    sub(/^[[:space:]]*\|/, "", s); sub(/\|[[:space:]]*$/, "", s)
    return split(s, arr, /\|/)
  }
  function flush(   i) {
    if (f == "") return
    files++
    if (fo + fu > 0) {
      bad++
      cls = (f ~ /^docs\/(systemverilog|verilog|vhdl)\/[^\/]*\/?md\//) ? "LRM" : "PROJECT"
      O[cls] += fo; U[cls] += fu; F[cls]++
      if (show == "--files") printf "%4d OVER %3d UNDER  %-8s %s\n", fo, fu, cls, f
    }
    O["ALL"] += fo; U["ALL"] += fu
  }
  { if (FILENAME != f) { flush(); f = FILENAME; fo = 0; fu = 0; tbl = 0; prev = "" } }
  /^\| *:?-+:? *\|/ { prev = $0; next }
  /^\|/ {
    if (prev !~ /^\|/) { tbl++; hdr[tbl] = cellcount($0); prev = $0; next }
    if (!tbl) { prev = $0; next }
    c = cellcount($0)
    if (c > hdr[tbl]) fo++; else if (c < hdr[tbl]) fu++
  }
  { prev = $0 }
  END {
    flush()
    printf "\n== table-arity population over %d tracked .md files ==\n", files
    printf "  files with >= 1 arity defect: %d\n", bad
    printf "  %-9s %5s %6s %7s\n", "class", "files", "OVER", "UNDER"
    for (c in F) printf "  %-9s %5d %6d %7d\n", c, F[c], O[c], U[c]
    printf "  %-9s %5d %6d %7d\n", "TOTAL", bad, O["ALL"], U["ALL"]
    if (O["ALL"] > 0) printf "  share of OVER  that is PROJECT-authored: %d %%\n", int(O["PROJECT"] * 100 / O["ALL"] + 0.5)
    if (U["ALL"] > 0) printf "  share of UNDER that is LRM-converted:    %d %%\n", int(U["LRM"] * 100 / U["ALL"] + 0.5)
  }' $(git ls-files '*.md')
