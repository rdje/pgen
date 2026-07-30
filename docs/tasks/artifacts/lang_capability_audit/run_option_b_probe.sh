#!/usr/bin/env bash
# LANG-CAPABILITY-AUDIT.10.2 — OPTION-B feasibility probe.
#
# WHAT IT PROVES
#   Defining `semantic_annotation` LOCALLY in the meta-grammar — as a DELIMITER with an
#   opaque payload — meets both requirements the container actually has:
#     R1  every reference in `ebnf.ebnf` resolves, so `"semantic_annotation"` can leave
#         codegen's NATIVE_UNRESOLVED_REFERENCE_BUILTINS allowlist (unblocks .10.3);
#     R2  the meta-grammar recognizes every annotation form tracked grammars use,
#         INCLUDING the multi-line brace payload -> self-hosting returns to 12/12.
#   …without any cross-file composition, name collision, or contract movement.
#
# ⛔ ZERO REPO EDITS. Everything happens in an untracked scratch dir under `rust/target/`
#    (on the repository's own volume — a path on another volume cannot be relativized into
#    an `include!` and the build fails with a misleading E0432; see the KM card
#    `probe-a-metagrammar-change`).
#
# Deterministic: fixed inputs, no seeds, no clock.
#
# Usage:  bash docs/tasks/artifacts/lang_capability_audit/run_option_b_probe.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

AST_PIPELINE="./rust/target/debug/ast_pipeline"
DUAL_RUN="./rust/target/debug/ebnf_dual_run_diff"
FRAGMENT="docs/tasks/artifacts/lang_capability_audit/option_b_semantic_annotation_fragment.ebnf"
# The probe grammar MUST be named `ebnf.ebnf`: codegen derives the parser struct from the
# grammar name, and `rust/src/lib.rs` include!s it expecting `EbnfParser`.
P="rust/target/option_b_probe"

divergences=0
note_divergence() { divergences=$((divergences + 1)); printf '   ⛔ DIVERGENCE: %s\n' "$1"; }

# ⛔ Always put the toolbox binary back on the SHIPPED parser, on every exit path —
# otherwise `ebnf_dual_run_diff` silently keeps answering for the probe grammar.
restore_binary() {
    echo
    echo "Restoring ebnf_dual_run_diff to the SHIPPED generated/ebnf.rs..."
    (cd rust && cargo build --features "generated_parsers ebnf_dual_run" \
        --bin ebnf_dual_run_diff) >/dev/null 2>&1 \
        && echo "   ✅ restored" || echo "   ⛔ RESTORE FAILED — rebuild it by hand"
}
trap restore_binary EXIT

echo "==============================================================================="
echo "LANG-CAPABILITY-AUDIT.10.2 — OPTION-B feasibility probe"
echo "==============================================================================="
echo

echo "Building the probe meta-grammar = grammars/ebnf.ebnf + the option-B fragment..."
rm -rf "$P"; mkdir -p "$P"
cat grammars/ebnf.ebnf "$FRAGMENT" > "$P/ebnf.ebnf"
printf '   probe grammar: %s lines (%s + %s)\n' \
    "$(wc -l < "$P/ebnf.ebnf" | tr -d ' ')" \
    "$(wc -l < grammars/ebnf.ebnf | tr -d ' ')" \
    "$(wc -l < "$FRAGMENT" | tr -d ' ')"

if ! (cd rust && cargo build --features "generated_parsers ebnf_dual_run" \
        --bin ast_pipeline) >"$P/astbuild.log" 2>&1; then
    echo "   ⛔ REFUSING: could not build the dual-feature ast_pipeline."
    grep -E '^error' "$P/astbuild.log" | head -5
    exit 1
fi

echo
echo "R1 — does every reference now resolve, and is the grammar still well-formed?"
"$AST_PIPELINE" "$P/ebnf.ebnf" --lint-grammar 2>&1 | head -1 | sed 's/^/   /'
echo "   (baseline for comparison — the SHIPPED meta-grammar:)"
"$AST_PIPELINE" grammars/ebnf.ebnf --lint-grammar 2>&1 | head -1 | sed 's/^/   /'

