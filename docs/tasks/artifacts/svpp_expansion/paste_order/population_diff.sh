#!/usr/bin/env bash
# SVPP-EXPANSION.5(i) — the CLASS, diffed: every corpus file that carries a `` paste token at all, plus the
# four reference expansions the earlier .5 slices hold byte-identical, expanded standalone under the .5
# include context by TWO ast_pipeline binaries (a pre-change one and the current one), and compared byte for
# byte together with the exit code. The population is DERIVED by grep on every run, never stored.
# Usage: bash docs/tasks/artifacts/svpp_expansion/paste_order/population_diff.sh <ast_pipeline.before> [<ast_pipeline.after>]
# Measured 2026-09-15 (PGEN-SVPP-EXPANSION-0025): files=188 identical=187 moved=1 — sv2v test/lex/macro_macro.sv.
set -u
cd "$(dirname "$0")/../../../../.." || exit 2
BEFORE=${1:?the pre-change ast_pipeline}; AFTER=${2:-rust/target/debug/ast_pipeline}
[ -x "$BEFORE" ] && [ -x "$AFTER" ] || { echo "NOT EVALUATED: $BEFORE or $AFTER is not executable"; exit 0; }
W=rust/target/svpp_paste_order_population            # repo-rooted (DATA-LOCALITY), never $TMPDIR
rm -rf "$W"; mkdir -p "$W/before" "$W/after"
{ grep -rl --include='*.sv' --include='*.v' --include='*.svh' --include='*.vh' -F '``' stimuli/sv/subs stimuli/sv/uvm 2>/dev/null
  printf '%s\n' stimuli/sv/subs/opentitan/hw/dv/sv/dv_utils/dv_utils_pkg.sv stimuli/sv/subs/opentitan/hw/ip/uart/rtl/uart.sv \
                stimuli/sv/subs/opentitan/hw/dv/sv/dv_lib/dv_base_env.sv stimuli/sv/uvm/uvm-core-2020.3.1/src/uvm_pkg.sv; } | sort -u > "$W/population.txt"
# the .5 include context: UVM's src plus every OpenTitan directory that holds a .svh
{ printf -- '--sv-include-dir\nstimuli/sv/uvm/uvm-core-2020.3.1/src\n'
  find stimuli/sv/subs/opentitan -name '*.svh' | sed 's#/[^/]*$##' | sort -u | sed 's/^/--sv-include-dir\n/'; } > "$W/include_dirs.txt"
INC=$(tr '\n' ' ' < "$W/include_dirs.txt")
expand_all() { # <binary> <outdir>
  local bin=$1 out=$2
  while read -r f; do n=$(echo "$f" | tr '/' '_');
    # shellcheck disable=SC2086
    "$bin" "$f" --preprocess-systemverilog --output "$out/$n.pp" $INC > "$out/$n.log" 2>&1; echo $? > "$out/$n.rc"; done < "$W/population.txt"
}
expand_all "$BEFORE" "$W/before"; expand_all "$AFTER" "$W/after"
same=0; moved=0
while read -r f; do n=$(echo "$f" | tr '/' '_')
  if cmp -s "$W/before/$n.rc" "$W/after/$n.rc" && { { [ ! -f "$W/before/$n.pp" ] && [ ! -f "$W/after/$n.pp" ]; } || cmp -s "$W/before/$n.pp" "$W/after/$n.pp"; }; then same=$((same+1))
  else moved=$((moved+1)); echo "MOVED: $f (rc $(cat "$W/before/$n.rc") -> $(cat "$W/after/$n.rc"))"; diff "$W/before/$n.pp" "$W/after/$n.pp" 2>/dev/null | head -6; fi
done < "$W/population.txt"
echo "POPULATION-DIFF: files=$(wc -l < "$W/population.txt" | tr -d ' ') identical=$same moved=$moved before=$BEFORE after=$AFTER"
