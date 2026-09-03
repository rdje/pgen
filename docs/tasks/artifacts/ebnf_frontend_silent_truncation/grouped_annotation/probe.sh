#!/usr/bin/env bash
# docs/tasks/artifacts/ebnf_frontend_silent_truncation/grouped_annotation/probe.sh — EBNF-FRONTEND-SILENT-TRUNCATION.4's
# bank. An inline `->` return annotation INSIDE a parenthesized/optional group is legal by grammars/ebnf.ebnf and not
# implemented by the frontend/IR; it used to be re-read as the QUANTIFIER of the preceding element (frozen in
# before_envelope.json). It is now REFUSED at the byte, naming the rule and the lift workaround. RED arms assert the
# refusal TEXT; the GREEN arms are the lifted form and the string/regex controls on the same wiring; the meta-parser
# arm proves the divergence is now LOUD (frontend refuses, meta-grammar accepts). Scratch only.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
AP=rust/target/debug/ast_pipeline; DR=rust/target/debug/ebnf_dual_run_diff; G=scripts/check_ebnf_alternative_parity.sh
W=rust/target/grouped_annotation_probe; rm -rf "$W"; mkdir -p "$W"; trap 'rm -rf "$ROOT/$W"' EXIT INT TERM HUP
pass=0; fail=0; ok(){ pass=$((pass+1)); printf 'PASS  %s\n' "$1"; }; ko(){ fail=$((fail+1)); printf 'FAIL  %s\n%s\n' "$1" "${2:-}"; }
[ -x "$AP" ] && [ -x "$DR" ] || { echo "REFUSING: ast_pipeline and ebnf_dual_run_diff are needed"; exit 2; }
printf 'a := (b -> {kind: "x"} | c) d\nb := "b"\nc := "c"\nd := "d"\n' > "$W/paren.ebnf"
printf 'a := [ b -> $1 ] d\nb := "b"\nd := "d"\n' > "$W/opt.ebnf"
printf 'a := g d\ng := b -> {kind: "x"} | c\nb := "b"\nc := "c"\nd := "d"\n' > "$W/lifted.ebnf"
printf 'a := ( "->" | c ) d\nc := "c"\nd := "d"\n' > "$W/str.ebnf"
printf 'a := ( /->/ | c ) d\nc := "c"\nd := "d"\n' > "$W/re.ebnf"
# 1-2 RED: refused at the byte, naming the rule and the workaround
for k in paren opt; do out="$("$AP" "$W/$k.ebnf" --emit-raw-ast-json "$W/$k.json" 2>&1)"; rc=$?
  if [ $rc -ne 0 ] && printf '%s' "$out" | grep -q "rule 'a'" && printf '%s' "$out" | grep -q 'inside a parenthesized/optional group at byte' && printf '%s' "$out" | grep -q 'lift the group into a named rule'; then ok "RED ($k): the frontend REFUSES, names rule 'a', the byte and the lift workaround"; else ko "RED ($k) rc=$rc" "$(printf '%s' "$out" | tail -3)"; fi; done
# 3 GREEN: the lifted form parses and carries the annotation on the lifted rule
"$AP" "$W/lifted.ebnf" --emit-raw-ast-json "$W/lifted.json" >/dev/null 2>&1; rc=$?
out="$(python3 -c "
import json; d=json.load(open('$W/lifted.json')); r=[x for x in d['raw_ast'] if x[0][1]=='g'][0]; print(json.dumps(r))" 2>&1)"
[ $rc -eq 0 ] && printf '%s' "$out" | grep -q '\["return_object", "{kind: \\"x\\"}"\]' && ok "3  GREEN: the lifted form parses; g carries the per-branch return_object" || ko "3 rc=$rc" "$out"
# 4 GREEN controls: `->` inside a string / a regex inside a group is not an annotation and not a quantifier
for k in str re; do "$AP" "$W/$k.ebnf" --emit-raw-ast-json "$W/$k.json" >/dev/null 2>&1; rc=$?
  q=$(python3 -c "import json; print(json.dumps(json.load(open('$W/$k.json'))['raw_ast'][0]).count('\"quantifier\"'))" 2>/dev/null)
  [ $rc -eq 0 ] && [ "$q" = "0" ] && ok "4  GREEN ($k): \`->\` inside a $( [ $k = str ] && echo string || echo regex ) in a group parses, no quantifier token" || ko "4 ($k) rc=$rc q=$q"; done
# 5 HISTORICAL RED from the frozen envelope: the pre-fix frontend read the payload as a quantifier of b
out="$(python3 -c "
import json; f=json.load(open('$(dirname "${BASH_SOURCE[0]}")/before_envelope.json')); t=f['arm1_rule_a']
print(['quantifier','kind: \"x\"'] in t, any(x[0].startswith('return_') for x in t))")"
[ "$out" = "True False" ] && ok "5  HISTORICAL RED (frozen): the pre-fix envelope carries [quantifier, kind: \"x\"] and NO return_* token" || ko "5" "$out"
# 6 doctrine 36 seam: the shape is NOT EVALUATED as 'refused by the frontend' (alone -> rc 2), never compared silently
printf '%s\n' "$W/paren.ebnf" > "$W/l.txt"; out="$(PGEN_ALTERNATIVE_PARITY_GRAMMARS="$W/l.txt" bash $G 2>&1)"; rc=$?
[ $rc -eq 2 ] && printf '%s' "$out" | grep -q 'refused by the frontend' && ok "6  doctrine 36: NOT EVALUATED 'refused by the frontend' (alone rc 2)" || ko "6 rc=$rc" "$out"
# 7 the divergence is LOUD now: the meta-parser still ACCEPTS the construct (a return_annotation inside the grouped alternation)
"$DR" --input "$W/paren.ebnf" --output "$W/dr.json" --emit-ast-json "$W/ast.json" >/dev/null 2>&1; rc=$?
out="$(python3 -c "
import json; s=json.dumps(json.load(open('$W/ast.json'))); print(rc if False else ('return_annotation' in s and 'grouped' in s))" 2>&1)"
[ $rc -eq 0 ] && [ "$out" = "True" ] && ok "7  LOUD divergence: the parser generated from ebnf.ebnf accepts the construct (return_annotation inside grouped) while the frontend refuses it" || ko "7 rc=$rc" "$out"
printf 'GROUPED-ANNOTATION: %d/%d as declared\n' "$pass" "$((pass+fail))"; [ "$fail" -eq 0 ]
