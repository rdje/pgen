#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.63 — the SECOND ceiling, found while re-adjudicating the first.

`SAFETY_LIMIT = 10_000` caps EVERY quantifier in every generated parser
(`ast_based_generator.rs:6587` protocol path, `ast_based_generator/cascade.rs:1704` fused
path). Reaching it does not refuse -- it `break`s, silently truncating the repetition, and
the parse then fails at the enclosing level with an ordinary "did not consume full input".
The protocol path logs a warning only under `trace_enabled()`; the fused path logs nothing.

This probe measures where it bites on the shapes a FLATTENED / GENERATED netlist actually
has, which is the population the director's ruling named. Coarse ladder + per-run timeout:
the point is the cliff's location, not a bisected exact value.
"""
import subprocess, sys, json

PROBE = "./rust/target/release/parseability_probe"
TIMEOUT = 600


def netlist(n):
    body = "".join(f"  AND2 u{i} (.A(a[{i}]), .B(b[{i}]), .Z(z[{i}]));\n" for i in range(n))
    return f"module top(input [{n-1}:0] a, input [{n-1}:0] b, output [{n-1}:0] z);\n{body}endmodule\n"

def decls(n):
    return "module top;\n" + "".join(f"  wire w{i};\n" for i in range(n)) + "endmodule\n"

def concat_items(n):
    return "module m; assign x = {" + ", ".join(f"a{i}" for i in range(n)) + "}; endmodule\n"

def case_items(n):
    arms = "".join(f"    {i}: y = {i};\n" for i in range(n))
    return f"module m; always_comb begin\n  case (s)\n{arms}    default: y = 0;\n  endcase\nend endmodule\n"

def ports(n):
    return "module top(" + ", ".join(f"input p{i}" for i in range(n)) + ");\nendmodule\n"

def stmts(n):
    return "module m; always_comb begin\n" + "".join(f"  y{i} = {i};\n" for i in range(n)) + "end endmodule\n"

SH = {"netlist_instances": netlist, "wire_decls": decls, "concat_items": concat_items,
      "case_items": case_items, "port_list": ports, "proc_statements": stmts}
LADDER = [100, 1000, 5000, 9999, 10000, 10001, 12000, 20000]


def run(name, gen, n):
    p = f"tmp/eus63/shapes/lim_{name}_{n}.sv"
    with open(p, "w") as fh:
        fh.write(gen(n))
    try:
        r = subprocess.run([PROBE, "--parse", "systemverilog", p, "--profile", "sv_2017"],
                           capture_output=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return "TIMEOUT", False
    out = (r.stdout + r.stderr).decode(errors="replace")
    return ("ACCEPT" if r.returncode == 0 else "REJECT"), ("recursion ceiling CONTACTED" in out)


res = {}
for name, gen in SH.items():
    res[name] = {}
    row = []
    for n in LADDER:
        v, lab = run(name, gen, n)
        res[name][n] = {"verdict": v, "ceiling_labelled": lab}
        row.append(f"{n}:{v}{'(L)' if lab else ''}")
    print(f"{name:20} " + "  ".join(row), flush=True)
with open("tmp/eus63/iteration_cap.json", "w") as fh:
    json.dump(res, fh, indent=1)
