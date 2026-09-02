#!/usr/bin/env bash
# docs/tasks/artifacts/lrm_grammar_fidelity/footnote_discriminator/probe.sh — LRM-GRAMMAR-FIDELITY.1f's bank.
# The LRM extractor's footnote canonicaliser stripped EVERY trailing digit run (merging strength0/strength1,
# delay2/delay3, and the digits off supply0/tri1/unique0/...); it now shares `.1e`'s discriminator. Arms:
# a HISTORICAL RED (the extractor as it stood at 4eebfe9d, run into scratch), a GREEN fresh run, the
# BLAST RADIUS pinned by name, the overwrite REFUSAL (text asserted), the .1e parity arms, the gap census
# gate, and the profile-inference refusal (text asserted). Scratch only; no tracked file is written.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
X=tools/extract_systemverilog_lrm_profiles.py
MD17=docs/systemverilog/2017/md/section-41-data-read-api.md; MD23=docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md
GEN=grammars/systemverilog_lrm_profiled_generated.ebnf
W=rust/target/footnote_discriminator_probe; rm -rf "$W"; mkdir -p "$W/old" "$W/new"; trap 'rm -rf "$ROOT/$W"' EXIT INT TERM HUP
pass=0; fail=0; ok(){ pass=$((pass+1)); printf 'PASS  %s\n' "$1"; }; ko(){ fail=$((fail+1)); printf 'FAIL  %s\n%s\n' "$1" "${2:-}"; }
git show 4eebfe9d:$X > "$W/old.py" 2>/dev/null
python3 "$W/old.py" --md-2017 $MD17 --md-2023 $MD23 --output-ebnf "$W/old/gen.ebnf" --output-report "$W/old/report.json" >/dev/null 2>&1; rc_old=$?
python3 $X --md-2017 $MD17 --md-2023 $MD23 --output-ebnf "$W/new/gen.ebnf" --output-report "$W/new/report.json" >/dev/null 2>&1; rc_new=$?
# 1 HISTORICAL RED: the 4eebfe9d extractor merges the pairs and strips the keywords' digits
if [ $rc_old -eq 0 ] && grep -q '^strength := ' "$W/old/gen.ebnf" && grep -q '^delay := ' "$W/old/gen.ebnf" && ! grep -q '^delay2 := ' "$W/old/gen.ebnf" && grep -q 'kw_supply_076df715' "$W/old/gen.ebnf"; then ok "1  HISTORICAL RED: at 4eebfe9d the fresh output has strength/delay MERGED and kw_supply_ (digit stripped)"; else ko "1 rc_old=$rc_old"; fi
# 2 GREEN: the fresh output splits both pairs and every *0/*1 keyword keeps its digit; no footnote leaks
if [ $rc_new -eq 0 ] && grep -q '^strength0 := ' "$W/new/gen.ebnf" && grep -q '^strength1 := ' "$W/new/gen.ebnf" && grep -q '^delay2 := ' "$W/new/gen.ebnf" && grep -q '^delay3 := ' "$W/new/gen.ebnf" && ! grep -q '^strength := \|^delay := ' "$W/new/gen.ebnf" && grep -q 'kw_supply0_' "$W/new/gen.ebnf" && grep -q 'kw_unique0_' "$W/new/gen.ebnf" && grep -q 'kw_tri0_' "$W/new/gen.ebnf" && ! grep -q 'kw_this41\|kw_eof56\|kw_dollar42\|kw_supply_0\|kw_tx_path' "$W/new/gen.ebnf"; then ok "2  GREEN: fresh output has strength0/strength1/delay2/delay3, kw_supply0/kw_unique0/kw_tri0, no footnote leak"; else ko "2 rc_new=$rc_new"; fi
# 3 BLAST RADIUS pinned by name: only these heads may appear/disappear/change between the two runs
out="$(python3 - "$W/old/gen.ebnf" "$W/new/gen.ebnf" <<'PY'
import re, sys
def rules(p):
    out={}
    for l in open(p,encoding="utf-8"):
        m=re.match(r"^([A-Za-z_][A-Za-z0-9_]*) := (.*)$", l.rstrip("\n"))
        if m: out[m.group(1)]=m.group(2)
    return out
