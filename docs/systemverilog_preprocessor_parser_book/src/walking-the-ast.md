# Walking the AST

This chapter is a recommended walker pattern for downstream consumers traversing the PGEN sv_preprocessor AST-dump JSON. It uses real sv_preprocessor rule and `kind` names from the live return-annotation inventory (`generated/systemverilog_preprocessor_return_annotations.json`, 64 annotations across 27 rules, schema version `1`).

## The dual-shape walker

Because the AST tree mixes typed objects (the SVPP-Slice-1 surface) and recursive-envelope arrays (terminal/regex leaves and the literal-text runs), the walker dispatches on JSON value kind, then on the `type` / `kind` discriminator:

```rust
use serde_json::Value as JsonValue;

fn walk(node: &JsonValue) {
    match node {
        JsonValue::Object(obj) => {
            // Typed-shape node. The root carries `type`; every other
            // sv_preprocessor typed shape uses a `kind` discriminator
            // (there is no `binop_chain`/`type`-tagged expression carrier).
            if obj.get("type").and_then(|v| v.as_str())
                == Some("systemverilog_preprocessor_file")
            {
                if let Some(items) = obj.get("items") {
                    walk(items); // recursive-envelope array of pp_item shapes
                }
                return;
            }
            match obj.get("kind").and_then(|v| v.as_str()) {
                Some("define")       => walk_define(obj.get("body")),
                Some("undef")        => walk_undef(obj.get("body")),
                Some("include")      => walk_include(obj.get("body")),
                Some("timescale")    => walk_timescale(obj.get("body")),
                Some("default_nettype") => walk_default_nettype(obj.get("body")),
                Some("conditional")  => walk_conditional(obj.get("body")),
                Some("non_directive_line") => walk_non_directive(obj.get("body")),
                // Bodyless pp_item branches:
                Some("celldefine")    => { /* enter celldefine region */ }
                Some("endcelldefine") => { /* leave celldefine region */ }
                Some("blank_line")    => { /* nothing to do */ }
                _ => {
                    // Unknown kind, a field-only typed object (e.g. the
                    // pp_define named-field shape, or a condition_atom /
                    // macro_body_fragment leaf the walker doesn't special-
                    // case). Fall through to a generic child walk so the
                    // walker stays robust to shapes it doesn't recognise.
                    for (_key, value) in obj.iter() {
                        walk(value);
                    }
                }
            }
        }
        JsonValue::Array(items) => {
            // Recursive-envelope shape (sequence / quantified / `rest`
            // iteration / un-matched optional `[]` / leaf envelope) —
            // walk children.
            for item in items {
                walk(item);
            }
        }
        JsonValue::String(_text) => {
            // Terminal text — identifier / literal / keyword value.
            // No children.
        }
        JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::Null => {
            // Annotation-produced scalars. The current sv_preprocessor
            // surface uses none of these (absent optionals are `[]`, not
            // null), but handle them so future slices don't break the
            // walker.
        }
    }
}
```

## Why the unknown-shape fallback matters

The fallthrough that walks the children of any typed object the walker doesn't special-case is **important**:

```rust
_ => {
    for (_key, value) in obj.iter() {
        walk(value);
    }
}
```

The sv_preprocessor grammar landed its full surface in one batch (SVPP-Slice-1), but subsequent shape-affecting slices can add new `kind` values or restructure a shape. A walker that hard-fails on an unrecognized `kind` will break on every parser-release that extends the grammar. A walker that walks the children of unknown shapes degrades gracefully — it won't extract structured info for shapes it doesn't understand, but it will still reach descendants (including any nested `pp_conditional` tree).

Recommendation: only hard-fail on unknown shapes when you are explicitly pinning to a specific `schema_version` and your test corpus exercises every `kind` value in that schema (see [Schema Versioning](schema-versioning.md)).

## Dispatching the `pp_item` list

Every parse roots at `{ "type": "systemverilog_preprocessor_file", "items": [ ... ] }`. The per-item dispatch is the 10-kind `pp_item`:

```rust
fn handle_pp_item(node: &serde_json::Value) {
    let kind = node.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    let body = node.get("body");
    match kind {
        "define"             => handle_define(body),          // {name, formals, body}
        "undef"              => handle_undef(body),            // {name, comment}
        "include"            => handle_include(body),          // {path, comment}
        "timescale"          => handle_timescale(body),        // {unit, precision, comment}
        "default_nettype"    => handle_default_nettype(body),  // {nettype, comment}
        "celldefine"         => set_celldefine_state(true),    // bodyless
        "endcelldefine"      => set_celldefine_state(false),   // bodyless
        "conditional"        => handle_conditional(body),      // {if_branch, elsif_branches, else_branch}
        "non_directive_line" => handle_non_directive(body),    // {text}
        "blank_line"         => { /* bodyless */ }
        _                    => { /* unknown — degrade gracefully */ }
    }
}
```

