#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.61(b) — the ceiling is 4096 FRAMES; what does a frame cost
in SOURCE nesting levels, and how much does that vary by CONSTRUCT?

`.61` measured one shape (nested parens in a continuous assign) and found the cliff at 322,
i.e. ~12.7 frames per level. That number is only the parser's capacity for THAT shape. A
construct costing more frames per level hits the ceiling at a shallower source depth, and it
is the WORST shape — not the paren shape — that bounds what a conforming consumer can write.

Each shape's accept boundary is BISECTED (accept -> deepest accepted, reject -> first
rejected), then frames/level = 4096 / boundary.
"""
import subprocess, sys, os

PROBE = "./rust/target/release/parseability_probe"
TMP = "tmp/eus61/shapes"
os.makedirs(TMP, exist_ok=True)

SHAPES = {
    "paren_expr":     lambda d: "module m; assign x = " + "("*d + "1" + ")"*d + "; endmodule\n",
    "concat":         lambda d: "module m; assign x = " + "{"*d + "1" + "}"*d + "; endmodule\n",
    "call":           lambda d: "module m; assign x = " + "f("*d + "1" + ")"*d + "; endmodule\n",
    "unary_not":      lambda d: "module m; assign x = " + "!"*d + "1; endmodule\n",
    "begin_end":      lambda d: "module m; initial " + "begin "*d + "x = 1; " + "end "*d + " endmodule\n",
    "if_nest":        lambda d: "module m; initial " + "if (a) "*d + "x = 1; endmodule\n",
    "ternary":        lambda d: "module m; assign x = " + "a ? b : ("*d + "1" + ")"*d + "; endmodule\n",
    "index_sel":      lambda d: "module m; assign x = " + "y" + "[0]"*d + "; endmodule\n",
    "generate_if":    lambda d: "module m; " + "if (1) begin "*d + " wire w; " + "end "*d + " endmodule\n",
    "cast_paren":     lambda d: "module m; assign x = " + "int'("*d + "1" + ")"*d + "; endmodule\n",
}

def accepts(name, d, gen):
    p = f"{TMP}/{name}_{d}.sv"
    open(p, "w").write(gen(d))
    r = subprocess.run([PROBE, "--parse", "systemverilog", p, "--profile", "sv_2017"],
                       capture_output=True, timeout=180)
    out = (r.stdout + r.stderr).decode()
    return r.returncode == 0, ("recursion ceiling CONTACTED" in out)

print(f"{'shape':14} {'deepest OK':>10} {'first bad':>10} {'frames/level':>13}  ceiling-labelled")
for name, gen in SHAPES.items():
    ok1, _ = accepts(name, 1, gen)
    if not ok1:
        print(f"{name:14} {'—':>10} {'—':>10} {'(shape rejects at depth 1)':>13}")
        continue
    lo, hi = 1, 2
    while hi <= 8192:
        ok, _ = accepts(name, hi, gen)
        if not ok:
            break
        lo = hi
        hi *= 2
    else:
        print(f"{name:14} {'>8192':>10} {'—':>10} {'no cliff found':>13}")
        continue
    while hi - lo > 1:
        mid = (lo + hi) // 2
        ok, _ = accepts(name, mid, gen)
        if ok: lo = mid
        else:  hi = mid
    _, labelled = accepts(name, hi, gen)
    print(f"{name:14} {lo:>10} {hi:>10} {4096/lo:>13.1f}  {'yes' if labelled else 'NO — different cause'}")