b=rules(sys.argv[1]); a=rules(sys.argv[2])
kwb=re.compile(r"^kw_(bufif|highz|notif|pull|rtranif|supply|tranif|tx_path_delay_expression)_[0-9a-f]{8}$")
kwa=re.compile(r"^kw_(bufif0|bufif1|highz0|highz1|notif0|notif1|pull0|pull1|rtranif0|rtranif1|strong0|strong1|supply0|supply1|tranif0|tranif1|tri0|tri1|unique0|weak0|weak1)_[0-9a-f]{8}$")
ob=set(b)-set(a); oa=set(a)-set(b); ch={k for k in set(a)&set(b) if a[k]!=b[k]}
bad_ob=[k for k in ob if k not in {"delay","delay_sv_2017","delay_sv_2023","strength"} and not kwb.match(k)]
bad_oa=[k for k in oa if k not in {"delay2","delay3","strength0","strength1"} and not kwa.match(k)]
want_ch={"continuous_assign","drive_strength","enable_gatetype","gate_instantiation_sv_2017","gate_instantiation_sv_2023","list_of_path_delay_expressions","net_declaration_sv_2017","net_declaration_sv_2023","net_type","pass_en_switchtype","pulldown_strength","pullup_strength","udp_instantiation","unique_priority"}
print("OK" if not bad_ob and not bad_oa and ch==want_ch else f"UNEXPECTED only_old={bad_ob} only_new={bad_oa} changed_delta={sorted(ch^want_ch)}", f"only_old={len(ob)} only_new={len(oa)} changed={len(ch)}")
PY
)"
case "$out" in OK*) ok "3  BLAST RADIUS: exactly the strength/delay split, the *0/*1 keyword literals, and 14 pinned bodies ($out)";; *) ko "3" "$out";; esac
# 4 RED: the tool still REFUSES to overwrite the hand-maintained ancestor, and the file is untouched
before=$(git hash-object $GEN); out="$(python3 $X --md-2017 $MD17 --md-2023 $MD23 --output-ebnf $GEN --output-report "$W/r4.json" 2>&1)"; rc=$?
if [ $rc -ne 0 ] && printf '%s' "$out" | grep -q 'REFUSING to overwrite' && [ "$(git hash-object $GEN)" = "$before" ]; then ok "4  RED: writing the tracked ancestor without --promote-outputs -> REFUSING to overwrite; file untouched"; else ko "4 rc=$rc" "$(printf '%s' "$out" | tail -2)"; fi
# 5 PARITY: .1e's reader pins both readers equal AND both keep the four real digits
out="$(python3 stimuli/sv/lrm_keyword_enumeration_census.py --self-test 2>&1)"; rc=$?
if [ $rc -eq 0 ] && [ "$(printf '%s\n' "$out" | grep -c 'real digits survive in BOTH readers')" -eq 3 ] && [ "$(printf '%s\n' "$out" | grep -c '✓ reader parity')" -eq 6 ]; then ok "5  PARITY: .1e self-test rc 0 — 3 editions, both readers equal, strength0/strength1/delay2/delay3 in both"; else ko "5 rc=$rc" "$(printf '%s\n' "$out" | tail -3)"; fi
# 6 GAP CENSUS: the other scraper that calls extract_rules is green on its tracked artifact
out="$(python3 stimuli/sv/lrm_annex_a_gap_census.py 2>&1)"; rc=$?
[ $rc -eq 0 ] && ok "6  GREEN: lrm_annex_a_gap_census gate mode rc 0 (tracked artifact equals a fresh derivation)" || ko "6 rc=$rc" "$(printf '%s\n' "$out" | tail -2)"
# 7 RED: a markdown path outside the vendored LRM layout, no profile -> REFUSES naming the layout
out="$(python3 -c "
import importlib.util,sys,pathlib
spec=importlib.util.spec_from_file_location('x','$X'); m=importlib.util.module_from_spec(spec); sys.modules['x']=m; spec.loader.exec_module(m)
try: m.extract_rules(pathlib.Path('rust/target/nowhere/section-1.md')); print('NO REFUSAL')
except SystemExit as e: print(e)" 2>&1)"
printf '%s' "$out" | grep -q 'cannot tell which LRM edition' && ! printf '%s' "$out" | grep -q 'NO REFUSAL' && ok "7  RED: extract_rules on a path outside docs/{systemverilog,verilog}/<ed>/ -> refuses, names the layout" || ko "7" "$out"
# 8 GREEN per-file: a CLAUSE-BODY file (§8, an EXCERPT of Annex A) is judged against the edition's whole Annex A:
#   class_new19 (footnote 19 on an excerpted head) -> class_new; delay2 / supply0 kept; unsigned_number33 -> unsigned_number
out="$(python3 -c "
import importlib.util,sys,pathlib,glob
spec=importlib.util.spec_from_file_location('x','$X'); m=importlib.util.module_from_spec(spec); sys.modules['x']=m; spec.loader.exec_module(m)
f=glob.glob('docs/systemverilog/2017/md/section-8-classes*.md')[0]
rules,canon=m.extract_rules_with_canon(pathlib.Path(f))
print(len(rules)>0, canon('class_new19'), canon('unsigned_number33'), canon('delay2'), canon('supply0'), 'class_new19' in rules)" 2>&1)"
[ "$out" = "True class_new unsigned_number delay2 supply0 False" ] && ok "8  GREEN: §8 clause-body file judged against Annex A: class_new19->class_new (no phantom head), delay2/supply0 kept, unsigned_number33 stripped" || ko "8" "$out"
printf 'FOOTNOTE-DISCRIMINATOR: %d/%d as declared\n' "$pass" "$((pass+fail))"; [ "$fail" -eq 0 ]
