#!/usr/bin/env bash
# probe.sh — SV-CORPUS-GRAD.13c.2x.9
#
# Does `known_unscoped_property_identifier` — SystemVerilog's LAST union-residual `UNKNOWN` —
# COMMIT on a given carrier, and if not, WHY not? Four independent arms per carrier.
#
# ⛔⛔ THE FIRST VERSION OF THIS SCRIPT MEASURED NOTHING, AND ITS SEVEN ZEROES WERE A STUCK NEEDLE.
# `-0272` scored each carrier with `grep -c known_unscoped_property_identifier <ast.json>`. PGEN's
# AST is ANNOTATION-SHAPED: nodes carry the `kind:`/field names a `->` return annotation writes,
# never rule names. `known_unscoped_property_identifier` carries NO return annotation, so its name
# can never appear in any dump — that column was structurally incapable of reporting a commit.
# Measured on `c9`, a dump that DOES contain a committed instance of the rule:
#   property_identifier=0  declaration_identifier=0  property_instance=0
#   prop_primary_sv_2017=0 hierarchical_property_identifier=0
# ⇒ the old "7 carriers, 0 commits" table was seven readings of an instrument pinned at 0, and the
# unreachability reading it invited is REFUTED by every row below.
#
# ARM 0 — COMMITTED COUNT, and it is the arm that settles the leaf (TOOLBOX 3.5).
#   `--dump-rule-outcome-counts-json` writes `rule_entry_counts` and `rule_committed_counts` under
#   the transactional coverage stack — the SAME stack certificate-coverage measures under. Its
#   committed semantics are C3-B: **tournament winners AND successful-but-losing branches both
#   survive**. One command answers, by rule NAME, what seven hand-written carriers and a stuck grep
#   could not. ⛔ It existed the whole time.
# ARM 1 — AST SHAPE: what the WINNING tree carries (a strictly narrower question than ARM 0).
#   `prop_primary_*`'s `property_instance` branch emits `{kind:"instance", body:{name,args}}`;
#   `seq_unary`'s `sequence_instance` branch emits `{kind:"instance", body:{…}, abbrev:…}` — the
#   extra `abbrev` key keeps the property and sequence families from ever being confused. Inside the
#   property node, `ps_or_hierarchical_property_identifier -> {body: $1}` makes its three
#   alternatives shape-distinct:
#     known_unscoped -> {"body": {"body": "<id>"}}                       <- THE TARGET
#     hierarchical   -> {"body": {"root":…, "scope":…, "name":…}}
#     scoped         -> {"body": {"scope":…, "name":…}}
# ARM 2 — TRACER: did the rule's own post-predicate pass at a USE site, in any explored branch?
#   `--trace-rules known_unscoped_property_identifier` prints every `has_fact(...)` verdict with its
#   caller stack; a `→ true` whose stack ENDS in `known_unscoped_property_identifier` is a
#   name-keyed positive that cannot be confused with the sequence family or the declaration site.
# ARM 3 — TOURNAMENT: `--trace-rules prop_primary_sv_2017`, read the way TOOLBOX Protocol D step 2
#   says to read it — codegen's own `🏁 Rule 'R' selected branch N/M consuming K chars
#   (priority=…, branch_policy=longest_match)` line, reported here as `sel=<branch>@<chars>`,
#   alongside the per-branch furthest successful extents `b1`/`b26` that make a TIE visible AS a tie. ⛔ PGEN does NOT compile alternatives to PEG first-match, and TOOLBOX.md says in
#   bold never to answer a selection question from the grammar text. The generated code runs a
#   27-branch LONGEST-MATCH TOURNAMENT (`best_end`/`best_priority`; `should_take` is
#   `candidate_end > best_end`, tie-broken by an explicit priority and then, because the comparison
#   is a STRICT `>`, by declaration order). Branch 1 is `sequence_expr`; branch 26 is
#   `property_instance`. This arm prints the extents that decided it.
#
# ⭐ WHAT THE FOUR ARMS TOGETHER SHOW, and no single arm could:
#   c1…c8   branch 26 succeeds to the SAME extent as branch 1 ⇒ it loses the strict-`>` tie to
#           declaration order and the winning tree keeps the `sequence` shape — yet the rule is
#           still COMMITTED under C3-B, so an ordinary two-line carrier already satisfies the
#           certificate's own committed semantics.
#   c9      an argument only `property_actual_arg -> property_expr` can parse (`not x`) makes
#           branch 1 SHORTER than branch 26 ⇒ branch 26 wins on length and the rule lands in the
#           winning tree as well.
#   r1/r2   two-sided controls at the SAME syntax: an UNDECLARED name (predicate false) and a DOTTED
#           name both fall to `hierarchical_property_identifier` and commit 0 — so ARM 0 and ARM 1
#           are proven able to say NO, not merely to say YES.
#   generated/wellformed
#           the shapes the reach PLANNER actually emits: `branch-not-reached`. The use site is a
#           `bind`/checker port connection, never a `prop_primary_sv_2017` position, so branch 26 is
#           never even attempted. ⇒ the union `UNKNOWN` is a PLANNER-CARRIER gap, not a grammar one.
#
#   VERDICTS: COMMITS:in-tree (committed AND the winning tree carries it) · COMMITS:tie-lost
#           (committed under C3-B, tree keeps the sequence shape) · entered-not-committed (the
#           predicate passed in some explored branch yet nothing survived — NOT exercised by this
#           population, kept because ARM 0 and ARM 2 can legitimately disagree that way) ·
#           predicate-false (branch 26 ran, the `has_fact` gate said no) ·
#           branch-not-reached (the use site is not a `prop_primary_sv_2017` position at all).
#
# ⛔ WHAT THIS STILL DOES NOT PROVE: that the reach PLANNER can produce any of these carriers.
# Making the certificate WITNESS the rule is the owning leaf's `(c)`.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT"
PROBE="$ROOT/rust/target/release/parseability_probe"
[[ -x "$PROBE" ]] || { echo "error: build it: make -C rust SHELL=/bin/bash focus_systemverilog && cargo build --release --features generated_parsers --bin parseability_probe" >&2; exit 1; }
HERE="$(dirname "${BASH_SOURCE[0]}")"
OUT="$ROOT/rust/target/kupi_diag"; mkdir -p "$OUT"

