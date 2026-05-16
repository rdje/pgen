# Worked Example: Conditional Compilation

The companion to [Single Define](examples-single-define.md): this one
walks the **recursive conditional-compilation tree** — the one place the
sv_preprocessor AST nests. Every JSON value here is the **real, captured
output** of `generated/systemverilog_preprocessor_parser.rs` (parser
release `1.0.1`, AST-dump schema version `1`, 64 return annotations) —
including one real released-parser defect surfaced honestly (the
`if_branch.keyword` `<invalid_sequence_access>` envelope, below). Nothing
here is idealized.

## Input

```systemverilog
`ifdef X
`define A 1
`else
`define B 2
`endif
```

(With a trailing newline. The whole input is 46 bytes:
`` `ifdef X\n`define A 1\n`else\n`define B 2\n`endif\n``.)

This input exercises the `pp_conditional` tree: an `` `ifdef`` guard with
a `` `define`` in its body, an `` `else`` branch with a different
`` `define``, and the terminating `` `endif`` — no `` `elsif``, so the
`elsif_branches` array is empty. It is the smallest source that produces
every node of the conditional family while keeping each branch body to a
single, already-understood `pp_define` (see
[Single Define](examples-single-define.md)).

## Reproducing the dump

The sv_preprocessor parser is on-demand-only (see
[Build Recipe](build-recipe.md)). With
`generated/systemverilog_preprocessor_parser.rs` in place, build the probe
with the generated backend and dump the AST:

```bash
cd rust
cargo build --release --features generated_parsers --bin parseability_probe
printf '`ifdef X\n`define A 1\n`else\n`define B 2\n`endif\n' > /tmp/cc.svpp
./target/release/parseability_probe \
    --parse-dump-ast-pretty systemverilog_preprocessor /tmp/cc.svpp /tmp/cc.json
