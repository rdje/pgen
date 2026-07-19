#!/usr/bin/env bash
# G1-A all-11 regen train (PGEN-RGX-0078-0166).
# ONE heavy job at a time, every step under the memory guard (16384 MB budget).
# ebnf.rs FIRST: it is include!'d into the lib, so regenerating it forces exactly
# one lib rebuild; doing it first keeps the rest of the train rebuild-free.
set -uo pipefail
R=/Users/richarddje/Documents/github/pgen
S=/private/tmp/claude-501/-Users-richarddje-Documents-github-pgen/ebe196bd-d754-4a06-ae42-207d4a2d6e48/scratchpad/g1a
G="$R/scripts/run_with_memory_guard.sh --budget-mb 16384 --floor-pct 10 --timeout-s 5400"
overall=0

step() {
  local name="$1"; shift
  echo "=== REGEN $name: START $(date +%H:%M:%S) ==="
  $G --marker "$S/regen_$name.marker" -- "$@" > "$S/regen_$name.log" 2>&1
  local rc=$?
  echo "=== REGEN $name: done exit=$rc $(date +%H:%M:%S) ==="
  if [ $rc -ne 0 ]; then overall=1; echo "FAILED: $name" >> "$S/regen_failures.txt"; fi
}

# 1. ebnf.rs — no focus_* target exists; canonical spelling is the bootstrap
#    Step C recipe (rust/Makefile:823) run against the current tool.
step ebnf bash -c "cd $R/rust && ./target/debug/ast_pipeline --generate-parser --debug --eliminate-left-recursion ../generated/ebnf.json -o ../generated/ebnf.rs"

# 2. Rebuild the tool now that ebnf.rs changed (it is compiled in).
step toolrebuild bash -c "cd $R/rust && cargo build --features generated_parsers --bin ast_pipeline"

# 3. ebnf.rs FIXED-POINT check — regen again with the rebuilt tool; must be
#    byte-identical, else the emitter is not at a fixed point.
step ebnf_fixpoint bash -c "cd $R/rust && ./target/debug/ast_pipeline --generate-parser --debug --eliminate-left-recursion ../generated/ebnf.json -o $S/ebnf_fixpoint.rs"

# 4. The 8 focus_* targets (focus_regex also regenerates the bootstrap
#    annotation pair, so these 8 cover the remaining 10 artifacts).
for t in json scratch rtl_const_expr systemverilog_preprocessor rtl_frontend vhdl regex systemverilog; do
  step "focus_$t" make -C "$R/rust" SHELL=/bin/bash "focus_$t"
done

echo "REGEN TRAIN COMPLETE overall=$overall"
exit $overall
