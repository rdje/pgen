#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.31 (b) + (c) — WHAT DOES CLASS L COST, AND DOES HOISTING IT LOSE
# ANYTHING?
#
# `.25` slice 1 split the emitted `-o` path into two classes. Class **D** — `let filename_str =
# "<path>";`, 2 704 sites, read by nothing — was deleted by `.31` slice 1. Class **L** is the other
# 60 482: the `file` argument of every `Logger::log_*` call the emitter writes. Those are LIVE (the
# trace prints them), so they cannot be deleted — only HOISTED, to one module constant referenced
# by name.
#
# `.31` acceptance (b) demands the price be MEASURED and forbids deriving it from
# `sites × Δlen`, on the stated grounds that `prettyplease` re-wraps lines when a 38-character
# literal becomes a short identifier. ⭐ MEASURED, IT DOES NOT — the residual in all eleven rows is
# exactly the length of the constant's own declaration line, with nothing left over (ARM F). The
# prohibition was still right: that is a *result*, and it took two arms to learn it.
#
# THE TWO ARMS
#   ARM 2  SHIPPED — `HEAD`'s emitter. The path is emitted once as
#          `const PGEN_SOURCE_LABEL: &str = "<path>";` and referenced by name at every log site.
#   ARM 1  UN-HOISTED — `HEAD` + the tracked `unhoist.patch`, which restores the per-site literal.
#          This is the emission every artifact carried up to `PGEN-ENGINE-UNIVERSAL-SERVICES-0069`.
#          ⛔ It is a MEASUREMENT arm. Nothing builds a deliverable from it and no `make` target
#          passes it; it exists because the "before" column disappears the moment the hoist ships.
#
# ⛔ THE MIMIC TREE IS LOAD-BEARING, NOT TIDINESS (TOOLBOX 5.6). A generated parser embeds its `-o`
#    path, so re-deriving to a scratch filename changes the artifact's SIZE for reasons unrelated to
#    the emitter. Both arms are generated from `<work>/root/rust` with `-o ../generated/…`, which is
#    byte-identical to what `rust/Makefile` passes — and ARM A then PROVES it by demanding ARM 2
#    reproduce the shipped tree exactly, rather than assuming the mimicry worked.
#
# ⛔ THE ENGINE IS RESTORED BY AN EXIT TRAP whatever happens, and the tree's own `ast_pipeline` /
#    `ast_pipeline_bootstrap` are REBUILT from the restored source before this script exits — an
#    A/B that leaves a patched binary at the canonical path is the `#140`-class trap (TOOLBOX 1.4).
#
# HOW:   bash docs/tasks/artifacts/engine_universal_services/es31_label_hoist/probe.sh
#        PGEN_ES31_KEEP=1 …    keep the 22 generated parsers (~470 MB) instead of deleting them
# COST:  4 debug builds + 22 codegen runs ≈ 20 min, ~1.5 GB peak under rust/target/.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../../.." && pwd)"; cd "$ROOT"

PATCH="$HERE/unhoist.patch"
RECON="$HERE/reconstruct.py"
CMP="$ROOT/scripts/compare_generated_parsers.py"
EMITTER="rust/src/ast_pipeline/ast_based_generator.rs"
WORK="rust/target/es31_label_hoist"
IDENT_LEN=17                       # len("PGEN_SOURCE_LABEL")

# The eight families `ast_pipeline` generates, plus `ebnf` (same binary, --bootstrap-mode) and the
# annotation PAIR (`ast_pipeline_bootstrap`, --bootstrap-mode). Mirrors rust/Makefile's own split;
# ARM A's byte-identity against the shipped tree is what proves the mirror has not drifted.
FAMILIES=(json regex systemverilog systemverilog_preprocessor vhdl rtl_const_expr rtl_frontend scratch)
PAIR=(return_annotation semantic_annotation)

fails=0; arms=0
pass() { arms=$((arms+1)); printf '  ✓ %s\n' "$1"; }
fail() { arms=$((arms+1)); fails=$((fails+1)); printf '  ✗ %s\n' "$1" >&2; }

