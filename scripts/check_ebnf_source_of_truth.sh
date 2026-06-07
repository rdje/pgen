#!/usr/bin/env bash
# scripts/check_ebnf_source_of_truth.sh
#
# EBNF-SOURCE-OF-TRUTH.5 (PGEN-EBNF-SOT-0004) enforcement gate.
#
# THE RULE: the EBNF — together with its @predicate / @generate / @semantic annotations — is the
# SINGLE SOURCE OF TRUTH for what a PGEN parser accepts. The stimuli generator derives samples from
# the EBNF and nothing else, so any acceptance constraint that lives OUTSIDE the EBNF — in a
# hand-written post-parse validation layer wired into the parser registry's parse paths — is
# INVISIBLE to the generator. The generator then emits structurally-valid strings the parser rejects,
# silently breaking the generator<->parser duality. Such out-of-band acceptance gates are DEFECTS:
# encode the constraint IN the EBNF (a semantic annotation, shared by generation and parsing) or
# remove/relax it. (Decision: docs/decisions/project_ebnf_is_single_source_of_truth.md;
# tree: docs/tasks/EBNF-SOURCE-OF-TRUTH.md.)
#
# This gate flags any NEW out-of-band acceptance validator so the invariant cannot silently regress.
# The repo-wide audit (EBNF-SOURCE-OF-TRUTH.2) found exactly ONE current instance, tracked for
# removal/relaxation by EBNF-SOURCE-OF-TRUTH.3:
#     crate::regex_compile_validation::validate_regex_compile_contract  (regex consumer parse path)
# Any OTHER `*_validation` module referenced from the parser registry is a NEW out-of-band gate -> FAIL.
#
# Sound by design: it passes clean on the current tree (verified at authoring) and only flags the
# unambiguous anti-pattern (a `crate::*_validation::` reference in the parse-dispatch registry).
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
fail=0

REGISTRY="rust/src/parser_registry.rs"
# Allowlist of out-of-band validator modules tolerated pending EBNF-SOURCE-OF-TRUTH.3 (one per line).
ALLOWED="regex_compile_validation"

if [ ! -f "$REGISTRY" ]; then
  echo "ebnf-source-of-truth: FAIL — $REGISTRY not found (expected the parser dispatch registry)" >&2
  exit 1
fi

# Every `*_validation` module referenced from the parser registry's parse paths (use or inline-qualified).
found="$(grep -oE 'crate::[a-z_]+_validation::' "$REGISTRY" 2>/dev/null \
  | sed -E 's/^crate::([a-z_]+_validation)::$/\1/' | sort -u || true)"
unexpected="$(printf '%s\n' "$found" | grep -v '^$' | grep -vxF "$ALLOWED" || true)"

if [ -n "$unexpected" ]; then
  echo "ebnf-source-of-truth: FAIL — a NEW out-of-band acceptance validator is wired into $REGISTRY:" >&2
  printf '    %s\n' $unexpected >&2
  echo "  The EBNF is the single source of truth for the accepted language. Encode this constraint as a" >&2
  echo "  semantic annotation in the grammar (so the stimuli generator honors it), or remove/relax it." >&2
  echo "  (Allowed pending EBNF-SOURCE-OF-TRUTH.3: ${ALLOWED}.) See docs/tasks/EBNF-SOURCE-OF-TRUTH.md." >&2
  fail=1
fi

if [ "$fail" -ne 0 ]; then
  exit 1
fi
echo "ebnf-source-of-truth: OK"
