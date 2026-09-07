#!/usr/bin/env python3
r"""SV-CORPUS-GRAD.3.45 — RAW-ENVELOPE CENSUS: for the eight rules `PARTIAL-TYPING-CENSUS:` names, what does the corpus
actually WRITE at each of their alternatives — the typed sibling, or the raw positional `Sequence` envelope?

`partial_typing_census.py` (TOOLBOX §1.5g) reads the GRAMMAR and names the alternatives that carry no return template;
this instrument reads the CORPUS through the release probe (every lane file's typed AST, `--parse-dump-ast`, TOOLBOX
§1.2 — the same reading `ast_field_census.py` uses) and classifies every node of the eight rules by the alternative that
produced it and whether the parser TYPED it or left the raw envelope. The eight rules and their alternatives are the
`.3.45` population: `assignment_pattern` (alternatives 2–4), `block_data_type` (2, 4, 5, 8), `case_statement` (2, 3),
`clocking_declaration` (2), `clocking_event_sv_2017` (2), `list_of_path_delay_expressions` (2–7),
`net_type_declaration_sv_2017` (2), `property_case_item` (2). A raw and a typed reading of one alternative are told
apart by SHAPE, never by the release under test, so the same classifier reads the HEAD census (the raw envelopes) and
the final-probe census (every row typed) and must agree row for row on the alternative.

Each node is read IN ITS PARENT'S CONTEXT where the shape alone is ambiguous: an `assignment_pattern` is the `pattern`
field of `assignment_pattern_expression`'s `{type, pattern}`; a `block_data_type` is the `body` of `{kind: "block_data",
body}`; a `list_of_path_delay_expressions` is the `delay.body` of a path declaration (`{kind, path, delay}` with `delay`
= `path_delay_value`'s `{kind: "bare" | "paren", body}`); a `property_case_item` is an item of `property_expr`'s
`{kind: "case", expr, items}`; a `clocking_event` is the `{kind: "sv_2017" | "sv_2023", body}` wrapper reached under an
`event` or `clocking` key, under a `{kind: "clocking", body}` wrapper (`coverage_event`, `event_control_sv_2023`), at
index 3 of the raw `cover sequence ( … )` envelope, at index 3 of the raw `global clocking` envelope, or inside an
optional group's list under those keys (`$rose(a, @(posedge clk))` — the pin pre-screen found this one) — the typed `{kind: "expression", body}` form is otherwise
indistinguishable from `event_control_sv_2017`, which emits the same shape under the same wrapper. `case_statement`,
`clocking_declaration` and the nettype declarations are read context-free: their typed key sets and raw signatures
occur nowhere else in the grammar (`grep -c` of each template in `grammars/systemverilog.ebnf` = 1).

⭐ THE NETTYPE ALIAS FORM IS SHADOWED, NOT RAW. IEEE 1800-2017 §6.6.7's second form `nettype wTsum nettypeid2;` (an alias
of an existing nettype) succeeds on BOTH alternatives of `net_type_declaration_sv_2017` — the first, `nettype data_type
name`, reads the nettype identifier as a `known_unscoped_data_type` — and the longest-match tie goes to declaration
order, so the alias alternative never wins (Protocol D on the LRM's own example: `🏁 … selected branch 1/2` on both
`sv_2017` and the fully typed `sv_2023` twin). The census therefore reads ZERO raw nettype rows on HEAD and instead
PREDICTS the alias rows: a data-type-form node whose data type is an unscoped or scoped type identifier with no
dimensions naming a nettype declared in the same file is bucketed `typed` with the note `alias_shadowed`, and those
files are movers for the release that lets the alias alternative win its tie (a negative lookahead on the data-type form). The prediction is order-insensitive within a
file (a use before its declaration cannot pass the store gate anyway) and is confirmed exactly by the interpreter
pre-screen before a byte is generated.

Usage:
  python3 docs/tasks/artifacts/sv_corpus_grad/partial_typing_residue/raw_envelope_census.py \
      --probe rust/target/release/parseability_probe --profile sv_2017 --files rust/target/sNNN/lane_sv.txt \
      --out rust/target/sNNN/raw_census_sv_2017.tsv --prediction rust/target/sNNN/movers_sv_2017.txt [--jobs 6] [--timeout 120]
  python3 .../raw_envelope_census.py --self-test [--probe rust/target/release/parseability_probe]

Headline: `RAW-ENVELOPE-CENSUS: profile=P files=N accepted=A rejected=R timeout=T nodump=D rows=M typed=X RAW=Y
passthrough=Z unclassified=U alias_shadowed=S` followed by one `  <rule>:alt<k>:<bucket> <count>` line per populated
bucket and the `PREDICTION` block: `raw_files=` (accepted files carrying a RAW row), `alias_shadowed_files=`,
`movers=` (their union — every accepted file whose typed AST the release must move; every other accepted file must be
byte-identical). Exit 0; exit 2 when the list or the probe is unusable; exit 7 on a self-test miss. TSV columns:
file  status  rule  alt  bucket  note.

⚠️ HONEST BOUND: `block_data_type`'s thirteen passthrough alternatives (a child node or keyword token passed through
unwrapped) are counted as one `other:passthrough` bucket, not by alternative; and `list_of_path_delay_expressions`
alternatives 6 and 7 can never appear — 6 spells the same six expressions as 4 and loses the tie to it, 7 requires the
literal source text `tx_path_delay_expression` — so a six-value list is always alternative 4 (Protocol D, `.3.45`).
"""
from __future__ import annotations
import argparse, concurrent.futures as cf, json, os, shutil, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "ast_field_census"))
from ast_field_census import dump_tree  # noqa: E402

