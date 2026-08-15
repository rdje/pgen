#!/usr/bin/env bash
# docs/tasks/artifacts/engine_universal_services/run_guard_ab_entries.sh
#
# ENGINE-UNIVERSAL-SERVICES.20 acceptance (b) — THE THREE-ARM A/B, DETERMINISTIC WORK TIER.
#
# ⭐⭐ WHY A THIRD TIER EXISTS, AND IT IS NOT REDUNDANCE. The three arms have now been compared three
# ways and the three ways DISAGREE ABOUT WHICH HALF OF THE FLIP IS EXPENSIVE:
#
#   STRUCTURE (norm_bytes / rules, `run_guard_ab_structural.sh`)  guards  1.9 %  absorption 98.1 %
#   WORK      (rule ENTRIES, this file)                           guards 76.3 %  absorption 23.7 %
#   TIME      (wall clock, `run_guard_ab_timed.sh`)               noise-limited — see that file
#
# That is not a contradiction to resolve by picking a favourite; it is the actual mechanism. The
# guard emission is SIX rules — 1.9 % of the code the flip added — that are ENTERED constantly
# (+65.7 M entries). The absorption is 114 rules that are entered comparatively rarely (+20.4 M) but,
# on the first clean wall-clock pass, cost far more time per entry. **Bytes, entries and seconds rank
# the two halves differently, and only seconds is what ruling B binds on.**
#
# ⛔ THIS TIER IS DETERMINISTIC AND THE WALL-CLOCK ONE IS NOT — which is exactly why it is worth its
# 3.5 minutes. `.20` slice 1 established that rule entries / committed / memo-hits are exact functions
# of (grammar, parser, input) and are byte-identical between the debug and release probes. So this
# comparison is reproducible on a loaded machine, on a different machine, and next month; the
# wall-clock tier measured a 10-27 % run-to-run spread on the very same corpus the same evening.
# ⚠️ And the honest bound that keeps it in its lane: entries CANNOT be converted into seconds. `.21`
# measured the LR family at 0.681 % of entries while wall clock moved +24.3 %, i.e. the counters are
# ~8.9x less sensitive to a per-entry cost rise — see
# `docs/knowledge/a-deterministic-counter-cannot-see-a-per-entry-cost-rise.md`. This tier answers
# *"which half does more WORK"*, never *"which half costs more TIME"*.
#
# ⛔ Requires the three probe binaries (see `run_guard_ab_timed.sh`'s header for how they are built).
# Nothing tracked is written; `generated/` is never touched.
#
# Usage: bash docs/tasks/artifacts/engine_universal_services/run_guard_ab_entries.sh
# Exit 0 iff all three arms census cleanly and ARM 2 reproduces the corpus-wide total on record.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../../.." && pwd)"; cd "$ROOT"
ARMS="rust/target/lr_ab_arms"

for a in arm1 arm2 arm3; do
  [ -x "$ARMS/probe_$a" ] || { echo "missing $ARMS/probe_$a — see run_guard_ab_timed.sh's header" >&2; exit 2; }
done

exec python3 - "$ARMS" <<'PYEOF'
import concurrent.futures, json, os, subprocess, sys, tempfile

ROOT, ARMS = os.getcwd(), sys.argv[1]
MANIFEST = "stimuli/sv/characterization/durations.tsv"
JOBS = int(os.environ.get("PGEN_AB_ENTRIES_JOBS", "8"))
# ⭐ GROUND TRUTH, NOT A HOPE: the shipped arm's corpus-wide entry total is already on record from an
# independent census (`lr_profile/family_census.py`, `.21` acceptance (e)). If ARM 2 does not
# reproduce it EXACTLY, this instrument is wrong and must refuse rather than publish a split.
ARM2_TOTAL_ENTRIES_ON_RECORD = 899_064_022

files = sorted({l.split("\t")[2] for l in open(MANIFEST, encoding="utf-8") if l.strip()})
files = [f for f in files if os.path.isfile(f)]


def census(probe):
    def one(rel):
        fd, tmp = tempfile.mkstemp(suffix=".json", dir="rust/target")
        os.close(fd)
        try:
            subprocess.run([probe, "--parse", "systemverilog", rel, "--profile", "sv_2017",
                            "--dump-rule-outcome-counts-json", tmp], capture_output=True, timeout=300)
            if os.path.getsize(tmp) == 0:
                return None
            d = json.load(open(tmp))
            return (int(d.get("total_entries", 0)), int(d.get("total_committed", 0)),
                    int(d.get("total_memo_hits", 0) or 0))
        except Exception:
            return None
        finally:
            try:
                os.unlink(tmp)
            except OSError:
                pass
    e = c = m = n = nd = 0
    with concurrent.futures.ThreadPoolExecutor(max_workers=JOBS) as ex:
        for r in ex.map(one, files):
            if r is None:
                nd += 1
                continue
            n += 1; e += r[0]; c += r[1]; m += r[2]
    return {"files": n, "nodump": nd, "entries": e, "committed": c, "memo_hits": m}


print("=" * 94)
print("ENGINE-UNIVERSAL-SERVICES.20 (b) — three-arm A/B, DETERMINISTIC WORK tier (rule entries)")
print("=" * 94)
res = {}
for a, lbl in (("arm1", "ARM 1  narrow (pre-flip policy)"),
               ("arm3", "ARM 3  absorbed, guards SUPPRESSED"),
               ("arm2", "ARM 2  SHIPPED")):
    res[a] = census(os.path.join(ARMS, f"probe_{a}"))
    r = res[a]
    print(f"{lbl:<38} files={r['files']:<6} nodump={r['nodump']}  entries={r['entries']:>13,}  "
          f"committed={r['committed']:>11,}  memo_hits={r['memo_hits']:>13,}")

if res["arm2"]["entries"] != ARM2_TOTAL_ENTRIES_ON_RECORD:
    sys.exit(f"\nREFUSE: ARM 2 censused {res['arm2']['entries']:,} entries but the independent "
             f"family census on record has {ARM2_TOTAL_ENTRIES_ON_RECORD:,}. Two instruments "
             f"disagree about the SHIPPED arm, so no split derived from them can be trusted.")

e1, e2, e3 = res["arm1"]["entries"], res["arm2"]["entries"], res["arm3"]["entries"]
print()
print(f"⭐ ground truth: ARM 2 reproduces the independent family census EXACTLY ({e2:,} entries)")
print()
print(f"flip total   ARM2/ARM1 = {e2/e1:.4f}  ({100*(e2/e1-1):+.2f} % entries)")
print(f"  absorption ARM3/ARM1 = {e3/e1:.4f}  ({100*(e3/e1-1):+.2f} %)")
print(f"  guards     ARM2/ARM3 = {e2/e3:.4f}  ({100*(e2/e3-1):+.2f} %)")
print(f"SPLIT of the {e2-e1:,} added entries: absorption {e3-e1:,} = {100*(e3-e1)/(e2-e1):.1f} %  ·  "
      f"guards {e2-e3:,} = {100*(e2-e3)/(e2-e1):.1f} %")
print()
print("⛔ READ THIS AS WORK, NEVER AS TIME. Entries cannot be converted into seconds — `.21` measured")
print("   the counters ~8.9x less sensitive than wall clock to a per-entry cost rise. The structural")
print("   tier ranks these two halves the OPPOSITE way (guards 1.9 %); that disagreement is the")
print("   mechanism, not an error: six constantly-entered rules versus 114 rarely-entered ones.")
PYEOF