The bodyless branches (`"celldefine"`, `"endcelldefine"`, `"blank_line"`) have no `body` key — do not `unwrap()` it. Note that the `pp_celldefine` / `pp_endcelldefine` rules are themselves typed (`{comment}`), but the `pp_item` annotation drops that body, so a consumer that needs the celldefine comment text must re-scan the source line; the AST only carries the marker.

## Walking the conditional-compilation tree

`pp_conditional` is the one recursive structure. Its body is `{if_branch, elsif_branches, else_branch}`; each branch's `items` is itself a `pp_item*` array, so the same `handle_pp_item` dispatcher recurses:

```rust
fn handle_conditional(body: Option<&serde_json::Value>) {
    let Some(c) = body else { return };

    // if_branch: {keyword, macro, tail, items}
    if let Some(ifb) = c.get("if_branch") {
        for it in ifb.get("items").and_then(|v| v.as_array()).into_iter().flatten() {
            handle_pp_item(it); // recursion: a nested `ifdef` lives here
        }
    }

    // elsif_branches: array of {condition, items}; [] when none.
    for eb in c.get("elsif_branches").and_then(|v| v.as_array()).into_iter().flatten() {
        // `condition` is a condition_expr -> {atoms: [...]}, a FLAT atom
        // list (no precedence tree). Evaluate it yourself if needed.
        let _atoms = eb.get("condition").and_then(|v| v.get("atoms"));
        for it in eb.get("items").and_then(|v| v.as_array()).into_iter().flatten() {
            handle_pp_item(it);
        }
    }

    // else_branch: {tail, items}; [] (empty array, NOT null/missing)
    // when there is no `else`.
    match c.get("else_branch") {
        Some(serde_json::Value::Array(a)) if a.is_empty() => { /* no else */ }
        Some(elseb) => {
            for it in elseb.get("items").and_then(|v| v.as_array()).into_iter().flatten() {
                handle_pp_item(it);
            }
        }
        None => {}
    }
}
```

`pp_conditional` does **not** carry the terminating `pp_endif` in its annotation (`pp_endif` is matched but not surfaced), so there is no `endif` field to walk — the branch list is exhaustive on its own.

## Walking the atom / fragment lists

The condition expression, macro default value, and macro body are flat `x+` lists of `kind`-tagged atoms — **not** `{first, rest}` lists, and **not** an expression tree:

