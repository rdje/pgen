#!/usr/bin/env bash
# LANG-CAPABILITY-AUDIT.7 — include() resolution: the measured before -> after.
#
# Read-only w.r.t. the repo: every synthetic probe grammar lives in a `mktemp -d`
# removed on exit. The two TRACKED grammars below are only READ.
#
# Re-run:  bash docs/tasks/artifacts/lang_capability_audit/run_include_resolution_probes.sh
# Capture: docs/tasks/artifacts/lang_capability_audit/include_resolution_probes.txt
#
# BEFORE (measured by `.4`, driver `run_primitive_pricing_probes.sh`): the frontend
# recognized an include directive and skipped the line, so a two-file grammar loaded
# only its own rules with exit 0 and no diagnostic, and the linter then reported the
# included rules as UNDEFINED "likely a typo".
#
# Each case declares the verdict it requires. A divergence prints `⛔`.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$REPO_ROOT"

PIPELINE="rust/target/debug/ast_pipeline"
[[ -x "$PIPELINE" ]] || PIPELINE="rust/target/release/ast_pipeline"
if [[ ! -x "$PIPELINE" ]]; then
  echo "FATAL: no ast_pipeline binary; build with" >&2
  echo "  (cd rust && cargo build --features 'generated_parsers ebnf_dual_run' --bin ast_pipeline)" >&2
  exit 2
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
GAPS=0

# rule_count FILE -> how many rules the SHIPPING frontend loads
rule_count() {
  timeout 120 "$PIPELINE" "$1" --lint-grammar 2>&1 |
    grep -oE "\([0-9]+ rules\)" | head -1 | tr -dc '0-9'
}
undef_refs() {
  timeout 120 "$PIPELINE" "$1" --lint-grammar 2>&1 |
    grep -oE "undefined_references=[0-9]+" | head -1 | cut -d= -f2
}

want_rules() {
  local file="$1" want="$2" label="$3" got
  got="$(rule_count "$file")"; got="${got:-0}"
  if [[ "$got" == "$want" ]]; then
    printf '  OK  %-52s rules=%-6s [want %s]\n' "$label" "$got" "$want"
  else
    printf '  ⛔  %-52s rules=%-6s [want %s]\n' "$label" "$got" "$want"
    GAPS=$((GAPS + 1))
  fi
}

want_exit() {
  local file="$1" want="$2" label="$3" rc
  timeout 120 "$PIPELINE" "$file" --lint-grammar >/dev/null 2>&1
  rc=$?
  if [[ "$rc" == "$want" ]]; then
    printf '  OK  %-52s exit=%-6s [want %s]\n' "$label" "$rc" "$want"
  else
    printf '  ⛔  %-52s exit=%-6s [want %s]\n' "$label" "$rc" "$want"
    GAPS=$((GAPS + 1))
  fi
}

echo "LANG-CAPABILITY-AUDIT.7 — include() resolution probes"
echo "binary: $PIPELINE"

echo
echo "=== 1. THE CORE REPAIR — a two-file grammar composes ==="
mkdir -p "$WORK/basic"
printf 'digit := /[0-9]/\n'                      > "$WORK/basic/common.ebnf"
printf 'include("common")\n\nstart := digit+\n'  > "$WORK/basic/main.ebnf"
want_rules "$WORK/basic/main.ebnf" 2 'include("common") composes the included rule'
printf '  and the LINTER honours the graph: undefined_references=%s [want 0]\n' \
  "$(undef_refs "$WORK/basic/main.ebnf")"
echo '  (before this leaf: rules=1, undefined_references=1, "likely a typo")'

echo
echo "=== 2. TRACKED grammars that were silently losing rules ==="
want_rules grammars/systemverilog_lrm_profiled_wrapper.ebnf 1403 'SV profiled wrapper (was 3 — 99.8% lost)'
want_rules grammars/ebnf.ebnf 131 'ebnf.ebnf after the stale include was removed'
printf '  ebnf.ebnf undefined_references=%s [want 0 — it is self-contained]\n' \
  "$(undef_refs grammars/ebnf.ebnf)"

