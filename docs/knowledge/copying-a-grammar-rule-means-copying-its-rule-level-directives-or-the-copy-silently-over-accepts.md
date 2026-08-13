---
id: copying-a-grammar-rule-means-copying-its-rule-level-directives-or-the-copy-silently-over-accepts
title: A transform that CLONES a grammar rule must copy what is attached to it by KIND — a dropped rule-level `@profiles:` turns a structural rewrite into a silent over-acceptance
answers:
  - "my codegen pass clones a rule and the clone is reachable under profiles the original was gated out of"
  - "what has to be copied when an engine pass synthesizes a copy of an existing rule"
  - "why did profile_orphans go from 0 to 4 after I added a left-recursion helper rule"
  - "can an EBNF alternative carry @profiles"
  - "how do I keep a dialect discriminator honest after left-recursion elimination"
  - "a purely structural grammar rewrite changed what the parser accepts — how"
tags: [engine, annotations, profiles, grammar-transform, over-acceptance, left-recursion, ast-pipeline]
date: 2026-08-13
status: current
evidence: rust/src/ast_pipeline/indirect_lr_elimination.rs (`CloneRule::rule_semantic`, the `@profiles:`-only helper inheritance, and `SuffixBranch::rule_name`); its unit test `a_profile_gate_survives_on_both_the_clone_and_the_suffix_route`; the measured emitted pair — `parse_constant_primary_sv_2017` carrying `rule_profile_is_enabled(&["sv_2017","verilog_2005"])` next to a first-draft clone carrying nothing; `profile_orphans` 0 -> 4 on the helper rules of a gated base
reverify: "n=$(grep -n 'fn parse_constant_primary_lr_seed_constant_primary_sv_2017(' generated/systemverilog_parser.rs | cut -d: -f1); awk -v s=$n 'NR>=s && NR<=s+200 && /rule_profile_is_enabled/' generated/systemverilog_parser.rs   # must print the SAME gate as the rule it clones"
---

**A rule is not its body.** Attached to it, by kind, are: per-branch return annotations, per-branch
semantic annotations, mid-sequence semantic annotations, **rule-level** semantic annotations
(`@profiles:`, `@predicate:`, `@emit_fact:` …) and the lexical follow-restriction. A transform that
synthesizes a copy and carries only the fields it happened to think of ships a rule that *parses*
the same text under conditions the original was excluded from.

Measured: an indirect-left-recursion pass cloned `constant_primary_sv_2017` — gated
`@profiles: ["sv_2017", "verilog_2005"]` — and copied only the per-branch annotations. The emitted
clone had **no** `rule_profile_is_enabled` guard, so the sv_2017 primary became reachable under an
`sv_2023` parse. Every unit test passed. Every lint passed. What found it was reading the generated
function next to the one it clones.

**Three consequences, all of them non-obvious:**

1. **The clone takes the source rule's rule-level directives verbatim.** It stands in for the
   original at that position, so it must be constrained identically.
2. **Synthetic HELPER rules inherit the base rule's `@profiles:` — and only `@profiles:`.** Helpers
   carrying a gated rule's productions while being universal themselves are *profile orphans*:
   present under a profile where nothing they reference is satisfiable (`profile_orphans` 0 → 4 on
   one gated base). But copy no other rule-level directive: `@profiles:` says whether the rule
   **exists**; the rest say what happens when it **runs**, and the original still runs, so a copy
   would fire them twice per parse.
3. ⭐ **An alternative cannot carry `@profiles:` — only a rule can.** So when a rewrite produces
   alternatives that need *different* gates, each must be hoisted into its own rule. Two dialect
   routes through a shared spine iterate byte-identical suffixes and differ only in the AST they
   declare (`{kind: "sv_2017", …}` vs `{kind: "sv_2023", …}`); inline, the ordered choice hands the
   first template to both profiles and stamps the wrong discriminator.

**And copy the directive, never synthesize it.** Where a gate must be *derived* (the intersection of
the `@profiles` lists along a route), install it by cloning the annotation of the rule whose declared
list IS that intersection, and refuse the plan when no rule's list matches. A constructed annotation
is a second spelling of a directive whose only authoritative spelling is the grammar's — and a second
spelling is a second thing to drift.

Related: [[a-grammar-rewrite-must-verify-its-own-postcondition-because-a-static-criterion-only-answers-what-you-encoded]],
[[a-bare-rule-reference-alternative-is-ast-transparent]].
