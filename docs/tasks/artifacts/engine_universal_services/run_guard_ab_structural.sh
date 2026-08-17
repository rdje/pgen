#!/usr/bin/env bash
# docs/tasks/artifacts/engine_universal_services/run_guard_ab_structural.sh
#
# ENGINE-UNIVERSAL-SERVICES.20 acceptance (b) — THE THREE-ARM A/B, STRUCTURAL TIER.
#
# `.20`(b) asks for the one measurement that separates *absorbing* the cast/call knot from *guarding*
# it. The published A/B has two arms and cannot answer it:
#
#   ARM 1  narrow admission (`--indirect-lr-admit-starvation-safe-only`, the pre-`.17`-slice-9
#          policy) — the knot is NOT absorbed and no guard is emitted.        `.17` s9: 303.0 s
#   ARM 2  SHIPPED — the knot is absorbed AND the call-site guards are emitted. `.17` s9: 376.7 s
#   ARM 3  ⭐ absorbed, guards SUPPRESSED — the missing arm. Everything ARM 2 does except emitting
#          the guard chains, so ARM 2 − ARM 3 is the guard's price and ARM 3 − ARM 1 is absorption's.
#
# ⛔ ARM 3 REQUIRES A PATCH, AND THE PATCH MUST NOT SHIP. `plan_guard_chains` is called
# unconditionally, and the call site's own comment argues against a switch: a second switch would
# have to agree with the admission criterion, and two things that must agree can drift. So the lever
# lives as a TRACKED PATCH (`guard_emission_suppressed.patch`) applied and reverted around one
# measurement — the same shape as `A2.5_direct_lr_normalization.patch` beside it. A parser built this
# way is DELIBERATELY UNSOUND on the sites the guard closes and is never a deliverable.
#
# ⛔ THE ENGINE IS RESTORED BY AN EXIT TRAP whatever happens, and the script asserts a clean
# `git diff` on `rust/src/` before it exits non-trivially.
#
# This is the STRUCTURAL tier — rules, bytes, LR names — which is cheap (two debug builds + three
# generations, ~4 min). The TIMED tier needs a release `parseability_probe` per arm (~22 min each,
# 12 GB peak) and is `.20`(b) slice 2.
#
# Usage: bash docs/tasks/artifacts/engine_universal_services/run_guard_ab_structural.sh
# Exit 0 iff all three arms generate and the engine is restored byte-exactly.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../.." && pwd)"; cd "$ROOT"

PATCH="$HERE/guard_emission_suppressed.patch"
ENGINE="rust/src/ast_pipeline/indirect_lr_elimination.rs"
OUT="rust/target/lr_ab_arms"
JSON="generated/systemverilog.json"
GEN="./rust/target/debug/ast_pipeline --generate-parser --eliminate-left-recursion"
BUILD='cd rust && cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline'

restore() {
  git checkout -- "$ENGINE" 2>/dev/null
  if ! git diff --quiet -- "$ENGINE"; then
    echo "⛔⛔ THE ENGINE IS STILL PATCHED: $ENGINE — restore it by hand before committing." >&2
  fi
}
trap restore EXIT

mkdir -p "$OUT"
if ! git diff --quiet -- rust/src/; then
  echo "refusing: rust/src/ is already dirty — this script patches the engine and must start clean." >&2
  exit 2
fi
[ -f "$JSON" ] || { echo "refusing: $JSON is missing — run 'make -C rust focus_systemverilog' first." >&2; exit 2; }

