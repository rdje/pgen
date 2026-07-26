#!/usr/bin/env bash
# LANG-CAPABILITY-AUDIT.4 — measured pricing inputs for the prioritized primitive roadmap.
#
# Read-only: every probe grammar is written to a TEMP directory and every generated
# artifact lands there too. No tracked grammar, no `generated/` artifact, no Rust
# source and no contract is touched. The repo is only READ.
#
# Re-run:  bash docs/tasks/artifacts/lang_capability_audit/run_primitive_pricing_probes.sh
# Capture: docs/tasks/artifacts/lang_capability_audit/primitive_pricing_probes.txt
#
# Purpose. `.4` must attach a COST MODEL to every gap before it becomes a work item
# ([[project_capability_growth_is_zero_cost_and_neutral]]). A cost model built on the
# `.3` matrix's prose verdicts would inherit their errors, so every row this leaf
# prices is re-measured against the SHIPPING binary first — the DESIGN-PRIOR-ART
# doctrine's "RE-MEASURE before citing engine behaviour" clause.
#
# Each probe declares the verdict it expects. A divergence prints `⛔`.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$REPO_ROOT"

PIPELINE="rust/target/debug/ast_pipeline"
if [[ ! -x "$PIPELINE" ]]; then
  PIPELINE="rust/target/release/ast_pipeline"
fi
if [[ ! -x "$PIPELINE" ]]; then
  echo "FATAL: no ast_pipeline binary; build with 'make -C rust ast_pipeline'" >&2
  exit 2
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

GAPS=0

hdr() { printf '\n=== %s ===\n' "$1"; }

# want_rule_count FILE N LABEL — how many rules does the SHIPPING frontend load?
want_rule_count() {
  local file="$1" want="$2" label="$3" got
  got="$("$PIPELINE" "$file" --lint-grammar 2>&1 |
          grep -oE "\([0-9]+ rules\)" | head -1 | tr -dc '0-9')"
  got="${got:-0}"
  if [[ "$got" == "$want" ]]; then
    printf '  OK  %-46s rules=%s [want %s]\n' "$label" "$got" "$want"
  else
    printf '  ⛔  %-46s rules=%s [want %s]\n' "$label" "$got" "$want"
    GAPS=$((GAPS + 1))
  fi
}

# want_codegen FILE {ok|fail} LABEL — does parser codegen succeed?
want_codegen() {
  local file="$1" want="$2" label="$3" rc got
  "$PIPELINE" "$file" --generate-parser --output "$WORK/out.rs" >/dev/null 2>&1
  rc=$?
  [[ $rc -eq 0 ]] && got=ok || got=fail
  if [[ "$got" == "$want" ]]; then
    printf '  OK  %-46s codegen=%-4s [want %s]\n' "$label" "$got" "$want"
  else
    printf '  ⛔  %-46s codegen=%-4s [want %s]\n' "$label" "$got" "$want"
    GAPS=$((GAPS + 1))
  fi
}

echo "LANG-CAPABILITY-AUDIT.4 — primitive pricing probes"
echo "binary: $PIPELINE"

# ─────────────────────────────────────────────────────────────────────────────
hdr "ROW 15 — grammar composition: include() is RECOGNIZED then DISCARDED"
# ebnf_frontend.rs:152-155 skips include directives without resolving them. The
# author-facing book chapter `docs/ebnf_parser_book/src/includes.md` states the
# opposite ("resolved into a single combined grammar before code generation").
mkdir -p "$WORK/inc"
printf 'digit := /[0-9]/\n'                 > "$WORK/inc/common.ebnf"
printf 'include("common")\n\nstart := digit+\n' > "$WORK/inc/main.ebnf"
# 2 rules would mean the include resolved; 1 means the included file was dropped.
want_rule_count "$WORK/inc/main.ebnf" 1 "include(\"common\") — included rule ABSENT"
echo "  --- and the diagnostic blames an innocent rule: ---"
"$PIPELINE" "$WORK/inc/main.ebnf" --lint-grammar 2>&1 | grep -F "[error]" | sed 's/^/    /'

hdr "ROW 15 — the drop reaches TRACKED grammars"
# A wrapper whose entire body is one include, and a dangling include in the
# meta-grammar itself. Neither is diagnosed, because the directive never resolves.
want_rule_count grammars/systemverilog_lrm_profiled_wrapper.ebnf   3    "SV profiled WRAPPER as loaded"
want_rule_count grammars/systemverilog_lrm_profiled_generated.ebnf 1400 "SV profiled GENERATED (its include target)"
printf '  note: the wrapper loses 1397 of 1400 rules (99.8%%) with exit 0 and no diagnostic\n'
printf '  ebnf.ebnf:18 include target exists? '
if [[ -f grammars/semantic_annotations.ebnf ]]; then echo "yes"; else
  echo "NO — grammars/semantic_annotations.ebnf is absent (dangling include, undetectable)"
