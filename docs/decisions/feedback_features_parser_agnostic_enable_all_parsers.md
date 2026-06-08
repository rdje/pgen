---
name: feedback-features-parser-agnostic-enable-all-parsers
description: STANDING DOCTRINE (director 2026-06-08) — every feature introduced while building/debugging ONE parser, ESPECIALLY semantic-annotation features (has_fact/lacks_fact/fact_attribute_equals/fact_count_at_least/the store/predicates), SHALL be parser-AGNOSTIC and, in fine, ENABLED for ALL PGEN parsers (present + future) when it makes sense and is rational. Proven-for-SV ⇒ safe-for-all. A feature being USED by only one grammar (e.g. SV) does NOT make it parser-specific — the mechanism is general and the enablement must be general. NO parser-specific divergence in the AST pipeline. Reinforces [[feedback_ast_pipeline_parser_agnostic]].
metadata:
  node_type: memory
  type: feedback
  director_directive: true
  standing: true
  created: 2026-06-08
---

**THE DOCTRINE (director, 2026-06-08, verbatim intent).** "Every feature we [are] introducing for SV
are and should be made parser-agnostic, especially those related to the semantic annotations — why they
wouldn't be. We proved they … work for the SV parser, so it is safe to enable them for any other PGEN
parsers, present and future. There must be a way to achieve that. The doctrine now, regarding features
introduced when debugging one parser, is to in fine enable it for all the parsers when it makes sense
and it is rational to do so."

**WHAT IT MEANS.**
- The semantic-annotation primitives (`has_fact`, `lacks_fact`, `fact_attribute_equals`,
  `fact_count_at_least`, the universal semantic store, `@predicate`/`@emit_fact`/`@fact_kind`,
  scopes, `resolve_path`, …) are **parser-agnostic engine features**. They live in the AST pipeline
  (`semantic_runtime.rs` et al.), NOT in any one grammar.
- A feature being **used by only one grammar today** (e.g. `has_fact`/`lacks_fact`/`fact_attribute_equals`
  are currently written only in `systemverilog.ebnf` — 41 `@predicate` sites) does **NOT** make it
  SV-specific. The MECHANISM is general; the ENABLEMENT (e.g. honouring those predicates during stimuli
  GENERATION) must be general too — gated on *whether a grammar uses the feature*, never on the grammar's
  *name*.
- **Proven-for-SV ⇒ safe-for-all.** Once a semantic-annotation feature is demonstrated working for the SV
  parser, it is safe — and required — to enable it for every other PGEN parser, present and future. The
  "verification" of enabling it broadly is normal regression checking (does any grammar regress?), NOT a
  reason to treat one grammar as a special risky case.
- **NO parser-specific divergence in the AST pipeline.** Grammar-name checks (`if grammar_name ==
  "systemverilog…"`), per-parser tuning branches, and "this only applies to SV" framing are an
  anti-pattern. They are technical debt to be de-specialized into general, capability-gated mechanisms.

**TRIGGER (the mistake this corrects).** In `STORE-AWARE-GEN.4` scoping (`PGEN-STORE-AWARE-GEN-0004`) I
framed generalizing the store-aware-GENERATION predicate-honoring (`has_fact`/`lacks_fact`/…) as
"SystemVerilog-only / high-risk because it touches the tuned SV surface." The director corrected: the
semantic-annotation honouring is a parser-AGNOSTIC feature and must be enabled generally — SV is merely
the current user, not a special case. (The `.3` `gen_semantic_state` mechanism is already parser-agnostic
— gated on the *presence of a `fact_count_at_least` predicate*, not on the grammar name — which is the
correct pattern; the error was the `.4` *framing*, and the standing risk is letting parser-specific
divergence accrete in the pipeline.)

**HOW TO APPLY.**
- When a feature is introduced to fix/build one parser, design it parser-agnostic from the start and,
  once proven, enable it for all parsers (capability-gated, not name-gated).
- Treat any existing `grammar_name`-keyed branch or per-parser special-case in the AST pipeline as a
  defect to de-specialize (a standing audit lane — likely its own task tree).
- For `STORE-AWARE-GEN.4`: the predicate-honoring generalization is the CORRECT parser-agnostic direction
  (not an SV-specific risk); verify it doesn't regress any grammar (SV included), but frame and build it
  as general. Sequence it after `GRAMMAR-WELLFORMED` Phase H gathers the per-grammar cert-coverage
  evidence.

Composes with — and sharpens — [[feedback_ast_pipeline_parser_agnostic]] (the AST pipeline is
parser-agnostic; parsers are data, not code) and [[feedback_no_workarounds_fix_hierarchy]] (a Level-3+
addition must be a GENERAL parser-agnostic primitive). Governs every feature slice going forward.
