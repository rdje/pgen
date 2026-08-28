#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.63(b) — re-derive the worst-shape capacity over a shape set
chosen from WHAT GENERATORS EMIT, not from what is easy to write.

`.61`(b) bisected ten shapes and reported a 3.3x spread (249-816), taking 249 (`int'()`
casts) as the honest worst. It also warned, in its own bound (i), that ten shapes are not
an enumeration. This harness widens the set with the shapes a machine RTL emitter actually
produces -- priority-encoder else-if chains, bracket-free right-associative ternary mux
chains, replication/concat bus assembly, sized-cast width coercion -- and bisects each the
same way, so the two tables are directly comparable.

Method (identical to `.61`(b)'s so the numbers compose): exponential probe to find a
rejecting depth, then binary search for the accept/reject boundary. frames/level =
4096 / deepest-accepted. Every boundary is then RE-CHECKED for monotonicity over
[D-2, D+2] -- bisection is only sound if accept is downward-closed, and `.61`(b) assumed
that without testing it.
"""
import subprocess, sys, os, time, json

PROBE = os.environ.get("PGEN_PARSE_PROBE_BIN", "./rust/target/release/parseability_probe")
TMP = "tmp/eus65/shapes"
CEILING = 4096
os.makedirs(TMP, exist_ok=True)

# ---------------------------------------------------------------- SystemVerilog shapes
SV = {
    # --- `.61`(b)'s ten, replayed so the wide table is self-contained and comparable ---
    "paren_expr":      lambda d: "module m; assign x = " + "("*d + "1" + ")"*d + "; endmodule\n",
    "concat":          lambda d: "module m; assign x = " + "{"*d + "1" + "}"*d + "; endmodule\n",
    "call":            lambda d: "module m; assign x = " + "f("*d + "1" + ")"*d + "; endmodule\n",
    "ternary":         lambda d: "module m; assign x = " + "a ? b : ("*d + "1" + ")"*d + "; endmodule\n",
    "cast_paren":      lambda d: "module m; assign x = " + "int'("*d + "1" + ")"*d + "; endmodule\n",
    "begin_end":       lambda d: "module m; initial " + "begin "*d + "x = 1; " + "end "*d + " endmodule\n",
    "if_nest":         lambda d: "module m; initial " + "if (a) "*d + "x = 1; endmodule\n",
    "generate_if":     lambda d: "module m; " + "if (1) begin "*d + " wire w; " + "end "*d + " endmodule\n",
    "index_sel":       lambda d: "module m; assign x = " + "y" + "[0]"*d + "; endmodule\n",
    "unary_not":       lambda d: "module m; assign x = " + "!"*d + "1; endmodule\n",

    # --- NEW: the shapes a machine RTL emitter produces -------------------------------
    # A priority mux lowered by a Chisel/FIRRTL-class emitter: right-associative and
    # BRACKET-FREE, so `.61`(b)'s bracket-depth oracle reads 0 on it at any width.
    "gen_ternary_chain":
        lambda d: "module m; assign x = " + "".join(f"c{i} ? v{i} : " for i in range(d)) + "vd; endmodule\n",
    # The same mux tree emitted with defensive parentheses (sv2v / Verilator style).
    "gen_ternary_chain_paren":
        lambda d: "module m; assign x = " + "".join(f"c{i} ? v{i} : (" for i in range(d)) + "vd" + ")"*d + "; endmodule\n",
    # A priority encoder emitted as an else-if ladder -- one branch per input bit.
    "gen_else_if_chain":
        lambda d: "module m; always_comb begin " + "".join(f"if (c{i}) y = {i}; else " for i in range(d)) + "y = 0; end endmodule\n",
    # The same ladder with begin/end blocks, which is what most emitters bracket with.
    "gen_else_if_block":
        lambda d: "module m; always_comb begin " + "".join(f"if (c{i}) begin y = {i}; end else begin " for i in range(d)) + "y = 0; " + "end "*d + "end endmodule\n",
    # Bus assembly as nested replication -- width coercion in generated datapaths.
    "gen_repl_nest":   lambda d: "module m; assign x = " + "{1{"*d + "1'b0" + "}"*d + "; endmodule\n",
    # Width coercion emitted as nested sized casts.
    "gen_sized_cast":  lambda d: "module m; assign x = " + "8'("*d + "1" + ")"*d + "; endmodule\n",
    # Sign coercion emitted as nested system calls -- the classic sv2v lowering.
    "gen_signed_nest": lambda d: "module m; assign x = " + "$signed("*d + "1" + ")"*d + "; endmodule\n",
    # A balanced mux tree written as a nested conditional with both arms parenthesised.
    "gen_mux_both_arms":
        lambda d: "module m; assign x = " + "(s ? a : ("*d + "b" + "))"*d + "; endmodule\n",
    # Arithmetic reassociated right by an optimiser.
    "gen_binop_right":  lambda d: "module m; assign x = " + "(a + "*d + "b" + ")"*d + "; endmodule\n",
    # Flat left-associative arithmetic -- the shape an unoptimised emitter produces.
    "gen_binop_flat":   lambda d: "module m; assign x = " + " + ".join(["a"]*(d+1)) + "; endmodule\n",
    # Reduction emitted as a flat boolean chain.
    "gen_and_flat":     lambda d: "module m; assign x = " + " && ".join(["a"]*(d+1)) + "; endmodule\n",
    # Generated hierarchy: nested generate-for, one level per array dimension.
    "gen_for_nest":
        lambda d: "module m; " + "".join(f"for (genvar i{i} = 0; i{i} < 2; i{i}++) begin " for i in range(d)) + " wire w; " + "end "*d + " endmodule\n",
    # Nested case statements -- decoded control emitted as a tree.
    "gen_case_nest":
        lambda d: "module m; always_comb begin " + "case (s) 0: "*d + "y = 1; " + "endcase "*d + "end endmodule\n",
    # Packed struct types nested by a type generator.
    "gen_struct_nest":
        lambda d: "module m; " + "typedef struct packed { logic f; } t0; " + "".join(f"typedef struct packed {{ t{i} f; }} t{i+1}; " for i in range(d)) + "endmodule\n",
    # Packed array dimensions stacked by a type generator.
    "gen_array_dims":   lambda d: "module m; logic " + "[1:0]"*d + " s; endmodule\n",
    # Deep hierarchical reference into generated instance names.
    "gen_hier_ref":     lambda d: "module m; assign x = " + ".".join(f"u{i}" for i in range(d)) + ".sig; endmodule\n",
    # Parameter expression folded by an elaborator into a nested arithmetic term.
    "gen_param_expr":
        lambda d: "module m; localparam P = " + "(1 + "*d + "1" + ")"*d + "; endmodule\n",
}

# ------------------------------------------------------------------------- VHDL shapes
VHDL_HEAD = "entity e is end entity;\narchitecture a of e is\n"
VHDL_TAIL = "end architecture;\n"
VHDL = {
    "paren_expr":   lambda d: VHDL_HEAD + "signal s : bit;\nbegin\n s <= " + "("*d + "'0'" + ")"*d + ";\n" + VHDL_TAIL,
    "gen_when_else":
        lambda d: VHDL_HEAD + "signal s : bit;\nbegin\n s <= " + "".join(f"v{i} when c{i} = '1' else " for i in range(d)) + "'0';\n" + VHDL_TAIL,
    "gen_elsif_chain":
        lambda d: VHDL_HEAD + "signal s : bit;\nbegin\n process begin\n  if c0 = '1' then s <= '0';\n"
                  + "".join(f"  elsif c{i} = '1' then s <= '0';\n" for i in range(1, d + 1))
                  + "  end if;\n end process;\n" + VHDL_TAIL,
    "gen_if_nest":
        lambda d: VHDL_HEAD + "signal s : bit;\nbegin\n process begin\n" + "  if c = '1' then\n"*d
                  + "   s <= '0';\n" + "  end if;\n"*d + " end process;\n" + VHDL_TAIL,
    "gen_concat_nest":
        lambda d: VHDL_HEAD + "signal s : bit;\nbegin\n s <= " + "("*d + "'0'" + " & '1')"*d + ";\n" + VHDL_TAIL,
    "gen_func_nest":
        lambda d: VHDL_HEAD + "signal s : bit;\nbegin\n s <= " + "f("*d + "'0'" + ")"*d + ";\n" + VHDL_TAIL,
}

FAMS = {"systemverilog": (SV, "sv", ["--profile", "sv_2017"]), "vhdl": (VHDL, "vhd", [])}

_ONLY = os.environ.get("PGEN_SHAPE_SUBSET")
if _ONLY:
    _keep = set(_ONLY.split(","))
    SV = {k: v for k, v in SV.items() if k in _keep}
    VHDL = {k: v for k, v in VHDL.items() if k in _keep}
    FAMS = {"systemverilog": (SV, "sv", ["--profile", "sv_2017"]), "vhdl": (VHDL, "vhd", [])}


def run(fam, name, d, gen, ext, extra):
    p = f"{TMP}/{fam}_{name}_{d}.{ext}"
    with open(p, "w") as fh:
        fh.write(gen(d))
    t0 = time.time()
    try:
        r = subprocess.run([PROBE, "--parse", fam, p] + extra,
                           capture_output=True, timeout=600)
    except subprocess.TimeoutExpired:
        return None, False, 600.0
    out = (r.stdout + r.stderr).decode(errors="replace")
    return r.returncode == 0, ("recursion ceiling CONTACTED" in out), time.time() - t0


def bisect(fam, name, gen, ext, extra, cap=16384):
    ok1, _, _ = run(fam, name, 1, gen, ext, extra)
    if ok1 is not True:
        return {"shape": name, "verdict": "rejects at depth 1"}
    lo, hi = 1, 2
    while hi <= cap:
        ok, _, _ = run(fam, name, hi, gen, ext, extra)
        if ok is not True:
            break
        lo, hi = hi, hi * 2
    else:
        return {"shape": name, "deepest_ok": f">{cap}", "first_bad": None,
                "frames_per_level": None, "labelled": None, "verdict": "no cliff"}
    while hi - lo > 1:
        mid = (lo + hi) // 2
        ok, _, _ = run(fam, name, mid, gen, ext, extra)
        if ok is True:
            lo = mid
        else:
            hi = mid
    _, labelled, secs = run(fam, name, hi, gen, ext, extra)
    # Monotonicity control: accept must be downward-closed for the bisection to mean
    # what it says. `.61`(b) never checked this.
    mono = []
    for d in range(max(1, lo - 2), hi + 3):
        a, _, _ = run(fam, name, d, gen, ext, extra)
        mono.append((d, a))
    monotone = all(a is True for d, a in mono if d <= lo) and \
               all(a is not True for d, a in mono if d >= hi)
    return {"shape": name, "deepest_ok": lo, "first_bad": hi,
            "frames_per_level": round(CEILING / lo, 1), "labelled": bool(labelled),
            "reject_secs": round(secs, 2), "monotone": monotone,
            "mono_window": mono, "verdict": "cliff"}


def main():
    want = sys.argv[1] if len(sys.argv) > 1 else "all"
    results = {}
    for fam, (shapes, ext, extra) in FAMS.items():
        if want not in ("all", fam):
            continue
        results[fam] = []
        print(f"\n=== {fam} ===", flush=True)
        print(f"{'shape':26} {'deepest OK':>10} {'first bad':>10} {'fr/lvl':>8} {'labelled':>9} {'mono':>5}", flush=True)
        for name, gen in shapes.items():
            row = bisect(fam, name, gen, ext, extra)
            results[fam].append(row)
            print(f"{row['shape']:26} {str(row.get('deepest_ok','-')):>10} "
                  f"{str(row.get('first_bad') or '-'):>10} "
                  f"{str(row.get('frames_per_level') or '-'):>8} "
                  f"{str(row.get('labelled')):>9} {str(row.get('monotone')):>5}", flush=True)
    with open("tmp/eus65/shape_cliff_subset.json", "w") as fh:
        json.dump(results, fh, indent=1)


if __name__ == "__main__":
    main()
