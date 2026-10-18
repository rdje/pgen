# Worked Example: Clause-22 Directives, a Macro-Usage Line, a Continued Define

Release `1.0.8` (AST-dump schema `6`) made the parser derive every IEEE 1800 clause-22 directive, a line that
opens with a text macro usage, a backslash-continued `` `define `` body and an `` `include `` whose path is a macro.
This chapter walks one small file that uses all of them: its real dump from the release `parseability_probe`,
item by item, then the inputs the parser deliberately refuses and why.

## Input

```systemverilog
`resetall
`line 12 "gen/top.sv" 1
`pragma protect begin
`define CHECK(v) \
  if (!(v)) $display("FAILED: `%s`", `"v`")
`uvm_object_utils(my_item)
`include `pkg_file(core)
```

Release `1.0.7` refused this file at byte 0: `` `resetall `` had no production, and neither did four of the other six
lines.

## Reproducing the dump

```bash
rust/target/release/parseability_probe --parse-dump-ast-pretty systemverilog_preprocessor clause22.sv clause22.json
# -> parse_full passed for grammar 'systemverilog_preprocessor' on 'clause22.sv' (AST dump: clause22.json)
```

The root is `{type: "systemverilog_preprocessor_file", items: […]}`, one `pp_item` per line form. The seven items
below are copied from that dump; only the define's fragment list is shortened, where marked.

## Item by item

**1 — `` `resetall `` (§22.3)**

```json
{"kind": "resetall", "body": {"comment": [[], []]}}
```

`comment` is the optional trailing comment (`directive_comment_tail`), empty here. The body is kept at the dispatch,
unlike `celldefine`'s.

**2 — `` `line 12 "gen/top.sv" 1 `` (§22.12, Syntax 22-9)**

```json
{"kind": "line", "body": {"number": [[" "], "12"], "filename": [[" "], "\"gen/top.sv\""], "level": [[" "], "1"]}}
```

Each parameter is the usual `[[leading white space], "text"]` envelope — the token text is the last element. The
filename keeps its quotes. Level `1` means *the next line is the first after entering an include file*.

**3 — `` `pragma protect begin `` (§22.11, Syntax 22-8)**

```json
{"kind": "pragma", "body": {"name": [[" "], "protect"],
                            "expressions": [{"kind": "keyword", "keyword": [[" "], "begin"]}],
                            "comment": [[], []]}}
```

`expressions` is the comma-separated `pragma_expression` list. A bare word is a `keyword`; `key = value` is a
`keyword_value` whose `value` is a `number`, a `string`, an `identifier` or a nested `list`.

**4 — the continued define (§22.5.1)**

```json
{"kind": "define", "body": {
  "name": [[" "], "CHECK"],
  "formals": [{"name": [[], "v"], "default": []}],
  "body": {"fragments": [
    {"kind": "line_continuation"},
    {"kind": "text", "body": [["  "], "if "]},
    {"kind": "lparen"}, {"kind": "text", "body": [[], "!"]}, {"kind": "lparen"},
    {"kind": "text", "body": [[], "v"]}, {"kind": "rparen"}, {"kind": "rparen"},
    {"kind": "text", "body": [[" "], "$display"]},
    {"kind": "lparen"},
    {"kind": "text", "body": [[], "\"FAILED: `%s`\""]},
    {"kind": "comma"},
    {"kind": "stringize"}, {"kind": "text", "body": [[], "v"]}, {"kind": "stringize"},
    {"kind": "rparen"}]}}}
```

Three things to read here:

- `line_continuation` stands where the `\` + newline was: the second source line is part of the SAME define. Through
  `1.0.7` the body stopped at `\` and the second line was a separate `non_directive_line`.
- `"FAILED: `%s`"` is ONE `text` fragment. A plain string literal is opaque in macro text (§22.5.1: *"Macro
  substitution and argument substitution shall not occur within string literals"*), so the backticks and the colon
  inside it are characters, not tokens. Through `1.0.7` the backtick ended the body and the file was refused.
- `` `"v`" `` is a stringize region: `stringize`, the formal `v` as text, `stringize`.

