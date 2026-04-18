# PGEN Linter Enablement Roadmap (Living)

Last updated: 2026-04-18

## Mission

Turn PGEN's annotation-capable EBNF pipeline into a trustworthy front-end substrate for serious linters, starting with HDL signoff-oriented consumers and generalizing cleanly to any language for which PGEN has an EBNF plus semantic intent annotations.

The target is not "a parser that a linter might be able to reuse somehow." The target is:

- grammar-driven parsing,
- grammar-driven semantic seed emission,
- provenance-carrying semantic facts and events,
- stable export and embedding APIs,
- and a disciplined handoff into later attribution and rule engines.

## Why HDL First, But Not HDL Only

SystemVerilog and VHDL are the first obvious proving ground because:

- PGEN already has live HDL grammars,
- HDL lint/signoff pressure is high,
- grammar-local classification matters a lot,
- and syntax-only parsing is not enough for downstream tool quality.

But the required platform work is intentionally broader than HDL. If this lane is done correctly, the same infrastructure should help any future linter built on a PGEN grammar:

- typed semantic seeds,
- scope-aware fact emission,
- stable source provenance,
- version/dialect gates,
- pragma and waiver capture,
- and deterministic semantic event export.

HDL is the flagship stress test, not the only beneficiary.

## Core Doctrine

The grammar should emit local semantic seeds. It should not try to solve full-program meaning.

What belongs in the grammar and annotation layer:

- lossless syntax recognition,
- AST shaping,
- local construct classification,
- source-span ownership,
- scope-open and scope-close points,
- declaration and construct seeds,
- inline pragma and waiver capture,
- profile and dialect gating,
- bounded semantic predicates that consult already-emitted local facts.

What belongs after parsing:

- global symbol binding,
- type or subtype propagation across files,
- parameter or generic substitution,
- driver and read/write reasoning,
- control-flow or data-flow analysis,
- elaboration decisions,
- project-level library or package resolution,
- signoff rule evaluation.

In short:

- grammar and annotations provide structure, locality, and semantic seeds,
- attribution provides meaning,
- the rule layer consumes attributed meaning rather than trying to infer everything from raw parse trees.

## Current PGEN Assessment

PGEN already has the right basic shape for this lane.

What already exists:

- return annotations for AST shaping,
- semantic annotations with typed lowering support,
- structured semantic payloads in `UnifiedSemanticAST`,
- profile-aware parsing via `@profiles`,
- scope and fact directives via:
  - `@open_scope`
  - `@close_scope`
  - `@emit_fact`
  - `@predicate`
- validator and runtime infrastructure for typed directive handling,
- preserved rule-level and branch-level semantic annotation paths,
- generated-parser and stimuli infrastructure that can validate semantic behavior under machine-checkable gates.

What is still missing for linter-grade use:

- a disciplined semantic-seed schema,
- provenance-rich fact and event records,
- stable semantic export APIs,
- first-class execution of mid-sequence semantic actions,
- schema-aware validation beyond generic object payload acceptance,
- and a documented handoff contract from parser-layer seeds to later attribution passes.

Conclusion:

- semantic annotations are already general enough structurally,
- but they are not yet linter-complete operationally.

The next step is not "make annotations more generic." The next step is "make them more disciplined, typed, and exportable."

## Directive Strategy

The first implementation waves should prefer existing semantic directives, not invent a brand-new top-level annotation language.

### Directives to standardize first

- `@profiles`
  - use for language revision, dialect, signoff subset, and policy mode gating.
- `@open_scope`
  - use to declare grammar-local scope transitions.
- `@close_scope`
  - use to close those same scopes deterministically.
- `@emit_fact`
  - use for declaration, construct, pragma, waiver, and policy-seed emission.
- `@predicate`
  - use for bounded semantic queries against already-collected local state.

### Existing semantic directives that also help the linter lane indirectly

- `@requires`
- `@implies`
- `@constraint`
- `@invalid_case`
- `@negative`
- `@coverage_target`
- `@critical_path`

Those are not the linter runtime itself, but they are useful for linter-oriented grammar validation, stimuli steering, and negative corpus construction.

### Candidate additions only if the existing directives become awkward

Do not add these on day one. Add them only if `@emit_fact` and `@predicate` become measurably too awkward or too lossy.