restore() {
  git checkout -- "$EMITTER" 2>/dev/null
  if ! git diff --quiet -- "$EMITTER"; then
    printf '⛔⛔ THE EMITTER IS STILL PATCHED: %s — restore it by hand before committing.\n' "$EMITTER" >&2
  fi
  printf 'restoring the tree'"'"'s own binaries from the restored source…\n'
  ( cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline ) >/dev/null 2>&1 \
    || printf '⛔ could not rebuild ast_pipeline — rebuild it by hand.\n' >&2
  ( cd rust && cargo build --bin ast_pipeline_bootstrap --no-default-features --features bootstrap ) >/dev/null 2>&1 \
    || printf '⛔ could not rebuild ast_pipeline_bootstrap — rebuild it by hand.\n' >&2
}
trap restore EXIT

for f in "$PATCH" "$RECON" "$CMP"; do
  [ -f "$f" ] || { printf 'es31-hoist-probe: missing %s\n' "$f" >&2; exit 2; }
done
git diff --quiet -- rust/src/ \
  || { printf 'es31-hoist-probe: refusing — rust/src/ is already dirty; this script patches the emitter and must start clean.\n' >&2; exit 2; }
ls generated/*_parser.rs >/dev/null 2>&1 \
  || { printf 'es31-hoist-probe: refusing — generated/ holds no parser, so ARM A has nothing to compare against.\n' >&2; exit 2; }

# ⛔⛔ ERA-PINNED SINCE `ENGINE-UNIVERSAL-SERVICES.31` (e). This bank measures the price of HOISTING
# the `-o` path from one literal per logging site to one module constant. (e) then changed what that
# constant HOLDS — the path was the wrong label (it named the generated parser beside an INPUT byte
# offset), so it is now `"<grammar> input byte"` and **no generated parser embeds its output path at
# all**. Consequences, both fatal to this bank and neither a defect:
#   - `unhoist.patch` no longer applies (the emitter moved);
#   - the arithmetic in ARM B/F is a function of the PATH's length, which is no longer emitted;
#   - `--sites` correctly reports 0, so ARM D's "one per artifact" no longer describes anything.
# ⭐ It REFUSES rather than failing arm by arm, because a bank whose subject has been removed on
# purpose must say so — reds that read as regressions are exactly the trap
# `a-bank-pinned-to-the-shipped-behaviour-is-pinned-to-a-moving-target` records. The measurement of
# record is `price.md` (the full per-artifact table) and `probe.txt` (the 9/9 run that produced it).
if ! grep -q 'const #source_label: &str = #filename;' "$EMITTER"; then
  printf 'ES31-LABEL-HOIST: ERA-PINNED — this tree is post-`.31`(e), where the emitted label is no\n'
  printf '  longer the `-o` path and no artifact embeds one. This bank measured the HOIST (slice 2)\n'
  printf '  and cannot run here; its result is recorded in price.md and probe.txt beside this file.\n'
  printf '  Refusing rather than reporting arms red against a correct tree.\n'
  exit 0
fi

rm -rf "$WORK"; mkdir -p "$WORK/root/generated" "$WORK/root/rust" "$WORK/bin"

printf '\nES31-LABEL-HOIST: what does class L cost, and does hoisting it lose anything?\n'
printf 'commit : %s\n\n' "$(git log -1 --format=%h)"

# ── build the two arms ────────────────────────────────────────────────────────────────────────────
build_arm() {                       # build_arm <suffix>
  ( cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline ) \
      >"$WORK/build_$1.log" 2>&1 || { printf 'es31-hoist-probe: ast_pipeline build FAILED (%s) — see %s\n' "$1" "$WORK/build_$1.log" >&2; exit 2; }
  cp rust/target/debug/ast_pipeline "$WORK/bin/pipeline_$1"
  ( cd rust && cargo build --bin ast_pipeline_bootstrap --no-default-features --features bootstrap ) \
      >>"$WORK/build_$1.log" 2>&1 || { printf 'es31-hoist-probe: ast_pipeline_bootstrap build FAILED (%s) — see %s\n' "$1" "$WORK/build_$1.log" >&2; exit 2; }
  cp rust/target/debug/ast_pipeline_bootstrap "$WORK/bin/boot_$1"
}

printf 'building ARM 2 (SHIPPED, hoisted) from HEAD…\n'
build_arm arm2
printf 'applying unhoist.patch and building ARM 1 (per-site literals)…\n'
git apply "$PATCH" || { printf 'es31-hoist-probe: unhoist.patch does not apply to this emitter — it has moved; regenerate the lever.\n' >&2; exit 2; }
build_arm arm1
git checkout -- "$EMITTER"
git diff --quiet -- "$EMITTER" || { printf 'es31-hoist-probe: the emitter did not restore — refusing to continue.\n' >&2; exit 2; }

# ── generate both arms into the mimic tree ────────────────────────────────────────────────────────
gen() {                             # gen <arm> <tool> <family> <artifact-basename> <extra-flag…>
  local arm="$1" tool="$2" fam="$3" base="$4"; shift 4
  ( cd "$WORK/root/rust" && "$ROOT/$WORK/bin/$tool" "$ROOT/generated/${fam}.json" \
        --generate-parser "$@" --eliminate-left-recursion -o "../generated/${base}" ) \
      >"$WORK/gen_${arm}_${fam}.log" 2>&1 \
    || { printf 'es31-hoist-probe: codegen FAILED (%s %s) — see %s\n' "$arm" "$fam" "$WORK/gen_${arm}_${fam}.log" >&2; exit 2; }
  mv "$WORK/root/generated/${base}" "$WORK/${arm}_${fam}.rs"
}

for arm in arm2 arm1; do
  printf 'generating %s …\n' "$arm"
  for fam in "${FAMILIES[@]}"; do gen "$arm" "pipeline_$arm" "$fam" "${fam}_parser.rs"; done
  gen "$arm" "pipeline_$arm" ebnf ebnf.rs --bootstrap-mode
  for fam in "${PAIR[@]}"; do gen "$arm" "boot_$arm" "$fam" "${fam}_parser.rs" --bootstrap-mode; done
done

ALL=("${FAMILIES[@]}" ebnf "${PAIR[@]}")
live_of() { [ "$1" = ebnf ] && printf 'generated/ebnf.rs' || printf 'generated/%s_parser.rs' "$1"; }
sha()     { shasum -a 256 "$1" | cut -d' ' -f1; }
size()    { wc -c < "$1" | tr -d ' '; }

# ── ARM A — ARM 2 IS WHAT SHIPS (so the mimicry is proven, not assumed) ───────────────────────────
printf '\nARM A  the mimic tree reproduces the shipped tree\n'
same=0; diffs=0
for fam in "${ALL[@]}"; do
  if [ "$(sha "$WORK/arm2_${fam}.rs")" = "$(sha "$(live_of "$fam")")" ]; then same=$((same+1)); else
    diffs=$((diffs+1)); printf '      %s: ARM 2 != the shipped artifact\n' "$fam" >&2; fi
done
[ "$diffs" = 0 ] \
  && pass "all $same artifacts: ARM 2 is byte-identical to generated/ — the A/B measures what ships" \
  || fail "$diffs artifact(s) differ from the shipped tree — regenerate generated/ before trusting any number below"

# ── ARM B — THE PRICE ─────────────────────────────────────────────────────────────────────────────
printf '\nARM B  the price (acceptance (b))\n'
printf '  %-30s %14s %14s %11s %9s %9s\n' family 'ARM1 bytes' 'ARM2 bytes' delta 'A1 sites' 'A2 sites'
t1=0; t2=0; s1=0; s2=0; residual_bad=0; rows=""
for fam in "${ALL[@]}"; do
  a="$WORK/arm1_${fam}.rs"; b="$WORK/arm2_${fam}.rs"
  ba=$(size "$a"); bb=$(size "$b")
  na=$(python3 "$CMP" --sites "$a") || exit 2
  nb=$(python3 "$CMP" --sites "$b") || exit 2
  lit=$(python3 "$CMP" --spelling "$a")
  d=$((bb - ba))
  # ARM F's per-row identity: the whole delta is the substitution plus the declaration line.
  #   decl = `const PGEN_SOURCE_LABEL: &str = "<path>";\n` = 33 + len(path) + 2 + 1
  predicted=$(( -na * ((${#lit} + 2) - IDENT_LEN) + 33 + ${#lit} + 3 ))
  [ "$d" = "$predicted" ] || { residual_bad=$((residual_bad+1)); rows="${rows}      ${fam}: measured ${d}, substitution+declaration predicts ${predicted}\n"; }
  printf '  %-30s %14s %14s %11s %9s %9s\n' "$fam" "$ba" "$bb" "$d" "$na" "$nb"
  t1=$((t1+ba)); t2=$((t2+bb)); s1=$((s1+na)); s2=$((s2+nb))
done
printf '  %-30s %14s %14s %11s %9s %9s\n' TOTAL "$t1" "$t2" "$((t2-t1))" "$s1" "$s2"
[ "$((t2-t1))" -lt 0 ] \
  && pass "class L costs $(( t1 - t2 )) bytes across the eleven artifacts ($(awk -v a="$t1" -v b="$t2" 'BEGIN{printf "%.2f", (a-b)*100/a}')% of the tree)" \
  || fail "the hoist did not shrink the artifacts (total delta $((t2-t1)))"

# ── ARM C — THE LOAD-BEARING ONE: the change is EXACTLY the substitution ──────────────────────────
printf '\nARM C  reconstruction identity — ARM 2 minus the constant, identifier→literal, IS ARM 1\n'
ident=0; bad=0
for fam in "${ALL[@]}"; do
  if python3 "$RECON" --arm1 "$WORK/arm1_${fam}.rs" --arm2 "$WORK/arm2_${fam}.rs" >/dev/null 2>&1; then
    ident=$((ident+1))
  else
    bad=$((bad+1)); printf '      %s: reconstruction differs from ARM 1\n' "$fam" >&2
  fi
done
[ "$bad" = 0 ] \
  && pass "all $ident artifacts reconstruct ARM 1 byte-identically ⇒ nothing but the substitution moved, and every \`file\` argument still receives the same string (acceptance (c), source level)" \
  || fail "$bad artifact(s) changed by more than the substitution"

# ── ARM D — the path is embedded ONCE per artifact now ────────────────────────────────────────────
printf '\nARM D  embedded -o sites\n'
[ "$s2" = "${#ALL[@]}" ] \
  && pass "ARM 2 embeds the -o path exactly ${#ALL[@]} times — once per artifact — where ARM 1 embeds it $s1 times" \
  || fail "ARM 2 embeds the -o path $s2 times, expected ${#ALL[@]} (one per artifact)"
pass "⇒ a one-character difference in the -o spelling now moves an artifact by 1 byte, not by $(( s1 / ${#ALL[@]} )) on average (TOOLBOX 5.6's trap, reduced by $(( s1 / ${#ALL[@]} ))×)"

# ── ARM E — RED CONTROL: the reconstruction must be able to FAIL ─────────────────────────────────
# A control never observed failing is not known to work (`docs/CLAIM_VERIFICATION.md` §3 leg 2).
printf '\nARM E  RED control — ARM C must be able to fail\n'
if python3 "$RECON" --arm1 "$WORK/arm1_json.rs" --arm2 "$WORK/arm2_json.rs" --wrong-literal >/dev/null 2>&1; then
  fail "reconstructing with a DELIBERATELY WRONG literal still matched — ARM C proves nothing"
else
  pass "reconstructing with a wrong literal is refused (RED as designed)"
fi
if python3 "$RECON" --arm1 "$WORK/arm1_json.rs" --arm2 "$WORK/arm1_json.rs" >/dev/null 2>&1; then
  fail "an artifact with NO declaration was accepted — the refusal path is dead"
else
  pass "an artifact carrying no \`const PGEN_SOURCE_LABEL\` is REFUSED (exit 2), not compared"
fi

# ── ARM F — the delta is the substitution plus the declaration, with NO remainder ─────────────────
printf '\nARM F  where the bytes went (acceptance (b): measured, then explained)\n'
if [ "$residual_bad" = 0 ]; then
  pass "all ${#ALL[@]} rows: delta == −sites×(len(literal)−len(ident)) + len(declaration), exactly"
  pass "⇒ prettyplease did NOT re-wrap — every log site already had its own line, so shortening one argument moved no line breaks"
else
  fail "$residual_bad row(s) carry bytes the substitution does not explain:"
  printf "$rows" >&2
fi

printf '\n'
if [ "$fails" -eq 0 ]; then
  printf 'ES31-LABEL-HOIST: %d/%d arms as declared\n' "$arms" "$arms"
else
  printf 'ES31-LABEL-HOIST: %d of %d ARMS FAILED\n' "$fails" "$arms" >&2
fi

if [ "${PGEN_ES31_KEEP:-0}" != "1" ]; then
  rm -f "$WORK"/*.rs
  printf '(the 22 generated parsers were deleted; PGEN_ES31_KEEP=1 keeps them)\n'
fi
exit "$fails"
