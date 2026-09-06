#!/usr/bin/env python3
"""SV-CORPUS-GRAD.3.42 — corpus LIVENESS census for a set of grammar rules: which corpus files make a
rule COMMIT (appear in a successful derivation) under a given probe and profile.

Runs `parseability_probe --parse systemverilog <file> --profile <p> --dump-rule-outcome-counts-json`
(TOOLBOX §3.5) over a file list, in parallel, and reports for every named rule the files whose
`rule_committed_counts[rule] > 0`, plus `entered` for context.

⛔ READ `committed` AS AN OCCURRENCE ORACLE, NEVER AS FINAL-TREE MEMBERSHIP (TOOLBOX §3.5, C3-B):
a tournament winner AND a successful-but-losing branch both commit, so `committed > 0` is a
SUPERSET of "the rule's node is in the accepted tree". `committed == 0` on every file IS proof of a
dead rule on that corpus; `committed > 0` names the files whose typed AST must then be compared
(`ast_identity_sweep.py` beside this script) to decide whether a tree moved.

Usage:
  python3 docs/tasks/artifacts/sv_corpus_grad/inline_alt_lift/liveness_census.py \
      --probe rust/target/release/parseability_probe --profile sv_2017 \
      --files rust/target/sNNN/all_sv_files.txt --rules a,b,c --out rust/target/sNNN/liveness_sv_2017.tsv [--jobs 8]
Prints one `LIVENESS-CENSUS:` headline and one line per (rule, file) with committed > 0; writes the TSV.
Exit 0 always on a completed census; a probe that fails to produce a dump or times out is COUNTED in the headline
AND NAMED in the TSV (`-\t<file>\ttimeout|nodump|malformed`), never dropped — re-run those files on an idle machine.
"""
from __future__ import annotations
import argparse, concurrent.futures, json, os, subprocess, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]

def one(args):
    probe, profile, rel, rules, timeout = args
    fd, tmp = tempfile.mkstemp(suffix=".json", dir=str(ROOT / "rust" / "target"))
    os.close(fd)
    try:
        try:
            proc = subprocess.run([probe, "--parse", "systemverilog", str(ROOT / rel), "--profile", profile,
                                   "--dump-rule-outcome-counts-json", tmp],
                                  capture_output=True, timeout=timeout)
        except subprocess.TimeoutExpired:
            return (rel, "timeout", None)
        if os.path.getsize(tmp) == 0:
            return (rel, "nodump", None)
        try:
            with open(tmp, encoding="utf-8") as fh:
                d = json.load(fh)
        except json.JSONDecodeError:
            return (rel, "malformed", None)
    finally:
        try: os.unlink(tmp)
        except OSError: pass
    ent = d.get("rule_entry_counts", {}); com = d.get("rule_committed_counts", {})
    return (rel, "accepted" if d.get("accepted") else "rejected",
            {r: (int(ent.get(r, 0)), int(com.get(r, 0))) for r in rules})

def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--probe", required=True); ap.add_argument("--profile", required=True)
    ap.add_argument("--files", required=True, help="one repo-relative path per line")
    ap.add_argument("--rules", required=True, help="comma-separated rule names")
    ap.add_argument("--out", required=True); ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--timeout-s", type=int, default=120)
    a = ap.parse_args()
    rules = [r for r in a.rules.split(",") if r]
    files = [l.strip() for l in open(a.files) if l.strip() and not l.startswith("#")]
    probe = str((ROOT / a.probe) if not os.path.isabs(a.probe) else Path(a.probe))
    rows = []; verdicts = {"accepted": 0, "rejected": 0, "timeout": 0, "nodump": 0, "malformed": 0}
    # ⛔ a row the probe could not measure is COUNTED in the headline AND NAMED in the TSV: a `timeout` under
    # machine contention hides that file's commits, and a census that only counts them cannot be re-run on
    # exactly the files it missed (SV-CORPUS-GRAD.3.42 measured 8 timeouts beside a build; the idle re-run had 0).
    unmeasured = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=a.jobs) as ex:
        for rel, verdict, counts in ex.map(one, [(probe, a.profile, f, rules, a.timeout_s) for f in files]):
            verdicts[verdict] += 1
            if counts: rows.append((rel, verdict, counts))
            else: unmeasured.append((rel, verdict))
    live = {r: [(rel, v, c[r]) for rel, v, c in rows if c[r][1] > 0] for r in rules}
    entered = {r: sum(1 for rel, v, c in rows if c[r][0] > 0) for r in rules}
    with open(a.out, "w") as fh:
        fh.write("# liveness census — probe %s profile %s files %d\n" % (a.probe, a.profile, len(files)))
        fh.write("rule\tfile\tverdict\tentered\tcommitted\n")
        for r in rules:
            for rel, v, (e, c) in sorted(live[r]):
                fh.write(f"{r}\t{rel}\t{v}\t{e}\t{c}\n")
        for rel, v in sorted(unmeasured):
            fh.write(f"-\t{rel}\t{v}\t-\t-\n")
    summary = " ".join(f"{r}:entered_files={entered[r]},committed_files={len(live[r])}" for r in rules)
    print(f"LIVENESS-CENSUS: profile={a.profile} files={len(files)} accepted={verdicts['accepted']} rejected={verdicts['rejected']} "
          f"timeout={verdicts['timeout']} nodump={verdicts['nodump']} malformed={verdicts['malformed']} | {summary}")
    for r in rules:
        for rel, v, (e, c) in sorted(live[r]):
            print(f"  {r}\t{rel}\t{v}\tentered={e}\tcommitted={c}")
    for rel, v in sorted(unmeasured):
        print(f"  UNMEASURED\t{rel}\t{v}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