- `@emit_ref`
  - for explicit reference-site emission when treating every read/write/reference as a fact becomes too opaque.
- `@diagnostic_anchor`
  - for stable user-facing diagnostic anchoring when the best source span is not the same as the declaration/reference span.
- `@waiver`
  - for first-class inline waiver payloads if generic pragma facts become too ad hoc.

Preferred rule:

- use existing directives first,
- standardize payload schema second,
- add new directive spellings only after the schema shows a real ergonomic or observability gap.

## Semantic Seed Schema

PGEN needs one shared semantic seed schema with language-specific overlays.

### Shared record shape

Every emitted semantic fact or event should eventually carry:

- `schema_version`
- `language`
- `profile`
- `kind`
- `name`
- `attributes`
- `scope_path`
- `origin`

### Required origin data

Every fact or event should carry stable provenance:

- grammar name,
- rule name,
- branch identity when applicable,
- capture reference or annotation-local reference when applicable,
- source span,
- node path or node id,
- and whether the event came from rule-level, branch-level, or mid-sequence annotation execution.

Without that origin data, a linter consumer cannot explain diagnostics convincingly and PGEN cannot debug semantic drift safely.

### Core cross-language fact kinds

- `scope`
- `decl`
- `ref`
- `type`
- `assignment`
- `process`
- `construct`
- `pragma`
- `waiver`
- `policy_marker`
- `diagnostic_anchor`

### Suggested core attributes

- `scope_kind`
- `decl_kind`
- `object_kind`
- `type_kind`
- `port_dir`
- `lifetime`
- `signing`
- `packedness`
- `assignment_kind`
- `procedural_kind`
- `edge_kind`
- `concurrency_kind`
- `version_gate`
- `dialect`
- `synthesis_hint`
- `waiver_rule`
- `waiver_scope`
- `source_role`

### HDL overlays

SystemVerilog and VHDL should share the cross-language core above, then add HDL-specific attributes only where needed:

- SystemVerilog-biased examples:
  - `always_kind`
  - `net_vs_var`
  - `packed_dims`
  - `unpacked_dims`
  - `modport_role`
  - `interface_role`
- VHDL-biased examples:
  - `design_unit_kind`
  - `subtype_indication`
  - `resolvedness_hint`
  - `sequential_vs_concurrent`
  - `library_kind`

Preferred schema rule:

- keep the shared vocabulary large enough to be genuinely reusable,
- keep language-specific overlays small and explicit rather than smuggling dialect-specific meaning into vague generic keys.

## Recommended API Shape

PGEN should expose semantic seeds as a first-class deliverable, not as an internal side effect.

### Rust embedding surface

Target output types should look like:

- `SemanticBundle`
- `SemanticFactRecord`
- `SemanticEventRecord`
- `SemanticScopeRecord`
- `SemanticOrigin`

At a high level:

- `SemanticBundle` groups facts, events, scopes, diagnostics, schema version, grammar id, and profile id.
- `SemanticFactRecord` is the stable emitted seed surface.
- `SemanticEventRecord` captures scope-open, scope-close, fact-emission, and predicate-evaluation flow where reproducibility matters.
- `SemanticOrigin` carries the provenance required for diagnostics and debugging.

### Parser-facing API expectations

Generated parsers and embedding surfaces should eventually support stable accessors such as:

- `semantic_bundle()`
- `semantic_facts()`
- `semantic_events()`
- `take_semantic_bundle()`

If parse performance or memory requires it, event capture should be configurable. But the API shape should stay versioned and explicit rather than ad hoc.

### CLI/export expectations

PGEN should also expose deterministic machine-readable sidecars, for example:

- `--emit-semantic-bundle-json`
- `--emit-semantic-facts-json`
- `--emit-semantic-events-json`

Those sidecars matter for:

- contract gates,
- corpus minimization,
- downstream CI,
- linter debugging,
- and user-facing transparency.

### Mid-sequence execution rule

Branch-level and mid-sequence semantic annotations should eventually be executable first-class semantic events, not only preserved metadata. For linter enablement, "preserved but never executed" is not enough.

## Example Annotation Shapes

The goal is not to invent a new annotation syntax. The goal is to make better disciplined use of the existing one.

Illustrative shapes:

