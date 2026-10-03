#!/usr/bin/env python3
"""SV-CORPUS-GRAD.41 / .44 — how often does the corpus WRITE the constructs the big simulators tolerate and
neither IEEE 1800-2017 nor 1800-2023 derives?

A TEXT scan (comments stripped, strings kept), not a parse: it answers "who writes this spelling", which is
what a ruling needs to be priced. It says nothing about whether a file parses.

  prototype_lifetime   `extern … function|task automatic|static` / `pure virtual … function|task automatic|static`
  cross_dotted         a `cross` item list that names a dotted (hierarchical) name
  foreach_selected     `foreach ( name[…][…] )` — a bracket pair before the loop-variable bracket
  solve_in_foreach     a `solve … before` whose nearest enclosing `{` was opened by `foreach ( … )`
  binsof_dotted        `binsof ( a.b.c… )` — a name of THREE or more parts (`.44`; a two-part name is
                       `cover_point_identifier . bin_identifier`, 19.6.1 Syntax 19-4, and is not counted)

    python3 docs/tasks/artifacts/sv_corpus_grad/dialect_constructs/scan.py            # one summary line
    python3 docs/tasks/artifacts/sv_corpus_grad/dialect_constructs/scan.py --files    # + every file, per construct
    python3 docs/tasks/artifacts/sv_corpus_grad/dialect_constructs/scan.py --self-test
"""
from __future__ import annotations

import collections
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
TREES = ("stimuli/sv/subs", "stimuli/sv/uvm")
EXT = (".sv", ".svh", ".v")

PROTO = re.compile(r"\b(?:extern|pure\s+virtual)(?:\s+(?:virtual|protected|local|static))*\s+(?:function|task)\s+(?:automatic|static)\b")
CROSS = re.compile(r"\bcross\s+([^;{]*)")
DOTTED = re.compile(r"[A-Za-z_0-9\]]\s*\.\s*[A-Za-z_]")
FOREACH_SEL = re.compile(r"\bforeach\s*\(\s*[A-Za-z_][A-Za-z0-9_.:$]*\s*(?:\[[^\],]*\]\s*)+\[")
SOLVE = re.compile(r"\bsolve\b[^;]*\bbefore\b")
OPENER = re.compile(r"\b(foreach|if|constraint)\b[^{};]*$")
BINSOF_DOTTED = re.compile(r"\bbinsof\s*\(\s*[A-Za-z_][A-Za-z0-9_$]*(?:\s*\.\s*[A-Za-z_][A-Za-z0-9_$]*){2,}")


def strip_comments(text: str) -> str:
    """`//` and `/* */` comments replaced by spaces; string literals kept (a `//` inside one is the string's)."""
    out, i, n = [], 0, len(text)
    while i < n:
        c = text[i]
        if c == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            out.append(text[i:j + 1]); i = j + 1
        elif text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            out.append(" " * (j - i)); i = j
        elif text.startswith("/*", i):
            j = text.find("*/", i + 2)
            j = n if j < 0 else j + 2
            out.append(re.sub(r"[^\n]", " ", text[i:j])); i = j
        else:
            out.append(c); i += 1
    return "".join(out)


def cross_items(text: str) -> list[str]:
    """The item list of each `cross`, cut before a trailing `iff ( … )` (its expression may be dotted, legally)."""
    return [re.split(r"\biff\b", m.group(1))[0] for m in CROSS.finditer(text)]


def solve_openers(text: str) -> list[str]:
    """For each `solve … before`, the keyword that opened its nearest enclosing `{` (`foreach`, `if`, `constraint`, `other`)."""
    kinds = []
    for m in SOLVE.finditer(text):
        depth, i, opener = 0, m.start() - 1, None
        while i >= 0:
            c = text[i]
            if c == "}":
                depth += 1
            elif c == "{":
                if depth == 0:
                    opener = i
                    break
                depth -= 1
            i -= 1
        if opener is None:
            kinds.append("other")
            continue
        k = OPENER.search(text[max(0, opener - 240):opener])
        kinds.append(k.group(1) if k else "other")
    return kinds


def scan_text(text: str) -> dict[str, int]:
    t = strip_comments(text)
    return {
        "prototype_lifetime": len(PROTO.findall(t)),
        "cross_dotted": sum(1 for items in cross_items(t) if DOTTED.search(items)),
        "foreach_selected": len(FOREACH_SEL.findall(t)),
        "solve_in_foreach": sum(1 for k in solve_openers(t) if k == "foreach"),
        "binsof_dotted": len(BINSOF_DOTTED.findall(t)),
    }