# ⛔⛔ RAW `stat` BYTES ARE NOT COMPARABLE ACROSS ARMS, AND READING THEM AS IF THEY WERE INVERTED THIS
# MEASUREMENT'S FIRST ANSWER. A generated parser embeds its own `-o` path as a diagnostic string —
# **36 346 times** in the SV parser as it stood when this runner was written — so one extra character
# in the output FILENAME added 36 346 bytes to the file.
# ⭐ SINCE `ENGINE-UNIVERSAL-SERVICES.31` SLICE 2 THAT COUNT IS **1**: the path is emitted once as
# `const PGEN_SOURCE_LABEL` and referenced by name. The normalisation below is therefore no longer
# load-bearing at six figures — but it is KEPT, because one site is still one byte per character and
# a helper that reads `path_sites` from the artifact costs nothing. ⛔ A re-run of this runner today
# reports `path_sites=1` on every arm; that is the hoist, not a broken instrument.
# Comparing ARM 3 against `generated/systemverilog_parser.rs` (whose
# embedded path is 12 characters shorter) made ARM 3 look 203 KB LARGER than the shipped parser; with
# the path normalised it is 233 KB SMALLER, which is the opposite conclusion about what guards cost.
# ⇒ every byte figure below is measured AFTER replacing the embedded path with a fixed token.
#
# ⛔⛔ THIS COMMENT SAID **43 615** UNTIL `ENGINE-UNIVERSAL-SERVICES.25` SLICE 1, AND THIS SCRIPT'S
# OWN OUTPUT REFUTED IT THE WHOLE TIME. `guard_ab_structural.txt` — written by this very runner, in
# the same commit — records `path_sites=33249 / 36346 / 36291` for arms 1/2/3. No arm reports 43 615.
# The two byte figures on the line above say the same thing independently (203 KB + 233 KB = 436 KB
# = 12 chars × 36 346 sites), as does re-running the ORIGINAL `row()` on the shipped parser rewritten
# to this script's own arm-2 spelling (`path_sites=36346`).
# ⛔ `.25` slice 1 ALSO published a mechanism for the wrong figure — *"36 346 × 42 ÷ 35, the right
# byte delta over the wrong character width"* — and that is RETRACTED under director challenge: the
# instrument never mis-computed, and the arithmetic was fitted (the identity needs a ratio of exactly
# 6/5; the real arm-2 numerator 48-5=43 implies a non-integer denominator 35.83). The honest class is
# **instrument right, prose copy wrong** (`DERIVED_STATE_CONTAINMENT.md` R1/R3) — a number that was
# CARRIED, not mis-derived.
# ⭐ Which is exactly why `row()` no longer derives the count itself and no reader should re-type it:
# the SHARED helper (`scripts/compare_generated_parsers.py`, `.25` (c)) reads the embedded spelling
# OUT OF THE ARTIFACT, so the count printed below is derived on every run.
#
# ⛔ `--token '<OUT>'` IS DELIBERATE AND MUST NOT BE "TIDIED" TO THE HELPER'S DEFAULT. The token's
# length enters every normalised byte count as `sites × Δlen`, and `.20` slice 3 PUBLISHED its
# three-arm table (ARM1 130 512 738 · ARM3 142 441 376 · ARM2 142 671 859) on `<OUT>`. Re-basing
# silently would move numbers already in the record; the helper takes the token as an argument so
# the choice is visible here instead.
CMP="$ROOT/scripts/compare_generated_parsers.py"
#
# ⛔ `normalised_chars`, NOT bytes. These artifacts carry emoji in their trace strings, so the two
# differ by 108 443 on the shipped SV parser — more than the whole figure some readings turn on.
# `.20` slice 3 published its table in CHARACTERS (Python `len(str)`); `wc -c` and `stat` are BYTES.
row() {  # row <label> <parser>
  local desc sites norm_bytes
  desc=$(python3 "$CMP" --describe "$2" --token '<OUT>') || {
    echo "row: the shared helper REFUSED to derive an embedded -o path from $2" >&2; return 2; }
  sites=$(printf '%s' "$desc"      | sed -n 's/.*"sites": \([0-9]*\).*/\1/p')
  norm_bytes=$(printf '%s' "$desc" | sed -n 's/.*"normalised_chars": \([0-9]*\).*/\1/p')
  python3 - "$1" "$2" "$sites" "$norm_bytes" <<'PY'
import re, sys
label, path, sites, norm_bytes = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
src = open(path, encoding="utf-8", errors="replace").read()
rules = re.search(r"RULE_COUNT: usize = (\d+)", src)
lr = {m for m in re.findall(r'"[a-z_0-9]*_lr_(?:base|suffix|seed|guard|alt)[a-z_0-9]*"', src)}
gd = {m for m in lr if "_lr_guard" in m}
print(f"{label:<46} norm_bytes={norm_bytes:<11} RULE_COUNT={rules.group(1) if rules else '?':<6} "
      f"lr_names={len(lr):<5} guard_names={len(gd):<3} path_sites={sites}")
PY
}

