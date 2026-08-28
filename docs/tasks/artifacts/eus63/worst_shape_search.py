#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.63(b) — `.61`(b)'s own bound (i) said "assume a worse shape
exists until the search says otherwise". This is that search.

The wide sweep already beat `.61`(b)'s worst (`cast_paren`, 249) with `gen_mux_both_arms`
at 146. That shape is not exotic: it is a two-arm mux with both arms parenthesised, which
is how a defensive emitter writes every conditional. This probe pushes further using only
COMPOSITES REAL EMITTERS PRODUCE -- sign/width coercion wrapped around a mux arm is the
canonical sv2v and Verilator lowering -- and reports frames/level so the cost of each
wrapper is legible.
"""
import subprocess, os, json

PROBE = "./rust/target/release/parseability_probe"
TMP = "tmp/eus63/shapes"
os.makedirs(TMP, exist_ok=True)
CEILING = 4096

SHAPES = {
    # one wrapper per level -------------------------------------------------------
    "w_paren":            ("(", ")"),
    "w_cast_int":         ("int'(", ")"),
    "w_cast_sized":       ("8'(", ")"),
    "w_signed":           ("$signed(", ")"),
    "w_unsigned":         ("$unsigned(", ")"),
    "w_concat":           ("{", "}"),
    "w_concat_paren":     ("{(", ")}"),
    # the mux cell, in the bracketings real emitters use --------------------------
    "w_mux_else_paren":   ("s ? a : (", ")"),
    "w_mux_both_arms":    ("(s ? a : (", "))"),
    "w_mux_signed_arms":  ("$signed(s ? a : $signed(", "))"),
    "w_mux_cast_arms":    ("int'(s ? int'(a) : int'(", "))"),
    # width coercion stacked on a mux -- the sv2v lowering of a mixed-width select -
    "w_sv2v_mux":         ("$signed({1'b0, (s ? a : (", "))})"),
    "w_cast_mux_cast":    ("int'((s ? int'(a) : (", ")))"),
    # a reduction/compare arm, as the register-file lowering writes it -------------
    "w_cmp_mux":          ("({a, b} == {c, d}) ? e : (", ")"),
}

HDR, TAIL = "module m; assign x = ", "; endmodule\n"


def ok(name, d, open_s, close_s):
    p = f"{TMP}/ws_{name}_{d}.sv"
    with open(p, "w") as fh:
        fh.write(HDR + open_s * d + "1" + close_s * d + TAIL)
    try:
        r = subprocess.run([PROBE, "--parse", "systemverilog", p, "--profile", "sv_2017"],
                           capture_output=True, timeout=900)
    except subprocess.TimeoutExpired:
        return None, False
    out = (r.stdout + r.stderr).decode(errors="replace")
    return r.returncode == 0, ("recursion ceiling CONTACTED" in out)


def bisect(name, open_s, close_s, cap=8192):
    a, _ = ok(name, 1, open_s, close_s)
    if a is not True:
        return {"shape": name, "verdict": "rejects at depth 1"}
    lo, hi = 1, 2
    while hi <= cap:
        a, _ = ok(name, hi, open_s, close_s)
        if a is not True:
            break
        lo, hi = hi, hi * 2
    else:
        return {"shape": name, "deepest_ok": f">{cap}", "verdict": "no cliff"}
    while hi - lo > 1:
        mid = (lo + hi) // 2
        a, _ = ok(name, mid, open_s, close_s)
        if a is True:
            lo = mid
        else:
            hi = mid
    _, lab = ok(name, hi, open_s, close_s)
    return {"shape": name, "deepest_ok": lo, "first_bad": hi,
            "frames_per_level": round(CEILING / lo, 1), "labelled": bool(lab),
            "verdict": "cliff"}


rows = []
print(f"{'shape':20} {'deepest OK':>10} {'first bad':>10} {'fr/lvl':>8} labelled")
for name, (o, c) in SHAPES.items():
    r = bisect(name, o, c)
    rows.append(r)
    print(f"{r['shape']:20} {str(r.get('deepest_ok','-')):>10} "
          f"{str(r.get('first_bad','-')):>10} {str(r.get('frames_per_level','-')):>8} "
          f"{r.get('labelled')}", flush=True)
with open("tmp/eus63/worst_shape_search.json", "w") as fh:
    json.dump(rows, fh, indent=1)
cliffs = [r for r in rows if r.get("verdict") == "cliff"]
worst = min(cliffs, key=lambda r: r["deepest_ok"])
print(f"\nWORST: {worst['shape']} accepts {worst['deepest_ok']}, "
      f"rejects {worst['first_bad']} ({worst['frames_per_level']} frames/level)")
