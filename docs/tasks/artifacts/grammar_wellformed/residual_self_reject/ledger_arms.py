#!/usr/bin/env python3
"""GRAMMAR-WELLFORMED.H.16.6c — ACCEPT-SET-LEDGER for the delimiter-containment arms.

A net reject count hides direction. This prints, per arm versus the control, every input
NEWLY ACCEPTED (a WIDEN) and every input NEWLY REJECTED (a NARROW), by name — so an arm
that fixes 5 and breaks 7 can never read as "+2 worse" and stop the investigation there.

⛔ REFUSES when the control accepts nothing (an arm that fails to load rejects everything
   and would read as a maximal NARROW rather than as an error).
usage: ledger_arms.py <arms-dir> <corpus.txt> [--out DIR]
"""
import concurrent.futures, json, pathlib, subprocess, sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
AST = ROOT / "rust/target/debug/ast_pipeline"
args = [a for a in sys.argv[1:] if not a.startswith("--")]
ARMS, CORPUS = pathlib.Path(args[0]), pathlib.Path(args[1])
OUT = next((pathlib.Path(a.split("=", 1)[1]) for a in sys.argv[1:] if a.startswith("--out")), None)
INDIR = ROOT / "rust/target/h1666c" / f"in_{CORPUS.stem}"
INDIR.mkdir(parents=True, exist_ok=True)

if not AST.exists():
    sys.exit(f"ledger_arms: REFUSED — no ast_pipeline at {AST}")

lines = [ln for ln in CORPUS.read_text(errors="replace").split("\n") if ln.strip()]
files = []
for i, ln in enumerate(lines):
    f = INDIR / f"s_{i:04d}.txt"
    if not f.exists() or f.read_text(errors="replace") != ln:
        f.write_text(ln)
    files.append(f)


def run(g, f):
    r = subprocess.run([str(AST), str(g), "--interpret-parse", str(f)], capture_output=True, text=True)
    if "INTERPRET-PARSE:" not in r.stdout:
        return None
    return "accepted=true" in r.stdout


arms = sorted(ARMS.glob("arm*.ebnf"))
control = ARMS / "arm0_pristine.ebnf"
if control not in arms:
    sys.exit("ledger_arms: REFUSED — no arm0_pristine.ebnf control in the arms dir")

res = {}
for g in arms:
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as ex:
        res[g.name] = list(ex.map(lambda f: run(g, f), files))
    v = res[g.name]
    print(f"scored {g.name:<24} accepted={sum(1 for x in v if x is True):<5} "
          f"rejected={sum(1 for x in v if x is False):<5} no_verdict={sum(1 for x in v if x is None)}")

ctrl = res[control.name]
if sum(1 for x in ctrl if x is True) == 0:
    sys.exit("ledger_arms: REFUSED — the CONTROL accepts nothing; the harness is broken")

ledger = {}
print()
for g in arms:
    if g.name == control.name:
        continue
    v = res[g.name]
    widen = [(i, lines[i]) for i in range(len(lines)) if ctrl[i] is False and v[i] is True]
    narrow = [(i, lines[i]) for i in range(len(lines)) if ctrl[i] is True and v[i] is False]
    ledger[g.name] = {"widen": widen, "narrow": narrow}
    print(f"ACCEPT-SET-LEDGER: arm={g.name} widen={len(widen)} narrow={len(narrow)}")
    for i, t in widen:
        print(f"    WIDEN  #{i:04d} {t}")
    for i, t in narrow:
        print(f"    NARROW #{i:04d} {t}")
    print()
if OUT:
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "delimiter_arm_ledger.json").write_text(json.dumps(
        {"corpus": CORPUS.name, "n": len(lines), "ledger": ledger}, indent=2))
    print(f"full ledger -> {OUT / 'delimiter_arm_ledger.json'}")
