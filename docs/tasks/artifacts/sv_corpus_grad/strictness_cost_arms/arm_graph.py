#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2k (c) — freeze ONE arm's grammar REFERENCE GRAPH beside its measurement.

⛔ WHY THIS EXISTS — A DEFECT CAUGHT IN FLIGHT, IN THIS SLICE. `containment.py` first read the
reference graph from `generated/systemverilog.json`. That file is a FLOATING BUILD ARTIFACT: the
arm driver regenerates it for every arm, so the first containment run for the `t_only` arm was
computed against the `designB` grammar's graph — the same class of error as
`ENGINE-UNIVERSAL-SERVICES.20` slice 4 (a probe built from one arm, measured as another) and
`.24`'s answer to it. The number happened to be right; the METHOD could not have known that.
⇒ an arm's graph is frozen HERE, tied to that arm's own grammar sha, and `containment.py` refuses
when the two disagree.

⭐ IT NEVER TOUCHES THE TRACKED GRAMMAR. The arm transform is applied in memory and written to a
temporary file under `rust/target/` which the shared derivation is pointed at, so this can run at
any time, on a clean tree, without a regeneration.

⭐ THE DERIVATION ITSELF MOVED OUT (`SV-CORPUS-GRAD.13c.2w`), because `PARSE-COST-RATCHET`'s coded
containment invariant reads the same graph and a second spelling of it would be
[[one-metric-name-two-predicates-is-a-contract-defect]]. ⚠️ MEASURED rather than assumed: `head`,
`t_only` and `designB` all re-derive BYTE-IDENTICALLY to their tracked `*.graph.json` through the
shared path.

USAGE  python3 …/arm_graph.py --arm {head,t_only,designB,designA} [--outdir DIR]
       (the graph derivation itself lives in `scripts/parse_cost_containment.py`)
EXIT   0 = graph written · 2 = refused
"""

from __future__ import annotations

import argparse
import hashlib
import os
import importlib.util
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
if not (ROOT / "grammars").is_dir():
    raise SystemExit(f"arm_graph: not at the repo root (derived {ROOT}) — fix the parents[] depth")
HERE = ROOT / "docs/tasks/artifacts/sv_corpus_grad/strictness_cost_arms"
GRAMMAR = ROOT / "grammars/systemverilog.ebnf"

# ⭐ ONE DERIVATION, SHARED (`SV-CORPUS-GRAD.13c.2w`). The rule-reference graph and the containment
# predicate over it now live in `scripts/parse_cost_containment.py`, because `PARSE-COST-RATCHET`'s
# coded invariant reads exactly the same two things. This file used to invoke the EBNF frontend and
# walk `raw_ast` itself; a second spelling of a predicate is
# [[one-metric-name-two-predicates-is-a-contract-defect]], measured in this repository on
# `unreachable_rules`. ⚠️ VERIFIED, not assumed: the tracked `*.graph.json` files re-derive
# byte-identically through the shared path.
sys.path.insert(0, str(ROOT / "scripts"))
import parse_cost_containment as PCC  # noqa: E402


def load_apply_arm():
    spec = importlib.util.spec_from_file_location("apply_arm", HERE / "apply_arm.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


# ⛔⛔ THE ARM BASE IS A PINNED COMMIT, NOT `HEAD`, AND THAT IS THE WHOLE POINT.
# These arms are transforms of the PRE-FIX grammar. Reading `HEAD` worked exactly until the fix
# landed — the moment `PGEN-SV-CORPUS-GRAD-0237` committed, every anchor moved and the harness
# refused ("anchor for .13c.2t occurs 0 times"), i.e. **a tracked reproduction script that could
# no longer reproduce the thing it documents.** It failed loudly rather than silently, which is
# the only reason it was noticed; a base that drifts with HEAD is not a base.
# ⇒ pinned. To add an arm on a LATER base, add a new constant — never repoint this one, because
# the measurements in `*.json` beside this file are deltas against THIS grammar.
ARM_BASE_COMMIT = "b532b540"   # PGEN-SV-CORPUS-GRAD-0236 — the commit before the fix landed


def head_grammar_text() -> str:
    """The ARM BASE grammar text, read from GIT at a PINNED commit — never from the working tree.

    ⛔ It read the working tree until `-0236`, and the arm driver calls this derivation while the
    ARM is still applied, so the transform was attempted on top of itself and refused, taking down
    a measurement whose 3-minute build had already succeeded. Then it read `HEAD`, which broke the
    day the fix landed. An arm's identity must depend on neither the working tree nor the branch.
    """
    r = subprocess.run(["git", "show", f"{ARM_BASE_COMMIT}:grammars/systemverilog.ebnf"],
                       cwd=ROOT, capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit(f"arm_graph: cannot read the pinned arm base {ARM_BASE_COMMIT} "
                         f"— is this a shallow clone? git says: {r.stderr.strip()}")
    return r.stdout


def arm_text(arm: str) -> str:
    """The arm's grammar TEXT, built in memory from HEAD's — never written over the tracked file."""
    text = head_grammar_text()
    if arm == "head":
        return text
    m = load_apply_arm()
    text = m.replace_once(text, m.T_BEFORE, m.T_AFTER, ".13c.2t")
    if arm == "designB":
        text = m.replace_once(text, m.B_BEFORE, m.B_AFTER, ".13c.2k design B (identifier)")
        text = m.replace_once(text, m.B_GUARD_BEFORE, m.B_GUARD_AFTER, ".13c.2k design B (alias)")
    elif arm == "designA":
        text, _n = m.rewrite_call_sites(text)
    elif arm == "designC":
        text, _n = m.rewrite_call_sites(text)
        text = m.replace_once(text, m.B_GUARD_BEFORE, m.C_GUARD_AFTER, ".13c.2k design C")
    return text


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--arm", required=True, choices=("head", "t_only", "designB", "designA", "designC"))
    ap.add_argument("--outdir", default=str(HERE))
    a = ap.parse_args()

    text = arm_text(a.arm)
    sha = hashlib.sha256(text.encode("utf-8")).hexdigest()
    # ⛔ ON-VOLUME BY POLICY (CLAUDE.md §13) AND REPO-ROOT-RELATIVE, because the shared derivation
    # takes a path relative to the root — which is also what makes it path-agnostic.
    with tempfile.TemporaryDirectory(dir=str(ROOT / "rust" / "target")) as td:
        g = Path(td) / "arm.ebnf"
        g.write_text(text, encoding="utf-8")
        try:
            raw = PCC.raw_ast_of(str(g.relative_to(ROOT)))
        except PCC.Refused as exc:
            print(f"arm_graph: {exc}", file=sys.stderr)
            return 2
    edges = PCC.edges_from_raw_ast(raw)
    out = Path(a.outdir) / f"{a.arm}.graph.json"
    out.write_text(json.dumps({"arm": a.arm, "grammar_sha256": sha, "rules": len(edges),
                               "edges": edges}, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    print(f"ARM-GRAPH: arm={a.arm} rules={len(edges)} grammar_sha256={sha}")
    # ⚠️ PRE-EXISTING, found by this slice's own verification run (`SV-CORPUS-GRAD.13c.2w`): a
    # RELATIVE `--outdir` made `Path.relative_to(ROOT)` raise AFTER the graph had been written
    # correctly, so a SUCCESSFUL derivation exited non-zero on its last line. `os.path.relpath`
    # resolves both forms.
    print(f"  -> {os.path.relpath(out, ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
