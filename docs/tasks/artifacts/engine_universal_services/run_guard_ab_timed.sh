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
# ⛔⛔⛔ AND THE PARAGRAPH ABOVE IS TRUE FOR **TWO** ARMS AND FALSE FOR THREE, WHICH IS THE DEFECT THIS
# SCRIPT SHIPPED WITH. Its rounds all ran the SAME fixed order — `arm2, arm1, arm3`, repeated. Under a
# monotonic within-round drift of Δ per slot, that gives arm2 `+0Δ`, arm1 `+1Δ` and arm3 `+2Δ` in
# EVERY round, so the bias does not cancel across rounds — it accumulates, and it accumulates in the
# direction that makes the guards (ARM2 − ARM3) look CHEAP or even negative. The A2→A1→A2→A1 argument
# in the paragraph above only cancels for a PAIR, where slot order reverses between the two members;
# with three arms a fixed order is just a slower version of the sequential run it replaced.
# ⭐ THE FIX IS COUNTERBALANCING, NOT MORE ROUNDS. Rounds now walk a 3×3 cyclic **Latin square**
#
#       round 1   arm2  arm1  arm3
#       round 2   arm1  arm3  arm2
#       round 3   arm3  arm2  arm1
#
# in which every arm occupies every slot exactly once. A drift that is linear in slot cancels EXACTLY
# over one full square, for every arm, instead of approximately. ⇒ the round count must be a MULTIPLE
# OF 3 or the square is incomplete and the cancellation claim is void; the script refuses to read a
# split off an unbalanced design rather than quietly reporting one.
#
# ⛔ THE ADMISSIBILITY TEST IS MECHANICAL, NOT AN EYEBALL. `.20` slice 4 refused to publish its split
# because the per-arm spread (up to 100.8 s, 27 %) exceeded the 24 % effect — but that refusal was a
# judgement made by reading a table. It is a computation now: a split is printed only when the largest
# per-arm spread is **< ¼ of the measured effect** (ARM 2 − ARM 1). Below that bar the script prints
# the bound it CAN support and says the split is unresolved. A number that cannot pass its own
# admissibility test must not appear in a form that can be quoted.
#
# ⛔ REQUIRES the three probe binaries to exist already; it does NOT build them (each is ~20 min /
# 12 GB peak). Build them with the tracked driver beside this script — which also proves each binary
# is the arm it claims to be, by the parser path the binary embeds:
#     bash docs/tasks/artifacts/engine_universal_services/run_guard_ab_structural.sh   # the 3 parsers
#     bash docs/tasks/artifacts/engine_universal_services/build_guard_ab_probes.sh     # the 3 probes
# ⛔ REBUILD ALL THREE TOGETHER whenever the ENGINE moves, not just when the grammar does: a probe
# embeds the engine that generated its parser. `.22`(e) added +2 566 bytes to every generated parser
# and shrank `MemoEntry` by two words, so probes from either side of it are not comparable. (⚠️ It did
# NOT change what an ordinary parse EXECUTES — the coverage payload is `None` either way when coverage
# is off; it changed how big the memo table's entries are. See `build_guard_ab_probes.sh`.)
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
ROUNDS="${1:-3}"
declare -A EXPECT=( [arm1]=9762 [arm2]=9774 [arm3]=9487 )

# ⭐ SMOKE SEAM. A full pass is ~55 minutes, so the orchestration around the measurement — the Latin
# square, the stale-round move, the per-run context files, the aggregator's admissibility branch —
# must be exercisable WITHOUT paying for it. `PGEN_AB_SMOKE_LIMIT=<n>` parses only the first n files
# per sub-corpus. ⛔ It is NOT a measurement mode: the corpus is truncated, so the verdict counts
# cannot match and the timings mean nothing. It says so loudly, and the verdict assertion is relaxed
# rather than silently passing, because a smoke run that LOOKS like a green measurement is exactly
# the kind of artifact that gets quoted later.
LIMIT="${PGEN_AB_SMOKE_LIMIT:-0}"
if [ "$LIMIT" != 0 ]; then
  echo "⛔⛔ SMOKE MODE — PGEN_AB_SMOKE_LIMIT=$LIMIT. Only the orchestration is under test."
  echo "    The corpus is TRUNCATED: verdict counts will not match and NO TIMING BELOW IS A"
  echo "    MEASUREMENT. Never quote a number from a smoke run."