```rust
fn handle_atoms(container: &serde_json::Value, key: &str) {
    // key is "atoms" for condition_expr / macro_default_value,
    // "fragments" for macro_body.
    let Some(list) = container.get(key).and_then(|v| v.as_array()) else { return };
    for atom in list {
        match atom.get("kind").and_then(|v| v.as_str()) {
            // Carry a body envelope:
            Some("macro_reference") => visit_macro_ref(atom.get("body")),
            Some("text")            => visit_text(atom.get("body")),
            // Bodyless punctuation/operator markers:
            Some("token_paste") => { /* `` */ }
            Some("stringize")   => { /* `" */ }
            Some("lparen") | Some("rparen") | Some("comma")
            | Some("question") | Some("colon")
            | Some("logical_or") | Some("logical_and") | Some("bang") => { /* token */ }
            _ => { /* unknown — degrade gracefully */ }
        }
    }
}
```

The admitted `kind` set differs by container: `condition_atom` has all 12 (`+ "comma" "logical_or" "logical_and" "bang"`), `macro_body_fragment` has 9 (`+ "comma"`), `macro_default_atom` has the shared 8 only. Walking with a `_ =>` catch-all keeps the same routine correct across all three.

## Identifier and text extraction

sv_preprocessor identifiers (macro names) and literal-text runs are bound to **un-annotated** leaf rules, so they surface as a recursive envelope, **not** a bare JSON string. The text is the terminal string nested inside that envelope (in practice `[ <trivia-prefix>, "<text>" ]`, text at index `[1]`). Walk to the terminal rather than indexing a fixed depth:

```rust
fn extract_text(node: &serde_json::Value) -> Option<String> {
    match node {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Array(items) => {
            // The leaf envelope nests through positional sequence arrays;
            // the payload is the deepest non-empty string. Walk every
            // child and take the first string we reach, skipping the
            // empty-array `[]` trivia/optional placeholders.
            items.iter().find_map(extract_text)
        }
        _ => None,
    }
}
```

For `pp_define` the macro name lives at `body["name"]`; the replacement text is each `body["body"]["fragments"][i]["body"]` for fragments whose `kind == "text"`. For the captured input `` `define FOO 1`` (see [The Json Carrier](json-carrier.md#worked-example-a-single-define-directive)), `name` is `[ [ " " ], "FOO" ]` and the single fragment's `body` is `[ [ " " ], "1" ]` — `extract_text` returns `"FOO"` and `"1"` respectively. The per-rule chapters ([Top-Level Rules](rules-top-level.md)) document the field that holds the identifier/text for each rule that produces one. Always treat these as envelopes, never assume a scalar.

## Iterating `{first, rest}` lists

`macro_formals` is the **only** separated-list rule on the sv_preprocessor surface. It uses the `{first, rest}` carrier: `first` is the leading `macro_formal`, `rest` is the recursive-envelope iteration of the `(comma macro_formal)*` tail, so each `rest` entry wraps a comma token plus one `macro_formal`:

```rust
fn iter_macro_formals<'a>(formals: &'a serde_json::Value) -> Vec<&'a serde_json::Value> {
    let mut out = Vec::new();
    if let Some(first) = formals.get("first") {
        out.push(first);
    }
    if let Some(rest) = formals.get("rest").and_then(|v| v.as_array()) {
        for entry in rest {
            // `entry` is the envelope of one `(comma macro_formal)`
            // iteration; the formal is the last child. Walk to it rather
            // than assuming a fixed index, so a grammar tweak to the
            // separator doesn't break extraction.
            if let Some(elem) = entry.as_array().and_then(|a| a.last()) {
                out.push(elem);
            } else {
                out.push(entry);
            }
        }
    }
    out
}
```

Each yielded element is a `macro_formal` `{name, default}` object (`default` is `[]` when the formal has no `= <default>`). The atom/fragment lists (`condition_expr.atoms`, `macro_default_value.atoms`, `macro_body.fragments`) are **plain `x+` arrays**, not `{first, rest}` — iterate them directly with `handle_atoms` above. This `{first, rest}` shape was **not** flattened to `[$N, $M::2*]` the way the SystemVerilog grammar's lists were in its slice-58 audit; a future flattening slice, if it lands, will get a [Schema Versioning](schema-versioning.md) row.

## Avoiding deep recursion

A pathologically deep nest of `` `ifdef`` / `` `elsif`` / `` `else`` blocks produces a deep AST tree (each branch's `items` recurses through `pp_item` → `pp_conditional`). It is much shallower than the VHDL/rtl_frontend expression cascades (there is no per-operator-level descent here), but defensive consumers should still:

1. **Use an explicit stack-based walker** (push children, pop work items in a loop) instead of recursive function calls. Stack depth becomes irrelevant.
2. **Or grow the thread stack** (e.g. the `stacker` crate). PGEN itself uses a large-stack worker internally for the parser; consumers can do the same for traversal.
3. **For pure AST drop**, use a non-recursive drop — `serde_json::Value`'s default `Drop` is recursive and can blow the stack on deeply-nested values. PGEN's test path uses a large-stack worker to avoid this; downstream consumers should consider similar patterns.

## Schema-version-aware walking

`AstDumpPayload` has **no** `schema_version` (or `root`) field — the real struct is `dump_json`/`truncated`/`full_bytes`/`emitted_bytes`. The AST-dump schema version is not discoverable at runtime from the payload; you **pin** it from `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` § "Contract Identity" as a constant in your consumer and re-validate that pin against the contract's "Schema Versioning" table whenever you bump PGEN. The shape-contract manifest (`rust/test_data/ast_shape_contract/systemverilog_preprocessor_v1.json`) is the machine-checkable lock that fails CI if the parser drifts from the pinned schema.

```rust
// The AST-dump schema version you integrated against (from the contract):
const SVPP_AST_SCHEMA_VERSION: u32 = 1;

let payload = outcome.ast_dump.expect("Success carries an AstDumpPayload");
if payload.truncated {
    // dump_json holds the truncation diagnostic envelope, not the AST.
    return Err("sv_preprocessor AST dump truncated".into());
}
let root: serde_json::Value = serde_json::from_str(&payload.dump_json)?;

// SVPP_AST_SCHEMA_VERSION selects which walker your code was built for.
// When you bump PGEN, diff the contract's Schema Versioning table; if the
// integer schema version moved, update the constant and the walker together.
match SVPP_AST_SCHEMA_VERSION {
    1 => walk_schema_v1(&root),
    // (future) 2 => walk_schema_v2(&root),
    other => eprintln!("no walker compiled for sv_preprocessor AST schema version {other}"),
}
```

See [Schema Versioning](schema-versioning.md) for what triggers a schema bump and what stays stable within a single schema version.
