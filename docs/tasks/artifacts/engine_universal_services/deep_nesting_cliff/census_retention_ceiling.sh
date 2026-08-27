#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.56 (c) — THE RETENTION CEILING PROBE.
#
# ⛔⛔⛔ THIS SCRIPT ONLY RUNS AGAINST A MEASUREMENT-ONLY BUILD THAT IS **UNSOUND BY DESIGN AND MUST
# NEVER BE LANDED.** The build env-gates both memos' failure-store refusals — the recursion-tainted
# gate (which caches nothing) and the depth-tainted gate (which files into the depth-stamped side
# map) — so that under `PGEN_EXPERIMENT_RETAIN_ALL_FAILURES=1` every failure is retained with no
# depth condition. `.43` measured that refusal as load-bearing: replaying such a failure from a
# shallower stack can reject a parse the guard would have allowed, and refusing to replay it
# regressed 4 corpus files pass→fail. The arm therefore CAN and DOES return wrong verdicts, and
# this script REPORTS THE VERDICTS rather than only the clock, because a faster wrong answer is not
# a result.
#
# WHY IT EXISTS: `.56`(a) established that the spine's failing calls are never retained, and (b)
# established that nothing observes the graph that actually ships. The only remaining way to price
# the fix before building it is the ceiling-probe pattern this repository already uses and
# publishes — the never-free bump `#[global_allocator]` that priced allocator removal at −40 % and
# was banked, never landed (`docs/book/src/inside-parser-performance.md`).
#
# THREE BINARIES' WORTH OF ARMS, FROM TWO BINARIES — because an instrument that changes two things
# at once cannot attribute an effect to either (`-0101`, this leaf's own founding correction):
#   S  SHIPPED     — `rust/target/parseability_probe.preexperiment`, the binary built from the
#                    TRACKED source before any probe edit. The reference.
#   A  PROBE-OFF   — the experiment binary with the flag UNSET. Differs from S by the added branch
#                    ONLY, so S vs A prices the PROBE'S OWN OVERHEAD.
#   B  RETAIN-ALL  — the experiment binary with the flag SET. A vs B prices RETENTION, with the
#                    branch held constant. This is the number the leaf owes.
#
# USAGE:  bash docs/tasks/artifacts/engine_universal_services/deep_nesting_cliff/census_retention_ceiling.sh
# Exit 0 on a successful measurement; exit 2 when it cannot be honest.
set -uo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT" || exit 1
P="rust/target/release/parseability_probe"
S="rust/target/parseability_probe.preexperiment"
[ -x "$P" ] || { echo "# ⛔ NOT EVALUATED — missing $P (the measurement-only build)"; exit 2; }
[ -x "$S" ] || { echo "# ⛔ NOT EVALUATED — missing $S (the pre-experiment reference binary)"; exit 2; }

# ⛔ The experiment binary must actually CARRY the probe, and the reference binary must NOT.
if ! grep -q "experiment_retain_all_failures" generated/systemverilog_parser.rs 2>/dev/null; then
  echo "# ⛔ NOT EVALUATED — generated/systemverilog_parser.rs carries no experiment gate, so the"
  echo "#   probe build is not in place (or has already been reverted). Re-apply, regenerate, rebuild."
  exit 2
fi
EXP_FP="$("$P" --parser-fingerprint 2>&1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["parsers"]["systemverilog"])')"
REF_FP="$("$S" --parser-fingerprint 2>&1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["parsers"]["systemverilog"])')"
DISK_FP="$(shasum -a 256 generated/systemverilog_parser.rs | cut -d' ' -f1)"
echo "# experiment probe SV fingerprint : $EXP_FP"
echo "# reference probe SV fingerprint  : $REF_FP"
echo "# generated/ on disk              : $DISK_FP"
if [ "$EXP_FP" != "$DISK_FP" ]; then
  echo "# ⛔ NOT EVALUATED — the experiment probe does not embed the parser now on disk."; exit 2
fi
if [ "$EXP_FP" = "$REF_FP" ]; then
  echo "# ⛔ NOT EVALUATED — the two binaries embed the SAME parser, so there is no experiment."; exit 2
fi
echo "# ⇒ two DIFFERENT parsers, the experiment one matching disk. Arms S / A / B are separable."
echo
echo "⛔⛔ EVERY NUMBER BELOW COMES FROM AN UNSOUND MEASUREMENT-ONLY BUILD. It prices a CEILING."
echo "   Nothing here is a proposal to land, and the VERDICT columns are why."

W="rust/target/retention_ceiling"; rm -rf "$W"; mkdir -p "$W"

run() {  # run <binary> <input> <env-on:0|1>  ->  "<verdict> <seconds>"
  local bin="$1" in="$2" on="$3" st en rc
  st=$(python3 -c 'import time;print(time.monotonic())')
  # ⛔ TIMED OUT rather than left to hang: the retain-all arm changes what the search explores and
  # is not guaranteed to terminate faster — or at all — so a run that exceeds the cap is reported
  # as NO-RESULT, never as a fast reject.
  if [ "$on" = "1" ]; then
    PGEN_EXPERIMENT_RETAIN_ALL_FAILURES=1 timeout 180 "$bin" --parse systemverilog "$in" --profile sv_2017 >/dev/null 2>&1
  else
    timeout 180 "$bin" --parse systemverilog "$in" --profile sv_2017 >/dev/null 2>&1
  fi
  rc=$?
  en=$(python3 -c 'import time;print(time.monotonic())')
  python3 -c "print('%s %.3f' % ({0:'accept',124:'TIMEOUT'}.get($rc,'reject'), $en-$st))"
}

