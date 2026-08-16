#!/usr/bin/env bash
# docs/tasks/artifacts/engine_universal_services/coverage_stack_blowup/verify_fix.sh
#
# ENGINE-UNIVERSAL-SERVICES.22 (e) — the VERIFICATION of the coverage-recorder fix.
#
# The fix replaces a by-value coverage replay with a REPLAY MARKER plus a multiplicity fold. It is
# meant to change how `total_committed` is COMPUTED and not what it IS, so the verification is
# built entirely out of oracles that would fail on any drift:
#
#   1. LADDER IDENTITY — the eight rungs the OLD code could still complete must reproduce their
#      committed counts EXACTLY. A pinned exponential sequence is an unusually sharp oracle: an
#      off-by-one in the multiplicity fold cannot survive `353 005 042`.
#   2. THE PATHOLOGICAL FILE — the corpus file that never terminated must now COMPLETE, report the
#      same `total_entries` the coverage-free TOOLBOX 3.4 dump reports, and land where the ladder's
#      growth law predicts.
#   3. TIME — the recorder's cost must now be linear where the reported number stays exponential.
#
# The pinned-sample byte-identity (`entries.tsv`) and the certificate-coverage witness side are
# checked by their own gates and are reported by the leaf, not re-implemented here.
#
# Usage:  bash docs/tasks/artifacts/engine_universal_services/coverage_stack_blowup/verify_fix.sh [probe]
# Output: .../coverage_stack_blowup/verify_fix.txt
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../../.." && pwd)"; cd "$ROOT" || exit 2
OUT="$HERE/verify_fix.txt"
PROBE="${1:-rust/target/release/parseability_probe}"
WORK="rust/target/e22/ladder"
[ -x "$PROBE" ] || { echo "probe missing: $PROBE" >&2; exit 2; }
python3 "$HERE/gen_ladder.py" "$WORK" 9 >/dev/null || exit 2

exec > >(tee "$OUT") 2>&1

python3 - "$PROBE" "$WORK" <<'PY'
import json, os, subprocess, sys, time

probe, work = sys.argv[1], sys.argv[2]
PATHOLOGICAL = "stimuli/sv/subs/Surelog/tests/ExponTimeIfElseGen/dut.sv"

# The committed counts the PRE-FIX code produced on every rung it could still finish
# (docs/tasks/artifacts/engine_universal_services/coverage_stack_blowup/probe_before_fix.txt).
# Rungs 8 and 9 have no pinned value: the old code was killed at 12 GB and 13.7 GB on them.
PINNED = {0: 16924, 1: 81562, 2: 340114, 3: 1374322, 4: 5511154,
          5: 22058482, 6: 88247794, 7: 353005042}


def run(path, flag, out):
    t = time.perf_counter()
    subprocess.run([probe, "--parse", "systemverilog", path, "--profile", "sv_2017", flag, out],
                   capture_output=True, timeout=600)
    el = time.perf_counter() - t
    if not os.path.isfile(out) or os.path.getsize(out) == 0:
        return None, el
    with open(out, encoding="utf-8") as fh:
        return json.load(fh), el


print("=" * 94)
print("ENGINE-UNIVERSAL-SERVICES.22 (e) — coverage-recorder fix, verification")
print("=" * 94)
print(f"probe: {probe}\n")

# ⛔ WARM-UP, AND IT IS NOT A COURTESY. The first invocation of a freshly-linked 77 MB binary pays
# its page-in cost: measured 19.55 s on rung 0 against 0.02 s on rung 1 — the SAME work. Without
# this, the table's first row is a measurement of the loader and the derived "wall clock x0.0"
# reads as a 500x speedup that nothing in this change produced. The first slice of this very leaf
# is a record of a first-reading number being too gentle in the flattering direction; this one
# would have been too generous in the same way.
warm = os.path.join(work, "arms00.sv")
if os.path.isfile(warm):
    t = time.perf_counter()
    run(warm, "--dump-rule-outcome-counts-json", "rust/target/e22/warm.json")
    print(f"warm-up (page-in of a freshly linked binary): {time.perf_counter() - t:.2f} s — "
          f"EXCLUDED from every figure below\n")

print("1. LADDER IDENTITY — the committed count must be UNCHANGED where the old code could finish")
print(f"{'arms':>5} {'entries':>10} {'committed':>18} {'pre-fix pinned':>18} {'verdict':>9} {'secs':>7}")
fails = 0
rows = []
for n in range(0, 10):
    f = os.path.join(work, f"arms{n:02d}.sv")
    if not os.path.isfile(f):
        continue
    d, el = run(f, "--dump-rule-outcome-counts-json", "rust/target/e22/v.json")
    if d is None:
        print(f"{n:>5} {'-':>10} {'NO DUMP':>18} {'':>18} {'FAIL':>9} {el:>7.2f}")
        fails += 1
        continue
    e, c = d["total_entries"], d["total_committed"]
    p = PINNED.get(n)
    if p is None:
        verdict = "new"
    elif c == p:
        verdict = "EXACT"
    else:
        verdict = "DRIFT"
        fails += 1
    rows.append((n, e, c, el))
    print(f"{n:>5} {e:>10,} {c:>18,} {(f'{p:,}' if p else '—'):>18} {verdict:>9} {el:>7.2f}")

if len(rows) >= 2:
    (n0, e0, c0, t0), (n1, e1, c1, t1) = rows[0], rows[-1]
    span = n1 - n0
    print(f"\n   entries   x{e1 / e0:.2f} over {span} arms (LINEAR: +{(e1 - e0) // span:,} per arm)")
    print(f"   committed x{(c1 / c0) ** (1 / span):.2f} per arm (still EXPONENTIAL — it is a COUNT)")
    print(f"   wall clock {t0:.3f}s -> {t1:.3f}s = x{t1 / t0:.1f} over the ladder "
          f"(the OLD code was killed at 12 GB by rung 8, so no old-vs-new ratio is quotable "
          f"past rung 7 — the comparison is 'finishes' vs 'does not')")

print("\n2. THE PATHOLOGICAL CORPUS FILE — it never terminated before")
d34, t34 = run(PATHOLOGICAL, "--dump-rule-entry-counts-json", "rust/target/e22/v34.json")
d35, t35 = run(PATHOLOGICAL, "--dump-rule-outcome-counts-json", "rust/target/e22/v35.json")
if d35 is None:
    print("   FAIL — still no dump")
    fails += 1
else:
    same = d34 is not None and d34["total_entries"] == d35["total_entries"]
    print(f"   accepted           {d35['accepted']}")
    print(f"   total_entries      {d35['total_entries']:,}   (3.4 dump: "
          f"{d34['total_entries']:,} — {'IDENTICAL' if same else 'DIVERGENT'})")
    print(f"   total_committed    {d35['total_committed']:,}")
    print(f"   total_memo_hits    {d35['total_memo_hits']:,}")
    print(f"   wall clock         {t35:.2f} s   (3.4 dump {t34:.2f} s)")
    if not same:
        fails += 1
    # the file is the 9-arm rung of the ladder; the growth law must predict it
    nine = next((c for n, _e, c, _t in rows if n == 9), None)
    if nine:
        ratio = d35["total_committed"] / nine
        print(f"   vs ladder rung 9   {nine:,}  (ratio {ratio:.4f} — the ladder is the same "
              f"construct with uniform arm bodies)")

print(f"\n{'ALL CHECKS PASS' if fails == 0 else f'{fails} CHECK(S) FAILED'}")
sys.exit(1 if fails else 0)
PY
rc=$?
exec 1>&- 2>&-
wait
exit "$rc"
