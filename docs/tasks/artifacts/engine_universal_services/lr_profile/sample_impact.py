#!/usr/bin/env python3
"""Per-file corpus census under BOTH LR-family predicates, so `.21` acceptance (b) —
*"how many of the pinned sample's 40 `lr`-tier files change?"* — is measured before the
sample is touched, not after.

Writes per_file.tsv: rel <TAB> entries <TAB> narrow_lr <TAB> wide_lr
"""
import concurrent.futures
import json
import os
import re
import subprocess
import sys
import tempfile

ROOT = os.getcwd()
PROBE = "rust/target/release/parseability_probe"
MANIFEST = "stimuli/sv/characterization/durations.tsv"
# ⚠️ Bulk scratch output (16 k rows) stays under `rust/target/`, which is on the repository volume
# and gitignored. The DERIVED verdict is what gets committed, beside this file.
OUT = os.environ.get("PGEN_SAMPLE_IMPACT_OUT", "rust/target/lr_profile/sample_impact")
JOBS = int(os.environ.get("PGEN_SAMPLE_IMPACT_JOBS", "8"))

# ⛔ HISTORICAL — this is what the tracked instrument matched BEFORE `.21` slice 1, kept so the
# before/after is measurable. It is a RULE-NAME predicate (end-anchored) and is not the live one.
NARROW = re.compile(r"_lr_base$|_lr_suffix(_r\d+)?$")
# every shape the two emission sites actually produce:
#   indirect_lr_elimination.rs  {b}_lr_base · {b}_lr_suffix · {b}_lr_suffix_r{i}
#                               {b}_lr_seed_{r} · {b}_lr_guard{v} · {b}_lr_guard{v}_suffix
#                               {b}_lr_guard{v}_{hop}
#   ast_pipeline/mod.rs         {r}_lr_alt{n} · {r}_lr_base · {r}_lr_suffix
# `(?![a-z])` is the anchor that keeps the pinned near-miss `something_lr_baseline` OUT
# while tolerating allocate()'s `_{index}` collision suffix.
#
# ⛔ PROMOTED OUT OF `rust/target/audit_scratch/` BY `.21` ACCEPTANCE (e), and the copy this replaces
# RE-TYPED the wide predicate here. That is the same defect the leaf is about, one file over: two
# spellings of one rule drift the moment either is corrected. WIDE is now IMPORTED from the single
# home, and the import failing is a refusal rather than a fallback.
sys.path.insert(0, os.path.join(ROOT, "stimuli", "sv"))
try:
    from corpus_parse_cost import LR_FAMILY_RE as WIDE
except Exception as exc:
    sys.exit(f"REFUSE: cannot import LR_FAMILY_RE from stimuli/sv/corpus_parse_cost.py: {exc}")


def one(rel):
    fd, tmp = tempfile.mkstemp(suffix=".json", dir=os.path.join(ROOT, "rust", "target"))
    os.close(fd)
    try:
        subprocess.run([os.path.join(ROOT, PROBE), "--parse", "systemverilog",
                        os.path.join(ROOT, rel), "--profile", "sv_2017",
                        "--dump-rule-outcome-counts-json", tmp],
                       capture_output=True, timeout=120)
        if os.path.getsize(tmp) == 0:
            return None
        with open(tmp, encoding="utf-8") as fh:
            d = json.load(fh)
    except (subprocess.TimeoutExpired, json.JSONDecodeError, OSError):
        return None
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass
    e = d.get("rule_entry_counts", {})
    return (rel, int(d.get("total_entries", 0)),
            sum(v for k, v in e.items() if NARROW.search(k)),
            sum(v for k, v in e.items() if WIDE.search(k)))


PER_FILE = os.path.join(ROOT, OUT, "per_file.tsv")
if "--reuse" in sys.argv and os.path.isfile(PER_FILE):
    # ⭐ The tier arithmetic is POST-HOC, so re-analysing costs milliseconds instead of the 66 s
    # census. Added during the `.21` (e) promotion, when fixing the tier definition below would
    # otherwise have meant re-parsing 16 335 files to change three lines of sorting.
    rows = []
    with open(PER_FILE, encoding="utf-8") as fh:
        next(fh)
        for line in fh:
            p = line.rstrip("\n").split("\t")
            if len(p) == 4:
                rows.append((p[0], int(p[1]), int(p[2]), int(p[3])))
    print(f"sample-impact: reusing {len(rows)} rows from {PER_FILE}", file=sys.stderr)
