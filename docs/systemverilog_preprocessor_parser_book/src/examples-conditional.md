# Worked Example: Conditional Compilation

The companion to [Single Define](examples-single-define.md): this one
walks the **recursive conditional-compilation tree** — the one place the
sv_preprocessor AST nests. Every JSON value here is the **real, captured
output** of `generated/systemverilog_preprocessor_parser.rs` (parser
release `1.0.3`, AST-dump schema version `3`, 66 return annotations) and
is the regression-locked `conditional` sample in
`rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`.
Nothing here is idealized.

> Schema `2` note: at `1.0.1` (schema `1`) this exact input emitted the
> malformed `"if_branch.keyword": {"items":
> "<invalid_sequence_access>", "keyword": [[], "\`ifdef"], "macro":
> "<invalid_sequence_access>", "tail": "<invalid_sequence_access>"}`
> object — a real parser defect (`SVPP-0001`) fixed in `1.0.2` by
> lifting the inline `(kw_ifdef | kw_ifndef)` alternation into a named
> `pp_if_keyword` rule (the proven `rtl_const_expr` RTL-CE-Slice-2 /
> `systemverilog.ebnf` idiom; see
> [Schema Versioning](schema-versioning.md)). The shape below is the
> corrected, gate-locked output: `if_branch.keyword` is now the typed
> polarity object `{"kind": "ifdef"}`.

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
content of `/tmp/cc.json` — note `if_branch.keyword` is now the clean
typed polarity object `{"kind": "ifdef"}` (the `1.0.2` `SVPP-0001` fix),
explained in
[The `if_branch.keyword` shape and the SVPP-0001 fix](#the-if_branchkeyword-shape-and-the-svpp-0001-fix):

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
                "kind": "ifdef"
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

## The consumer-facing view (parsed `dump_json`)

Downstream code calls the public API ([Public API Surface](public-api.md)),
which hands back a `NamedGrammarAstDumpOutcome` whose `ast_dump` is an
`AstDumpPayload` (real fields:
`dump_json`/`truncated`/`full_bytes`/`emitted_bytes` — there is no
`root` field). After confirming `truncated == false`,
`serde_json::from_str(&payload.dump_json)` yields the typed root object
below, which is byte-identical to `content.Json` above (no abridgement;
the fixed `keyword: {"kind": "ifdef"}` shape included):

```json
{
  "type": "systemverilog_preprocessor_file",
  "items": [
    {
      "kind": "conditional",
      "body": {
        "if_branch": {
          "keyword": { "kind": "ifdef" },
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
grammar order for readability; the JSON *value* is identical, the typed
`keyword: {"kind": "ifdef"}` polarity object and all.)

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
    `pp_if_branch := pp_if_keyword macro_name directive_tail? newline pp_item* -> {keyword: $1, macro: $2, tail: $3, items: $5}`:
    - **`keyword: { "kind": "ifdef" }`** — `$1`, the typed
      `pp_if_keyword` polarity object from
      `pp_if_keyword := kw_ifdef -> {kind: "ifdef"} | kw_ifndef -> {kind: "ifndef"}`.
      Read the conditional polarity directly from
      `if_branch.keyword.kind` (`"ifdef"` here; `"ifndef"` for an
      `` `ifndef`` guard). This is the `1.0.2` `SVPP-0001` fix — at
      `1.0.1` / schema `1` this field was the malformed
      `<invalid_sequence_access>` object; see
      [The `if_branch.keyword` shape and the SVPP-0001 fix](#the-if_branchkeyword-shape-and-the-svpp-0001-fix).
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

## The `if_branch.keyword` shape and the SVPP-0001 fix

`pp_if_branch`'s `keyword: $1` is now the typed **`pp_if_keyword`**
polarity object. The grammar is:

```ebnf
pp_if_keyword := kw_ifdef  -> {kind: "ifdef"}
               | kw_ifndef -> {kind: "ifndef"}
pp_if_branch  := pp_if_keyword macro_name directive_tail? newline pp_item*
              -> {keyword: $1, macro: $2, tail: $3, items: $5}
```

so for the input above `if_branch.keyword` is the clean typed object:

```json
"keyword": { "kind": "ifdef" }
```

For an `` `ifndef`` guard it would be `{ "kind": "ifndef" }`. The
consumer reads the conditional polarity directly from
`if_branch.keyword.kind` — `"ifdef"` means *defined-true* (compile the
body when the macro is defined), `"ifndef"` means *defined-false*
(compile the body when the macro is **not** defined).

> Schema `2` note: at `1.0.1` (schema `1`) this exact field emitted the
> malformed object
> `{"items": "<invalid_sequence_access>", "keyword": [[], "\`ifdef"],
> "macro": "<invalid_sequence_access>", "tail":
> "<invalid_sequence_access>"}` — a real parser defect (`SVPP-0001`)
> fixed in `1.0.2`; the shape above is the corrected, gate-locked
> output. The defect's root cause: `pp_if_branch`'s `keyword: $1`
> pointed at an **inline alternation group** `(kw_ifdef | kw_ifndef)`,
> and PGEN's emit-time machinery did not resolve a bare `$N` reference
> to an inline alternation group cleanly — it substituted the literal
> sentinel string `"<invalid_sequence_access>"` into the mis-recursed
> positional slots. This is the same emit-time defect class first fixed
> for the `rtl_const_expr` `binop_chain` `rest` field in RTL-CE-Slice-2
> (see that book's schema-versioning chapter, repo path
> `docs/rtl_const_expr_parser_book/src/schema-versioning.md`). The
> `1.0.2` fix applies the identical, proven playbook: lift the inline
> `(kw_ifdef | kw_ifndef)` alternation into the **named**
> `pp_if_keyword` rule (the `systemverilog.ebnf` op-chain idiom),
> keeping `pp_if_branch`'s annotation unchanged — only `$1` now binds
> the clean named rule. The history is kept honestly here and in
> [Schema Versioning](schema-versioning.md) (the schema `1.0.0` row);
> it is not whitewashed.

What is safe to rely on for this field, on the **current** release
(`1.0.2`, schema `2`):

- **`if_branch.keyword.kind`** is the typed polarity string `"ifdef"`
  or `"ifndef"` — branch on it directly. There is **no**
  `<invalid_sequence_access>` anywhere in the conditional tree any more.
- **`if_branch.macro`** is the guard macro name envelope
  (`[ [ " " ], "X" ]` here — name at index `[1]`); **`if_branch.tail`**
  is the optional `directive_tail?` (`[]` here); **`if_branch.items`**
  is the recursively-nested `pp_item*` body. All three are, and always
  were, correct — they are the *outer* `if_branch` fields, distinct
  from the (now-removed) malformed `keyword` sub-keys of the pre-fix
  shape.

The robust consumer reads the polarity from `if_branch.keyword.kind`
and the guard macro from `if_branch.macro`. A consumer written against
the pre-fix `1.0.1` schema-`1` shape (treating `keyword` as
opaque/text-only) must repin to schema `2` and switch to the
`keyword.kind` discriminator — this fix bumped the schema `1 → 2` and
has a [Schema Versioning](schema-versioning.md) row and a
[Changelog Index](changelog-index.md) entry.

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
/// envelope, skipping empty-array placeholders. (The defensive
/// `<invalid_sequence_access>` skip arm is kept so the helper stays
/// correct against any pre-`1.0.2` schema-`1` dump a consumer might
/// still have vendored; the `1.0.2` parser never emits that sentinel.)
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

// The AST-dump schema version you integrated against, pinned from the
// contract (NOT a field of AstDumpPayload):
const SVPP_AST_SCHEMA_VERSION: u32 = 3;

match outcome.status {
    ParseStatus::Success => {
        let dump = outcome.ast_dump.expect("Success carries an AST dump");
        assert!(!dump.truncated, "dump_json would hold the truncation envelope");
        let _ = SVPP_AST_SCHEMA_VERSION; // re-check vs the contract on PGEN bumps

        // AstDumpPayload exposes dump_json/truncated/full_bytes/emitted_bytes;
        // parse dump_json to get the typed root object.
        let root: serde_json::Value =
            serde_json::from_str(&dump.dump_json).expect("dump_json is valid JSON");
        assert_eq!(root["type"], "systemverilog_preprocessor_file");

        let cond = &root["items"][0];
        assert_eq!(cond["kind"], "conditional");
        let body = &cond["body"];

        // --- if_branch ---
        let ifb = &body["if_branch"];
        // `keyword` is the typed pp_if_keyword polarity object as of the
        // 1.0.2 SVPP-0001 fix: read the `ifdef`/`ifndef` polarity
        // directly from keyword.kind (was the malformed
        // <invalid_sequence_access> object at 1.0.1 / schema 1).
        assert_eq!(ifb["keyword"]["kind"], "ifdef");
        // The guard macro is the outer field.
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

The `keyword` polarity is read directly from the typed
`if_branch.keyword.kind` discriminator (`"ifdef"` / `"ifndef"`) — no
text-walk is needed for it any more, as of the `1.0.2` `SVPP-0001`
fix. The `extract_text` helper still keeps an `Object` arm and a
defensive `<invalid_sequence_access>` skip versus the
[Single Define](examples-single-define.md) version, so it stays correct
even against a pre-`1.0.2` schema-`1` dump a consumer might still have
vendored. The
[Walking the AST](walking-the-ast.md#walking-the-conditional-compilation-tree)
chapter has the full recursive `handle_conditional` dispatcher; this
example is the concrete instantiation of it.

## Why this example is the canonical conditional-tree test

- **It exercises every `pp_conditional` node** —
  `pp_conditional → pp_if_branch` (with the typed `pp_if_keyword`
  polarity) / `pp_else_branch`, the empty `elsif_branches`, the
  absent-optional `tail: []`, and the recursive `pp_item*` branch
  bodies — in 46 bytes.
- **It proves the recursion** — each branch body is a full `pp_item`
  (here a `pp_define`), reached one level deeper, so the same dispatcher
  used at the root handles branch contents unchanged.
- **It is the regression-locked `conditional` sample** — it is the
  `conditional` sample in
  `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`.
  Combined with the now-66-entry `declared_annotation_inventory` (which
  includes the 2 new `pp_if_keyword` `{kind: "ifdef"}` /
  `{kind: "ifndef"}` branches), any reversion to the pre-`1.0.2`
  `<invalid_sequence_access>` shape fails
  `systemverilog_preprocessor_ast_shape_contract` immediately.
- **It honestly transitions a real released-parser defect** — the
  `if_branch.keyword` field was the malformed
  `<invalid_sequence_access>` object at `1.0.1` / schema `1`
  (`SVPP-0001`); the schema-`2` corrected shape (`{kind: "ifdef"}`) is
  shown, with the pre-fix history kept (not whitewashed) in the
  schema-`2` transition note and
  [Schema Versioning](schema-versioning.md). A book that hid the defect
  history would mislead every downstream `` `ifdef``-handling
  integration repinning across the schema bump.

For the non-recursive baseline shape see
[Single Define](examples-single-define.md); for the full per-rule field
tables (including `pp_if_keyword` / `pp_elsif_branch` / `condition_expr`
/ the atom families) see [Top-Level Rules](rules-top-level.md); for the
schema `1 → 2` `SVPP-0001` fix row see
[Schema Versioning](schema-versioning.md).
