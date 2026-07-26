set -uo pipefail
cd /Volumes/SSD/Documents/github/pgen
W=rust/target/quant_plus_iter_probe/bi
G=scripts/run_with_memory_guard.sh
rm -rf $W && mkdir -p $W/new $W/head
GRAMMARS="json regex vhdl systemverilog systemverilog_preprocessor rtl_frontend rtl_const_expr return_annotation semantic_annotation ebnf"

gen_all() {  # $1 = label
  for g in $GRAMMARS; do
    # SAME output path for both vintages (.5 trap: codegen embeds the output path)
    ./rust/target/debug/ast_pipeline grammars/$g.ebnf --generate-parser \
      --output $W/pinned_$g.rs > $W/$1/$g.log 2>&1
    rc=$?
    if [ $rc -eq 0 ]; then cp $W/pinned_$g.rs $W/$1/$g.rs; else echo "GENFAIL $g rc=$rc" >> $W/$1/FAILED; fi
  done
}

echo "--- generating with NEW binary (@entry landed) ---"
gen_all new
echo "--- stashing to HEAD vintage ---"
git stash push -- rust/src/main.rs rust/src/ast_pipeline/semantic_runtime.rs rust/src/ast_pipeline/semantic_directive_registry.rs > $W/stash.log 2>&1 || { echo STASH_FAIL; exit 1; }
$G --budget-mb 16384 --timeout-s 2400 -- bash -c 'cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline' > $W/build_head.log 2>&1 || { echo HEADBUILD_FAIL; git stash pop; exit 1; }
echo "--- generating with HEAD binary ---"
gen_all head
echo "--- restoring my changes ---"
git stash pop > $W/unstash.log 2>&1 || echo UNSTASH_FAIL
$G --budget-mb 16384 --timeout-s 2400 -- bash -c 'cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline' > $W/build_new.log 2>&1 || echo NEWBUILD_FAIL

echo "===== BYTE-IDENTITY RESULT ====="
same=0; diff=0; missing=0
for g in $GRAMMARS; do
  if [ -f $W/new/$g.rs ] && [ -f $W/head/$g.rs ]; then
    if cmp -s $W/new/$g.rs $W/head/$g.rs; then printf '  %-32s IDENTICAL (%s lines)\n' "$g" "$(wc -l < $W/new/$g.rs|tr -d ' ')"; same=$((same+1));
    else printf '  %-32s *** DIFFERS ***\n' "$g"; diff=$((diff+1)); fi
  else printf '  %-32s (not generated both vintages)\n' "$g"; missing=$((missing+1)); fi
done
echo "identical=$same differs=$diff missing=$missing"
echo "BYTE_IDENTITY_DONE"
