<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_ast_pipeline_parser_agnostic.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback-ast-pipeline-parser-agnostic
description: Any AST-pipeline / semantic-annotation-runtime change MUST be parser-agnostic and general-purpose. Parser-specific concepts (e.g. regex capture-groups) stay in the .ebnf grammar; new steering primitives must benefit ANY parser.
metadata:
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**Binding (user, 2026-05-18, during RGX-0084):** every change to the
shared AST pipeline / semantic-annotation runtime
(`rust/src/ast_pipeline/semantic_runtime.rs`, the `@predicate` /
`@emit_fact` / `@open_scope` vocabulary, etc.) **must be
parser-agnostic** — a general capability usable by *any* current or
future parser, not tailored to one grammar.

**Why:** the AST pipeline is shared infrastructure for all parser
families (regex/systemverilog/vhdl/rtl_*/preprocessor/return_annotation/
…). A parser-specific amendment there is architectural debt and
violates the platform's separation: parser-specific knowledge belongs
in that parser's `.ebnf` grammar, NOT in the engine.

**How to apply:**
- Before amending the AST pipeline / semantic-annotation runtime,
  **confirm 100%** (with concrete cross-parser use-cases) that the new
  primitive benefits parsers beyond the one motivating it. If it
  encodes a parser-specific concept (e.g. regex "capture group", SV
  "module"), it is wrong — generalize it.
- Context-steered parsing is done via **semantic annotations** (the
  user's confirmed preferred mechanism), and the vocabulary MAY be
  extended with new general steering strategies — but each strategy
  must be expressible/usable by any grammar.
- The parser-specific *decision* (which rule emits which fact, what to
  gate) lives entirely in the `.ebnf` grammar; the engine only
  provides the generic mechanism.
- Concrete RGX-0084 application of this rule: do NOT add a
  `capture_group_count` predicate (regex-specific). Add a **generic
  running-fact-count predicate** (e.g. `fact_count_at_least(kind, M)`
  over the existing generic fact store — a strict generalization of
  `has_fact`, which is `count(kind,name) >= 1`). `regex.ebnf` then
  chooses to `@emit_fact` a generic fact on `capturing_group` and
  `@predicate`-gate `backreference`'s numeric branch on its running
  count (PCRE2: backref iff group N opened up to here ⟺ ≥ N such
  facts so far; octal otherwise). Other future consumers:
  declaration-count-before-use, nesting/recursion bounds, ordinal
  -sensitive parsing in SV/VHDL/etc.

**Explicitly includes `rust/src/ast_pipeline/stimuli_generator.rs`**
(the stimuli / closed-loop generator) — user reaffirmed 2026-05-18,
"of utmost importance", during SV-EXH-PROOF.2.3.2. A generator fix
must be a **general property of grammar structure** (e.g. "a
permissive *leading negated* regex class implies its negated chars
are content-hazardous for that rule" — P-a; or "a required trailing
element of a sequence/recursive rule must not be truncated under
budget/target-drive pressure"), derived purely from the regex
HIR / rule shape — **never** hardcode a grammar's rule names or
sigils (no `pp_endif`, `` ` ``, `non_directive_text`,
`sv_preprocessor`, …). Verify agnosticism by inspecting the actual
diff: the production logic must contain zero grammar/parser/EBNF
identifiers (concrete tokens are allowed only inside *tests* as
inputs exercising the agnostic logic). P-a (SV-EXH-PROOF.2.3.2)
passed this bar; the Mode-B fix must too.

Related: [[feedback_ebnf_consult_annotation_docs]] (consult annotation
refs + copy the proven idiom before any `@`/`->` edit),
[[feedback_grammar_edit_proof_gate_lockstep]] (a grammar/engine change
owns ALL its downstream proof/contract gates same-slice — here the
SC-01..SC-07 semantic-runtime gates + the semantic-annotation
book/contract).
