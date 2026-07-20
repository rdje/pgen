#!/usr/bin/env bash
# PGEN-RGX-0078-0201 expected-delta review: every regenerated artifact may
# differ from its banked pre-regen copy ONLY inside the `-0201` surfaces —
# the `try_parse_bare` body (coverage snapshot/truncate removed; the bare
# rollback twin; the new comment block) and the island C3-B cleanup sites
# (the bare rollback twin + comment). Any hunk outside those regions fails
# the review. This is a tripwire on top of the manual diff read, not a
# replacement for it.
set -euo pipefail
R=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)
S=${R0201_SCRATCH:?set R0201_SCRATCH to the session scratch dir}
overall=0

ALLOW='RGX-0078\.5\.j\.4|`-0201`'
ALLOW+='|saved_coverage_len|coverage_stack\.truncate'
ALLOW+='|rollback_to_labeled_bare|rollback_to_labeled'
ALLOW+='|bare wrapper carries NO|coverage snapshot/truncate|bare_parse'
ALLOW+='|coverage_enabled|-gated, and the fused graph emits no'
ALLOW+='|pushes, so the stack length is invariant across the bare'
ALLOW+='|region\. The rollback uses the bare twin, which skips only'
ALLOW+='|the diagnostic-only|rollbacks_nonempty_chain'
ALLOW+='|classification per the documented observed-parse boundary\.'
ALLOW+='|bare-graph-only site|the bare rollback twin|observed-parse boundary'
# prettyplease line-wrap continuations of the expected calls
ALLOW+='|^[<>][[:space:]]*(self|parser)$'
ALLOW+='|^[<>][[:space:]]*self\.semantic_runtime_state$'
ALLOW+='|^[<>][[:space:]]*parser\.semantic_runtime_state$'
ALLOW+='|^[<>][[:space:]]*saved_semantic_checkpoint,$'
ALLOW+='|^[<>][[:space:]]*tournament_semantic_checkpoint\.clone\(\),$'
ALLOW+='|^[<>][[:space:]]*crate::ast_pipeline::RollbackLabel::'
ALLOW+='|^[<>][[:space:]]*(//.*)?$'
ALLOW+='|^[<>][[:space:]]*[]){};,[]*[[:space:]]*$'

for post in "$R"/generated/*.rs; do
  name=$(basename "$post")
  pre="$S/artifacts_pre_regen/$name"
  if [[ ! -f "$pre" ]]; then
    echo "NEW ARTIFACT (no pre-regen copy): $name"
    overall=1
    continue
  fi
  if cmp -s "$pre" "$post"; then
    echo "UNCHANGED: $name"
    continue
  fi
  stray=$(diff "$pre" "$post" | grep -E '^[<>]' | grep -vE "$ALLOW" || true)
  if [[ -n "$stray" ]]; then
    echo "UNEXPECTED DELTA in $name:"
    echo "$stray" | head -20
    overall=1
  else
    changed=$(diff "$pre" "$post" | grep -cE '^[<>]' || true)
    echo "EXPECTED-ONLY DELTA: $name ($changed changed lines, all in the -0201 surfaces)"
  fi
done

echo "REVIEW COMPLETE overall=$overall"
exit $overall
