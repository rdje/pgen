#!/usr/bin/env python3
"""GRAMMAR-WELLFORMED.H.16.6e — score ONE corpus against ONE grammar via the interpreter.

usage: score_corpus.py <grammar.ebnf> <corpus.txt> <label> [--json OUT.json]

⛔ REFUSES if the grammar accepts nothing (a load error rejects every row and would read as
   a maximal NARROW rather than as a broken harness).
⛔ The repo root is found by walking up to the `.git` directory, never by a fixed
   `parents[N]` index — the sibling `residual_self_reject/score_pair.py` pins `parents[3]`
   for a file that sits five levels down and dies on `FileNotFoundError` before reaching
   its own refusal check, which is the shape H.16.6c's red-control note warns about.
"""
import concurrent.futures, json, pathlib, subprocess, sys


def repo_root(start: pathlib.Path) -> pathlib.Path:
    for p in [start, *start.parents]:
        if (p / ".git").exists():
            return p
    sys.exit("score_corpus: REFUSED — no .git ancestor; cannot locate the repo root")


ROOT = repo_root(pathlib.Path(__file__).resolve())
AST = ROOT / "rust/target/debug/ast_pipeline"
if not AST.exists():
    sys.exit(f"score_corpus: REFUSED — no ast_pipeline at {AST}")

G, C, LABEL = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), sys.argv[3]
OUT = next((pathlib.Path(a.split("=", 1)[1]) for a in sys.argv[4:] if a.startswith("--json=")), None)
INDIR = ROOT / "rust/target/h1666e" / f"in_{C.stem}"
INDIR.mkdir(parents=True, exist_ok=True)

lines = [ln for ln in C.read_text(errors="replace").split("\n") if ln.strip()]
files = []
for i, ln in enumerate(lines):
    f = INDIR / f"s_{i:05d}.txt"
    if not f.exists() or f.read_text(errors="replace") != ln:
        f.write_text(ln)
    files.append(f)


def run(f):
    r = subprocess.run([str(AST), str(G), "--interpret-parse", str(f)],
                       capture_output=True, text=True)
    if "INTERPRET-PARSE:" not in r.stdout:
        return None
    return "accepted=true" in r.stdout


with concurrent.futures.ThreadPoolExecutor(max_workers=8) as ex:
    v = list(ex.map(run, files))

acc = sum(1 for x in v if x is True)
if acc == 0:
    sys.exit(f"score_corpus: REFUSED — {G.name} accepts NOTHING; the harness, not the grammar, is broken")
rej = [(i, lines[i]) for i, x in enumerate(v) if x is False]
nov = sum(1 for x in v if x is None)
print(f"SELF-REJECT: label={LABEL} grammar={G.name} corpus={C.name} n={len(lines)} "
      f"accepted={acc} rejected={len(rej)} no_verdict={nov}")
for i, ln in rej:
    print(f"  REJECTED #{i:05d} {ln}")
if OUT:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps({"label": LABEL, "grammar": G.name, "corpus": C.name,
                               "n": len(lines), "accepted": acc, "rejected": len(rej),
                               "no_verdict": nov, "rejects": rej}, indent=2))
    print(f"verdicts -> {OUT}")
