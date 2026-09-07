#!/usr/bin/env python3
r"""SV-CORPUS-GRAD.3.44 — classify every function / task BODY a corpus writes by the IEEE 1800-2017 A.2.6 / A.2.7
alternative that produced it, and by whether the parser typed it or left the raw `Sequence` envelope.

    function_body_declaration ::=
        function_data_type_or_implicit [ interface_identifier . | class_scope ] function_identifier ;
            { tf_item_declaration } { function_statement_or_null } endfunction [ : function_identifier ]
      | function_data_type_or_implicit [ interface_identifier . | class_scope ] function_identifier ( [ tf_port_list ] ) ;
            { block_item_declaration } { function_statement_or_null } endfunction [ : function_identifier ]
    task_body_declaration ::= (the same two forms, without the return type)
    (docs/systemverilog/2017/md/section-41-data-read-api.md:1090-1135; 1800-2023 Annex A :1125-1175 reads the same)

The grammar spells each production as FOUR alternatives — the optional out-of-block prefix × the optional port
list — and through parser release 1.0.217 only alternative 1 (`type f; … endfunction`) carried a return template.
Input: the TSV of `ast_field_census.py --select-keys lifetime,body --field body` (every node
`function_declaration_sv_2017` / `task_declaration_sv_2017` return — `{lifetime, body}` — and its body), or the
`--select-keys dynamic_override,lifetime,body` twin for the sv_2023 declarations.

Buckets (one per row; the name is `<function|task>:<alt>:<typed|RAW>`):
  function:alt1:typed   {return_type, name, items, statements, end_label}          — `type f; …`
  function:alt2:RAW     a 10-element envelope [rt, name, (, ports, ), ;, items, stmts, endkw, label]   — `type f(ports); …`
  function:alt3:RAW     an 8-element envelope [rt, scope, name, ;, items, stmts, endkw, label]          — `type I.f; …` / `type C::f; …`
  function:alt4:RAW     an 11-element envelope [rt, scope, name, (, ports, ), ;, items, stmts, endkw, label]
  task:alt1:typed       {name, items, statements, end_label}                                   — `t; …`
  task:alt2:RAW         a 9-element envelope [name, (, ports, ), ;, items, stmts, endkw, label]        — `t(ports); …`
  task:alt3:RAW         a 7-element envelope [scope, name, ;, items, stmts, endkw, label]
  task:alt4:RAW         a 10-element envelope [scope, name, (, ports, ), ;, items, stmts, endkw, label]
  function:altN:typed / task:altN:typed — the same alternatives once typed (release 1.0.218): `ports` present ⇔ the
                        ported form (alternatives 2 and 4), `scope` present ⇔ the out-of-block form (3 and 4)
  unclassified          a rendering this classifier does not know (printed; a census with any of these is not finished)

The typed and raw readings of one alternative are told apart by SHAPE, never by the release under test, so the
same classifier reads the HEAD census (where alternatives 2–4 are RAW) and the final-probe census (where every
row is typed) — the two runs must agree row for row on the alternative and differ only in the typed/RAW tag.

Usage:
  python3 docs/tasks/artifacts/sv_corpus_grad/tf_body_census/classify.py <census.tsv> [<census2.tsv> ...]
  python3 docs/tasks/artifacts/sv_corpus_grad/tf_body_census/classify.py --self-test

Prints `TF-BODY-CLASSIFY: <tsv> rows=N parser=P function=F task=T typed=A RAW=B unclassified=U` per file, the
per-bucket counts, every `unclassified` row, and a PREDICTION block: every ACCEPTED file carrying at least one RAW
row is a file whose typed AST MUST move when the alternatives are typed (the corpus AST identity sweep's
`ast_moved` set, TOOLBOX §1.5e), and every accepted file whose rows are all typed must be byte-identical.
Exit 0; exit 7 on a self-test miss.
"""
import collections, json, sys

FUNCTION_ALT1_KEYS = frozenset({"return_type", "name", "items", "statements", "end_label"})
TASK_ALT1_KEYS = frozenset({"name", "items", "statements", "end_label"})
OPTIONAL_KEYS = frozenset({"ports", "scope"})


def _is_token(x, kind):
    return isinstance(x, dict) and x.get("kind") == kind and len(x) == 1


def _is_name(x):
    return isinstance(x, dict) and set(x.keys()) == {"body"} and isinstance(x["body"], str)


def _is_return_type(x):
    return isinstance(x, dict) and x.get("kind") in ("data_type", "implicit") and "body" in x