echo
echo "=============================================================================="
echo "THE THREE ARMS on nested-parenthesis SystemVerilog (plain runs = the FUSED graph)"
echo "=============================================================================="
printf '%-7s | %-8s %8s | %-8s %8s | %-8s %8s | %8s %8s\n' \
  depth S_verdict S_sec A_verdict A_sec B_verdict B_sec 'A/S' 'A/B'
for d in 300 315 320 350 400 500 800; do
  IN="$W/in_$d.sv"
  python3 -c "import sys;n=$d;sys.stdout.write('module m; assign x = %s1%s; endmodule' % ('('*n, ')'*n))" > "$IN"
  read -r SV SS <<<"$(run "$S" "$IN" 0)"
  read -r AV AS <<<"$(run "$P" "$IN" 0)"
  read -r BV BS <<<"$(run "$P" "$IN" 1)"
  python3 -c "
print('%-7s | %-8s %8s | %-8s %8s | %-8s %8s | %8s %8s' % (
  $d, '$SV', '$SS', '$AV', '$AS', '$BV', '$BS',
  ('%.2fx' % ($AS/$SS)) if $SS > 0 else '-',
  ('%.2fx' % ($AS/$BS)) if $BS > 0 else '-'))"
done
echo "   A/S = the PROBE's own overhead (branch only, retention off)."
echo "   A/B = what RETENTION buys, with the branch held constant. Read the VERDICT columns first:"
echo "         a row where B disagrees with S is a wrong answer, not a win."

echo
echo "=============================================================================="
echo "THE (d) BAR under each arm — a rejection may cost <= 2x the deepest ACCEPTED parse"
echo "=============================================================================="
python3 - "$W" <<'PY'
import os, subprocess, sys, time
W = sys.argv[1]
BINS = [("S shipped", "rust/target/parseability_probe.preexperiment", "0"),
        ("A probe-off", "rust/target/release/parseability_probe", "0"),
        ("B retain-all", "rust/target/release/parseability_probe", "1")]
LADDER = [300, 315, 320, 350, 400, 800]

def run(binary, path, on):
    env = dict(os.environ)
    if on == "1":
        env["PGEN_EXPERIMENT_RETAIN_ALL_FAILURES"] = "1"
    else:
        env.pop("PGEN_EXPERIMENT_RETAIN_ALL_FAILURES", None)
    t = time.monotonic()
    try:
        rc = subprocess.run([binary, "--parse", "systemverilog", path, "--profile", "sv_2017"],
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=env,
                            timeout=180).returncode
    except subprocess.TimeoutExpired:
        return None, time.monotonic() - t          # None = NO RESULT, never counted as a reject
    return rc == 0, time.monotonic() - t

print("%-14s %10s %9s %10s %9s %9s  %s"
      % ("arm", "accepted", "cost", "reject", "cost", "ratio", "verdict"))
for name, binary, on in BINS:
    best = worst = None
    for d in LADDER:
        p = os.path.join(W, "in_%d.sv" % d)
        ok, secs = run(binary, p, on)
        if ok is None:
            print("%-14s depth %d TIMED OUT after %.1fs — arm NOT EVALUATED" % (name, d, secs))
            best = worst = None
            break
        if ok:
            best = (d, secs)
        else:
            worst = (d, secs)
    if not best or not worst:
        print("%-14s %10s %9s %10s %9s %9s  NOT EVALUATED (ladder never crossed the cliff)"
              % (name, best[0] if best else "-", "-", worst[0] if worst else "-", "-", "-"))
        continue
    r = worst[1] / best[1]
    print("%-14s %10d %8.3fs %10d %8.3fs %8.1fx  %s"
          % (name, best[0], best[1], worst[0], worst[1], r,
             "PASS(<=2x)" if r <= 2.0 else "RED"))
print()
print("⛔ A PASS on arm B is a CEILING, not a result: it says a sound fix that retained everything")
print("   would clear the bar. It does not say a sound fix CAN retain everything — arm B is unsound,")
print("   and the verdict table above is where that shows up.")
PY

cat <<'NOTE'

==============================================================================
READ IT LIKE THIS
==============================================================================
VERDICTS FIRST.  Arm B changes what the parser ANSWERS, deliberately. Any depth where B's verdict
                 differs from S's is the unsoundness being visible, and it is the reason this build
                 is a probe rather than a patch.
A/S              isolates the cost of the added branch. If it is ~1.00x the probe is free and arm B
                 measures retention alone; if it is not, say so beside every B number.
A/B              is the answer this leaf owes: the UPPER BOUND on what any sound retention fix can
                 buy on the over-ceiling plateau. Compare it against `.44`(d)'s requirement of 41x.
NOTE
