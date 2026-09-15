#!/usr/bin/env bash
# SVPP-EXPANSION.5(j) — the CLASS, diffed: every corpus file that carries a `" macro-string marker at all, plus the
# four reference expansions the earlier .5 slices hold byte-identical, expanded standalone under the .5
# include context by TWO ast_pipeline binaries (a pre-change one and the current one), and compared byte for
# byte together with the exit code. The population is DERIVED by grep on every run, never stored.
# Usage: bash docs/tasks/artifacts/svpp_expansion/string_region/population_diff.sh <ast_pipeline.before> [<ast_pipeline.after>] [<needle>|REGEX_EMPTY_INVOCATION|ALL] [<workdir>]   # ALL = every lane file: a region reaching a file through an `include is invisible to any needle
# Measured 2026-09-15 (PGEN-SVPP-EXPANSION-0026): see docs/tasks/SVPP-EXPANSION.md .5(j).
set -u
cd "$(dirname "$0")/../../../../.." || exit 2
BEFORE=${1:?the pre-change ast_pipeline}; AFTER=${2:-rust/target/debug/ast_pipeline}
NEEDLE=${3:-'`"'}     # the fixed string that selects the population (default: the region marker); `NAME() for .5(j′)
W=${4:-rust/target/svpp_string_region_population}
[ -x "$BEFORE" ] && [ -x "$AFTER" ] || { echo "NOT EVALUATED: $BEFORE or $AFTER is not executable"; exit 0; }
# W is repo-rooted (DATA-LOCALITY), never $TMPDIR
rm -rf "$W"; mkdir -p "$W/before" "$W/after"
{ if [ "$NEEDLE" = ALL ]; then find stimuli/sv/subs stimuli/sv/uvm -type f \( -name '*.sv' -o -name '*.v' -o -name '*.svh' -o -name '*.vh' \) 2>/dev/null; elif [ "$NEEDLE" = REGEX_EMPTY_INVOCATION ]; then grep -rlE --include='*.sv' --include='*.v' --include='*.svh' --include='*.vh' '`[A-Za-z_][A-Za-z0-9_$]*\(\s*\)' stimuli/sv/subs stimuli/sv/uvm 2>/dev/null; else grep -rl --include='*.sv' --include='*.v' --include='*.svh' --include='*.vh' -F "$NEEDLE" stimuli/sv/subs stimuli/sv/uvm 2>/dev/null; fi
  printf '%s\n' stimuli/sv/subs/opentitan/hw/dv/sv/dv_utils/dv_utils_pkg.sv stimuli/sv/subs/opentitan/hw/ip/uart/rtl/uart.sv \
                stimuli/sv/subs/opentitan/hw/dv/sv/dv_lib/dv_base_env.sv stimuli/sv/uvm/uvm-core-2020.3.1/src/uvm_pkg.sv; } | sort -u > "$W/population.txt"
# the .5 include context: UVM's src plus every OpenTitan directory that holds a .svh
{ printf -- '--sv-include-dir\nstimuli/sv/uvm/uvm-core-2020.3.1/src\n'
  find stimuli/sv/subs/opentitan -name '*.svh' | sed 's#/[^/]*$##' | sort -u | sed 's/^/--sv-include-dir\n/'; } > "$W/include_dirs.txt"
INC=$(tr '\n' ' ' < "$W/include_dirs.txt")
expand_one() { # <binary> <outdir> <file>
  local bin=$1 out=$2 f=$3 n; n=$(echo "$f" | tr '/' '_')
  # shellcheck disable=SC2086
  "$bin" "$f" --preprocess-systemverilog --output "$out/$n.pp" $INC > "$out/$n.log" 2>&1; echo $? > "$out/$n.rc"
}
export -f expand_one; export INC
expand_all() { # <binary> <outdir> — 8-way, like the lanes
  xargs -P 8 -I{} bash -c 'expand_one "$@"' _ "$1" "$2" {} < "$W/population.txt"
}
expand_all "$BEFORE" "$W/before"; expand_all "$AFTER" "$W/after"
same=0; moved=0
while read -r f; do n=$(echo "$f" | tr '/' '_')
  if cmp -s "$W/before/$n.rc" "$W/after/$n.rc" && { { [ ! -f "$W/before/$n.pp" ] && [ ! -f "$W/after/$n.pp" ]; } || cmp -s "$W/before/$n.pp" "$W/after/$n.pp"; }; then same=$((same+1))
  else moved=$((moved+1)); echo "MOVED: $f (rc $(cat "$W/before/$n.rc") -> $(cat "$W/after/$n.rc"))"; diff "$W/before/$n.pp" "$W/after/$n.pp" 2>/dev/null | head -6; fi
done < "$W/population.txt"
echo "POPULATION-DIFF: needle=$NEEDLE files=$(wc -l < "$W/population.txt" | tr -d ' ') identical=$same moved=$moved before=$BEFORE after=$AFTER"
