#!/usr/bin/env bash
# PGEN-RGX-0078-0198 expected-delta review: every regenerated artifact may
# differ from its banked pre-regen copy ONLY inside the `prepare_parse_state`
# ceremony (the comment block + the reset call replacing the four-step
# snapshot/new/replay/clone ceremony). Any hunk outside that region fails.
set -euo pipefail
R=/Users/richarddje/Documents/github/pgen
S=${R0198_SCRATCH:?set R0198_SCRATCH to the session scratch dir}
overall=0

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
  # Changed lines (added or removed) that are NOT part of the expected
  # ceremony replacement. The expected region is identified by its
  # distinctive tokens on both sides of the diff.
  stray=$(diff "$pre" "$post" \
    | grep -E '^[<>]' \
    | grep -vE 'SV-EXH-PROOF\.3\.3\.4\.b\.6\.2\.37\.2|SV-EXH-PROOF\.3\.3\.4\.b\.5\.1\.5\.c|PGEN-RGX-0078-0198|preloaded_facts|push_fact_record\(record\)|SemanticRuntimeState::new\(\)|set_predicate_defs\(|clone_predicate_defs\(\)|reset_for_new_parse\(|predicate_defs_map\(\)|^[<>][[:space:]]*(//.*)?$|^[<>][[:space:]]*\);?[[:space:]]*$|^[<>][[:space:]]*for record in|^[<>][[:space:]]*}[[:space:]]*$|^[<>][[:space:]]*(self)?\.semantic_runtime_state$|^[<>][[:space:]]*\.set_predicate_defs|^[<>][[:space:]]*\.facts\(\)$|^[<>][[:space:]]*\.to_vec\(\);$|^[<>][[:space:]]*let preloaded_facts' \
    || true)
  if [[ -n "$stray" ]]; then
    echo "UNEXPECTED DELTA in $name:"
    echo "$stray" | head -20
    overall=1
  else
    changed=$(diff "$pre" "$post" | grep -cE '^[<>]' || true)
    echo "EXPECTED-ONLY DELTA: $name ($changed changed lines, all in the prepare_parse_state ceremony)"
  fi
done

echo "REVIEW COMPLETE overall=$overall"
exit $overall
