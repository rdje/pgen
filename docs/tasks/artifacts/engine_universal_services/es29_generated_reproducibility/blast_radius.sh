#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.29 acceptance (a) — REPAIR the annotation pair, and MEASURE the blast
# radius in the same act.
#
# THE DEFECT. `generated/return_annotation_parser.rs` and `generated/semantic_annotation_parser.rs`
# carry a line — `self.coverage_deltas.clear();` — that the tracked code generator cannot emit
# (`grep -c` over `ast_pipeline/ast_based_generator.rs` -> 0, `git log -S` -> no commit ever) and
# whose absence that generator's own source comment deliberately defends. They were emitted from an
# uncommitted editor state ~80 minutes before the commit that finalised the emission. See
# `../es19_residual_attribution/reproducibility_probe.sh`, which reports it in ~3 s.
#
# WHY IT NEEDS MEASURING AND NOT JUST FIXING. Those two artifacts are the pair `rust/src/lib.rs`
# includes by literal path and the annotation backend links — i.e. the pair that participates in
# generating EVERY other parser. The leaf argues their divergence cannot affect a family artifact,
# because the divergent line sits inside `enable_coverage()` and touches only `coverage_deltas`,
# which nothing on the parse path reads. ⛔ That is an ARGUMENT, not a byte comparison. This script
# turns it into one: it snapshots every family artifact, repairs the pair through the CANONICAL make
# target, relinks `ast_pipeline` against the repaired pair, regenerates all eight, and demands
# byte-identity. If any differs, the argument is wrong and THAT is the finding.
#
# ⭐ EVERY COMPARISON USES MAKE'S OWN `-o` SPELLING. A generated parser embeds its output path once
# per rule-entry site (TOOLBOX 5.6), so regeneration into a scratch filename would differ in size for
# reasons having nothing to do with this repair. The regenerations go into a mimic tree
# `<work>/root/{generated,rust}` entered from `<work>/root/rust`, making the emitted string
# byte-identical to `../generated/<fam>_parser.rs` — and the script ASSERTS the site counts match
# rather than trusting that they do.
#
# HOW:  bash docs/tasks/artifacts/engine_universal_services/es29_generated_reproducibility/blast_radius.sh --repair
#       (without --repair it prints the plan and exits 0 without touching anything)
# COST: ~5 min — the annotation pair + an `ast_pipeline` relink + 8 codegen runs.
# EXIT: 0 = repaired and every family artifact byte-identical · 1 = a family artifact MOVED
#       · 2 = the run could not complete (missing input, build failure, unusable comparison)

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT"

WORK="$ROOT/rust/target/es29_blast_radius"
BASE="$WORK/baseline"
PIPELINE="$ROOT/rust/target/debug/ast_pipeline"

# The 8 non-annotation generated parsers: GENERATED_PARSER_FAMILIES (7) + the scratch slot.
FAMILIES=(json regex systemverilog systemverilog_preprocessor vhdl rtl_const_expr rtl_frontend scratch)
PAIR=(return_annotation semantic_annotation)

moved=0
pass() { printf '  ✓ %s\n' "$1"; }
fail() { printf '  ✗ %s\n' "$1" >&2; moved=$((moved + 1)); }
die()  { printf 'es29: %s\n' "$1" >&2; exit 2; }

sha() { shasum -a 256 "$1" | cut -d' ' -f1; }

printf '\nES29-BLAST-RADIUS: repair the annotation pair, prove the families did not move\n'
printf 'HEAD: %s\n\n' "$(LC_ALL=en_US.UTF-8 bash -c 's=$(git log -1 --format="%h %s"); [ ${#s} -gt 90 ] && printf "%s…" "${s:0:90}" || printf "%s" "$s"')"

if [ "${1:-}" != "--repair" ]; then
  printf 'DRY RUN (pass --repair to execute). The plan:\n'
  printf '  1. snapshot sha256 of %d family artifacts + %d annotation artifacts\n' "${#FAMILIES[@]}" "${#PAIR[@]}"
  printf '  2. make -C rust annotation_parsers          # the CANONICAL repair, one home for the recipe\n'
  printf '  3. re-run the reproducibility probe         # the pair must now re-derive from HEAD\n'
  printf '  4. rebuild ast_pipeline (dual features)     # so it LINKS the repaired pair\n'
  printf '  5. regenerate all %d families into a mimic tree at make'"'"'s -o spelling\n' "${#FAMILIES[@]}"
  printf '  6. demand byte-identity against the step-1 snapshot\n'
  exit 0
fi

