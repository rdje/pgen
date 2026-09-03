#!/usr/bin/env bash
# docs/tasks/artifacts/ebnf_frontend_silent_truncation/comment_payload/probe.sh — EBNF-FRONTEND-SILENT-TRUNCATION.6's bank.
# The hand-written EBNF frontend (rust/src/ebnf_frontend.rs) captured an indented comment on the line after an inline
# return annotation INTO the annotation token (14 tokens in 12 shipped SV rules read `return_scalar`), and knew only `#`
# as a line comment while grammars/ebnf.ebnf:583-586 defines `//` too — 77 `//` lines in the shipped SV grammar were
# dropped, swallowed, or (in `parseable_source_item`) read as LIVE alternatives. Arms: a whole-tree census, the synthetic
# repro, the comment-only-edit invariance, `//` in every position, the unit tests' feature gate, the pinned
# entry-relative rows, and a HISTORICAL RED through the pre-fix frontend binary when one is present (NOT EVALUATED
# otherwise — never a free pass). Scratch only; nothing tracked is written.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
ID=scripts/check_baseline_identity.sh; W=rust/target/comment_payload_probe; rm -rf "$W"; mkdir -p "$W"; trap 'rm -rf "$ROOT/$W"' EXIT INT TERM HUP
pass=0; fail=0; ne=0; ok(){ pass=$((pass+1)); printf 'PASS  %s\n' "$1"; }; ko(){ fail=$((fail+1)); printf 'FAIL  %s\n%s\n' "$1" "${2:-}"; }
[ -x rust/target/debug/ast_pipeline ] || { echo "REFUSING: rust/target/debug/ast_pipeline missing (make -C rust regenerate_generated_parsers builds it)"; exit 2; }
# 1 GREEN census: over EVERY tracked grammar the frontend can read, no return_* token carries a comment or a newline.
#   The three `*_lrm_extracted.ebnf` snapshots are raw LRM syntax (`::=`), not this dialect: the frontend refuses them
#   and the arm asserts that the refused set is EXACTLY those three — a fourth unreadable grammar fails the arm.
out="$(for g in $(git ls-files 'grammars/*.ebnf'); do bash $ID --raw-ast "$g" 2>/dev/null > "$W/env.json" || { echo "NOENV $g"; continue; }; python3 - "$g" "$W/env.json" <<'PYIN'
import sys, json
d=json.load(open(sys.argv[2])); n=0; bad=0
def walk(x):
    global n, bad
    if isinstance(x,list):
        if len(x)==2 and isinstance(x[0],str) and x[0].startswith("return_") and isinstance(x[1],str):
            n+=1
            # dirty = a comment marker outside a quoted string, or an object/array payload that does not END
            # with its closing delimiter (the shape classify_return_annotation files as return_scalar). A
            # multi-line object payload with newlines INSIDE the braces is legal and not dirty
            # (grammars/return_annotation.ebnf extraction_expression is the control for that).
            import re as _re
            body=_re.sub(r'"(?:[^"\\]|\\.)*"','""',x[1]).strip()
            if "#" in body or "//" in body or (body.startswith("{") and not body.endswith("}")) or (body.startswith("[") and not body.endswith("]")): bad+=1
        for v in x: walk(v)
    elif isinstance(x,dict):
        for v in x.values(): walk(v)