# The binary must embed the parser on disk, or every row below is about a different grammar.
disk="$(shasum -a 256 "$ROOT/generated/systemverilog_parser.rs" | cut -d' ' -f1)"
emb="$("$PROBE" --parser-fingerprint | python3 -c 'import json,sys; print(json.load(sys.stdin)["parsers"]["systemverilog"])')"
[[ "$disk" == "$emb" ]] || { echo "error: probe embeds systemverilog $emb but disk carries $disk — rebuild before trusting any row" >&2; exit 1; }
echo "# parser sha256 (binary == disk): $disk"

printf '%-11s %-7s %-11s %-15s %-6s %-31s %s\n' carrier parses entry/comm ast_route pred tournament verdict
for f in "$HERE"/*.sv; do
    b="$(basename "$f" .sv)"
    p="$("$PROBE" --parse systemverilog "$f" --profile sv_2017 2>&1 | grep -c passed)"

    "$PROBE" --parse systemverilog "$f" --profile sv_2017 \
        --dump-rule-outcome-counts-json "$OUT/$b.outcome.json" >/dev/null 2>&1
    oc="$(python3 "$HERE/kupi_arms.py" outcome "$OUT/$b.outcome.json")"

    "$PROBE" --parse-dump-ast-pretty systemverilog "$f" "$OUT/$b.pretty.json" --profile sv_2017 >/dev/null 2>&1
    route="$(python3 "$HERE/kupi_arms.py" route "$OUT/$b.pretty.json")"

    PGEN_TRACE_VERBOSITY=debug "$PROBE" --parse systemverilog "$f" --profile sv_2017 \
        --trace-rules known_unscoped_property_identifier \
        --trace-log-file "$OUT/$b.kupi.log" >/dev/null 2>&1
    pred="$(grep -c '→ true caller=.*> known_unscoped_property_identifier$' "$OUT/$b.kupi.log" 2>/dev/null)"; pred="${pred:-0}"

    PGEN_TRACE_VERBOSITY=debug "$PROBE" --parse systemverilog "$f" --profile sv_2017 \
        --trace-rules prop_primary_sv_2017 \
        --trace-log-file "$OUT/$b.tour.log" >/dev/null 2>&1
    tour="$(python3 "$HERE/kupi_arms.py" tournament "$OUT/$b.tour.log")"

    comm="${oc#*/}"; b26="${tour##*b26=}"
    if   [[ "$comm" != "-" && "$comm" -gt 0 && "$route" == *known_unscoped* ]]; then v="COMMITS:in-tree"
    elif [[ "$comm" != "-" && "$comm" -gt 0 ]];                                 then v="COMMITS:tie-lost"
    elif [[ "$pred" -gt 0 ]];                                                   then v="entered-not-committed"
    elif [[ "$b26" != "-" ]];                                                   then v="predicate-false"
    else                                                                             v="branch-not-reached"; fi
    printf '%-11s %-7s %-11s %-15s %-6s %-31s %s\n' "$b" "$p" "$oc" "$route" "$pred" "$tour" "$v"
done
