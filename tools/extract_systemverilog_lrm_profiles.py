#!/usr/bin/env python3
"""
Build a profile-aware SystemVerilog grammar fragment from IEEE 1800 markdown workspaces.

This script extracts Annex-A rules from the converted markdown sources, normalizes
footnote-suffixed rule names, and emits a profile-aware EBNF fragment that supports
both `sv_2017` and `sv_2023` through rule-level `@profiles` annotations. It can
also emit the promoted flattened active `systemverilog.ebnf` file used by the HDL
pipeline.

The output is intentionally pragmatic:
- shared rules are emitted once,
- version-divergent rules are emitted as shared wrappers plus profiled subrules,
- a stable hand-maintained lexical foundation is used for the parser-facing layer.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from collections import OrderedDict, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Iterator


RULE_HEAD_RE = re.compile(r"^\s*([A-Za-z_$][A-Za-z0-9_$,]*)\s*::\s*=\s*(.*?)\s*$")
HEADING_RE = re.compile(r"^(A(\.\d+)*|\d+(\.\d+)*)\b")
GRAMMAR_LINE_RE = re.compile(r"[A-Za-z0-9_$`'.,;:|?*/+\-<>=!~@#&()\[\]{} \\]+")
RULE_NAME_REF_RE = re.compile(r"[A-Za-z_$][A-Za-z0-9_$,]*")
DOUBLE_BRACKET_LITERAL_PATTERNS = [
    "constant_range_expression",
    "constant_part_select_range",
    "part_select_range",
    "range_expression",
    "constant_expression",
]

# Rules where `[...]` in the LRM body is actually literal bracket punctuation and
# cannot be treated as EBNF optionality after markdown conversion strips formatting.
LITERAL_BRACKET_RULES = {
    "bit_select",
    "constant_bit_select",
    "unpacked_dimension",
    "associative_dimension",
    "queue_dimension",
    "dynamic_array_new",
}

# Rules where outer `{...}` denotes literal SV braces, not EBNF repetition.
LITERAL_BRACE_RULES = {
    "concatenation",
    "constant_concatenation",
    "multiple_concatenation",
    "constant_multiple_concatenation",
    "net_lvalue",
    "variable_lvalue",
    "assignment_pattern",
    "assignment_pattern_net_lvalue",
    "assignment_pattern_variable_lvalue",
    "pattern",
    "empty_unpacked_array_concatenation",
    "streaming_concatenation",
}

PUNCTUATION_TOKEN_NAMES = {
    ";": "semi",
    ",": "comma",
    ":": "colon",
    ".": "dot",
    ".*": "dot_star",
    "?": "question",
    "#": "hash",
    "@": "at_sign",
    "*": "star",
    "+": "plus",
    "-": "minus",
    "/": "slash",
    "%": "percent",
    "!": "bang",
    "~": "tilde",
    "=": "assign",
    "<=": "less_equal",
    ">=": "greater_equal",
    "<": "less_than",
    ">": "greater_than",
    "<<": "shift_left",
    ">>": "shift_right",
    "<<<": "arithmetic_shift_left",
    ">>>": "arithmetic_shift_right",
    "**": "power",
    "++": "plus_plus",
    "--": "minus_minus",
    "+=": "plus_assign",
    "-=": "minus_assign",
    "*=": "star_assign",
    "/=": "slash_assign",
    "%=": "percent_assign",
    "&=": "and_assign",
    "|=": "or_assign",
    "^=": "xor_assign",
    "<<=": "shift_left_assign",
    ">>=": "shift_right_assign",
    "<<<=": "arithmetic_shift_left_assign",
    ">>>=": "arithmetic_shift_right_assign",
    "||": "logical_or",
    "&&": "logical_and",
    "&&&": "logical_and3",
    "|": "bitwise_or",
    "^": "bitwise_xor",
    "&": "bitwise_and",
    "~&": "reduction_nand",
    "~|": "reduction_nor",
    "~^": "reduction_xnor",
    "^~": "reduction_xnor_alt",
    "==": "equal",
    "!=": "not_equal",
    "===": "case_equal",
    "!==": "case_not_equal",
    "==?": "wildcard_equal",
    "!=?": "wildcard_not_equal",
    "::": "scope_resolution",
    "(": "lparen",
    ")": "rparen",
    "{": "lbrace",
    "}": "rbrace",
    "[": "lbrack",
    "]": "rbrack",
    "_": "underscore",
    "'": "tick",
    "->": "implies",
    "<->": "iff_arrow",
    "=>": "sequence_implies",
    "*>": "full_path_arrow",
    "(*": "attr_open",
    "*)": "attr_close",
}
PUNCTUATION_LITERAL_BY_NAME = {
    token_name: literal for literal, token_name in PUNCTUATION_TOKEN_NAMES.items()
}

TOKEN_RE = re.compile(
    r"__SV_[A-Z]+__"
    r'|"[^"\n]*"'
    r"|\(\*|\*\)|\.\*|::=|::|##\[\*\]|##\[\+\]|##\[=\]|##|<->|->|=>|\*>"
    r"|<<<=|>>>=|<<<|>>>|<<=|>>=|===|!==|==\?|!=\?|<=|>=|\+\+|--|\+=|-=|\*=|/=|%=|&=|\|=|\^=|&&&|&&|\|\||~&|~\||~\^|\^~|<<|>>|\*\*|==|!="
    r"|[\[\]{}(),;:.?#@*/+\-=%!~&|^'_]"
    r"|[A-Za-z_$`\\][A-Za-z0-9_$`\\-]*"
    r"|[0-9][A-Za-z0-9_$']*"
)

MANUAL_RULE_BODIES = OrderedDict(
    [
        ("compiler_directive", 'trivia /`[^\\r\\n]*/'),
        (
            "binary_operator",
            "plus | minus | star | slash | percent | equal | not_equal | case_equal | "
            "case_not_equal | wildcard_equal | wildcard_not_equal | logical_and | "
            "logical_or | power | less_than | less_equal | greater_than | "
            "greater_equal | bitwise_and | bitwise_or | bitwise_xor | "
            "reduction_xnor_alt | reduction_xnor | shift_right | shift_left | "
            "arithmetic_shift_right | arithmetic_shift_left | implies | iff_arrow",
        ),
        (
            "assignment_operator",
            "assign | plus_assign | minus_assign | star_assign | slash_assign | "
            "percent_assign | and_assign | or_assign | xor_assign | "
            "shift_left_assign | shift_right_assign | arithmetic_shift_left_assign | "
            "arithmetic_shift_right_assign",
        ),
        (
            "constant_expression",
            "constant_primary ( binary_operator attribute_instance* constant_primary )* | "
            "unary_operator attribute_instance* constant_primary | "
            "constant_primary question attribute_instance* constant_expression colon constant_expression",
        ),
        (
            "expression",
            "primary ( binary_operator attribute_instance* primary )* | "
            "unary_operator attribute_instance* primary | inc_or_dec_expression | "
            "lparen operator_assignment rparen | conditional_expression | "
            "inside_expression | tagged_union_expression",
        ),
        ("identifier", "escaped_identifier | simple_identifier"),
        ("simple_identifier", "trivia /[a-zA-Z_][a-zA-Z0-9_$]*/"),
        ("escaped_identifier", r"trivia /\\[!-~]+/"),
        ("number", "real_number | integral_number"),
        (
            "integral_number",
            "trivia /[0-9][0-9_]*/ | trivia /[0-9][0-9_]*'[sS]?[dDhHoObB][0-9a-fA-FxXzZ?_]+/",
        ),
        (
            "real_number",
            "trivia /[0-9][0-9_]*\\.[0-9][0-9_]*([eE][+-]?[0-9][0-9_]*)?/ | trivia /[0-9][0-9_]*([eE][+-]?[0-9][0-9_]*)/",
        ),
        ("unsigned_number", "integral_number"),
        ("string_literal", 'trivia /"([^"\\\\]|\\\\.)*"/'),
        ("unbased_unsized_literal", "trivia /'(0|1|x|X|z|Z)/"),
        ("binary_number", "integral_number"),
        ("octal_number", "integral_number"),
        ("decimal_number", "integral_number"),
        ("hex_number", "integral_number"),
        ("size", "unsigned_number"),
        ("fixed_point_number", "real_number"),
        ("time_literal", "number time_unit"),
        ("c_identifier", "identifier"),
        ("system_tf_identifier", r"trivia /\$[a-zA-Z0-9_$]+/"),
        ("identifier_code", r'trivia /[^\\r\\n]*/'),
        ("comment_text", r'trivia /([^*]|\*+[^*/])*/'),
        (
            "generate_block",
            "kw_begin_8cbd0a74 ( colon generate_block_identifier )? generate_item* "
            "kw_end_7a92f3d2 ( colon generate_block_identifier )? | "
            "generate_block_identifier colon kw_begin_8cbd0a74 "
            "( colon generate_block_identifier )? generate_item* "
            "kw_end_7a92f3d2 ( colon generate_block_identifier )? | generate_item",
        ),
        (
            "trivia",
            "(white_space | line_comment | block_comment)*",
        ),
        ("white_space", r"/[ \t\r\n]+/"),
        ("line_comment", r"/\/\/[^\n]*(\n|$)/"),
        ("block_comment", r"/\/\*([^*]|\*+[^*/])*\*+\//"),
        ("modport_simple_ports_declaration", "port_direction modport_simple_port"),
    ]
)

SPECIAL_WORD_TOKEN_NAMES = {
    "$": "kw_dollar",
}

ACTIVE_GRAMMAR_PREAMBLE_LINES = [
    "# systemverilog.ebnf",
    "#",
    "# Active profile-aware IEEE 1800 SystemVerilog grammar.",
    "#",
    "# This file is the promoted single-file grammar used by the normal HDL flow. It",
    "# keeps stable entry rules (`systemverilog_file`, `systemverilog_parseable_file`)",
    "# and carries the clause-derived dual-profile surface for:",
    "# - `sv_2017` from `docs/systemverilog/2017/md/section-41-data-read-api.md`",
    "# - `sv_2023` from `docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md`",
    "#",
    "# The profile synthesis artifacts remain alongside this file:",
    "# - `systemverilog_lrm_profiled_generated.ebnf`",
    "# - `systemverilog_lrm_profiled_wrapper.ebnf`",
    "#",
    "# The active grammar is flattened into one file because the legacy Perl EBNF",
    "# frontend used in the current HDL pipeline does not expand `include(...)`.",
    "",
    "@branch_policy: priority_first",
    "@priority: [12, 3, 2, 1, 1]",
    "@coverage_target: 4",
    "@critical_path: true",
    "systemverilog_file := source_text",
    "",
    "@branch_policy: priority_first",
    "@priority: [20, 6, 4, 2]",
    "systemverilog_parseable_file := parseable_source_item*",
    "",
    "@branch_policy: priority_first",
    "@priority: [20, 6, 4, 2]",
    "parseable_source_item := semi",
    "                       | package_import_declaration",
    "                       | timeunits_declaration",
    "                       | compiler_directive",
    "",
]


@dataclass(frozen=True)
class Atom:
    value: str


@dataclass(frozen=True)
class Sequence:
    items: tuple[object, ...]


@dataclass(frozen=True)
class Alternation:
    items: tuple[object, ...]


@dataclass(frozen=True)
class OptionalNode:
    item: object


@dataclass(frozen=True)
class RepeatNode:
    item: object


def normalize_text(text: str) -> str:
    return (
        text.replace("‘", "`")
        .replace("’", "`")
        .replace("“", '"')
        .replace("”", '"')
        .replace("—", "-")
        .replace("–", "-")
        .replace("\u00a0", " ")
    )


def canonicalize_rule_name(name: str) -> str:
    name = normalize_text(name).strip()
    name = name.replace(":: =", "::=")
    name = re.sub(r"(\d+(,\d+)*)$", "", name)
    if name.startswith("$"):
        name = "sv_dollar_" + name[1:]
    name = name.replace("$", "_dollar_")
    name = name.replace("-", "_")
    name = re.sub(r"[^A-Za-z0-9_]", "_", name)
    name = re.sub(r"_+", "_", name).strip("_")
    if not name:
        return "sv_rule"
    if not re.match(r"^[A-Za-z_]", name):
        name = "sv_" + name
    return name


def is_noise_line(line: str) -> bool:
    s = line.strip()
    if not s:
        return False
    if s.startswith("```"):
        return True
    if s.startswith(("---", "title:", "document:", "standard:", "domain:", "section:", "source_txt:", "source_pdf:")):
        return True
    if s.startswith(
        (
            "Authorized licensed use limited to:",
            "IEEE Std 1800-",
            "IEEE Standard for SystemVerilog",
            "Copyright ©",
            "# Section ",
        )
    ):
        return True
    if "Downloaded on" in s or "Restrictions apply" in s:
        return True
    return bool(HEADING_RE.match(s))


def looks_like_grammar_line(line: str) -> bool:
    s = line.strip()
    if not s or is_noise_line(s):
        return False
    return bool(GRAMMAR_LINE_RE.fullmatch(s))


def extract_rules(markdown_path: Path) -> OrderedDict[str, list[str]]:
    lines = normalize_text(markdown_path.read_text(encoding="utf-8", errors="replace")).splitlines()
    rules: OrderedDict[str, list[str]] = OrderedDict()
    idx = 0
    while idx < len(lines):
        match = RULE_HEAD_RE.match(lines[idx])
        if not match:
            idx += 1
            continue
        raw_name = match.group(1)
        canonical_name = canonicalize_rule_name(raw_name)
        body_parts: list[str] = []
        inline = match.group(2).strip()
        if inline:
            body_parts.append(inline)
        idx += 1
        while idx < len(lines):
            if RULE_HEAD_RE.match(lines[idx]):
                break
            stripped = lines[idx].strip()
            if stripped.startswith("```") or is_noise_line(stripped):
                idx += 1
                continue
            if not stripped:
                idx += 1
                continue
            if not looks_like_grammar_line(lines[idx]):
                break
            body_parts.append(stripped)
            idx += 1
        body = re.sub(r"\s+", " ", " ".join(body_parts)).strip()
        rules.setdefault(canonical_name, [])
        if body:
            rules[canonical_name].append(body)
    return rules


def unique_preserve_order(items: Iterable[str]) -> list[str]:
    seen: set[str] = set()
    out: list[str] = []
    for item in items:
        if item in seen:
            continue
        seen.add(item)
        out.append(item)
    return out


def normalize_rule_reference_candidates(body: str) -> str:
    def repl(match: re.Match[str]) -> str:
        token = match.group(0)
        if token == "$":
            return token
        if token in {"__", "sv"}:
            return token
        if token.startswith("__SV_"):
            return token
        if any(ch.isalpha() or ch == "_" or ch == "$" for ch in token):
            return canonicalize_rule_name(token)
        return token

    return RULE_NAME_REF_RE.sub(repl, body)


def _find_balanced_bracket(body: str, start: int) -> int:
    """Given body[start] is `[` or `{`, return the index of the matching close."""
    opener = body[start]
    if opener == "[":
        closer = "]"
    elif opener == "{":
        closer = "}"
    else:
        return -1
    depth = 1
    i = start + 1
    n = len(body)
    while i < n:
        c = body[i]
        if c == opener:
            depth += 1
        elif c == closer:
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


def _structural_bracket_rewrite_pass(body: str) -> tuple[str, bool]:
    """Single pass. Rewrite one structural delimiter-nesting occurrence.

    Two opposite directions handled:
    - `[ [ X ] ]` and `{ [ X ] }`: rewrite INNER brackets to literal markers.
      (Array-dim brackets sit inside a meta wrapper that owns optionality or
      repetition.)
    - `{ { X } }`: rewrite OUTER braces to literal markers.
      (Block-delimiter braces own the outside; the meta repeat lives inside.)
    """
    i = 0
    n = len(body)
    while i < n:
        ch = body[i]
        if ch in ("[", "{"):
            outer_close = _find_balanced_bracket(body, i)
            if outer_close >= 0:
                a = i + 1
                while a < outer_close and body[a].isspace():
                    a += 1
                if a < outer_close:
                    inner_ch = body[a]
                    inner_close = _find_balanced_bracket(body, a)
                    if 0 <= inner_close < outer_close:
                        tail = body[inner_close + 1 : outer_close]
                        if tail.strip() == "":
                            if inner_ch == "[":
                                new_body = (
                                    body[:a]
                                    + "__SV_LBRACK__"
                                    + body[a + 1 : inner_close]
                                    + "__SV_RBRACK__"
                                    + body[inner_close + 1 :]
                                )
                                return new_body, True
                            if inner_ch == "{" and ch == "{":
                                new_body = (
                                    body[:i]
                                    + "__SV_LBRACE__"
                                    + body[i + 1 : outer_close]
                                    + "__SV_RBRACE__"
                                    + body[outer_close + 1 :]
                                )
                                return new_body, True
        i += 1
    return body, False


def apply_structural_literal_bracket_rewrite(body: str) -> str:
    """Structurally recover literal SV delimiter tokens from LRM nested metasyntax.

    The IEEE 1800 LRM uses the same `[ ... ]` and `{ ... }` character sequences
    for both metasyntax (optionality, repetition) and literal SV tokens (array
    dimensions, block-delimiter braces). After markdown extraction there is no
    typographic signal to distinguish them, but there is a reliable structural
    one: nested metasyntax that wraps only another delimiter group (with
    nothing else inside) is semantically redundant as pure metasyntax, so one
    of the pairs must be literal. Which one depends on the SV semantics of
    each delimiter family:

    Brackets — array-dim style, outer is meta, inner is literal:
        [ [ X ] ]   ->  [ __SV_LBRACK__ X __SV_RBRACK__ ]
        [ [ ] ]     ->  [ __SV_LBRACK__ __SV_RBRACK__ ]
        { [ X ] }   ->  { __SV_LBRACK__ X __SV_RBRACK__ }

    Braces — block-delimiter style, outer is literal, inner is meta:
        { { X } }   ->  __SV_LBRACE__ { X } __SV_RBRACE__

    Iterates until fixed point so deeper nestings like `[ [ [ X ] ] ]`
    decompose as outer-meta / middle-literal / inner-meta by repeated
    application.
    """
    changed = True
    while changed:
        body, changed = _structural_bracket_rewrite_pass(body)
    return body


def pre_rewrite_body(rule_name: str, body: str) -> str:
    body = normalize_text(body)
    body = re.sub(r"(?<=[A-Za-z_)\]}])(\d+(,\d+)*)", "", body)
    body = normalize_rule_reference_candidates(body)
    body = apply_structural_literal_bracket_rewrite(body)
    for inner in DOUBLE_BRACKET_LITERAL_PATTERNS:
        body = body.replace(
            f"[ [ {inner} ] ]",
            f"[ __SV_LBRACK__ {inner} __SV_RBRACK__ ]",
        )
    if rule_name == "bit_select":
        body = body.replace(
            "{ [ expression ] }",
            "{ __SV_LBRACK__ expression __SV_RBRACK__ }",
        )
    if rule_name == "constant_bit_select":
        body = body.replace(
            "{ [ constant_expression ] }",
            "{ __SV_LBRACK__ constant_expression __SV_RBRACK__ }",
        )
    if rule_name == "unpacked_dimension":
        body = body.replace(
            "[ constant_range ]",
            "__SV_LBRACK__ constant_range __SV_RBRACK__",
        )
        body = body.replace(
            "[ constant_expression ]",
            "__SV_LBRACK__ constant_expression __SV_RBRACK__",
        )
    if rule_name == "packed_dimension":
        body = body.replace(
            "[ constant_range ]",
            "__SV_LBRACK__ constant_range __SV_RBRACK__",
        )
        body = body.replace(
            "[ constant_expression ]",
            "__SV_LBRACK__ constant_expression __SV_RBRACK__",
        )
    if rule_name == "unsized_dimension":
        body = body.replace("[ ]", "__SV_LBRACK__ __SV_RBRACK__")
    if rule_name == "associative_dimension":
        body = body.replace("[ data_type ]", "__SV_LBRACK__ data_type __SV_RBRACK__")
        body = body.replace("[ * ]", "__SV_LBRACK__ * __SV_RBRACK__")
    if rule_name == "queue_dimension":
        body = body.replace(
            "[ $ [ : constant_expression ] ]",
            "__SV_LBRACK__ $ [ : constant_expression ] __SV_RBRACK__",
        )
    if rule_name == "dynamic_array_new":
        body = body.replace(
            "new [ expression ] [ ( expression ) ]",
            "new __SV_LBRACK__ expression __SV_RBRACK__ [ ( expression ) ]",
        )
    return re.sub(r"\s+", " ", body).strip()


def tokenize(body: str) -> list[str]:
    body = pre_tokenize_cleanup(body)
    tokens = TOKEN_RE.findall(body)
    if not tokens:
        return []
    return tokens


def pre_tokenize_cleanup(body: str) -> str:
    body = body.replace("__SV_LBRACK__", " __SV_LBRACK__ ")
    body = body.replace("__SV_RBRACK__", " __SV_RBRACK__ ")
    body = body.replace("__SV_LBRACE__", " __SV_LBRACE__ ")
    body = body.replace("__SV_RBRACE__", " __SV_RBRACE__ ")
    return re.sub(r"\s+", " ", body).strip()


def is_literal_vertical_bar(tokens: list[str], index: int, current_sequence_empty: bool) -> bool:
    if tokens[index] != "|":
        return False
    if not current_sequence_empty:
        return False
    return True


def parse_expression(rule_name: str, tokens: list[str], stop_tokens: set[str] | None = None, index: int = 0) -> tuple[object, int]:
    stop_tokens = stop_tokens or set()
    alternatives: list[object] = []
    current: list[object] = []
    while index < len(tokens):
        token = tokens[index]
        if token in stop_tokens:
            break
        if token == "|" and not is_literal_vertical_bar(tokens, index, not current):
            alternatives.append(build_sequence(current))
            current = []
            index += 1
            continue
        node, index = parse_item(rule_name, tokens, index)
        if node is not None:
            current.append(node)
    if current or not alternatives:
        alternatives.append(build_sequence(current))
    if len(alternatives) == 1:
        return alternatives[0], index
    return Alternation(tuple(alternatives)), index


def parse_item(rule_name: str, tokens: list[str], index: int) -> tuple[object | None, int]:
    token = tokens[index]
    if token == "[":
        inner, next_index = parse_expression(rule_name, tokens, {"]"}, index + 1)
        next_index += 1
        return OptionalNode(inner), next_index
    if token == "{":
        inner, next_index = parse_expression(rule_name, tokens, {"}"}, index + 1)
        next_index += 1
        if should_treat_braces_as_literal(rule_name, inner):
            return Sequence((Atom("lbrace"), inner, Atom("rbrace"))), next_index
        return RepeatNode(inner), next_index
    if token == "__SV_LBRACK__":
        return Atom("lbrack"), index + 1
    if token == "__SV_RBRACK__":
        return Atom("rbrack"), index + 1
    if token == "__SV_LBRACE__":
        return Atom("lbrace"), index + 1
    if token == "__SV_RBRACE__":
        return Atom("rbrace"), index + 1
    return Atom(token), index + 1


def build_sequence(items: list[object]) -> object:
    flat: list[object] = []
    for item in items:
        if isinstance(item, Sequence):
            flat.extend(item.items)
        else:
            flat.append(item)
    if not flat:
        return Atom("epsilon")
    if len(flat) == 1:
        return flat[0]
    return Sequence(tuple(flat))


def reorder_declaration_alternatives(rule_name: str, node: object) -> object:
    if not isinstance(node, Alternation):
        return node

    preferred_heads: dict[str, tuple[str, ...]] = {
        "module_declaration": ("module_ansi_header", "module_nonansi_header"),
        "interface_declaration": ("interface_ansi_header", "interface_nonansi_header"),
    }
    preferred = preferred_heads.get(rule_name)
    if not preferred:
        return node

    head_order = {head: index for index, head in enumerate(preferred)}

    def sequence_head(alternative: object) -> str | None:
        if isinstance(alternative, Sequence) and alternative.items:
            first = alternative.items[0]
            if isinstance(first, Atom):
                return first.value
        if isinstance(alternative, Atom):
            return alternative.value
        return None

    indexed_items = list(enumerate(node.items))
    reordered = sorted(
        indexed_items,
        key=lambda pair: (head_order.get(sequence_head(pair[1]), len(head_order)), pair[0]),
    )
    return Alternation(tuple(item for _, item in reordered))


def is_simple_repeat_child(node: object) -> bool:
    return isinstance(node, Atom)


def should_treat_braces_as_literal(rule_name: str, node: object) -> bool:
    if rule_name in LITERAL_BRACE_RULES:
        if isinstance(node, Sequence) and node.items and isinstance(node.items[0], Atom):
            first = node.items[0].value
            if first in {"comma", ",", "|"}:
                return False
        return True
    if isinstance(node, Sequence) and node.items and isinstance(node.items[0], Atom):
        first = node.items[0].value
        if first in {"comma", ",", "dot", "."}:
            return False
    return False


def normalize_atom_token(token: str, rule_names: set[str], literal_tokens: dict[str, str]) -> str:
    if token == "epsilon":
        return token
    if token in PUNCTUATION_TOKEN_NAMES.values():
        literal = PUNCTUATION_LITERAL_BY_NAME[token]
        literal_tokens.setdefault(literal, token)
        return token
    if token in PUNCTUATION_TOKEN_NAMES:
        literal_tokens.setdefault(token, PUNCTUATION_TOKEN_NAMES[token])
        return PUNCTUATION_TOKEN_NAMES[token]
    if token.startswith('"') and token.endswith('"'):
        name = literal_tokens.setdefault(token, make_literal_token_name(token))
        return name
    if token in rule_names or token in MANUAL_RULE_BODIES:
        return token
    if token.startswith("__SV_"):
        raise ValueError(f"unexpected unhandled placeholder token: {token}")
    name = literal_tokens.setdefault(token, make_literal_token_name(token))
    return name


def make_literal_token_name(token: str) -> str:
    raw_token = token
    if token in SPECIAL_WORD_TOKEN_NAMES:
        return SPECIAL_WORD_TOKEN_NAMES[token]
    if token in PUNCTUATION_TOKEN_NAMES:
        return PUNCTUATION_TOKEN_NAMES[token]
    if token.startswith('"') and token.endswith('"'):
        token = token[1:-1]
        prefix = "lit"
    elif token.startswith("$"):
        prefix = "kw"
        token = "dollar_" + token[1:]
    elif token.startswith("`"):
        prefix = "kw"
        token = "bt_" + token[1:]
    else:
        prefix = "kw"
    token = token.replace("-", "_")
    token = token.replace("$", "_dollar_")
    token = token.replace("'", "_tick_")
    token = re.sub(r"[^A-Za-z0-9_]", "_", token)
    token = re.sub(r"_+", "_", token).strip("_")
    if not token:
        token = "token"
    if token[0].isdigit():
        token = "n_" + token
    digest = hashlib.sha1(raw_token.encode("utf-8")).hexdigest()[:8]
    return f"{prefix}_{token}_{digest}"


def serialize(node: object, rule_names: set[str], literal_tokens: dict[str, str]) -> str:
    if isinstance(node, Atom):
        atom = normalize_atom_token(node.value, rule_names, literal_tokens)
        return atom
    if isinstance(node, Sequence):
        return " ".join(serialize(item, rule_names, literal_tokens) for item in node.items)
    if isinstance(node, Alternation):
        return " | ".join(serialize(item, rule_names, literal_tokens) for item in node.items)
    if isinstance(node, OptionalNode):
        inner = serialize(node.item, rule_names, literal_tokens)
        return f"( {inner} )?"
    if isinstance(node, RepeatNode):
        inner = serialize(node.item, rule_names, literal_tokens)
        if is_simple_repeat_child(node.item):
            return f"{inner}*"
        return f"( {inner} )*"
    raise TypeError(f"unsupported node: {node!r}")


def emit_literal_token_rule(token: str, token_name: str) -> str:
    if token_name in PUNCTUATION_TOKEN_NAMES.values():
        literal = token
        return f'{token_name} := trivia "{escape_string_literal(literal)}"'
    if token.startswith('"') and token.endswith('"'):
        literal = token
        return f"{token_name} := trivia {literal}"
    escaped = re.escape(token)
    if re.match(r"^[A-Za-z0-9_$`\\-]+$", token):
        return f"{token_name} := trivia /{escaped}\\b/"
    return f'{token_name} := trivia "{escape_string_literal(token)}"'


def escape_string_literal(text: str) -> str:
    return text.replace("\\", "\\\\").replace('"', '\\"')


def emit_profiled_rules(
    versions: OrderedDict[str, OrderedDict[str, list[str]]],
) -> tuple[list[str], dict[str, str], dict[str, object]]:
    all_rule_names = set(MANUAL_RULE_BODIES.keys())
    for version_rules in versions.values():
        all_rule_names.update(version_rules.keys())

    literal_tokens: dict[str, str] = {}
    emitted_lines: list[str] = []
    report_rules: dict[str, object] = {}

    unified_names = sorted(set().union(*(set(rules.keys()) for rules in versions.values())))
    for rule_name in unified_names:
        if rule_name in MANUAL_RULE_BODIES:
            continue
        per_version_serialized: OrderedDict[str, str | None] = OrderedDict()
        for profile, rules in versions.items():
            bodies = unique_preserve_order(rules.get(rule_name, []))
            if not bodies:
                per_version_serialized[profile] = None
                continue
            translated_bodies: list[str] = []
            for body in bodies:
                prepared = pre_rewrite_body(rule_name, body)
                tokens = tokenize(prepared)
                expr, consumed = parse_expression(rule_name, tokens)
                if consumed != len(tokens):
                    raise ValueError(
                        f"rule '{rule_name}' ({profile}) was only partially consumed: "
                        f"{tokens[consumed:]}"
                    )
                expr = reorder_declaration_alternatives(rule_name, expr)
                translated_bodies.append(serialize(expr, all_rule_names, literal_tokens))
            per_version_serialized[profile] = " | ".join(unique_preserve_order(translated_bodies))

        present_profiles = [profile for profile, body in per_version_serialized.items() if body]
        if not present_profiles:
            continue
        unique_bodies = unique_preserve_order(
            body for body in per_version_serialized.values() if body is not None
        )

        report_rules[rule_name] = {
            "profiles": {profile: body for profile, body in per_version_serialized.items() if body},
            "profile_count": len(present_profiles),
            "variant_count": len(unique_bodies),
            "shared_body": unique_bodies[0] if len(unique_bodies) == 1 else None,
        }

        if len(unique_bodies) == 1 and len(present_profiles) == len(versions):
            emitted_lines.append(f"{rule_name} := {unique_bodies[0]}")
            emitted_lines.append("")
            continue

        wrapper_targets: list[str] = []
        for profile, body in per_version_serialized.items():
            if not body:
                continue
            profiled_rule_name = f"{rule_name}_{profile}"
            emitted_lines.append(f'@profiles: ["{profile}"]')
            emitted_lines.append(f"{profiled_rule_name} := {body}")
            emitted_lines.append("")
            wrapper_targets.append(profiled_rule_name)
        emitted_lines.append(f"{rule_name} := {' | '.join(wrapper_targets)}")
        emitted_lines.append("")

    return emitted_lines, literal_tokens, report_rules


def load_version_rules(paths: dict[str, Path]) -> OrderedDict[str, OrderedDict[str, list[str]]]:
    versions: OrderedDict[str, OrderedDict[str, list[str]]] = OrderedDict()
    for profile, path in paths.items():
        versions[profile] = extract_rules(path)
    return versions


def emit_manual_rule_lines(literal_tokens: dict[str, str]) -> list[str]:
    lines: list[str] = []
    for rule_name, body in MANUAL_RULE_BODIES.items():
        for token in body.split():
            if token in PUNCTUATION_TOKEN_NAMES.values():
                literal = PUNCTUATION_LITERAL_BY_NAME[token]
                literal_tokens.setdefault(literal, token)
            elif token in PUNCTUATION_TOKEN_NAMES:
                literal_tokens.setdefault(token, PUNCTUATION_TOKEN_NAMES[token])
        lines.append(f"{rule_name} := {body}")
        lines.append("")
    return lines


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--md-2017", required=True, help="2017 Annex markdown path")
    ap.add_argument("--md-2023", required=True, help="2023 Annex markdown path")
    ap.add_argument("--output-ebnf", required=True, help="Output profile-aware EBNF path")
    ap.add_argument(
        "--output-active-ebnf",
        help="Optional output path for the flattened active systemverilog.ebnf grammar",
    )
    ap.add_argument("--output-report", required=True, help="Output JSON report path")
    args = ap.parse_args()

    version_paths = OrderedDict(
        [
            ("sv_2017", Path(args.md_2017).expanduser().resolve()),
            ("sv_2023", Path(args.md_2023).expanduser().resolve()),
        ]
    )
    for path in version_paths.values():
        if not path.is_file():
            raise SystemExit(f"error: markdown source not found: {path}")

    versions = load_version_rules(version_paths)
    profiled_lines, literal_tokens, report_rules = emit_profiled_rules(versions)

    output_lines: list[str] = []
    output_lines.append("# Auto-generated by tools/extract_systemverilog_lrm_profiles.py")
    output_lines.append("# Source markdowns:")
    for profile, path in version_paths.items():
        output_lines.append(f"# - {profile}: {path}")
    output_lines.append("")
    output_lines.append("# Shared lexical foundation")
    output_lines.append("")
    output_lines.extend(emit_manual_rule_lines(literal_tokens))
    output_lines.append("# Profile-aware LRM-derived syntax")
    output_lines.append("")
    output_lines.extend(profiled_lines)
    output_lines.append("# Generated literal tokens")
    output_lines.append("")
    for literal, token_name in sorted(literal_tokens.items(), key=lambda item: item[1]):
        if token_name in MANUAL_RULE_BODIES:
            continue
        output_lines.append(emit_literal_token_rule(literal, token_name))
        output_lines.append("")

    output_path = Path(args.output_ebnf).expanduser().resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(output_lines).rstrip() + "\n", encoding="utf-8")

    if args.output_active_ebnf:
        active_output_path = Path(args.output_active_ebnf).expanduser().resolve()
        active_output_path.parent.mkdir(parents=True, exist_ok=True)
        active_output_lines = list(ACTIVE_GRAMMAR_PREAMBLE_LINES)
        active_output_lines.extend(output_lines)
        active_output_path.write_text(
            "\n".join(active_output_lines).rstrip() + "\n", encoding="utf-8"
        )

    report = {
        "profiles": {
            profile: {
                "source_markdown": str(path),
                "extracted_rule_count": len(rules),
            }
            for profile, (path, rules) in zip(version_paths.keys(), zip(version_paths.values(), versions.values()))
        },
        "generated_rule_count": len(report_rules) + len(MANUAL_RULE_BODIES),
        "generated_literal_token_count": len(literal_tokens),
        "rules": report_rules,
    }
    report_path = Path(args.output_report).expanduser().resolve()
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print(f"md_2017: {version_paths['sv_2017']}")
    print(f"md_2023: {version_paths['sv_2023']}")
    print(f"extracted_2017_rules: {len(versions['sv_2017'])}")
    print(f"extracted_2023_rules: {len(versions['sv_2023'])}")
    print(f"output_ebnf: {output_path}")
    if args.output_active_ebnf:
        print(
            f"output_active_ebnf: {Path(args.output_active_ebnf).expanduser().resolve()}"
        )
    print(f"output_report: {report_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
