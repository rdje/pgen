#!/usr/bin/env python3
"""SV external-corpus RULE-COVERAGE instrument (SV-CORPUS-GRAD.7).

Measures which grammar rules the vendored external corpus actually exercises,
per profile, against the grammar's own per-profile satisfiability inventory —
the corpus-sufficiency axis of the graduation bar
([[project_sv_corpus_100pct_lrm_coverage_mandate]]: the corpus must exercise
100% of the parseable LRM surface, MEASURED; an uncovered rule = a corpus gap).

Numerator  — the union of PER-FILE COMMITTED rule sets over every corpus file
             the parser ACCEPTS, from the generated parser's transactional
             coverage testimony (`parseability_probe --parse ...
             --dump-rule-outcome-counts-json`: `rule_committed_counts` — sound
             under PEG backtracking; failed parses contribute NO testimony).
Denominator — the grammar's rule inventory with per-profile satisfiability
             (`ast_pipeline <grammar>.ebnf --dump-rule-profiles`: the same
             transitive `derive_rule_profiles` computation the profile-orphan
             lint gates on).

Classification per rule (for the run profile P):
  covered      satisfiable under P and fired in >=1 accepted corpus file
  GAP          satisfiable under P but fired in NO accepted file  <- the .9 worklist
  na_profile   not satisfiable under P (its home profiles listed) — measured
               under its own profile's run, never counted against this one

Deterministic: sorted outputs, content-derived only; the TSV + report are
tracked and diffable across sessions.

Usage:
  python3 stimuli/sv/corpus_rule_coverage.py \
      [--profile sv_2017] [--jobs 8] \
      [--results stimuli/sv/characterization/results.tsv] \
      [--rule-profiles <path.json>]   # default: regenerated via ast_pipeline \
      [--out-prefix stimuli/sv/characterization/rule_coverage]
"""

import argparse
import json
import os
import subprocess
import sys
import tempfile
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent  # repo root


