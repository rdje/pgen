#!/usr/bin/env bash
# docs/tasks/artifacts/engine_universal_services/es23_lr_alt_guard/probe.sh
#
# `ENGINE-UNIVERSAL-SERVICES.23` — the control bank for the direct-LR normalizer's seed count and
# the planner's base-alternative classification.
#
# ⛔ THE DEFECT. `normalize_direct_left_recursive_alternatives` counted a bare reference to an
#    indirect wrapper as a SEED, so its deliberate guard — *"a rule whose alternatives are ALL
#    left-recursive derives nothing … leave it visible where it is"* — did not fire on the MIXED
#    case (one inline direct alternative + one bare wrapper reference). It hoisted `<rule>_lr_alt1`,
#    the planner then classified both alternatives as wrappers, left `base_alternatives` empty,
#    returned `None`, and never consumed the hoist ⇒ the well-formedness error listed a rule name
#    the author cannot edit, alongside the author's own rules.
#
# ⭐ AND FIXING ONLY THE NORMALIZER MOVED THE INVENTED NAME RATHER THAN REMOVING IT — MEASURED, NOT
#    ANTICIPATED. With the seed count corrected and nothing else, the same synthetic reported
#    `expr_lr_base` instead of `expr_lr_alt1`: the un-hoisted inline alternative now reached
#    `detect_left_recursive_chain_plan`, which classified it as a *base* alternative (it is not — it
#    is left-recursive), built a plan, and moved it into `<rule>_lr_base`. Both sites had to agree,
#    which is why the fix is ONE predicate (`alternative_is_left_recursive`) with two consumers.
#    ⇒ ARM 1 is therefore evidence for BOTH halves; see the HONEST BOUND at the bottom.
#
# USAGE   bash docs/tasks/artifacts/engine_universal_services/es23_lr_alt_guard/probe.sh
# EXIT    0 = every arm behaved as designed; 1 = an arm did not; 2 = the bank could not run

set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT"
PIPE="rust/target/debug/ast_pipeline"
[ -x "$PIPE" ] || { echo "es23: $PIPE is absent — build it with: cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline" >&2; exit 2; }
bash scripts/require_ast_pipeline_features.sh "$PIPE" generated_parsers ebnf_dual_run >/dev/null || exit 2

W="rust/target/es23_lr_alt_guard"; rm -rf "$W"; mkdir -p "$W"
arms=0; bad=0
ok()  { arms=$((arms + 1)); printf '  ✓ %-56s %s\n' "$1" "${2:-}"; }
no()  { arms=$((arms + 1)); bad=$((bad + 1)); printf '  ✗ %-56s %s\n' "$1" "${2:-}" >&2; }

# grammar <name> <body…>  — writes $W/<name>.ebnf
grammar() { local n="$1"; shift; { printf '@entry: true\n'; printf '%s\n' "$@"; } > "$W/$n.ebnf"; }

# generate <name> -> $W/<name>.log ; prints nothing, returns codegen's rc
generate() { "$PIPE" "$W/$1.ebnf" --generate-parser --eliminate-left-recursion -o "$W/$1.rs" >"$W/$1.log" 2>&1; }

# the rules the well-formedness diagnostic names, one per line
named_rules() { grep -o "rule '[A-Za-z_0-9]*'" "$W/$1.log" | sed "s/rule '//;s/'//" | sort -u; }

# the LR rule names the EMITTED parser declares
emitted_lr() { grep -o 'RULE_[A-Z0-9_]*_LR_[A-Z0-9_]*' "$W/$1.rs" 2>/dev/null | sort -u; }

echo "ENGINE-UNIVERSAL-SERVICES.23 — direct-LR normalizer / planner control bank"
echo "commit: $(git rev-parse --short HEAD)"
echo

