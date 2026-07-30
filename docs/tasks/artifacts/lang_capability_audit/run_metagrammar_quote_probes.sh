#!/usr/bin/env bash
# LANG-CAPABILITY-AUDIT.10.5 — driver for the meta-grammar unterminated-quote defect.
#
# WHAT IT PROVES
#   `grammars/ebnf.ebnf`'s `single_quoted_string` regex opened a quote and never
#   closed it, so a `'` swallowed input to the next `'`/`\` — ACROSS NEWLINES. The
#   self-hosting meta-parser therefore ACCEPTED arbitrary text, and its green verdict
#   over the tracked grammars was a FALSE GREEN.
#
# ARMS
#   A  static  — the defect sites, and the bug-class sweep census over every tracked
#                grammar (each non-instance carries the reason it is not one).
#   B  behaviour — declared-verdict probes through the REAL generated meta-parser
#                (`generated/ebnf.rs` via `ebnf_dual_run_diff`).
#   C  corpus  — the tracked-grammar acceptance map (the leaf's regression oracle).
#
# ⭐ GROUND TRUTH IS PINNED INSIDE THE INSTRUMENT (feedback_instrument_needs_ground_truth):
#   POSITIVE control  `quoted_ok`      — a LEGAL single-quoted string must still parse.
#   NEGATIVE control  `dq_unterminated` — the already-correct DOUBLE-quoted twin must
#                     still reject. It shares every code path with the fixed rule except
#                     the one character, so if it ever flips, the instrument is lying and
#                     this script REFUSES rather than reporting a verdict.
#
# Deterministic: fixed inputs, no seeds, no clock. Re-runnable from a clean tree.
# Regenerates `generated/ebnf.rs` from the CURRENT `grammars/ebnf.ebnf` with BOTH the
# input and the output path PINNED (`.5` / `BIN-BUILD-INTEGRITY.3`) — generating to a
# different path makes byte-identity structurally unachievable, because the emitted
# parser embeds its own output path as a logging literal (the trap `.10.1` banked).
#
# Usage:  bash docs/tasks/artifacts/lang_capability_audit/run_metagrammar_quote_probes.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

AST_PIPELINE="./rust/target/debug/ast_pipeline"
DUAL_RUN="./rust/target/debug/ebnf_dual_run_diff"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

divergences=0
note_divergence() {
    divergences=$((divergences + 1))
    printf '   ⛔ DIVERGENCE: %s\n' "$1"
}

echo "==============================================================================="
echo "LANG-CAPABILITY-AUDIT.10.5 — meta-grammar unterminated-quote probes"
echo "==============================================================================="
echo