def _is_scope(x):
    # `interface_identifier .` is the pair [<identifier>, {kind: "dot"}]; a class scope is the `class_scope`
    # node `{body: <class_scope_type>}` whose body is an object (never a bare string, unlike a name node)
    if isinstance(x, list):
        return len(x) == 2 and _is_name(x[0]) and _is_token(x[1], "dot")
    return isinstance(x, dict) and set(x.keys()) == {"body"} and isinstance(x["body"], (dict, list))


def _is_end_keyword(x, word):
    # `kw_endfunction_84b8f967` / `kw_endtask_eb64ab0d` render as [<trivia>, "endfunction"]
    return isinstance(x, list) and len(x) == 2 and x[1] == word


def classify_raw(v):
    """A raw `Sequence` envelope: decode the alternative from its marker tokens and element shapes."""
    n = len(v)
    if n == 0 or not isinstance(v, list):
        return "unclassified", "empty envelope"
    is_function = _is_return_type(v[0])
    fam = "function" if is_function else "task"
    endword = "endfunction" if is_function else "endtask"
    if n < 7 or not _is_end_keyword(v[-2], endword):
        return "unclassified", "no %s keyword at [-2] (len %d)" % (endword, n)
    off = 1 if is_function else 0                    # the return type occupies [0] on a function
    has_scope = _is_scope(v[off])
    name_at = off + (1 if has_scope else 0)
    if not _is_name(v[name_at]):
        return "unclassified", "no name at [%d] (len %d)" % (name_at, n)
    ported = _is_token(v[name_at + 1], "lparen")
    expect = (off + (1 if has_scope else 0) + 1) + (4 if ported else 1) + 4   # name(+scope+rt) + ((ports)); | ; + items stmts endkw label
    if n != expect:
        return "unclassified", "%s envelope of %d elements, expected %d (scope=%s ported=%s)" % (fam, n, expect, has_scope, ported)
    alt = 1 + (1 if ported else 0) + (2 if has_scope else 0)
    return "%s:alt%d:RAW" % (fam, alt), "%s %s" % (fam, ("ported " if ported else "") + ("out-of-block" if has_scope else "in-block"))


def classify_typed(v):
    keys = frozenset(v.keys())
    base = keys - OPTIONAL_KEYS
    if base == FUNCTION_ALT1_KEYS:
        fam = "function"
    elif base == TASK_ALT1_KEYS:
        fam = "task"
    else:
        return "unclassified", "typed object with keys %s" % sorted(keys)
    ported = "ports" in keys
    has_scope = "scope" in keys
    alt = 1 + (1 if ported else 0) + (2 if has_scope else 0)
    return "%s:alt%d:typed" % (fam, alt), "%s %s" % (fam, ("ported " if ported else "") + ("out-of-block" if has_scope else "in-block"))


def classify_json(v):
    if isinstance(v, list):
        return classify_raw(v)
    if isinstance(v, dict):
        return classify_typed(v)
    return "unclassified", json.dumps(v)[:80]


def read_rows(path):
    return [l.rstrip("\n").split("\t") for l in open(path).read().splitlines()[1:]]


def main_classify(paths):
    for path in paths:
        rows = read_rows(path)
        cnt = collections.Counter(); per_file_raw = collections.defaultdict(int); per_file_rows = collections.defaultdict(int)
        accepted_files = set(); unclassified = []; parser_rows = 0
        for f, status, idx, val in rows:
            if status == "accepted":
                accepted_files.add(f)
            if idx == "-" or idx.startswith("regex"):
                continue
            parser_rows += 1
            cat, _r = classify_json(json.loads(val))
            cnt[cat] += 1; per_file_rows[f] += 1
            if cat.endswith(":RAW"):
                per_file_raw[f] += 1
            if cat == "unclassified":
                unclassified.append((f, idx, _r))
        fn = sum(n for c, n in cnt.items() if c.startswith("function:")); tk = sum(n for c, n in cnt.items() if c.startswith("task:"))
        typed = sum(n for c, n in cnt.items() if c.endswith(":typed")); raw = sum(n for c, n in cnt.items() if c.endswith(":RAW"))
        print("TF-BODY-CLASSIFY: %s rows=%d parser=%d function=%d task=%d typed=%d RAW=%d unclassified=%d"
              % (path, len(rows), parser_rows, fn, tk, typed, raw, cnt["unclassified"]))
        for c, n in sorted(cnt.items(), key=lambda x: (x[0] == "unclassified", x[0])):
            print("  %6d  %s" % (n, c))
        for f, idx, r in unclassified:
            print("  unclassified\t%s\t%s\t%s" % (f, idx, r))
        movers = sorted(f for f in accepted_files if per_file_raw.get(f, 0) > 0)
        stays = sorted(f for f in accepted_files if per_file_raw.get(f, 0) == 0)
        print("PREDICTION accepted files carrying a RAW body (typed AST must move when the alternatives are typed): %d" % len(movers))
        for f in movers:
            print("  MOVES\t%s\tRAW=%d of %d bodies" % (f, per_file_raw[f], per_file_rows[f]))
        print("PREDICTION accepted files whose bodies are all typed (typed AST must be byte-identical): %d" % len(stays))
    return 0


