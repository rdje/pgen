#!/usr/bin/env python3
"""H.16.6d — verify the LANDED grammar: interpreter and SHIPPED parser must agree on every row.

⛔ Uses the DEBUG probe deliberately: the release probe on disk was built before this
   regeneration and is stale, which is exactly the two-vintage trap TOOLBOX §1.3 names.
"""
import concurrent.futures, pathlib, re, subprocess, sys
ROOT = pathlib.Path(__file__).resolve().parents[5]
AST = ROOT / "rust/target/debug/ast_pipeline"
PROBE = ROOT / "rust/target/debug/parseability_probe"
G = ROOT / "grammars/semantic_annotation.ebnf"
C = pathlib.Path(sys.argv[1])
IND = ROOT / "rust/target/h1666d" / f"vin_{C.stem}"
IND.mkdir(parents=True, exist_ok=True)
lines = [l for l in C.read_text(errors="replace").split("\n") if l.strip()]
files = []
for i, l in enumerate(lines):
    f = IND / f"s_{i:04d}.txt"
    if not f.exists() or f.read_text(errors="replace") != l:
        f.write_text(l)
    files.append(f)
def interp(f):
    r = subprocess.run([str(AST), str(G), "--interpret-parse", str(f)], capture_output=True, text=True)
    if "INTERPRET-PARSE:" not in r.stdout: return None
    return "accepted=true" in r.stdout
def ship(f):
    return subprocess.run([str(PROBE), "--parse", "semantic_annotation", str(f)],
                          capture_output=True, text=True).returncode == 0
with concurrent.futures.ThreadPoolExecutor(max_workers=8) as ex:
    a = list(ex.map(interp, files))
with concurrent.futures.ThreadPoolExecutor(max_workers=8) as ex:
    b = list(ex.map(ship, files))
dis = [(i, lines[i], a[i], b[i]) for i in range(len(lines)) if a[i] is not None and a[i] != b[i]]
print(f"TWO-ORACLE-CENSUS: corpus={C.name} n={len(lines)} interp_accept={sum(1 for x in a if x)} "
      f"shipped_accept={sum(1 for x in b if x)} disagreements={len(dis)}")
for i, t, x, y in dis[:20]:
    print(f"  ⛔ #{i:04d} interp={x} shipped={y} :: {t[:120]}")
sys.exit(1 if dis else 0)