```text
@profiles: ["sv_2017", "sv_2023", "signoff"]
@open_scope: { kind: "module", name: $2 }
@emit_fact: {
  kind: "decl",
  name: $port_name,
  decl_kind: "port",
  object_kind: "net",
  port_dir: "input",
  signing: "signed"
}
@emit_fact: {
  kind: "process",
  name: "always_ff",
  procedural_kind: "always_ff"
}
@emit_fact: {
  kind: "pragma",
  name: "lint_waive",
  waiver_rule: "unused_decl"
}
@close_scope: { kind: "module", name: $2 }
```

The important part is not the exact spelling of those example keys. The important part is that:

- the payloads are typed,
- the emitted meaning is local and bounded,
- and later passes can rely on a stable schema instead of parsing arbitrary prose-like blobs.

## Validation and Proof Expectations

This lane should follow the same machine-checkable discipline as the rest of PGEN.

Target proof surfaces to add over time:

- semantic-seed schema validator coverage,
- provenance contract suites,
- generated-parser semantic bundle export contracts,
- HDL pilot corpora with golden semantic bundles,
- linter-focused negative and near-valid corpora,
- cross-language replay over more than one grammar family,
- deterministic semantic event parity under regeneration.

Potential gate names:

- `semantic_seed_schema_gate`
- `semantic_event_provenance_gate`
- `hdl_linter_seed_contract_gate`
- `cross_language_linter_platform_gate`

These names are roadmap placeholders, not yet-landed surfaces.

## Milestone Order

### Milestone 1: Freeze doctrine and schema

- publish the grammar-vs-attribution boundary clearly,
- freeze the first shared semantic-seed vocabulary,
- document the HDL overlay vocabulary,
- state what is intentionally out of scope for grammar annotations.

### Milestone 2: Add provenance-rich semantic record types

- give `@emit_fact`, `@open_scope`, and `@close_scope` stable origin metadata,
- carry source spans and node ids or node paths,
- make facts/events exportable in a stable bundle.

### Milestone 3: Execute branch and mid-sequence semantic actions

- promote preserved branch and mid-sequence annotations into runtime semantics where safe,
- ensure event ordering is deterministic,
- cover rule-level, branch-level, and mid-sequence provenance explicitly.

### Milestone 4: Land HDL pilot semantic seeds

- SystemVerilog pilot:
  - declarations,
  - scopes,
  - process kinds,
  - assignment kinds,
  - pragma and waiver capture.
- VHDL pilot:
  - design units,
  - declarations,
  - process vs concurrent constructs,
  - pragma and waiver capture.

### Milestone 5: Publish stable export APIs

- embedding API support for semantic bundles,
- CLI sidecars for semantic bundles,
- versioned schema discipline,
- compatibility policy for field additions and meaning changes.

### Milestone 6: Define attribution handoff contract

- document what later attribution is expected to compute,
- define the query model that downstream rule engines can rely on,
- keep the parser-layer seed contract separate from the later semantic-model contract.

### Milestone 7: Add linter-oriented proof corpora

- golden semantic bundle tests,
- negative and near-valid diagnostics corpora,
- waiver and pragma regression suites,
- profile-aware acceptance and rejection suites.

### Milestone 8: Generalize beyond HDL

- prove the same seed/export/provenance surface on at least one non-HDL grammar family,
- treat HDL as the first flagship user, not the end of the story.

## Recommended Near-Term Implementation Order

If this lane starts now, the best first sequence is:

1. Publish this roadmap and wire it into the main roadmap, book, and continuity docs.
2. Freeze the first semantic-seed schema using existing directives, not new syntax.
3. Add provenance-bearing `SemanticFactRecord` and `SemanticEventRecord` types.
4. Make mid-sequence semantic annotations executable and observable.
5. Pilot declaration and scope seeds on constrained SystemVerilog and VHDL subsets.
6. Add semantic bundle JSON export plus embedding API access.

That ordering gives PGEN a reusable platform asset early instead of tying the whole effort to one linter implementation.

## Explicit Answer To The Core Question

Yes.

If PGEN adds the right linter-enablement features for an HDL signoff linter, and does so in a disciplined, schema-first, provenance-rich way, those same features should materially benefit any linter built on any PGEN-backed grammar.

The right abstraction is:

- not "special HDL support,"
- but "cross-language semantic-seed and semantic-export infrastructure,"

with HDL as the first serious proving ground.
