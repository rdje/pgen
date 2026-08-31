#!/usr/bin/env python3
"""`ENGINE-UNIVERSAL-SERVICES.69`(a) — the SHAPE corpus for the overshoot census.

`.69`(d) left (a) an acceptance criterion in its own words: the overshoot past the
spendable budget is *SHAPE-driven*, not reservation-driven — 6 800 B to 597 280 B, an
88x spread at ONE reservation — so the 6.25 % headroom reserve must be sized against the
WORST shape, and *nothing measures that distribution*. Two shapes is not a distribution.
This generates the population.

⛔ EVERY SHAPE MUST BE LRM-VALID AND ACCEPTED BY THE SHIPPED PROFILE. The north star is
that SystemVerilog is 100 % LRM-compliant by DEFAULT and over-acceptance is a defect, so
a shape that only parses because the parser is too permissive would measure the defect
rather than the stack. `validate_shapes.sh` checks every shape at a shallow depth BEFORE
the census runs, and a shape that does not accept is dropped with its reason recorded —
it is never quietly left in.

⛔ Written under the repository's own gitignored `tmp/` so every byte stays on the
repository volume (directive 13).

  python3 <this> <shape> <levels> [--out-dir DIR]
  python3 <this> --list
"""
import argparse
import os

# name -> (extension, builder). `n` is SOURCE nesting levels, not parser frames.
# Each shape drives recursion through a DIFFERENT chain of the SystemVerilog
# expression/statement hierarchy, which is what makes the frame composition —
# and therefore the per-frame cost and the overshoot — differ between them.
SHAPES = {
    # the two `.69`(d) already measured, kept verbatim so the new census can be
    # cross-checked against the published 6 800 / 597 280 B readings
    "paren":      lambda n: "module m;\n  localparam int p = " + "(" * n + "1" + ")" * n + ";\nendmodule\n",
    "cast":       lambda n: "module m;\n  assign x = " + "int'(" * n + "1" + ")" * n + ";\nendmodule\n",
    # new shapes
    "concat":     lambda n: "module m;\n  assign x = " + "{" * n + "1'b1" + "}" * n + ";\nendmodule\n",
    "ternary":    lambda n: "module m;\n  assign x = " + "c ? " * n + "1" + " : 1" * n + ";\nendmodule\n",
    "func":       lambda n: "module m;\n  assign x = " + "f(" * n + "1" + ")" * n + ";\nendmodule\n",
    "sysfunc":    lambda n: "module m;\n  assign x = " + "$unsigned(" * n + "1" + ")" * n + ";\nendmodule\n",
    "pattern":    lambda n: "module m;\n  localparam p = " + "'{" * n + "1" + "}" * n + ";\nendmodule\n",
    "if_stmt":    lambda n: "module m;\n  always_comb\n    " + "if (c) " * n + "x = 1;\nendmodule\n",
    "begin_end":  lambda n: "module m;\n  always_comb\n    " + "begin " * n + "x = 1;" + " end" * n + "\nendmodule\n",
    "unary_not":  lambda n: "module m;\n  assign x = " + "!" * n + "1" + ";\nendmodule\n",
    "unary_neg":  lambda n: "module m;\n  assign x = " + "-" * n + "1" + ";\nendmodule\n",
    "binary_par": lambda n: "module m;\n  assign x = " + "(1 + " * n + "1" + ")" * n + ";\nendmodule\n",
}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("shape", nargs="?", choices=sorted(SHAPES))
    ap.add_argument("levels", nargs="?", type=int)
    ap.add_argument("--out-dir", default="tmp/eus69a/shapes")
    ap.add_argument("--list", action="store_true")
    args = ap.parse_args()

    if args.list:
        print(" ".join(sorted(SHAPES)))
        return 0
    if not args.shape or args.levels is None:
        ap.error("shape and levels are required unless --list is given")

    os.makedirs(args.out_dir, exist_ok=True)
    path = os.path.join(args.out_dir, f"{args.shape}_{args.levels}.sv")
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(SHAPES[args.shape](args.levels))
    print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