echo
echo "=== 3. AN UNRESOLVABLE INCLUDE IS A HARD ERROR (was: silent, exit 0) ==="
# The defect class this leaf closes is a directive that vanishes without a word.
# A silent partial resolve would preserve the disease, so this must fail loudly.
printf 'include("does_not_exist")\n\nstart := "a"\n' > "$WORK/bad.ebnf"
want_exit "$WORK/bad.ebnf" 1 'missing include target fails the load'
echo "  --- the diagnostic names the spec, the directive and the search path: ---"
timeout 120 "$PIPELINE" "$WORK/bad.ebnf" --lint-grammar 2>&1 |
  grep -F "unresolvable include" | head -1 | fold -w 100 -s | sed 's/^/    /'

echo
echo "=== 4. GRAPH SEMANTICS — cycles, diamonds, depth, spelling ==="
mkdir -p "$WORK/graph/sub" "$WORK/graph/dirinc"
# a <-> b: a cycle must terminate, contributing each file once.
printf 'include("b")\n\nstart := x y\n'      > "$WORK/graph/a.ebnf"
printf 'include("a")\n\nx := "x"\ny := "y"\n' > "$WORK/graph/b.ebnf"
want_rules "$WORK/graph/a.ebnf" 3 'CYCLE a<->b terminates, each file once'
# diamond: top -> {l, r} -> shared. `shared` must be composed exactly once.
printf 'shared := "s"\n'                       > "$WORK/graph/shared.ebnf"
printf 'include("shared")\nl := "l"\n'         > "$WORK/graph/l.ebnf"
printf 'include("shared")\nr := "r"\n'         > "$WORK/graph/r.ebnf"
printf 'include("l", "r")\n\ntop := l r shared\n' > "$WORK/graph/top.ebnf"
want_rules "$WORK/graph/top.ebnf" 4 'DIAMOND composes the shared file exactly once'
# depth 3, a BARE spec (ebnf.ebnf used this spelling), and a relative subdirectory.
printf 'deep := "d"\n'                         > "$WORK/graph/sub/deep.ebnf"
printf 'include("sub/deep")\nmid := deep\n'    > "$WORK/graph/mid.ebnf"
printf 'include(mid)\n\nroot := mid\n'         > "$WORK/graph/root.ebnf"
want_rules "$WORK/graph/root.ebnf" 3 'NESTED depth-3 + BARE spec + relative subdir'
# directory include: every *.ebnf, alphabetical.
printf 'b_rule := "b"\n'                       > "$WORK/graph/dirinc/b.ebnf"
printf 'a_rule := "a"\n'                       > "$WORK/graph/dirinc/a.ebnf"
printf 'include_dir("dirinc")\n\nmain := a_rule b_rule\n' > "$WORK/graph/dmain.ebnf"
want_rules "$WORK/graph/dmain.ebnf" 3 'include_dir() pulls every *.ebnf'

echo
echo "=== 5. THE ENTRY RULE MUST NOT MOVE ==="
# Downstream reachability and linting treat rule_order[0] as the canonical entry.
# If an included file supplied rule 0 the grammar would be silently re-rooted.
timeout 120 "$PIPELINE" "$WORK/graph/top.ebnf" --emit-raw-ast-json "$WORK/top.json" >/dev/null 2>&1
first="$(python3 -c "
import json; print(json.load(open('$WORK/top.json'))['raw_ast'][0][0][1])" 2>/dev/null)"
if [[ "$first" == "top" ]]; then
  printf '  OK  %-52s rule[0]=%s [want top]\n' 'main-file rule stays first' "$first"
else
  printf '  ⛔  %-52s rule[0]=%s [want top]\n' 'main-file rule stays first' "$first"
  GAPS=$((GAPS + 1))
fi
python3 -c "
import json
print('   full order:', [r[0][1] for r in json.load(open('$WORK/top.json'))['raw_ast']])" 2>/dev/null

echo
echo "=== 6. NO REGRESSION — grammars with no include are untouched ==="
# Pinned counts: any movement here means include resolution leaked into the
# single-file path.
for pair in "json:9" "regex:269" "vhdl:216" "rtl_frontend:169" "rtl_const_expr:48"; do
  g="${pair%%:*}"; want="${pair##*:}"
  want_rules "grammars/$g.ebnf" "$want" "grammars/$g.ebnf unchanged"
done

