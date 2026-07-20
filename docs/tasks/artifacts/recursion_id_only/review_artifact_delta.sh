#!/usr/bin/env bash
# PGEN-RGX-0078-0200 expected-delta review: every regenerated artifact may
# differ from its banked pre-regen copy ONLY inside the `-0200` surfaces —
# the two-snapshot `try_parse`, the new `try_parse_bare` (cascade-active
# parsers), the `create_contextual_error` ID-map re-point, and the cascade
# internal frames' bare enter/exit swap. Any hunk outside those regions
# fails the review. This is a tripwire on top of the manual diff read, not a
# replacement for it.
set -euo pipefail
R=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)
S=${R0200_SCRATCH:?set R0200_SCRATCH to the session scratch dir}
overall=0

ALLOW='RGX-0078\.5\.j\.4|`-0200`'
ALLOW+='|saved_name_stack_len|saved_id_stack_len|saved_stack_len'
ALLOW+='|truncate_stacks|truncate_stack'
ALLOW+='|try_parse_bare|fn try_parse|\.try_parse\(\|p\| \{'
ALLOW+='|\.enter_id_bare\(|\.exit_bare\(|\.enter_id\(|\.exit\(\)'
ALLOW+='|rule_id_stack|parse_stack'
ALLOW+='|RULE_NAMES|filter_map|\.map\(\|\(rule, _\)\| \*rule\)'
ALLOW+='|saved_pos|saved_coverage_len|saved_semantic_checkpoint'
ALLOW+='|match f\(self\)|Ok\(result\) => Some\(result\)|Err\(_\) =>'
ALLOW+='|coverage_stack\.truncate|rollback_to_labeled|TryParseErr|try_parse_rule'
ALLOW+='|FnOnce\(&mut Self\)|-> Option<T>|\.and_then\(\|\(rid, _\)\||\.copied\(\)|\.last\(\)'
# prettyplease line-wrap continuations of the expected calls
ALLOW+='|^[<>][[:space:]]*(self|parser)$'
ALLOW+='|^[<>][[:space:]]*(self|parser)\.recursion_guard$'
ALLOW+='|^[<>][[:space:]]*\.?recursion_guard$'
ALLOW+='|^[<>][[:space:]]*self\.semantic_runtime_state$'
ALLOW+='|^[<>][[:space:]]*Self::RULE_[A-Z0-9_]+,?$'
ALLOW+='|^[<>][[:space:]]*"[A-Za-z0-9_]+",$'
ALLOW+='|^[<>][[:space:]]*position,?$'
ALLOW+='|^[<>][[:space:]]*(//.*)?$'
ALLOW+='|^[<>][[:space:]]*[]){};,[]*[[:space:]]*$'
ALLOW+='|^[<>][[:space:]]*None$'
ALLOW+='|^[<>][[:space:]]*where$'
ALLOW+='|^[<>][[:space:]]*self\.position = saved_pos;$'
ALLOW+='|^[<>][[:space:]]*let position = self\.position;$'

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
    echo "EXPECTED-ONLY DELTA: $name ($changed changed lines, all in the -0200 surfaces)"
  fi
done

echo "REVIEW COMPLETE overall=$overall"
exit $overall
