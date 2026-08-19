#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2k — what does the CODE GENERATOR's inline decision actually key on?

⛔⛔ WHY THIS EXISTS: IT REFUTED A PUBLISHED CLAIM OF MINE, UNDER DIRECTOR CHALLENGE.
`-0236`/`-0237` published that a rule's memo-hit rate turns on *"how many places call it"* and
illustrated it with counts of `inlined_frame_call(Self::RULE_X` sites in the GENERATED PARSER —
46 at HEAD, 341 for the bare-alias arm. Those counts are real, but they are EMITTED sites after
transitive expansion, not the decision, and the story built on them was wrong in two ways:

  * the generator's own census names the mechanism — `INLINE-DECISIONS … duplication_cap` — and it
    is REFERENCE COUNT × BODY SIZE against a budget, not reference count alone;
  * the bare-alias arm changed NEITHER rule's decision (both kept HEAD's), so its cost was never an
    inlining effect at all. It was redirected speculation, the `-0235` mechanism, full stop.

⇒ never infer a generator decision from emitted code when the generator will state it. `ast_pipeline
--report-fusibility-census` with `PGEN_FUSIBILITY_DUMP_ALL=1` prints, per inline-eligible rule,
`<class> refs=<n> body_nodes=<n> <INLINED|over-budget>`. That is the decision, from the decider.

MEASURED, over the four arms of one correctness fix (`identifier` / `non_keyword_identifier`):

  arm          identifier                     non_keyword_identifier
  pre-fix      refs=47 body=3  over-budget    refs=7  body=4  INLINED
  designB      refs=47 body=6  over-budget    refs=7  body=1  INLINED     <- neither moved
  designA      refs=1          INLINED        refs=52 body=7  over-budget
  designC      refs=1          INLINED        refs=52 body=8  over-budget

⭐ THE FIX MOVED 45 REFERENCES AND BOTH RULES CROSSED THE BUDGET, IN OPPOSITE DIRECTIONS. That is
why the memo boundary moved, and it is the honest form of the lesson: a rule's inline/memo status is
a property of its reference count (against a duplication budget), so an edit that moves references
between rules moves the very number any cost model was read from.

USAGE  python3 …/inline_decision.py [--rules R1,R2] [--arms a,b,…]
EXIT   0 = printed · 2 = refused (no ast_pipeline, or an arm cannot be built)
"""

from __future__ import annotations

import argparse
import importlib.util
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
if not (ROOT / "grammars").is_dir():
    raise SystemExit(f"inline_decision: not at the repo root (derived {ROOT}) — fix parents[]")
HERE = ROOT / "docs/tasks/artifacts/sv_corpus_grad/strictness_cost_arms"
PIPE = ROOT / "rust/target/debug/ast_pipeline"
ROW = r"^\s+{rule}: (\S+) refs=(\d+) body_nodes=(\d+) (.+)$"


def load_arm_graph():
    spec = importlib.util.spec_from_file_location("arm_graph", HERE / "arm_graph.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def census_for(text: str) -> str:
    """Run the fusibility census over one arm's grammar TEXT.

    ⛔ The temp file is named `systemverilog.ebnf` deliberately: the frontend derives
    `grammar_name` from the FILENAME, and a differently-named copy of the same grammar produces a
    different parser — measured, after a reconstruction check failed for exactly that reason.
    """
    with tempfile.TemporaryDirectory(dir=str(ROOT / "rust" / "target")) as td:
        g = Path(td) / "systemverilog.ebnf"
        g.write_text(text, encoding="utf-8")
        r = subprocess.run([str(PIPE), str(g), "--report-fusibility-census"],
                           capture_output=True, text=True, timeout=900,
                           env={**os.environ, "PGEN_FUSIBILITY_DUMP_ALL": "1"})
        return r.stdout


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--rules", default="identifier,non_keyword_identifier")
    ap.add_argument("--arms", default="head,t_only,designB,designA,designC")
    a = ap.parse_args()
    if not PIPE.exists():
        print(f"inline_decision: no ast_pipeline at {PIPE} — build it first", file=sys.stderr)
        return 2

    ag = load_arm_graph()
    rules = [r.strip() for r in a.rules.split(",") if r.strip()]
    print(f"INLINE-DECISION (arm base {ag.ARM_BASE_COMMIT}; the GENERATOR's own verdict, not the "
          f"emitted call sites)")
    print(f"  {'arm':<12} {'rule':<24} {'class':<14} {'refs':>5} {'body':>5}  decision")
    for arm in [x.strip() for x in a.arms.split(",") if x.strip()]:
        out = census_for(ag.arm_text(arm))
        budget = re.search(r"^INLINE-DECISIONS: .*$", out, re.M)
        for rule in rules:
            m = re.search(ROW.format(rule=re.escape(rule)), out, re.M)
            if m:
                print(f"  {arm:<12} {rule:<24} {m.group(1):<14} {m.group(2):>5} {m.group(3):>5}"
                      f"  {m.group(4)}")
            else:
                print(f"  {arm:<12} {rule:<24} not inline-eligible (or absent) — see the census's "
                      f"blocker histogram")
        if budget:
            print(f"  {'':<12} {budget.group(0)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
