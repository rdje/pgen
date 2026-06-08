---
id: lexical-follow-restrictions
title: "[> …] / [>! …] lexical follow-restriction annotations — the declarative, parser-agnostic way to control token adjacency in BOTH parse and generation"
answers:
  - "how do I stop the stimuli generator from fusing a keyword with the following token"
  - "the generator emits `define_CV / a keyword fused with an identifier, how do I fix it"
  - "how do I declare a lexical follow restriction in an EBNF grammar"
  - "what is [> ...] / [>! ...] in a pgen grammar"
  - "how do I forbid a rule from being immediately followed by a word char"
  - "parser-agnostic way to enforce a word boundary / separator after a keyword in generation"
  - "REQUIRE vs FORBID follow restriction notation"
  - "what is the fourth declarative pillar / lexical annotations"
  - "how do I declare that token X must not be followed by token Y"
  - "generator over-fuses two adjacent tokens that the parser then rejects"
date: 2026-06-08
status: current
tags: [lexical-annotations, follow-restriction, ebnf, stimuli-generator, word-boundary, parser-agnostic, generation, grammar-wellformed, cert-coverage]
evidence: "Notation parsed by rust/src/ebnf_frontend.rs::parse_lexical_annotation_line (polarity: `[>` REQUIRE / `[>!` FORBID; LIST = one-or-more /regex/ + \"string\" items, ws/comma-separated, union). Collected column-0 by scan_top_level_rules (ebnf_frontend.rs, `trimmed.starts_with(\"[>\")`), bound to the FOLLOWING rule, stored in Annotations.lexical_follow_restrictions: HashMap<String, FollowRestriction{forbid:bool, items:Vec<FollowItem::{Regex,Literal}>}> (ast_pipeline/mod.rs:~1137). Generator: stimuli_generator.rs::apply_lexical_follow_restriction (:7184) — FORBID self-terminates the rule's output with forbid_follow_separator (space→newline whose lead char is not itself forbidden); REQUIRE is a documented generation no-op (parse-direction only). Tests: parses_lexical_follow_restriction_directives, before_rule_lexical_annotation_binds_following_rule, transform_from_raw_ast_carries_lexical_follow_restriction, lexical_forbid_follow_restriction_prevents_distinct_longer_token_fusion, obligation_c_forbid_follow_restriction_golden_and_relex. Introduced LEXICAL-ANNOTATIONS.3c (PGEN-LEXICAL-ANNOTATIONS-0013, 2026-06-06) while cert-covering SV — parser-AGNOSTIC."
reverify: "grep -n 'parse_lexical_annotation_line\\|lexical_follow_restrictions' rust/src/ebnf_frontend.rs rust/src/ast_pipeline/mod.rs; grep -n 'fn apply_lexical_follow_restriction' rust/src/ast_pipeline/stimuli_generator.rs"
---

## The construct

`[> LIST ]` and `[>! LIST ]` are **lexical follow-restriction annotations** — a column-0
directive placed immediately ABOVE the rule it binds (like a standalone `@`-directive). They
are the declarative half of the **fourth declarative pillar, "lexical annotations"** (the other
pillars: return annotations, semantic annotations; the lexical pillar = layout/adjacency
faithfulness). Canonical home: `docs/book/src/lexical-annotations.md` +
[docs/tasks/LEXICAL-ANNOTATIONS.md]. Memory: `project_lexical_annotations_fourth_pillar`.

- **Polarity.** `[> LIST ]` = REQUIRE ("the rule MUST be followed by one of LIST"). `[>! LIST ]`
  = FORBID ("the rule must NOT be immediately followed by any of LIST").
- **LIST.** One or more items, each a `/regex/` or a `"string"`/`'string'` literal, freely mixed
  and whitespace/comma separated; the list is a union. Example: `[>! /\w/, "endmodule"]`.
- **Parser-agnostic.** Pure EBNF surface; nothing parser-specific. Introduced when cert-covering
  the SystemVerilog parser, but applies to ANY grammar.

## What it does, in BOTH directions

- **Generation (FORBID):** `apply_lexical_follow_restriction` self-terminates the rule's rendered
  output with the minimal separator whose leading char is NOT in the forbidden set (`" "` then
  `"\n"`), so the rule's last token can never fuse with whatever the generator emits next. This is
  EAGER and baked into the returned string, so it is robust against every downstream concatenation
  path — it bypasses the DERIVED word-boundary heuristic entirely.
- **Parse direction:** REQUIRE (`[> …]`) is carried for parse-time disambiguation and is a
  documented generation **no-op** (generation cannot force a successor token locally).

## When to reach for it (vs. the derived guard)

There are TWO follow-guards in the generator. **Obligation B** (`apply_word_boundary_spacing`)
*derives* a trailing separator from the rule's regex (a trailing `\b` or a greedy unbounded class
tail) — automatic but heuristic, and it has gaps. **Obligation C** = these `[>! …]` annotations —
*declarative*, explicit, and the robust answer for any fusion derivation can't infer or gets
wrong: e.g. a fixed-literal `<` that must not be followed by another `<` (no regex `\b` to derive
from), or a `\b`-keyword whose rendered text has a non-word prefix (`` `define\b `` → `` `define ``)
where the derived path's whole-token word-shape check mis-classifies the tail and silently drops
the deferred space (the svpp `` `define_CV ``/`` `ifndefR7Sh `` directive-keyword fusion class,
GRAMMAR-WELLFORMED.H.5.1.2). Per the strict fix hierarchy ([[feedback_no_workarounds_fix_hierarchy]]),
this **existing semantic annotation is the level-1 fix** — prefer it over patching the generator's
derived heuristic (a level-5 engine change).

Related: [[cert-coverage-measures-structural-not-validator]], [[ebnf-single-source-of-truth]],
[[stimuli-generator-construction-path]].
