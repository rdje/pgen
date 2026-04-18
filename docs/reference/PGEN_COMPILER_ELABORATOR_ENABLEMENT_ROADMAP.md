# PGEN Compiler And Elaborator Enablement Roadmap (Living)

Last updated: 2026-04-18

## Mission

Make PGEN not only a parser generator, but a front-end workbench that materially accelerates compiler and elaborator creation.

The target is not "auto-generate the whole compiler." The target is:

- trustworthy grammar-driven parsing,
- lossless and shaped front-end products,
- provenance-rich semantic seeds,
- generated helpers and stable APIs that downstream passes can build on,
- and clean handoff into name binding, typing, lowering, elaboration, and diagnostics.

## North-Star Doctrine

Everything PGEN can do to make linter, compiler and elaborator creation dramatically easier, it should do, without breaking its principles.

That means:

- maximize downstream leverage,
- preserve platform discipline.

In practice:

- PGEN should aggressively own the repetitive, structural, cross-language front-end work,
- but it should not collapse into ad hoc language-specific magic,
- and it should not violate its EBNF-first, proof-first, contract-first doctrine.

This is intentionally a strong ambition, not a narrow one. The point is not to keep PGEN minimally scoped. The point is to make it maximally useful in the right layer.

## Why This Matters

The potential consumer base for PGEN widens materially if it helps not only parser authors, but also:

- compiler teams,
- elaborator and synthesis-front-end teams,
- IDE and language-server authors,
- static-analysis and refactoring tools,
- translation and migration pipelines,
- and any downstream system that needs a real front-end rather than only a recognizer.

The central idea is simple:

- far more people need a front-end substrate than need a raw parser generator,
- so every reusable front-end capability PGEN can generate or standardize broadens adoption potential.

## Core Boundary

PGEN should own front-end structure, provenance, and semantic seeding.

It should not pretend to auto-generate all later semantic passes.

### What PGEN should own

- lossless CST production when the language benefits from it,
- shaped AST production,
- source spans and source-map fidelity,
- trivia ownership and token preservation where diagnostics or rewrites need it,
- semantic seed emission from grammar-local knowledge,
- profile, dialect, and version gating,
- stable front-end export APIs,
- generated visitors, walkers, and query helpers,
- deterministic recovery and diagnostic anchoring,
- rewrite and unparse support where the language family benefits from it,
- handoff scaffolding into later IR or semantic-model layers.

### What downstream compiler and elaborator passes should still own

- whole-program symbol binding,
- full type propagation and checking,
- data-flow and control-flow analysis,
- optimization,
- scheduling,
- backend lowering,
- final elaboration semantics,
- code generation,
- project-wide dependency policy and build orchestration.

In short:

- PGEN should generate or standardize the front-end substrate,
- downstream tools should still own the deeper language-specific semantics and back-end logic.

That is the right boundary.

Operational reading of that boundary:

- PGEN should do everything it reasonably can to make compiler and elaborator writing easier,
- but it should do so by strengthening reusable front-end infrastructure,
- not by pretending to solve every language's deepest semantics generically.

## Current PGEN Assessment

PGEN already has several ingredients that point in the right direction:

- EBNF-backed parser generation,
- return annotations for AST shaping,
- semantic annotations for steering and semantic seed beginnings,
- parser-family contracts and proof gates,
- deterministic stimuli and coverage infrastructure,
- embedding API surfaces,
- and active HDL work that already pressures the project toward compiler and elaborator needs.

What is still missing is not raw parsing power. It is a stronger front-end product shape.

The biggest opportunity is to make PGEN emit more than:

- "parse success"
- plus "some tree"

and instead emit:

- lossless CST where needed,
- shaped AST,
- semantic bundles,
- stable node ids,
- source and trivia provenance,
- generated traversal helpers,
- and explicit handoff artifacts for later passes.

## High-Value Front-End Capabilities

### 1. Lossless CST plus shaped AST plus semantic seeds

For compiler and elaborator creation, the most valuable baseline is:

- lossless CST or token stream,
- shaped AST for normal downstream use,
- semantic seeds emitted during parsing or immediate post-parse normalization.

This gives downstream tools:

- exact source fidelity when they need it,
- ergonomic ASTs when they do not,
- and a seed layer for later attribution.

### 2. Provenance-rich source fidelity

PGEN should preserve:

- source spans,
- delimiter ownership,
- trivia ownership,
- source-map information where preprocessing or rewriting matters,
- and diagnostic-anchor metadata.

Without that, compiler and elaborator users quickly end up rebuilding infrastructure around the parser instead of using it directly.

### 3. Generated traversal and query helpers

PGEN can save downstream teams a huge amount of routine work by generating:

- visitors,
- walkers,
- typed node APIs,
- query helpers,
- child iteration helpers,
- and ergonomic pattern-matching utilities over generated AST shapes.

This is not glamorous work, but it is exactly the kind of workbench support that makes front-end adoption real.

### 4. Stable node identity

Compilers and elaborators benefit a lot from stable node ids for:

- cross-pass linking,
- diagnostics,
- side tables,
- incremental recomputation later,
- and binding/type/elaboration maps that should not depend on pointer identity.

PGEN should eventually make stable node identity a first-class surface where appropriate.

### 5. Deterministic recovery and diagnostics

Compiler and elaborator creation becomes much easier when the generated front-end already supports:

- bounded recovery,
- deterministic diagnostic anchoring,
- rule or production context in failures,
- and machine-readable diagnostics.

That is useful for both batch tools and interactive tooling.

### 6. Profile, dialect, and version gating

Compiler-grade and elaborator-grade consumers need stronger mode discipline than "accept whatever the grammar can swallow."

PGEN should make it easy to express:

- language version,
- dialect,
- synthesis subset,
- signoff subset,
- policy mode,
- and deprecation or feature gates.

That discipline helps compilers, elaborators, and linters alike.

## What Helps Elaborator Creation Most

Elaborators need a narrower but deeper front-end product than many compilers do.

The highest-value PGEN-side accelerators for elaborators are:

- constant-expression parsing and evaluation hooks,
- parameter and generic substitution scaffolding,
- scope and symbol-table seeding,
- import, package, library, and unit-visibility seeds,
- generated dependency graphs between design units,
- first-pass instance and connectivity facts,
- and canonical IR handoff points for later elaboration passes.

### Elaborator-specific examples

- parse-time and immediate post-parse constant-expression capture,
- typed parameter or generic defaults and overrides,
- declaration and instance seeds,
- first-pass child binding shapes,
- object and design-unit visibility seeds,
- and explicit "needs later elaboration" placeholders instead of ad hoc holes.

The purpose is not to solve elaboration in the grammar. The purpose is to make later elaboration deterministic and much less repetitive to build.

## What Helps Compiler Creation Most

For compilers, the best PGEN-side accelerators are:

- semantic-bundle export,
- stable node ids,
- generated HIR or lowering skeleton handoff points,
- rewrite and unparse support,
- and eventually incremental parse or changed-region support.

### Compiler-specific examples

- typed semantic bundles ready for name-binding or type-checking passes,
- grammar-derived HIR entry scaffolding,
- AST-to-HIR lowering helpers,
- rewrite-safe formatting or roundtrip support,
- fix-it and refactoring anchors,
- and deterministic diagnostic-source mapping.

Again, the point is not to auto-generate the compiler. The point is to remove a large amount of front-end boilerplate and make the later passes start from stronger ground.

## Semantic Seeds As Shared Infrastructure

The linter roadmap and this roadmap are siblings, not duplicates.

- The linter roadmap focuses on semantic seeds as a rule-engine and diagnostics substrate.
- This roadmap focuses on the broader front-end workbench needed by compiler and elaborator builders.

Shared infrastructure between the two includes:

- semantic bundle schema,
- provenance-rich fact and event records,
- stable source spans and node ids,
- profile and dialect gating,
- and parser-to-attribution handoff contracts.

The distinction is mostly about the downstream consumer:

- linter work emphasizes rules, waivers, and diagnostics,
- compiler and elaborator work emphasizes binding, typing, lowering, elaboration, and IR handoff.

## Recommended API Shape

PGEN should eventually expose a richer front-end bundle surface, not only parser-specific entrypoints.

Target output types should evolve toward shapes like:

- `ParseBundle`
- `SemanticBundle`
- `SourceMapBundle`
- `AstNodeId`
- `CstNodeId`
- `DiagnosticBundle`

### Parse bundle expectations

A useful `ParseBundle` should be able to carry:

