# Worked Example: Groups, Comments and Lines That Do Not Parse

Release `1.0.9` (AST-dump schema `7`) reads passthrough text as LEXEMES and lets a directive begin anywhere on a line.
This chapter walks one nine-line file through the release `parseability_probe` and reads its real dump item by item.

## Input

```systemverilog
`define USE_FAST
`ifdef USE_FAST wire w; `endif
assign c = d; /* the old way:
`else
   assign c = 0; */
`ifdef NOT_DEFINED_INC
`include NOT_DEFINED_INC
`elsif FALLBACK wire f;
`endif
```

Release `1.0.8` gets three of these wrong. It reads line 2's `` wire w; `endif `` as the `` `ifdef ``'s tail, so the
conditional never closes. It reads line 4's `` `else `` as a real directive, though it sits inside a comment. And it
refuses line 7, though the group that holds it may be skipped.

## Reproducing the dump

```bash
rust/target/release/parseability_probe --parse-dump-ast-pretty systemverilog_preprocessor groups.sv groups.json
# -> parse_full passed for grammar 'systemverilog_preprocessor' on 'groups.sv' (AST dump: groups.json)
```

## Item by item

**Line 2 — a conditional on one line.**

```json
{"kind": "conditional", "body": {"if_branch": {"keyword": {"kind": "ifdef"}, "macro": [[" "], "USE_FAST"],
  "tail": [], "items": [{"kind": "non_directive_line", "body": {"text": [[" "], "wire w; "]}}]},
  "elsif_branches": [], "else_branch": []}}
```

The text after `` `ifdef USE_FAST `` is the group's first ITEM, and the `` `endif `` on the same line closes the
conditional. IEEE 1800-2023 §22.6: a block of text ends at the next `` `elsif `` / `` `else `` / `` `endif ``, and
*"White space is not considered when determining termination"*. `tail` holds only a trailing `//` comment.

**Lines 3 – 5 — a comment hides a directive.**

```json
{"kind": "non_directive_line", "body": {"text": [[], "assign c = d; /* the old way:\n`else\n   assign c = 0; */"]}}
```

The block comment opened on line 3 runs to its `*/` on line 5, so the passthrough item spans three lines and the
`` `else `` is comment text. IEEE 1800-2023 §22.2: *"Directives are not recognized within comments or string
literals."* (Release `1.0.8`: ledger `SVPP-0010`.)

**Lines 6 – 9 — a group line that does not parse, and an `` `elsif `` with text.**

```json
{"kind": "conditional", "body": {
  "if_branch": {"keyword": {"kind": "ifdef"}, "macro": [[" "], "NOT_DEFINED_INC"], "tail": [],
                "items": [{"kind": "malformed_directive", "body": {"text": "`include NOT_DEFINED_INC"}}]},
  "elsif_branches": [{"macro": [[" "], "FALLBACK"], "tail": [],
                      "items": [{"kind": "non_directive_line", "body": {"text": [[" "], "wire f;"]}}]}],
  "else_branch": []}}
```

`` `include NOT_DEFINED_INC `` is not a valid `` `include ``: the path must be quoted or angle-bracketed. Inside a
conditional group it is RECORDED as a `malformed_directive` item, not refused. The group may be skipped in a
compilation, and §22.6 asks a skipped group only to be lexically valid. Whether a group is skipped is decided by the
expansion stage, which has the command-line symbols and the included files. In a compilation where the group is
live, the pipeline refuses the line. At the top level, live in every compilation, this parser refuses it itself.

`` `elsif FALLBACK wire f; `` reads like line 2: `macro` is the identifier, and the text after it is the group's first
item. Release `1.0.8` read `FALLBACK wire f;` as the condition (ledger `SVPP-0012`).

## What is still refused inside a group

A group line must still be LEXICALLY valid, because §22.6's lexical conventions hold in every group:

| inside a skipped group | verdict | why |
|---|---|---|
| `` `include NOT_DEFINED_INC `` | recorded (`malformed_directive`) | a malformed directive; lexically valid |
| `` `FOO `include "x" `` | recorded | a misplaced directive; lexically valid |
| `initial $display("open);` | **refused** | a string left open (§5.9) |
| `/* open` with no `*/` before the `` `endif `` | **refused** | the comment hides the `` `endif ``, and the conditional never closes |
| a malformed `` `ifdef `` / `` `else `` / `` `endif `` | **refused** | the conditional keywords delimit the groups; they are never recorded |

## A walker for these kinds

```python
import json

def walk(items, depth=0):
    for item in items:
        kind, body = item["kind"], item.get("body")
        if kind == "conditional":
            for branch in [body["if_branch"], *body["elsif_branches"], *( [body["else_branch"]] if body["else_branch"] else [] )]:
                cond = branch.get("macro")
                label = cond[-1] if isinstance(cond, list) else (cond or {}).get("kind", "else")
                print("  " * depth, "group", label)
                walk(branch["items"], depth + 1)
        elif kind == "malformed_directive":
            print("  " * depth, "recorded line:", body["text"])
        else:
            print("  " * depth, kind)

walk(json.load(open("groups.json"))["content"]["Json"]["items"])
```

The condition `macro` is the identifier envelope (an array), or an object with a `kind` (`macro_usage`, or under
`sv_2023` an `expression`) — dispatch on its type.
