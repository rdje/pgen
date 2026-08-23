#!/usr/bin/env python3
"""GRAMMAR-WELLFORMED.H.16.6e — every verdict scored by BOTH oracles; hard error on disagreement.

This leaf's whole ruling is read off `--interpret-parse`, which reads the `.ebnf` directly. That
is the right oracle for a grammar question (it cannot be stale), but it is NOT the artifact
consumers have. Each corpus is therefore re-scored through the SHIPPED generated parser and any
disagreement is a hard failure.

⛔ Uses the DEBUG probe deliberately — the release probe on disk predates the H.16.6d
   regeneration and is stale, the two-vintage trap TOOLBOX §1.3 names. The debug probe's vintage
   is asserted here in BOTH directions before any row is scored.

usage: verify_two_oracles.py <corpus.txt> [<corpus.txt> ...]
"""
import concurrent.futures, pathlib, subprocess, sys


def repo_root(start: pathlib.Path) -> pathlib.Path:
    for p in [start, *start.parents]:
        if (p / ".git").exists():
            return p
    sys.exit("verify_two_oracles: REFUSED — no .git ancestor")


ROOT = repo_root(pathlib.Path(__file__).resolve())
AST = ROOT / "rust/target/debug/ast_pipeline"
PROBE = ROOT / "rust/target/debug/parseability_probe"
G = ROOT / "grammars/semantic_annotation.ebnf"
for p in (AST, PROBE, G):
    if not p.exists():
        sys.exit(f"verify_two_oracles: REFUSED — missing {p}")
IND = ROOT / "rust/target/h1666e/two_oracle"
IND.mkdir(parents=True, exist_ok=True)


def write(name, text):
    f = IND / name
    if not f.exists() or f.read_text(errors="replace") != text:
        f.write_text(text)
    return f


def interp(f):
    r = subprocess.run([str(AST), str(G), "--interpret-parse", str(f)], capture_output=True, text=True)
    if "INTERPRET-PARSE:" not in r.stdout:
        return None
    return "accepted=true" in r.stdout


def ship(f):
    return subprocess.run([str(PROBE), "--parse", "semantic_annotation", str(f)],
                          capture_output=True, text=True).returncode == 0


# ⛔ VINTAGE ASSERTION, both directions — a stale probe would agree with the interpreter on
# everything this leaf cares about and the census would read clean for the wrong reason.
vin_ok = ship(write("vintage_escaped.txt", r"@x: https://example.com/a\,b"))
vin_no = ship(write("vintage_bare.txt", "@x: https://example.com/a,b"))
if not (vin_ok and not vin_no):
    sys.exit(f"verify_two_oracles: REFUSED — probe vintage check failed "
             f"(escaped={vin_ok} want True, bare={vin_no} want False); rebuild parseability_probe")
print(f"probe vintage: escaped-comma URL accepted={vin_ok} bare-comma URL accepted={vin_no}  ⇒ post-H.16.6d")

rc = 0
for arg in sys.argv[1:]:
    C = pathlib.Path(arg)
    lines = [l for l in C.read_text(errors="replace").split("\n") if l.strip()]
    files = [write(f"{C.stem}_{i:05d}.txt", l) for i, l in enumerate(lines)]
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as ex:
        a = list(ex.map(interp, files))
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as ex:
        b = list(ex.map(ship, files))
    dis = [(i, lines[i], a[i], b[i]) for i in range(len(lines)) if a[i] is not None and a[i] != b[i]]
    print(f"TWO-ORACLE-CENSUS: corpus={C.name} n={len(lines)} "
          f"interp_accept={sum(1 for x in a if x)} shipped_accept={sum(1 for x in b if x)} "
          f"disagreements={len(dis)}")
    for i, t, x, y in dis[:20]:
        print(f"  ⛔ #{i:05d} interp={x} shipped={y} :: {t[:120]}")
    rc = rc or (1 if dis else 0)
sys.exit(rc)
