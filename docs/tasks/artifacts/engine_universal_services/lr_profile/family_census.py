#!/usr/bin/env python3
"""Corpus-wide PER-RULE entry aggregation, so any LR-family definition can be priced.

`.20` slice 4, open audit item 4 (and a finding it surfaced): the tracked instrument's
`LR_FAMILY_RE` is `_lr_base$|_lr_suffix(_r\\d+)?$`, which EXCLUDES the `*_lr_seed_*` rules
the same elimination pass emits. The published `0.681 %` family share — and therefore the
`~35x` blind-spot factor mirrored in four places — is computed on that narrow set.

This aggregates the raw per-rule entry map over the whole corpus ONCE, so the share under
ANY predicate is a post-hoc sum rather than another 10-minute run.

Writes:  rule_totals.tsv   rule <TAB> entries <TAB> committed <TAB> files_entering
         summary.json      totals + per-predicate shares + file-share
"""
import concurrent.futures
import json
import os
import re
import subprocess
import sys
import tempfile

ROOT = os.getcwd()
GRAMMAR, PROFILE = "systemverilog", "sv_2017"
PROBE = "rust/target/release/parseability_probe"
CORPUS_MANIFEST = "stimuli/sv/characterization/durations.tsv"
OUTDIR = sys.argv[1] if len(sys.argv) > 1 else "rust/target/lr_profile/family_census"
JOBS = int(sys.argv[2]) if len(sys.argv) > 2 else 8
TIMEOUT_S = 300

# ⛔ PROMOTED OUT OF `rust/target/audit_scratch/` BY `.21` ACCEPTANCE (e). This is the producer of the
# corpus-wide `2.7411 %` family share and the `73 rules / 24 644 435 entries` decomposition that four
# published surfaces rest on; while it lived in a gitignored directory those numbers could not be
# re-derived by anyone, including their author after a `cargo clean`.
#
# NARROW / SEED / GUARD are the HISTORICAL decomposition — kept because the published breakdown is
# stated in those terms (narrow 0.6813 %, seed 1.6840 %, guard 0.3758 %). LIVE is imported from the
# single home so the headline share can never drift from what the ratchet reports.
NARROW = re.compile(r"_lr_base$|_lr_suffix(_r\d+)?$")          # pre-`.21` tracked set
SEED = re.compile(r"_lr_seed")
GUARD = re.compile(r"_lr_guard")
sys.path.insert(0, os.path.join(ROOT, "stimuli", "sv"))
try:
    from corpus_parse_cost import LR_FAMILY_RE as LIVE
except Exception as exc:
    sys.exit(f"REFUSE: cannot import LR_FAMILY_RE from stimuli/sv/corpus_parse_cost.py: {exc}")


def one(rel):
    fd, tmp = tempfile.mkstemp(suffix=".json", dir=os.path.join(ROOT, "rust", "target"))
    os.close(fd)
    try:
        subprocess.run([os.path.join(ROOT, PROBE), "--parse", GRAMMAR,
                        os.path.join(ROOT, rel), "--profile", PROFILE,
                        "--dump-rule-outcome-counts-json", tmp],
                       capture_output=True, timeout=TIMEOUT_S)
        if os.path.getsize(tmp) == 0:
            return None
        with open(tmp, encoding="utf-8") as fh:
            return rel, json.load(fh)
    except (subprocess.TimeoutExpired, json.JSONDecodeError, OSError):
        return None
    finally:
        try:
            os.unlink(tmp)
        except OSError:
            pass


files = sorted({l.split("\t")[2] for l in open(os.path.join(ROOT, CORPUS_MANIFEST),
                                               encoding="utf-8") if l.strip()})
files = [f for f in files if os.path.isfile(os.path.join(ROOT, f))]
print(f"family-census: {len(files)} files at -j{JOBS}", file=sys.stderr)

ent_tot, com_tot, files_entering = {}, {}, {}
n_ok = n_nodump = 0
grand_entries = grand_committed = 0
files_with_narrow = files_with_seed = files_with_any = 0

with concurrent.futures.ThreadPoolExecutor(max_workers=JOBS) as ex:
    for i, res in enumerate(ex.map(one, files)):
        if i % 2000 == 0:
            print(f"  … {i}/{len(files)}", file=sys.stderr)
        if res is None:
            n_nodump += 1
            continue
        n_ok += 1
        _rel, d = res
        e = d.get("rule_entry_counts", {})
        c = d.get("rule_committed_counts", {})
        grand_entries += int(d.get("total_entries", 0))
        grand_committed += int(d.get("total_committed", 0))
        hit_narrow = hit_seed = False
        for k, v in e.items():
            ent_tot[k] = ent_tot.get(k, 0) + v
            files_entering[k] = files_entering.get(k, 0) + 1
            if NARROW.search(k):
                hit_narrow = True
            elif SEED.search(k) or GUARD.search(k):
                hit_seed = True
        for k, v in c.items():
            com_tot[k] = com_tot.get(k, 0) + v
        files_with_narrow += hit_narrow
        files_with_seed += hit_seed
        files_with_any += (hit_narrow or hit_seed)

os.makedirs(os.path.join(ROOT, OUTDIR), exist_ok=True)
with open(os.path.join(ROOT, OUTDIR, "rule_totals.tsv"), "w", encoding="utf-8") as fh:
    fh.write("rule\tentries\tcommitted\tfiles_entering\n")
    for k in sorted(ent_tot, key=lambda r: -ent_tot[r]):
        fh.write(f"{k}\t{ent_tot[k]}\t{com_tot.get(k, 0)}\t{files_entering.get(k, 0)}\n")


def share(pred):
    s = sum(v for k, v in ent_tot.items() if pred(k))
    return s, (100.0 * s / grand_entries if grand_entries else 0.0)


narrow_e, narrow_p = share(lambda k: NARROW.search(k))
seed_e, seed_p = share(lambda k: SEED.search(k))
guard_e, guard_p = share(lambda k: GUARD.search(k))
wide_e, wide_p = share(lambda k: NARROW.search(k) or SEED.search(k) or GUARD.search(k))
live_e, live_p = share(lambda k: LIVE.search(k))
live_rules = sorted(k for k in ent_tot if LIVE.search(k))

summary = {
    "live_predicate": LIVE.pattern,
    "live_lr_entries": live_e, "live_lr_pct": round(live_p, 4),
    "live_lr_rules_entered": len(live_rules),
    "live_equals_historical_wide": live_e == wide_e,
    "files_measured": n_ok, "files_nodump": n_nodump,
    "total_entries": grand_entries, "total_committed": grand_committed,
    "narrow_lr_entries": narrow_e, "narrow_lr_pct": round(narrow_p, 4),
    "seed_entries": seed_e, "seed_pct": round(seed_p, 4),
    "guard_entries": guard_e, "guard_pct": round(guard_p, 4),
    "wide_lr_entries": wide_e, "wide_lr_pct": round(wide_p, 4),
    "files_entering_narrow": files_with_narrow,
    "files_entering_narrow_pct": round(100.0 * files_with_narrow / n_ok, 2) if n_ok else 0,
    "files_entering_any_lr": files_with_any,
    "files_entering_any_lr_pct": round(100.0 * files_with_any / n_ok, 2) if n_ok else 0,
    "distinct_rules": len(ent_tot),
}
with open(os.path.join(ROOT, OUTDIR, "summary.json"), "w", encoding="utf-8") as fh:
    json.dump(summary, fh, indent=2, sort_keys=True)
print(json.dumps(summary, indent=2, sort_keys=True))
