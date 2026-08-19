#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2s — price and regress candidate spellings of the starving-star fix.

⛔ WHY. `.13c.2r` fixed the same mechanism in `n_output_gate_instance` and its lesson is recorded:
*an isolation proves a mechanism EXISTS; it does not bound where the mechanism APPLIES.* And
`.13c.2q`'s lesson is that a VERDICT cannot see a construct parsing through the wrong production.
So each candidate arm is run over the FULL pinned reproducer manifest, on every profile each row
declares, and BOTH the verdict and the typed AST are compared against the HEAD grammar.

Usage:
    python3 docs/tasks/artifacts/sv_corpus_grad/systf_clocking_event/sweep_arms.py ARM0.ebnf ARM.ebnf [...]

The first grammar is the BASELINE; every later one is compared against it.
Exit 0 = measured. Exit 1 = a tool failed.
"""
from __future__ import annotations

import csv
import hashlib
import os
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

# FIVE directories up from this FILE: systf_clocking_event -> sv_corpus_grad -> artifacts ->
# tasks -> docs -> root. The guard below is not decoration: this script was written with
# `parents[4]` and the guard is what named the error instead of a silent empty sweep.
ROOT = Path(__file__).resolve().parents[5]
REPROS = ROOT / "stimuli/sv/adjudication_repros"
PIPELINE = ROOT / "rust/target/debug/ast_pipeline"
WORK = ROOT / "tmp/systf_arm_sweep"
REJECTED = "<rejected>"
JOBS = int(os.environ.get("PGEN_ARM_SWEEP_JOBS", "8"))

if not (ROOT / "stimuli/sv/adjudication_repros/MANIFEST.tsv").is_file():
    raise SystemExit(f"⛔ ROOT resolved to {ROOT}, which holds no reproducer manifest — a sweep that "
                     "cannot find its inputs must SAY SO, not report a clean tree")


def observe(grammar: Path, sample: Path, profile: str) -> str:
    with tempfile.TemporaryDirectory(dir=str(WORK)) as workdir:
        ast = Path(workdir) / "ast.json"
        proc = subprocess.run(
            [str(PIPELINE), str(grammar), "--interpret-parse", str(sample),
             "--grammar-profile", profile, "--interpret-parse-ast-json", str(ast)],
            capture_output=True, text=True, timeout=300)
        if "INTERPRET-PARSE:" not in proc.stdout:
            raise SystemExit(f"⛔ no verdict for {sample.name} @ {grammar.name}\n{proc.stdout}{proc.stderr}")
        if "accepted=true" not in proc.stdout:
            return REJECTED
        return hashlib.sha256(ast.read_bytes()).hexdigest()[:16]


def rows() -> list[tuple[str, str]]:
    out = []
    with (REPROS / "MANIFEST.tsv").open(encoding="utf-8") as fh:
        for row in csv.DictReader(fh, delimiter="\t"):
            spec = (row.get("profiles") or "").strip()
            for p in ([x.strip() for x in spec.split(",") if x.strip()] or ["sv_2017"]):
                out.append((row["id"], p))
    return out


def sweep(grammar: Path, pairs) -> dict:
    with ThreadPoolExecutor(max_workers=JOBS) as pool:
        got = list(pool.map(lambda p: observe(grammar, REPROS / p[0], p[1]), pairs))
    return dict(zip(pairs, got))


def main() -> int:
    arms = [Path(a) for a in sys.argv[1:]]
    if len(arms) < 2:
        raise SystemExit("usage: sweep_arms.py BASELINE.ebnf ARM.ebnf [ARM2.ebnf ...]")
    WORK.mkdir(parents=True, exist_ok=True)
    pairs = rows()
    print(f"ARM-SWEEP: baseline={arms[0].name} arms={len(arms)-1} repro_checks={len(pairs)} jobs={JOBS}")
    base = sweep(arms[0], pairs)
    for arm in arms[1:]:
        got = sweep(arm, pairs)
        widen = [p for p in pairs if base[p] == REJECTED and got[p] != REJECTED]
        narrow = [p for p in pairs if base[p] != REJECTED and got[p] == REJECTED]
        shape = [p for p in pairs if REJECTED not in (base[p], got[p]) and base[p] != got[p]]
        print(f"  {arm.name:<14} widen={len(widen):<3} narrow={len(narrow):<3} shape={len(shape):<3}")
        for label, group in (("WIDEN", widen), ("NARROW", narrow), ("AST-SHAPE", shape)):
            for rid, profile in group:
                print(f"      {label:<10} {rid} [{profile}]")
    return 0


if __name__ == "__main__":
    sys.exit(main())
