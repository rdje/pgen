#!/usr/bin/env bash
# docs/tasks/artifacts/engine_universal_services/run_guard_ab_timed.sh
#
# ENGINE-UNIVERSAL-SERVICES.20 acceptance (b) — THE THREE-ARM A/B, TIMED TIER.
#
# The structural tier (`run_guard_ab_structural.sh`) answers *how much machinery* each half of the
# flip added. This answers *how much TIME*, which is the question ruling B actually binds on — and
# the two answers differ by an order of magnitude, which is why both exist.
#
#   ARM 1  narrow admission (pre-`.17`-slice-9 policy)  — the knot is NOT absorbed, no guard emitted
#   ARM 3  absorbed, guard emission SUPPRESSED          — the missing arm (a tracked patch, never shipped)
#   ARM 2  SHIPPED                                      — absorbed AND guarded
#
# ⛔⛔ INTERLEAVED, NOT SEQUENTIAL, AND THAT IS THE POINT OF THIS SCRIPT. The first cut ran each arm
# once, back to back, in the order 2 → 3 → 1 — each immediately after a 20-minute release build. It
# produced a shipped arm **5.5 % slower than `.17` slice 9 measured** while the narrow arm reproduced
# that same slice to within **0.5 %**, which reads as a fresh regression in the shipped parser. That
# reading is not safe from a sequential run: a monotonic machine effect (thermal state after a long
# compile, page cache, background indexing) lands entirely on whichever arm went first. Interleaving
# A2 → A1 → A2 → A1 makes any monotonic drift cancel between the pair, and reporting the MEDIAN of
# each arm's runs makes a single outlier visible instead of decisive.
# ⭐ This is the same failure `.17` slice 9 already paid for once, in a different disguise: its own
# `~11 %` was wrong because it compared against a baseline measured *"under materially faster
# conditions"*. A stale baseline and an un-interleaved arm order are the same defect — a timing
# comparison across conditions that were not held equal.
#
# ⛔ REQUIRES the three probe binaries to exist already; it does NOT build them (each is ~20 min /
# 12 GB peak). Build them with, from `rust/`:
#     PGEN_SYSTEMVERILOG_PARSER_PATH=<arm parser> cargo build --release \
#         --features generated_parsers --bin parseability_probe
#   then copy the result aside. `sv_arm{1,3}_*_parser.rs` come from `run_guard_ab_structural.sh`.
#
# ⛔ Nothing tracked is written: every run redirects through `PGEN_CORPUS_OUT_DIR`, and
# `PGEN_CORPUS_REBASELINE=1` is required only because the runner's drift refusal is evaluated BEFORE
# it resolves `OUT_DIR` — the defect `SV-CORPUS-GRAD.3.28` owns. Both arms use a copied binary so the
# two sides take the identical code path.
#
# Usage: bash docs/tasks/artifacts/engine_universal_services/run_guard_ab_timed.sh [rounds]
# Exit 0 iff every run completes and every arm reproduces its expected corpus verdicts.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../.." && pwd)"; cd "$ROOT"

ARMS="rust/target/lr_ab_arms"
ROUNDS="${1:-2}"
declare -A EXPECT=( [arm1]=9762 [arm2]=9774 [arm3]=9487 )

for a in arm1 arm2 arm3; do
  [ -x "$ARMS/probe_$a" ] || { echo "missing $ARMS/probe_$a — see the header for how to build it" >&2; exit 2; }
done