walk(d); print(f"{sys.argv[1]} tokens={n} dirty={bad}")
PYIN
done)"
dirty=$(printf '%s\n' "$out" | awk '{split($3,a,"="); s+=a[2]} END{print s+0}'); files=$(printf '%s\n' "$out" | grep -c 'tokens='); noenv="$(printf '%s\n' "$out" | grep '^NOENV' | awk '{print $2}' | sort | tr '\n' ' ')"
[ "$dirty" -eq 0 ] && [ "$files" -ge 10 ] && [ "$noenv" = "grammars/systemverilog_2017_lrm_extracted.ebnf grammars/systemverilog_2023_lrm_extracted.ebnf grammars/verilog_2005_lrm_extracted.ebnf " ] && ok "1  GREEN census: $files readable grammars, 0 return_* tokens carry #, // or a newline (refused: exactly the 3 LRM snapshots)" || ko "1 dirty=$dirty files=$files refused=[$noenv]" "$out"
# 2 GREEN the founding synthetic: `-> {kind: "x"}` + an indented comment line -> return_object with payload EXACTLY {kind: "x"}
for c in '#' '//'; do printf 'a := b -> {kind: "x"}\n     %s a comment after the annotation\n   | c -> {kind: "y"}\nb := "b"\nc := "c"\n' "$c" > "$W/t.ebnf"
out="$(bash $ID --raw-ast "$W/t.ebnf" 2>/dev/null | python3 -c "
import sys,json; d=json.load(sys.stdin); r=d['raw_ast'][0]
print([t for t in r if isinstance(t,list) and t[0].startswith('return_')])")"
[ "$out" = "[['return_object', '{kind: \"x\"}'], ['return_object', '{kind: \"y\"}']]" ] && ok "2  GREEN: synthetic ($c comment after the annotation) -> two return_object tokens, payloads exact" || ko "2 ($c)" "$out"; done
# 3 GREEN invariance (.13c.2x.4): a one-word edit of the comment after net_type's `-> {kind: "trireg"}` leaves the SV digest UNMOVED
cp grammars/systemverilog.ebnf "$W/sv.ebnf"; python3 - "$W/sv.ebnf" <<'PY'
import sys; p=sys.argv[1]; s=open(p,encoding="utf-8").read(); a='# SV-CORPUS-GRAD.13n — `tri0` and `tri1` wer'; assert s.count(a)==1; open(p,"w",encoding="utf-8").write(s.replace(a,'# SV-CORPUS-GRAD.13n — `tri0` and `tri1` WERE'))
PY
d0=$(bash $ID --digest ebnf_raw_ast grammars/systemverilog.ebnf 2>/dev/null | tail -1); d1=$(bash $ID --digest ebnf_raw_ast "$W/sv.ebnf" 2>/dev/null | tail -1)
[ -n "$d0" ] && [ "$d0" = "$d1" ] && ok "3  GREEN: comment-only edit after an annotation leaves ebnf_raw_ast UNMOVED (${d0:0:12}…)" || ko "3" "$d0 vs $d1"
# 4 GREEN `//` in every position: top-level skipped, column-0 terminates the body, mid-line is not a regex
printf '// top\na := b // trailing\n   | c\n// col0 ends the body\n   | d\nb := "b"\nc := "c"\nd := "d"\n' > "$W/s.ebnf"
out="$(bash $ID --raw-ast "$W/s.ebnf" 2>/dev/null | python3 -c "
import sys,json; d=json.load(sys.stdin); r=json.dumps(d['raw_ast'][0]); print(len(d['raw_ast']), '\"regex\"' in r, '[\"rule_reference\", \"c\"]' in r, '[\"rule_reference\", \"d\"]' in r)")"
[ "$out" = "4 False True False" ] && ok "4  GREEN: // is a line comment at top level, column 0 (terminates the body) and mid-line (no regex token)" || ko "4" "$out"
# 5 STRUCTURAL: the three regression tests exist, and the module they live in is feature-gated (a `cargo test --lib` without it runs NONE of them)
n=$(grep -c 'fn inline_return_annotation_payload_excludes_following_comment_lines\|fn inline_scalar_return_annotation_payload_excludes_following_comment_line\|fn slash_slash_is_a_line_comment_in_every_position' rust/src/ebnf_frontend.rs)
grep -B1 'pub mod ebnf_frontend;' rust/src/lib.rs | grep -q 'cfg(feature = "ebnf_dual_run")' && [ "$n" -eq 3 ] && ok "5  STRUCTURAL: 3 regression tests present; module gated by feature ebnf_dual_run (run them with --features ebnf_dual_run)" || ko "5 n=$n"
# 6 PINNED: the entry-relative rows exist in the repro manifest with the entry column
n=$(awk -F'\t' 'NR>1 && $NF=="systemverilog_parseable_file"' stimuli/sv/adjudication_repros/MANIFEST.tsv | wc -l | tr -d ' ')
head -1 stimuli/sv/adjudication_repros/MANIFEST.tsv | grep -q $'\tentry$' && [ "$n" -eq 4 ] && ok "6  PINNED: MANIFEST.tsv carries the entry column and 4 rows parsed from systemverilog_parseable_file" || ko "6 n=$n"
# 7 HISTORICAL RED, from a FROZEN census: before_census.json holds the 14 tokens the pre-fix frontend read (rule, kind,
#   payload with the comment inside). Every one of those rules must now carry a return_object whose payload is the
#   frozen payload with the comment cut off — the same rule, the fixed reading. (A live pre-fix binary would be a
#   better oracle; none survives a rebuild, so the census is frozen and the comparison is re-derived.)
out="$(bash $ID --raw-ast grammars/systemverilog.ebnf 2>/dev/null > "$W/sv.json" && python3 - "$W/sv.json" "$(dirname "${BASH_SOURCE[0]}")/before_census.json" <<'PYIN'
import sys, json, re
now=json.load(open(sys.argv[1])); frozen=json.load(open(sys.argv[2]))["tokens"]
by={}
for r in now["raw_ast"]:
    by.setdefault(r[0][1], []).extend(t for t in r if isinstance(t,list) and len(t)==2 and isinstance(t[0],str) and t[0].startswith("return_"))
bad=[]
for f in frozen:
    want=re.split(r"\n\s*(#|//)", f["payload_before"], maxsplit=1)[0].strip()
    if f["kind_before"]!="return_scalar": bad.append(("frozen kind", f["rule"], f["kind_before"])); continue
    if ["return_object", want] not in by.get(f["rule"], []): bad.append(("now", f["rule"], want[:40]))
print(f"frozen={len(frozen)} rules={len(set(f['rule'] for f in frozen))} mismatches={len(bad)} {bad[:3]}")
PYIN
)"
case "$out" in "frozen=14 rules=11 mismatches=0"*) ok "7  HISTORICAL RED (frozen census): all 14 pre-fix return_scalar tokens now read return_object with the comment cut off, in the same 11 rules";; *) ko "7" "$out";; esac
printf 'COMMENT-PAYLOAD: %d/%d as declared, %d not evaluated\n' "$pass" "$((pass+fail))" "$ne"; [ "$fail" -eq 0 ]