INTEGER_ATOM_KINDS = {"byte", "shortint", "int", "longint", "integer", "time"}
PATH_DELAY_RAW_ALTS = {3: "2", 5: "3", 11: "4", 23: "5"}
PATH_DELAY_TYPED_KINDS = {"two": "2", "three": "3", "six": "4", "twelve": "5"}
RULES = ("assignment_pattern", "block_data_type", "case_statement", "clocking_declaration", "clocking_event_sv_2017",
         "list_of_path_delay_expressions", "net_type_declaration_sv_2017", "nettype_declaration_sv_2023", "property_case_item")


def is_kw(x, text):
    """A keyword token as the generated parser emits it: `[<trivia>, "text"]`."""
    return isinstance(x, list) and len(x) == 2 and x[1] == text


def is_tok(x, kind):
    """A punctuation token: `{"kind": "<kind>"}` and nothing else."""
    return isinstance(x, dict) and len(x) == 1 and x.get("kind") == kind


def kset(x):
    return frozenset(x.keys()) if isinstance(x, dict) else None


def ident(x):
    """The identifier node `{body: "name"}` → "name", else None."""
    if isinstance(x, dict) and kset(x) == {"body"} and isinstance(x["body"], str):
        return x["body"]
    return None


class FileCensus:
    """Every classified node of one file: rows of (rule, alt, bucket, note)."""

    def __init__(self, tree, profile: str):
        self.tree = tree
        self.profile = profile
        self.rows: list[tuple[str, str, str, str]] = []
        self.nettype_rule = "nettype_declaration_sv_2023" if profile == "sv_2023" else "net_type_declaration_sv_2017"
        self.declared_nettypes = self._declared_nettypes()

    def add(self, rule, alt, bucket, note=""):
        self.rows.append((rule, str(alt), bucket, note))

    # -- pre-pass: every nettype NAME this file declares (either form, either edition twin) -------------------
    def _declared_nettypes(self) -> set:
        names = set()
        stack = [self.tree]
        while stack:
            o = stack.pop()
            if isinstance(o, dict):
                k = kset(o)
                if k in ({"data_type", "name", "with_clause"}, {"kind", "data_type", "name", "with_clause"},
                         {"kind", "scope", "base", "name"}) and ident(o.get("name")):
                    names.add(ident(o["name"]))
                stack.extend(o.values())
            elif isinstance(o, list):
                if len(o) == 5 and is_kw(o[0], "nettype") and is_tok(o[4], "semi") and ident(o[3]):
                    names.add(ident(o[3]))
                stack.extend(o)
        return names

    # -- per-rule classifiers ------------------------------------------------------------------------------------
    def assignment_pattern(self, v):
        r = "assignment_pattern"
        if isinstance(v, dict):
            k = kset(v)
            if k == {"exprs"}:
                return self.add(r, "1" if v["exprs"] != [] else "5", "typed")
            if v.get("kind") == "structure" and k == {"kind", "keys", "values"}:
                return self.add(r, 2, "typed")
            if v.get("kind") == "array" and k == {"kind", "keys", "values"}:
                return self.add(r, 3, "typed")
            if v.get("kind") == "replication" and k == {"kind", "count", "exprs"}:
                return self.add(r, 4, "typed")
        elif isinstance(v, list):
            if (len(v) == 7 and is_tok(v[0], "tick") and is_tok(v[1], "lbrace") and is_tok(v[3], "colon")
                    and is_tok(v[6], "rbrace") and isinstance(v[2], dict)):
                kk = v[2].get("kind")
                if kk in ("member", "pattern"):
                    return self.add(r, 2, "RAW")
                if kk in ("expression", "pattern_key"):
                    return self.add(r, 3, "RAW")
            if (len(v) == 8 and is_tok(v[0], "tick") and is_tok(v[1], "lbrace") and is_tok(v[3], "lbrace")
                    and is_tok(v[6], "rbrace") and is_tok(v[7], "rbrace")):
                return self.add(r, 4, "RAW")
        self.add(r, "?", "unclassified", json.dumps(v, sort_keys=True)[:80])

    def block_data_type(self, v):
        r = "block_data_type"
        if isinstance(v, dict):
            k = kset(v)
            if k == {"base", "signing", "dims"}:
                return self.add(r, 1, "typed")
            kind = v.get("kind")
            if kind == "integer_atom" and k == {"kind", "base", "signing"}:
                return self.add(r, 2, "typed")
            if kind == "struct_union" and k == {"kind", "header", "packed_signing", "members", "dims"}:
                return self.add(r, 4, "typed")
            if kind == "enum" and k == {"kind", "base_type", "names", "dims"}:
                return self.add(r, 5, "typed")
            if kind == "virtual_interface" and k == {"kind", "interface_keyword", "name", "params", "modport"}:
                return self.add(r, 8, "typed")
            return self.add(r, "other", "passthrough")
        if isinstance(v, str):
            return self.add(r, "other", "passthrough")
        if isinstance(v, list):
            if len(v) == 2 and isinstance(v[0], dict) and v[0].get("kind") in INTEGER_ATOM_KINDS and len(v[0]) == 1:
                return self.add(r, 2, "RAW")
            if len(v) == 2 and isinstance(v[0], list) and isinstance(v[1], str):
                return self.add(r, "other", "passthrough")      # a keyword token: `event`, `string`, `chandle`
            if len(v) == 7 and is_tok(v[2], "lbrace") and is_tok(v[5], "rbrace"):
                if is_kw(v[0], "enum"):
                    return self.add(r, 5, "RAW")
                if isinstance(v[0], dict) and v[0].get("kind") in ("sv_2017", "sv_2023"):
                    return self.add(r, 4, "RAW")
            if len(v) == 5 and is_kw(v[0], "virtual"):
                return self.add(r, 8, "RAW")
        self.add(r, "?", "unclassified", json.dumps(v, sort_keys=True)[:80])

    def case_statement_dict(self, v) -> bool:
        k = kset(v)
        if k == {"unique_priority", "keyword", "expr", "items"}:
            self.add("case_statement", 1, "typed"); return True
        if v.get("kind") in ("matches", "inside") and k == {"kind", "unique_priority", "keyword", "expr", "items"}:
            self.add("case_statement", "2" if v["kind"] == "matches" else "3", "typed"); return True
        return False

    def case_statement_list(self, v) -> bool:
        if len(v) == 9 and is_tok(v[2], "lparen") and is_tok(v[4], "rparen") and is_kw(v[8], "endcase"):
            if is_kw(v[5], "matches"):
                self.add("case_statement", 2, "RAW"); return True
            if is_kw(v[5], "inside"):
                self.add("case_statement", 3, "RAW"); return True
        return False

    def clocking_declaration_dict(self, v) -> bool:
        k = kset(v)
        if k == {"default_keyword", "name", "event", "items", "end_label"}:
            self.add("clocking_declaration", 1, "typed"); return True
        if v.get("kind") == "global" and k == {"kind", "name", "event", "end_label"}:
            self.add("clocking_declaration", 2, "typed"); return True
        return False

    def clocking_declaration_list(self, v) -> bool:
        if len(v) == 7 and is_kw(v[0], "global") and is_kw(v[1], "clocking") and is_kw(v[5], "endclocking"):
            self.add("clocking_declaration", 2, "RAW"); return True
        return False

    def clocking_event(self, wrapper):
        """`wrapper` is the `{kind: "sv_2017" | "sv_2023", body}` node of `clocking_event`, reached in a clocking context."""
        b = wrapper.get("body")
        if wrapper.get("kind") == "sv_2023":
            if isinstance(b, dict) and b.get("kind") in ("ps", "hierarchical", "expression") and kset(b) == {"kind", "body"}:
                return self.add("clocking_event_sv_2023", {"ps": 1, "hierarchical": 2, "expression": 3}[b["kind"]], "typed")
            return self.add("clocking_event_sv_2023", "?", "unclassified", json.dumps(b, sort_keys=True)[:80])
        r = "clocking_event_sv_2017"
        if isinstance(b, dict):
            if kset(b) == {"body"} and ident(b["body"]):
                return self.add(r, 1, "typed")
            if b.get("kind") == "expression" and kset(b) == {"kind", "body"}:
                return self.add(r, 2, "typed")
        elif isinstance(b, list) and len(b) == 4 and is_tok(b[0], "at_sign") and is_tok(b[1], "lparen") and is_tok(b[3], "rparen"):
            return self.add(r, 2, "RAW")
        self.add(r, "?", "unclassified", json.dumps(b, sort_keys=True)[:80])

    def path_delay(self, v):
        r = "list_of_path_delay_expressions"
        if isinstance(v, dict):
            if kset(v) == {"body"}:
                return self.add(r, 1, "typed")
            if v.get("kind") in PATH_DELAY_TYPED_KINDS:
                return self.add(r, PATH_DELAY_TYPED_KINDS[v["kind"]], "typed")
        elif isinstance(v, list) and len(v) in PATH_DELAY_RAW_ALTS:
            if all(is_tok(x, "comma") for x in v[1::2]) and all(isinstance(x, dict) and kset(x) == {"body"} for x in v[0::2]):
                return self.add(r, PATH_DELAY_RAW_ALTS[len(v)], "RAW")
        self.add(r, "?", "unclassified", json.dumps(v, sort_keys=True)[:80])

    def nettype_dict(self, v) -> bool:
        k = kset(v)
        r = self.nettype_rule
        if k == {"data_type", "name", "with_clause"} or (v.get("kind") == "data_type" and k == {"kind", "data_type", "name", "with_clause"}):
            self.add(r, 1, "typed", self._alias_note(v)); return True
        if v.get("kind") == "nettype" and k == {"kind", "scope", "base", "name"}:
            self.add(r, 2, "typed"); return True
        return False

    def nettype_list(self, v) -> bool:
        if len(v) == 5 and is_kw(v[0], "nettype") and is_tok(v[4], "semi"):
            self.add(self.nettype_rule, 2, "RAW"); return True
        return False

    def _alias_note(self, v) -> str:
        """A data-type-form nettype whose data type is a dimension-less type identifier naming a nettype declared in
        this file is the LRM's alias form parsed down the wrong alternative (shadowed by the tie): note it."""
        dt = v.get("data_type")
        if not (isinstance(dt, dict) and dt.get("kind") in ("known_unscoped_data_type", "scoped_data_type")):
            return ""
        body = dt.get("body")
        if not isinstance(body, dict) or body.get("dims") != []:
            return ""
        name = ident(body.get("type"))
        if name and name in self.declared_nettypes and v.get("with_clause") == []:
            return "alias_shadowed:%s" % name
        return ""

    def property_case_item(self, item):
        r = "property_case_item"
        if isinstance(item, dict):
            if kset(item) == {"expressions", "body"}:
                return self.add(r, 1, "typed")
            if item.get("kind") == "default" and kset(item) == {"kind", "body"}:
                return self.add(r, 2, "typed")
        elif isinstance(item, list) and len(item) == 4 and is_kw(item[0], "default") and is_tok(item[3], "semi"):
            return self.add(r, 2, "RAW")
        self.add(r, "?", "unclassified", json.dumps(item, sort_keys=True)[:80])

    # -- the walk ----------------------------------------------------------------------------------------------------
    def walk(self):
        # (node, parent, key, ctx_key): ctx_key is the nearest enclosing DICT key, carried through lists — a
        # clocking event can sit inside an optional group's list under the `clocking` key (`$rose(a, @(posedge clk))`
        # is `( comma ( clocking_event )? )?` → `[comma, <wrapper>]`), and the key is what names the context.
        stack = [(self.tree, None, None, None)]
        while stack:
            node, parent, key, ctx_key = stack.pop()
            if isinstance(node, dict):
                k = kset(node)
                if k == {"type", "pattern"}:
                    self.assignment_pattern(node["pattern"])
                elif k == {"kind", "body"} and node.get("kind") == "block_data":
                    self.block_data_type(node["body"])
                elif k == {"kind", "body"} and node.get("kind") in ("sv_2017", "sv_2023") and self._clocking_context(parent, key, ctx_key):
                    self.clocking_event(node)
                elif k == {"kind", "path", "delay"} and isinstance(node["delay"], dict) and kset(node["delay"]) == {"kind", "body"} \
                        and node["delay"].get("kind") in ("bare", "paren"):
                    self.path_delay(node["delay"]["body"])
                elif k == {"kind", "expr", "items"} and node.get("kind") == "case" and isinstance(node["items"], list):
                    for item in node["items"]:
                        self.property_case_item(item)
                else:
                    self.case_statement_dict(node) or self.clocking_declaration_dict(node) or self.nettype_dict(node)
                for kk, vv in node.items():
                    stack.append((vv, node, kk, kk))
            elif isinstance(node, list):
                self.case_statement_list(node) or self.clocking_declaration_list(node) or self.nettype_list(node)
                for i, vv in enumerate(node):
                    stack.append((vv, node, i, ctx_key))
        return self.rows

    @staticmethod
    def _clocking_context(parent, key, ctx_key=None) -> bool:
        if isinstance(parent, dict):
            if key in ("event", "clocking"):
                return True
            if key == "body" and kset(parent) == {"kind", "body"} and parent.get("kind") == "clocking":
                return True
        if isinstance(parent, list) and ctx_key in ("event", "clocking"):
            return True     # the wrapper inside an optional group's list under the key (`$rose(a, @(posedge clk))`)
        if isinstance(parent, list) and key == 3 and len(parent) >= 4 and is_kw(parent[0], "cover") \
                and is_kw(parent[1], "sequence") and is_tok(parent[2], "lparen"):
            return True
        if isinstance(parent, list) and key == 3 and len(parent) == 7 and is_kw(parent[0], "global") \
                and is_kw(parent[1], "clocking"):
            return True     # the raw `global clocking` envelope (clocking_declaration alternative 2 before 1.0.219)
        return False


