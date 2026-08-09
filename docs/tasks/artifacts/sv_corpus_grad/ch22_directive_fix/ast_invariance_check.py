#!/usr/bin/env python3
"""SV-CORPUS-GRAD.3.14b — the AST-invariance measurement.

QUESTION: did the new `in_scope_compiler_directive` arm change the AST of any file that
ALREADY parsed before the change?

WHY THIS ANSWERS IT WITHOUT THE PRE-CHANGE BINARY. The grammar delta adds exactly ONE new
production, to two rules, and it emits exactly one node shape. So an AST can differ from
its pre-change self ONLY by containing a node produced by that arm. Counting those nodes
therefore decides invariance outright.

THE DISCRIMINATOR IS THE CONTAINER, NOT THE KIND STRING — measured, not assumed:

    top level   (source_text_item, UNCHANGED by this leaf)
        $/content/Json/source_text[0]
    in scope    (the NEW arm, module/class body)
        $/content/Json/source_text[0]/body/body/body/items[0]

Both emit the byte-identical node `{"body": "...", "kind": "compiler_directive"}` — which
is also why the schema does not change. A directive node is therefore NEW iff its path
extends beyond `source_text[N]`.

⛔ GROUND TRUTH IS PINNED INSIDE THE INSTRUMENT ([[feedback_instrument_needs_ground_truth]]):
the two minimal reproducers above are re-parsed first and must land on those exact paths.
On a miss the script REFUSES to report, because a path-shape change would silently turn
every "0 deep nodes" verdict into a vacuous pass.
"""
import json
import subprocess
import sys
from pathlib import Path

PROBE = Path("rust/target/release/parseability_probe")
TOP_PREFIX = "$/content/Json/source_text"


def dump_ast(path, profile, tmp):
    rc = subprocess.run(
        [str(PROBE), "--parse-dump-ast-pretty", "systemverilog", str(path), str(tmp),
         "--profile", profile],
        capture_output=True, timeout=300,
    )
    if rc.returncode != 0 or not tmp.exists():
        return None
    try:
        return json.loads(tmp.read_text())
    except json.JSONDecodeError:
        return None


def directive_paths(node, p="$"):
    if isinstance(node, dict):
        if node.get("kind") == "compiler_directive":
            yield p
        for k, v in node.items():
            yield from directive_paths(v, f"{p}/{k}")
    elif isinstance(node, list):
        for i, v in enumerate(node):
            yield from directive_paths(v, f"{p}[{i}]")


def is_top_level(path):
    """`$/content/Json/source_text[N]` and nothing deeper."""
    return path.startswith(TOP_PREFIX) and "/" not in path[len(TOP_PREFIX):]


def main(listfile, profile, outp):
    tmp = Path("tmp/ch22b/_inv.ast.json")

    # ---- controls -----------------------------------------------------------------
    ctl = []
    for name, expect_deep in (("min_top", False), ("min_mod", True)):
        ast = dump_ast(f"tmp/ch22b/{name}.sv", profile, tmp)
        if ast is None:
            print(f"⛔ CONTROL {name}: no AST — REFUSING"); return 3
        ps = list(directive_paths(ast))
        if len(ps) != 1:
            print(f"⛔ CONTROL {name}: expected 1 directive node, got {len(ps)} — REFUSING"); return 3
        deep = not is_top_level(ps[0])
        ctl.append((name, ps[0], deep))
        if deep != expect_deep:
            print(f"⛔ CONTROL {name}: path {ps[0]} deep={deep}, expected deep={expect_deep} — REFUSING")
            return 3
    print("=== CONTROLS (pinned; the script refuses to report on a miss) ===")
    for n, p, d in ctl:
        print(f"  {n:8s} deep={str(d):5s}  {p}")

    # ---- the population -----------------------------------------------------------
    files = [l.strip() for l in Path(listfile).read_text().splitlines() if l.strip()]
    rows, n_ast, n_noast, deep_hits, top_nodes = [], 0, 0, [], 0
    for f in files:
        ast = dump_ast(f, profile, tmp)
        if ast is None:
            n_noast += 1
            rows.append(f"{f}\tNO_AST\t")
            continue
        n_ast += 1
        ps = list(directive_paths(ast))
        deep = [p for p in ps if not is_top_level(p)]
        top_nodes += len(ps) - len(deep)
        if deep:
            deep_hits.append((f, deep[:3]))
        rows.append(f"{f}\t{len(ps)}\t{len(deep)}")
    tmp.unlink(missing_ok=True)
    Path(outp).write_text("\n".join(rows) + "\n")

    print()
    print("=== RESULT ===")
    print(f"previously-PASSING files probed          : {len(files)}")
    print(f"  AST dumped                             : {n_ast}")
    print(f"  no AST (parse changed / timeout)       : {n_noast}")
    print(f"top-level compiler_directive nodes seen  : {top_nodes}  (pre-existing path, unchanged)")
    print(f"DEEP (in-scope, NEW-ARM) directive nodes : {len(deep_hits)} file(s)")
    for f, d in deep_hits[:20]:
        print(f"  ⚠️ {f}  {d}")
    print()
    print("VERDICT: AST byte-invariant on previously-passing files"
          if not deep_hits and not n_noast else
          "VERDICT: NOT invariant — inspect the rows above")
    return 0 if (not deep_hits and not n_noast) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1], sys.argv[2], sys.argv[3]))
