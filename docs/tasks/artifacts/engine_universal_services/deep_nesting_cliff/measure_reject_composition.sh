#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.44 (a) — re-runnable COMPOSITION of an over-ceiling reject.
#
# `.43` proved the over-ceiling parse TERMINATES; `.44` asks what the plateau is made of.
# `measure_cliff.sh` times it. This attributes it: it samples the shipped release probe on an
# N-deep parenthesised expression and splits the non-idle self-time into
#     error construction | memmove | allocator | actual parsing
#
# ⛔ WHY A SECOND FAMILY IS THE WHOLE POINT. `.44` opened on ONE 6-second VHDL sample and
#   published "60 % of it is ERROR CONSTRUCTION, not parsing", with an explicit honest bound that
#   SystemVerilog had never been measured. SystemVerilog is the family that SHIPS to Nexsim, and
#   its plateau does NOT have the same composition. Run both before quoting either.
#
# ⚠️ THE MAIN THREAD IS PARKED, NOT WORKING. The parse runs on a dedicated 256 MiB thread, so the
#   main thread's `__ulock_wait` samples are IDLE and must be excluded from the denominator.
#   Counting them makes every share look ~2x smaller.
#
# ⛔⛔ THE DENOMINATOR IS THE WORKER THREAD'S OWN TOTAL, **NOT** THE SUM OF THE LISTED ROWS, AND
#   GETTING THAT WRONG OVERSTATES EVERY SHARE. `sample` prints its self-time section as
#   "same collapsed (when >= 5)" -- entries with FEWER THAN FIVE samples are NOT LISTED AT ALL.
#   Summing the listed rows therefore divides by a number smaller than the truth. MEASURED under a
#   director re-verification challenge (2026-08-27): the unlisted tail is **7.9 %** of the
#   SystemVerilog worker thread and 1.9 % of the VHDL one, so the first cut published
#   create_contextual_error at 18.7 % / 62.0 % where the honest figures are **17.2 % / 61.9 %**.
#   The tail is a long list of DISTINCT parse functions each below the threshold, so it belongs to
#   the parsing bucket, not to error construction -- which means the error-construction share was
#   overstated and the parsing share understated, both in the same direction.
#   ⇒ This script now reads `N Thread_...` for the non-main thread and divides by THAT, and prints
#   the unlisted tail explicitly so the reader can see what the collapse hid.
#
# USAGE:  measure_reject_composition.sh [depth] [seconds] [family ...]
#         defaults: depth 800, 6 s, both families
# Exit 0 always: this is a MEASUREMENT, not a gate.
set -uo pipefail
# ⛔ Ask git for the root rather than counting `..` levels — this file sits five directories
# deep and the first cut counted four, so every run reported the probe missing while it was
# sitting there. `measure_cliff.sh` beside it already did it this way.
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT" || exit 1
PROBE="rust/target/release/parseability_probe"
DEPTH="${1:-800}"; SECS="${2:-6}"; shift 2 2>/dev/null || true
FAMS=("$@"); [ "${#FAMS[@]}" -eq 0 ] && FAMS=(systemverilog vhdl)
WORK="rust/target/reject_composition"; rm -rf "$WORK"; mkdir -p "$WORK"

[ -x "$PROBE" ] || { echo "missing $PROBE — build it with:"; \
  echo "  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)"; exit 2; }
command -v /usr/bin/sample >/dev/null 2>&1 || { echo "NOT EVALUATED — /usr/bin/sample absent"; exit 2; }

# ⛔ The fingerprint is printed, never assumed: a sample of a probe embedding a DIFFERENT parser
# than the one on disk attributes cost to code that is not what would ship.
echo "# parser-fingerprint: $("$PROBE" --parser-fingerprint 2>&1 | cut -c1-160)"
echo "# depth=$DEPTH sample=${SECS}s"

for fam in "${FAMS[@]}"; do
  case "$fam" in
    systemverilog) tmpl='module m; assign x = %s1%s; endmodule'; prof=sv_2017 ;;
    vhdl) tmpl='entity e is end entity; architecture a of e is begin s <= %s1%s; end architecture;'; prof=vhdl_1076_2019 ;;
    *) echo "unknown family $fam — REPORTED, not skipped silently"; continue ;;
  esac
  python3 -c "import sys;n=$DEPTH;sys.stdout.write('''$tmpl''' % ('('*n, ')'*n))" > "$WORK/in_$fam.txt"
  "$PROBE" --parse "$fam" "$WORK/in_$fam.txt" --profile "$prof" >/dev/null 2>&1 &
  pid=$!
  sleep 1
  /usr/bin/sample "$pid" "$SECS" -file "$WORK/$fam.sample.txt" >/dev/null 2>&1
  wait "$pid"; rc=$?
  printf '\n== %s (depth %s, probe rc=%s — nonzero is the REJECT this leaf is about) ==\n' "$fam" "$DEPTH" "$rc"
  python3 - "$WORK/$fam.sample.txt" <<'PY'
import re, sys
try:
    txt = open(sys.argv[1], encoding='utf-8', errors='replace').read()
except OSError:
    print("   NOT EVALUATED — no sample file"); raise SystemExit
marker = "Sort by top of stack, same collapsed (when >= 5):"
if marker not in txt:
    print("   NOT EVALUATED — sample produced no collapsed self-time section"); raise SystemExit
rows = []
for line in txt.split(marker)[1].split("\n\n")[0].strip().split("\n"):
    m = re.match(r'\s*(.+?)\s+(\d+)\s*$', line)
    if m:
        rows.append((m.group(1).strip(), int(m.group(2))))
idle = sum(n for f, n in rows if '__ulock_wait' in f or 'thread_start' in f)
work = [(f, n) for f, n in rows if not ('__ulock_wait' in f or 'thread_start' in f)]
listed = sum(n for _, n in work)
# ⛔ The TRUE denominator is the worker thread's own sample total, not the sum of listed rows:
# the self-time section omits every entry below 5 samples.
threads = re.findall(r'^\s*(\d+) Thread_\w+(.*)$', txt, re.M)
worker = [int(n) for n, d in threads if 'main-thread' not in d]
if not worker:
    print("   NOT EVALUATED — no worker thread found in the sample"); raise SystemExit
tot = worker[0]
if not tot:
    print("   NOT EVALUATED — worker thread has no samples"); raise SystemExit
def share(pred):
    s = sum(n for f, n in work if pred(f)); return s, 100.0 * s / tot
cce = share(lambda f: 'create_contextual_error' in f)
mm  = share(lambda f: 'memmove' in f or 'memcpy' in f)
alc = share(lambda f: re.search(r'malloc|free|realloc|nanov2|szone', f, re.I))
par = share(lambda f: re.search(r'::(parse_|cascade_)', f))
nfn = sum(1 for f, n in work if re.search(r'::(parse_|cascade_)', f))
print("   worker-thread samples (THE denominator): %d   main thread parked: %d" % (tot, idle))
print("   unlisted tail (<5 samples each, collapsed away): %d  %.1f%%  — parse functions, not error construction"
      % (tot - listed, 100.0 * (tot - listed) / tot))
print("   create_contextual_error   %5d  %5.1f%%" % cce)
print("   memmove/memcpy            %5d  %5.1f%%" % mm)
print("   allocator                 %5d  %5.1f%%" % alc)
print("   parse_*/cascade_*         %5d  %5.1f%%   over %d distinct fns" % (par[0], par[1], nfn))
print("   => error construction %.1f%%  |  actual parsing %.1f%%" % (cce[1] + mm[1], par[1]))
PY
done