fi

# The 3×3 cyclic Latin square: row r gives the order for round r, and over any three consecutive
# rounds every arm has occupied every slot exactly once. See the header for why a fixed order is not
# merely less tidy but actively biased.
SQUARE=( "arm2 arm1 arm3" "arm1 arm3 arm2" "arm3 arm2 arm1" )

if [ $(( ROUNDS % 3 )) -ne 0 ] || [ "$ROUNDS" -lt 3 ]; then
  echo "refusing: rounds=$ROUNDS — the counterbalancing square is 3 rows, so the round count must be" >&2
  echo "  a multiple of 3 and at least 3. An incomplete square leaves each arm with a different slot" >&2
  echo "  mix, which is the bias this design exists to remove." >&2
  exit 2
fi

for a in arm1 arm2 arm3; do
  [ -x "$ARMS/probe_$a" ] || { echo "missing $ARMS/probe_$a — see the header for how to build it" >&2; exit 2; }
done
# ⛔ PRECONDITION, NOT A COURTESY: prove each probe is the arm it claims BEFORE spending an hour
# measuring. `.20` slice 4's arms were checked with `nm | grep -c _lr_guard`, which reads 0 for ARM 1
# and 0 for ARM 3 alike and so cannot detect a crossed arm at all.
bash "$HERE/build_guard_ab_probes.sh" --verify-only \
  || { echo "⛔ arm identity check FAILED — not measuring. Rebuild the probes." >&2; exit 2; }

# ⚠️ QUIET-MACHINE ADVISORY. `.20` slice 4 was noise-limited because it measured on a host that had
# just absorbed three 21-minute / 12 GB release builds. This cannot enforce quiet, so it REPORTS the
# two things that would explain a wide spread afterwards — how long ago the tree was last built, and
# the load average — and it samples the load again around every run so drift is in the record.
newest_build="$(find rust/target/release rust/target/lr_ab_arms -maxdepth 1 -type f -newermt '-60 minutes' 2>/dev/null | head -1)"
[ -n "$newest_build" ] && echo "⚠️  a file under rust/target was written in the last hour ($newest_build)
   — the host may not be settled; read the SPREAD below before trusting any split."

# ⛔⛔ A STALE ROUND IS WORSE THAN A MISSING ONE, AND ONLY THE MISSING ONE WAS GUARDED. The aggregator
# REFUSES when `<arm>_r<n>/durations.tsv` is absent — the fix for the path-aliasing defect. It cannot
# refuse when the file is PRESENT but left over from an earlier invocation: a round that dies halfway
# leaves the previous run's numbers exactly where this one's belong, and they are read, aggregated and
# published as if they had just been measured. That is the same silent-wrong-number shape, one turn of
# the screw further on. So previous output is MOVED ASIDE (never deleted — it is evidence) before the
# first round, and every directory the aggregator reads is therefore one this invocation created.
if [ -d "$ARMS/timed" ] && [ -n "$(ls -A "$ARMS/timed" 2>/dev/null)" ]; then
  prev="$ARMS/timed.prev.$$"
  mv "$ARMS/timed" "$prev"
  echo "note: previous timed output moved aside to ${prev#"$PWD/"} — no run below can read a stale round."