def self_test():
    """Synthetic rows for all eight raw/typed shapes plus the refusals: each must land in its own bucket, and the
    envelope-length check must refuse a mis-shaped envelope rather than guess."""
    rt = {"kind": "data_type", "body": {"kind": "void"}}
    name = {"body": "f"}; tname = {"body": "t"}; lp = {"kind": "lparen"}; rp = {"kind": "rparen"}; semi = {"kind": "semi"}
    endf = [[], "endfunction"]; endt = [[], "endtask"]; label = []; iscope = [{"body": "I"}, {"kind": "dot"}]
    cscope = {"body": {"body": {"kind": "x"}}}
    arms = [
        ([rt, name, semi, [], [], endf, label], "function:alt1:RAW"),                             # the alt-1 shape, were it ever raw
        ([rt, name, lp, [], rp, semi, [], endf, label], "unclassified"),                          # 9 elements: a ported function missing a slot
        ([rt, name, lp, [], rp, semi, [], [], endf, label], "function:alt2:RAW"),
        ([rt, iscope, name, semi, [], [], endf, label], "function:alt3:RAW"),
        ([rt, cscope, name, lp, [], rp, semi, [], [], endf, label], "function:alt4:RAW"),
        ([tname, lp, [], rp, semi, [], [], endt, label], "task:alt2:RAW"),
        ([iscope, tname, semi, [], [], endt, label], "task:alt3:RAW"),
        ([cscope, tname, lp, [], rp, semi, [], [], endt, label], "task:alt4:RAW"),
        ({"return_type": rt, "name": name, "items": [], "statements": [], "end_label": []}, "function:alt1:typed"),
        ({"return_type": rt, "name": name, "ports": [], "items": [], "statements": [], "end_label": []}, "function:alt2:typed"),
        ({"return_type": rt, "scope": iscope, "name": name, "items": [], "statements": [], "end_label": []}, "function:alt3:typed"),
        ({"return_type": rt, "scope": cscope, "name": name, "ports": [], "items": [], "statements": [], "end_label": []}, "function:alt4:typed"),
        ({"name": tname, "items": [], "statements": [], "end_label": []}, "task:alt1:typed"),
        ({"name": tname, "ports": [], "items": [], "statements": [], "end_label": []}, "task:alt2:typed"),
        ({"scope": iscope, "name": tname, "items": [], "statements": [], "end_label": []}, "task:alt3:typed"),
        ({"scope": cscope, "name": tname, "ports": [], "items": [], "statements": [], "end_label": []}, "task:alt4:typed"),
        ({"name": tname, "items": [], "end_label": []}, "unclassified"),                          # a typed object missing a key
        ([rt, name, lp, [], rp, semi, [], [], [[], "endtask"], label], "unclassified"),           # a function ending in endtask
        ("a string", "unclassified"),
    ]
    misses = []
    for v, want in arms:
        got, _r = classify_json(v)
        if got != want:
            misses.append("%s -> %s (want %s)" % (json.dumps(v)[:70], got, want))
    if misses:
        print("TF-BODY-CLASSIFY SELF-TEST: MISS — " + "; ".join(misses)); return 7
    print("TF-BODY-CLASSIFY SELF-TEST: OK — %d arms (8 raw envelopes incl. one mis-shaped, 8 typed objects, 3 refusals)" % len(arms))
    return 0


if __name__ == "__main__":
    if len(sys.argv) >= 2 and sys.argv[1] == "--self-test":
        sys.exit(self_test())
    if len(sys.argv) < 2:
        print(__doc__); sys.exit(2)
    sys.exit(main_classify(sys.argv[1:]))
