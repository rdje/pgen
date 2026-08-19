#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2r — the SWEEP: every `( … )*` that can STARVE the mandatory element after it.

⛔ THE DEFECT CLASS. PGEN's engine does not backtrack into a committed repetition, so

      out ( comma out )* comma inp

is unmatchable for EVERY input: the greedy star consumes the final `comma inp` (because `out` and
`inp` match the same texts) and the mandatory tail then has nothing left. Its mirror

      out comma inp ( comma inp )*

works, because the star is at the END and starves nothing. That is the whole mechanism, and it is
isolated on two 8-line scratch grammars beside this file (`star_starve.ebnf`, `star_tail.ebnf`)
driven by TOOLBOX 1.5b, so no claim here rests on PEG folklore:

    $ ast_pipeline star_starve.ebnf --interpret-parse s2.txt   # "(o, i)"  -> accepted=false
    $ ast_pipeline star_tail.ebnf   --interpret-parse s2.txt   # "(o, i)"  -> accepted=true
    $ ast_pipeline star_fixed.ebnf  --interpret-parse s2.txt   # "(o, i)"  -> accepted=true

⭐ WHY A CENSUS. `n_output_gate_instance` was found only because a DIFFERENT fix (`.13c.2k`)
regressed the corpus onto it. Nothing in the repository looks for this shape: `--lint-grammar`
reports `nullable_repetition` and `always_succeeds_alternatives`, neither of which is this. One
instance is never the class (`.13c.2e`), so the class gets an instrument.

THE SIGNAL. For each rule alternative, find a repetition group whose body's FIRST rule reference is
the same rule as the FIRST element of the mandatory sequence that follows the group. That is exactly
the `( comma X )* comma Y` shape. It is deliberately a CANDIDATE signal, not a verdict:

  * it can over-report — the star's body may still be unable to match the tail's later elements, in
    which case the star stops on its own and the rule is fine;
  * so every hit is adjudicated BY MEASUREMENT (parse a probe through the rule), never by reading.

⛔ IT IS NOT A SUFFICIENT SIGNAL EITHER, and this says so rather than implying completeness: a star
whose body starts with a DIFFERENT rule that happens to match the same text would starve just as
hard and this census would not see it. Widening that needs a first-set computation over the
grammar; until then the honest statement is "these are the hits of this signal", not "these are all
of them".

USAGE   python3 docs/tasks/artifacts/sv_corpus_grad/gate_route_unmasking/starving_star_census.py
EXIT    0 = the census ran; 2 = it could not run correctly
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
FRONTEND_JSON = ROOT / "generated/systemverilog.json"
GRAMMAR = ROOT / "grammars/systemverilog.ebnf"

if not (ROOT / "grammars").is_dir():
    raise SystemExit(f"census: not at the repo root (derived {ROOT}) — fix the parents[] depth")

# ⛔ THE POPULATION COMES FROM A BUILD ARTIFACT, SO STALENESS IS A LIE THIS CENSUS CAN TELL.
# Measured: the first run of this script reported `n_output_gate_instance` as a live candidate
# minutes AFTER its fix landed in the grammar, because `generated/systemverilog.json` had not been
# regenerated. A census that reads a build artifact must check the artifact is not behind its
# source, or it reports the previous parser's grammar as if it were this one.
if GRAMMAR.stat().st_mtime > FRONTEND_JSON.stat().st_mtime:
    raise SystemExit(
        f"census: {FRONTEND_JSON.relative_to(ROOT)} is OLDER than {GRAMMAR.relative_to(ROOT)} — "
        f"it describes a previous grammar. Regenerate first:\n"
        f"  make -C rust SHELL=/bin/bash focus_systemverilog")


def alternatives(body: list) -> list[list]:
    """Split a rule body on top-level `|` operators (those outside any group)."""
    out, cur, depth = [], [], 0
    for tok in body:
        if tok[0] == "group_open":
            depth += 1
        elif tok[0] == "group_close":
            depth -= 1
        if depth == 0 and tok[0] == "operator" and tok[1] == "|":
            out.append(cur)
            cur = []
            continue
        cur.append(tok)
    out.append(cur)
    return out


def scan(alt: list) -> list[tuple[str, str]]:
    """[(star_body_head_rule, following_head_rule)] for every starving-shaped site in one alternative."""
    hits = []
    i = 0
    while i < len(alt):
        if alt[i][0] != "group_open":
            i += 1
            continue
        depth, j = 1, i + 1
        while j < len(alt) and depth:
            if alt[j][0] == "group_open":
                depth += 1
            elif alt[j][0] == "group_close":
                depth -= 1
            j += 1
        # `j` is one past the matching group_close; the quantifier (if any) sits there.
        quant = alt[j] if j < len(alt) else None
        if quant is not None and quant[0] == "operator" and quant[1] in ("*", "+"):
            inner = alt[i + 1 : j - 1]
            after = alt[j + 1 :]
            head_in = next((t[1] for t in inner if t[0] == "rule_reference"), None)
            head_af = next((t[1] for t in after if t[0] in ("rule_reference", "regex", "quoted_string")), None)
            # A lookahead already guarding the star's tail is the FIX, not the defect.
            guarded = any(t[0] == "operator" and t[1] in ("&", "!") for t in inner)
            if head_in and head_af and head_in == head_af and not guarded:
                hits.append((head_in, head_af))
            i = j
            continue
        i = j
    return hits


def main() -> int:
    raw = json.loads(FRONTEND_JSON.read_text(encoding="utf-8"))["raw_ast"]
    if not raw:
        print("census: EMPTY grammar — refusing", file=sys.stderr)
        return 2

    text = GRAMMAR.read_text(encoding="utf-8").splitlines()
    findings: list[tuple[str, str, int]] = []
    for rule in raw:
        name = rule[0][1]
        for alt in alternatives(rule[1:]):
            for head_in, _head_af in scan(alt):
                lineno = next((n for n, l in enumerate(text, 1) if l.startswith(name + " :=")), 0)
                findings.append((name, head_in, lineno))

    print(f"STARVING-STAR-CENSUS: rules={len(raw)} candidates={len(findings)}")
    print()
    if findings:
        print(f"  {'rule':<44} {'starving head':<22} where")
        for name, head, lineno in sorted(findings):
            print(f"  {name:<44} {head:<22} grammars/systemverilog.ebnf:{lineno}")
        print()
        print("⛔ Each candidate needs a MEASURED verdict (parse a probe through the rule); the signal")
        print("   over-reports by design. A confirmed hit is unmatchable for every input.")
    else:
        print("  no candidate sites — every repetition whose body head repeats the following element")
        print("  already carries a `&`/`!` lookahead, or no such site exists.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
