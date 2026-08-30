#!/bin/bash
# `ENGINE-UNIVERSAL-SERVICES.79`(a) — derive the MARGIN table from a recorded census.
#
# WHY IT EXISTS. `-0145` published the margin column and `-0146` re-derived it by an
# ad-hoc script that was never tracked, so the published table was checkable exactly
# once. This makes it a COMMAND: every margin, the spread and the tightest cell come
# out of the recorded census file plus the engine's OWN constants, never out of a
# number somebody re-typed (`docs/DERIVED_STATE_CONTAINMENT.md` R1/R3).
#
# ⛔ THE BUDGET IS NOT A LITERAL HERE. `DEDICATED_PARSE_STACK_BYTES` and
# `STACK_HEADROOM_RESERVE_DIVISOR` are READ OUT OF THE RUST SOURCE, so if either moves
# this script moves with it and a stale margin cannot survive the next run.
#
# Usage:  bash <this script> <census-file> [requested-depth]
#   requested-depth defaults to 4096 — the depth every generated parser asks
#   `RecursionGuard::new` for, i.e. the depth the margin is a margin AGAINST.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 1

census="${1:?usage: derive_margins.sh <census-file> [requested-depth]}"
depth="${2:-4096}"
[ -r "$census" ] || { echo "unreadable census: $census" >&2; exit 2; }

stack_src="rust/src/dedicated_parse_stack.rs"
pipeline_src="rust/src/ast_pipeline/mod.rs"
# `256 * 1024 * 1024` -> 268435456, evaluated by the shell rather than transcribed.
reservation_expr="$(grep -o 'DEDICATED_PARSE_STACK_BYTES: usize = [0-9 *]*' "$stack_src" \
                    | grep -v MIN_ | head -1 | sed 's/.*= //')"
divisor="$(grep -o 'STACK_HEADROOM_RESERVE_DIVISOR: usize = [0-9]*' "$pipeline_src" \
           | head -1 | sed 's/.*= //')"
[ -n "$reservation_expr" ] && [ -n "$divisor" ] || {
  echo "could not read the reservation/divisor constants out of the engine" >&2; exit 3; }
reservation=$(( reservation_expr ))
budget=$(( reservation - reservation / divisor ))

printf 'reservation  %s B (%s MiB, %s)\n' "$reservation" "$((reservation/1024/1024))" "$stack_src"
printf 'divisor      %s  (%s)\n' "$divisor" "$pipeline_src"
printf 'budget       %s B (%s MiB) = reservation - reservation/divisor\n' "$budget" "$((budget/1024/1024))"
printf 'depth        %s frames (RecursionGuard::new request)\n\n' "$depth"

printf '%-28s %-14s %-10s %-14s %s\n' FAMILY PROFILE B_PER_FRAME COST_AT_DEPTH MARGIN
awk -v budget="$budget" -v depth="$depth" '
  NR == 1 { next }                              # the census header
  $NF ~ /^[0-9]+$/ {
    bpf = $NF + 0; if (bpf <= 0) next
    fam = $1; prof = $2
    cost = depth * bpf
    margin = budget / cost
    printf "%-28s %-14s %-10d %-14d %.2fx\n", fam, prof, bpf, cost, margin
    n++
    if (n == 1 || margin < min) { min = margin; minfam = fam "/" prof; minfamily = fam }
    if (n == 1 || margin > max) { max = margin; maxfam = fam "/" prof }
    if (n == 1 || bpf < lo)     { lo = bpf }
    if (n == 1 || bpf > hi)     { hi = bpf }
    fams[n] = fam; margins[n] = margin
  }
  END {
    if (!n) { print "no numeric cell in this census" > "/dev/stderr"; exit 4 }
    # SEPARATION IS DERIVED, NOT A THRESHOLD SOMEBODY TYPED. -0145 published
    # "SystemVerilog is the only family under 11x, the next-nearest is ~5.5x further
    # away"; an 11 hard-coded here would make that sentence true by construction. The
    # quantity is the ratio between the tightest margin and the tightest margin
    # OUTSIDE the family of the binding cell, which is what "further away" means.
    # (No apostrophe may appear in this awk program: it lives inside a single-quoted
    # shell string, which is the same class of quoting defect the doctrine registry
    # paid for four times with unescaped backticks.)
    sep = 0; sepfam = "(none — one family only)"
    for (i = 1; i <= n; i++)
      if (fams[i] != minfamily && (sep == 0 || margins[i] < sep)) { sep = margins[i]; sepfam = fams[i] }
    printf "\nnumeric cells   %d\n", n
    printf "spread          %.2fx  (%d -> %d B/frame)\n", hi / lo, lo, hi
    printf "tightest margin %.2fx  (%s)\n", min, minfam
    printf "widest margin   %.2fx  (%s)\n", max, maxfam
    if (sep > 0)
      printf "separation      %.2fx  — the binding family is %s, the nearest other family is %s at %.2fx\n", \
             sep / min, minfamily, sepfam, sep
    else
      printf "separation      %s\n", sepfam
  }
' "$census"