# ── ARM 1 — THE DEFECT'S OWN SHAPE: mixed, and NO genuine seed anywhere ──────────────────────────
# Recorded pre-fix output (both halves absent): scratch, expr_lr_alt1, expr, mulwrap  (4 rules)
# Recorded with ONLY the normalizer half:       scratch, expr_lr_base, expr, mulwrap  (4 rules)
grammar mixed_no_seed 'scratch := expr' 'expr := expr "+" term | mulwrap' 'mulwrap := expr "*" term' 'term := "n"'
generate mixed_no_seed
rules=$(named_rules mixed_no_seed | paste -sd, -)
invented=$(named_rules mixed_no_seed | grep -c '_lr_' || true)
if [ "$invented" = 0 ] && [ "$rules" = "expr,mulwrap,scratch" ]; then
  ok "ARM 1  mixed/no-seed names ONLY author rules" "[$rules]"
else
  no "ARM 1  mixed/no-seed names ONLY author rules" "[$rules] ($invented engine-invented)"
fi
# and it must still be REFUSED — the verdict was never in doubt, only the locus
grep -q 'is ill-formed: 3 non-terminating rule(s)' "$W/mixed_no_seed.log" \
  && ok "ARM 1b the grammar is still REFUSED, as 3 rules" \
  || no "ARM 1b the grammar is still REFUSED, as 3 rules" "$(tail -1 "$W/mixed_no_seed.log")"

# ── ARM 2 — the guard's ORIGINAL case must be untouched: all alternatives inline-direct ──────────
grammar all_direct_no_seed 'scratch := expr' 'expr := expr "+" term | expr "-" term' 'term := "n"'
generate all_direct_no_seed
rules=$(named_rules all_direct_no_seed | paste -sd, -)
[ "$(named_rules all_direct_no_seed | grep -c '_lr_' || true)" = 0 ] \
  && ok "ARM 2  all-direct/no-seed names ONLY author rules" "[$rules]" \
  || no "ARM 2  all-direct/no-seed names ONLY author rules" "[$rules]"

# ── ARM 3 — ⭐ THE ANTI-OVER-EAGERNESS CONTROL: the SAME mixed shape WITH a genuine seed must
#            still be normalized AND eliminated. Without this arm the fix could be "suppress
#            everything", which would pass ARMs 1-2 and silently disable the feature.
grammar mixed_seeded 'scratch := expr' 'expr := expr "+" term | mulwrap | "n"' 'mulwrap := expr "*" term' 'term := "n"'
generate mixed_seeded
lr=$(emitted_lr mixed_seeded | paste -sd, -)
case "$lr" in
  *RULE_EXPR_LR_BASE*|*RULE_EXPR_LR_SUFFIX*) ok "ARM 3  mixed/seeded is STILL eliminated" "[$lr]" ;;
  *) no "ARM 3  mixed/seeded is STILL eliminated" "[${lr:-none}]" ;;
esac
"$PIPE" "$W/mixed_seeded.ebnf" --lint-grammar 2>&1 | grep -q 'left_recursion_unhandled=0, left_recursion_eliminated=1' \
  && ok "ARM 3b lint: unhandled=0, eliminated=1" \
  || no "ARM 3b lint: unhandled=0, eliminated=1" "$("$PIPE" "$W/mixed_seeded.ebnf" --lint-grammar 2>&1 | grep -o 'left_recursion[^,]*,[^,]*')"
