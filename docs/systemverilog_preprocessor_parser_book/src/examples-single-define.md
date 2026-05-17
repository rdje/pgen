# Worked Example: Single Define

A "what does the AST actually look like" walkthrough for the smallest
sv_preprocessor input that produces a complete typed directive — a single
`` `define``. Every JSON value shown here is the **real, captured output**
of `generated/systemverilog_preprocessor_parser.rs` (parser release
`1.0.3`, AST-dump schema version `3`, 66 return annotations) — not a
hypothetical shape — and it is the exact input the AST shape contract
regression-locks (sample `single_define` in
`rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`).

## Input

```systemverilog
`define FOO 1
```

(With a trailing newline. The whole input is 14 bytes:
`` `define FOO 1\n``.)

This is the canonical first-test input because it is the smallest source
that exercises `systemverilog_preprocessor_file → pp_item → pp_define →
macro_body → macro_body_fragment` — five layers of the typed surface —
while having no formal parameter list and a single-fragment body, so the
tree is shallow enough to reason about by hand, with two instructive
exceptions (the `name` and the fragment `body` envelopes, below).

## Reproducing the dump

The sv_preprocessor parser is on-demand-only (see
[Build Recipe](build-recipe.md)). With
`generated/systemverilog_preprocessor_parser.rs` in place, build the probe
with the generated backend and dump the AST:

```bash
cd rust
cargo build --release --features generated_parsers --bin parseability_probe
printf '`define FOO 1\n' > /tmp/sd.svpp
./target/release/parseability_probe \
    --parse-dump-ast-pretty systemverilog_preprocessor /tmp/sd.svpp /tmp/sd.json
# -> parse_full passed for grammar 'systemverilog_preprocessor' on '/tmp/sd.svpp'
```

## The captured AST (raw probe envelope)

`parseability_probe --parse-dump-ast-pretty` serialises the parser's
`ParseNode` directly, so the dump is wrapped in the parse-node envelope
(`content` / `rule_name` / `span`) described in
[AST Envelope Structure](ast-envelope.md). This is the byte-exact
content of `/tmp/sd.json`:

```json
{
  "content": {
    "Json": {
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
              "FOO"
            ]
          },
          "kind": "define"
        }
      ],
      "type": "systemverilog_preprocessor_file"
    }
  },
  "rule_name": "systemverilog_preprocessor_file",
  "span": {
    "end": 14,
    "start": 0
  }
}
```

`content.Json` is the typed AST. `rule_name` is the entry rule
(`systemverilog_preprocessor_file`); `span` is the byte range the parse
covered (`0..14` — the whole input, including the trailing newline).

## The consumer-facing view (parsed `dump_json`)

Downstream code does not call the probe — it calls the public API
([Public API Surface](public-api.md)), which hands back a
`NamedGrammarAstDumpOutcome` whose `ast_dump` is an `AstDumpPayload`
(real fields: `dump_json`/`truncated`/`full_bytes`/`emitted_bytes` —
there is no `root` field). After confirming `truncated == false`,
`serde_json::from_str(&payload.dump_json)` yields the typed root object
below — the shape every per-rule chapter in this book describes:

```json
{
  "type": "systemverilog_preprocessor_file",
  "items": [
    {
      "kind": "define",
      "body": {
        "name": [ [ " " ], "FOO" ],
        "formals": [],
        "body": {
          "fragments": [
            { "kind": "text", "body": [ [ " " ], "1" ] }
          ]
        }
      }
    }
  ]
}
```

(This is byte-identical to the worked example in
[The Json Carrier](json-carrier.md#worked-example-a-single-define-directive);
both are the same real capture. The object-key order differs only because
the probe dump emits keys alphabetically — see
[Determinism](json-carrier.md#determinism) — while the prose view above
groups them readably; the JSON *value* is identical.)

## Field-by-field walk

- **`type: "systemverilog_preprocessor_file"`** — the typed root object
  from
  `systemverilog_preprocessor_file := pp_item* -> {type: "systemverilog_preprocessor_file", items: $1}`.
  This is the only sv_preprocessor rule that carries a `type`
  discriminator; dispatch on `obj["type"]` at the root, then iterate
  `obj["items"]`.
- **`items`** — the `pp_item*` array (a recursive-envelope iteration),
  one element here.
- **`items[0].kind: "define"`** — the `pp_item` dispatcher matched the
  `pp_define` branch (`-> {kind: "define", body: $1}`). Every
  sv_preprocessor dispatcher below the root uses `kind`, never `type`;
  dispatch on `obj["kind"]`.
- **`body`** — the `pp_define` named-field object, from
  `pp_define := kw_define macro_name macro_formals? macro_body? newline? -> {name: $2, formals: $3, body: $4}`:
  - **`name: [ [ " " ], "FOO" ]`** — `$2` is the **un-annotated**
    `macro_name` → `identifier` rule, so it surfaces as that rule's
    recursive envelope, **not** a bare string. The leading `[ " " ]` is
    the inline-trivia prefix (the space after `` `define``); the
    identifier text `"FOO"` is at index `[1]`. See
    [The Json Carrier](json-carrier.md#recursive-envelope-shape-rules-without-annotations)
    for why un-annotated leaves carry an envelope, and
    [Walking the AST](walking-the-ast.md#identifier-and-text-extraction)
    for the robust walk-to-terminal extraction (do not hardcode `[1]`).
  - **`formals: []`** — `$3` is the optional `macro_formals?`. There is
    no `(...)` formal list, so it is the empty array `[]` — the
    absent-optional convention (`[]`, **never** `null`; see
    [The Json Carrier](json-carrier.md#object--array--string--scalar-mapping)).
  - **`body`** — `$4` is the optional `macro_body?`, present here, so it
    is the typed `macro_body` object `{ "fragments": [ … ] }` from
    `macro_body := macro_body_fragment+ -> {fragments: $1}`. (When the
    body is absent — `` `define MAX`` — this field is `[]`, exactly like
    `formals`.)
    - **`fragments`** — the `macro_body_fragment+` array (a
      recursive-envelope iteration), one element here.
    - **`fragments[0]: { "kind": "text", "body": [ [ " " ], "1" ] }`** —
      the `macro_body_fragment` `"text"` branch
      (`macro_body_text -> {kind: "text", body: $1}`). `macro_body_fragment`
      is a 9-kind dispatcher (`token_paste`, `stringize`,
      `macro_reference`, `text`, `lparen`, `rparen`, `comma`, `question`,
      `colon`); only `"macro_reference"` and `"text"` carry a `body`. Its
      `body` is the un-annotated `macro_body_text` envelope
      `[ [ " " ], "1" ]` — same `[ <trivia-prefix>, "<text>" ]` shape as
      `name`; the replacement text `"1"` is at index `[1]`.

The text-extraction rule of thumb is uniform across the sv_preprocessor
surface: any field bound to an un-annotated identifier-or-text rule
(`name`, `macro`, every `"text"`/`"macro_reference"` atom's `body`) is the
leaf rule's recursive envelope `[ <trivia-prefix>, "<text>" ]`, **not** a
bare string. Walk to the terminal string rather than expecting a scalar.
This was confirmed against a second input: `` `define MAX`` →
`{ "name": [ [ " " ], "MAX" ], "formals": [], "body": [] }` (bare define,
both optionals `[]`).

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
/// envelope, without assuming a fixed array depth or offset.
fn extract_text(node: &serde_json::Value) -> Option<String> {
    match node {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Array(items) => items.iter().find_map(extract_text),
        _ => None,
    }
}

let outcome = parse_grammar_profile_ast_dump_named(
    "systemverilog_preprocessor",
    "default",
    "`define FOO 1\n",
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

        let item = &root["items"][0];
        assert_eq!(item["kind"], "define");

        let define = &item["body"];

        // `name` is the un-annotated macro_name/identifier envelope:
        // walk to the terminal rather than indexing a fixed offset.
        assert_eq!(extract_text(&define["name"]).as_deref(), Some("FOO"));

        // Absent optional `macro_formals?` is `[]`, never null/missing.
        assert!(define["formals"].as_array().unwrap().is_empty());

        // `body` is the typed macro_body { fragments: [...] }.
        let fragments = define["body"]["fragments"].as_array().unwrap();
        assert_eq!(fragments.len(), 1);
        assert_eq!(fragments[0]["kind"], "text");
        // The "text" fragment's body is the macro_body_text envelope.
        assert_eq!(
            extract_text(&fragments[0]["body"]).as_deref(),
            Some("1"),
        );
    }
    ParseStatus::Failure => {
        eprintln!("parse failed: {:?}", outcome.diagnostic);
    }
}
```

## Why this example is the canonical first-test

- **It exercises `systemverilog_preprocessor_file → pp_item → pp_define →
  macro_body → macro_body_fragment`** — five layers of the typed surface
  in 14 bytes.
- **It has no formal parameter list and a single-fragment body** — so the
  tree is shallow enough to verify by hand while still proving the
  `kind`-dispatch path and the absent-optional `[]` convention.
- **It teaches the identifier/text-envelope rule of thumb early.** Both
  `name` and the fragment `body` are envelopes, not bare strings; meeting
  that on the smallest possible input is the point.
- **It is the regression-locked sample** `single_define` in
  `rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`
  (`rule_under_test: "systemverilog_preprocessor_file"`,
  `expected_content_kind: "json_object"`,
  `expected_json_object_keys_present: ["type", "items"]`,
  `expected_json_object_string_values: {"type":
  "systemverilog_preprocessor_file"}`, `drift_status: "aligned"`). Drift
  in this shape fails the sv_preprocessor AST shape-contract test
  immediately, surfacing the slice that introduced it.

If your downstream integration cannot parse this input and reach
`type == "systemverilog_preprocessor_file"` with one `define` `pp_item`,
the integration is broken upstream of any sv_preprocessor-specific
concern. Start triage here, then move on to
[Conditional Compilation](examples-conditional.md) for the recursive
`` `ifdef`` tree and the [Top-Level Rules](rules-top-level.md) shape
reference.
