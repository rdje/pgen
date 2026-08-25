#!/usr/bin/env python3
"""Census the IEEE-1800-only PUNCTUATION a `verilog_2005` parse can still reach — lens **L4**.

WHY THIS EXISTS (`SV-CORPUS-GRAD.13e.10` items (a) and (b))
-----------------------------------------------------------
`v2005_keyword_faithfulness_census.py` closes lens **L1** — the reserved KEYWORD — against IEEE
1364-2005 Annex B, which is a normative closed list.  It states, in its own header, that three
other lenses exist and that a clean L1 run is therefore not a clean bill of health:

    L1  KEYWORD ADMISSION   L2  POSITION   L3  SHAPE / CARDINALITY   L4  PUNCTUATION-ONLY

L4 was found by accident while fixing the first three: `{ << { a } }` — an IEEE 1800 streaming
concatenation — PARSED under `verilog_2005`, and **no keyword-level census can ever see it**,
because `<<` and `>>` are OPERATORS and introduce no keyword at all.  `.13e.7` gated that one
instance as a whole rule.  Nothing enumerated the rest of its class.  This instrument does.

⛔ THERE IS NO "ANNEX B FOR OPERATORS", WHICH IS WHY THIS IS A SET COMPARISON AND NOT A LOOKUP.
IEEE 1364-2005 reserves keywords in a normative list; it does not publish a closed operator list.
So the oracle here is the **alphabet of Annex A's own BNF** — every punctuation-only token that
appears on the right-hand side of a production.  A reachable terminal whose literal is not in that
alphabet is a candidate: the normative grammar of the standard never writes it.

⛔⛔ AND THE FIRST CUT OF THAT ORACLE WAS A SUBSTRING COUNT, WHICH FAILED IN THE SILENT DIRECTION.
Counting `annex_a.count("--")` reported the decrement operator PRESENT twice.  Both occurrences are
the **YAML front-matter delimiter** `---` on lines 1 and 9.  IEEE 1364-2005 has no decrement
operator, so the count produced a FALSE NEGATIVE on a real member of the population — and substring
counting can only err in that direction, which makes its zeros sound and its non-zeros worthless.
A set-vs-set comparison over a tokenized BNF has no such failure mode, and it recovers `--`.
⇒ **an instrument's census is only as good as its own tokenizer** (the operator-level twin of
`a-heading-census-is-only-as-good-as-the-heading-grammar`).

⛔⛔ THE TERMINAL EXTRACTOR WAS BLIND ON EXACTLY THE MEMBERS THIS LENS CARES ABOUT.
A rule is `name := trivia <literal> -> {…}`, so the obvious extractor splits the body on the
return-annotation arrow `->`.  A terminal whose LITERAL CONTAINS THAT ARROW is then truncated:
`iff_arrow := trivia "<->"` read as `"<`, and `implies := trivia "->"` read as `"`.  Both are
operators, which is the entire subject of L4 — the extractor was silent precisely where it
mattered, and the population read 66 instead of 68.  Parse the literal ANCHORED first, then require
what remains to be empty or an annotation.

WHAT THIS INSTRUMENT DOES *NOT* DO
-----------------------------------
It derives the CANDIDATE set — items (a) and (b) of the leaf.  It does not adjudicate.  A candidate
becomes a confirmed over-acceptance only under the probe legs, and ⛔ **leg 3 of the keyword census
does not transfer**: substituting the token for a fresh identifier is meaningless for punctuation.
The punctuation analogue is recorded in the leaf and is item (c): require the operator's own
terminal rule to COMMIT on the witness under `verilog_2005` (`--dump-rule-outcome-counts-json`,
TOOLBOX 3.5), which is what proves the ACCEPT was carried by the operator rather than by the
surrounding text parsing as something else.

Read-only.  Changes no byte of the grammar, the parser, or any tracked artifact.
"""
from __future__ import annotations

import importlib
import json
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve()
REPO_ROOT = HERE.parents[2]
sys.path.insert(0, str(HERE.parent))

# Reachability is REUSED from the L1 census rather than re-implemented, so the two lenses are
# commensurable by construction and a fix to one reach model fixes both.  That model is
# alternative-aware: an alternative whose mandatory members are not all satisfiable under the
# profile contributes no edges (SV-CORPUS-GRAD.13e.7(c), which measured the rule-level model
# reporting nine phantom candidates through `data_type`'s struct alternative).
L1 = importlib.import_module("v2005_keyword_faithfulness_census")

ANNEX_A = (REPO_ROOT / "docs" / "verilog" / "2005" / "md"
           / "section-Annex_A-normative-formal-syntax-definition.md")

# The characters a punctuation-only token may be built from.  Deliberately WIDE: a token is
# classified punctuation-only when EVERY character is in this set, so widening it can only ever
# admit more candidates for adjudication, never hide one.
PUNCT_CHARS = set("!#%&'()*+,-./:;<=>?@[]^{|}~\"`$\\")

RULE_HEAD = re.compile(r"^([a-z_][a-z_0-9]*)\s*:=\s*trivia\s+(.*)$")
STR_LIT = re.compile(r'^"((?:[^"\\]|\\.)*)"')
REGEX_LIT = re.compile(r"^/((?:[^/\\]|\\.)*)/")
ANNOTATION_TAIL = re.compile(r"^\s*(?:->.*)?$")


class Refused(Exception):
    """The instrument cannot inspect its subject.  Never a pass, never a silent zero."""