def census_tree(tree, profile: str):
    return FileCensus(tree, profile).walk()


def one_file(probe: str, profile: str, rel: str, timeout: int):
    status, tree = dump_tree(probe, profile, rel, timeout)
    rows = census_tree(tree, profile) if status == "accepted" else []
    return status, rows


def run(args) -> int:
    files = [l.strip() for l in Path(args.files).read_text().splitlines() if l.strip()]
    if not files:
        print("⛔ REFUSING: the file list is empty (%s)" % args.files); return 2
    probe = str(Path(args.probe).resolve())
    if not os.access(probe, os.X_OK):
        print("⛔ REFUSING: probe is not executable: %s" % probe); return 2
    counts = {"accepted": 0, "rejected": 0, "timeout": 0, "nodump": 0}
    results = {}
    with cf.ThreadPoolExecutor(max_workers=args.jobs) as ex:
        futs = {ex.submit(one_file, probe, args.profile, f, args.timeout): f for f in files}
        for fut in cf.as_completed(futs):
            results[futs[fut]] = fut.result()
    out = Path(args.out); out.parent.mkdir(parents=True, exist_ok=True)
    buckets: dict[tuple, int] = {}
    raw_files, alias_files = [], []
    n_rows = n_typed = n_raw = n_pt = n_uncl = n_alias = 0
    with open(out, "w") as fh:
        fh.write("file\tstatus\trule\talt\tbucket\tnote\n")
        for f in files:
            status, rows = results[f]
            counts[status] += 1
            if not rows:
                fh.write("%s\t%s\t-\t-\t-\t-\n" % (f, status))
            has_raw = has_alias = False
            for rule, alt, bucket, note in rows:
                fh.write("%s\t%s\t%s\t%s\t%s\t%s\n" % (f, status, rule, alt, bucket, note))
                buckets[(rule, alt, bucket)] = buckets.get((rule, alt, bucket), 0) + 1
                n_rows += 1
                if bucket == "typed": n_typed += 1
                elif bucket == "RAW": n_raw += 1; has_raw = True
                elif bucket == "passthrough": n_pt += 1
                else: n_uncl += 1
                if note.startswith("alias_shadowed"): n_alias += 1; has_alias = True
            if has_raw: raw_files.append(f)
            if has_alias: alias_files.append(f)
    print("RAW-ENVELOPE-CENSUS: profile=%s files=%d accepted=%d rejected=%d timeout=%d nodump=%d rows=%d typed=%d RAW=%d passthrough=%d unclassified=%d alias_shadowed=%d"
          % (args.profile, len(files), counts["accepted"], counts["rejected"], counts["timeout"], counts["nodump"],
             n_rows, n_typed, n_raw, n_pt, n_uncl, n_alias))
    for (rule, alt, bucket), n in sorted(buckets.items()):
        print("  %s:alt%s:%s %d" % (rule, alt, bucket, n))
    movers = sorted(set(raw_files) | set(alias_files))
    print("PREDICTION raw_files=%d alias_shadowed_files=%d movers=%d (accepted files whose typed AST the release must move; every other accepted file must be byte-identical)"
          % (len(raw_files), len(alias_files), len(movers)))
    if args.prediction:
        Path(args.prediction).write_text("".join(m + "\n" for m in movers))
        print("movers -> %s" % args.prediction)
    print("rows -> %s" % out)
    return 0


