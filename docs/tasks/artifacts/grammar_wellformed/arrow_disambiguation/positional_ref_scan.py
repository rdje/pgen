#!/usr/bin/env python3
"""GRAMMAR-WELLFORMED — POSITIONAL-REF-SCAN: does a `-> {... $N ...}` point at a SEPARATOR?

⛔ WHY: `$N` in a return annotation indexes EVERY top-level element of the rule body — the
`/\\s*/` layout regexes included. A `$N` that lands on such a separator always resolves to the
EMPTY STRING, so the field is published as `""` and the real payload is silently discarded.
Nothing reports it: the grammar lints clean, the rule is witnessed, and the parse ACCEPTS.

Founding defect: `semantic_annotation := "@" /\\s*/ annotation_name /\\s*/ ":" /\\s*/ annotation_value`
carries `-> {…, name: $3, value: $6}`. Element 6 is the third `/\\s*/`; `annotation_value` is
element 7. Measured: the entry rule's typed AST publishes `value: ""` for 12 of 12 value shapes.

Reads the normalized gen-AST (`--dump-gen-ast`, TOOLBOX 5.2) — the same IR codegen consumes.

⛔ HONEST BOUND: only rules whose body is a FLAT top-level `Sequence` are scanned; anything else
(a top-level choice, a quantified body) is reported as UNSCANNED rather than passed silently.

usage: positional_ref_scan.py <gen_ast.json>
"""
import json
import re
import sys

SEP = re.compile(r'^[\\\\s*+?|()\[\]]*$')  # a token that can match the empty string and carries no payload


def elements(node):
    if isinstance(node, dict) and "Sequence" in node:
        return node["Sequence"].get("elements", [])
    return None


def token_of(el):
    try:
        tok = el["Atom"]["value"]["Token"]
        return tok[0]["String"], tok[1]["String"]
    except Exception:
        return None, None


def refs(parsed):
    """every PositionalRef index appearing anywhere in a parsed return annotation"""
    out = []
    if isinstance(parsed, dict):
        for k, v in parsed.items():
            if k == "PositionalRef" and isinstance(v, dict) and "index" in v:
                out.append(v["index"])
            else:
                out += refs(v)
    elif isinstance(parsed, list):
        for v in parsed:
            out += refs(v)
    return out


def main(path):
    d = json.load(open(path))
    tree = d["grammar_tree"]
    anns = d["annotations"]["branch_return_annotations"]
    flagged = scanned = unscanned = 0
    for rule, branches in sorted(anns.items()):
        body = tree.get(rule)
        els = elements(body)
        if els is None:
            unscanned += 1
            continue
        scanned += 1
        for b in branches:
            for idx in sorted(set(refs(b.get("parsed_ast")))):
                if not (1 <= idx <= len(els)):
                    print(f"POSITIONAL-REF-SCAN: rule={rule:<28} ${idx} OUT OF RANGE "
                          f"(rule has {len(els)} elements)")
                    flagged += 1
                    continue
                kind, val = token_of(els[idx - 1])
                if kind == "regex" and SEP.match(val or ""):
                    print(f"POSITIONAL-REF-SCAN: rule={rule:<28} ${idx} points at SEPARATOR "
                          f"/{val}/ — always resolves to \"\"   |   {b.get('annotation_content')}")
                    flagged += 1
    print(f"POSITIONAL-REF-SCAN: rules_with_return_annotations={len(anns)} "
          f"flat_sequence_scanned={scanned} unscanned_non_sequence={unscanned} flagged={flagged}")
    return 1 if flagged else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
