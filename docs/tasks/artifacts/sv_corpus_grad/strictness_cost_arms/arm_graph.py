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
temporary file which the EBNF frontend is pointed at directly, so this can run at any time, on a
clean tree, without a regeneration.

USAGE  python3 …/arm_graph.py --arm {head,t_only,designB,designA} [--outdir DIR]
EXIT   0 = graph written · 2 = refused
"""

from __future__ import annotations

import argparse
import hashlib
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
FRONTEND = ROOT / "rust/target/ebnf_frontend_build/debug/ast_pipeline"


def load_apply_arm():
    spec = importlib.util.spec_from_file_location("apply_arm", HERE / "apply_arm.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def head_grammar_text() -> str:
    """HEAD's grammar text, read from GIT — never from the working tree.

    ⛔ It read the working tree until this slice, and the arm driver calls the derivation while the
    ARM is still applied — so the transform was attempted on top of itself and refused
    ("anchor occurs 0 times"), taking down a measurement whose 3-minute build had already
    succeeded. An arm's identity must not depend on what is checked out at the moment it is asked.
    """
    return subprocess.run(["git", "show", "HEAD:grammars/systemverilog.ebnf"],
                          cwd=ROOT, capture_output=True, text=True, check=True).stdout


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
        text, _n = m.rewrite_call_sites(text, m.load_census())
    elif arm == "designC":
        text, _n = m.rewrite_call_sites(text, m.load_census())
        text = m.replace_once(text, m.B_GUARD_BEFORE, m.C_GUARD_AFTER, ".13c.2k design C")
    return text


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--arm", required=True, choices=("head", "t_only", "designB", "designA", "designC"))
    ap.add_argument("--outdir", default=str(HERE))
    a = ap.parse_args()

    if not FRONTEND.exists():
        print(f"arm_graph: no EBNF frontend at {FRONTEND} — build it first", file=sys.stderr)
        return 2

    text = arm_text(a.arm)
    sha = hashlib.sha256(text.encode("utf-8")).hexdigest()
    with tempfile.TemporaryDirectory(dir=str(ROOT / "rust" / "target")) as td:
        g = Path(td) / "arm.ebnf"
        j = Path(td) / "arm.json"
        g.write_text(text, encoding="utf-8")
        r = subprocess.run([str(FRONTEND), str(g), "--emit-raw-ast-json", str(j)],
                           capture_output=True, text=True, timeout=600)
        if r.returncode != 0 or not j.exists():
            print(f"arm_graph: the frontend refused arm {a.arm}:\n{r.stderr[-2000:]}", file=sys.stderr)
            return 2
        raw = json.loads(j.read_text(encoding="utf-8"))["raw_ast"]

    edges: dict[str, list[str]] = {}
    for rule in raw:
        name = rule[0][1]
        refs = sorted({t[1] for t in rule[1:] if t[0] == "rule_reference"})
        edges[name] = refs
    out = Path(a.outdir) / f"{a.arm}.graph.json"
    out.write_text(json.dumps({"arm": a.arm, "grammar_sha256": sha, "rules": len(edges),
                               "edges": edges}, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    print(f"ARM-GRAPH: arm={a.arm} rules={len(edges)} grammar_sha256={sha}")
    print(f"  -> {out.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
