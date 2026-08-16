#!/usr/bin/env bash
# docs/tasks/artifacts/engine_universal_services/build_guard_ab_probes.sh
#
# ENGINE-UNIVERSAL-SERVICES.20 acceptance (b) — build the three release probes the TIMED tier needs.
#
# ⛔ THIS EXISTS BECAUSE THE RECIPE USED TO BE A COMMENT. `run_guard_ab_timed.sh` refuses when a probe
# is missing and points at its own header, where the build was written out in prose. A recipe that
# only a human can execute is the same defect `.21`(e) is a record of — the INSTRUMENTS were tracked
# and the thing that PRODUCES their input was not — so it is a script now, and it records what it
# built.
#
#   ARM 1  narrow admission (pre-`.17`-slice-9 policy)  — the knot is NOT absorbed, no guard emitted
#   ARM 3  absorbed, guard emission SUPPRESSED          — the missing arm (a tracked patch, never shipped)
#   ARM 2  SHIPPED                                      — absorbed AND guarded
#
# ⛔⛔ ALL THREE ARE REBUILT TOGETHER, FROM THE ARM PARSERS `run_guard_ab_structural.sh` JUST WROTE,
# AND THAT IS NOT TIDINESS. A probe embeds the ENGINE that generated its parser as well as the parser
# itself, and `.22`(e) moved that engine: it added exactly +2 566 bytes to every generated parser and
# reshaped `MemoEntry.coverage_delta` from `Option<Vec<u32>>` to `Option<u32>`.
# ⚠️ SAY WHAT THAT DOES AND NOT MORE. Read at `ast_based_generator.rs:9289` (and `:9166` before the
# fix), the coverage payload is `None` when `coverage_enabled` is false on BOTH sides of the change —
# the engine's own comment says *"ordinary parsing pays nothing"* — and `parseability_probe` parses
# with coverage OFF. So this did NOT change the work an ordinary memo insert performs. What it changed
# is the SIZE of every `MemoEntry`: three words of `Vec` header become one word, so the memo table's
# footprint and cache behaviour move on every parse even though its instruction path does not. That is
# a second-order effect, not a hot-path rewrite — but it is an effect, it lands on a table walked
# hundreds of millions of times per corpus run, and it is unmeasured. Combined with the +2 566 bytes,
# reusing a probe from before the change would vary two things at once and charge the difference to
# the guards. So: regenerate all three arms, then rebuild all three probes. Never mix vintages.
#
# ⛔ Each build is ~21 min and peaks near 12 GB, hence the memory guard. Budget 16 GB, timeout 45 min
# per arm. The three run SEQUENTIALLY: two concurrent 12 GB builds do not fit in 24 GB of host RAM.
#
# ⛔⛔ AFTER THIS SCRIPT, THE MACHINE IS NOT QUIET. `.20` slice 4's timed tier was noise-limited
# precisely because it measured on a host that had just absorbed three of these builds — ARM 1 moved
# +21 % between passes on an IDENTICAL binary. Let the host settle (≥1 h, no builds) BEFORE running
# `run_guard_ab_timed.sh`, and read its spread verdict rather than its split.
#
# Usage: bash docs/tasks/artifacts/engine_universal_services/build_guard_ab_probes.sh [arm ...]
#        (no argument = all three, in the order 1, 3, 2)
# Exit 0 iff every requested probe built AND passed its identity check.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../.." && pwd)"; cd "$ROOT"

OUT="rust/target/lr_ab_arms"
STAMP="$OUT/probe_identity.txt"

# arm -> the generated parser it is built from (repo-root-relative; build.rs resolves against rust/)
declare -A SRC=(
  [arm1]="$OUT/sv_arm1_narrow_parser.rs"
  [arm2]="$OUT/sv_arm2_shipped_parser.rs"
  [arm3]="$OUT/sv_arm3_noguard_parser.rs"
)
# ⛔⛔ THE IDENTITY CHECK IS THE EMBEDDED PARSER PATH, NOT A SYMBOL COUNT, AND THE DIFFERENCE MATTERS.
# `.20` slice 4 identified its arms with `nm probe | grep -c _lr_guard`. Measured here, that reads
# **0 for ARM 1 and 0 for ARM 3** — it cannot tell the un-absorbed arm from the absorbed-but-unguarded
# one, so it would have passed a run in which one arm's parser was silently used for both. Adding
# `_lr_suffix` does not repair it either: ARM 1 has **13** such symbols, not 0, so any rule would be a
# hand-picked threshold rather than a fact.
#
# ⭐ The exact identity was already sitting there, in the defect `.25` is a record of: a generated
# parser embeds its own `-o` path as a diagnostic string (36 346 sites in SV), so every probe NAMES
# the parser it was built from and names no other. Measured on the three pre-`.22`(e) probes, the
# matrix is diagonal — each probe matches exactly one arm parser, 3 distinct strings, 0 for the other
# two — which makes this an exact mutually-exclusive check, not a threshold.
#
# ⛔⛔ AND ON ITS FIRST RUN THE CHECK FOUND A REAL ASYMMETRY IN `.20` SLICE 4's OWN ARMS. Measured:
#     probe_arm1  rust/target/lr_ab_arms/sv_arm1_narrow_parser.rs    (46 chars)
#     probe_arm3  rust/target/lr_ab_arms/sv_arm3_noguard_parser.rs   (47 chars)
#     probe_arm2  ../generated/systemverilog_parser.rs               (36 chars)  ⛔ not an arm parser
# ARM 2's probe was built from `generated/`, not from `sv_arm2_shipped_parser.rs`, so the SHIPPED arm
# was the one arm whose binary did not come off the same shelf as the others. ⚠️ Say exactly what that
# does and does not mean: the two parsers are byte-identical once the path is normalised (the
# structural runner's own revert control proves it), so the arm was BEHAVIOURALLY right — the defect
# is homogeneity, not correctness. Its price is dead string text: 11 characters × 36 346 sites ≈
# **400 KB** of `__cstring` that ARM 3's binary carries and ARM 2's does not, ~0.5 % of a 76 MB
# binary. Well under the noise floor this tier fights, but it was neither measured nor declared, and
# an unstated asymmetry in a comparison is exactly what `.25` is a record of. Rebuilding all three
# from `$OUT/` leaves a 1-character residue (ARM 1, ~33 KB) which is reported per arm below.
declare -A WANT_GUARD=( [arm1]=0 [arm3]=0 [arm2]=1 )   # 0 = must be absent, 1 = must be present