# ------------------------------------------------------------------------------------------------------------------
# self-test
# ------------------------------------------------------------------------------------------------------------------
def _tok(k): return {"kind": k}
def _kw(t): return [[], t]
def _id(n): return {"body": n}
EXPR = {"kind": "base", "first": {"kind": "primary"}, "rest": []}


def _synthetic_cases():
    """(name, tree, profile, expected rows) — one raw envelope per alternative in the observed shape, the typed
    siblings, the post-release typed shapes, and the nettype alias shadow."""
    ap_raw2 = {"type": [], "pattern": [_tok("tick"), _tok("lbrace"), {"kind": "member", "body": _id("a")}, _tok("colon"), EXPR,
                                       [[_tok("comma"), {"kind": "member", "body": _id("b")}, _tok("colon"), EXPR]], _tok("rbrace")]}
    ap_raw3 = {"type": [], "pattern": [_tok("tick"), _tok("lbrace"), {"kind": "expression", "body": EXPR}, _tok("colon"), EXPR, [], _tok("rbrace")]}
    ap_raw4 = {"type": [], "pattern": [_tok("tick"), _tok("lbrace"), EXPR, _tok("lbrace"), EXPR, [[_tok("comma"), EXPR]], _tok("rbrace"), _tok("rbrace")]}
    ap_t1 = {"type": [], "pattern": {"exprs": [EXPR]}}
    ap_t5 = {"type": [], "pattern": {"exprs": []}}
    ap_t2 = {"type": [], "pattern": {"kind": "structure", "keys": [], "values": []}}
    ap_t4 = {"type": [], "pattern": {"kind": "replication", "count": EXPR, "exprs": [EXPR]}}
    bd = lambda body: {"kind": "block_data", "body": body}
    bd_raw2 = bd([_tok("int"), _tok("unsigned")])
    bd_raw2b = bd([_tok("integer"), []])
    bd_raw4 = bd([{"kind": "sv_2017", "body": _tok("struct")}, [[_kw("packed"), []]], _tok("lbrace"), {"attributes": []}, [], _tok("rbrace"), []])
    bd_raw5 = bd([_kw("enum"), [], _tok("lbrace"), {"name": _id("A")}, [], _tok("rbrace"), []])
    bd_raw8 = bd([_kw("virtual"), _kw("interface"), _id("I"), [], []])
    bd_t1 = bd({"base": _tok("logic"), "signing": [], "dims": []})
    bd_t2 = bd({"kind": "integer_atom", "base": _tok("int"), "signing": []})
    bd_pt = bd(_kw("event"))
    bd_pt2 = bd({"kind": "string"})
    cs_raw2 = [[], _tok("case"), _tok("lparen"), EXPR, _tok("rparen"), _kw("matches"), {"kind": "pattern"}, [], _kw("endcase")]
    cs_raw3 = [_tok("priority"), _kw("case"), _tok("lparen"), EXPR, _tok("rparen"), _kw("inside"), {"kind": "sv_2017"}, [], _kw("endcase")]
    cs_t1 = {"unique_priority": [], "keyword": _tok("case"), "expr": EXPR, "items": {"first": {}, "rest": []}}
    cs_t3 = {"kind": "inside", "unique_priority": [], "keyword": _tok("case"), "expr": EXPR, "items": {"first": {}, "rest": []}}
    ev_raw = {"kind": "sv_2017", "body": [_tok("at_sign"), _tok("lparen"), [{"edge": _tok("posedge")}], _tok("rparen")]}
    ev_t1 = {"kind": "sv_2017", "body": {"body": _id("clk")}}
    ev_t2 = {"kind": "sv_2017", "body": {"kind": "expression", "body": [{"edge": _tok("posedge")}]}}
    cd_raw2 = [_kw("global"), _kw("clocking"), _id("gcb"), ev_raw, _tok("semi"), _kw("endclocking"), []]
    cd_t1 = {"default_keyword": [], "name": _id("cb"), "event": ev_t1, "items": [], "end_label": []}
    cd_t2 = {"kind": "global", "name": _id("cb"), "event": ev_t2, "end_label": []}
    cov = {"kind": "clocking", "body": ev_raw}
    cover_raw = [_kw("cover"), _kw("sequence"), _tok("lparen"), ev_t1, [], {"kind": "seq"}, _tok("rparen"), {}]
    # an event_control `@(posedge clk)` OUTSIDE a clocking context must NOT be counted (the ambiguity the context resolves)
    evctl = {"kind": "statement", "timing": {"kind": "sv_2017", "body": {"kind": "expression", "body": [{"edge": _tok("posedge")}]}}}
    # `$rose(a, @(posedge clk))`: the wrapper inside the optional group's list under the `clocking` key (the pre-screen found it)
    systf = {"kind": "expr_clocking", "name": _id("$rose"), "first_expr": EXPR, "rest_exprs": [], "clocking": [_tok("comma"), ev_raw]}
    pd = lambda body: {"kind": "parallel", "path": {}, "delay": {"kind": "paren", "body": body}}
    B = {"body": EXPR}
    pd_t1 = pd(B)
    pd_raw2 = pd([B, _tok("comma"), B])
    pd_raw3 = pd([B, _tok("comma"), B, _tok("comma"), B])
    pd_raw4 = pd([B, _tok("comma")] * 5 + [B])
    pd_raw5 = pd([B, _tok("comma")] * 11 + [B])
    pd_t3 = pd({"kind": "three", "trise": B, "tfall": B, "tz": B})
    nt_t1 = {"data_type": {"kind": "non_integer", "base": _tok("real")}, "name": _id("T0"), "with_clause": []}
    nt_alias = {"data_type": {"kind": "known_unscoped_data_type", "body": {"dims": [], "type": _id("T0")}}, "name": _id("T2"), "with_clause": []}
    nt_scoped_alias = {"data_type": {"kind": "scoped_data_type", "body": {"dims": [], "scope": {}, "type": _id("T0")}}, "name": _id("T3"), "with_clause": []}
    nt_not_alias = {"data_type": {"kind": "known_unscoped_data_type", "body": {"dims": [], "type": _id("st")}}, "name": _id("T4"), "with_clause": []}
    nt_raw2 = [_kw("nettype"), [], _id("T0"), _id("T5"), _tok("semi")]
    nt_t2 = {"kind": "nettype", "scope": [], "base": _id("T0"), "name": _id("T6")}
    nt23_t1 = {"kind": "data_type", "data_type": {"kind": "known_unscoped_data_type", "body": {"dims": [], "type": _id("T0")}}, "name": _id("T7"), "with_clause": []}
    pc = lambda items: {"kind": "case", "expr": EXPR, "items": items}
    pc_mixed = pc([{"expressions": [EXPR], "body": {}}, [_kw("default"), _tok("colon"), {"kind": "sv_2017"}, _tok("semi")], {"kind": "default", "body": {}}])
    plain = {"content": {"Json": {"source_text": [{"kind": "description", "body": {"items": [EXPR, _kw("endmodule")]}}]}}}
    return [
        ("assignment patterns", [ap_raw2, ap_raw3, ap_raw4, ap_t1, ap_t5, ap_t2, ap_t4], "sv_2017",
         [("assignment_pattern", "2", "RAW"), ("assignment_pattern", "3", "RAW"), ("assignment_pattern", "4", "RAW"),
          ("assignment_pattern", "1", "typed"), ("assignment_pattern", "5", "typed"), ("assignment_pattern", "2", "typed"), ("assignment_pattern", "4", "typed")]),
        ("block data types", [bd_raw2, bd_raw2b, bd_raw4, bd_raw5, bd_raw8, bd_t1, bd_t2, bd_pt, bd_pt2], "sv_2017",
         [("block_data_type", "2", "RAW"), ("block_data_type", "2", "RAW"), ("block_data_type", "4", "RAW"), ("block_data_type", "5", "RAW"),
          ("block_data_type", "8", "RAW"), ("block_data_type", "1", "typed"), ("block_data_type", "2", "typed"),
          ("block_data_type", "other", "passthrough"), ("block_data_type", "other", "passthrough")]),
        ("case statements", [cs_raw2, cs_raw3, cs_t1, cs_t3], "sv_2017",
         [("case_statement", "2", "RAW"), ("case_statement", "3", "RAW"), ("case_statement", "1", "typed"), ("case_statement", "3", "typed")]),
        ("clocking declarations and events", [cd_raw2, cd_t1, cd_t2, cov, cover_raw, evctl, systf], "sv_2017",
         [("clocking_declaration", "2", "RAW"), ("clocking_event_sv_2017", "2", "RAW"),
          ("clocking_declaration", "1", "typed"), ("clocking_event_sv_2017", "1", "typed"),
          ("clocking_declaration", "2", "typed"), ("clocking_event_sv_2017", "2", "typed"),
          ("clocking_event_sv_2017", "2", "RAW"), ("clocking_event_sv_2017", "1", "typed"),
          ("clocking_event_sv_2017", "2", "RAW")]),
        ("path delays", [pd_t1, pd_raw2, pd_raw3, pd_raw4, pd_raw5, pd_t3], "sv_2017",
         [("list_of_path_delay_expressions", "1", "typed"), ("list_of_path_delay_expressions", "2", "RAW"), ("list_of_path_delay_expressions", "3", "RAW"),
          ("list_of_path_delay_expressions", "4", "RAW"), ("list_of_path_delay_expressions", "5", "RAW"), ("list_of_path_delay_expressions", "3", "typed")]),
        ("nettypes sv_2017", [nt_t1, nt_alias, nt_scoped_alias, nt_not_alias, nt_raw2, nt_t2], "sv_2017",
         [("net_type_declaration_sv_2017", "1", "typed", ""), ("net_type_declaration_sv_2017", "1", "typed", "alias_shadowed:T0"),
          ("net_type_declaration_sv_2017", "1", "typed", "alias_shadowed:T0"), ("net_type_declaration_sv_2017", "1", "typed", ""),
          ("net_type_declaration_sv_2017", "2", "RAW", ""), ("net_type_declaration_sv_2017", "2", "typed", "")]),
        ("nettypes sv_2023", [nt_t1, nt23_t1, nt_t2], "sv_2023",
         [("nettype_declaration_sv_2023", "1", "typed", ""), ("nettype_declaration_sv_2023", "1", "typed", "alias_shadowed:T0"),
          ("nettype_declaration_sv_2023", "2", "typed", "")]),
        ("property case items", [pc_mixed], "sv_2017",
         [("property_case_item", "1", "typed"), ("property_case_item", "2", "RAW"), ("property_case_item", "2", "typed")]),
        ("RED control: a tree with none of the eight rules", plain, "sv_2017", []),
    ]


