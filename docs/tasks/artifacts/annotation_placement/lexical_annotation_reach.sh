#!/usr/bin/env bash
# ANNOTATION-PLACEMENT.1 / LEX-ADJACENCY — where does the EXISTING lexical-annotation
# notation (`[> … ]` / `[>! … ]`) actually reach? Read-only, structural greps.
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"; cd "$REPO" || exit 1
c() { grep -cE 'follow_restriction|FollowRestriction|FollowItem' "$1" 2>/dev/null || true; }

echo "=== Q1. is the notation consumed by the PARSER codegen, or generator-only? ==="
printf '  %-46s %s hits   <- PARSER codegen\n'  "ast_pipeline/ast_based_generator.rs" "$(c rust/src/ast_pipeline/ast_based_generator.rs)"
printf '  %-46s %s hits   <- STIMULI generator\n' "ast_pipeline/stimuli_generator.rs"  "$(c rust/src/ast_pipeline/stimuli_generator.rs)"
printf '  %-46s %s hits   <- frontend (parses the notation)\n' "ebnf_frontend.rs"      "$(c rust/src/ebnf_frontend.rs)"
echo
echo "=== Q2. is the notation expressible in PGEN's OWN meta-grammar? ==="
echo "  productions mentioning follow/lexical/restriction in grammars/ebnf.ebnf:"
grep -nE 'follow|lexical|restriction' grammars/ebnf.ebnf | sed 's/^/    /' || echo "    (NONE)"
echo "  the annotation position admits only:"
grep -n 'annotation_list :=' grammars/ebnf.ebnf | sed 's/^/    /'
echo
echo "=== Q3. do real shipped grammars depend on it? ==="
for g in grammars/*.ebnf; do
  n=$(grep -cE '^\[>' "$g" 2>/dev/null || true)
  [ "${n:-0}" -gt 0 ] && printf '  %-44s %s before-rule lexical annotation(s)\n' "$g" "$n"
done
