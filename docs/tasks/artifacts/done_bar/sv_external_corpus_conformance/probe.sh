#!/usr/bin/env bash
# docs/tasks/artifacts/done_bar/sv_external_corpus_conformance/probe.sh — DONE-BAR.3b's bank: the SystemVerilog
# external-corpus CONFORMANCE gate and the leg-3 surface that reads it. Two bits are kept apart on purpose — the
# gate's EXIT CODE (a regression against the tracked answer-key agreement) and `full_pass` (leg 3's "the corpus,
# PASSING") — and every reader (the compare module, the family-status evaluator, the done-bar audit) must say
# UNMET on an honest full_pass=false artifact. RED arms assert text. Scratch only; the real gate's artifact is
# read, not produced, here (arm 5 refuses loudly when it is absent).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
CMP=stimuli/sv/corpus_conformance_compare.py; G=rust/scripts/sv_external_corpus_conformance_gate.sh; LIB=rust/scripts/lib/parser_family_status_bar.sh
REG=rust/test_data/grammar_quality/done_bar_family_register_v0.json; ART=rust/target/sv_external_corpus_conformance_gate
W=rust/target/sv_conformance_probe; rm -rf "$W"; mkdir -p "$W"; trap 'rm -rf "$ROOT/$W"' EXIT INT TERM HUP
pass=0; fail=0; ok(){ pass=$((pass+1)); printf 'PASS  %s\n' "$1"; }; ko(){ fail=$((fail+1)); printf 'FAIL  %s\n%s\n' "$1" "${2:-}"; }
# 1 the comparison's ground truth on literals (regression / improvement / no-verdict flip / refusal / full_pass)
out="$(python3 $CMP --self-test 2>&1)"; rc=$?; n=$(printf '%s\n' "$out" | grep -c '^  ✓')
[ $rc -eq 0 ] && [ "$n" -ge 8 ] && ok "1  compare self-test rc 0, $n ✓ arms" || ko "1 rc=$rc" "$out"
# 2 RED through the real CLI: a tracked match that stops matching -> exit 1, text names the regression
printf 'suite\trelpath\tobserved\texpected\tadjudication\tbasis\ns\ta\tpass\tmust_accept\tmatch\tk\ns\tb\tfail\tchained_only\tdeferred:chained_only\tk\n' > "$W/t.tsv"
printf 'suite\trelpath\tobserved\texpected\tadjudication\tbasis\ns\ta\tfail\tmust_accept\tdivergence:unexplained_rejects_valid\tk\ns\tb\tfail\tchained_only\tdeferred:chained_only\tk\n' > "$W/f.tsv"
out="$(python3 $CMP --tracked "$W/t.tsv" --fresh "$W/f.tsv" --out "$W/c.json" 2>&1)"; rc=$?
[ $rc -eq 1 ] && printf '%s' "$out" | grep -q '1 regression(s) against the recorded answer-key agreement' && ok "2  RED: a match that stops matching -> exit 1 naming the regression" || ko "2 rc=$rc" "$out"
# 3 GREEN sibling: identical manifests -> exit 0, full_pass false (a deferred row remains), and the text says which bit is which
out="$(python3 $CMP --tracked "$W/t.tsv" --fresh "$W/t.tsv" --out "$W/c2.json" 2>&1)"; rc=$?
[ $rc -eq 0 ] && printf '%s' "$out" | grep -q 'OK — 0 regressions; full_pass=False (leg 3 reads this bit, not the exit code)' && ok "3  GREEN: identical -> exit 0 with full_pass=False named as leg 3's bit" || ko "3 rc=$rc" "$out"
# 4 the register declares the surface and it is external-backed by the script's own text (the audit's grep)
python3 -c "
import json; s=json.load(open('$REG'))['families']['systemverilog']['leg3_surface']; assert s['gate']=='sv_external_corpus_conformance_gate' and s['pass_query']=='.full_pass == true'" 2>/dev/null && grep -q 'stimuli/sv/subs' $G && grep -q 'stimuli/sv/uvm' $G && ok "4  register: leg3_surface declared; the gate script names both corpus roots (external-backed)" || ko "4"
# 5 the REAL artifact (the gate must have run on this machine): no regression, full_pass FALSE, counts equal the tracked tuple's
if [ -s "$ART/summary.json" ]; then
  out="$(jq -r '[.no_regression, .full_pass, .regressions_total, .sv_2017.match, .sv_2017.unexplained, .sv_2017.routed, .sv_2017.no_verdict, .sv_2017.rows] | @tsv' "$ART/summary.json")"
  [ "$out" = "$(printf 'true\tfalse\t0\t5851\t268\t2606\t6174\t16336')" ] && ok "5  REAL artifact: no_regression=true full_pass=false regressions=0 match=5851 unexplained=268 routed=2606 no_verdict=6174 rows=16336 (= the published tuple)" || ko "5" "$out"