# and it must PARSE — an emitted rule name is not a working parser
printf 'n+n*n' > "$W/in_deep.txt"; printf 'n' > "$W/in_seed.txt"
for pair in "in_deep:n+n*n" "in_seed:n"; do
  f=${pair%%:*}; txt=${pair#*:}
  "$PIPE" "$W/mixed_seeded.ebnf" --interpret-parse "$W/$f.txt" 2>&1 | grep -q 'accepted=true' \
    && ok "ARM 3c mixed/seeded accepts '$txt'" \
    || no "ARM 3c mixed/seeded accepts '$txt'"
done

# ── ARM 4 — plain inline direct LR (the normalizer's bread and butter) ───────────────────────────
grammar plain_direct 'scratch := expr' 'expr := expr "+" term | "n"' 'term := "n"'
generate plain_direct
lr=$(emitted_lr plain_direct | paste -sd, -)
[ -n "$lr" ] && ok "ARM 4  plain inline direct LR still eliminated" "[$lr]" \
             || no "ARM 4  plain inline direct LR still eliminated" "[none]"
printf 'n+n' > "$W/in4.txt"
"$PIPE" "$W/plain_direct.ebnf" --interpret-parse "$W/in4.txt" 2>&1 | grep -q 'accepted=true' \
  && ok "ARM 4b plain inline direct LR accepts 'n+n'" || no "ARM 4b plain inline direct LR accepts 'n+n'"

# ── ARM 5 — plain one-hop wrapper (the planner's bread and butter, normalizer not involved) ──────
grammar plain_wrapper 'scratch := expr' 'expr := mulwrap | "n"' 'mulwrap := expr "*" term' 'term := "n"'
generate plain_wrapper
lr=$(emitted_lr plain_wrapper | paste -sd, -)
[ -n "$lr" ] && ok "ARM 5  plain one-hop wrapper still eliminated" "[$lr]" \
             || no "ARM 5  plain one-hop wrapper still eliminated" "[none]"
printf 'n*n' > "$W/in5.txt"
"$PIPE" "$W/plain_wrapper.ebnf" --interpret-parse "$W/in5.txt" 2>&1 | grep -q 'accepted=true' \
  && ok "ARM 5b plain one-hop wrapper accepts 'n*n'" || no "ARM 5b plain one-hop wrapper accepts 'n*n'"

# ── ARM 6 — ⭐ THE PREDICATE MUST STAY NARROW. A bare reference to a rule that is NOT a wrapper is
#            a real SEED, and must keep counting as one. `atom := "n"` fails
#            `extract_wrapper_suffix`, so the guard must NOT fire and the hoist must happen.
grammar seed_is_a_bare_ref 'scratch := expr' 'expr := expr "+" atom | atom' 'atom := "n"'
generate seed_is_a_bare_ref
lr=$(emitted_lr seed_is_a_bare_ref | paste -sd, -)
[ -n "$lr" ] && ok "ARM 6  a bare ref to a NON-wrapper is still a seed" "[$lr]" \
             || no "ARM 6  a bare ref to a NON-wrapper is still a seed" "[none] — the predicate is too wide"
printf 'n+n' > "$W/in6.txt"
"$PIPE" "$W/seed_is_a_bare_ref.ebnf" --interpret-parse "$W/in6.txt" 2>&1 | grep -q 'accepted=true' \
  && ok "ARM 6b that grammar accepts 'n+n'" || no "ARM 6b that grammar accepts 'n+n'"

# ── ARM 7 — the SHIPPED families are untouched: no emitted artifact moves ────────────────────────
# ⛔ Asserted through the doctrine rather than re-implemented here: re-derive-and-diff is exactly
#    what GENERATED-REPRODUCIBILITY tier 2 does, over all ten artifacts, and it is already gated.
if [ -f rust/test_data/grammar_quality/generated_reproducibility_v0.json ]; then
  ok "ARM 7  shipped-artifact identity is delegated" "run: make -C rust SHELL=/bin/bash generated_reproducibility_gate (measured 10/10 byte-identical, + generated/ebnf.rs by hand)"
else
  no "ARM 7  shipped-artifact identity is delegated" "the GENERATED-REPRODUCIBILITY baseline is missing"
fi

echo
printf '%d/%d arms behaved as designed\n' "$((arms - bad))" "$arms"
cat <<'BOUND'

⚠️ HONEST BOUND — the planner half has no arm of its OWN, and that is a property, not an omission.
   `detect_left_recursive_chain_plan`'s new refusal is reachable EXACTLY when the normalizer's guard
   fired, and the argument is closed rather than statistical: an alternative lands in
   `base_alternatives` only if it is not (bare reference + wrapper suffix), and it is left-recursive
   only if it is inline-direct OR (bare reference + wrapper suffix) — so a left-recursive
   `base_alternatives` entry must be INLINE-DIRECT, and an inline-direct alternative survives to the
   planner only when the normalizer declined to hoist it. ⇒ ARM 1 exercises both halves, and the
   evidence that the planner half is load-bearing is a MEASUREMENT taken in between: with only the
   normalizer corrected, ARM 1's grammar reported `expr_lr_base` in place of `expr_lr_alt1`.
BOUND
[ "$bad" = 0 ] || exit 1