def self_test(args) -> int:
    problems = []
    for name, tree, profile, want in _synthetic_cases():
        rows = census_tree(tree, profile)
        got = sorted((r[0], r[1], r[2]) + ((r[3],) if len(want) and len(want[0]) == 4 else ()) for r in rows)
        want_s = sorted(tuple(w) for w in want)
        if got != want_s:
            problems.append("%s: got %r want %r" % (name, got, want_s))
    if problems:
        print("RAW-ENVELOPE-CENSUS SELF-TEST: MISS — " + " | ".join(problems)); return 7
    msg = "%d synthetic arms (every raw alternative in its observed shape, the typed siblings, the post-release shapes, the nettype alias shadow on both editions, an event control outside a clocking context NOT counted, the system-call clocking slot, a RED control with no rows)" % len(_synthetic_cases())
    if args.probe:
        # the probe arm: eighteen reproducers, one per raw alternative (plus the LRM's own nettype alias example), on the
        # release probe — every raw envelope the grammar still emits must be found, by alternative
        probe = str(Path(args.probe).resolve())
        if not os.access(probe, os.X_OK):
            print("⛔ REFUSING: probe is not executable: %s" % probe); return 2
        d = Path(tempfile.mkdtemp(prefix="rec_selftest_", dir=str(ROOT / "rust" / "target")))
        try:
            srcs = _reproducers()
            for fname, text in srcs.items():
                (d / fname).write_text(text)
            lst = d / "list.txt"
            lst.write_text("".join("%s\n" % (d / f).relative_to(ROOT) for f in srcs))
            ns = argparse.Namespace(probe=probe, profile="sv_2017", files=str(lst), out=str(d / "out.tsv"), prediction=str(d / "movers.txt"),
                                    jobs=4, timeout=120)
            rc = run(ns)
            if rc:
                return rc
            rows = [l.rstrip("\n").split("\t") for l in (d / "out.tsv").read_text().splitlines()[1:]]
            raw = {}
            alias = 0
            rejected = [r[0] for r in rows if r[1] != "accepted"]
            for r in rows:
                if r[4] == "RAW":
                    raw[(r[2], r[3])] = raw.get((r[2], r[3]), 0) + 1
                if r[5].startswith("alias_shadowed"):
                    alias += 1
            want_raw = {("assignment_pattern", "2"): 1, ("assignment_pattern", "3"): 1, ("assignment_pattern", "4"): 1,
                        ("block_data_type", "2"): 1, ("block_data_type", "4"): 1, ("block_data_type", "5"): 1, ("block_data_type", "8"): 1,
                        ("case_statement", "2"): 1, ("case_statement", "3"): 1, ("clocking_declaration", "2"): 1,
                        ("clocking_event_sv_2017", "2"): 3, ("list_of_path_delay_expressions", "2"): 1, ("list_of_path_delay_expressions", "3"): 1,
                        ("list_of_path_delay_expressions", "4"): 1, ("list_of_path_delay_expressions", "5"): 1, ("property_case_item", "2"): 1}
            typed_only = sum(1 for r in rows if r[4] == "typed")
            alias_typed = sum(1 for r in rows if r[2] == "net_type_declaration_sv_2017" and r[3] == "2" and r[4] == "typed")
            if rejected:
                problems.append("probe arm: reproducers rejected: %r" % rejected)
            # Two readings are admitted — the arm holds the CLASSIFIER, not the release: on a probe up to 1.0.218 every raw
            # alternative is found by alternative and the three alias declarations are `alias_shadowed`; on 1.0.219 and
            # later every row is typed, no row is raw, and the three alias declarations are the typed alias alternative.
            pre = (raw == want_raw and alias == 3)
            post = (raw == {} and alias == 0 and alias_typed == 3 and typed_only > 0)
            if not (pre or post):
                problems.append("probe arm: neither the pre-release reading (18 RAW rows over 16 alternatives + 3 alias_shadowed) nor the post-release reading (0 RAW, 0 alias_shadowed, 3 typed alias rows): RAW=%r alias_shadowed=%d alias_typed=%d typed=%d"
                                % (sorted(raw.items()), alias, alias_typed, typed_only))
            uncl = [r for r in rows if r[4] == "unclassified"]
            if uncl:
                problems.append("probe arm: unclassified rows %r" % uncl[:3])
            if problems:
                print("RAW-ENVELOPE-CENSUS SELF-TEST: MISS — " + " | ".join(problems)); return 7
            msg += "; probe arm: %d reproducers, %s" % (
                len(srcs), "RAW rows by alternative as expected (18 rows, 16 alternatives) and 3 alias-shadowed nettype rows (a pre-1.0.219 probe)" if pre
                else "every alternative typed, 0 RAW, the 3 alias declarations on the typed alias alternative (a 1.0.219+ probe)")
        finally:
            shutil.rmtree(d, ignore_errors=True)
    print("RAW-ENVELOPE-CENSUS SELF-TEST: OK — " + msg)
    return 0