else ko "5  no artifact at $ART/summary.json — run: make -C rust sv_external_corpus_conformance_gate"; fi
# 6 the family-status evaluator on the tracked register: surface valid, reachable, artifact present, pass assertion FALSE -> UNMET, and says so
out="$(bash -c "source $LIB; PGEN_FAMILY_STATUS_BAR_LIB_ROOT=\$PWD; mkdir -p $W/st; family_done_bar_leg3 systemverilog $W/st; echo MET=\$DONE_BAR_LEG3_MET; echo GATE=\$DONE_BAR_LEG3_SURFACE_GATE; echo DETAIL=\$DONE_BAR_LEG3_DETAIL" 2>&1)"
printf '%s' "$out" | grep -q 'MET=false' && printf '%s' "$out" | grep -q 'GATE=sv_external_corpus_conformance_gate' && printf '%s' "$out" | grep -q 'does NOT satisfy its declared pass assertion' && ok "6  evaluator: MET=false, surface named, 'does NOT satisfy its declared pass assertion' (honest UNMET, not UNPROVEN, not MET)" || ko "6" "$(printf '%s' "$out" | tail -4)"
# 7 RED: a register naming a *triage* gate as the surface is REFUSED by the evaluator with the vhdl lesson
python3 -c "
import json; d=json.load(open('$REG')); d['families']['systemverilog']['leg3_surface']['gate']='sv_external_corpus_triage_gate'; json.dump(d, open('$W/reg_triage.json','w'))"
out="$(bash -c "source $LIB; PGEN_FAMILY_STATUS_BAR_LIB_ROOT=\$PWD; PGEN_FAMILY_STATUS_DONE_BAR_REGISTER=$W/reg_triage.json family_done_bar_leg3 systemverilog $W/st" 2>&1)"; rc=$?
[ $rc -eq 2 ] && printf '%s' "$out" | grep -q 'a TRIAGE gate is not a conformance' && ok "7  RED: a triage gate declared as the surface is REFUSED (rc 2, the vhdl 0.058% lesson)" || ko "7 rc=$rc" "$out"
# 8 the done-bar audit's C7 controls are wired (presence is not a pass) and the audit runs its controls green
out="$(PGEN_DONE_BAR_TARGET_DIR="$W/empty" bash scripts/audit_done_bar.sh --json "$W/audit.json" 2>&1)"; rc=$?
# ⛔ rc alone is one bit: the first cut accepted rc 1 here while the audit had CRASHED (NameError) — assert no traceback,
#    not MISCALIBRATED, and the leg-3 line that proves the audit EVALUATED the surface's assertion over the real artifact.
out2="$(bash scripts/audit_done_bar.sh --json "$W/audit_local.json" 2>&1)"; rc2=$?
if [ $rc -ne 3 ] && [ $rc2 -ne 3 ] && ! printf '%s' "$out$out2" | grep -q 'Traceback' && printf '%s' "$out2" | grep -q 'pass_query `.full_pass == true` is FALSE over the artifact' && grep -q 'C7 conformance artifact with full_pass=false is NOT a pass' scripts/audit_done_bar.sh; then ok "8  audit: no traceback, C7 controls present, and on this machine's artifact it reads 'pass_query .full_pass == true is FALSE' (evaluated, not presumed)"; else ko "8 rc=$rc rc2=$rc2" "$(printf '%s\n' "$out2" | grep -i 'traceback\|error\|conformance' | head -4)"; fi
printf 'SV-CORPUS-CONFORMANCE: %d/%d as declared\n' "$pass" "$((pass+fail))"; [ "$fail" -eq 0 ]
