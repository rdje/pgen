#!/usr/bin/env bash
# PGEN-RGX-0078-0214 all-11 regen train (the `-0202` recipe verbatim). The
# tool is rebuilt FIRST so every artifact is emitted by the changed emitter.
# ONE heavy job at a time, every step under the memory guard (16384 MB).
set -uo pipefail
R=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)
S=${R0214_SCRATCH:?set R0214_SCRATCH to the session scratch dir}
G="$R/scripts/run_with_memory_guard.sh --budget-mb 16384 --floor-pct 10 --timeout-s 5400"
overall=0

step() {
  local name="$1"; shift
  echo "=== REGEN $name: START $(date +%H:%M:%S) ==="
  $G --marker "$S/regen_$name.marker" -- caffeinate -i "$@" > "$S/regen_$name.log" 2>&1
  local rc=$?
  echo "=== REGEN $name: done exit=$rc $(date +%H:%M:%S) ==="
  if [ $rc -ne 0 ]; then overall=1; echo "FAILED: $name" >> "$S/regen_failures.txt"; fi
}

# 0. The pre-regen artifact identities are banked by the session BEFORE any
#    regeneration; assert that bank exists rather than re-banking.
[ -s "$S/artifacts_pre_regen.sha256" ] || { echo "REFUSE: pre-regen bank missing"; exit 2; }

# 1. Rebuild the tool FIRST — it must carry the changed emitter before any
#    artifact is regenerated.
step toolbuild bash -c "cd $R/rust && cargo build --features generated_parsers --bin ast_pipeline"

# 2. ebnf.rs — canonical spelling (bootstrap Step C recipe) with the fresh tool.
step ebnf bash -c "cd $R/rust && ./target/debug/ast_pipeline --generate-parser --debug --eliminate-left-recursion ../generated/ebnf.json -o ../generated/ebnf.rs"

# 3. Rebuild the tool now that ebnf.rs changed (it is compiled in).
step toolrebuild bash -c "cd $R/rust && cargo build --features generated_parsers --bin ast_pipeline"

# 4. ebnf.rs FIXED-POINT check (byte-identical modulo the embedded output path).
step ebnf_fixpoint bash -c "cd $R/rust && ./target/debug/ast_pipeline --generate-parser --debug --eliminate-left-recursion ../generated/ebnf.json -o $S/ebnf_fixpoint.rs && sed 's|$S/ebnf_fixpoint\.rs|../generated/ebnf.rs|g' $S/ebnf_fixpoint.rs | cmp - ../generated/ebnf.rs"

# 5. The 8 focus_* targets (focus_regex also regenerates the bootstrap
#    annotation pair, so these 8 cover the remaining 10 artifacts).
for t in json scratch rtl_const_expr systemverilog_preprocessor rtl_frontend vhdl regex systemverilog; do
  step "focus_$t" make -C "$R/rust" SHELL=/bin/bash "focus_$t"
done

# 6. Bank the post-regen identities.
( cd "$R/generated" && shasum -a 256 *.rs ) > "$S/artifacts_post_regen.sha256"

echo "REGEN TRAIN COMPLETE overall=$overall"
exit $overall
