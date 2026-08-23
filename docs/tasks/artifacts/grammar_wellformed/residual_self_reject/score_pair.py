#!/usr/bin/env python3
"""H.16.6c — score ONE corpus against ONE grammar via the interpreter, naming every reject.

usage: score_pair.py <grammar.ebnf> <corpus.txt> <label>
⛔ REFUSES if the grammar accepts nothing (a load error rejects everything and reads as a
   maximal narrow rather than as an error).
"""
import concurrent.futures, pathlib, subprocess, sys


def repo_root(start: pathlib.Path) -> pathlib.Path:
    """H.16.6e: `parents[3]` resolved to `docs/tasks` for a file five levels down, so this
    script died on `FileNotFoundError` BEFORE reaching its own accepts-nothing refusal —
    the shape H.16.6c's red-control note warns about. Walk to `.git` instead."""
    for q in [start, *start.parents]:
        if (q / ".git").exists():
            return q
    sys.exit("score_pair: REFUSED — no .git ancestor; cannot locate the repo root")


ROOT = repo_root(pathlib.Path(__file__).resolve())
AST = ROOT / "rust/target/debug/ast_pipeline"
if not AST.exists():
    sys.exit(f"score_pair: REFUSED — no ast_pipeline at {AST}")
G, C, LABEL = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), sys.argv[3]
INDIR = ROOT / "rust/target/h1666c" / f"in_{C.stem}"
INDIR.mkdir(parents=True, exist_ok=True)

lines = [ln for ln in C.read_text(errors="replace").split("\n") if ln.strip()]
items = []
for i, ln in enumerate(lines):
    f = INDIR / f"s_{i:04d}.txt"
    if not f.exists() or f.read_text(errors="replace") != ln:
        f.write_text(ln)
    items.append((i, ln, f))

def run(f):
    r = subprocess.run([str(AST), str(G), "--interpret-parse", str(f)],
                       capture_output=True, text=True)
    if "INTERPRET-PARSE:" not in r.stdout:
        return None
    return "accepted=true" in r.stdout

with concurrent.futures.ThreadPoolExecutor(max_workers=8) as ex:
    v = list(ex.map(lambda it: run(it[2]), items))

acc = sum(1 for x in v if x is True)
if acc == 0:
    sys.exit(f"score_pair: REFUSED — {G.name} accepts NOTHING; the harness, not the grammar, is broken")
rej = [(i, lines[i]) for i, x in enumerate(v) if x is False]
nov = sum(1 for x in v if x is None)
print(f"SELF-REJECT: label={LABEL} grammar={G.name} corpus={C.name} n={len(items)} "
      f"accepted={acc} rejected={len(rej)} no_verdict={nov}")
for i, ln in rej:
    print(f"  REJECTED #{i:04d} {ln}")
