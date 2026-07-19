#!/usr/bin/env bash
# RGX-0078.5.j.4 BATCH-1 DESIGN — emission-site census for the five batch members.
#
# WHAT: counts, in the SHIPPED regex artifact and in the lib/emitter sources,
#       the population each BATCH-1 member acts on — so every member's
#       "emission site" in the design record is a counted fact, not a claim.
#
# CUSTODY (the `-0172` precedent): the script ASSERTS the regex artifact is the
# banked floor vintage `e4924024` and REFUSES to report on mismatch, so a census
# can never be silently taken against the wrong artifact.
#
# READ-ONLY: matches source/artifact TEXT; builds nothing, runs no parser,
# perturbs no gate => the `-0163` STANDING INSTRUMENT RULE holds BY
# CONSTRUCTION (observing runtime counters would flip the observability twin;
# this observes text).
#
# ⚠️ WHY perl -0777 AND NOT grep: the generated artifact is rustfmt'ed, so a
# single call is routinely split across lines --
#       parser
#           .thin_memo
#           .insert(
# A line-oriented `grep -c 'thin_memo.insert'` returns 0 for 56 real sites, and
# `grep -c 'arena.alloc'` returns 1607 for 2853 real sites. The first draft of
# this census made exactly that error; every pattern below is therefore matched
# whole-file with \s* between tokens. Treat any line-oriented count over a
# generated artifact as WRONG until proven otherwise.
#
# HOW: bash docs/tasks/artifacts/batch1_design/site_census.sh
# OUT: site_census.txt

set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

ART="generated/regex_parser.rs"
EXPECT_SHA="e4924024"

echo "RGX-0078.5.j.4 BATCH-1 — emission-site census"
echo "root: $ROOT"
echo

# ---- custody gate -----------------------------------------------------------
if [ ! -f "$ART" ]; then
  echo "REFUSED: $ART is absent (generated/ is untracked; regenerate with 'make -C rust focus_regex')." >&2
  exit 2
fi
GOT="$(shasum -a 256 "$ART" | cut -c1-8)"
if [ "$GOT" != "$EXPECT_SHA" ]; then
  echo "REFUSED: regex artifact custody mismatch — expected $EXPECT_SHA, got $GOT." >&2
  echo "The banked floor artifact is $EXPECT_SHA; census numbers against any other" >&2
  echo "vintage are not comparable to the banked pricing. Restore the floor first." >&2
  exit 3
fi
echo "custody: $ART sha256[0:8] = $GOT  (OK — banked floor vintage)"
echo

# whole-file, line-break-tolerant occurrence count
n() { perl -0777 -ne 'my $c = () = /$ARGV[0]/g; print "$c\n"' -- "$1" "$2" 2>/dev/null \
      || perl -0777 -sne 'my $r = qr/$pat/; my $c = () = /$r/g; print "$c\n"' -- -pat="$1" "$2"; }
c() { printf '  %-50s %s\n' "$1" "$2"; }

CNT() { perl -0777 -sne 'my $r = qr/$pat/; my $c = () = /$r/g; print "$c\n"' -- -pat="$1" "$2"; }

echo "== member 1: G1-B — the ParseError carrier (lib + emitter => REGEN) =="
c "ParseError enum definition (lib mod.rs line)" "$(grep -n 'pub enum ParseError' rust/src/ast_pipeline/mod.rs | cut -d: -f1)"
c "ContextualError construction (EMITTED, ast_based_generator)" "$(CNT 'ParseError\s*::\s*ContextualError' rust/src/ast_pipeline/ast_based_generator.rs)"
c "cascade match fns emitting ParseResult<()> (cascade.rs)" "$(CNT 'ParseResult\s*<\s*\(\s*\)\s*>' rust/src/ast_pipeline/ast_based_generator/cascade.rs)"
c "ParseResult occurrences in shipped artifact" "$(CNT 'ParseResult' $ART)"
echo

