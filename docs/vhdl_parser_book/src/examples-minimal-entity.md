# Worked Example: Minimal Entity

A "what does the AST actually look like" walkthrough for the simplest
VHDL input that produces a complete design unit. Every JSON value shown
here is the **real, captured output** of `generated/vhdl_parser.rs` —
not a hypothetical shape — and it is the exact input the AST shape
contract regression-locks (sample `minimal_entity` in
`rust/test_data/ast_shape_contract/vhdl_v1.json`).

## Input

```vhdl
entity e is end e;
```

(With a trailing newline. The whole input is 18 bytes:
`entity e is end e;\n`.)

This is the canonical first-test input because it is the smallest source
that exercises `vhdl_file → design_unit → entity_declaration` — three
layers of the top-level dispatch — while having no declarative items and
no architecture, so the tree is shallow enough to reason about by hand.

## Reproducing the dump

The VHDL parser is on-demand-only (see [Build Recipe](build-recipe.md)).
With `generated/vhdl_parser.rs` in place, build the probe with the
generated backend and dump the AST:

```bash
cd rust
cargo build --release --features generated_parsers --bin parseability_probe
printf 'entity e is end e;\n' > /tmp/min_entity.vhd
./target/release/parseability_probe \
    --parse-dump-ast-pretty vhdl /tmp/min_entity.vhd /tmp/min_entity_ast.json
# -> parse_full passed for grammar 'vhdl' on '/tmp/min_entity.vhd'
```

## The captured AST (raw probe envelope)

`parseability_probe --parse-dump-ast` serialises the parser's
`ParseNode` directly, so the dump is wrapped in the parse-node envelope
(`content` / `rule_name` / `span`) described in
[AST Envelope Structure](ast-envelope.md):

```json
{
  "content": {
    "Json": {
      "design_units": [
        {
          "body": {
            "end_label": [
              [],
              "e"
            ],
            "items": [],
            "name": [
              [],
              "e"
            ]
          },
          "kind": "entity"
        }
      ],
      "type": "vhdl_file"
    }
  },
  "rule_name": "vhdl_file",
  "span": {
    "end": 18,
    "start": 0
  }
}
```

`content.Json` is the typed AST. `rule_name` is the entry rule
(`vhdl_file`); `span` is the byte range the parse covered (`0..18` — the
whole input).

## The consumer-facing view (parsed `dump_json`)

Downstream code does not call the probe — it calls the public API
([Public API Surface](public-api.md)), which hands back an
`AstDumpPayload` (real fields: `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`
— there is no `root` field). After confirming `truncated == false`,
`serde_json::from_str(&payload.dump_json)` yields the **unwrapped**
typed AST (the same value the probe shows under `content.Json`) — the
shape every per-rule chapter in this book describes:

```json
{
  "type": "vhdl_file",
  "design_units": [
    {
      "kind": "entity",
      "body": {
        "name": [ [], "e" ],
        "items": [],
        "end_label": [ [], "e" ]
      }
    }
  ]
}
```

## Field-by-field walk

- **`type: "vhdl_file"`** — the typed root object from
  `vhdl_file := design_unit* -> {type: "vhdl_file", design_units: $1}`.
  Dispatch on `obj["type"] == "vhdl_file"` at the root.
- **`design_units`** — the `design_unit*` array, one element here.
- **`design_units[0].kind: "entity"`** — the `design_unit` dispatcher
  matched the `entity_declaration` branch
  (`-> {kind: "entity", body: $1}`). Dispatch on `obj["kind"]`.
