#!/usr/bin/env bash
# docs/tasks/artifacts/sv_corpus_grad/not_source_text/probe.sh — SV-CORPUS-GRAD.13c.1's bank for the
# `not_source_text` adjudication class: the detector decides from BYTES, the key outranks the shape in the
# verdict direction, a must_accept key on non-source text REFUSES, the class is PERMANENT and bounded, and
# over the REAL corpus the detector fires on exactly the rows the manifest labels — ZERO on any adjudicated
# row. Every RED arm asserts the refusal TEXT (a crash also returns rc 1). Scratch only; nothing tracked mutated.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
A=stimuli/sv/adjudicate_external_corpus.py; I=stimuli/sv/corpus_verdict_coverage.py
M=stimuli/sv/characterization/adjudication_manifest.tsv
REG=docs/tasks/artifacts/sv_corpus_grad/verdict_coverage/permanent_deferral_ceilings.json
W=rust/target/not_source_text_probe; rm -rf "$W"; mkdir -p "$W"; trap 'rm -rf "$ROOT/$W"' EXIT INT TERM HUP
pass=0; fail=0
ok(){ pass=$((pass+1)); printf 'PASS  %s\n' "$1"; }; ko(){ fail=$((fail+1)); printf 'FAIL  %s\n%s\n' "$1" "${2:-}"; }
load='import importlib.util,sys; spec=importlib.util.spec_from_file_location("adj","'"$A"'"); adj=importlib.util.module_from_spec(spec); spec.loader.exec_module(adj)'

# 1 RED: a must_accept key on a memory image REFUSES, naming the broken key
out="$(python3 -c "$load
try:
    adj.apply_not_source_text('friscv','x.v','@00010000 93 00 00 00\n','must_accept','k'); print('NO REFUSAL')
except SystemExit as e: print(e)" 2>&1)"
printf '%s' "$out" | grep -qF 'broken key' && ! printf '%s' "$out" | grep -q 'NO REFUSAL' && ok "1  RED: must_accept on a \$readmemh image -> REFUSES with the broken-key text" || ko "1" "$out"
# 2 GREEN: a must_reject key OUTRANKS the shape (rejecting non-source text is a verdict) — untouched
out="$(python3 -c "$load
print(adj.apply_not_source_text('s','r.v','\"just a string\"\n','must_reject','k'))" 2>&1)"
[ "$out" = "('must_reject', 'k')" ] && ok "2  GREEN: must_reject on a bare string literal keeps its key untouched" || ko "2" "$out"
# 3 GREEN: an unkeyed row with non-source bytes becomes not_source_text, basis names the shape and the leaf
out="$(python3 -c "$load
print(adj.apply_not_source_text('Surelog','t.sv','-parse -d uhdm dut.sv -nobuiltin\n','chained_only','k'))" 2>&1)"
printf '%s' "$out" | grep -qF "'not_source_text'" && printf '%s' "$out" | grep -qF 'tool command line' && printf '%s' "$out" | grep -qF 'SV-CORPUS-GRAD.13c.1' && ok "3  GREEN: unkeyed tool-command-line row -> not_source_text with a shape-naming basis" || ko "3" "$out"
# 4 RED-by-construction: the detector's own ground truth is WIRED (a miscalibrated positive/negative refuses on every run)
out="$(python3 -c "$load
adj._self_check_not_source_text(); print('self-check ran')" 2>&1)"
[ "$out" = "self-check ran" ] && grep -q '^    _self_check_not_source_text()$' "$A" && ok "4  GREEN: the detector's ground truth runs clean AND is called from main() on every invocation" || ko "4" "$out"
# 5 RED: a widened detector would be caught — feed SOURCE TEXT shapes and demand None for every one
out="$(python3 -c "$load
bad=[t for t in ['', '// c\n', '\`define A 1\n', '\`include \"x.svh\"\n', 'module m; endmodule\n', 'CUSTOM_1,\n', 'abc\n', 'deadbeef cafe\n', '-1;\n', '\"open\n'] if adj.not_source_text(t) is not None]
print('WIDENED' if bad else 'tight', bad)" 2>&1)"
[ "$out" = "tight []" ] && ok "5  GREEN: ten source-text / degenerate shapes all classify None (detector is tight)" || ko "5" "$out"
# 6 GREEN over the REAL corpus: detector(file) fires on EXACTLY the rows labelled not_source_text; ZERO adjudicated rows
out="$(python3 - "$A" "$M" <<'PY' 2>&1
import sys, csv, importlib.util
spec=importlib.util.spec_from_file_location("adj",sys.argv[1]); adj=importlib.util.module_from_spec(spec); spec.loader.exec_module(adj)
def path(r): return ("stimuli/sv/uvm/"+r["relpath"]) if r["suite"]=="uvm-core" else f"stimuli/sv/subs/{r['suite']}/{r['relpath']}"
labelled=set(); fired=set(); adjudicated_hits=[]; suites={}; not_fail=0
for r in csv.DictReader(open(sys.argv[2]),delimiter="\t"):
    k=(r["suite"],r["relpath"])
    if r["adjudication"]=="deferred:not_source_text":
        labelled.add(k); suites[r["suite"]]=suites.get(r["suite"],0)+1
        if r["observed"]!="fail": not_fail+=1
    shape=adj.not_source_text(open(path(r),encoding="utf-8",errors="replace").read())
    if shape is not None:
        fired.add(k)
        if r["adjudication"]=="match" or r["adjudication"].startswith("divergence:"): adjudicated_hits.append((k,r["adjudication"]))