def suite_of(rel: str) -> str:
    parts = rel.split("/")
    return "uvm-core" if parts[2] == "uvm" else parts[3]


def self_test() -> int:
    fails = []

    def check(name, cond):
        print(f"  {'PASS' if cond else 'FAIL'}  {name}")
        if not cond:
            fails.append(name)

    check("a prototype lifetime is counted, both spellings", scan_text("pure virtual task automatic t();\nextern protected function automatic int f();")["prototype_lifetime"] == 2)
    check("a prototype WITHOUT a lifetime is not", scan_text("pure virtual task t();\nextern function int f();")["prototype_lifetime"] == 0)
    check("a commented-out prototype is not (verilator's t_class_virtual_pure.v)", scan_text("// pure virtual task automatic fin();\n/* extern task static t(); */")["prototype_lifetime"] == 0)
    check("a dotted cross item is counted", scan_text("xc: cross cp_a, cfg.vif.key0_in;")["cross_dotted"] == 1)
    check("a plain cross is not, and a dotted `iff` guard is not an item", scan_text("xc: cross a, b iff (cfg.en) { }\nyc: cross a, b;")["cross_dotted"] == 0)
    check("`cross … with option.weight` in a comment is not", scan_text("// cross with option.weight=2\n")["cross_dotted"] == 0)
    check("a foreach over a selected array is counted", scan_text("foreach (prv_read[i][j]) begin end\nforeach(this.foo.x[i][j]) ;")["foreach_selected"] == 2)
    check("the LRM spellings are not", scan_text("foreach (a[i, j]) ;\nforeach (a.b[i]) ;\nforeach (a[i]) x[i][j] = 0;")["foreach_selected"] == 0)
    check("a solve inside a foreach constraint set is counted", scan_text("constraint k { foreach (x[i]) { solve y before x[i]; } }")["solve_in_foreach"] == 1)
    check("a block-level solve is not, even after a closed foreach", scan_text("constraint k { foreach (x[i]) { x[i] > 0; } solve y before x; }")["solve_in_foreach"] == 0)
    check("a `//` inside a string does not start a comment", strip_comments('s = "a // b"; // c') == 's = "a // b";     ')
    check("a three-part name inside binsof is counted, spaces or not", scan_text("bins h = binsof(cfg.vif.a) intersect {1};\nbins k = binsof( cs_registers_i . mstatus_q . mpp );")["binsof_dotted"] == 2)
    check("a coverage point, and a coverage point bin, are not (19.6.1 Syntax 19-4)", scan_text("bins h = binsof(cp_a) intersect {1};\nbins k = binsof(cp_a.one);\nbins j = !binsof(cp_b.two) && binsof(cp_c);")["binsof_dotted"] == 0)
    print(f"DIALECT-CONSTRUCTS-SELFTEST: arms=13 failed={len(fails)}")
    return 1 if fails else 0


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()
    statements: dict[str, collections.Counter] = collections.defaultdict(collections.Counter)
    files: dict[str, list[str]] = collections.defaultdict(list)
    scanned = 0
    for tree in TREES:
        for p in sorted((ROOT / tree).rglob("*")):
            # the lanes' own population: regular files only (`find -type f`), so a symlinked twin is not counted twice
            if p.is_symlink() or not p.is_file() or p.suffix not in EXT:
                continue
            scanned += 1
            rel = p.relative_to(ROOT).as_posix()
            for k, n in scan_text(p.read_text(encoding="utf-8", errors="replace")).items():
                if n:
                    statements[k][suite_of(rel)] += n
                    files[k].append(rel)
    parts = []
    for k in ("prototype_lifetime", "cross_dotted", "foreach_selected", "solve_in_foreach", "binsof_dotted"):
        by_suite = collections.Counter(suite_of(f) for f in files[k])
        parts.append(f"{k}: files={len(files[k])} statements={sum(statements[k].values())} ["
                     + " ".join(f"{s}={by_suite[s]}/{statements[k][s]}" for s in sorted(by_suite)) + "]")
    print(f"DIALECT-CONSTRUCTS: scanned={scanned} | " + " | ".join(parts) + "  (suite=files/statements)")
    if "--files" in sys.argv:
        for k in ("prototype_lifetime", "cross_dotted", "foreach_selected", "solve_in_foreach", "binsof_dotted"):
            for f in files[k]:
                print(f"  {k}\t{f}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
