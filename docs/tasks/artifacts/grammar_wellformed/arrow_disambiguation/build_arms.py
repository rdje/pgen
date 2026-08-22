#!/usr/bin/env python3
"""GRAMMAR-WELLFORMED.H.16.6a — build the `=>` disambiguation ARMS on SCRATCH COPIES.

⛔ No arm ever touches `grammars/semantic_annotation.ebnf`. Each arm is a COPY under
`rust/target/` (git-ignored, on the repository volume) carrying exactly one edit, so every arm
is scored by `ast_pipeline … --interpret-parse` with ZERO regeneration and ZERO grammar bytes
moved. Every substitution is a VERBATIM literal replace that REFUSES if its anchor is absent —
a silently-skipped edit would produce an arm identical to the control and score as a clean pass.

usage: build_arms.py [outdir]      (default: rust/target/h1666a/arms)
"""
import os
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[5]
SRC = ROOT / "grammars" / "semantic_annotation.ebnf"
OUT = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "rust" / "target" / "h1666a" / "arms"
OUT.mkdir(parents=True, exist_ok=True)

TEXT = SRC.read_text()

MAP_ENTRY = 'map_entry := annotation_value /\\s*/ "=>" /\\s*/ annotation_value'
IMPL = 'implication_expr := logical_or_expr (/\\s*/ "=>" /\\s*/ logical_or_expr)?'
LAMBDA_ALT1 = '    "(" /\\s*/ (lambda_parameter (/\\s*/ "," /\\s*/ lambda_parameter)*)? /\\s*/ ")" /\\s*/ "=>" /\\s*/ annotation_value |'
LAMBDA_ALT2 = '    lambda_parameter /\\s*/ "=>" /\\s*/ annotation_value'
FUNC_TYPE = 'function_type := "(" /\\s*/ (type_reference (/\\s*/ "," /\\s*/ type_reference)*)? /\\s*/ ")" /\\s*/ "=>" /\\s*/ type_reference'

GUARD = ' !(/\\s*/ ("}" | ","))'


def sub(text, old, new, label):
    """Literal replace that REFUSES on a missing or non-unique anchor."""
    n = text.count(old)
    if n != 1:
        sys.exit(f"build_arms: REFUSED — anchor {label!r} occurs {n} times (want exactly 1)")
    return text.replace(old, new)


def write(name, text, edits):
    path = OUT / name
    path.write_text(text)
    delta = sum(1 for a, b in zip(TEXT.splitlines(), text.splitlines()) if a != b)
    print(f"{name:<38} chars={len(text):<7} edited_anchors={edits}")


# ------------------------------------------------------- arm 0: PRISTINE (the control)
write("arm0_pristine.ebnf", TEXT, 0)

# --- arm (a): a dedicated map-key rule routing around the IMPLICATION level (the leaf's (a)).
#     `logical_expression` -> `logical_or_expr` in the KEY position ONLY. The lambda and
#     function_type reaches are deliberately left intact: this is option (a) AS PRICED.
ARM_A_RULES = """

# --- H.16.6a arm (a): map key routes around the IMPLICATION level only -----------------------
map_key := (
    primitive_value |
    structured_value |
    map_key_expression |
    reference_value
) -> $1

map_key_expression := (
    arithmetic_expression |
    logical_or_expr |
    comparison_expression |
    conditional_expression |
    function_call |
    lambda_expression
) -> $1
"""
t = sub(TEXT, MAP_ENTRY, 'map_entry := map_key /\\s*/ "=>" /\\s*/ annotation_value', "map_entry")
write("arm_a_mapkey_impl.ebnf", t + ARM_A_RULES, 1)

# --- arm (a+): the CORRECTED (a) — the key routes around ALL THREE arrow-consuming reaches
#     (implication, lambda, function_type), which the ARROW-CENSUS proved are exactly what
#     `annotation_value` can swallow.
ARM_APLUS_RULES = """

# --- H.16.6a arm (a+): map key routes around implication AND lambda AND function_type ---------
map_key := (
    primitive_value |
    structured_value |
    map_key_expression |
    map_key_reference
) -> $1

map_key_expression := (
    arithmetic_expression |
    logical_or_expr |
    comparison_expression |
    conditional_expression |
    function_call
) -> $1

map_key_reference := (
    map_key_type_reference |
    rule_reference |
    symbol_reference |
    path_reference |
    url_reference
) -> $1

map_key_type_reference := (
    generic_type |
    array_type |
    optional_type |
    primitive_type
) -> $1
"""
write("arm_aplus_mapkey_all.ebnf", t + ARM_APLUS_RULES, 1)

# --- arm (b): negative-lookahead guard on `implication_expr`'s optional tail (the leaf's (b)) —
#     the tail declines when its right operand is followed by a map terminator.
t = sub(TEXT, IMPL,
        'implication_expr := logical_or_expr (/\\s*/ "=>" /\\s*/ logical_or_expr' + GUARD + ')?',
        "implication_expr")
write("arm_b_lookahead_impl.ebnf", t, 1)

# --- arm (b+): the same guard on ALL THREE arrow-consuming tails.
t2 = sub(t, LAMBDA_ALT1, LAMBDA_ALT1[:-2] + GUARD + ' |', "lambda_alt1")
t2 = sub(t2, LAMBDA_ALT2, LAMBDA_ALT2 + GUARD, "lambda_alt2")
t2 = sub(t2, FUNC_TYPE, FUNC_TYPE + GUARD, "function_type")
write("arm_bplus_lookahead_all.ebnf", t2, 4)

# --- arm (c): re-spell the MAP arrow — the only edit that removes the shared token outright.
t = sub(TEXT, MAP_ENTRY, 'map_entry := annotation_value /\\s*/ "=>>" /\\s*/ annotation_value', "map_entry")
write("arm_c_respell_map.ebnf", t, 1)

# --- arm (c2): re-spell the IMPLICATION operator instead (leaves lambda + function_type).
t = sub(TEXT, IMPL, 'implication_expr := logical_or_expr (/\\s*/ "==>" /\\s*/ logical_or_expr)?', "implication_expr")
write("arm_c2_respell_impl.ebnf", t, 1)
