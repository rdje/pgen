#!/usr/bin/env python3
"""GRAMMAR-WELLFORMED.H.16.6a — AST-identity sweep between an ARM and the PRISTINE control.

⛔ WHY THIS EXISTS: a verdict-only accept-set ledger is blind to the LARGEST transition class —
a replaced AST SHAPE with ZERO verdict movement (TOOLBOX 5.7; 12 pinned witnesses once moved
that way). "widen-only on verdicts" is NOT the claim "no consumer regresses".

⛔⛔ AND THE FIRST VERSION OF THIS SWEEP WAS VACUOUS — read this before choosing `--entry`.
Run at the DEFAULT entry (`semantic_annotation`) it reported `1156/1156 byte-identical,
ast_moved=0` for an arm deliberately built to move every map AST (`map_entry` retyped to
`map_entry_RED_CONTROL`). Cause: `semantic_annotation`'s own `-> {…, value: $6}` publishes the
`/\\s*/` SEPARATOR rather than `annotation_value` (element 7), so the entry AST's `value` is
ALWAYS `""` and NOTHING below the entry rule can reach the dump. A control that cannot fail is
not a control. ⇒ sweep at a rule where the shape is the ROOT (`--entry annotation_value`, with
a value-only corpus), and re-prove the red control fires whenever the entry or corpus changes.

usage: ast_identity_sweep.py <arm.ebnf> <control.ebnf> <corpus-dir> <input-dir> [--entry RULE]
"""
import concurrent.futures
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
BIN = ROOT / "rust" / "target" / "debug" / "ast_pipeline"
pos = [a for a in sys.argv[1:] if not a.startswith("--")]
ARM, CTRL, CORPUS, INPUTS = (pathlib.Path(pos[0]), pathlib.Path(pos[1]),
                             pathlib.Path(pos[2]), pathlib.Path(pos[3]))
ENTRY = None
for a in sys.argv[1:]:
    if a.startswith("--entry="):
        ENTRY = a.split("=", 1)[1]
WORK = ROOT / "rust" / "target" / "h1666a" / "astsweep"
WORK.mkdir(parents=True, exist_ok=True)

if not BIN.exists():
    sys.exit(f"ast_identity_sweep: REFUSED — no ast_pipeline at {BIN}")


def ast_of(g, inp, tag):
    out = WORK / f"{tag}.json"
    if out.exists():
        out.unlink()
    cmd = [str(BIN), str(g), "--interpret-parse", str(inp), "--interpret-parse-ast-json", str(out)]
    if ENTRY:
        cmd += ["--interpret-entry-rule", ENTRY]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if "accepted=true" not in r.stdout:
        return None
    return out.read_text() if out.exists() else ""


items = []
for p in sorted(CORPUS.glob("corpus_*.txt")):
    cname = p.stem.replace("corpus_", "")
    lines = [ln for ln in p.read_text(errors="replace").split("\n") if ln.strip()]
    for i, ln in enumerate(lines):
        items.append((cname, i, ln, INPUTS / f"{cname}_{i}.txt"))


def one(job):
    idx, (cname, i, text, path) = job
    return cname, text, ast_of(ARM, path, f"a{idx}"), ast_of(CTRL, path, f"c{idx}")


with concurrent.futures.ThreadPoolExecutor(max_workers=8) as ex:
    rows = list(ex.map(one, enumerate(items)))

both = [r for r in rows if r[2] is not None and r[3] is not None]
same = [r for r in both if r[2] == r[3]]
diff = [r for r in both if r[2] != r[3]]

print(f"AST-IDENTITY-SWEEP: arm={ARM.name} control={CTRL.name} entry={ENTRY or '<default>'} "
      f"inputs={len(rows)} accepted_by_both={len(both)} ast_byte_identical={len(same)} "
      f"ast_moved={len(diff)}")
for cname, text, a, c in diff[:25]:
    print(f"  AST MOVED [{cname}] {text[:120]}")
sys.exit(1 if diff else 0)
