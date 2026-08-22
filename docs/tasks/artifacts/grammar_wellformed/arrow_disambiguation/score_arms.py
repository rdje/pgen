#!/usr/bin/env python3
"""GRAMMAR-WELLFORMED.H.16.6a — score every `=>` disambiguation ARM against the PRISTINE control.

For each arm grammar and each corpus input, run
    ast_pipeline <arm>.ebnf --interpret-parse <input-file>
(TOOLBOX 1.5b) and record the `accepted=` verdict. Then emit, per arm, the exact
ACCEPT-SET-LEDGER transition versus the pristine control: how many inputs the arm NEWLY
ACCEPTS (a WIDEN) and how many it NEWLY REJECTS (a NARROW), naming every moved input.

⛔ Read-only: no codegen, no regeneration, no registry edit, and no arm is the shipped grammar.
⛔ An arm that fails to load would reject EVERY input and read as a maximal NARROW rather than
   as an error, so the runner asserts the control's own accept count is non-zero and reports
   any input on which the loader itself errored.

usage: score_arms.py <arms-dir> <corpus-dir> [--jobs N]
"""
import concurrent.futures
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
BIN = ROOT / "rust" / "target" / "debug" / "ast_pipeline"

args = [a for a in sys.argv[1:] if not a.startswith("--")]
JOBS = 8
for a in sys.argv[1:]:
    if a.startswith("--jobs"):
        JOBS = int(a.split("=", 1)[1]) if "=" in a else 8

ARMS_DIR = pathlib.Path(args[0])
CORPUS_DIR = pathlib.Path(args[1])

if not BIN.exists():
    sys.exit(f"score_arms: REFUSED — no ast_pipeline at {BIN}")

# ---------------------------------------------------------------- corpora
corpora = {}
for p in sorted(CORPUS_DIR.glob("corpus_*.txt")):
    name = p.stem.replace("corpus_", "")
    lines = [ln for ln in p.read_text(errors="replace").split("\n") if ln.strip()]
    corpora[name] = lines

# materialise every input as its own file (an input is parsed from a FILE, and inputs
# contain quotes/braces/backslashes that must never go through a shell)
INDIR = ROOT / "rust" / "target" / "h1666a" / "inputs"
INDIR.mkdir(parents=True, exist_ok=True)
items = []  # (corpus, idx, text, path)
for cname, lines in corpora.items():
    for i, ln in enumerate(lines):
        path = INDIR / f"{cname}_{i}.txt"
        path.write_text(ln)
        items.append((cname, i, ln, path))

arms = sorted(ARMS_DIR.glob("arm*.ebnf"))
control = ARMS_DIR / "arm0_pristine.ebnf"
if control not in arms:
    sys.exit("score_arms: REFUSED — no arm0_pristine.ebnf control in the arms dir")


def run(arm, path):
    r = subprocess.run([str(BIN), str(arm), "--interpret-parse", str(path)],
                       capture_output=True, text=True)
    out = r.stdout
    if "INTERPRET-PARSE:" not in out:
        return None  # loader error / no verdict line at all
    return "accepted=true" in out


results = {}
for arm in arms:
    with concurrent.futures.ThreadPoolExecutor(max_workers=JOBS) as ex:
        verdicts = list(ex.map(lambda it: run(arm, it[3]), items))
    results[arm.name] = verdicts
    bad = sum(1 for v in verdicts if v is None)
    acc = sum(1 for v in verdicts if v is True)
    print(f"scored {arm.name:<38} accepted={acc:<5} rejected={sum(1 for v in verdicts if v is False):<5} no_verdict={bad}")

ctrl = results[control.name]
if sum(1 for v in ctrl if v is True) == 0:
    sys.exit("score_arms: REFUSED — the CONTROL accepts nothing; the harness, not the grammar, is broken")

# ---------------------------------------------------------------- ledger
ledger = {}
print()
for arm in arms:
    if arm.name == control.name:
        continue
    v = results[arm.name]
    widen, narrow = [], []
    for (cname, i, text, _), a, b in zip(items, ctrl, v):
        if a is False and b is True:
            widen.append((cname, text))
        elif a is True and b is False:
            narrow.append((cname, text))
    ledger[arm.name] = {"widen": widen, "narrow": narrow}
    per_c = {}
    for c in corpora:
        per_c[c] = {
            "widen": sum(1 for x in widen if x[0] == c),
            "narrow": sum(1 for x in narrow if x[0] == c),
        }
    print(f"ACCEPT-SET-LEDGER: arm={arm.name} widen={len(widen)} narrow={len(narrow)} "
          + " ".join(f"{c}(+{d['widen']}/-{d['narrow']})" for c, d in per_c.items()))

out = CORPUS_DIR / "ledger.json"
out.write_text(json.dumps(
    {"corpora": {c: len(l) for c, l in corpora.items()}, "ledger": ledger}, indent=2))
print(f"\nfull per-input ledger -> {out}")