def _shown(path: pathlib.Path) -> str:
    """A repo-relative path for messages, and NEVER a raised exception.

    ⛔ `Path.relative_to` RAISES on a path outside the root, and the only place this is called is
    inside a REFUSAL message — the least-exercised branch in the file.  Measured while driving the
    red controls: the missing-oracle arm died with a `ValueError` traceback and exit **1** instead
    of the published refusal code **2**.  A check that tracebacks is indistinguishable from a check
    that is broken, so the formatting of a refusal must not be able to fail.
    """
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def terminal_literals() -> tuple[dict[str, str], dict[str, str], list[str]]:
    """`(string-literal terminals, regex terminals, `:= trivia …` lines that are NOT terminals)`."""
    literal: dict[str, str] = {}
    regexish: dict[str, str] = {}
    not_terminal: list[str] = []
    for line in L1.GRAMMAR.read_text().splitlines():
        head = RULE_HEAD.match(line)
        if not head:
            continue
        name, body = head.group(1), head.group(2)
        as_str = STR_LIT.match(body)
        if as_str and ANNOTATION_TAIL.match(body[as_str.end():]):
            literal[name] = as_str.group(1)
            continue
        as_rx = REGEX_LIT.match(body)
        if as_rx and ANNOTATION_TAIL.match(body[as_rx.end():]):
            regexish[name] = as_rx.group(1)
            continue
        not_terminal.append(name)
    if not literal:
        raise Refused("no string-literal terminal was extracted from the grammar — the rule shape "
                      "this instrument reads has changed, and a census that reads nothing must "
                      "refuse rather than report an empty candidate set")
    return literal, regexish, not_terminal


def annex_a_punctuation_alphabet() -> tuple[set[str], int, int]:
    """Every punctuation-only token on a right-hand side of the tracked Annex A BNF."""
    if not ANNEX_A.exists():
        raise Refused(f"{_shown(ANNEX_A)} is missing — the oracle is absent, so "
                      "this instrument has nothing to compare against")
    lines = ANNEX_A.read_text().splitlines()
    # ⛔ Drop the YAML front matter FIRST.  Its `---` delimiters are what made a substring counter
    # report the decrement operator as present in a standard that does not have it.
    if lines and lines[0].strip() == "---":
        closing = next((i for i in range(1, len(lines)) if lines[i].strip() == "---"), None)
        if closing is None:
            raise Refused("Annex A opens with `---` and never closes it — the front matter cannot "
                          "be bounded, and guessing where it ends is how the `--` false negative "
                          "was produced in the first place")
        lines = lines[closing + 1:]

    bnf: list[str] = []
    in_production = False
    for line in lines:
        if "::=" in line:
            in_production = True
            bnf.append(line)
        elif in_production and line.strip() and line[:1] in " \t":
            bnf.append(line)          # a continuation line of the current production
        elif not line.strip():
            continue                  # a blank line does not end a production
        else:
            in_production = False     # prose

    productions = sum("::=" in line for line in bnf)
    if productions < 300:
        raise Refused(f"only {productions} Annex A productions were recognized (expected the full "
                      "formal syntax, ~386) — the extraction no longer describes the document, and "
                      "a shrunken alphabet reports phantom candidates in the FLATTERING direction")

    alphabet: set[str] = set()
    for line in bnf:
        rhs = line.split("::=", 1)[1] if "::=" in line else line
        for token in rhs.split():
            if token and all(ch in PUNCT_CHARS for ch in token):
                alphabet.add(token)
    return alphabet, len(bnf), productions


def main() -> int:
    try:
        raw = L1.raw_ast()
        satisfiable = L1.rule_satisfiability()
        reachable, source_satisfiable, synthesised = L1.reachable_rules(raw, satisfiable)
        literal, regexish, not_terminal = terminal_literals()
        alphabet, bnf_lines, productions = annex_a_punctuation_alphabet()
    except (L1.Refused, Refused) as exc:
        print(f"v2005-punctuation-census: REFUSED — {exc}", file=sys.stderr)
        return 2

    non_keyword = {n: v for n, v in literal.items() if not n.startswith("kw_")}
    reachable_punct = {n: v for n, v in non_keyword.items() if n in reachable}
    candidates = {n: v for n, v in reachable_punct.items() if v not in alphabet}
    accounted = {n: v for n, v in reachable_punct.items() if v in alphabet}

    print(f"grammar:   rules reachable under {L1.PROFILE} = {len(reachable)} "
          f"(source-satisfiable {source_satisfiable}; {len(synthesised)} eliminator-synthesised "
          f"rules excluded as unadjudicable)")
    print(f"terminals: string-literal {len(literal)}, regex {len(regexish)}; "
          f"`:= trivia …` lines that are NOT terminals: {len(not_terminal)} {not_terminal}")
    print(f"           non-keyword string-literal terminals {len(non_keyword)}, "
          f"of which REACHABLE under {L1.PROFILE}: {len(reachable_punct)}")
    print(f"oracle:    IEEE 1364-2005 Annex A — {productions} productions over {bnf_lines} BNF "
          f"lines; punctuation-only alphabet = {len(alphabet)} tokens")
    print(f"VERDICT:   {len(accounted)} reachable punctuation terminals appear in the Annex A "
          f"alphabet; {len(candidates)} DO NOT and are L4 candidates")
    for name, lit in sorted(candidates.items(), key=lambda kv: (-len(kv[1]), kv[1])):
        print(f"  candidate  {name:34s} {lit!r}")
    print()
    print("⛔ A CANDIDATE IS NOT A DEFECT. Each owes the probe legs of `SV-CORPUS-GRAD.13e.10` (c), "
          "and leg 3 of the KEYWORD census does not transfer — identifier substitution is "
          "meaningless for punctuation. The punctuation leg 3 is: the operator's own terminal rule "
          "must COMMIT on the witness under verilog_2005 (--dump-rule-outcome-counts-json).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