fi
mkdir -p "$ARMS/timed"

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
  local slot="$3"
  local out="$ARMS/timed/${a}_r${r}"
  mkdir -p "$out"
  local load_before
  load_before="$(uptime | sed 's/.*load averages*: *//' | awk '{print $1}')"
  PGEN_PARSE_PROBE_BIN="$ARMS/probe_$a" PGEN_CORPUS_OUT_DIR="$out" PGEN_CORPUS_REBASELINE=1 \
    bash stimuli/run_external_corpus.sh sv 60 8 "$LIMIT" >"$out/run.log" 2>&1
  local pass load_after
  load_after="$(uptime | sed 's/.*load averages*: *//' | awk '{print $1}')"
  pass="$(awk -F'\t' '$2=="pass"' "$out/results.tsv" 2>/dev/null | wc -l | tr -d ' ')"
  # The slot and the loads are recorded per run so the counterbalancing can be AUDITED from the
  # artifacts rather than taken on the script's word.
  printf 'arm=%s round=%s slot=%s load_before=%s load_after=%s pass=%s\n' \
    "$a" "$r" "$slot" "$load_before" "$load_after" "$pass" >"$out/context.txt"
  if [ "$LIMIT" != 0 ]; then
    printf '  %-5s round %s slot %s  pass=%-6s (SMOKE — verdict assertion relaxed, corpus truncated)\n' \
      "$a" "$r" "$slot" "$pass"
    [ -s "$out/durations.tsv" ]   # the one thing smoke mode DOES assert: the run produced timings
    return
  fi
  printf '  %-5s round %s slot %s  pass=%-6s %s\n' "$a" "$r" "$slot" "$pass" \
    "$([ "$pass" = "${EXPECT[$a]}" ] && echo '✓ verdicts as expected' || echo "⛔ expected ${EXPECT[$a]}")"
  [ "$pass" = "${EXPECT[$a]}" ]
}

echo "=============================================================================="
echo "ENGINE-UNIVERSAL-SERVICES.20 (b) — three-arm A/B, TIMED tier (interleaved)"
echo "=============================================================================="
# ⛔⛔ WARM-UP, DISCARDED — AND IT IS NOT OPTIONAL BOOKKEEPING. Measured on a smoke pass, every arm's
# FIRST run costs ~5.2 s against ~0.5 s for its second and third on identical work: the cold page-in
# of a freshly linked 77 MB binary. `.22`(e) slice 2 published an `x0.0` speedup that was ENTIRELY
# this effect (19.55 s rung 0 vs 0.02 s rung 1, same work), so it is a known, named artifact here.
# It does not bias one arm over another — each arm pays it once — but it lands wholly in round 1 and
# so inflates EVERY arm's spread by several seconds. The spread is what the admissibility test
# adjudicates, so leaving it in spends the noise budget on a harness artifact rather than on the
# machine. One tiny discarded parse per probe pages each binary in for a few seconds' total cost.
echo "warm-up (discarded — pages each probe in; see the comment above)"
for a in arm1 arm2 arm3; do
  w="$ARMS/timed/warmup_$a"; mkdir -p "$w"
  PGEN_PARSE_PROBE_BIN="$ARMS/probe_$a" PGEN_CORPUS_OUT_DIR="$w" PGEN_CORPUS_REBASELINE=1 \
    bash stimuli/run_external_corpus.sh sv 60 8 5 >"$w/run.log" 2>&1
  printf '  %-5s warmed\n' "$a"
done
# ⛔ The warm-up directories are removed so the aggregator cannot mistake one for a round. It globs
# `<arm>_r<n>`, which `warmup_<arm>` does not match — but relying on a naming near-miss to keep
# non-measurement data out of a measurement is precisely the shape that has already cost this leaf
# two defects, so they go.
rm -rf "$ARMS/timed"/warmup_arm[123]

rc=0
for r in $(seq 1 "$ROUNDS"); do
  order="${SQUARE[$(( (r - 1) % 3 ))]}"
  echo "round $r — order $order (Latin square row $(( (r - 1) % 3 + 1 )): every arm visits every slot)"
  slot=1
  for a in $order; do
    run "$a" "$r" "$slot" || rc=1
    slot=$(( slot + 1 ))
  done
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
print(f"{'arm':<40}{'runs':>6}{'median':>12}{'spread':>12}   per-round (in run order)")
for a, lbl in (("arm1", "ARM 1  narrow (pre-flip policy)"),
               ("arm3", "ARM 3  absorbed, guards SUPPRESSED"),
               ("arm2", "ARM 2  SHIPPED")):
    v = tot.get(a) or []
    if not v:
        print(f"{lbl:<40}{0:>6}{'—':>12}{'—':>12}"); continue
    # ⛔ The per-round values are printed, not just the median and the spread. `.20` slice 4's
    # aggregator reported a "median 325.1 s, spread 53.0 s" that was three different arms averaged
    # together; a reader given only the summary had no way to see that. Raw values are the audit.
    print(f"{lbl:<40}{len(v):>6}{statistics.median(v):>11.1f}s"
          f"{(max(v) - min(v)):>11.1f}s   " + "  ".join(f"{x:.1f}" for x in v))
