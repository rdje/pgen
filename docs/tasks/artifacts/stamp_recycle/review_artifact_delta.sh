#!/usr/bin/env bash
# PGEN-RGX-0078-0210 expected-delta review: every regenerated artifact may
# differ from its banked pre-regen (`-0207`-vintage) copy ONLY inside the
# `-0210` surfaces — the four thread-recycled lease fields (`memo_fail`,
# `memo_fail_tainted`, `recursion_guard`, `semantic_runtime_state`:
# `ParseScratchLease<…>` types over the two STAMPED fail-memo lib types +
# the take/ensure_capacity/set_max_depth construction blocks replacing the
# with_capacity/`new()` constructions) and their doc comments — for ALL 11
# artifacts (the `-0209` addendum's corrected expectation: the annotation
# pair is emitted by ast_based_generator in bootstrap MODE, so the surfaces
# reach every artifact). Any hunk outside those regions fails the review.
# This is a tripwire on top of the manual diff read, not a replacement.
set -uo pipefail
R=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)
S=${R0210_SCRATCH:?set R0210_SCRATCH to the session scratch dir}
overall=0

ALLOW='PGEN-RGX-0078-0210|`-0210`|-0210|`-0209`|-0209|RGX-0078\.5\.j\.4'
# the NEW lease/stamp spellings (either diff side)
ALLOW+='|ParseScratchLease|__pgen_lease|set_max_depth|ensure_capacity'
ALLOW+='|StampedFailSet|StampedTaintedFailMap'
ALLOW+='|thread-recycled|lease|recycled|recycling|generation-'
ALLOW+='|stamp bump|stamped|STAMPED|high-water|warm path'
# the OLD construction spellings this fix replaces
ALLOW+='|SemanticRuntimeState::new\(\)|with_capacity_and_hasher|RecursionGuard::new\('
# the four fields' DECL/INIT lines (colon-anchored so behavioral uses of the
# fields elsewhere are NOT masked)
ALLOW+='|memo_fail: |memo_fail_tainted: |recursion_guard: |semantic_runtime_state: '
ALLOW+='|RecursionGuard,$|RecursionGuard >,?$'
# the sizing formulas (they move from ctor args onto ensure_capacity lines)
ALLOW+='|\(\(input\.len\(\) \+ 1\) \* 6\)\.min\(32768\)'
ALLOW+='|\(input\.len\(\) \+ 1\)\.min\(256\)'
ALLOW+='|Default::default\(\),?'
# doc-comment lines of the changed constructs
ALLOW+='|constructor postcondition|mem::take doctrine|SV-EXH-PROOF\.3\.3\.3'
ALLOW+='|K3a/C2|with_capacity invariant|grown allocation|fail sets|fail memos'
ALLOW+='|-0205|new\(\) postcondition|tearing down|invalidates the prior parse'
ALLOW+='|O\(high-water-capacity\)|O\(1\)|design law'
# prettyplease reflow shards of the changed constructs
ALLOW+='|^[<>][[:space:]]*crate::ast_pipeline::'
ALLOW+='|^[<>][[:space:]]*crate :: ast_pipeline ::'
ALLOW+='|^[<>][[:space:]]*rustc_hash::'
ALLOW+='|^[<>][[:space:]]*RuleId, usize'
ALLOW+='|^[<>][[:space:]]*u64,?$'
ALLOW+='|^[<>][[:space:]]*(//.*)?$'
ALLOW+='|^[<>][[:space:]]*[]){};,><[]*[[:space:]]*$'

for post in "$R"/generated/*.rs; do
  name=$(basename "$post")
  pre="$S/artifacts_pre_regen/$name"
  if [[ ! -f "$pre" ]]; then
    echo "NEW ARTIFACT (no pre-regen copy): $name"
    overall=1
    continue
  fi
  if cmp -s "$pre" "$post"; then
    echo "REFUSE: $name unchanged — the emitter change did not reach it"
    overall=1
    continue
  fi
  # Cancel pure block MOVES first: a line removed and re-added with identical
  # content is not a delta. The residual = the SYMMETRIC DIFFERENCE of the
  # two sides' line multisets, re-annotated with its side and ALLOW-filtered.
  diff "$pre" "$post" | grep '^< ' | sed 's/^< //' | sort > "$S/.delta_old"
  diff "$pre" "$post" | grep '^> ' | sed 's/^> //' | sort > "$S/.delta_new"
  stray=$(
    {
      comm -23 "$S/.delta_old" "$S/.delta_new" | sed 's/^/< /'
      comm -13 "$S/.delta_old" "$S/.delta_new" | sed 's/^/> /'
    } | grep -vE "$ALLOW" || true
  )
  if [[ -n "$stray" ]]; then
    echo "UNEXPECTED DELTA in $name:"
    echo "$stray" | head -20
    overall=1
  else
    changed=$(diff "$pre" "$post" | grep -cE '^[<>]' || true)
    echo "EXPECTED-ONLY DELTA: $name ($changed changed lines, all in the -0210 surfaces)"
  fi
done

echo "REVIEW COMPLETE overall=$overall"
exit $overall