- **`body`** — the `entity_declaration` named-field object, from
  `entity_declaration := kw_entity identifier kw_is entity_declarative_item* kw_end kw_entity? identifier? semi -> {name: $2, items: $4, end_label: $7}`:
  - **`name: [ [], "e" ]`** — `$2` is the `identifier` rule. `identifier`
    is **un-annotated**, so it surfaces as its recursive-envelope shape,
    not a bare string. The two-element array is the envelope of the
    identifier's internal sequence; the actual text is at `name[1]`
    (`"e"`). See [The Json Carrier](json-carrier.md) for why un-annotated
    leaves carry an envelope and how to extract the text.
  - **`items: []`** — `$4` is `entity_declarative_item*`; this entity has
    no declarative items, so the quantified shape is the empty array.
  - **`end_label: [ [], "e" ]`** — `$7` is the optional trailing
    `identifier?` (the `e` after `end`). It is present here, so it
    carries the same `identifier` envelope as `name`. When the trailing
    label is omitted (`entity e is end;`), `end_label` is `[]` — the
    un-matched-optional shape, **not** `null`.

The text-extraction rule of thumb: a field typed as an `identifier`
position is `[ <prefix-envelope>, "<text>" ]`; read index `[1]` (or walk
to the trailing string) rather than expecting a bare string. This is the
uniform behaviour of every un-annotated `identifier` position in the
VHDL surface, confirmed against a second input
(`entity foo is end;` → `name = [ [], "foo" ]`, `end_label = []`).

## Walker code for this input

Using the stable public API ([Public API Surface](public-api.md)):

```rust
use pgen::embedding_api::{parse_vhdl_1076_2019_ast_dump, AstDumpOptions, ParseStatus};

let outcome = parse_vhdl_1076_2019_ast_dump(
    "entity e is end e;\n",
    &AstDumpOptions { pretty: true, max_ast_bytes: None },
);

// AST-dump schema version you integrated against, pinned from the
// contract (NOT a field of AstDumpPayload):
const VHDL_AST_SCHEMA_VERSION: u32 = 1;

match outcome.status {
    ParseStatus::Success => {
        let dump = outcome.ast_dump.expect("Success carries an AstDumpPayload");
        assert!(!dump.truncated, "dump_json would hold the truncation envelope");
        let _ = VHDL_AST_SCHEMA_VERSION; // re-check vs the contract on PGEN bumps

        // AstDumpPayload exposes dump_json/truncated/full_bytes/emitted_bytes;
        // parse dump_json to get the typed root object.
        let root: serde_json::Value =
            serde_json::from_str(&dump.dump_json).expect("dump_json is valid JSON");
        assert_eq!(root["type"], "vhdl_file");

        let unit = &root["design_units"][0];
        assert_eq!(unit["kind"], "entity");

        // `name` is the un-annotated identifier envelope: read [1] for text.
        let entity_name = unit["body"]["name"][1]
            .as_str()
            .expect("entity name text at name[1]");
        assert_eq!(entity_name, "e");

        // Absent optionals are `[]`, never null. Here the trailing
        // label is present, so end_label[1] == "e".
        let end_label = &unit["body"]["end_label"];
        let label_text = end_label.get(1).and_then(|v| v.as_str()); // Some("e")
        assert_eq!(label_text, Some("e"));

        assert!(unit["body"]["items"].as_array().unwrap().is_empty());
    }
    ParseStatus::Failure => {
        eprintln!("parse failed: {:?}", outcome.diagnostic);
    }
}
```

## Why this example is the canonical first-test

- **It exercises `vhdl_file → design_unit → entity_declaration`** —
  three layers of the top-level dispatch in 18 bytes.
- **It has no declarative items, generics, ports, or architecture** — so
  the tree is shallow enough to verify by hand.
- **It is the regression-locked sample** `minimal_entity` in
  `rust/test_data/ast_shape_contract/vhdl_v1.json`
  (`rule_under_test: "vhdl_file"`,
  `expected_content_kind: "json_object"`,
  `expected_json_object_keys_present: ["type", "design_units"]`). Drift
  in this shape fails the `vhdl_ast_shape_contract_holds_against_running_generated_parser`
  test immediately, surfacing the slice that introduced it.

If your downstream integration cannot parse this input and reach
`type == "vhdl_file"` with one `entity` design unit, the integration is
broken upstream of any VHDL-specific concern. Start triage here, then
move on to richer inputs and the [Top-Level Rules](rules-top-level.md)
shape reference.
