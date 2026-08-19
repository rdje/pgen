#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2l — WHICH consumer-visible parser changes shipped since the SV downstream
contract was last written, WHICH commit made each one, and in WHICH of the two directions?

⛔ WHY THIS EXISTS. `.13c.2l` was opened because three accept-set changes had shipped with no
contract entry and no ledger row. It enumerated them BY HAND, from the session that noticed — and
the hand-kept table then went stale twice more (`-0232` and `-0233` landed and were never added, so
`-0237` was routed here as "the FIFTH change" when it was the SEVENTH commit and the FIFTH to move
a verdict). A hand-kept list of accept-set changes is the same object as the debt it tracks.
This derives it.

TWO AXES, BECAUSE A VERDICT IS NOT ENOUGH
-----------------------------------------
⛔ The first cut of this instrument measured VERDICTS only and reported `-0233` as "no verdict
moved (comment-only)" — which is exactly the blind spot `run_adjudication_repros.py` grew its `arm`
column for. `bufif0 g(o,i,e);` PARSED before that fix and PARSES after it; what changed is that it
now parses as a `gate_instantiation` instead of a `udp_instantiation`. A consumer reading the tree
sees a different tree for the same text, with no error to warn it. So both axes are measured:

  * ACCEPT-SET — the verdict moved. WIDEN (REJECT→ACCEPT) or NARROW (ACCEPT→REJECT).
  * AST-SHAPE  — the verdict did NOT move and the typed AST did. This is the SCHEMA axis: a
    previously-emitted shape was REPLACED, which is the contract's own stated trigger for a schema
    bump (`SV-0053`'s schema-`21` note) as against the additive `SV-0052` class.

WHAT IT MEASURES
----------------
The BASE is DERIVED, never typed: the commit that last wrote
`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`. Every commit touching
`grammars/systemverilog.ebnf` after it is a candidate. Each candidate's grammar is materialised
from git and every pinned reproducer in `stimuli/sv/adjudication_repros/MANIFEST.tsv` is parsed
against it, on every profile that row declares. A candidate that moves neither axis is
comment-only and owes no release bump (the `MEMO-STORE-SOUNDNESS.2` precedent).

⚠️ HONEST BOUND, STATED RATHER THAN IMPLIED. The population of INPUTS is the pinned manifest, not
the language. A change whose witness nothing has pinned is invisible here — measured: `-0220`
(IEEE 1800-2017 §19.6.1's cross-body function) widened the accept set and moved no manifest row,
because no row pinned it. That is a real gap in the manifest, not a property of the change, and it
is why `.13c.2l` pins that witness in the same commit that lands this. Read a `changes=0` verdict
as "no PINNED witness moved", never as "nothing changed".

THE ORACLE, AND WHY IT IS TRUSTED
---------------------------------
Verdicts and trees come from the grammar-AST INTERPRETER (`ast_pipeline --interpret-parse`,
TOOLBOX §1.5b), because a historical grammar cannot be measured on the shipped parser without a
~22-minute regeneration per commit. The interpreter is authoritative by VERIFICATION, not
construction, and its one measured hole is un-eliminated left recursion — `--lint-grammar` reports
`left_recursion_unhandled=0` for this grammar, so the hole does not apply.

⭐ That is an argument, so this script does not rely on it. It CHECKS it: at HEAD every row is also
parsed by the SHIPPED release probe, and any disagreement is a hard error. The interpreter is
trusted for the historical arms only because it is observed to agree with the shipped parser on the
one arm we can see.

Run:
    python3 docs/tasks/artifacts/sv_corpus_grad/contract_accept_set_ledger/measure_accept_set_transitions.py

Writes `accept_set_transitions.tsv` beside itself and prints a per-commit summary.
Exit 0 = measured. Exit 1 = the two oracles disagreed at HEAD, or a tool failed.
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

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
GRAMMAR = "grammars/systemverilog.ebnf"
CONTRACT = "docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md"
MANIFEST = ROOT / "stimuli/sv/adjudication_repros/MANIFEST.tsv"
REPROS = ROOT / "stimuli/sv/adjudication_repros"
PIPELINE = ROOT / "rust/target/debug/ast_pipeline"
PROBE = ROOT / "rust/target/release/parseability_probe"
WORK = ROOT / "tmp/contract_accept_set_ledger"
OUT = HERE / "accept_set_transitions.tsv"
DEFAULT_PROFILE = "sv_2017"
REJECTED = "<rejected>"
JOBS = int(os.environ.get("PGEN_ACCEPT_SET_JOBS", "8"))


def git(*args: str) -> str:
    return subprocess.run(["git", "-C", str(ROOT), *args],
                          capture_output=True, text=True, check=True).stdout.strip()


def materialise(rev: str, dest: Path) -> Path:
    """The grammar exactly as `rev` left it, written where the interpreter can load it."""
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(git("show", f"{rev}:{GRAMMAR}"), encoding="utf-8")
    return dest


def observe(grammar: Path, sample: Path, profile: str) -> str:
    """This grammar's verdict AND tree for one input, as one comparable token.

    `REJECTED` on a refusal; otherwise the sha-256 of the typed AST JSON, so an AST-SHAPE move is
    detectable without holding ~1500 trees in memory.
    """
    with tempfile.TemporaryDirectory(dir=str(WORK)) as workdir:
        ast = Path(workdir) / "ast.json"
        proc = subprocess.run(
            [str(PIPELINE), str(grammar), "--interpret-parse", str(sample),
             "--grammar-profile", profile, "--interpret-parse-ast-json", str(ast)],
            capture_output=True, text=True, timeout=300)
        if "INTERPRET-PARSE:" not in proc.stdout:
            raise SystemExit(f"⛔ interpreter produced no verdict for {sample.name} @ "
                             f"{grammar.name}:\n{proc.stdout}\n{proc.stderr}")
        if "accepted=true" not in proc.stdout:
            return REJECTED
        if not ast.exists():
            raise SystemExit(f"⛔ {sample.name} @ {grammar.name} accepted but wrote no AST")
        return hashlib.sha256(ast.read_bytes()).hexdigest()[:16]


def shipped(sample: Path, profile: str) -> bool:
    proc = subprocess.run(
        [str(PROBE), "--parse", "systemverilog", str(sample), "--profile", profile],
        capture_output=True, text=True, timeout=300)
    return proc.returncode == 0


def rows() -> list[tuple[str, str]]:
    """Every (reproducer, profile) pair the pinned manifest binds on."""
    out = []
    with MANIFEST.open(encoding="utf-8") as fh:
        for row in csv.DictReader(fh, delimiter="\t"):
            spec = (row.get("profiles") or "").strip()
            for profile in ([p.strip() for p in spec.split(",") if p.strip()]
                            or [DEFAULT_PROFILE]):
                out.append((row["id"], profile))
    return out


def sweep(grammar: Path, pairs: list[tuple[str, str]]) -> dict[tuple[str, str], str]:
    with ThreadPoolExecutor(max_workers=JOBS) as pool:
        got = list(pool.map(lambda p: observe(grammar, REPROS / p[0], p[1]), pairs))
    return dict(zip(pairs, got))


def classify(before: str, after: str) -> str:
    if before == after:
        return ""
    if before == REJECTED:
        return "WIDEN"
    if after == REJECTED:
        return "NARROW"
    return "AST-SHAPE"


def main() -> int:
    for tool in (PIPELINE, PROBE):
        if not tool.exists():
            raise SystemExit(f"⛔ missing {tool.relative_to(ROOT)} — build it first (TOOLBOX.md)")

    base = git("log", "-1", "--format=%H", "--", CONTRACT)
    candidates = git("rev-list", "--reverse", f"{base}..HEAD", "--", GRAMMAR).split()
    if not candidates:
        print("ACCEPT-SET-LEDGER: the contract is newer than every grammar change — nothing owed")
        return 0

    pairs = rows()
    print(f"ACCEPT-SET-LEDGER: base={base[:8]} candidates={len(candidates)} "
          f"repro_checks={len(pairs)} jobs={JOBS}")

    WORK.mkdir(parents=True, exist_ok=True)
    versions = [("base", base)] + [(c[:8], c) for c in candidates]
    grammars = {tag: materialise(rev, WORK / f"{tag}.ebnf") for tag, rev in versions}

    observed = {tag: sweep(grammars[tag], pairs) for tag, _ in versions}

    # Leg 2 — falsify the interpreter against an oracle it is not: the shipped release parser.
    head_tag = versions[-1][0]
    disagree = [p for p in pairs
                if (observed[head_tag][p] != REJECTED) != shipped(REPROS / p[0], p[1])]
    if disagree:
        for rid, profile in disagree:
            print(f"⛔ ORACLE DISAGREEMENT at HEAD: {rid} [{profile}]", file=sys.stderr)
        print("⛔ the interpreter does not reproduce the shipped parser on the arm we can see; "
              "its historical verdicts are therefore not quotable.", file=sys.stderr)
        return 1
    print(f"ACCEPT-SET-LEDGER: interpreter agrees with the shipped parser on {len(pairs)}/"
          f"{len(pairs)} verdicts at HEAD — historical arms are quotable")

    tags = [tag for tag, _ in versions]
    lines = []
    for index, (tag, rev) in enumerate(versions[1:], start=1):
        prev = tags[index - 1]
        subject = git("log", "-1", "--format=%s", rev).split(" (")[0]
        counts = {"WIDEN": 0, "NARROW": 0, "AST-SHAPE": 0}
        for rid, profile in pairs:
            before, after = observed[prev][(rid, profile)], observed[tag][(rid, profile)]
            kind = classify(before, after)
            if not kind:
                continue
            counts[kind] += 1
            lines.append({
                "commit": rev[:8], "slice": subject, "repro": rid, "profile": profile,
                "before": "REJECT" if before == REJECTED else f"ACCEPT:{before}",
                "after": "REJECT" if after == REJECTED else f"ACCEPT:{after}",
                "axis": "AST-SHAPE" if kind == "AST-SHAPE" else "ACCEPT-SET",
                "direction": kind,
            })
        moved = sum(counts.values())
        verdict = "CONSUMER-VISIBLE CHANGE" if moved else "no pinned witness moved"
        print(f"  {rev[:8]}  {subject:<28}  widen={counts['WIDEN']:<3} "
              f"narrow={counts['NARROW']:<3} shape={counts['AST-SHAPE']:<3} {verdict}")

    with OUT.open("w", encoding="utf-8", newline="") as fh:
        w = csv.DictWriter(fh, delimiter="\t",
                           fieldnames=["commit", "slice", "repro", "profile",
                                       "before", "after", "axis", "direction"])
        w.writeheader()
        w.writerows(lines)

    changed = len({line["commit"] for line in lines})
    shapes = len({line["commit"] for line in lines if line["axis"] == "AST-SHAPE"})
    print(f"ACCEPT-SET-LEDGER: consumer_visible_commits={changed}/{len(candidates)} "
          f"(of which move an AST SHAPE: {shapes}) transitions={len(lines)} "
          f"-> {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