# -> parse_full passed for grammar 'systemverilog_preprocessor' on '/tmp/cc.svpp'
```

## The captured AST (raw probe envelope)

`parseability_probe --parse-dump-ast-pretty` serialises the parser's
`ParseNode` directly, so the dump is wrapped in the parse-node envelope
(`content` / `rule_name` / `span`) described in
[AST Envelope Structure](ast-envelope.md). This is the byte-exact
content of `/tmp/cc.json` — including the `if_branch.keyword`
`<invalid_sequence_access>` markers, which are shown honestly and
explained in
[The `if_branch.keyword` defect](#the-if_branchkeyword-defect-shown-honestly):

```json
{
  "content": {
    "Json": {
      "items": [
        {
          "body": {
            "else_branch": {
              "items": [
                {
                  "body": {
                    "body": {
                      "fragments": [
                        {
                          "body": [
                            [
                              " "
                            ],
                            "2"
                          ],
                          "kind": "text"
                        }
                      ]
                    },
                    "formals": [],
                    "name": [
                      [
                        " "
                      ],
                      "B"
                    ]
                  },
                  "kind": "define"
                }
              ],
              "tail": []
            },
            "elsif_branches": [],
            "if_branch": {
              "items": [
                {
                  "body": {
                    "body": {
                      "fragments": [
                        {
                          "body": [
                            [
                              " "
                            ],
                            "1"
                          ],
                          "kind": "text"
                        }
                      ]
                    },
                    "formals": [],
                    "name": [
                      [
                        " "
                      ],
                      "A"
                    ]
                  },
                  "kind": "define"
                }
              ],
              "keyword": {
                "items": "<invalid_sequence_access>",
                "keyword": [
                  [],
                  "`ifdef"
                ],
                "macro": "<invalid_sequence_access>",
                "tail": "<invalid_sequence_access>"
              },
              "macro": [
                [
                  " "
                ],
                "X"
              ],
              "tail": []
            }
          },
          "kind": "conditional"
        }
      ],
      "type": "systemverilog_preprocessor_file"
    }
  },
  "rule_name": "systemverilog_preprocessor_file",
  "span": {
    "end": 46,
    "start": 0
  }
}
```

`content.Json` is the typed AST. `rule_name` is the entry rule
(`systemverilog_preprocessor_file`); `span` is `0..46` — the whole input.

## The consumer-facing view (`AstDumpPayload.root`)

Downstream code calls the public API ([Public API Surface](public-api.md)),
which hands back a `NamedGrammarAstDumpOutcome` whose `ast_dump.root` is
the **unwrapped** `content.Json` value. This is byte-identical to
`content.Json` above (no abridgement, defect markers included):

```json
{
  "type": "systemverilog_preprocessor_file",
  "items": [
    {
      "kind": "conditional",
      "body": {
        "if_branch": {
          "keyword": {
            "keyword": [ [], "`ifdef" ],
            "items": "<invalid_sequence_access>",
            "macro": "<invalid_sequence_access>",
            "tail": "<invalid_sequence_access>"
          },
          "macro": [ [ " " ], "X" ],
          "tail": [],
          "items": [
            {
              "kind": "define",
              "body": {
                "name": [ [ " " ], "A" ],
                "formals": [],
                "body": { "fragments": [ { "kind": "text", "body": [ [ " " ], "1" ] } ] }
              }
            }
          ]
        },
        "elsif_branches": [],
        "else_branch": {
          "tail": [],
          "items": [
            {
              "kind": "define",
              "body": {
                "name": [ [ " " ], "B" ],
                "formals": [],
                "body": { "fragments": [ { "kind": "text", "body": [ [ " " ], "2" ] } ] }
              }
            }
          ]
        }
      }
    }
  ]
}
```

(Object-key order differs from the probe dump only because the probe
emits keys alphabetically — see
[Determinism](json-carrier.md#determinism) — while the view above follows
grammar order for readability; the JSON *value* is identical, defect
markers and all.)

## Field-by-field walk

- **`type: "systemverilog_preprocessor_file"`** — the typed root; one
  `pp_item` here.
- **`items[0].kind: "conditional"`** — the `pp_item` dispatcher matched
  the `pp_conditional` branch (`-> {kind: "conditional", body: $1}`).
- **`body`** — the `pp_conditional` named-field object, from
  `pp_conditional := pp_if_branch pp_elsif_branch* pp_else_branch? pp_endif -> {if_branch: $1, elsif_branches: $2, else_branch: $3}`.
  Note `pp_endif` (`$4`) is matched but **not** surfaced — there is no
  `endif` field; the branch list is exhaustive on its own.
  - **`if_branch`** — `$1`, the `pp_if_branch` object from
    `pp_if_branch := (kw_ifdef | kw_ifndef) macro_name directive_tail? newline pp_item* -> {keyword: $1, macro: $2, tail: $3, items: $5}`:
    - **`keyword`** — `$1`, the `(kw_ifdef | kw_ifndef)` alternation
      group. **This field is defective in release `1.0.1`** — see
      [The `if_branch.keyword` defect](#the-if_branchkeyword-defect-shown-honestly).
      The actual matched token `` `ifdef`` is recoverable at
      `keyword["keyword"][1]`; the three sibling keys
      (`items` / `macro` / `tail`) are the literal string
      `"<invalid_sequence_access>"`.
    - **`macro: [ [ " " ], "X" ]`** — `$2`, the un-annotated `macro_name`
      → `identifier` envelope (the guard macro). Same
      `[ <trivia-prefix>, "<text>" ]` shape as in
      [Single Define](examples-single-define.md); the name `"X"` is at
      index `[1]`. Walk to the terminal (see
      [Walking the AST](walking-the-ast.md#identifier-and-text-extraction)).
    - **`tail: []`** — `$3`, the optional `directive_tail?`. There is no
      trailing directive text after `` `ifdef X``, so it is the empty
      array `[]` — the absent-optional convention (`[]`, never `null`).
    - **`items`** — `$5`, the branch's recursively-nested `pp_item*`
      body. One element here: the already-familiar
      `{ "kind": "define", "body": { "name": [ [ " " ], "A" ], "formals":
      [], "body": { "fragments": [ { "kind": "text", "body": [ [ " " ],
      "1" ] } ] } } }` — the `` `define A 1`` from
      [Single Define](examples-single-define.md), reached one level
      deeper. The same `handle_pp_item` dispatcher recurses here.
  - **`elsif_branches: []`** — `$2`, the `pp_elsif_branch*` array. There
    is no `` `elsif``, so it is the empty array `[]`. (When present, each
    entry is `pp_elsif_branch → {condition, items}`, where `condition` is
    the flat `condition_expr → {atoms: [...]}` atom list — **not** a
    parsed boolean tree; see
    [The Json Carrier](json-carrier.md#no-expression-cascade).)
  - **`else_branch`** — `$3`, the optional `pp_else_branch?`, present
    here, so it is the typed `pp_else_branch` object from
    `pp_else_branch := kw_else directive_tail? newline pp_item* -> {tail: $2, items: $4}`:
    - **`tail: []`** — `$2`, the optional `directive_tail?` after
      `` `else``; absent, so `[]`.
    - **`items`** — `$4`, the else body's `pp_item*` array. One element:
      the `` `define B 2`` — `name` envelope `[ [ " " ], "B" ]`, single
      `"text"` fragment with `body` envelope `[ [ " " ], "2" ]`.
    - When there is no `` `else``, `else_branch` is the empty array `[]`
      (the absent-optional convention) — **not** `null` and **not** a
      missing key. Test `else_branch` for an empty array, not for null.

The branch bodies (`if_branch.items`, each `elsif_branches[i].items`,
`else_branch.items`) are themselves `pp_item*` arrays, so the conditional
tree nests recursively: a `pp_conditional` can appear inside any branch's
`items`, and the same `handle_pp_item` dispatcher handles it.

## The `if_branch.keyword` defect (shown honestly)

`pp_if_branch`'s `keyword: $1` points at the **alternation group**
`(kw_ifdef | kw_ifndef)`. In release `1.0.1` the generated parser does
not resolve a bare `$N` reference to an inline alternation group cleanly:
instead of the matched keyword token's envelope, `keyword` is captured as
a **malformed nested object** where the matched alternative's envelope is
present under a `"keyword"` key but the other positional slots are filled
with the literal sentinel string `"<invalid_sequence_access>"`:

```json
"keyword": {
  "keyword": [ [], "`ifdef" ],
  "items": "<invalid_sequence_access>",
  "macro": "<invalid_sequence_access>",
  "tail": "<invalid_sequence_access>"
}
```

`"<invalid_sequence_access>"` is PGEN's emit-time sentinel for a
return-annotation positional reference (`$N`) that could not be resolved
to a sequence element — the same class of released-parser defect first
documented for the rtl_const_expr `binop_chain` `rest` field (see that
book's schema-versioning chapter, repo path
`docs/rtl_const_expr_parser_book/src/schema-versioning.md`, for the
historical precedent). It is **not** valid AST data and **not** a shape a
consumer should attempt to interpret beyond text recovery.

What is and is not safe to rely on for this field, on the **current**
release:

- **Recoverable:** the matched directive token text is at
  `if_branch["keyword"]["keyword"][1]` (here `` `ifdef``; for an
  `` `ifndef`` guard it would be `` `ifndef``). The
  [walk-to-terminal](walking-the-ast.md#identifier-and-text-extraction)
  `extract_text` helper applied to `if_branch["keyword"]` reaches
  `` `ifdef`` and skips the sentinel strings (it returns the first
  string it finds; with alphabetical key order that is the sentinel —
  see the defensive variant in the walker code below).
- **Not recoverable from `keyword`:** nothing else lives there. The
  `items` / `macro` / `tail` sibling keys inside `keyword` are sentinels,
  **not** the branch's real `items` / `macro` / `tail` — those are the
  *outer* `if_branch.items` / `if_branch.macro` / `if_branch.tail`
  fields, which are correct (`macro = [ [ " " ], "X" ]`, `tail = []`,
  `items = [ <the `define`> ]`). Do **not** read branch data out of the
  `keyword` sub-object.

This defect is reported, not hidden: it is surfaced verbatim in the
captured AST above and called out here. The robust consumer reads the
guard *macro name* (which it almost always needs) from the **correct**
outer `if_branch["macro"]` field, and treats the `keyword` field as
"directive token text only, via the nested `keyword` key". A future
correctness slice that lifts `(kw_ifdef | kw_ifndef)` into a named rule
(the same fix pattern rtl_const_expr used for its operator alternations)
would bump the schema version and get a
[Schema Versioning](schema-versioning.md) row and a
[Changelog Index](changelog-index.md) entry; until then this is the real
schema-`1` shape and consumers must not target a cleaner one.

## Walker code for this input

The sv_preprocessor family exposes the **generic-by-grammar** host
surface — there is **no** `parse_systemverilog_preprocessor` convenience
function (see [Public API Surface](public-api.md)). The stable entry
point is `parse_grammar_profile_ast_dump_named` (string-name overload,
grammar `"systemverilog_preprocessor"`, profile `"default"`):

```rust
use pgen::embedding_api::{
    parse_grammar_profile_ast_dump_named, AstDumpOptions, ParseStatus,
};

/// Walk to the terminal text inside an un-annotated identifier/text
/// envelope, skipping empty-array placeholders AND the parser's
/// `<invalid_sequence_access>` defect sentinel.
fn extract_text(node: &serde_json::Value) -> Option<String> {
    match node {
        serde_json::Value::String(s) if s == "<invalid_sequence_access>" => None,
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Array(items) => items.iter().find_map(extract_text),
        serde_json::Value::Object(map) => map.values().find_map(extract_text),
        _ => None,
    }
}

let outcome = parse_grammar_profile_ast_dump_named(
    "systemverilog_preprocessor",
    "default",
    "`ifdef X\n`define A 1\n`else\n`define B 2\n`endif\n",
    &AstDumpOptions { pretty: true, max_ast_bytes: None },
);

match outcome.status {
    ParseStatus::Success => {
        let dump = outcome.ast_dump.expect("Success carries an AST dump");
        assert_eq!(dump.schema_version, 1);

        let root = &dump.root;
        assert_eq!(root["type"], "systemverilog_preprocessor_file");

        let cond = &root["items"][0];
        assert_eq!(cond["kind"], "conditional");
        let body = &cond["body"];

        // --- if_branch ---
        let ifb = &body["if_branch"];
        // The directive token is recoverable only via the nested
        // `keyword` key; the other keyword sub-keys are defect sentinels.
        assert_eq!(
            extract_text(&ifb["keyword"]["keyword"]).as_deref(),
            Some("`ifdef"),
        );
        // The guard macro is the CORRECT outer field, not the sentinel
        // inside `keyword`.
        assert_eq!(extract_text(&ifb["macro"]).as_deref(), Some("X"));
        assert!(ifb["tail"].as_array().unwrap().is_empty());
        let if_items = ifb["items"].as_array().unwrap();
        assert_eq!(if_items.len(), 1);
        assert_eq!(if_items[0]["kind"], "define");
        assert_eq!(
            extract_text(&if_items[0]["body"]["name"]).as_deref(),
            Some("A"),
        );

        // --- elsif_branches: [] (no `elsif) ---
        assert!(body["elsif_branches"].as_array().unwrap().is_empty());

        // --- else_branch: typed object (present), [] when absent ---
        let elseb = &body["else_branch"];
        assert!(elseb["tail"].as_array().unwrap().is_empty());
        let else_items = elseb["items"].as_array().unwrap();
        assert_eq!(else_items.len(), 1);
        assert_eq!(else_items[0]["kind"], "define");
        assert_eq!(
            extract_text(&else_items[0]["body"]["name"]).as_deref(),
            Some("B"),
        );
    }
    ParseStatus::Failure => {
        eprintln!("parse failed: {:?}", outcome.diagnostic);
    }
}
```

The `extract_text` here adds an `Object` arm and an explicit
`<invalid_sequence_access>` skip versus the
[Single Define](examples-single-define.md) version, precisely so it stays
correct over the defective `keyword` sub-object. The
[Walking the AST](walking-the-ast.md#walking-the-conditional-compilation-tree)
chapter has the full recursive `handle_conditional` dispatcher; this
example is the concrete instantiation of it.

## Why this example is the canonical conditional-tree test

- **It exercises every `pp_conditional` node** —
  `pp_conditional → pp_if_branch` / `pp_else_branch`, the empty
  `elsif_branches`, the absent-optional `tail: []`, and the recursive
  `pp_item*` branch bodies — in 46 bytes.
- **It proves the recursion** — each branch body is a full `pp_item`
  (here a `pp_define`), reached one level deeper, so the same dispatcher
  used at the root handles branch contents unchanged.
- **It surfaces a real released-parser defect honestly** — the
  `if_branch.keyword` `<invalid_sequence_access>` shape is shown
  verbatim, explained, and given a safe consumer rule (read the *macro*
  from the correct outer field; treat `keyword` as token-text-only),
  rather than being idealized away. A book that hid this would mislead
  every downstream `` `ifdef``-handling integration.

For the non-recursive baseline shape see
[Single Define](examples-single-define.md); for the full per-rule field
tables (including `pp_elsif_branch` / `condition_expr` / the atom
families) see [Top-Level Rules](rules-top-level.md); for what triggers a
schema bump when the `keyword` defect is fixed see
[Schema Versioning](schema-versioning.md).