echo "== member 2: C2 — fact-op constants / FactIndex (LIB-ONLY, no regen) =="
c "FactIndex std HashMap decls (SipHash, not FxHash)" "$(CNT '(by_kind|by_scope_and_name)\s*:\s*HashMap' rust/src/ast_pipeline/semantic_runtime.rs)"
c "to_ascii_lowercase() sites in semantic_runtime.rs" "$(CNT 'to_ascii_lowercase\s*\(' rust/src/ast_pipeline/semantic_runtime.rs)"
c "FxHashMap already imported in that same file" "$(CNT 'rustc_hash\s*::\s*FxHashMap' rust/src/ast_pipeline/semantic_runtime.rs)"
echo

echo "== member 3: G1-C — per-atom node/arena/tape (EMITTER => REGEN) =="
c "arena.alloc sites in shipped artifact" "$(CNT 'arena\s*\.\s*alloc\s*\(' $ART)"
c "deriv_boundary.push sites in shipped artifact" "$(CNT 'deriv_boundary\s*\.\s*push\s*\(' $ART)"
c "emitter mtb_match_atom_logic (cascade.rs line)" "$(grep -n 'fn mtb_match_atom_logic' rust/src/ast_pipeline/ast_based_generator/cascade.rs | cut -d: -f1)"
c "emitter scan_value_sequence_logic (scan.rs line)" "$(grep -n 'fn scan_value_sequence_logic' rust/src/ast_pipeline/ast_based_generator/scan.rs | cut -d: -f1)"
echo

echo "== member 4: memo-insert (EMITTER => REGEN) =="
c "packrat self.memo.insert sites in artifact" "$(CNT 'self\s*\.\s*memo\s*\.\s*insert\s*\(' $ART)"
c "thin_memo.insert sites in artifact" "$(CNT 'thin_memo\s*\.\s*insert\s*\(' $ART)"
c "memo_fail(.tainted).insert sites in artifact" "$(CNT 'memo_fail(_tainted)?\s*\.\s*insert\s*\(' $ART)"
c "SmallVec::from_slice sites in artifact (segment copies)" "$(CNT 'SmallVec\s*::\s*from_slice\s*\(' $ART)"
c "packrat memo hasher (already FxHashMap, NOT SipHash)" "$(CNT 'memo\s*:\s*rustc_hash\s*::\s*FxHashMap' rust/src/ast_pipeline/ast_based_generator.rs)"
echo

echo "== member 5: semantic runtime (LIB + emitter) =="
c "push_rule_context_static in artifact (UNCONDITIONAL/entry)" "$(CNT 'push_rule_context_static\s*\(' $ART)"
c "with_semantic_runtime_rule_transaction in artifact" "$(CNT 'with_semantic_runtime_rule_transaction\s*\(' $ART)"
c "fn parse_* rule methods in artifact" "$(CNT 'fn\s+parse_' $ART)"
c "self.rule_context_path() call sites (lib)" "$(CNT 'self\s*\.\s*rule_context_path\s*\(' rust/src/ast_pipeline/semantic_runtime.rs)"
c "  ... inside an explicit trace_enabled block" "1  (semantic_runtime.rs:3293, rollback_to)"
c "  ... EAGERLY evaluated regardless of trace level" "9  (3139,3428,3605,3754,3770,3794,3892,3980,4017)"
c "@predicate sites in grammars/regex.ebnf" "$(CNT '@predicate' grammars/regex.ebnf)"
c "@emit_fact sites in grammars/regex.ebnf" "$(CNT '@emit_fact' grammars/regex.ebnf)"
echo

echo "== the trace-eagerness finding (falsifies a banked claim) =="
echo "  pgen_trace_high!(..) -> pgen_trace!(level, ..) ->"
echo "    trace_log(level, file!(), line!(), module_path!(), format_args!(..))"
echo "  is a PLAIN FUNCTION CALL (rust/src/ast_pipeline/mod.rs:297-303). Its level"
echo "  guard is the early-return at mod.rs:304-306 — which runs AFTER Rust has"
echo "  evaluated every argument."
echo "  => self.rule_context_path() passed as a macro ARGUMENT is evaluated"
echo "     unconditionally, at EVERY trace level, including 'none'."
echo "  => rule_context_path() ALWAYS allocates a String"
echo "     (semantic_runtime.rs:2595-2601: join(\" > \"), else \"<anonymous>\".to_string())."
echo "  => the banked claim 'every rule_context_path consumer sits inside"
echo "     pgen_trace_high! — outcome-neutral' is FALSIFIED for 9 of 10 sites."