echo
echo "Generating the probe parser (same two steps as the Makefile's one-time seed)..."
"$AST_PIPELINE" "$P/ebnf.ebnf" --emit-raw-ast-json "$P/ebnf.json" >"$P/genA.log" 2>&1 \
    || { echo "   ⛔ raw-AST emission failed"; tail -5 "$P/genA.log"; exit 1; }
"$AST_PIPELINE" --generate-parser --bootstrap-mode --debug --eliminate-left-recursion \
    "$P/ebnf.json" -o "$P/ebnf.rs" >"$P/genB.log" 2>&1 \
    || { echo "   ⛔ parser generation failed"; tail -5 "$P/genB.log"; exit 1; }
printf '   emitted struct: %s\n' "$(grep -o 'pub struct [A-Za-z]*Parser' "$P/ebnf.rs" | head -1)"

echo
echo "R1 (continued) — is \`semantic_annotation\` a REAL generated rule now, or still the"
echo "native \`@\`-to-end-of-line fallback codegen synthesizes for an UNRESOLVED reference?"
shipped_len=$(awk '/pub fn parse_semantic_annotation/{f=1} f{n++} f&&/^    }$/{print n; exit}' generated/ebnf.rs)
probe_len=$(awk '/pub fn parse_semantic_annotation/{f=1} f{n++} f&&/^    }$/{print n; exit}' "$P/ebnf.rs")
printf '   shipped  parse_semantic_annotation : %-5s lines  (synthesized native fallback)\n' "$shipped_len"
printf '   option-B parse_semantic_annotation : %-5s lines  (generated from the rule)\n' "$probe_len"
for r in annotation_key annotation_payload braced_payload brace_body brace_text line_payload; do
    n=$(grep -c "pub fn parse_$r\b" "$P/ebnf.rs")
    if [ "$n" -eq 1 ]; then printf '   ✅ parse_%-20s emitted\n' "$r"
    else printf '   ⛔ parse_%-20s MISSING\n' "$r"; note_divergence "helper rule $r not emitted"; fi
done

echo
echo "Building ebnf_dual_run_diff against the probe parser..."
if ! (cd rust && PGEN_EBNF_PARSER_PATH="target/option_b_probe/ebnf.rs" \
        cargo build --features "generated_parsers ebnf_dual_run" \
        --bin ebnf_dual_run_diff) >"$P/build.log" 2>&1; then
    echo "   ⛔ REFUSING: build failed."
    grep -E '^error' "$P/build.log" | head -5
    exit 1
fi

echo
echo "R2 — the tracked-grammar corpus under the OPTION-B meta-parser"
echo "-------------------------------------------------------------------------------"
accept=0; total=0
for g in grammars/*.ebnf grammars/scratch/*.ebnf; do
    base="$(basename "$g")"
    # The three raw IEEE-LRM extraction snapshots are traceability artifacts, not part of
    # the tracked self-hosting set; they do not parse today either.
    case "$base" in *_lrm_extracted.ebnf) continue ;; esac
    total=$((total + 1))
    "$DUAL_RUN" --input "$g" --output "$P/r.json" >/dev/null 2>&1
    v=$(python3 -c "
import json
try:
    d=json.load(open('$P/r.json'))
    print('ACCEPT' if d['parse_full']['ok'] else 'REJECT @'+str(d.get('unconsumed_start')))
except Exception:
    print('ERROR')
")
    if [ "$v" = "ACCEPT" ]; then accept=$((accept + 1)); printf '   ✅ %-46s %s\n' "$base" "$v"
    else printf '   ⛔ %-46s %s\n' "$base" "$v"; note_divergence "$base: $v"; fi
done
echo
printf '   SELF-HOSTING: %s/%s   (shipped meta-grammar today: regex.ebnf REJECTs at byte 55,925)\n' \
    "$accept" "$total"

echo
echo "==============================================================================="
if [ "$divergences" -eq 0 ]; then
    echo "RESULT: exit 0, 0 divergences — OPTION B meets R1 and R2."
else
    echo "RESULT: $divergences DIVERGENCE(S)."
fi
echo "==============================================================================="
exit "$divergences"