# `--verify-only` runs the identity check against whatever probes are already on disk and builds
# nothing. It exists so the check itself is testable: proving a check can go RED costs 21 minutes a
# time if the only way to exercise it is to build an arm
# ([[deriving-a-control-from-the-producer-is-not-the-same-as-observing-it]]). It is also what the
# timed tier should call before it starts measuring.
VERIFY_ONLY=0
if [ "${1:-}" = "--verify-only" ]; then VERIFY_ONLY=1; shift; fi

ARMS=("${@:-}")
[ -z "${ARMS[0]:-}" ] && ARMS=(arm1 arm3 arm2)

mkdir -p "$OUT"
for a in "${ARMS[@]}"; do
  [ -f "${SRC[$a]}" ] || { echo "refusing: ${SRC[$a]} is missing — run run_guard_ab_structural.sh first." >&2; exit 2; }
done

echo "=============================================================================="
[ "$VERIFY_ONLY" = 1 ] \
  && echo "ENGINE-UNIVERSAL-SERVICES.20 (b) — VERIFY ONLY (no build): ${ARMS[*]}" \
  || echo "ENGINE-UNIVERSAL-SERVICES.20 (b) — building release probes: ${ARMS[*]}"
echo "=============================================================================="
: >"$STAMP.new"
rc=0
for a in "${ARMS[@]}"; do
  src="${SRC[$a]}"
  if [ "$VERIFY_ONLY" = 0 ]; then
    echo "--- $a  from $src"
    # `build.rs` resolves PGEN_SYSTEMVERILOG_PARSER_PATH against rust/ (CARGO_MANIFEST_DIR), so the
    # repo-root-relative path is passed with its leading `rust/` stripped. Relative throughout: the
    # repository must survive being moved, including across filesystems (CLAUDE.md §12).
    ( cd rust && PGEN_SYSTEMVERILOG_PARSER_PATH="${src#rust/}" \
        ../scripts/run_with_memory_guard.sh --budget-mb 16384 --timeout-s 2700 -- \
        cargo build --release --features generated_parsers --bin parseability_probe ) \
      >"$OUT/build_$a.log" 2>&1
    brc=$?
    if [ "$brc" -ne 0 ]; then
      echo "  ⛔ build FAILED (exit $brc) — see $OUT/build_$a.log"; rc=1; continue
    fi
    cp rust/target/release/parseability_probe "$OUT/probe_$a"
  fi
  [ -x "$OUT/probe_$a" ] || { echo "  ⛔ $OUT/probe_$a is missing"; rc=1; continue; }
  g="$(nm "$OUT/probe_$a" 2>/dev/null | grep -c '_lr_guard')"
  s="$(nm "$OUT/probe_$a" 2>/dev/null | grep -c '_lr_suffix')"
  h="$(shasum -a 256 "$src" | cut -c1-16)"
  # EXACT identity: the probe must name ITS OWN arm parser and NEITHER of the others.
  mine="$(strings -a "$OUT/probe_$a" | grep -c "$(basename "$src")")"
  theirs=0
  for b in arm1 arm2 arm3; do
    [ "$b" = "$a" ] && continue
    theirs=$(( theirs + $(strings -a "$OUT/probe_$a" | grep -c "$(basename "${SRC[$b]}")") ))
  done
  ok=ok
  [ "$mine" -eq 0 ] && ok="⛔ does not name $(basename "$src") — it was built from some other parser"
  [ "$theirs" -ne 0 ] && ok="⛔ names ANOTHER arm's parser ($theirs sites) — the arms are crossed"
  { [ "${WANT_GUARD[$a]}" = 0 ] && [ "$g" -ne 0 ]; } && ok="⛔ _lr_guard symbols present but must be absent"
  { [ "${WANT_GUARD[$a]}" = 1 ] && [ "$g" -eq 0 ]; } && ok="⛔ _lr_guard symbols absent but must be present"
  printf '%-6s parser_sha256=%s  names_own=%-3s names_other=%-3s _lr_guard=%-4s _lr_suffix=%-4s bytes=%-10s %s\n' \
    "$a" "$h" "$mine" "$theirs" "$g" "$s" "$(wc -c <"$OUT/probe_$a" | tr -d ' ')" "$ok" \
    | tee -a "$STAMP.new"
  [ "$ok" = ok ] || rc=1
done
mv "$STAMP.new" "$STAMP"
echo "------------------------------------------------------------------------------"
[ "$rc" -eq 0 ] && echo "✅ every requested probe built and is the arm it claims to be (identity in $STAMP)." \
                || echo "⛔ see above — do NOT run the timed tier until every arm is its own."
exit "$rc"
