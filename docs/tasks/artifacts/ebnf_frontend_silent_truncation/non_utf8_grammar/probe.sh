#!/usr/bin/env bash
# docs/tasks/artifacts/ebnf_frontend_silent_truncation/non_utf8_grammar/probe.sh — EBNF-FRONTEND-SILENT-TRUNCATION.5's bank.
# The four grammar readers used `fs::read_to_string`, so a Latin-1 `©` in a header comment refused the whole file.
# HISTORICAL RED, frozen (measured 2026-09-03 on ast_pipeline at 0dada5df, before the change):
#     $ ast_pipeline rust/test_data/ebnf_frontend/encoding_latin1.ebnf --emit-raw-ast-json x.json
#     Caused by: stream did not contain valid UTF-8      (rc 1; the UTF-16LE twin identically)
# Now every grammar reader goes through pgen::source_text::read_source_file (UTF-8 ± BOM, UTF-16 with BOM, else
# ISO-8859-1), corruption stays LOUD with a file offset, and a non-UTF-8 grammar reads to the SAME envelope as its
# UTF-8 twin. RED arms assert text. Scratch only.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2
AP=rust/target/debug/ast_pipeline; DR=rust/target/debug/ebnf_dual_run_diff; ID=scripts/check_baseline_identity.sh; G=scripts/check_ebnf_alternative_parity.sh
FX=rust/test_data/ebnf_frontend; W=rust/target/non_utf8_grammar_probe; rm -rf "$W"; mkdir -p "$W"; trap 'rm -rf "$ROOT/$W"' EXIT INT TERM HUP
pass=0; fail=0; ok(){ pass=$((pass+1)); printf 'PASS  %s\n' "$1"; }; ko(){ fail=$((fail+1)); printf 'FAIL  %s\n%s\n' "$1" "${2:-}"; }
[ -x "$AP" ] && [ -x "$DR" ] || { echo "REFUSING: ast_pipeline and ebnf_dual_run_diff are needed"; exit 2; }
# 1 GREEN: the Latin-1 and UTF-16LE twins READ, and their raw_ast digests equal the UTF-8 twin's
d8=$(bash $ID --digest ebnf_raw_ast $FX/encoding_utf8.ebnf 2>/dev/null | tail -1); dl=$(bash $ID --digest ebnf_raw_ast $FX/encoding_latin1.ebnf 2>/dev/null | tail -1); du=$(bash $ID --digest ebnf_raw_ast $FX/encoding_utf16le.ebnf 2>/dev/null | tail -1)
[ -n "$d8" ] && [ "$d8" = "$dl" ] && [ "$d8" = "$du" ] && ok "1  GREEN: Latin-1 and UTF-16LE grammars read to the UTF-8 twin's envelope (ebnf_raw_ast ${d8:0:12}… ×3)" || ko "1" "utf8=$d8 latin1=$dl utf16=$du"
# 2 GREEN: the decoder's notice reaches stderr for the Latin-1 file, naming the encoding and the offending offset
out="$("$AP" $FX/encoding_latin1.ebnf --emit-raw-ast-json "$W/l.json" 2>&1 >/dev/null)"
printf '%s' "$out" | grep -q 'source-encoding: .*encoding_latin1.ebnf decoded as' && printf '%s' "$out" | grep -q 'first non-UTF-8 byte at file offset' && ! printf '%s' "$out" | grep -q 'stream did not contain valid UTF-8' && ok "2  GREEN: stderr carries the source-encoding notice (encoding + first non-UTF-8 offset); the old refusal is gone" || ko "2" "$out"
# 3 GREEN: a UTF-8 grammar that include()s the Latin-1 file parses with the included rules
"$AP" $FX/encoding_includer_utf8.ebnf --emit-raw-ast-json "$W/i.json" >/dev/null 2>&1; rc=$?
[ $rc -eq 0 ] && python3 -c "import json; d=json.load(open('$W/i.json')); assert any(r[0][1]=='top' for r in d['raw_ast'])" 2>/dev/null && ok "3  GREEN: a UTF-8 includer of the Latin-1 file parses and carries its rules" || ko "3 rc=$rc"
# 4 RED: corruption stays LOUD — a UTF-8 BOM over invalid bytes is refused with a file offset
out="$("$AP" $FX/encoding_corrupt_bom.ebnf --emit-raw-ast-json "$W/c.json" 2>&1)"; rc=$?
[ $rc -ne 0 ] && printf '%s' "$out" | grep -q 'starts with a UTF-8 BOM but is not valid UTF-8' && printf '%s' "$out" | grep -q 'file offset' && ok "4  RED: a BOM over invalid bytes is REFUSED, naming the file offset (corruption is not decoded away)" || ko "4 rc=$rc" "$(printf '%s' "$out" | tail -3)"
# 5 GREEN: the dual-run differential reads the Latin-1 grammar (the second of the four readers)
"$DR" --input $FX/encoding_latin1.ebnf --output "$W/dr.json" --emit-ast-json "$W/dra.json" >/dev/null 2>&1; rc=$?
[ $rc -eq 0 ] && [ -s "$W/dra.json" ] && ok "5  GREEN: ebnf_dual_run_diff reads the Latin-1 grammar (rc 0, typed AST written)" || ko "5 rc=$rc"
# 6 GREEN: doctrine 36 pairs both readers on the Latin-1 grammar and they agree
printf '%s\n' "$FX/encoding_latin1.ebnf" > "$W/l.txt"; out="$(PGEN_ALTERNATIVE_PARITY_GRAMMARS="$W/l.txt" bash $G 2>&1)"; rc=$?
[ $rc -eq 0 ] && printf '%s' "$out" | grep -q 'OK (grammars=1 rules=3 not_evaluated=0)' && ok "6  GREEN: doctrine 36 on the Latin-1 grammar: both readers read it, 3 rules agree" || ko "6 rc=$rc" "$out"
# 7 STRUCTURAL: no grammar reader calls fs::read_to_string any more (the four sites of the .12c.1 enumeration)
n=$(grep -c 'fs::read_to_string(' rust/src/ebnf_frontend.rs rust/src/bin/ebnf_dual_run_diff.rs | awk -F: '{s+=$2} END{print s}')  # CALLS, not the comment that names the old call
m=$(grep -n 'expect("read grammar file")' -B2 rust/src/parser_registry.rs | grep -c 'read_source_file')
[ "$n" -eq 0 ] && [ "$m" -eq 1 ] && ok "7  STRUCTURAL: 0 fs::read_to_string in the frontend + dual-run; the registry's grammar scan reads through source_text" || ko "7 n=$n m=$m"
printf 'NON-UTF8-GRAMMAR: %d/%d as declared\n' "$pass" "$((pass+fail))"; [ "$fail" -eq 0 ]
