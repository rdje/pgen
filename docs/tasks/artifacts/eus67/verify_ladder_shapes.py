#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.67(b) — the ladder's SYNTHETIC inputs are reproducible.

Five of the eleven ladder inputs live under `tmp/` (untracked by policy), so a reader of
`attribution_owned.tsv` cannot see what was parsed. They are not ad-hoc: every one is the
exact output of a TRACKED generator, `docs/tasks/artifacts/eus63/iteration_cap_probe.py`.

This re-derives them and compares BYTE FOR BYTE, so "reproducible" is a check rather than a
claim. Run it from the repository root; exit 0 = every present shape reproduces.

⚠️ It verifies the SYNTHETIC half only. The six real inputs are vendored corpus submodule
files, identified by repo-relative path in the TSV.
"""
import os
import sys

GEN = "docs/tasks/artifacts/eus63/iteration_cap_probe.py"


def load_generators():
    """Exec only the function definitions — importing the module would run its probe."""
    src = open(GEN).read()
    namespace = {}
    exec(compile(src.split("SH = {")[0], GEN, "exec"), namespace)
    return {
        "netlist_instances": namespace["netlist"],
        "concat_items": namespace["concat_items"],
        "case_items": namespace["case_items"],
        "port_list": namespace["ports"],
        "wire_decls": namespace["decls"],
        "proc_statements": namespace["stmts"],
    }


def main():
    generators = load_generators()
    checked = failed = 0
    for name, gen in sorted(generators.items()):
        for n in (2500, 5000, 10000):
            path = f"tmp/eus67/shapes/syn_{name}_{n}.sv"
            if not os.path.exists(path):
                continue
            checked += 1
            same = gen(n) == open(path).read()
            failed += 0 if same else 1
            print(f"{'MATCH  ' if same else 'DIFFER '} {path}")
    if checked == 0:
        # ⛔ Nothing to compare is NOT a pass: it is the shapes being absent.
        print("REFUSE: no ladder shape found under tmp/eus67/shapes/ — nothing was verified")
        return 2
    print(f"\n{checked - failed}/{checked} ladder shapes reproduce from {GEN}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