run() {  # run <arm> <round>
  # ⛔⛔ THE THREE `local`s ARE SEPARATE STATEMENTS, AND THAT IS NOT STYLE — IT IS THE DEFECT THIS
  # SCRIPT SHIPPED FIRST. Written as one statement,
  #     local a="$1" r="$2" out="$ARMS/timed/${a}_r${r}"
  # bash expands the whole command line BEFORE performing any of the assignments, so `${a}` and
  # `${r}` resolve against the OUTER scope — where the arm-validation loop above had left `a=arm3`.
  # Every arm therefore wrote to `timed/arm3_r$r`, each run silently overwriting the previous one.
  # It did not look like a failure: each run read back ITS OWN results.tsv from that path, so all six
  # verdict checks printed ✓, and only the aggregator's "0 runs" for two arms exposed it. The mixed
  # column it produced (median 325.1 s, spread 53.0 s) was three different arms averaged together and
  # read as a plausible number.
  local a="$1"
  local r="$2"
  local out="$ARMS/timed/${a}_r${r}"
  mkdir -p "$out"
  PGEN_PARSE_PROBE_BIN="$ARMS/probe_$a" PGEN_CORPUS_OUT_DIR="$out" PGEN_CORPUS_REBASELINE=1 \
    bash stimuli/run_external_corpus.sh sv 60 8 0 >"$out/run.log" 2>&1
  local pass
  pass="$(awk -F'\t' '$2=="pass"' "$out/results.tsv" 2>/dev/null | wc -l | tr -d ' ')"
  printf '  %-5s round %s  pass=%-6s %s\n' "$a" "$r" "$pass" \
    "$([ "$pass" = "${EXPECT[$a]}" ] && echo '✓ verdicts as expected' || echo "⛔ expected ${EXPECT[$a]}")"
  [ "$pass" = "${EXPECT[$a]}" ]
}

echo "=============================================================================="
echo "ENGINE-UNIVERSAL-SERVICES.20 (b) — three-arm A/B, TIMED tier (interleaved)"
echo "=============================================================================="
rc=0
for r in $(seq 1 "$ROUNDS"); do
  echo "round $r — order arm2, arm1, arm3 (interleaved so monotonic drift cancels)"
  run arm2 "$r" || rc=1
  run arm1 "$r" || rc=1
  run arm3 "$r" || rc=1
done

python3 - "$ARMS/timed" "$ROUNDS" <<'PY'
import os, statistics, sys
base, rounds = sys.argv[1], int(sys.argv[2])
tot = {}
for a in ("arm1", "arm3", "arm2"):
    vals = []
    for r in range(1, rounds + 1):
        p = os.path.join(base, f"{a}_r{r}", "durations.tsv")
        # ⛔ REFUSE, DO NOT SKIP. The first cut `continue`d past a missing file and printed "—" for
        # the arm, which is how the path-aliasing defect above survived a whole run: two arms
        # reported nothing, one reported a mixed column, and the script still exited 0 saying
        # "every arm reproduced its expected corpus verdicts". An aggregator that cannot find its
        # input must say so, not average what is left.
        if not os.path.isfile(p):
            sys.exit(f"REFUSE: {p} is missing — {a} round {r} produced no durations. The runs and "
                     f"the aggregation disagree about where output goes; fix that before reading "
                     f"any number below it.")
        vals.append(sum(float(l.split("\t")[3]) for l in open(p)
                        if len(l.rstrip("\n").split("\t")) >= 4))
    tot[a] = vals
print()
print(f"{'arm':<40}{'runs':>6}{'median':>12}{'spread':>12}")
for a, lbl in (("arm1", "ARM 1  narrow (pre-flip policy)"),
               ("arm3", "ARM 3  absorbed, guards SUPPRESSED"),
               ("arm2", "ARM 2  SHIPPED")):
    v = tot.get(a) or []
    if not v:
        print(f"{lbl:<40}{0:>6}{'—':>12}{'—':>12}"); continue
    print(f"{lbl:<40}{len(v):>6}{statistics.median(v):>11.1f}s"
          f"{(max(v) - min(v)):>11.1f}s")
m = {a: statistics.median(v) for a, v in tot.items() if v}
if len(m) == 3:
    t1, t2, t3 = m["arm1"], m["arm2"], m["arm3"]
    print()
    print(f"flip total   ARM2/ARM1 = {t2/t1:.4f}  ({100*(t2/t1-1):+.1f} %)")
    print(f"  absorption ARM3/ARM1 = {t3/t1:.4f}  ({100*(t3/t1-1):+.1f} %)")
    print(f"  guards     ARM2/ARM3 = {t2/t3:.4f}  ({100*(t2/t3-1):+.1f} %)")
    print(f"SPLIT of the {t2-t1:.1f}s regression: "
          f"absorption {t3-t1:.1f}s = {100*(t3-t1)/(t2-t1):.1f} %  ·  "
          f"guards {t2-t3:.1f}s = {100*(t2-t3)/(t2-t1):.1f} %")
PY
echo "------------------------------------------------------------------------------"
[ "$rc" -eq 0 ] && echo "✅ every arm reproduced its expected corpus verdicts." \
                || echo "⛔ an arm's verdicts moved — the timing comparison is not safe; read above."
exit "$rc"
