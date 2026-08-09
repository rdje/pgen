#!/usr/bin/env python3
"""Is any LIVE language missing today because of the column-0-comment truncation?

(`SV-CORPUS-GRAD.3.19` diagnosis → routed to `EBNF-FRONTEND-SILENT-TRUNCATION`.)

Once `frontend_truncation_probe.sh` establishes THAT the frontend can silently drop
alternatives, the only question that matters for the SV release is WHETHER IT ALREADY HAS.
Two censuses answer it from opposite directions:

  1. TEXTUAL, all grammars — find every column-0 `#` line sitting inside a rule body with a
     `|`/`->` continuation after it. This is the known truncating shape. Cheap; catches the
     defect anywhere in the repo, including grammars with no IR dump handy.

  2. STRUCTURAL, one grammar — compare the SOURCE `|`-alternative count against the IR
     alternative count, rule by rule, for every rule that carries an interior comment.
     Mechanism-agnostic: it would catch a drop even if the truncating shape were something
     other than what probe A/E found.

⚠️ HONEST BOUND on census 1: the scan is line-based, so a `#` inside a string or a regex
token on its own line could in principle be miscounted. That is precisely why census 2
exists and why it is the one quoted for the SV release — it reads the IR the generator
actually consumes, not the text.

⭐ CENSUS 2 IS DELIBERATELY ONE-SIDED, and that is the whole reason it can be trusted. Its
source counter counts `|` at LINE STARTS, so a same-line alternation
(`forward_type_keyword := kw_enum | kw_struct | kw_union`, `:5845`) under-counts: source 1,
IR 3. Reporting that as a finding would be crying wolf — an IR with MORE alternatives than
the counter saw is the counter being naive, never language being lost. So only
`source > IR` is a DEFECT; `source < IR` is reported separately as an instrument
limitation. The question this census exists to answer — "has the frontend silently dropped
an alternative?" — only ever produces `source > IR`. A two-sided version would need the
frontend's own tokenizer (see `EBNF-FRONTEND-SILENT-TRUNCATION.1`).

Usage (from anywhere; paths resolve against the repository root, directive 12):
  python3 docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/truncation_census.py
  python3 .../truncation_census.py --gen-ast rust/target/sv_config_use_probe/gen_ast_fixed.json
"""

import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _repo_root import repo_root  # noqa: E402

ROOT = repo_root()
RULEDEF = re.compile(r'^([A-Za-z_][A-Za-z0-9_]*)\s*:=')


def census_textual():
    """-> list of (grammar, line_no, the continuation line that would be lost)."""
    hits = []
    for g in sorted(ROOT.glob('grammars/**/*.ebnf')):
        lines = g.read_text(encoding='utf-8', errors='replace').split('\n')
        in_body = False
        for i, ln in enumerate(lines):
            stripped = ln.strip()
            if RULEDEF.match(ln):
                in_body = True
                continue
            if not in_body:
                continue
            if ln.startswith('#'):
                j = i + 1
                while j < len(lines) and (lines[j].startswith('#') or not lines[j].strip()):
                    j += 1
                if j < len(lines) and lines[j].strip().startswith(('|', '->')):
                    hits.append((str(g.relative_to(ROOT)), i + 1, lines[j].strip()[:70]))
                in_body = False
            elif stripped.startswith(('|', '->')) or (ln[:1] in ' \t' and stripped):
                continue
            else:
                in_body = False
    return hits


def census_structural(grammar_path: Path, gen_ast_path: Path):
    """-> list of (rule, source_alts, ir_alts) for every rule carrying an interior comment."""
    tree = json.loads(gen_ast_path.read_text())['grammar_tree']
    src = grammar_path.read_text(encoding='utf-8', errors='replace').split('\n')

    def source_alts(start):
        """Count `|` continuation lines, skipping comment and blank lines."""
        n, i = 1, start + 1
        while i < len(src):
            s = src[i].strip()
            if s.startswith('#') or not s:
                i += 1
                continue
            if s.startswith('|'):
                n += 1
                i += 1
                continue
            if s.startswith('->'):
                i += 1
                continue
            break
        return n

    def has_interior_comment(start):
        """A comment INSIDE the body — so the scan stops at a blank line, which ends it.

        Without the blank-line stop, the comment block introducing the NEXT rule counts as
        this rule's interior comment (measured on `forward_type_keyword:5845`).
        """
        i = start + 1
        while i < len(src):
            s = src[i].strip()
            if not s:
                return False
            if s.startswith('#'):
                return True
            if s.startswith(('|', '->')):
                i += 1
                continue
            break
        return False

    out = []
    for i, ln in enumerate(src):
        m = RULEDEF.match(ln)
        if not m or not has_interior_comment(i):
            continue
        name = m.group(1)
        node = tree.get(name)
        if node is None:
            continue
        ir = len(node['Or']['alternatives']) if 'Or' in node else 1
        out.append((name, source_alts(i), ir))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--grammar', type=Path, default=ROOT / 'grammars/systemverilog.ebnf')
    ap.add_argument('--gen-ast', type=Path,
                    default=ROOT / 'rust/target/sv_config_use_probe/gen_ast_fixed.json',
                    help="an ast_pipeline --dump-gen-ast JSON for --grammar")
    args = ap.parse_args()

    print("== census 1 (textual, ALL grammars): column-0 comments inside a rule body "
          "with a continuation after them")
    hits = census_textual()
    print(f"   truncating sites: {len(hits)}")
    for h in hits:
        print(f"     {h[0]}:{h[1]}  would lose: {h[2]}")

    print(f"\n== census 2 (structural, {args.grammar.relative_to(ROOT)}): source vs IR "
          f"alternative count, for every rule with an interior comment")
    if not args.gen_ast.is_file():
        print(f"   SKIPPED: no gen-AST dump at {args.gen_ast.relative_to(ROOT)} — produce one with\n"
              f"   ./rust/target/debug/ast_pipeline {args.grammar.relative_to(ROOT)} "
              f"--generate-stimuli --count 1 --seed 0 --dump-gen-ast <out>.json")
        return 1
    rows = census_structural(args.grammar, args.gen_ast)
    lost, undercount = [], []
    print(f"   {'rule':<52}{'source':>8}{'IR':>6}  verdict")
    for name, s, ir in rows:
        if s > ir:
            lost.append((name, s, ir))
            verdict = f"*** {s - ir} ALTERNATIVE(S) LOST ***"
        elif s < ir:
            undercount.append((name, s, ir))
            verdict = "line-counter under-count (same-line alternation) — not a loss"
        else:
            verdict = "OK"
        print(f"   {name:<52}{s:>8}{ir:>6}  {verdict}")
    print(f"\n   rules with an interior comment: {len(rows)}")
    print(f"   ALTERNATIVES LOST (source > IR): {len(lost)}"
          + (f" -> {[r[0] for r in lost]}" if lost else ""))
    print(f"   instrument under-counts (source < IR, one-sidedness by design): "
          f"{len(undercount)}" + (f" -> {[r[0] for r in undercount]}" if undercount else ""))
    return 0 if (not hits and not lost) else 1


if __name__ == "__main__":
    raise SystemExit(main())