# ── 1. snapshot ───────────────────────────────────────────────────────────────────────────────────
rm -rf "$WORK"; mkdir -p "$BASE" "$WORK/root/generated" "$WORK/root/rust"
printf 'STEP 1  snapshot\n'
for fam in "${FAMILIES[@]}" "${PAIR[@]}"; do
  f="$ROOT/generated/${fam}_parser.rs"
  [ -f "$f" ] || die "generated/${fam}_parser.rs is absent — regenerate generated/ before repairing it"
  [ -f "$ROOT/generated/${fam}.json" ] || die "generated/${fam}.json is absent"
  sha "$f" > "$BASE/${fam}.sha"
done
printf '  %d artifacts snapshotted\n' $(( ${#FAMILIES[@]} + ${#PAIR[@]} ))

# ── 2. the repair, through the canonical target ───────────────────────────────────────────────────
printf 'STEP 2  repair (make -C rust annotation_parsers)\n'
make -C rust SHELL=/bin/bash annotation_parsers >"$WORK/repair.log" 2>&1 \
  || { tail -20 "$WORK/repair.log" >&2; die "the canonical repair target FAILED — see $WORK/repair.log"; }
for fam in "${PAIR[@]}"; do
  before=$(cat "$BASE/${fam}.sha"); after=$(sha "$ROOT/generated/${fam}_parser.rs")
  if [ "$before" = "$after" ]; then
    printf '  = %-22s unchanged by the repair (%s…)\n' "$fam" "${after:0:12}"
  else
    printf '  ~ %-22s REPAIRED %s… -> %s…\n' "$fam" "${before:0:12}" "${after:0:12}"
  fi
done

# ── 3. the pair must now re-derive from HEAD ──────────────────────────────────────────────────────
printf 'STEP 3  the pair re-derives from HEAD\n'
if bash "$ROOT/docs/tasks/artifacts/engine_universal_services/es19_residual_attribution/reproducibility_probe.sh" \
     >"$WORK/repro.log" 2>&1; then
  pass "reproducibility probe GREEN — generated/ now matches what HEAD produces"
else
  fail "reproducibility probe still RED after the repair — see $WORK/repro.log"
  sed -n '4,20p' "$WORK/repro.log" | sed 's/^/      /' >&2
fi

# ── 4. relink ast_pipeline against the repaired pair ──────────────────────────────────────────────
printf 'STEP 4  relink ast_pipeline\n'
( cd "$ROOT/rust" && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline ) \
  >"$WORK/build.log" 2>&1 || { tail -20 "$WORK/build.log" >&2; die "ast_pipeline does not build against the repaired pair"; }
surface=$("$PIPELINE" --report-feature-surface 2>&1)
case "$surface" in
  *"ebnf_dual_run=true generated_parsers=true"*) pass "relinked, dual-feature ($surface)" ;;
  *) die "the relinked binary is under-featured: $surface" ;;
esac

# ── 5+6. regenerate every family and demand byte-identity ─────────────────────────────────────────
printf 'STEP 5  regenerate %d families at make'"'"'s -o spelling and compare\n' "${#FAMILIES[@]}"
for fam in "${FAMILIES[@]}"; do
  out="../generated/${fam}_parser.rs"
  ( cd "$WORK/root/rust" && "$PIPELINE" "$ROOT/generated/${fam}.json" \
      --generate-parser --eliminate-left-recursion -o "$out" ) >/dev/null 2>&1 \
    || die "codegen FAILED for $fam"

  fresh="$WORK/root/generated/${fam}_parser.rs"
  live_sites=$(grep -oF "$out" "$ROOT/generated/${fam}_parser.rs" | wc -l | tr -d ' ')
  fresh_sites=$(grep -oF "$out" "$fresh" | wc -l | tr -d ' ')
  [ "$live_sites" = "$fresh_sites" ] && [ "$live_sites" != 0 ] \
    || die "cannot compare $fam — embedded -o sites live=$live_sites fresh=$fresh_sites (TOOLBOX 5.6)"

  before=$(cat "$BASE/${fam}.sha"); after=$(sha "$fresh")
  if [ "$before" = "$after" ]; then
    pass "$(printf '%-28s byte-identical across the repair (%s sites)' "$fam" "$live_sites")"
  else
    fail "$(printf '%-28s MOVED: %s… -> %s…' "$fam" "${before:0:12}" "${after:0:12}")"
    printf '      the leaf'"'"'s bound is WRONG — the annotation pair DOES reach family codegen\n' >&2
    diff -u0 "$ROOT/generated/${fam}_parser.rs" "$fresh" | sed -n '3,15p' | sed 's/^/      /' >&2
  fi
  rm -f "$fresh"
done

printf '\n'
if [ "$moved" -eq 0 ]; then
  printf 'ES29-BLAST-RADIUS: annotation pair REPAIRED; all %d family artifacts byte-identical\n' "${#FAMILIES[@]}"
else
  printf 'ES29-BLAST-RADIUS: %d check(s) FAILED\n' "$moved" >&2
fi
exit $(( moved > 0 ? 1 : 0 ))
