#!/usr/bin/env python3
r"""SV-CORPUS-GRAD.3.44 — PARTIAL-TYPING CENSUS: which multi-alternative rules carry a return template on SOME
top-level alternatives and not on others?

`function_body_declaration` was listed as typed in every register that counts annotations (the json-carrier row
showed its alternative-1 template; the annotation inventory counted the one annotation) while three of its four
alternatives — the ported form being the most common function in any corpus — emitted the raw `Sequence`
envelope for the whole life of the parser. A rule is typed when EVERY alternative carries a template; a rule
with one template and four alternatives is a partial-typing claim. This census reads the EBNF frontend's own
`raw_ast` envelope (`ast_pipeline <grammar> --emit-raw-ast-json <out>`, the same reading the code generator
consumes) and, per rule, counts its top-level alternatives (`|` operators at group depth 0) and which of them
end in a `return_*` annotation at depth 0.

Usage:
  ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --emit-raw-ast-json rust/target/sNNN/raw.json
  python3 docs/tasks/artifacts/sv_corpus_grad/tf_body_census/partial_typing_census.py rust/target/sNNN/raw.json [--list]
  python3 docs/tasks/artifacts/sv_corpus_grad/tf_body_census/partial_typing_census.py --self-test

Headline: `PARTIAL-TYPING-CENSUS: grammar=<json> rules=N multi_alternative=M fully_typed=F untyped=U PARTIAL=P
partial_untyped_alternatives=A passthrough=B RAW_ENVELOPE=C` — `PARTIAL` rules are the class. Each untyped
alternative of a PARTIAL rule is bucketed by what it EMITS: `passthrough` when the alternative is exactly one
rule reference (the child's own node passes through unwrapped — the profile-split `_sv_only` / `_v2005` helper
pattern, typed by design one level down) and `RAW_ENVELOPE` when it holds two or more elements (a terminal, a
group, a quantifier, a second reference), because such an alternative emits the positional `Sequence` envelope —
the shape this slice removed from `function_body_declaration` / `task_body_declaration`. `--list` prints each
PARTIAL rule with the indices of its untyped alternatives and each one's bucket. Exit 0; exit 7 on a self-test miss.
⚠️ HONEST BOUND: a rule whose EVERY alternative is untyped is `untyped` here, not `PARTIAL` — the campaign's
ordinary backlog, a different (larger) population the annotation inventory already sees; and a single-reference
passthrough may still be an omission (a child that is itself raw) — the bucket names the mechanism, not intent.
"""
import json, sys

RETURN_TAGS = {"return_scalar", "return_array", "return_object"}


ELEMENT_TAGS = {"rule_reference", "quoted_string", "regex", "group_open"}


def census_rule(items):
    """items: the rule's raw_ast entry (a list of [tag, payload] items, the first being ["rule", name]).
    Returns (name, n_alternatives, typed_alternative_indices, elements_per_alternative) where an element is a
    reference, a terminal or a group at depth 0 (quantifiers and annotations are not elements)."""
    name = items[0][1]
    depth = 0; alt = 0; typed = set(); elements = [0]
    for it in items[1:]:
        tag = it[0]
        if tag == "group_open":
            if depth == 0:
                elements[alt] += 1
            depth += 1
        elif tag == "group_close":
            depth -= 1
        elif tag == "operator" and it[1] == "|" and depth == 0:
            alt += 1; elements.append(0)
        elif tag in RETURN_TAGS and depth == 0:
            typed.add(alt)
        elif tag in ELEMENT_TAGS and depth == 0:
            elements[alt] += 1
    return name, alt + 1, typed, elements


def bucket(n_elements):
    return "passthrough" if n_elements == 1 else "RAW_ENVELOPE"


def census(raw_ast):
    out = []
    for ent in raw_ast:
        if not ent or ent[0][0] != "rule":
            continue
        out.append(census_rule(ent))
    return out


def main(argv):
    if argv and argv[0] == "--self-test":
        return self_test()
    if not argv:
        print(__doc__); return 2
    path = argv[0]; want_list = "--list" in argv
    raw = json.load(open(path))["raw_ast"]
    rows = census(raw)
    multi = [r for r in rows if r[1] > 1]
    full = [r for r in multi if len(r[2]) == r[1]]
    none = [r for r in multi if not r[2]]
    partial = [r for r in multi if r[2] and len(r[2]) < r[1]]
    untyped_alts = sum(r[1] - len(r[2]) for r in partial)
    pt = sum(1 for r in partial for i in range(r[1]) if i not in r[2] and bucket(r[3][i]) == "passthrough")
    raw = untyped_alts - pt
    print("PARTIAL-TYPING-CENSUS: grammar=%s rules=%d multi_alternative=%d fully_typed=%d untyped=%d PARTIAL=%d partial_untyped_alternatives=%d passthrough=%d RAW_ENVELOPE=%d"
          % (path, len(rows), len(multi), len(full), len(none), len(partial), untyped_alts, pt, raw))
    if want_list:
        for name, n, typed, els in sorted(partial):
            missing = ["%d:%s" % (i + 1, bucket(els[i])) for i in range(n) if i not in typed]
            print("  PARTIAL\t%s\talternatives=%d\tuntyped=%s" % (name, n, ",".join(missing)))
    return 0


def self_test():
    ent_partial = [["rule", "r"], ["rule_reference", "a"], ["return_object", "{x: $1}"], ["operator", "|"], ["rule_reference", "b"],
                   ["operator", "|"], ["group_open", "("], ["rule_reference", "c"], ["operator", "|"], ["rule_reference", "d"], ["group_close", ")"],
                   ["return_object", "{y: $1}"]]
    ent_full = [["rule", "f"], ["rule_reference", "a"], ["return_scalar", "$1"], ["operator", "|"], ["rule_reference", "b"], ["return_array", "[$1]"]]
    ent_none = [["rule", "n"], ["rule_reference", "a"], ["operator", "|"], ["rule_reference", "b"]]
    ent_single = [["rule", "s"], ["rule_reference", "a"]]
    ent_raw = [["rule", "w"], ["rule_reference", "a"], ["return_scalar", "$1"], ["operator", "|"], ["rule_reference", "b"], ["quoted_string", "';'"],
               ["operator", "|"], ["group_open", "("], ["rule_reference", "c"], ["group_close", ")"], ["operator", "?"], ["rule_reference", "d"]]
    got = {r[0]: (r[1], sorted(r[2]), [bucket(e) for e in r[3]]) for r in census([ent_partial, ent_full, ent_none, ent_single, ent_raw])}
    want = {"r": (3, [0, 2], ["passthrough", "passthrough", "passthrough"]), "f": (2, [0, 1], ["passthrough", "passthrough"]),
            "n": (2, [], ["passthrough", "passthrough"]), "s": (1, [], ["passthrough"]),
            "w": (3, [0], ["passthrough", "RAW_ENVELOPE", "RAW_ENVELOPE"])}
    if got != want:
        print("PARTIAL-TYPING-CENSUS SELF-TEST: MISS — %r != %r" % (got, want)); return 7
    print("PARTIAL-TYPING-CENSUS SELF-TEST: OK — a 3-alternative rule typed on 1 and 3 (the inline `|` inside a group is not an alternative), a fully typed pair, "
          "an untyped pair, a single-alternative rule, and a rule whose untyped alternatives are one passthrough-free sequence (`b ';'`) and one group-led "
          "sequence (`( c )? d`) — both RAW_ENVELOPE, the typed single reference a passthrough")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
