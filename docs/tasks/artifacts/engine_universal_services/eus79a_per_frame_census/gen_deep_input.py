#!/usr/bin/env python3
"""`ENGINE-UNIVERSAL-SERVICES.79`(a) — deep-nesting input generator, one shape per family.

Each shape is the CHEAPEST recursive construct that family accepts, so the ladder
reaches the recursion guard on the fewest source bytes. The measured quantity is
`bytes_per_frame` at the refusal, which is a property of the family/profile/build
mode and NOT of the shape's source depth (`-0141`: the same reading comes back at
2 048 / 4 096 / 8 192 levels), so the shape only has to reach the ceiling at all.

⛔ Written under the repository's own gitignored `tmp/` so every byte stays on the
repository volume (directive 13).
"""
import argparse
import os

# family -> (filename, builder). `n` is source nesting levels, not parser frames.
SHAPES = {
    "systemverilog": (
        "sv.sv",
        lambda n: "module m; wire w; assign w = " + "(" * n + "1" + ")" * n + "; endmodule\n",
    ),
    "vhdl": (
        "vhdl.vhd",
        lambda n: (
            "entity e is end entity;\narchitecture a of e is\nbegin\n"
            "  p : process begin x <= " + "(" * n + "1" + ")" * n + "; end process;\n"
            "end architecture;\n"
        ),
    ),
    "json": ("json.json", lambda n: "[" * n + "1" + "]" * n),
    "rtl_const_expr": ("rtl_const_expr.txt", lambda n: "(" * n + "1" + ")" * n),
    "rtl_frontend": (
        "rtl_frontend.v",
        lambda n: "module m; wire w; assign w = " + "(" * n + "1" + ")" * n + "; endmodule\n",
    ),
    "return_annotation": ("return_annotation.txt", lambda n: "-> " + "(" * n + "$1" + ")" * n),
    "semantic_annotation": (
        "semantic_annotation.txt",
        lambda n: "@profiles: " + "[" * n + '"a"' + "]" * n,
    ),
    "systemverilog_preprocessor": (
        "svpp.sv",
        lambda n: "`define A 1\n" + "`ifdef A\n" * n + "wire x;\n" + "`endif\n" * n,
    ),
    "regex": ("regex.rgx", lambda n: "(" * n + "a" + ")" * n),
}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("family", choices=sorted(SHAPES))
    ap.add_argument("levels", type=int)
    ap.add_argument("--out-dir", default="tmp/eus79a/census")
    args = ap.parse_args()

    name, build = SHAPES[args.family]
    os.makedirs(args.out_dir, exist_ok=True)
    path = os.path.join(args.out_dir, f"{args.levels}_{name}")
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(build(args.levels))
    print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
