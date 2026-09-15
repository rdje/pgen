#!/usr/bin/env bash
# SVPP-EXPANSION.5(j) — the `"…`" string region (an embedded usage expanded, an object macro's region rendered, an escaped quote), measured against two reference tools this
# repository did not build. For each corpus file that carries a paste next to a macro usage and for
# each shape under shapes/, the expansion is produced three ways — PGEN (`ast_pipeline
# --preprocess-systemverilog`), Verilator (`verilator -E -P`) and Icarus (`iverilog -E`) — and
# compared modulo `line markers, // comments and all whitespace. One row per input: PGEN=V means PGEN agrees
# with Verilator, PGEN=I with Icarus, V=I the two oracles agree with each other. The oracles are
# EXTERNAL: a missing one is reported NOT EVALUATED for its column, never counted as agreement.
# ⛔ The two oracles do not always agree (sv2v macro_macro.sv: Verilator 3 , 4 — sv2v's own reference
# — against Icarus 1_MAGIC); where they split, the row's basis in docs/tasks/SVPP-EXPANSION.md says
# which reading the standard's own sentence supports and why. This bank is the record of that
# measurement, re-runnable; it exits 0 whatever it finds, because "PGEN agrees with Verilator" is a
# claim the leaf makes per row, not a gate.
# Run: bash docs/tasks/artifacts/svpp_expansion/string_region/probe.sh
set -u
cd "$(dirname "$0")/../../../../.." || exit 2
HERE=docs/tasks/artifacts/svpp_expansion/string_region
AST=${PGEN_AST_PIPELINE_BIN:-rust/target/debug/ast_pipeline}
SCRATCH=rust/target/svpp_string_region_probe          # repo-rooted (DATA-LOCALITY), never $TMPDIR
[ -x "$AST" ] || { echo "NOT EVALUATED: $AST is not built"; exit 0; }
mkdir -p "$SCRATCH"
have_v=1; have_i=1
command -v verilator >/dev/null 2>&1 || have_v=0
command -v iverilog  >/dev/null 2>&1 || have_i=0
echo "oracles: verilator=$([ $have_v = 1 ] && verilator --version | head -1 || echo ABSENT) ; iverilog=$([ $have_i = 1 ] && iverilog -V 2>&1 | head -1 || echo ABSENT)"
# modulo layout: `line markers (Icarus's `# N "file"`), // comments, and ALL whitespace — a
# backslash-newline in a macro body is a newline to Verilator and a space here, and both are layout
norm() { grep -v '^#' "$1" | sed 's://.*$::' | tr -d ' \t\r\n'; }
CORPUS="stimuli/sv/subs/iverilog/ivtest/ivltests/sv_macro3a.v stimuli/sv/subs/iverilog/ivtest/ivltests/sv_macro3b.v stimuli/sv/subs/verilator/test_regress/t/t_preproc_str_undef.v stimuli/sv/subs/verilator/test_regress/t/t_preproc_strify_join.v stimuli/sv/subs/sv2v/test/lex/string_macro.sv stimuli/sv/subs/sv2v/test/lex/macro_string.sv stimuli/sv/subs/Surelog/tests/PreprocString/dut.sv stimuli/sv/subs/opentitan/hw/ip/otbn/dv/uvm/env/otbn_ssctrl_if.sv"
rows=0; pv=0; pi=0; vi=0
printf '%-8s %-8s %-8s  %s\n' 'PGEN=V' 'PGEN=I' 'V=I' input
for f in $CORPUS "$HERE"/shapes/*.sv; do
  [ -f "$f" ] || { printf '%-8s %-8s %-8s  %s\n' ABSENT ABSENT ABSENT "$f"; continue; }
  n=$(echo "$f" | tr '/' '_')
  "$AST" --preprocess-systemverilog "$f" --output "$SCRATCH/$n.pgen" >/dev/null 2>&1 || : > "$SCRATCH/$n.pgen"
  a=NOTEVAL; b=NOTEVAL; c=NOTEVAL
  if [ $have_v = 1 ]; then verilator -E -P "$f" > "$SCRATCH/$n.v" 2>/dev/null || :; fi
  if [ $have_i = 1 ]; then iverilog -E -o "$SCRATCH/$n.i" "$f" 2>/dev/null || :; fi
  if [ $have_v = 1 ]; then if [ "$(norm "$SCRATCH/$n.pgen")" = "$(norm "$SCRATCH/$n.v")" ]; then a=yes; pv=$((pv+1)); else a=NO; fi; fi
  if [ $have_i = 1 ]; then if [ "$(norm "$SCRATCH/$n.pgen")" = "$(norm "$SCRATCH/$n.i")" ]; then b=yes; pi=$((pi+1)); else b=NO; fi; fi
  if [ $have_v = 1 ] && [ $have_i = 1 ]; then if [ "$(norm "$SCRATCH/$n.v")" = "$(norm "$SCRATCH/$n.i")" ]; then c=yes; vi=$((vi+1)); else c=NO; fi; fi
  rows=$((rows+1))
  printf '%-8s %-8s %-8s  %s\n' "$a" "$b" "$c" "$f"
done
echo "STRING-REGION-PROBE: rows=$rows pgen_eq_verilator=$pv pgen_eq_iverilog=$pi verilator_eq_iverilog=$vi ast_pipeline=$AST"