m = {a: statistics.median(v) for a, v in tot.items() if v}
if len(m) != 3:
    sys.exit("REFUSE: fewer than three arms produced timings.")

t1, t2, t3 = m["arm1"], m["arm2"], m["arm3"]
spread = {a: max(v) - min(v) for a, v in tot.items()}
worst_arm = max(spread, key=spread.get)
worst = spread[worst_arm]
effect = t2 - t1                       # the quantity (b) is chartered to split
bar = abs(effect) / 4.0                # "spread < ¼ of the effect" — `.20` slice 4's own criterion
# The routing evidence measured the flip at +24.3 %; carrying it as a second, FIXED anchor keeps the
# admissibility bar from collapsing along with the effect if the effect itself comes out near zero.
chartered = 0.243 * t1

print()
print(f"ratios (medians)   ARM2/ARM1 = {t2/t1:.4f} ({100*(t2/t1-1):+.1f} %)   "
      f"ARM3/ARM1 = {t3/t1:.4f} ({100*(t3/t1-1):+.1f} %)   "
      f"ARM2/ARM3 = {t2/t3:.4f} ({100*(t2/t3-1):+.1f} %)")
print()
print("ADMISSIBILITY — is this instrument able to resolve the effect it is aimed at?")
print(f"  measured effect  ARM2 - ARM1        = {effect:>8.1f}s   ({100*effect/t1:+.1f} % of ARM 1)")
print(f"  chartered effect 0.243 x ARM1       = {chartered:>8.1f}s   (the +24.3 % the routing "
      f"evidence measured)")
print(f"  bar              effect / 4         = {bar:>8.1f}s")
print(f"  worst per-arm spread ({worst_arm})              = {worst:>8.1f}s")

if worst < bar:
    print("  ✅ ADMISSIBLE — the spread is under a quarter of the effect; the split below is readable.")
    print()
    print(f"SPLIT of the {effect:.1f}s regression: "
          f"absorption {t3-t1:.1f}s = {100*(t3-t1)/effect:.1f} %  ·  "
          f"guards {t2-t3:.1f}s = {100*(t2-t3)/effect:.1f} %")
else:
    print("  ⛔ NOT ADMISSIBLE — the noise floor is not below a quarter of the effect.")
    print("     ⇒ NO SPLIT IS PRINTED. This is the whole point: a split this instrument cannot")
    print("     support must not exist in a quotable form. What it CAN support:")
    # An honest bound rather than a point estimate: the effect is bracketed by the run-to-run
    # extremes, which is a statement the spread itself justifies.
    lo = (min(tot["arm2"]) - max(tot["arm1"])) / max(tot["arm1"])
    hi = (max(tot["arm2"]) - min(tot["arm1"])) / min(tot["arm1"])
    print(f"       ARM2 vs ARM1 lies within [{100*lo:+.1f} %, {100*hi:+.1f} %] across all runs.")
    if worst >= abs(chartered) / 4.0:
        print("       ⛔ The spread also exceeds a quarter of the CHARTERED +24.3 % effect, so this")
        print("          is a noisy host, not merely a shrunken effect. Settle the machine and re-run.")
    else:
        print("       ⭐ But the spread IS under a quarter of the chartered +24.3 % effect. The host is")
        print("          quiet enough to have resolved the ORIGINAL regression — so the reason it")
        print("          cannot resolve this one is that the effect itself has SHRUNK. That is a")
        print("          finding about the parser, not about the instrument: report it as one.")
PY
echo "------------------------------------------------------------------------------"
if [ "$LIMIT" != 0 ]; then
  # ⛔ A smoke run must NEVER sign off with the sentence a real run signs off with. It did in its
  # first cut — printing "✅ every arm reproduced its expected corpus verdicts" after relaxing the
  # very assertion that sentence reports on.
  echo "⛔ SMOKE RUN (PGEN_AB_SMOKE_LIMIT=$LIMIT) — orchestration exercised, NOTHING measured."
elif [ "$rc" -eq 0 ]; then
  echo "✅ every arm reproduced its expected corpus verdicts."
else
  echo "⛔ an arm's verdicts moved — the timing comparison is not safe; read above."
fi
exit "$rc"