fi

# ─────────────────────────────────────────────────────────────────────────────
hdr "ROW 6 — error recovery SHIPS (@recover/@sync/@panic_until) but @recover: true BREAKS CODEGEN"
# The `.3` matrix recorded row 6 as declared-unwired with 0 consumers, quoting the
# horizon record's "NOT yet in PGEN". The meta-grammar productions really do have 0
# consumers — but the CAPABILITY ships under a different surface: registered semantic
# directives (semantic_directive_registry.rs:253/269/273) that codegen turns into a
# `parser.recover_with_hints(...)` call (ast_based_generator.rs:4222-4257).
emit_recovery_grammar() {
  printf '%s\nstmt := "a" ";"\n     | "b" ";"\n\nstart := stmt+\n' "$1"
}
emit_recovery_grammar '# no annotations'            > "$WORK/rec_none.ebnf"
emit_recovery_grammar '@sync: [";", "end"]'         > "$WORK/rec_sync.ebnf"
emit_recovery_grammar '@recover: true'              > "$WORK/rec_on.ebnf"
emit_recovery_grammar '@recover: true
@recover_budget: 4'                                 > "$WORK/rec_one.ebnf"
emit_recovery_grammar '@recover: true
@recover_budget: 4
@recover_parse_budget: 8
@recover_global_budget: 16'                         > "$WORK/rec_all.ebnf"

want_codegen "$WORK/rec_none.ebnf" ok   "control: no recovery annotations"
want_codegen "$WORK/rec_sync.ebnf" ok   "@sync alone (inert without @recover)"
want_codegen "$WORK/rec_on.ebnf"   fail "@recover: true, NO budgets — DEFECT"
want_codegen "$WORK/rec_one.ebnf"  fail "@recover: true + 1 of 3 budgets — DEFECT"
want_codegen "$WORK/rec_all.ebnf"  ok   "@recover: true + ALL 3 budgets — the only usable form"

echo "  --- WHY: Option<usize> budgets interpolate to NOTHING when None ---"
"$PIPELINE" "$WORK/rec_on.ebnf" --generate-parser --output "$WORK/broken.rs" >/dev/null 2>&1
if [[ -f "$WORK/broken.rs.tokens_dump.rs" ]]; then
  grep -o 'recover_with_hints ("stmt".\{0,40\}' "$WORK/broken.rs.tokens_dump.rs" |
    head -1 | sed 's/^/    /'
  echo "    ^ three empty argument slots -> rustc: expected an expression"
fi
echo "  --- and the working form emits real arguments: ---"
"$PIPELINE" "$WORK/rec_all.ebnf" --generate-parser --output "$WORK/works.rs" >/dev/null 2>&1
grep -A 8 '\.recover_with_hints($' "$WORK/works.rs" | tr -d ' \n' | sed 's/^/    /' | cut -c1-120
echo

# ─────────────────────────────────────────────────────────────────────────────
hdr "ROW 6 — ZERO-COST test: does a NON-recovering parser pay?"
# Acceptance test (1) from [[project_capability_growth_is_zero_cost_and_neutral]]:
# non-users must pay ZERO. Measured as call sites, not as "negligible".
for f in rec_none rec_all; do
  "$PIPELINE" "$WORK/$f.ebnf" --generate-parser --output "$WORK/$f.rs" >/dev/null 2>&1
  defs=$(grep -c 'fn recover_with_hints' "$WORK/$f.rs" 2>/dev/null; true)
  calls=$(grep -c '\.recover_with_hints($' "$WORK/$f.rs" 2>/dev/null; true)
  printf '  %-12s helper-definitions=%s  CALL-SITES=%s\n' "$f" "${defs:-0}" "${calls:-0}"
done
echo "  => the helper is emitted unconditionally but INERT; only an opted-in rule calls it."

# ─────────────────────────────────────────────────────────────────────────────
hdr "ROW 1 — parametric productions: the DECLARED surface silently miscompiles"
# ebnf.ebnf:595 declares `parametric_rule := rule_name "[" parameter_list "]"`, i.e.
# ECMA-262's `Expression[In, Yield]`. But `[` is the OPTIONAL-ELEMENT form
# (ebnf.ebnf:288 — the same collision `.5` found for character classes), so the
# author-facing syntax lowers to an optional GROUP and passes every check.
cat > "$WORK/param.ebnf" <<'EOF'
start := expr[In, Yield]
expr := "e"
In := "i"
Yield := "y"
EOF
"$PIPELINE" "$WORK/param.ebnf" --emit-raw-ast-json "$WORK/param.json" >/dev/null 2>&1
echo "  expr[In, Yield] compiles to:"
python3 -c "
import json
for r in json.load(open('$WORK/param.json'))['raw_ast']:
    if r[0][1] == 'start':
        print('   ', r)
