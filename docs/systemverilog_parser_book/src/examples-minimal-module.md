# Worked Example: Minimal Module

A "what does the AST actually look like" walkthrough for the simplest possible SystemVerilog input.

## Input

```systemverilog
module m; endmodule
```

(Note the trailing newline — the parser typically requires it for EOF handling on the final construct.)

## Pre-annotation (recursive envelope)

Until the annotation campaign types `systemverilog_file` and its descendants, the AST dump for the minimal module is the recursive-envelope shape — a JSON tree of arrays and strings reflecting the grammar's sequence / quantified / alternation structure.

The exact shape is recorded in `rust/test_data/ast_shape_contract/systemverilog_v1.json` once the parser has been run for first calibration:

```json
{
  "version": 1,
  "grammar": "systemverilog",
  "samples": [
    {
      "name": "minimal_module",
      "input": "module m; endmodule\n",
      "rule_under_test": "systemverilog_file",
      "current_content_kind": "<calibrated_after_first_run>",
      "expected_json_object_keys_present": [],
      ...
    }
  ]
}
```

The `current_content_kind` field is the regression-lock key — once calibrated, any drift in the AST dump shape will fail the `cargo test` AST-shape contract test, surfacing the change to the slice that introduced it.

## Post-annotation (typed shape — landing per slice)

Once the first annotation slice lands and types `systemverilog_file`, this section will show:

```json
// Hypothetical post-slice shape — actual annotation lands per-slice:
{
  "type": "systemverilog_file",
  "items": [
    {
      "type": "description",
      "kind": "module_declaration",
      "name": "m",
      "ports": [],
      "items": []
    }
  ]
}
```

(The actual shape lands when the slice is implemented and verified; this example is a placeholder showing the expected pattern.)

## Walker code for this input

```rust
use pgen::embedding_api::{
    parse_grammar_profile_ast_dump_named, AstDumpOptions, ParseStatus,
};

let outcome = parse_grammar_profile_ast_dump_named(
    "systemverilog",
    "sv_2017",
    "module m; endmodule\n",
    &AstDumpOptions { pretty: true, max_ast_bytes: None },
);

// AST-dump schema version you integrated against, pinned from the
// contract (NOT a field of AstDumpPayload):
const SV_AST_SCHEMA_VERSION: u32 = 3;

match outcome.status {
    ParseStatus::Success => {
        let ast_dump = outcome.ast_dump.expect("Success outcome carries AST dump");
        assert!(!ast_dump.truncated, "dump_json would hold the truncation envelope");
        let _ = SV_AST_SCHEMA_VERSION; // re-check vs the contract on PGEN bumps

        // AstDumpPayload exposes dump_json/truncated/full_bytes/emitted_bytes;
        // parse dump_json to get the typed root object.
        let root: serde_json::Value = serde_json::from_str(&ast_dump.dump_json)
            .expect("dump_json is valid JSON");
        assert_eq!(root["type"], "systemverilog_file");
        println!("AST root: {}", serde_json::to_string_pretty(&root).unwrap());
    }
    ParseStatus::Failure => {
        eprintln!("parse failed: {:?}", outcome.diagnostic);
    }
}
```

## Why this example is the canonical first-test

The minimal module is the canonical first-test for SV parser integration because:

- **It exercises `systemverilog_file → description → module_declaration`** — three layers of the top-level dispatch.
- **It has no statements, ports, or parameters** — so the AST tree is shallow enough to reason about by hand.
- **It is in the integration contract manifest** as `minimal_module` — drift here surfaces immediately.

If your downstream integration cannot parse this input successfully, the integration is broken upstream of any SV-specific concern. Start here when triaging.