else:
    files = sorted({l.split("\t")[2] for l in open(os.path.join(ROOT, MANIFEST), encoding="utf-8")
                    if l.strip()})
    files = [f for f in files if os.path.isfile(os.path.join(ROOT, f))]
    print(f"sample-impact: {len(files)} files at -j{JOBS}", file=sys.stderr)

    rows = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=JOBS) as ex:
        for i, r in enumerate(ex.map(one, files)):
            if i % 4000 == 0:
                print(f"  … {i}/{len(files)}", file=sys.stderr)
            if r:
                rows.append(r)

    os.makedirs(os.path.join(ROOT, OUT), exist_ok=True)
    with open(PER_FILE, "w", encoding="utf-8") as fh:
        fh.write("rel\tentries\tnarrow_lr\twide_lr\n")
        for rel, e, n, w in rows:
            fh.write(f"{rel}\t{e}\t{n}\t{w}\n")

# ⛔⛔ THE TIER DEFINITION THIS FILE USED WAS WRONG, AND ITS HEADLINE LINE WAS THEREFORE MEANINGLESS.
# The scratch version commented *"the instrument's `lr` tier = the 40 heaviest by guarded-admission
# entries"* and printed `pinned ∩ top-40 by NARROW`. That intersection is **structurally 0** and
# always was: `select_sample` (`stimuli/sv/corpus_parse_cost.py:496`) fills `hot` FIRST — the 40
# heaviest by TOTAL entries — and then takes the `lr` tier from what is LEFT (`if lrv > 0 and rel not
# in chosen`). The heaviest-by-LR files are precisely the heaviest-by-total files, so `hot` consumes
# them and the pinned `lr` tier can never intersect a raw top-40 by LR entries.
# ⇒ reading that `0/40` as evidence about the pinned sample would say the sample is entirely wrong,
# when the sample is fine and the PROXY was. `.21` slice 1's authoritative answer (`lr` 5 of 40 move)
# came from `select_sample` itself, never from here. The tiers below now mirror `select_sample`'s
# exclusion, so this file estimates the same quantity it names.
HOT_N, LR_N = 40, 40


def tiers(key):
    """(hot, lr) exactly as select_sample orders them: hot by total entries, then lr by family
    entries over what hot did not already take."""
    hot = [r[0] for r in sorted(rows, key=lambda r: (-r[1], r[0]))[:HOT_N]]
    hot_set = set(hot)
    lr = []
    for r in sorted(rows, key=lambda r: (-key(r), r[0])):
        if len(lr) >= LR_N:
            break
        if key(r) > 0 and r[0] not in hot_set:
            lr.append(r[0])
    return hot, lr


hot_n, lr_narrow = tiers(lambda r: r[2])
_, lr_wide = tiers(lambda r: r[3])
same = set(lr_narrow) & set(lr_wide)
pinned_lr = [l.split("\t")[1].strip() for l in
             open(os.path.join(ROOT, "stimuli/sv/parse_cost_sample.tsv"), encoding="utf-8")
             if l.startswith("lr\t")]
pinned_hot = [l.split("\t")[1].strip() for l in
              open(os.path.join(ROOT, "stimuli/sv/parse_cost_sample.tsv"), encoding="utf-8")
              if l.startswith("hot\t")]
print(f"\nrows measured                       : {len(rows)}")
print(f"pinned tiers                        : hot {len(pinned_hot)}, lr {len(pinned_lr)}")
print(f"`hot` tier reproduced               : {len(set(pinned_hot) & set(hot_n))}/{HOT_N}")
print(f"`lr` tier under NARROW reproduced   : {len(set(pinned_lr) & set(lr_narrow))}/{LR_N}")
print(f"`lr` tier under LIVE   reproduced   : {len(set(pinned_lr) & set(lr_wide))}/{LR_N}")
print(f"`lr` NARROW ∩ LIVE                  : {len(same)}/{LR_N}  "
      f"({LR_N - len(same)} rows the predicate fix would move)")
for r in sorted(rows, key=lambda r: (-r[3], r[0]))[:5]:
    print(f"   wide-heaviest: {r[3]:>9,} wide  {r[2]:>9,} narrow  {r[0]}")
