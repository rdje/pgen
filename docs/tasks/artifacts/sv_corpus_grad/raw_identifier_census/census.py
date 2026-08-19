#!/usr/bin/env python3
"""SV-CORPUS-GRAD.13c.2k (a) — the SWEEP: every site that spells the RAW `identifier` rule.

⛔ THE DEFECT CLASS. IEEE 1800-2017 A.9.3 writes

      identifier ::= simple_identifier | escaped_identifier

and §5.6.2 forbids a reserved word (Annex B) from being a `simple_identifier`. PGEN spells that
exclusion in a SEPARATE rule —

      non_keyword_identifier := !reserved_non_keyword_identifier identifier

— so every site that references the raw `identifier` rule instead is UNGUARDED and admits a
reserved keyword. `.13c.2k` measured three such over-acceptances in the hierarchy-component loops
(`ral.module[0].g()`, `ral.module.g()`, `module.g()`) against three correctly-guarded positions.

⭐ WHY AN INSTRUMENT AND NOT A HAND COUNT. The leaf opened claiming **43** non-comment uses. That
number came from a grep nobody re-ran, and it is wrong — a `grep -c` counts LINES, and five lines
in this grammar carry two or three references each. The same failure was measured one commit
earlier: `.13c.2m` published *"every `kw_n_<digits>` is a marker by construction"* from a hand
count that was wrong for 5 of 8 (`.13c.2p`). A census produced by a search nobody can re-run is
not a census (`docs/CLAIM_VERIFICATION.md` §1).

TWO INDEPENDENT SIGNALS, because a single parse of the grammar cannot check itself:

  A  the FRONTEND's own structured view, `generated/systemverilog.json` → `raw_ast`, where a
     reference is a literal `["rule_reference", "identifier"]` token. This is what the code
     generator consumed, so it is the population that actually shipped.
  B  a TEXT scan of `grammars/systemverilog.ebnf` that strips `#`/`//` comments, `/…/` regex
     bodies, `"…"` strings and `-> …` return directives, then matches the bare word.

⛔ B exists to catch A being stale (the JSON is a build artifact — regenerate the grammar and it
moves), and A exists to catch B's stripping being wrong. They must agree on the TOTAL and on the
PER-RULE breakdown; a disagreement is a hard error, never a preference for one of them.

CLASSIFICATION — the axis that decides the fix, from the grammar's shape, not from reading prose:

  guard     the reference INSIDE `non_keyword_identifier` — the exclusion itself, not a site.
  negation  the reference sits under a `!` operator, so guarding `identifier` INVERTS there and
            makes the site strictly MORE permissive. ⛔ This is the one class where the fix
            widens rather than narrows, so it must be enumerated before the fix is written.
  alias     the rule body is exactly `identifier` (Annex A's `X_identifier ::= identifier`), so
            the site inherits whatever `identifier` admits.
  inline    a reference inside a longer body — the hierarchy-component loops live here.

USAGE   python3 docs/tasks/artifacts/sv_corpus_grad/raw_identifier_census/census.py
EXIT    0 = the two signals agree and the census printed
        2 = it could not run correctly (signals disagree, or an empty population)
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
GRAMMAR = ROOT / "grammars/systemverilog.ebnf"
FRONTEND_JSON = ROOT / "generated/systemverilog.json"

# ⛔ A ROOT GUARD, because this exact mistake has already been made once in this directory: the
# `accepts_invalid_class` probe shipped one level short and its own guard is what caught it
# (`.13c.2k`). An unguarded `parents[N]` fails as a confusing FileNotFoundError deep in a helper.
if not (GRAMMAR.exists() and (ROOT / "grammars").is_dir()):
    raise SystemExit(f"census: not at the repo root (derived {ROOT}) — fix the parents[] depth")
TARGET = "identifier"
GUARD_RULE = "non_keyword_identifier"


# ── signal A — the frontend's own structured view ───────────────────────────────────────────────
def signal_a() -> dict[str, list[str]]:
    """{rule_name: [classification per reference, in source order]} from `raw_ast`."""
    raw = json.loads(FRONTEND_JSON.read_text(encoding="utf-8"))["raw_ast"]
    out: dict[str, list[str]] = {}
    for rule in raw:
        name = rule[0][1]
        # A reference is "negated" when the token immediately before it is `!`. The frontend
        # emits `!` as a standalone operator token, so this is positional, not textual.
        refs: list[str] = []
        body = rule[1:]
        for i, tok in enumerate(body):
            if tok[0] != "rule_reference" or tok[1] != TARGET:
                continue
            prev = body[i - 1] if i else None
            negated = prev is not None and prev[0] == "operator" and prev[1] == "!"
            # `!( identifier )` — the `!` sits before the group_open, not before the reference.
            if not negated and prev is not None and prev[0] == "group_open":
                prev2 = body[i - 2] if i >= 2 else None
                negated = prev2 is not None and prev2[0] == "operator" and prev2[1] == "!"
            if name == GUARD_RULE:
                refs.append("guard")
            elif negated:
                refs.append("negation")
            else:
                refs.append("alias" if _is_alias(body) else "inline")
        if refs:
            out[name] = refs
    return out


def _is_alias(body: list) -> bool:
    """True when the whole rule body is the single reference `identifier` (+ a return directive)."""
    meaningful = [t for t in body if t[0] not in ("semantic_annotation", "semantic_annotation_inline")]
    meaningful = [t for t in meaningful if not t[0].startswith("return_")]
    return len(meaningful) == 1 and meaningful[0] == ["rule_reference", TARGET]


# ── signal B — an independent text scan, with line numbers signal A cannot give ──────────────────
RULE_HEAD = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\s*:=")
WORD = re.compile(rf"(?<![A-Za-z0-9_]){TARGET}(?![A-Za-z0-9_])")


def _strip(line: str) -> str:
    """Remove everything a reference cannot legally live in: comments, regexes, strings, returns."""
    line = re.sub(r"//.*$", "", line)
    line = re.sub(r"(?<![A-Za-z0-9_])#.*$", "", line)
    line = re.sub(r'"(?:[^"\\]|\\.)*"', '""', line)
    line = re.sub(r"/(?:[^/\\]|\\.)*/", "//", line)          # `/…regex…/` bodies
    line = re.sub(r"->.*$", "", line)                        # return directives
    return line


def signal_b() -> tuple[dict[str, list[str]], list[tuple[int, str, str]]]:
    """({rule_name: [classification…]}, [(lineno, rule, text)…]) from the grammar TEXT."""
    out: dict[str, list[str]] = {}
    sites: list[tuple[int, str, str]] = []
    current = None
    in_comment_block = False
    for lineno, raw_line in enumerate(GRAMMAR.read_text(encoding="utf-8").splitlines(), 1):
        stripped = raw_line.lstrip()
        if stripped.startswith("#") or stripped.startswith("//"):
            in_comment_block = True
            continue
        in_comment_block = False
        head = RULE_HEAD.match(raw_line)
        if head:
            current = head.group(1)
        body = _strip(raw_line)
        if head:
            body = body[head.end():]
        hits = WORD.findall(body)
        if not hits or current is None:
            continue
        # Classify each hit by its immediate textual context, independently of signal A.
        for match in WORD.finditer(body):
            before = body[: match.start()]
            negated = bool(re.search(r"!\s*\(?\s*$", before))
            if current == GUARD_RULE:
                kind = "guard"
            elif negated:
                kind = "negation"
            elif _text_is_alias(current):
                kind = "alias"
            else:
                kind = "inline"
            out.setdefault(current, []).append(kind)
            sites.append((lineno, current, raw_line.rstrip()))
    assert not in_comment_block
    return out, sites


def _text_is_alias(rule: str) -> bool:
    """`rule := identifier` on one line with nothing else — checked against the raw text."""
    pattern = re.compile(rf"^{re.escape(rule)}\s*:=\s*{TARGET}\s*$")
    return any(pattern.match(l) for l in GRAMMAR.read_text(encoding="utf-8").splitlines())


def main() -> int:
    a = signal_a()
    b, sites = signal_b()

    total_a = sum(len(v) for v in a.values())
    total_b = sum(len(v) for v in b.values())
    if total_a == 0 or total_b == 0:
        print(f"census: EMPTY population (A={total_a}, B={total_b}) — refusing", file=sys.stderr)
        return 2

    disagreements = []
    for rule in sorted(set(a) | set(b)):
        if sorted(a.get(rule, [])) != sorted(b.get(rule, [])):
            disagreements.append((rule, a.get(rule, []), b.get(rule, [])))
    if disagreements:
        print("census: THE TWO SIGNALS DISAGREE — one of them is wrong, refusing to pick", file=sys.stderr)
        for rule, av, bv in disagreements:
            print(f"  {rule}: frontend-json={av} grammar-text={bv}", file=sys.stderr)
        return 2

    by_kind: dict[str, list[tuple[int, str]]] = {}
    for (lineno, rule, _text), kind in zip(sites, _kinds_in_site_order(b, sites)):
        by_kind.setdefault(kind, []).append((lineno, rule))

    print(f"RAW-IDENTIFIER-CENSUS: refs={total_a} rules={len(a)} "
          + " ".join(f"{k}={len(v)}" for k, v in sorted(by_kind.items()))
          + "  (two signals agree)")
    print()
    for kind in ("guard", "negation", "inline", "alias"):
        entries = by_kind.get(kind, [])
        if not entries:
            continue
        print(f"── {kind} ({len(entries)}) " + "─" * (60 - len(kind)))
        for lineno, rule in entries:
            print(f"  {GRAMMAR.relative_to(ROOT)}:{lineno}  {rule}")
        print()
    return 0


def _kinds_in_site_order(b: dict[str, list[str]], sites: list[tuple[int, str, str]]) -> list[str]:
    """Re-walk `b`'s per-rule classification lists in the order the sites were emitted."""
    cursor: dict[str, int] = {}
    out = []
    for _lineno, rule, _text in sites:
        i = cursor.get(rule, 0)
        out.append(b[rule][i])
        cursor[rule] = i + 1
    return out


if __name__ == "__main__":
    sys.exit(main())
