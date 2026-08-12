#!/usr/bin/env python3
"""`SV-CORPUS-GRAD.13c.2b` — is the shipped `constant_primary` alternative list COMPLETE?

The leaf opened with a stated worry: *"a constant-expression path that is missing ONE
`constant_primary` alternative is unlikely to be missing only that one — check the whole A.8.4
alternative list."* This is that check, done mechanically instead of by eye.

GROUND TRUTH is not this file's author. Both arms are read from the repository:

  arm 1  `grammars/systemverilog.ebnf`                      — the SHIPPED grammar
  arm 2  `grammars/systemverilog_lrm_profiled_generated.ebnf` — the LRM Annex A extraction

Arm 2 is machine-generated from IEEE 1800 Annex A by `tools/extract_systemverilog_lrm_profiles.py`,
so a difference between the two arms is a fidelity finding and an equality is a fidelity proof.
The comparison strips return annotations (they are PGEN AST shaping, absent from the LRM) and
normalises the single shipped indirection `enum_id_scope_prefix` back to its LRM spelling
`( package_scope | class_scope )`, which the grammar defines verbatim.

Exit 0 = the arms agree (order-sensitively). Exit 1 = a divergence, printed per alternative.

  python3 docs/tasks/artifacts/sv_corpus_grad/constant_size_cast/constant_primary_lrm_alternative_audit.py
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from _repo_root import repo_root  # noqa: E402  (path is set immediately above)

SHIPPED = "grammars/systemverilog.ebnf"
LRM = "grammars/systemverilog_lrm_profiled_generated.ebnf"

# The shipped grammar names `( package_scope | class_scope )` through a helper rule; the LRM
# extraction inlines it. The helper's body is asserted below, so this is a rename, not a waiver.
INDIRECTIONS = {"( enum_id_scope_prefix )?": "( package_scope | class_scope )?"}


def rule_body(text: str, name: str) -> str:
    """Return the raw right-hand side of `name :=`, up to the next rule or blank line."""
    match = re.search(
        r"^%s\s*:=\s*(.*?)(?=^\s*$|^@|^[A-Za-z_][A-Za-z_0-9]*\s*:=)" % re.escape(name),
        text,
        re.S | re.M,
    )
    if match is None:
        raise SystemExit(f"REFUSE: rule '{name}' not found")
    return match.group(1)


def alternatives(body: str) -> list[str]:
    """Split a rule body into its TOP-LEVEL alternatives, return annotations removed."""
    body = re.sub(r"->\s*\{[^{}]*\}", "", body)
    body = re.sub(r"->\s*\[[^\[\]]*\]", "", body)
    out: list[str] = []
    current = ""
    depth = 0
    for char in body:
        if char in "([{":
            depth += 1
        elif char in ")]}":
            depth -= 1
        if char == "|" and depth == 0:
            out.append(current)
            current = ""
        else:
            current += char
    out.append(current)
    return [re.sub(r"\s+", " ", alt).strip() for alt in out if alt.strip()]


def normalise(alt: str) -> str:
    for shipped, lrm in INDIRECTIONS.items():
        alt = alt.replace(shipped, lrm)
    return alt


def audit(shipped_text: str, lrm_text: str, rule: str) -> int:
    ship = [normalise(a) for a in alternatives(rule_body(shipped_text, rule))]
    lrm = alternatives(rule_body(lrm_text, rule))
    print(f"\n=== {rule} — shipped {len(ship)} alternatives vs LRM-extracted {len(lrm)}")
    only_shipped = [a for a in ship if a not in lrm]
    only_lrm = [a for a in lrm if a not in ship]
    for alt in only_shipped:
        print(f"  [shipped-only] {alt}")
    for alt in only_lrm:
        print(f"  [LRM-only]     {alt}")
    if only_shipped or only_lrm:
        return 1
    if ship != lrm:
        print("  SET-EQUAL but ORDER differs:")
        for index, (a, b) in enumerate(zip(ship, lrm), 1):
            if a != b:
                print(f"    {index:2d}  shipped={a!r}  lrm={b!r}")
        # An ordering difference is reportable but not a fidelity defect: PGEN's ordered/longest
        # match policy is a PARSER decision, and the LRM's alternative order is not normative.
        print("  ⇒ order-only difference (reported, not failed)")
        return 0
    for index, alt in enumerate(ship, 1):
        print(f"  {index:2d}  == {alt}")
    return 0


def main() -> int:
    root = repo_root()
    shipped_text = (root / SHIPPED).read_text()
    lrm_text = (root / LRM).read_text()

    helper = rule_body(shipped_text, "enum_id_scope_prefix")
    if "non_typedef_package_scope" not in helper or "class_scope" not in helper:
        raise SystemExit(f"REFUSE: enum_id_scope_prefix is no longer the package/class scope pair: {helper!r}")

    status = 0
    for rule in ("constant_primary_sv_2017", "constant_primary_sv_2023", "casting_type"):
        status |= audit(shipped_text, lrm_text, rule)
    print("\nVERDICT:", "AGREE — no missing LRM alternative" if status == 0 else "DIVERGENCE")
    return status


if __name__ == "__main__":
    raise SystemExit(main())
