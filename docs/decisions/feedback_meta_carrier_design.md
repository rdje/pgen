<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_meta_carrier_design.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: _meta carrier design (approved 2026-05-05) — span/rule/line_col/trivia info on typed AST
description: User-approved architectural design for adding _meta information (span, rule, branch_index, line_col, source_text, trivia/comments) to every typed JSON object produced by return annotations. Implementation pending.
type: feedback
originSessionId: f74f4acc-7183-408a-ae5d-dcdce15a103c
---
## Approved design (user direction 2026-05-05)

User raised concern that the typed-shape return-annotation campaign produces clean JSON shapes but **strips span/rule_name info** that the underlying ParseNode tree carries. Downstream compilers/elaborators (Nexsim, RGX, etc.) need this info for diagnostics, LSP integration, source mapping.

**Approved approach: Option A — universal `_meta` wrapper, additive, schema stays at 1.**

Every typed JSON object emitted by a return annotation gets a sibling `_meta` field carrying span + rule + branch_index + line_col + source_text + (trivia/comments later).

```json
{
  "_meta": {
    "span": {"start": 7, "end": 14},
    "line_col": {"start": {"line": 1, "col": 8}, "end": {"line": 1, "col": 15}},
    "rule": "module_ansi_header",
    "branch_index": 0,
    "source_text": "module m"
  },
  "type": "systemverilog_file",
  "source_text": [...]
}
```

## Field tier inventory (from the 2026-05-05 design discussion)

### Tier 1 — free, always emit
- `span: {start, end}` — from `ParseNode.span`
- `rule: "<rule_name>"` — from `ParseNode.rule_name`
- `branch_index: <int>` — for Or-rule branches (parser-side knowledge)

### Tier 2 — needs source-text plumbing (one-time per parse)
- `line_col: {start: {line, col}, end: {line, col}}` — computed from span + line-table (one O(n) scan per parse, then O(log n) per lookup)
- `source_text: "<slice>"` — `input[span.start..span.end]`

### Tier 3 — explicit handling, larger scope
- `leading_trivia` — whitespace + comments before the matched span
- `trailing_trivia` — whitespace + comments after
- `comments: [{kind, text, span}, ...]` — extracted from trivia

### Tier 4 — nice-to-have
- `node_id: <stable_int>` — for incremental elaboration / AST-diff
- `error_recovery: true` flag — for synthetic nodes (when error-recovery codegen lands)

User direction: collect as much info as possible; downstream consumers decide what to use.

## API control

`AstDumpOptions.meta_level` enum:
- `None` — omit `_meta` entirely (back-compat)
- `Basic` — Tier 1 (default for new builds)
- `Full` — Tier 1 + Tier 2
- `Trivia` — Tier 1 + 2 + 3

## Implementation order

**Phase 1**: Tier 1 fields. Modify `ast_return_transform.rs` codegen — `generate_object_transform` must accept `rule_name: &str` and `branch_index: Option<usize>`. Trace through call chain. Regenerate all 5+ generated parsers. Update integration contracts (regex, sv, return_annotation, semantic_annotation). Update mdBooks. Update regression-lock tests in embedding_api.rs (test framework needs to handle extra `_meta` key — likely already ⊇-style for `expected_json_object_keys_present`).

**Phase 2**: Tier 2 fields. Plumb source-text into the codegen path. Compute line-table once per parse.

**Phase 3**: Tier 3 fields. Trivia attribution pass.

**Phase 4**: Tier 4. Optional.

## Schema versioning

Per user: keep schema at `1`. Treat as additive (new `_meta` sibling key on existing typed objects; existing fields unchanged).

## Implementation entry points

- Codegen: `rust/src/ast_pipeline/ast_return_transform.rs` — `generate_object_transform`, `generate_value_extraction`, `generate_positional_ref` and related.
- ParseNode definition: `rust/src/ast_pipeline/mod.rs:607` (`pub span: std::ops::Range<usize>`, `pub rule_name: &'static str`).
- Per-rule emit site (example): `generated/regex_parser.rs:996` shows where `ParseContent::Json(serde_json::Value::Object(__pgen_obj))` is constructed inside a per-rule method that has `start_pos` and `parser.position` in scope.
- Integration contract: `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`, `PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`.
- mdBook chapter to update: `docs/regex_parser_book/src/parse-content-variants.md` (and SV equivalent) — document the `_meta` field structure.

## Pending question

What `meta_level` should be the default for new builds? `Basic` is the natural default but flips behavior for existing consumers reading the dump output. Consider a contract-version transition where `meta_level: Basic` becomes default at a specific parser-release version.

## Status

Plan documented; implementation deferred to a fresh-context session. SV slicing campaign should pause typing more rules until `_meta` is in place — every typed annotation added now will need to be re-verified after `_meta` lands (since the test framework's `expected_json_object_keys_present` checks may need updating).
