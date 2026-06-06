---
id: ast-pipeline-architecture
title: The AST pipeline end-to-end — stages, IR data structures, the two consumers, and the annotation consumption matrix
answers:
  - "how does the AST pipeline work end-to-end"
  - "what are the stages from .ebnf to generated parser / stimuli"
  - "what is ASTNode vs ParseContent (grammar IR vs runtime parse result)"
  - "what does the Annotations struct hold and how is it populated"
  - "which consumer (codegen vs stimuli generator) reads which annotation granularity"
  - "where does transform_from_raw_ast / extract_rule_annotations fit"
  - "is a position-specific (mid-sequence) annotation consumed by the stimuli generator"
tags: [ast-pipeline, architecture, ir, annotations, codegen, stimuli-generator, reference]
date: 2026-06-06
status: current
evidence: "code-verified — src/ast_pipeline/mod.rs (ASTNode enum :861, ParseContent enum :715, Annotations struct :1074, MidSequenceSemanticAnnotation :1066, transform_from_raw_ast :1355, extract_rule_annotations :2246); src/ebnf_frontend.rs (tokenizer); src/ast_pipeline/ast_based_generator.rs (codegen, mid-sequence at :190); src/ast_pipeline/stimuli_generator.rs (generator annotation reads ~:8150-8248). 5-step transform algorithm: docs/ast_transformation_pipeline.md; codegen layer: docs/AST_GENERATOR_ARCHITECTURE.md"
reverify: "`grep -nE 'pub enum ASTNode|pub enum ParseContent|pub struct Annotations|fn transform_from_raw_ast|fn extract_rule_annotations' rust/src/ast_pipeline/mod.rs`; `grep -nE 'semantic_annotations\\.get|branch_semantic_annotations\\.get|branch_mid_sequence' rust/src/ast_pipeline/stimuli_generator.rs`"
---

PGEN turns a `.ebnf` grammar into a parser (and stimuli) through three stages. The model below is
**code-verified** (line refs above); the only doc-sourced part is the *internal* 5-step transform
algorithm.

## Stage 1 — parse `.ebnf` → raw-AST token envelope (`src/ebnf_frontend.rs`, HAND-WRITTEN, authoritative)
`scan_top_level_rules` collects each rule's before-rule annotations + body; `convert_scanned_rule`
emits `["rule", name]` + `["semantic_annotation", …]`; `tokenize_rule_expression` tokenizes the body
into a flat list of `[token_type, value]` arrays: `rule_reference`, `quoted_string`, `regex`,
`operator` (`|`/`?`/`*`/`+`/`&`/`!` and `[`→`group_open "("` … `]`→`group_close ")" + operator "?"`),
`quantifier` (`{N,M}`), `group_open`/`group_close`, `return_scalar`/`return_array`/`return_object`,
`semantic_annotation_inline` (branch-start) / `semantic_annotation_mid_sequence` (after some syntax).
`generated/ebnf.rs` (the generated `EbnfParser`) is a **non-fatal cross-check** only — see
[[ebnf-frontend-architecture]].

## Stage 2 — raw-AST → grammar IR (`transform_from_raw_ast`, mod.rs:1355)
Two outputs are built from the token stream:
1. **The structure** via the **5-step transform** (per `docs/ast_transformation_pipeline.md`):
   group-by-OR → handle-parens → parse-sequences → handle-quantifiers (a quantifier binds the
   *preceding* element by look-ahead) → build-tree.
2. **The annotations** via `extract_rule_annotations` (mod.rs:2246), which walks the tokens tracking
   `branch_idx`, `group_depth`, `syntax_position`, and an **inner→outer branch remap** (documented
   fix history — see [[feedback_codegen_outer_branch_remap]]) and fills the `Annotations` side-table.

### The grammar IR = `ASTNode` tree + `Annotations` side-table
- **`ASTNode`** (mod.rs:861) — the structure: `Or { alternatives }`, `Sequence { elements }`,
  `Atom { value: ASTValue }` (leaf — terminal / regex / rule-reference, discriminated by `ASTValue`),
  `Quantified { element, quantifier: String }` (postfix; bounds via `parse_quantifier_bounds`),
  `Lookahead { element, positive }` (`&X`/`!X`, **parse-time** assertion).
- **`Annotations`** (mod.rs:1074), keyed by rule name:
  `semantic_annotations` (per-rule), `branch_semantic_annotations` (per-branch),
  `branch_mid_sequence_semantic_annotations` (per-branch, per-position via
  `MidSequenceSemanticAnnotation { syntax_position, group_depth, annotation }`, mod.rs:1066),
  `branch_return_annotations` (per-branch).

## Stage 3 — two CONSUMERS of `(ASTNode + Annotations)`
- **Codegen** (`ast_based_generator.rs`): emits a Rust parser via `quote!`/`syn` (compile-time-correct
  output — see `docs/AST_GENERATOR_ARCHITECTURE.md`). Consumes **all** annotation granularities,
  including position-specific mid-sequence (`:190`) to emit parse-time semantic actions.
- **Stimuli generator** (`stimuli_generator.rs`, `StimuliGenerator`): emits valid sample inputs.
  Consumes per-rule + per-branch annotations for hints/steering, but **not** position-specific ones.

### ⚠️ `ASTNode` (grammar IR) ≠ `ParseContent` (runtime parse result)
The **generated parser**, at runtime, produces **`ParseContent`** (mod.rs:715): `Terminal(&str)`,
`TransformedTerminal(String)`, `Json(serde_json::Value)` (typed return-annotation carrier),
`Sequence(Vec<ParseNode>)`, `Alternative(Box<ParseNode>)`, `Quantified(Vec<ParseNode>, &str)`. This is
the *output* of parsing, **distinct** from the `ASTNode` grammar IR that *drives* generation. A
`-> {…}` return annotation folds a rule's subtree into `ParseContent::Json` (which is why an AST-walk
of a parse result loses annotated children — see [[ebnf-frontend-architecture]] / the coverage work).

## Annotation consumption matrix (the decisive table)
| Annotation granularity | field | codegen | stimuli generator |
|---|---|---|---|
| per-rule | `semantic_annotations` | ✅ | ✅ |
| per-branch | `branch_semantic_annotations` | ✅ | ✅ |
| per-position (mid-sequence) | `branch_mid_sequence_semantic_annotations` | ✅ | ❌ |

**Implication:** anything the *stimuli generator* must honor has to be expressible at per-rule or
per-branch granularity (or the generator needs new position-keyed consumption it does not have today).