echo "=============================================================================="
echo "ENGINE-UNIVERSAL-SERVICES.20 (b) — three-arm A/B, STRUCTURAL tier"
echo "=============================================================================="

echo "--- ARM 1: narrow admission (existing flag, no patch) ------------------------"
$GEN --indirect-lr-admit-starvation-safe-only "$JSON" -o "$OUT/sv_arm1_narrow_parser.rs" >/dev/null 2>&1 \
  || { echo "arm 1 generation FAILED" >&2; exit 1; }

echo "--- ARM 3: absorbed, guard emission SUPPRESSED (patched engine) --------------"
git apply "$PATCH" || { echo "cannot apply $PATCH — has the engine moved?" >&2; exit 1; }
bash -c "$BUILD" >/dev/null 2>&1 || { echo "patched build FAILED" >&2; exit 1; }
$GEN "$JSON" -o "$OUT/sv_arm3_noguard_parser.rs" >/dev/null 2>&1 \
  || { echo "arm 3 generation FAILED" >&2; exit 1; }
restore
bash -c "$BUILD" >/dev/null 2>&1 || { echo "restored build FAILED" >&2; exit 1; }

echo "--- ARM 2: SHIPPED (the tracked engine, regenerated as a revert control) -----"
$GEN "$JSON" -o "$OUT/sv_arm2_shipped_parser.rs" >/dev/null 2>&1 \
  || { echo "arm 2 generation FAILED" >&2; exit 1; }

echo
row "ARM 1  narrow admission (pre-slice-9)" "$OUT/sv_arm1_narrow_parser.rs"
row "ARM 2  SHIPPED (absorbed + guarded)"   "$OUT/sv_arm2_shipped_parser.rs"
row "ARM 3  absorbed, guards SUPPRESSED"    "$OUT/sv_arm3_noguard_parser.rs"
echo
python3 - "$OUT" "$ROOT/generated/systemverilog_parser.rs" <<'PY'
import hashlib, os, sys
out, shipped = sys.argv[1], sys.argv[2]
def norm(p, drops):
    b = open(p, encoding="utf-8", errors="replace").read()
    for d in drops:
        b = b.replace(d, "<OUT>")
    return hashlib.sha256(b.encode()).hexdigest()[:16]
a2 = os.path.join(out, "sv_arm2_shipped_parser.rs")
drops = (a2, "../generated/systemverilog_parser.rs", "generated/systemverilog_parser.rs", a2.lstrip("./"))
# ⛔ The comparison NORMALISES the embedded output path: a generated parser records its own `-o`
# target as a diagnostic string (5+ sites), so two parsers written to different paths can never be
# byte-identical. Measured while promoting the census instruments (CI-PARITY-GATE-ROT.32 (d)).
x, y = norm(a2, drops), norm(shipped, drops)
print(f"revert control: ARM 2 regenerated {x} vs shipped generated/ {y}  "
      f"{'IDENTICAL — the patch left no residue' if x == y else '⛔ DIFFER — the revert did NOT restore behaviour'}")
sys.exit(0 if x == y else 1)
PY
rc=$?
git diff --quiet -- rust/src/ || { echo "⛔ rust/src/ is dirty at exit" >&2; rc=1; }
echo "------------------------------------------------------------------------------"
[ "$rc" -eq 0 ] && echo "✅ three arms generated; engine restored byte-exactly." \
                || echo "⛔ see above."
exit "$rc"