- CST or token-stream fidelity when enabled,
- shaped AST,
- stable node ids,
- diagnostics,
- profile or dialect metadata,
- source-map metadata where relevant,
- semantic bundle linkage or inclusion.

### Export expectations

CLI and embedding surfaces should eventually support deterministic sidecars such as:

- `--emit-ast-json`
- `--emit-cst-json`
- `--emit-semantic-bundle-json`
- `--emit-source-map-json`
- `--emit-diagnostics-json`

These exports matter for:

- downstream CI,
- reduction and triage,
- corpus and fixture management,
- and transparent contract-driven integration.

## Validation And Proof Expectations

This lane should be held to the same proof-first doctrine as the rest of PGEN.

Useful future proof surfaces include:

- AST shape contract gates,
- CST or token-fidelity contract gates where relevant,
- semantic bundle contract gates,
- source-map contract gates,
- generated traversal-helper contract gates,
- elaborator-handoff contract gates,
- compiler-handoff contract gates,
- and deterministic export parity gates.

Potential gate names:

- `frontend_bundle_contract_gate`
- `semantic_bundle_contract_gate`
- `source_map_contract_gate`
- `compiler_handoff_contract_gate`
- `elaborator_handoff_contract_gate`

These are roadmap placeholders, not yet-landed gates.

## Milestone Order

### Milestone 1: Freeze the front-end workbench doctrine

- publish the "PGEN as front-end workbench" boundary clearly,
- state what PGEN should and should not own,
- align this roadmap with the linter-enablement roadmap.

### Milestone 2: Strengthen front-end product shape

- standardize shaped AST export,
- define optional CST or token-fidelity surfaces where needed,
- stabilize source spans, node ids, and diagnostic anchoring expectations.

### Milestone 3: Generate more front-end helper infrastructure

- visitors,
- walkers,
- typed query helpers,
- child iteration helpers,
- and node-id aware helper APIs.

### Milestone 4: Land semantic bundle export as shared infrastructure

- reuse the semantic-seed work from the linter lane,
- make it available as a compiler and elaborator substrate too,
- keep the export stable and versioned.

### Milestone 5: Add elaborator-oriented scaffolding

- constant-expression capture and evaluation hooks,
- substitution scaffolding,
- design-unit dependency graphs,
- first-pass instance and connectivity seeds,
- explicit IR handoff points for later elaboration.

### Milestone 6: Add compiler-oriented scaffolding

- HIR or lowering skeleton handoff points,
- stable node-id aware mapping helpers,
- rewrite and unparse support,
- groundwork for later incremental front-end support if justified.

### Milestone 7: Prove the workbench on real downstream slices

- at least one HDL elaboration-oriented slice,
- at least one broader compiler-oriented or transformation-oriented slice,
- and explicit machine-checkable handoff contracts instead of narrative confidence.

## Recommended Near-Term Implementation Order

If this lane starts now, the most sensible first sequence is:

1. Publish this roadmap and wire it into the maintained docs.
2. Freeze the "front-end workbench" doctrine and boundary.
3. Reuse the semantic-bundle work from the linter lane as shared infrastructure.
4. Add provenance-rich front-end bundle types and stable node ids.
5. Generate visitors, walkers, and typed query helpers.
6. Add the first elaborator-handoff contract on an HDL slice.

That order makes the broader compiler and elaborator lane build on shared platform improvements rather than splitting into isolated one-off features.

## Explicit Answer To The Core Question

What else can PGEN do to hugely facilitate compiler and elaborator creation?

A lot, if it stays disciplined about what it should own.

The biggest opportunity is to make PGEN not just a parser generator, but a front-end generator and workbench.

For compiler and elaborator creation, the highest-value additions are:

- lossless CST plus shaped AST plus semantic-seed emission,
- provenance-rich spans, trivia ownership, and source maps,
- stable node ids,
- generated visitors, walkers, and typed node APIs,
- deterministic recovery and diagnostics,
- profile, dialect, and version gating,
- semantic-bundle export,
- constant-expression and substitution scaffolding,
- dependency and connectivity seeds,
- and canonical handoff points into later semantic or IR passes.

My take is:

- the more PGEN can own front-end structure, provenance, semantic seeds, and deterministic export,
- the more it becomes a serious compiler and elaborator accelerator,
- without pretending to auto-generate the whole compiler.

That is the right boundary, and it should materially broaden the potential PGEN consumer base.
