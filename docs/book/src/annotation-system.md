# Annotation System

Annotations are one of the defining differences between PGEN and a simpler grammar-to-parser tool.

## Two Annotation Families

### Return annotations

Return annotations shape the AST that generated parsers return. They are the normative way to control parse-result structure instead of treating the generated tree as fixed.

### Semantic annotations

Semantic annotations steer parser-generation behavior and related transformation/runtime choices in the Rust AST pipeline.

They also now have a stricter same-line scanner contract. Inline rule-body annotations consume only their own payload:

- quoted payloads,
- balanced structured payloads such as `{...}`, `[...]`, or `(...)`,
- or a scalar token payload.

They do not get to swallow the rest of the rule body. That matters because branch-local hints like `@sample: "..." alpha | beta` are only useful if `alpha | beta` still survives as real branch syntax after tokenization.

That steering now includes more than regex-target tweaks. Literalish directives such as `@sample`, `@literal`, `@example`, and legacy `@stimulus` can now be used as parser-proven stimuli seeds for:

- regex atoms,
- non-regex non-OR rule expansions,
- and inline branch-local OR alternatives.

PGEN also now has a narrower replay-only variant: `@probe_sample`.

- `@sample` is the ordinary always-on literalish steering tool.
- `@probe_sample` is for target-drive replay.
- `@probe_sample` only short-circuits when that rule is the active generation entry, so it can help probe broad dependency rules without collapsing ordinary top-level generation transitively.

That widened the annotation system from "token-shape nudges" into a real narrow branch-steering surface for coverage-guided replay, while still keeping the project rule that sample hints must be justified by parser-backed evidence rather than sprayed across a grammar blindly.

Together, these two annotation families make PGEN a parser platform rather than only a parser emitter.

## Semantic Seeds, Linters, And Front-End Workbenches

The next major widening for semantic annotations is not "more random annotation flexibility." It is a disciplined semantic-seed layer that downstream tools can trust.

The intended model is:

- the grammar emits local semantic seeds,
- the parser preserves source fidelity and provenance,
- later attribution passes compute broader meaning such as binding, typing, and flow,
- and downstream rule engines consume that attributed model rather than guessing from raw parse trees.

That matters first for HDL signoff-style consumers, but it is not an HDL-only idea. If PGEN lands the right semantic-seed, provenance, and export infrastructure, the same platform work should help:

- linters,
- compiler front-ends,
- elaborators,
- and other downstream semantic tools built on PGEN-backed grammars.

The detailed planning surfaces for those adjacent lanes now live in:

- `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`
- `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`

## Why They Matter

Without annotations, grammar-driven generation can still produce parsers. With annotations, PGEN can also control:

- AST shape,
- transformation behavior,
- steering metadata,
- downstream usability of generated parsers.

That is why annotation grammars are core platform surfaces, not optional extras.

## Bootstrap Reality

Annotations also sit at the center of one of PGEN's historic bootstrapping constraints.

Because annotation parsers are needed by the generation pipeline itself, PGEN carries bootstrap-safe annotation grammar contracts so those parsers can be generated without circular dependency on themselves.

This is why the docs distinguish between:

- bootstrap-safe built-in annotation grammars,
- full main annotation grammars,
- generated parser steady-state behavior.

## Proof Expectations

Annotation support is not considered real just because syntax exists. It is expected to have:

- validator coverage,
- shared/built-in suite coverage,
- round-trip or comparable contract evidence,
- maintained aggregate gates.

## Implicit `-> $1` default — what it does and what it doesn't

When a rule body is a **single Atom** (one terminal, one regex, one rule-reference) or a **single-element Sequence**, and the author has not declared a return annotation, the codegen synthesizes an implicit `-> $1` so the matched value flows through cleanly. This is what lets `boolean_literal := 'true' | 'false'` produce a clean string output without forcing per-branch `-> $1` everywhere.

The implicit default does **not** fire on:

- **Multi-element Sequence bodies** (e.g. `'(' expression ')'`). Picking which `$N` to surface would be an arbitrary author decision; require an explicit declaration.
- **Quantified bodies** (`X+`, `X*`, `X?`). The natural reading of `$1` here is "the whole capture group" — i.e. passthrough — and that is just what no-transform produces. The earlier (now-fixed) shape that synthesized `-> $1` on Quantified bodies emitted an `elements[0].content.clone()` extraction that silently dropped every match past the first; the regression is documented in the codegen-tightening tracker entry on 2026-04-30. Authors who want a different shape on a Quantified body declare it explicitly (`concatenation = piece+ -> [$1*]`).
- **Or-bodies as a whole.** Each Or branch is judged independently against the same rule (single Atom → implicit; single-element Sequence → implicit; multi-element Sequence → no synthetic; Quantified → no synthetic).

Synthetic defaults are codegen-only — they never appear in the inventory artifact, and the grammar-author-written annotations remain the visible declared surface.

## Phase 2: Eliminate Stringification Roundtrips In Return-Annotation Transforms (retargeted)

Last updated: 2026-04-26.

### Earlier framing was wrong

An earlier framing said return + semantic annotations had drifted to a "post-parse transform" applied by `UnifiedReturnAST::parse_generated_return_annotation` walking a generic `ParseNode` tree at parse time, and that Phase 2 had to "restore inline application." A direct read of the codebase shows that framing is wrong:

- Return annotations are already applied inline at runtime. [rust/src/ast_pipeline/ast_based_generator.rs](../../../rust/src/ast_pipeline/ast_based_generator.rs) emits `result = #transform;` directly inside each rule's parse function whenever the rule carries a return annotation, with the transform tokens produced by [rust/src/ast_pipeline/ast_return_transform.rs](../../../rust/src/ast_pipeline/ast_return_transform.rs).
- `UnifiedReturnAST::parse_generated_return_annotation` is a build-time parser of annotation source text (e.g. `-> $1.foo`); it runs during PGEN code generation, not during downstream input parsing.
- Semantic annotations already use a typed structured carrier (`UnifiedSemanticValue` / `SemanticRuntimeValue`) — they do not carry an analogous problem.

The earlier framing also claimed PGEN-RGX-0073 closure depended on "moving annotations inline for the regex grammar." That claim was retracted after a direct check: [generated/regex_parser.rs](../../../generated/regex_parser.rs) currently has zero hits for `json_obj`, `serde_json::to_string`, or `serde_json::json!(`, and the `parse_regex` / `parse_piece` functions emit raw `ParseContent::Quantified(...)` / `ParseContent::Sequence(...)` with no `result = #transform` step. The regex grammar's two object-literal annotations (`-> {type: "regex", pattern: $1}` and `-> {type: "piece", atom: $1, quantifier: $2}`) are silently dropped at codegen for that grammar today. Whatever the dominant cost in the regex parser is, it is not a stringification roundtrip — that code is not present in the regex parser at all.

### What is actually wrong

The real defect is in how return-annotation object literals and property/array access are carried at runtime in the generated parsers that *do* emit transforms (notably [generated/return_annotation_parser.rs](../../../generated/return_annotation_parser.rs)):

- [`generate_object_transform`](../../../rust/src/ast_pipeline/ast_return_transform.rs) builds a `serde_json::Value` then `serde_json::to_string`s it and wraps the resulting `String` in `ParseContent::TransformedTerminal(String)`. From that point on the shaped value lives as JSON-encoded text inside a string variant.
- [`generate_property_access`](../../../rust/src/ast_pipeline/ast_return_transform.rs) deserialises that string back into `serde_json::Value`, looks up a property, and re-stringifies before wrapping again. Each property access pays serialise → parse → serialise.
- [`generate_array_transform`](../../../rust/src/ast_pipeline/ast_return_transform.rs) builds a `ParseContent::Sequence(Vec<ParseNode>)` with synthetic `element_N` rule names and zero spans, which is a different shape than the JSON-string carrier and does not compose cleanly with property access.
- `parse_content_to_string` falls back to `format!("{:?}", other)` (Debug formatter) for any non-trivial `ParseContent`, so structured shapes degrade silently into Rust Debug strings rather than failing visibly.

That serialise/parse/serialise roundtrip — and the Debug-format fallback — is the "stringification nonsense" Phase 2 retargets to remove.

### What Phase 2 now does

Phase 2 introduces a typed structured carrier in `ParseContent` so return-annotation transforms operate on values, not on serialised strings.

1. Extend `ParseContent` with a typed structured variant (e.g. `Json(serde_json::Value)`).
2. `generate_object_transform` builds `serde_json::Value::Object(...)` directly and wraps it as the new variant. No `to_string`.
3. `generate_array_transform` builds `serde_json::Value::Array(...)` and wraps it as the new variant. The synthetic-`ParseNode` array shape is no longer needed for array literals (it remains for sequence captures the grammar already produces).
4. `generate_property_access` and `generate_array_access` operate in place on the typed value (`value.get(prop)` / `value.get(idx)`). No `from_str`, no `to_string`.
5. `parse_content_to_string` is rewritten to handle the typed variant explicitly, and the `format!("{:?}", other)` Debug fallback is removed in favour of structured handling.
6. `parse_full_<entry>_typed` (M1's seam) returns the typed value directly rather than `serde_json::to_value(&node)`-wrapping a string-encoded tree.

Semantic annotations are out of scope of this work — they already use a typed carrier and are not affected.

### What this phase is not

- It is not a "move post-parse to inline" phase. Inline application is already in place.
- It is not on the critical path for PGEN-RGX-0073 closure as currently understood. The regex parser does not carry the stringify roundtrip today, so removing it cannot directly speed regex parsing. It is, however, a precondition for any future typed-shape API on regex, because turning the dropped annotations on without first fixing the carrier would just import the stringify cost into regex.
- It is not a `serde_json::Value` lock-in for the public API. `parse_full_<entry>_typed` keeps its `ParseResult<serde_json::Value>` signature; only the internal carrier changes.

### Separate defect surfaced during this investigation

The regex grammar declares object-literal return annotations that never reach the generated regex parser. That is a silent codegen drop, separate from the stringification work, and it is not closed by Phase 2 alone. It is tracked as a follow-up to investigate after the typed carrier lands.

### Commit cadence

Phase 2 lands in two commits:

1. Documentation retarget — replaces the wrong "post-parse" framing across the book chapter, live tracker, and continuity docs. No code or test changes.
2. Code change — introduces the typed structured carrier, rewrites the affected codegen helpers, regenerates the affected tracked parsers, and adds a focused differential test that asserts byte-identical wire-JSON output for the existing return + semantic annotation contract corpora before and after the change.

The earlier M1 commit (`4450b93`) remains useful: the typed-entry-skeleton flag (originally `--inline-annotations`, renamed in slice 4 to `--emit-typed-entry-skeleton` for honesty about its scope) and the `parse_full_<entry>_typed` skeleton are the right seam for surfacing the typed value through the public API; only the Phase 2 narrative attached to that commit was wrong.

## Primary Source Docs

- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
- `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`
- `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`
- `docs/RETURN_ANNOTATIONS_REFERENCE.md`
- `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `README.md`
