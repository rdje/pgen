#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.46 (b) — SIZE THE POPULATION BEFORE DECIDING WHETHER THE ARM BLOCKS.
#
# WHY THIS FILE EXISTS
# --------------------
# `.46`'s routing sized the live SystemVerilog population BY HAND at 3 sites. A hand count is a
# claim about a 1 537-rule grammar, so it is re-derived here by the engine's own traversal — the
# same `detect_profile_gated_lookaheads` the lint runs — over EVERY tracked grammar, not just SV.
#
# ⭐⭐ WHAT THIS CENSUS IS FOR CHANGED WHEN `.46` (c) LANDED, AND THE NUMBER DID NOT MOVE. It was
# built to size a DEFECT and to decide whether a blocking lint arm was affordable. The repair fixed
# the SEMANTICS rather than the spelling, so the population stayed at 3 while the defect went to 0 —
# which is the answer to the blocking question, not an obstacle to it: there is nothing left to
# block. The census now measures a documentation surface, and its self-check is what keeps it honest.
# ⭐ The re-derivation CONFIRMED the 3 and found a FOURTH the hand count missed: a POSITIVE
# `&scope_resolution` in `class_scope_type`, nested at `root/s2/q/s3` — under a quantifier, which is
# exactly where an eye scanning top-level sequences stops looking.
#
# ⛔ A GRAMMAR THAT CANNOT BE LOADED IS REPORTED AS `not-measured`, NEVER AS ZERO. Four tracked
# `.ebnf` files under `grammars/` are raw LRM EXTRACTS / fragments that the frontend refuses (an
# unterminated regex literal, an unterminated quoted literal, no declared entry rule). They are
# source material, not shipped parsers. Counting them 0 would inflate the clean population with
# files nothing measured — the failure-in-the-passing-direction this repository keeps finding.
#
# Usage: bash docs/tasks/artifacts/engine_universal_services/profile_gate_neg_lookahead/census.sh

set -uo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT_DIR" || exit 2

PIPE="rust/target/debug/ast_pipeline"
if [[ ! -x "$PIPE" ]]; then
    printf 'REFUSED: %s is absent — build it first:\n' "$PIPE" >&2
    printf '  cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline\n' >&2
    exit 2
fi

# ⭐⭐ SELF-CHECK FIRST — A CENSUS WHOSE INSTRUMENT IS BLIND REPORTS A CLEAN POPULATION.
# The control is a five-rule reduction of `grammars/regex.ebnf`'s shape: `@default_profile: pcre2`
# plus a rule gated to `["relaxed"]`, so the ONLY route to its defect is a profile that appears in no
# `@profiles` list. Two real defects were caught by this arm, both of which made the census print a
# confident zero: the lint derived its profile context from the FILTERED grammar (whose load-time
# filter had already deleted the gated rule AND its annotation — the lint printed `profiles=[]`), and
# the universe omitted `@default_profile`. Until both were fixed, regex measured 0 over 95 `!` sites
# without the shipping profile ever being tested.
CONTROL="docs/tasks/artifacts/engine_universal_services/profile_gate_neg_lookahead/default_profile_universe_control.ebnf"
ctl_neg="$("$PIPE" "$CONTROL" --lint-grammar 2>&1 \
    | grep -o 'profile_gated_negative_lookaheads=[0-9]*' | grep -o '[0-9]*$' | head -1)"
if [[ "${ctl_neg:-0}" -lt 1 ]]; then
    printf 'REFUSED: the @default_profile control reported %s findings, expected >= 1.\n' "${ctl_neg:-<none>}" >&2
    printf 'The instrument cannot see a rule gated out of a profile that appears in no @profiles\n' >&2
    printf 'list, so every zero below would be a blind spot rather than a clean grammar.\n' >&2
    exit 1
fi
printf 'SELF-CHECK: the @default_profile control reports %s finding(s) — the instrument can see the\n' "$ctl_neg"
printf '            shipping-profile case, so a zero below is a measurement.\n\n'

printf '%-48s %-10s %-10s %s\n' "grammar" "negative" "positive" "status"
printf '%-48s %-10s %-10s %s\n' "------------------------------------------------" "--------" "--------" "------"

total_neg=0; total_pos=0; measured=0; not_measured=0
for g in grammars/*.ebnf grammars/scratch/scratch.ebnf; do
    out="$("$PIPE" "$g" --lint-grammar 2>&1)"
    neg="$(printf '%s' "$out" | grep -o 'profile_gated_negative_lookaheads=[0-9]*' | grep -o '[0-9]*$' | head -1)"
    pos="$(printf '%s' "$out" | grep -o 'profile_gated_positive_lookaheads=[0-9]*' | grep -o '[0-9]*$' | head -1)"
    if [[ -z "$neg" || -z "$pos" ]]; then
        reason="$(printf '%s' "$out" | head -1 | cut -c1-60)"
        printf '%-48s %-10s %-10s not-measured (%s…)\n' "$(basename "$g")" "-" "-" "$reason"
        not_measured=$((not_measured + 1))
        continue
    fi
    printf '%-48s %-10s %-10s measured\n' "$(basename "$g")" "$neg" "$pos"
    total_neg=$((total_neg + neg)); total_pos=$((total_pos + pos)); measured=$((measured + 1))
done

echo
printf 'POPULATION: negative=%d (the SOUNDNESS INVERSION) positive=%d (monotonic note) over %d measured grammar(s); %d NOT MEASURED\n' \
    "$total_neg" "$total_pos" "$measured" "$not_measured"
echo
echo "⭐ THESE ARE NOTES, NOT DEFECTS — AND THE COUNT DID NOT FALL WHEN THE DEFECT WAS FIXED."
echo "   .46 (c) repaired the SEMANTICS, not the spelling. At a site whose guard is VACUOUS under"
echo "   some profile, the lookahead body is now evaluated with @profiles gating IGNORED under"
echo "   exactly that profile, so the guard survives and the construct is monotonic. The sites are"
echo "   still counted because the reading is non-obvious — a reader seeing !X with X gated out will"
echo "   assume the guard vanishes, and it does not."
echo
echo "⛔ THE CONDITION IS SATISFIABILITY, NOT REACHABILITY, AND THAT DISTINCTION IS THE WHOLE ARM."
echo "   A gate NARROWS (the rule is absent and nothing replaces it — the guard is DELETED) or it"
echo "   SELECTS (sibling rules, one per dialect, behind a dispatcher — the guard is SWITCHED)."
echo "   SystemVerilog does both. Keying on \"the body reaches a gated rule\" flagged the SELECTION"
echo "   case too, and repairing THAT would have made !reserved_non_keyword_identifier see every"
echo "   dialect keyword list at once — rejecting \"class\" as an identifier under verilog_2005, a"
echo "   regression in the REJECTING direction introduced by a fix for one in the accepting one."
echo
echo "   ⇒ the BLOCKING half of (b) is DECLINED: there is nothing wrong here to block."
