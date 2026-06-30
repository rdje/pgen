# Glossary

- **Semantic annotation** — the `@name: value` directive on a grammar rule that steers the parser's
  *behavior and state* (vs. a return annotation `-> …`, which shapes the rule's returned value). The
  semantic-annotation parser is the parser for this `@…` language.
- **Steering directive** — a semantic annotation PGEN's AST pipeline *interprets* (`@predicate`,
  `@emit_fact`, `@profiles`, `@transform`, …); see [Steering Directives](steering-directives.md).
- **Value language** — the literal / structured / expression / reference language that appears after the
  colon in `@name: value`; what the `semantic_annotation` parser literally parses
  (`grammars/semantic_annotation.ebnf`, version `2.0`).
- **Predefined vs. custom name** — `annotation_name` is either a recognized standard name (resolving to
  the bare name) or any other identifier (resolving to `{type: "custom_annotation", name}`).
- **Semantic store** — the parser's memory: facts organized into scopes, written by `@emit_fact` /
  `@open_scope`, queried by `@predicate`, made cross-unit by `@export_to_library` /
  `@import_from_library`, and transactional so backtracking rolls back. See
  [The Semantic Store](semantic-store.md).
- **Fact / fact kind** — a named, attributed datum in the store; its schema is declared with
  `@fact_kind:` (DECLARE) and written with `@emit_fact` (EMIT).
- **Predicate** — a store query (`has_fact`, `lacks_fact`, `fact_attribute_equals`,
  `lacks_fact_attribute_equals`, `fact_count_at_least`, `len_bounds`, `numeric_bounds`) used by
  `@predicate` to gate a rule/branch; `@predicate_def:` names a reusable composed predicate.
- **Profile** — a named grammar edition; `@profiles` restricts a rule to one or more of them (e.g.
  SystemVerilog `sv_2017` / `sv_2023`).
- **Rule reference (`$…`)** — a dotted + non-negative-integer-indexed, depth-unbounded reference to a
  captured sub-result inside a directive payload (a subset of JSONPath).
- **Bootstrap classification** — the bootstrap backend's `TransformExpr → Structured → Raw` triage of a
  payload (it never hard-fails; unknown ⇒ `Raw`).
- **Compiled rule views** — how a rule's directives lower into the generated parser: *pre predicates*,
  *post predicates*, *branch predicates*, and *effect directives*.
- **SC-01…SC-13** — the thirteen semantic-steering control slices (transform, sample hints, name
  routing, token steering, precedence/associativity, branch policy, error recovery, value-domain,
  relational, coverage, negative-case, determinism, profile gating), each a Tier-4 gate-enforced
  contract.
- **Bootstrap backend** — the hand-written, permissive parser (documented by
  `grammars/builtin_semantic_annotation.ebnf`); always part of the contract.
- **Generated backend** — the generated parser (`generated/semantic_annotation_parser.rs`, from
  `grammars/semantic_annotation.ebnf`); part of the contract when available. Bootstrap↔generated drift
  is a contract failure.
- **Annotation family (`AnnotationFamily::Semantic`)** — the embedding-API selector for the
  semantic-annotation language; its sibling is `AnnotationFamily::Return`.
