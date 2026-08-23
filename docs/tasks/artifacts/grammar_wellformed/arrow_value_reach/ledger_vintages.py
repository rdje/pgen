#!/usr/bin/env python3
"""GRAMMAR-WELLFORMED.H.16.6e — ACCEPT-SET LEDGER across grammar VINTAGES, by name.

usage: ledger_vintages.py <corpus.txt> <control.ebnf> <arm.ebnf> [<arm.ebnf> ...] [--json=OUT]

A net reject count hides direction (H.16.6c). This prints, per arm versus the control, every
row NEWLY ACCEPTED (WIDEN) and every row NEWLY REJECTED (NARROW) — so a vintage that fixes
25 and breaks 1 can never read as "only widens".

⛔ REFUSES when the control accepts nothing (a grammar that fails to load rejects every row
   and would read as a maximal NARROW rather than as a broken harness).
"""
import concurrent.futures, json, pathlib, subprocess, sys


def repo_root(start: pathlib.Path) -> pathlib.Path:
    for p in [start, *start.parents]:
        if (p / ".git").exists():
            return p
    sys.exit("ledger_vintages: REFUSED — no .git ancestor; cannot locate the repo root")


ROOT = repo_root(pathlib.Path(__file__).resolve())
AST = ROOT / "rust/target/debug/ast_pipeline"
if not AST.exists():
    sys.exit(f"ledger_vintages: REFUSED — no ast_pipeline at {AST}")

pos = [a for a in sys.argv[1:] if not a.startswith("--")]
OUT = next((pathlib.Path(a.split("=", 1)[1]) for a in sys.argv[1:] if a.startswith("--json=")), None)
CORPUS, CONTROL = pathlib.Path(pos[0]), pathlib.Path(pos[1])
ARMS = [pathlib.Path(p) for p in pos[2:]]

INDIR = ROOT / "rust/target/h1666e" / f"in_{CORPUS.stem}"
INDIR.mkdir(parents=True, exist_ok=True)
lines = [ln for ln in CORPUS.read_text(errors="replace").split("\n") if ln.strip()]
files = []
for i, ln in enumerate(lines):
    f = INDIR / f"s_{i:05d}.txt"
    if not f.exists() or f.read_text(errors="replace") != ln:
        f.write_text(ln)
    files.append(f)


def run(g, f):
    r = subprocess.run([str(AST), str(g), "--interpret-parse", str(f)], capture_output=True, text=True)
    if "INTERPRET-PARSE:" not in r.stdout:
        return None
    return "accepted=true" in r.stdout


res = {}
for g in [CONTROL, *ARMS]:
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as ex:
        res[g.name] = list(ex.map(lambda f: run(g, f), files))
    v = res[g.name]
    print(f"scored {g.name:<24} accepted={sum(1 for x in v if x is True):<6} "
          f"rejected={sum(1 for x in v if x is False):<6} no_verdict={sum(1 for x in v if x is None)}")

ctrl = res[CONTROL.name]
if sum(1 for x in ctrl if x is True) == 0:
    sys.exit("ledger_vintages: REFUSED — the CONTROL accepts nothing; the harness is broken")

ledger = {}
print()
for g in ARMS:
    v = res[g.name]
    widen = [[i, lines[i]] for i in range(len(lines)) if ctrl[i] is False and v[i] is True]
    narrow = [[i, lines[i]] for i in range(len(lines)) if ctrl[i] is True and v[i] is False]
    ledger[g.name] = {"widen": widen, "narrow": narrow}
    print(f"ACCEPT-SET-LEDGER: control={CONTROL.name} arm={g.name} widen={len(widen)} narrow={len(narrow)}")
    for i, t in narrow:
        print(f"    NARROW #{i:05d} {t}")
    for i, t in widen:
        print(f"    WIDEN  #{i:05d} {t}")
    print()

if OUT:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps({"corpus": CORPUS.name, "n": len(lines),
                               "control": CONTROL.name, "ledger": ledger}, indent=2))
    print(f"full ledger -> {OUT}")