print(f"labelled={len(labelled)} fired={len(fired)} equal={labelled==fired} adjudicated_hits={len(adjudicated_hits)} observed_not_fail={not_fail} suites={dict(sorted(suites.items()))}")
PY
)"
[ "$out" = "labelled=350 fired=350 equal=True adjudicated_hits=0 observed_not_fail=0 suites={'Surelog': 1, 'friscv': 346, 'slang': 3}" ] && ok "6  GREEN: over all 16 336 corpus files the detector fires on EXACTLY the 350 labelled rows; 0 adjudicated rows hit" || ko "6" "$out"
# 7 RED: the class is BOUNDED — a register ceiling of 349 refuses naming the class
python3 -c "import json;d=json.load(open('$REG'));d['ceilings']['deferred:not_source_text']['ceiling']=349;json.dump(d,open('$W/low.json','w'))"
out="$(python3 "$I" --ceilings "$W/low.json" --outdir "$W/out7" 2>&1)"; rc=$?
[ "$rc" -eq 1 ] && printf '%s' "$out" | grep -qF 'not_source_text' && printf '%s' "$out" | grep -qF 'above its recorded ceiling' && ok "7  RED: ceiling 349 < 350 rows -> the instrument REFUSES naming deferred:not_source_text" || ko "7 rc=$rc" "$out"
# 8 HISTORICAL RED: at the pre-slice commit the class did not exist and the 350 rows sat in DARK chained_only
git show 4eebfe9d:"$M" > "$W/head.tsv" 2>/dev/null
out="$(python3 - "$W/head.tsv" "$M" <<'PY' 2>&1
import sys, csv
old={(r["suite"],r["relpath"]):r for r in csv.DictReader(open(sys.argv[1]),delimiter="\t")}
new=[r for r in csv.DictReader(open(sys.argv[2]),delimiter="\t") if r["adjudication"]=="deferred:not_source_text"]
had=sum(1 for r in old.values() if r["adjudication"]=="deferred:not_source_text")
was_dark=sum(1 for r in new if old[(r["suite"],r["relpath"])]["adjudication"]=="deferred:chained_only" and old[(r["suite"],r["relpath"])]["observed"]=="fail")
print(f"class_rows_at_4eebfe9d={had} now_labelled={len(new)} of_which_were_dark_chained_only={was_dark}")
PY
)"
[ "$out" = "class_rows_at_4eebfe9d=0 now_labelled=350 of_which_were_dark_chained_only=350" ] && ok "8  HISTORICAL RED: at 4eebfe9d the class had 0 rows and all 350 were DARK chained_only" || ko "8" "$out"
# 9 GREEN: strata.tsv carries the rows in the not_source column and NOT in dark; the tick column stays LAST
out="$(python3 - <<'PY' 2>&1
rows=[l.rstrip("\n").split("\t") for l in open("docs/tasks/artifacts/sv_corpus_grad/verdict_coverage/strata.tsv")]
h=rows[0]; ns=h.index("not_source"); d=h.index("dark"); r=[x for x in rows[1:] if x[0]=="deferred:not_source_text"][0]
print(f"not_source={r[ns]} dark={r[d]} last_col={h[-1]}")
PY
)"
[ "$out" = "not_source=350 dark=0 last_col=dark_no_backtick" ] && ok "9  GREEN: strata.tsv: 350 in not_source, 0 in dark, corpus-dependent tick column still last" || ko "9" "$out"
printf 'NOT-SOURCE-TEXT: %d/%d as declared\n' "$pass" "$((pass+fail))"; [ "$fail" -eq 0 ]
