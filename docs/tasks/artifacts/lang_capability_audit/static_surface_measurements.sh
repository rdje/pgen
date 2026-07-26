#!/usr/bin/env bash
# LANG-CAPABILITY-AUDIT.3b — the ZERO-BUILD half of the row measurements.
#
#   bash docs/tasks/artifacts/lang_capability_audit/static_surface_measurements.sh
#
# Read-only source/CLI measurements for the row questions that are about whether an
# ENGINE SURFACE EXISTS AT ALL, where a parse probe would only ever re-confirm an
# absence. Needs nothing built beyond the DEBUG `ast_pipeline` (dual features).
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/../../../.." && pwd)"
cd "$REPO" || exit 1

echo "=== ROW 10/11 — the semantic-store predicate vocabulary (the DYNAMIC-guard surface) ==="
echo "  ENGINE_BUILTIN_PREDICATE_NAMES (semantic_runtime.rs):"
sed -n '/^const ENGINE_BUILTIN_PREDICATE_NAMES/,/^];/p' rust/src/ast_pipeline/semantic_runtime.rs \
  | grep -oE '"[a-z_]+"' | tr -d '"' | sed 's/^/    - /'
echo "  scope-local NEGATIVE query present? \
$(grep -c '"lacks_fact_in_current_scope"' rust/src/ast_pipeline/semantic_runtime.rs) occurrence(s)"
echo "  fact-RETRACTION directive present? \
$(grep -cE '(Retract|retract_fact)' rust/src/ast_pipeline/semantic_runtime.rs) occurrence(s)"
echo "  scope-local fact index is keyed by DEPTH, not scope identity:"
grep -n "by_scope_and_name" rust/src/ast_pipeline/semantic_runtime.rs | head -2 | sed 's/^/    /'
echo "  close_scope pops the chain but never retracts facts (the code says so):"
grep -n "never retracts\|never retracted on close" rust/src/ast_pipeline/semantic_runtime.rs | sed 's/^/    /'
echo

echo "=== ROW 13(b) — Unicode NORMALIZATION (PEP 3131 compares identifiers after NFKC) ==="
echo "  normalization crate in the dependency set:"
n=$(grep -ciE "unicode-normalization|unicode_normalization|^icu|caseless" rust/Cargo.toml)
echo "    $n match(es)  (0 = absent)"
echo "  NFKC/NFC/NFD mentions anywhere in rust/src:"
n=$(grep -rniE "nfkc|nfkd|\bnfc\b|unicode_normal" rust/src/ --include=*.rs | wc -l | tr -d ' ')
echo "    $n match(es)  (0 = absent)"
echo "  the ONLY 'normalize_identifier' in the tree, and what it does:"
grep -rn "fn normalize_identifier" rust/src/ --include=*.rs | sed 's/^/    /'
sed -n '/fn normalize_identifier/,/^}/p' rust/src/test_runner/normalization.rs | sed 's/^/      /'
echo

echo "=== ROW 12 — is there a cross-rule PRECEDENCE-LADDER declaration? ==="
echo "  what the validator accepts for @priority / @precedence:"
grep -n "expects an integer payload" rust/src/ast_pipeline/annotation_validator.rs | sed 's/^/    /'
echo "  the payload parser (integers / integer lists ONLY):"
grep -n "pub fn parse_semantic_numeric_list" rust/src/ast_pipeline/semantic_directive_registry.rs | sed 's/^/    /'
echo "  what the meta-grammar's OWN documentation block advertises:"
grep -n "@precedence" grammars/ebnf.ebnf | sed 's/^/    /'
echo "  ...and the engine test that pins that shape as INVALID:"
grep -n "fn semantic_validator_warns_on_invalid_priority_payload" rust/src/ast_pipeline/annotation_validator.rs | sed 's/^/    /'
grep -n 'content: "{level: 5}"' rust/src/ast_pipeline/annotation_validator.rs | sed 's/^/    /'
echo
echo "  re-run that oracle with:"
echo "    cd rust && cargo test --features 'generated_parsers ebnf_dual_run' --lib \\"
echo "      semantic_validator_warns_on_invalid_priority_payload"
echo

echo "=== ROW 10 PRE-REQUISITE — where quantifier give-back is decided in codegen ==="
grep -n "fn generate_quantified_logic" rust/src/ast_pipeline/ast_based_generator.rs | sed 's/^/    /'
echo "    (the loop breaks on the first failing iteration and enforces only a MIN"
echo "     count; no emission path re-tries the loop with fewer iterations)"