def _reproducers() -> dict:
    """One file per raw alternative (the `.3.45` reproducers), plus IEEE 1800-2017 §6.6.7's own nettype alias example."""
    return {
        "ap2.sv": "module m;\n  typedef struct { int a; int b; } st;\n  st s;\n  initial s = '{a: 1, b: 2};\nendmodule\n",
        "ap3.sv": "module m;\n  int arr [0:1];\n  initial arr = '{0: 1, default: 0};\nendmodule\n",
        "ap4.sv": "module m;\n  int arr [0:3];\n  initial arr = '{2{0, 1}};\nendmodule\n",
        "bdt2.sv": "module m;\n  initial begin\n    int unsigned x;\n    x = 1;\n  end\nendmodule\n",
        "bdt4.sv": "module m;\n  initial begin\n    struct packed { int a; } s;\n    s.a = 1;\n  end\nendmodule\n",
        "bdt5.sv": "module m;\n  initial begin\n    enum { A, B } e;\n    e = A;\n  end\nendmodule\n",
        "bdt8.sv": "interface I; endinterface\nmodule m;\n  initial begin\n    virtual interface I vi;\n    vi = null;\n  end\nendmodule\n",
        "cs2.sv": "module m;\n  typedef union tagged { void Invalid; int Valid; } u_t;\n  u_t u;\n  initial case (u) matches\n    tagged Valid .n: ;\n    default: ;\n  endcase\nendmodule\n",
        "cs3.sv": "module m;\n  int a;\n  initial priority case (a) inside\n    [0:1]: ;\n    default: ;\n  endcase\nendmodule\n",
        "cd1.sv": "module m(input clk);\n  default clocking cb @(posedge clk); endclocking : cb\nendmodule\n",
        "cd2.sv": "module m(input clk);\n  global clocking gcb @(posedge clk); endclocking : gcb\nendmodule\n",
        "ce2.sv": "module m(input clk);\n  clocking cb @(posedge clk); endclocking\nendmodule\n",
        "lpd2.sv": "module m(input a, output b);\n  specify\n    (a => b) = (1, 2);\n  endspecify\nendmodule\n",
        "lpd3.sv": "module m(input a, output b);\n  specify\n    (a => b) = (1, 2, 3);\n  endspecify\nendmodule\n",
        "lpd4.sv": "module m(input a, output b);\n  specify\n    (a => b) = (1, 2, 3, 4, 5, 6);\n  endspecify\nendmodule\n",
        "lpd5.sv": "module m(input a, output b);\n  specify\n    (a => b) = (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);\n  endspecify\nendmodule\n",
        "ntd2.sv": "package p; nettype real T1; endpackage\nmodule m;\n  nettype real T0;\n  nettype T0 T2;\n  nettype p::T1 T3;\nendmodule\n",
        "ntd_lrm.sv": "package p;\n  typedef struct { real field1; bit field2; } T;\n  function automatic T Tsum(input T driver[]);\n    Tsum.field1 = 0.0;\n  endfunction\n  nettype T wTsum with Tsum;\n  nettype wTsum nettypeid2;\nendpackage\n",
        "pci2.sv": "module m(input clk, a, b);\n  property p;\n    case (a) 1: b; default: 1; endcase\n  endproperty\nendmodule\n",
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--probe"); ap.add_argument("--profile", default="sv_2017")
    ap.add_argument("--files"); ap.add_argument("--out"); ap.add_argument("--prediction")
    ap.add_argument("--jobs", type=int, default=4); ap.add_argument("--timeout", type=int, default=120)
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return self_test(args)
    for k in ("probe", "files", "out"):
        if getattr(args, k) is None:
            ap.error("--%s is required" % k)
    return run(args)


if __name__ == "__main__":
    sys.exit(main())
