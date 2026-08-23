#!/usr/bin/env bash
# GRAMMAR-WELLFORMED.H.16.2b — the introducer-PREFIXED terminal class, censused and probed.
#
# `consume_layout_for_terminal`'s dynamic backstop asks "is this terminal a comment introducer?"
# so that matching `#` / `//` / `/*` is not defeated by the engine skipping those very bytes as
# trivia. It used to ask by EXACT EQUALITY over six spellings, so a terminal that merely STARTS
# with an introducer -- `#{`, `##`, `/**`, `///` -- was not equal to any of them and kept comment
# skipping enabled in front of itself. `H.16.2b` replaced it with a prefix test.
#
# THREE ARMS:
#   A. CENSUS over the SHIPPED artifacts, not over grammar text. The population is the set of
#      string literals reaching `match_lit_ascii` / `match_string` (the only callers of
#      `consume_layout_for_terminal`), intersected with "starts with an introducer but is not one".
#      ⛔ Read alongside whether the parser emits the guard AT ALL: a grammar that claims every
#      introducer gets no comment arms and no `allow_comment_skip`, so a prefixed terminal there is
#      not exposed in either direction.
#   B. PER-SITE accept/reject probe. `H.16.1`'s census cleared 10 of its 11 sites by
#      arm-suppression and witness status rather than by an individual probe, and named that as a
#      bound it owed. This is that probe.
#   C. The behavioural site, both ways. Exactly ONE site in the repository both emits the guard and
#      is prefixed-but-not-exact: SystemVerilog's `##`. Its exposure is a comment sitting directly
#      in front of a `##`, which the prefix test stops skipping -- so it is probed explicitly, in
#      line-comment and block-comment form, against a no-comment control.
#
# Usage:  bash docs/tasks/artifacts/grammar_wellformed/comment_skip_prefix_guard/probe.sh
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT"
PIPELINE=rust/target/debug/ast_pipeline
bash scripts/require_ast_pipeline_features.sh "$PIPELINE" generated_parsers ebnf_dual_run || exit 2

echo "== ARM A — CENSUS over the shipped generated parsers =="
printf '%-40s %-14s %s\n' "artifact" "emits_guard" "introducer-PREFIXED terminals"
prefixed_total=0
for f in generated/*.rs; do
  [ -r "$f" ] || continue
  arm=$(grep -c 'let allow_comment_skip' "$f")
  # BOTH terminal call families: `match_lit_ascii` (the fast path codegen emits for a fixed
  # literal) and `match_string` (the general one it falls back to). They are the only callers of
  # `consume_layout_for_terminal`, so their union IS the population `expected` can ever hold.
  lits=$(grep -oE '(match_lit_ascii(_bare)?|match_string(_bare)?)\("((\\.|[^"\\])*)"' "$f" \
        | sed -E 's/^(match_lit_ascii(_bare)?|match_string(_bare)?)\("//; s/"$//' | sort -u \
        | grep -E '^(#|//|/\*)' | grep -vxE '#|//|/\*' | tr '\n' ' ')
  n=$(printf '%s' "$lits" | wc -w | tr -d ' ')
  prefixed_total=$((prefixed_total + n))
  printf '%-40s %-14s [%s]\n' "$(basename "$f")" "$arm" "$lits"
done
echo "H162B-CENSUS: prefixed_sites=$prefixed_total"

echo
echo "== ARM B — per-site accept probe (each site's own construct, through its own grammar) =="
probe() { # <label> <grammar> <text> [profile]
  local label="$1" g="$2" text="$3" prof="${4:-}"
  printf '%b' "$text" > rust/target/h162b_probe.txt
  local args=(--interpret-parse rust/target/h162b_probe.txt)
  [ -n "$prof" ] && args+=(--grammar-profile "$prof")
  local v
  v="$(timeout 300 "$PIPELINE" "grammars/$g.ebnf" "${args[@]}" 2>&1 \
       | grep -oE 'accepted=[a-z]+' || echo 'NO-VERDICT')"
  printf '  %-46s %s\n' "$label" "$v"
}
probe "ebnf '/**' (documentation_comment)"      ebnf '/** doc */\nX := "a"\n'
probe "ebnf '///' (documentation_comment)"      ebnf '/// doc\nX := "a"\n'
probe "semantic_annotation '#{' (set_value)"    semantic_annotation '@type: #{"a", "b"}'
probe "semantic_annotation '///' (doc_comment)" semantic_annotation '@type: "a"'
probe "regex '##' (callout_hash_payload)"       regex '(?C1)a' pcre2
probe "systemverilog '##' (cycle_delay)"        systemverilog 'module m; logic clk,a,b; sequence s; @(posedge clk) a ##1 b; endsequence endmodule\n' sv_2017

echo
echo "== ARM C — the ONE behavioural site: a comment directly in front of SV '##' =="
probe "SV ##  (control: no comment)"            systemverilog 'module m; logic clk,a,b; sequence s; @(posedge clk) a ##1 b; endsequence endmodule\n' sv_2017
probe "SV ##  after a // line comment"          systemverilog 'module m; logic clk,a,b; sequence s; @(posedge clk) a // c\n##1 b; endsequence endmodule\n' sv_2017
probe "SV ##  after a /* block comment */"      systemverilog 'module m; logic clk,a,b; sequence s; @(posedge clk) a /* c */ ##1 b; endsequence endmodule\n' sv_2017

echo
echo "⛔ A CLEAN RESULT HERE IS NOT EVIDENCE ON ITS OWN. The corpus-wide oracle for this guard is"
echo "   'make -C rust SHELL=/bin/bash parse_harness_equivalence_gate' with the change applied to"
echo "   ONE side only: any divergence it reports IS an input where the guard matters. That sweep"
echo "   was CLEAN for the prefix test and RED for a control that disables comment skipping"
echo "   outright (rtl_frontend 5, systemverilog_preprocessor 2, ebnf 3 divergences), which is what"
echo "   makes the clean reading a measurement rather than a blind spot."