**5 — a line that opens with a macro usage (§22.5.1 Syntax 22-3)**

```json
{"kind": "non_directive_line", "body": {"text": [[], "`uvm_object_utils(my_item)"]}}
```

The usage is source text to this parser. Recording directive structure is its job; expanding the usage is the
[expansion stage's](status-and-distance.md#the-expansion-stage-is-a-sibling-deliverable). This was the dominant
refusal at `1.0.7`: 4,182 of the 4,477 corpus files it rejected stopped at such a line.

**6 — an `` `include `` whose path is a macro (§22.5.1)**

```json
{"kind": "include", "body": {"path": {"kind": "macro", "body": {"name": [[" "], "`pkg_file"], "arguments": "(core)"}},
                             "comment": [[], []]}}
```

`arguments` is the `( … )` span as one string, or `[]` for a bare `` `include `NAME ``. The span is lexed per §22.5.1
(release `1.0.9`, ledger `SVPP-0015`): a `)` inside a string, a block comment, an escaped identifier or a `[ ]` / `{ }`
pair does not end it — `` `include `M(")") `` is one usage whose `arguments` is `"(\")\")"`. The other two path
kinds are `{kind: "quoted", text}` and `{kind: "angle", text}`.

## A walker for these kinds

```python
import json

def walk(items, depth=0):
    for item in items:
        kind, body = item["kind"], item.get("body")
        if kind == "conditional":
            walk(body["if_branch"]["items"], depth + 1)
            for branch in body["elsif_branches"]:
                walk(branch["items"], depth + 1)
            if body["else_branch"]:
                walk(body["else_branch"]["items"], depth + 1)
        elif kind == "line":
            print("  " * depth, "line", body["number"][-1], body["filename"][-1], "level", body["level"][-1])
        elif kind == "pragma":
            print("  " * depth, "pragma", body["name"][-1], len(body["expressions"] or []), "expression(s)")
        elif kind == "define":
            parts = body["body"]["fragments"] if body["body"] else []
            lines = 1 + sum(1 for f in parts if f["kind"] == "line_continuation")
            print("  " * depth, "define", body["name"][-1], f"({lines} source line(s))")
        else:
            print("  " * depth, kind)

root = json.load(open("clause22.json"))["content"]["Json"]
walk(root["items"])
```

Dispatch on `kind`, never on the branch index: the eight new kinds shifted every later `pp_item` index in `1.0.8`.

## What the parser refuses, and why

Each line below is a case of the reproducer bank (`docs/tasks/artifacts/sv_corpus_grad/svpp_corpus_census/bank/`, run by
`census.py --bank`), refused on purpose:

| Input | Refused because |
|---|---|
| `` `line 5 "f.sv" 0 // c `` | §22.12: *"Comments are not allowed on the same line as a `` `line `` directive"* |
| `` `line 0 "f.sv" 0 `` | the number *"shall be a positive integer"* |
| `` `line 5 "f.sv" 3 `` | the level *"shall be 0, 1, or 2"* |
| `` `unconnected_drive pullup `` | §22.9: the argument is `pull1` or `pull0` |
| `` `begin_keywords "1800-2010" `` | not a Syntax 22-10 `version_specifier` (§22.14: an unrecognized one *"shall be an error"*) |
| `` `pragma p a, `` | Syntax 22-8 needs an expression after the comma, and nothing may follow a pragma on its line |
| `` `include "a.sv" x `` | §22.4: *"Only white space or a comment may appear on the same line as the `` `include `` compiler directive"* (accepted through `1.0.7`) |
| `` `include `else `` | a directive keyword is not a macro |
| `` `timescale 1ns `` | Syntax 22-6 needs the precision — and a malformed directive is never re-read as a macro-usage line |
| a stray `` `else `` | no open conditional |

## Why this example

It is the smallest file that crosses every change `1.0.8` made: three of the seven new directive kinds, a define that
spans two lines with a string literal holding backticks, a macro-usage line, and a macro-valued include path. If your
walker handles this file, it handles schema `6`.