echo
echo "=== 7. EVERY EBNF CONSUMER INHERITS IT (the director's requirement) ==="
# All four in-repo callers funnel through parse_ebnf_file_to_raw_ast_envelope /
# parse_ebnf_text_to_raw_ast_envelope, so composing there is what makes support
# universal. Verified by behaviour, not by reading the call graph.
BASIC="$WORK/basic/main.ebnf"
timeout 120 "$PIPELINE" "$BASIC" --generate-parser --output "$WORK/gen.rs" >/dev/null 2>&1
gen_hits=$(grep -c 'fn parse_digit' "$WORK/gen.rs" 2>/dev/null; true)
if [[ "${gen_hits:-0}" -ge 1 ]]; then
  printf '  OK  %-52s fn parse_digit emitted\n' 'parser CODEGEN sees the included rule'
else
  printf '  ⛔  %-52s fn parse_digit MISSING\n' 'parser CODEGEN sees the included rule'
  GAPS=$((GAPS + 1))
fi

stim=$(timeout 120 "$PIPELINE" "$BASIC" --generate-stimuli --count 5 --seed 0 2>/dev/null |
       tr -d '\n ' | tr -dc '0-9' | head -c 20)
if [[ -n "$stim" ]]; then
  printf '  OK  %-52s emitted digits: %s\n' 'STIMULI generation exercises the included rule' "$stim"
else
  printf '  ⛔  %-52s no stimuli produced\n' 'STIMULI generation exercises the included rule'
  GAPS=$((GAPS + 1))
fi

timeout 120 "$PIPELINE" "$BASIC" --emit-raw-ast-json "$WORK/ra.json" >/dev/null 2>&1
names=$(python3 -c "
import json; print(','.join(r[0][1] for r in json.load(open('$WORK/ra.json'))['raw_ast']))" 2>/dev/null)
if [[ "$names" == "start,digit" ]]; then
  printf '  OK  %-52s raw_ast=%s\n' 'RAW-AST export (the pipeline input)' "$names"
else
  printf '  ⛔  %-52s raw_ast=%s [want start,digit]\n' 'RAW-AST export (the pipeline input)' "$names"
  GAPS=$((GAPS + 1))
fi
echo "  (--lint-grammar is covered by case 1; the parse harness and the equivalence"
echo "   suite call the same entry point, so they inherit resolution identically.)"

echo
echo "=== 8. RULE-DEFINITION UNIQUENESS (.9 — director ruling) ==="
# "when loading a given EBNF file, any rule definition shall be unique and any rule
# reference shall have one and only one rule definition."
#
# The unit of uniqueness is the FILE, not the clause. Repeating a header WITHIN one file
# is an established PGEN idiom — the clauses merge into alternatives of a single rule, so
# a reference still resolves to exactly one definition. Two DIFFERENT files defining the
# same name is the case with no such intent, and it used to merge silently.
mkdir -p "$WORK/uniq"
printf 'value := "x"\n'                                    > "$WORK/uniq/other.ebnf"
printf 'include("other")\n\nstart := value\nvalue := "y"\n' > "$WORK/uniq/collide.ebnf"
want_exit "$WORK/uniq/collide.ebnf" 1 'CROSS-FILE duplicate definition is rejected'
echo "  --- the diagnostic names the rule and BOTH files: ---"
timeout 120 "$PIPELINE" "$WORK/uniq/collide.ebnf" --lint-grammar 2>&1 |
  grep -F "duplicate rule definition" | head -1 | cut -c1-96 | sed 's/^/    /'

# The idiom must survive. These two are TRACKED, SHIPPED grammars that depend on it:
# json.ebnf gives each `value` alternative its own return annotation across 7 clauses,
# and rtl_const_expr.ebnf writes its precedence cascade the same way.
printf 'start := "a"\nstart := "b"\n' > "$WORK/uniq/multi.ebnf"
want_rules "$WORK/uniq/multi.ebnf" 1 'WITHIN-FILE multi-clause merges to one rule'
want_rules grammars/json.ebnf            9  'json.ebnf (7 `value` clauses) still loads'
want_rules grammars/rtl_const_expr.ebnf  48 'rtl_const_expr.ebnf (cascade clauses) still loads'
# A diamond visits the shared file twice but defines its rules once — must not false-positive.
want_rules "$WORK/graph/top.ebnf" 4 'DIAMOND does not false-positive as a duplicate'

printf '\n=== SUMMARY ===\n'
printf 'declared-verdict divergences (⛔): %s\n' "$GAPS"
[[ "$GAPS" -eq 0 ]] && { echo "all probes matched their declared verdicts"; exit 0; }
echo "one or more probes diverged from the verdict recorded in the leaf — re-adjudicate"
exit 1