" 2>/dev/null
echo "  and the linter's verdict on it:"
"$PIPELINE" "$WORK/param.ebnf" --lint-grammar 2>&1 |
  grep -oE "undefined_references=[0-9]+ \(error\)" | sed 's/^/    /'
echo "    ^ LINT-CLEAN. The comma is swallowed; the parameter list became an optional group."

# ─────────────────────────────────────────────────────────────────────────────
hdr "ROW 5 — case-insensitive keywords: EXPRESSIBLE today, at 69x the boilerplate"
printf '  (?i:...) occurrences in grammars/vhdl.ebnf : %s\n' "$(grep -c '(?i' grammars/vhdl.ebnf)"
printf '  rules in grammars/vhdl.ebnf                : %s\n' \
  "$(grep -cE '^[a-zA-Z_][a-zA-Z0-9_]* *:=' grammars/vhdl.ebnf)"
echo "  the DECLARED alternative (ebnf.ebnf:524 case_control, ~i\"begin\") instead misparses:"
printf 'start := ~i"begin"\n' > "$WORK/case.ebnf"
"$PIPELINE" "$WORK/case.ebnf" --emit-raw-ast-json "$WORK/case.json" >/dev/null 2>&1
python3 -c "
import json
print('   ', json.load(open('$WORK/case.json'))['raw_ast'][0])
" 2>/dev/null
echo "    ^ the '~' is dropped and 'i' becomes a RULE REFERENCE."

# ─────────────────────────────────────────────────────────────────────────────
hdr "ROWS 10/11 — fact lifetime: the store is PAY-PER-USE (zero-cost precondition)"
# `.3b` named fact retraction / instance-scoped lifetime as the sole blocker on rows
# 10 and 11. Acceptance test (1) asks what a grammar that never emits a fact pays.
for f in generated/json_parser.rs generated/systemverilog_parser.rs; do
  if [[ -f "$f" ]]; then
    printf '  %-40s store-hits=%-4s lines=%s\n' "$f" \
      "$(grep -c 'fact_index\|emit_fact\|has_fact' "$f")" "$(wc -l < "$f" | tr -d ' ')"
  else
    printf '  %-40s (absent — regenerate with make -C rust focus_<grammar>)\n' "$f"
  fi
done
echo "  => a fact-free grammar emits ZERO store code, so a new fact-lifetime"
echo "     primitive inherits 'non-users pay zero' by construction."

# ─────────────────────────────────────────────────────────────────────────────
hdr "The declared-unwired productions: engine consumers (re-measured)"
# TWO columns on purpose. `.1` and `.3` counted with a bare substring grep; this
# leaf re-ran it word-anchored and two rows moved, so the method is shown, not just
# the number.
printf '  %-24s %8s %8s\n' "production" "substr" "word"
for s in parametric_rule parameter_list lexer_mode case_control case_modifier \
         error_production error_recovery_action panic_mode skip_to \
         import_statement grammar_inheritance named_capture rule_modifier; do
  sub=$(grep -rc "$s" rust/src --include=*.rs 2>/dev/null | awk -F: '{t+=$2} END{print t+0}')
  wrd=$(grep -rcw "$s" rust/src --include=*.rs 2>/dev/null | awk -F: '{t+=$2} END{print t+0}')
  printf '  %-24s %8s %8s%s\n' "$s" "$sub" "$wrd" \
    "$( [[ "$sub" != "$wrd" ]] && echo '   <- substring FALSE POSITIVE' )"
done
cat <<'NOTE'
  TWO WAYS THIS TABLE LIES, both measured in this run:
    FALSE POSITIVE — `parameter_list` matched `parse_macro_parameter_list`
      (rust/src/sv_preprocessor.rs:1079, SV macro parameters) and `named_capture`
      matched `named_captures` (the REGEX grammar's capture generation in
      stimuli_generator.rs). Word-anchoring removes both; `.1`'s verdict survives.
    FALSE NEGATIVE — a 0 means the META-GRAMMAR PRODUCTION is unconsumed. It says
      NOTHING about whether the capability exists. Row 6 is the proof: every
      error-recovery production reads 0, and error recovery SHIPS as registered
      semantic directives.
  => this table is a NAME census, never a capability inventory.
NOTE

printf '\n=== SUMMARY ===\n'
printf 'declared-verdict divergences (⛔): %s\n' "$GAPS"
if [[ "$GAPS" -eq 0 ]]; then
  echo "all probes matched their declared verdicts"
  exit 0
fi
echo "one or more probes diverged from the verdict recorded in the leaf — re-adjudicate"
exit 1