def repo_relative(p) -> str:
    """The repo-root-relative spelling of a results.tsv path, whatever it arrived as.

    ⛔ Column 3 of results.tsv has been repo-root-relative since CORPUS-GRAD-ALL.2.1;
    before that it was ABSOLUTE. A bare `Path(p).relative_to(ROOT)` therefore raises
    `ValueError: … is not in the subpath of …` on every current artifact — measured in
    SV-CORPUS-GRAD.7c, where it killed the run at the report stage AFTER all 9 694 files
    had been probed. This is the THIRD consumer of that column found to assume the old
    spelling (adjudicate_external_corpus.py in .10, cluster_rejects_valid.py routed as
    .11b), so the convention is normalized here at ONE site rather than patched per
    call-site, and both spellings are accepted forever.
    """
    path = Path(p)
    if not path.is_absolute():
        return str(path)
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def probe_committed_rules(probe: str, grammar: str, profile: str, path: str,
                          timeout_s: int):
    """Run one coverage-instrumented parse.

    Returns (status, committed-rule set); status in {"accepted", "rejected",
    "timeout"}. The timeout lane exists because the transactional coverage
    stack can be >100x slower than the plain parse on pathological-
    backtracking inputs (measured: Surelog `ExponTimeIfElseGen/dut.sv` —
    plain 0.28 s, instrumented >30 s); such files are EXCLUDED WITH CAUSE
    from the testimony, never silently dropped
    ([[feedback_dont_run_jobs_that_hit_known_pathological_inputs]])."""
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tf:
        out = tf.name
    try:
        rc = subprocess.run(
            [probe, "--parse", grammar, path, "--profile", profile,
             "--dump-rule-outcome-counts-json", out],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            timeout=timeout_s,
        ).returncode
        try:
            with open(out, encoding="utf-8") as fh:
                d = json.load(fh)
        except (OSError, json.JSONDecodeError):
            return ("rejected", frozenset())
        committed = frozenset(
            r for r, n in d.get("rule_committed_counts", {}).items() if n > 0)
        ok = rc == 0 and d.get("accepted") is True
        return ("accepted" if ok else "rejected", committed)
    except subprocess.TimeoutExpired:
        return ("timeout", frozenset())
    finally:
        try:
            os.unlink(out)
        except OSError:
            pass


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--profile", default="sv_2017")
    ap.add_argument("--grammar", default="systemverilog")
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--results",
                    default=ROOT / "stimuli/sv/characterization/results.tsv",
                    type=Path)
    ap.add_argument("--rule-profiles", type=Path, default=None,
                    help="pre-dumped rule-profiles JSON; default regenerates "
                         "via ast_pipeline --dump-rule-profiles")
    ap.add_argument("--out-prefix",
                    default=ROOT / "stimuli/sv/characterization/rule_coverage",
                    type=Path)
    ap.add_argument("--max-files", type=int, default=0, help="0 = no cap (smoke aid)")
    ap.add_argument("--per-file-timeout", type=int, default=45,
                    help="seconds per instrumented parse (the coverage stack "
                         "is far slower than a plain parse on pathological-"
                         "backtracking inputs)")
    args = ap.parse_args()

    probe = os.environ.get(
        "PGEN_PARSE_PROBE_BIN", str(ROOT / "rust/target/debug/parseability_probe"))
    if not os.access(probe, os.X_OK):
        sys.exit(f"parseability_probe not executable at {probe}")

    # --- denominator: the per-profile rule inventory
    if args.rule_profiles is None:
        rp_path = Path(tempfile.mkstemp(suffix=".json")[1])
        ast_pipeline = os.environ.get(
            "PGEN_AST_PIPELINE_BIN", str(ROOT / "rust/target/debug/ast_pipeline"))
        subprocess.run(
            [ast_pipeline, str(ROOT / f"grammars/{args.grammar}.ebnf"),
             "--dump-rule-profiles", str(rp_path)],
            check=True, stdout=subprocess.DEVNULL)
    else:
        rp_path = args.rule_profiles
    with open(rp_path, encoding="utf-8") as fh:
        inventory = json.load(fh)
    rules = inventory["rules"]

    # --- numerator: union committed testimony over the ACCEPTED corpus rows
    pass_rows = []
    for line in args.results.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        suite, observed, path = line.split("\t")
        if observed == "pass":
            pass_rows.append((suite, path))
    pass_rows.sort()
    if args.max_files:
        pass_rows = pass_rows[: args.max_files]

    fired_file_count = Counter()
    per_suite_fired = defaultdict(set)
    accepted_files = 0
    disagreements = []
    timeouts = []

    def work(row):
        suite, path = row
        return (suite, path, *probe_committed_rules(
            probe, args.grammar, args.profile, path, args.per_file_timeout))

    with ThreadPoolExecutor(max_workers=args.jobs) as ex:
        for i, (suite, path, status, committed) in enumerate(
                ex.map(work, pass_rows), 1):
            if status == "timeout":
                # excluded-with-cause: instrumentation blowup, not a parse fact
                timeouts.append(path)
                continue
            if status != "accepted":
                # results.tsv said pass; the instrumented run disagreed — surface it.
                disagreements.append(path)
                continue
            accepted_files += 1
            for r in committed:
                fired_file_count[r] += 1
            per_suite_fired[suite].update(committed)
            if i % 2000 == 0:
                print(f"  ... {i}/{len(pass_rows)} files", file=sys.stderr)

    fired = set(fired_file_count)
    unknown_fired = sorted(fired - set(rules))
    covered, gaps, na_profile = [], [], []
    for rule in sorted(rules):
        sat = args.profile in rules[rule]["satisfiable_under"]
        if sat and rule in fired:
            covered.append(rule)
        elif sat:
            gaps.append(rule)
        else:
            na_profile.append(rule)

    denom = len(covered) + len(gaps)
    pct = 100.0 * len(covered) / denom if denom else 0.0

    # --- tracked TSV (per-rule, machine-diffable)
    tsv_path = Path(f"{args.out_prefix}_{args.profile}.tsv")
    with tsv_path.open("w", encoding="utf-8") as fh:
        fh.write("rule\tstatus\tfired_file_count\tsatisfiable_under\n")
        for rule in sorted(rules):
            sat_list = ",".join(rules[rule]["satisfiable_under"])
            if rule in fired and args.profile in rules[rule]["satisfiable_under"]:
                status = "covered"
            elif args.profile in rules[rule]["satisfiable_under"]:
                status = "GAP"
            else:
                status = "na_profile"
            fh.write(f"{rule}\t{status}\t{fired_file_count.get(rule, 0)}\t{sat_list}\n")

    # --- tracked report
    md_path = Path(f"{args.out_prefix}_{args.profile}.md")
    thin = [(r, fired_file_count[r]) for r in covered if fired_file_count[r] <= 3]
    thin.sort(key=lambda x: (x[1], x[0]))
    lines = [
        f"# SV external-corpus rule coverage — profile `{args.profile}` (SV-CORPUS-GRAD.7)",
        "",
        f"Generated by `stimuli/sv/corpus_rule_coverage.py` (deterministic; numerator = "
        f"committed-rule testimony unioned over the {accepted_files} accepted corpus files "
        f"of `results.tsv`; denominator = `--dump-rule-profiles` satisfiability inventory, "
        f"{inventory['rule_count']} rules).",
        "",
        "| metric | value |",
        "|---|---|",
        f"| rules satisfiable under `{args.profile}` | {denom} |",
        f"| covered (fired in >=1 accepted file) | {len(covered)} |",
        f"| **UNCOVERED (corpus GAPS — the `.9` worklist)** | **{len(gaps)}** |",
        f"| **measured coverage** | **{pct:.1f}%** |",
        f"| na_profile (not satisfiable under `{args.profile}`) | {len(na_profile)} |",
        f"| accepted files contributing testimony | {accepted_files} |",
        "",
        "## Uncovered rules (satisfiable under the profile, fired by ZERO accepted corpus files)",
        "",
    ]
    for rule in gaps:
        lines.append(f"- `{rule}`")
    lines += [
        "",
        "## Thin coverage (covered by <=3 files — fragility watchlist, informational)",
        "",
        "| rule | files |",
        "|---|---|",
    ]
    for rule, n in thin:
        lines.append(f"| `{rule}` | {n} |")
    lines += [
        "",
        "## Per-suite fired-rule contribution",
        "",
        "| suite | distinct rules fired |",
        "|---|---|",
    ]
    for suite in sorted(per_suite_fired):
        lines.append(f"| {suite} | {len(per_suite_fired[suite])} |")
    if unknown_fired:
        lines += ["", f"⚠️ fired rules ABSENT from the inventory ({len(unknown_fired)}): "
                  + ", ".join(f"`{r}`" for r in unknown_fired)]
    fired_na = [r for r in na_profile if fired_file_count.get(r, 0) > 0]
    if fired_na:
        lines += ["", f"⚠️ rules that FIRED despite being classified not-satisfiable under "
                  f"`{args.profile}` ({len(fired_na)} — an inventory-vs-reality "
                  "contradiction; root-cause before trusting the na_profile lane): "
                  + ", ".join(f"`{r}`" for r in fired_na)]
    if timeouts:
        lines += ["", "## Excluded with cause: coverage-instrumentation timeouts",
                  "",
                  f"{len(timeouts)} pass-file(s) exceeded the {args.per_file_timeout} s "
                  "instrumented-parse timeout (the transactional coverage stack is "
                  ">100x slower than a plain parse on pathological-backtracking "
                  "inputs — measured on Surelog `ExponTimeIfElseGen`). Their "
                  "testimony is EXCLUDED, named here:", ""]
        for p in timeouts:
            lines.append(f"- `{repo_relative(p)}`")
    if disagreements:
        lines += ["", f"⚠️ results.tsv pass rows the instrumented run did NOT accept "
                  f"({len(disagreements)} — investigate before trusting the union):"]
        for p in disagreements[:20]:
            lines.append(f"- `{repo_relative(p)}`")
    lines.append("")
    md_path.write_text("\n".join(lines), encoding="utf-8")

    print(f"coverage[{args.profile}]: {len(covered)}/{denom} = {pct:.1f}% "
          f"({len(gaps)} gaps, {len(na_profile)} na_profile, "
          f"{accepted_files} files, {len(timeouts)} timeout-excluded, "
          f"{len(disagreements)} disagreements)")
    print(f"report:  {md_path}")
    print(f"per-rule: {tsv_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
