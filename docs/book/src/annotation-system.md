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

## Phase 2: Inline Annotation Application (planned, in progress)

Last updated: 2026-04-26.

### Why this phase exists

The PGEN annotation design intent has always been that return + semantic annotations apply **inline during parse** — the generated `parse_X` methods directly emit shaped output as they parse, with no separate post-parse transform step. The current implementation has drifted from that intent: parsers emit a generic `ParseNode` tree, and shaped output is built by a post-parse transform (`UnifiedReturnAST::parse_generated_return_annotation` walking the tree). The drift was surfaced during the PGEN-RGX-0073 perf investigation, which measured roughly 12,500ns per AST node in the regex parser — a cost class consistent with allocating a generic node, populating its `rule_name` / `content` / `span` fields, and then re-walking the tree post-parse to extract the shape downstream consumers actually want.

Phase 2 restores the original design contract. Restoring it removes a class of allocations from the per-parse hot path, shrinks the shape downstream consumers see, and removes the two-phase indirection that has been a source of confusion and drift.

### Decomposition

Phase 2 is decomposed into ~8 milestones. Each milestone is independently committable with: designed contract, implementation, preserved tests, perf measurement, sign-off before next.

- **M0** — Plan agreed (this section).
- **M1** — Parallel emit infrastructure: `--inline-annotations` flag in `ast_pipeline`, off by default. When on, the generator emits a second set of `parse_X_typed` methods alongside existing `parse_X`. Default behavior unchanged. Output type: `serde_json::Value` (matches the AST-dump path consumers already see; future newtype wrap is a follow-up only).
- **M2** — Differential validation: run both paths on `return_annotation` grammar's existing test corpus + semantic corpus; assert byte-identical output. Proves the inline emit is correct against the post-parse oracle before any other grammar migrates.
- **M3** — Migrate `regex` grammar to inline path. Add `parse_full_regex_typed()` to the embedding API. Validate against legacy on the regex_parser_integration_contract corpus (93 success + failure samples).
- **M4** — Perf measurement: `regex_perf_probe` runs both paths. Confirm inline is faster than legacy + post-parse, with the size of the difference reported honestly.
- **M5** — RGX integration: get RGX to consume `parse_full_regex_typed`. Validate end-to-end across the RGX-side test surface. PGEN-RGX-0073 closure depends on this milestone landing.
- **M6** — Migrate remaining grammars (`semantic_annotation`, `return_annotation`, `systemverilog`, `vhdl`, `rtl_const_expr`, `rtl_frontend`). One per commit, in that order. Each one runs differential validation against its post-parse oracle.
- **M7** — Retire post-parse transform. Remove `UnifiedReturnAST::parse_generated_return_annotation` walkers and friends. Default `--inline-annotations` to ON; remove the flag itself in a subsequent commit.
- **M8** — Cleanup: shrink the `Annotations` runtime struct to only what the stimuli/semantic-runtime paths still need. Generated parser size shrinks. AST-dump JSON either retired or kept as an explicitly opt-in path.

### Output type decision

`parse_X_typed` returns `ParseResult<serde_json::Value>`. Rationale:

- Matches the shape RGX and other downstream consumers already see via the existing AST-dump path.
- Avoids a parser-family explosion of newtypes (one shape per grammar).
- A future wrap-in-newtype refactor is non-breaking on the JSON wire format.

A typed-enum alternative was considered and deferred. It would be more type-safe but adds API surface and forces RGX-side changes that are not justified by the perf or correctness goals of this phase.

### What this phase is not

- Not a one-shot atomic refactor. Each milestone has an explicit rollback granularity.
- Not a public API change at the `parse_full_*` legacy entry points until M5+ explicitly opts in. M1-M3 are additive only.
- Not a substitute for the parser-agnostic micro-optims (Optims #2-#7 already landed). Phase 2 attacks a different cost class — AST shape and indirection — and those gains compose.

### Risks

| Milestone | Top risk | Mitigation |
|---|---|---|
| M1 | Codegen complexity — porting `UnifiedReturnAST::generate_code` logic into the parser-emit template. | M1 emits a wrapper around the legacy path; no shape logic ports. M2 is when the actual port happens. |
| M2 | Differential mismatch reveals undocumented post-parse-only behavior. | The post-parse transform is the truth oracle; either the inline emit matches it or the inline emit is wrong. No mismatch is acceptable. |
| M3 | Regex grammar's 582 generated rule constructions all regenerate. | Tests at the regex_parser_integration_contract surface (93 success + failure samples) are the gate. |
| M5 | RGX consumer code reads our output. | Cross-repo coordination; opt-in API surface, RGX migrates on its own schedule. |
| M7 | Removing the safety net. | Only after every grammar has migrated, every test has been migrated, and every consumer has been migrated. |

### Commit cadence

Each milestone produces at least one focused commit. Continuity-doc updates are mandatory per the project's `COMMIT.md` workflow. A milestone is not declared complete until its commit lands and continuity docs are synchronized.

## Primary Source Docs

- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`
- `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`
- `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`
- `docs/RETURN_ANNOTATIONS_REFERENCE.md`
- `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `README.md`