# ─────────────────────────────────────────────────────────────────────────────
echo "ARM A — static: the defect sites and the bug-class sweep"
echo "-------------------------------------------------------------------------------"
echo "The two string rules of the meta-grammar, side by side:"
grep -n 'double_quoted_string := \|single_quoted_string := ' grammars/ebnf.ebnf grammars/semantic_annotation.ebnf
echo
echo "Sweep — every quote-opening regex terminal in a tracked, PGEN-authored grammar."
echo "(The LRM-extracted intermediates are excluded: they are raw LRM BNF, not PGEN"
echo " grammars, and none of them parses today for unrelated reasons.)"
grep -rnE "^\s*[a-zA-Z_][a-zA-Z0-9_]*\s*:?:?=\s*/[^/]*\\\\?['\"]" \
    grammars/*.ebnf grammars/scratch/*.ebnf \
  | grep -vE "systemverilog_2017_lrm|systemverilog_2023_lrm|verilog_2005_lrm|lrm_profiled" \
  | sed 's/^/   /'
echo
# ⚠️ The adjudication below DERIVES its line numbers from the tree instead of hard-coding
# them. The first revision hard-coded them and they went stale within one leaf — .10.5's own
# comment block shifted every site below it. A citation that drifts is worse than no citation.
loc() { grep -n "^$2" "$1" | head -1 | cut -d: -f1; }
cat <<ADJUDICATION
   Adjudication of every quote-opening regex terminal (line numbers derived, not pinned):
     ebnf.ebnf:$(loc grammars/ebnf.ebnf 'single_quoted_string := ') single_quoted_string
                                                 ⛔ WAS THE DEFECT — opened \`'\`, never closed it
     semantic_annotation.ebnf:$(loc grammars/semantic_annotation.ebnf 'single_quoted_string := ') single_quoted_string
                                                 ⛔ WAS THE DEFECT — byte-identical twin
     ebnf.ebnf:$(loc grammars/ebnf.ebnf 'double_quoted_string := ') / semantic_annotation.ebnf:$(loc grammars/semantic_annotation.ebnf 'double_quoted_string := ') double_quoted_string
                                                 ✅ correct — closing \`"\` present (the CONTROL)
     ebnf.ebnf:$(loc grammars/ebnf.ebnf 'raw_quoted_string := ') raw_quoted_string, semantic_annotation.ebnf:$(loc grammars/semantic_annotation.ebnf 'raw_string := ') raw_string
                                                 ✅ correct — closing \`"\` present
     semantic_annotation.ebnf:$(loc grammars/semantic_annotation.ebnf 'multiline_string := ') multiline_string
                                                 ✅ correct — closing \`"""\` present
     json.ebnf:$(loc grammars/json.ebnf 'string := ') string                          ✅ correct — closing \`"\` present
     builtin_semantic_annotation.ebnf:$(loc grammars/builtin_semantic_annotation.ebnf 'dq_char := ')/$(loc grammars/builtin_semantic_annotation.ebnf 'sq_char := ') dq_char / sq_char
                                                 ✅ N/A — per-CHARACTER rules; the quotes
                                                    are matched by their wrappers
                                                    (\`sq_string := "'" sq_char* "'"\`)
     return_annotation.ebnf:$(loc grammars/return_annotation.ebnf 'string_content_double := ')/$(loc grammars/return_annotation.ebnf 'string_content_single := ') string_content_double / string_content_single
                                                 ✅ N/A — content-only rules; the quotes
                                                    are matched by their wrapper
     systemverilog.ebnf:$(loc grammars/systemverilog.ebnf 'integral_number := ') integral_number      ✅ N/A — the SV sized-literal apostrophe
                                                    (\`8'hFF\`), not a string delimiter
   ⇒ exactly TWO instances, both fixed by .10.5. No silent second instance.
ADJUDICATION
echo

# ─────────────────────────────────────────────────────────────────────────────
echo "ARM B — behaviour: declared verdicts through the REAL generated meta-parser"
echo "-------------------------------------------------------------------------------"
# ⚠️ The `focus_*` make targets rebuild `ast_pipeline` with THEIR feature set,
# DROPPING `ebnf_dual_run` — the trap `.10.4` banked, and this driver hit it for real when
# run straight after a byte-identity sweep. Rebuild with both features unconditionally
# rather than refusing, so this evidence is re-runnable from ANY tree state.
echo "Ensuring ast_pipeline carries both features (focus_* targets drop ebnf_dual_run)..."
if ! (cd rust && cargo build --features "generated_parsers ebnf_dual_run" \
        --bin ast_pipeline) >"$WORK/astbuild.log" 2>&1; then
    echo "   ⛔ REFUSING: could not build the dual-feature ast_pipeline."
    grep -E '^error' "$WORK/astbuild.log" | head -5
    exit 1
fi
"$AST_PIPELINE" --report-feature-surface | sed 's/^/   /'

echo "Regenerating generated/ebnf.rs from the CURRENT grammars/ebnf.ebnf"
echo "(input AND output paths pinned — the .10.1 path-embedding trap)..."
if ! "$AST_PIPELINE" grammars/ebnf.ebnf --emit-raw-ast-json generated/ebnf.json >"$WORK/gen.log" 2>&1; then
    echo "   ⛔ REFUSING: raw-AST emission failed."
    tail -5 "$WORK/gen.log"
    exit 1
fi
if ! "$AST_PIPELINE" --generate-parser --bootstrap-mode --debug --eliminate-left-recursion \
        generated/ebnf.json -o generated/ebnf.rs >>"$WORK/gen.log" 2>&1; then
    echo "   ⛔ REFUSING: parser generation failed."
    tail -5 "$WORK/gen.log"
    exit 1
fi
echo "   generated/ebnf.rs sha256: $(shasum -a 256 generated/ebnf.rs | cut -d' ' -f1)"
if ! (cd rust && cargo build --features "generated_parsers ebnf_dual_run" \
        --bin ebnf_dual_run_diff) >"$WORK/build.log" 2>&1; then
    echo "   ⛔ REFUSING: could not rebuild ebnf_dual_run_diff against the new artifact."
    grep -E '^error' "$WORK/build.log" | head -5
    exit 1
fi
echo

# probe NAME EXPECTED_VERDICT DESCRIPTION <<< grammar text on stdin
probe() {
    local name="$1" expected="$2" desc="$3"
    cat > "$WORK/$name.ebnf"
    "$DUAL_RUN" --input "$WORK/$name.ebnf" --output "$WORK/$name.json" >/dev/null 2>&1
    local actual
    actual=$(python3 -c "
import json,sys
try:
    d=json.load(open('$WORK/$name.json'))
    print('ACCEPT' if d['parse_full']['ok'] else 'REJECT')
except Exception:
    print('ERROR')
")
    if [ "$actual" = "$expected" ]; then
        printf '   ✅ %-18s %-6s  %s\n' "$name" "$actual" "$desc"
    else
        printf '   ⛔ %-18s %-6s  (declared %s)  %s\n' "$name" "$actual" "$expected" "$desc"
        note_divergence "$name: declared $expected, measured $actual"
    fi
}

echo "   CONTROLS (ground truth — a miss means the instrument is lying):"
probe quoted_ok ACCEPT "POSITIVE control: a legal single-quoted string still parses" <<'EOF'
r = 'abc'
esc := /(a)/
EOF
probe dq_unterminated REJECT "NEGATIVE control: the already-correct double-quoted twin still rejects" <<'EOF'
r = "a
esc := /(a)/
EOF
echo
echo "   THE DEFECT (these three ACCEPTED before the fix):"
probe sq_unterminated REJECT "an unterminated single quote must not parse" <<'EOF'
r = 'a
esc := /(a)/
EOF
probe sq_swallows_garbage REJECT "a closed quote followed by non-EBNF text must not parse" <<'EOF'
r = 'a' ]]]
esc := /(a)/
EOF
probe sq_swallows_block REJECT "an odd \`'\` must not swallow across a NEWLINE" <<'EOF'
r = '"' | "b"
]]]
esc := /(a)/
EOF
echo
echo "   UNAFFECTED SURFACE (regression pins):"
probe sq_in_dq ACCEPT "a single quote INSIDE a double-quoted string is untouched" <<'EOF'
r = "'" | "b"
esc := /(a)/
EOF
probe dq_in_sq ACCEPT "a double quote INSIDE a single-quoted string is untouched" <<'EOF'
r = '"' | "b"
esc := /(a)/
EOF
probe single_line_annotation ACCEPT "a single-line annotation payload is untouched" <<'EOF'
@semantic_value: {type: "escape", pattern: $1}
esc := /(a)/
EOF
probe rust_code_annotation ACCEPT "a Rust-code \`@transform\` payload is untouched" <<'EOF'
@transform: str::parse::<usize>().unwrap_or(0)
num := /(\d+)/
EOF
echo
echo "   ⭐ CLOSED BY .10.2 — these two probes are the proof that \`semantic_annotation\` is now"
echo "   a REAL rule rather than codegen's synthesized \`@\`-to-END-OF-LINE fallback. Both were"
echo "   declared the other way round while the fallback was in force; .10.2 flipped them, and"
echo "   they are kept as the standing signal that the delegation is doing the work:"
probe native_slurp_swallows_at_text REJECT "\`@@@\` is no longer a well-formed annotation (the native slurp is GONE)" <<'EOF'
r = 'a' @@@
esc := /(a)/
EOF
probe multiline_payload ACCEPT "a multi-line brace payload now PARSES (the .10.5 self-hosting gap, closed)" <<'EOF'
@dispatch_table: {
    "x": "y"
}
esc := /(a)/
EOF
echo
echo "   ⚠️ TWO PROBES WERE CONFOUNDED ALONG THE WAY, both recorded rather than quietly fixed."
echo "   (1) The original garbage probe was \`r = 'a' THIS IS GARBAGE @@@ !!! (((\`, which still"
echo "   ACCEPTed after the .10.5 fix — not the quote's fault: \`THIS IS GARBAGE\` are legal"
echo "   non-terminal references and \`@@@ …\` was eaten by the native slurp. It now uses"
echo "   \`]]]\`, which no EBNF construct can claim."
echo "   (2) \`sq_swallows_block\` used a multi-line annotation block as its non-parsing tail."
echo "   .10.2 made such a block LEGAL, so the probe could no longer tell a swallow from a"
echo "   successful parse. It now uses \`]]]\` across a newline, which tests the quote alone."
echo

# ─────────────────────────────────────────────────────────────────────────────
echo "ARM C — corpus: the tracked-grammar acceptance map (the regression oracle)"
echo "-------------------------------------------------------------------------------"
for g in grammars/*.ebnf grammars/scratch/*.ebnf; do
    "$DUAL_RUN" --input "$g" --output "$WORK/corpus.json" >/dev/null 2>&1
    verdict=$(python3 -c "
import json
try:
    d=json.load(open('$WORK/corpus.json'))
    print('ACCEPT' if d['parse_full']['ok'] else str(d.get('unconsumed_start')))
except Exception:
    print('ERROR')
")
    if [ "$verdict" = "ACCEPT" ] || [ "$verdict" = "ERROR" ]; then
        printf '   %-46s %s\n' "$(basename "$g")" "$verdict"
    else
        # Map the byte offset to a source line so a REJECT explains itself.
        line=$(head -c "$verdict" "$g" | wc -l | tr -d ' ')
        printf '   %-46s REJECT @byte %s (line %s: %s)\n' \
            "$(basename "$g")" "$verdict" "$((line + 1))" \
            "$(sed -n "$((line + 1))p" "$g" | cut -c1-40)"
    fi
done
echo
echo "==============================================================================="
if [ "$divergences" -eq 0 ]; then
    echo "RESULT: exit 0, 0 divergences — every declared verdict reproduced."
else
    echo "RESULT: $divergences DIVERGENCE(S) — declared verdicts did NOT reproduce."
fi
echo "==============================================================================="
exit "$divergences"
