<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/reference_annotation_binds_following_rule.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: reference-annotation-binds-following-rule
description: "A standalone @-directive (@emit_fact/@predicate/@open_scope/@transform/...) binds to the rule that FOLLOWS it, not the preceding one. Authoritative: ebnf.ebnf:70 grammar_rule := annotation_list? rule_definition."
metadata: 
  node_type: memory
  type: reference
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

A standalone `@…` semantic/transform directive in any `*.ebnf` binds
to the rule declaration that **immediately FOLLOWS it**, NOT the
preceding rule. Put the `@`-directive **directly above** the rule it
should annotate.

**The trap (own it — it is NOT the layout's fault):** the
`systemverilog.ebnf` declared-X idiom (`X := …` / blank /
`@emit_fact` / `@predicate` / `declared_X := X`) is fully consistent
with the normative leading-annotation rule — the block is the
LEADING `annotation_list?` of the `declared_X` that follows it
(verified: `generated/systemverilog_parser.rs:1807`
`directives_by_rule.insert("declared_let_identifier", …)`). The
mistake is to infer direction from visual adjacency instead of
consulting `ebnf.ebnf:70` first (the binding discipline in
[[feedback_ebnf_consult_annotation_docs]] requires consulting the
meta-grammar/docs BEFORE any `.ebnf` annotation edit). A mis-placed
directive silently mis-binds to the wrong rule (e.g. a `@predicate`
whose `$N` ref then can't resolve → predicate no-ops, no error).

**Authoritative anchors (verify before relying):**
- `grammars/ebnf.ebnf:69-75` — `grammar_rule := annotation_list?
  rule_definition -> {…, annotations:$1, rule:$2}`; comment: "semantic
  annotations that can **precede a rule**" (annotation = leading part,
  rule = the one that follows).
- `rust/src/ebnf_frontend.rs:111-156` — `pending_annotations`
  accumulates `@…` blocks and `std::mem::take`s them onto the *next*
  rule header (intervening `#` comments / blank lines are skipped, so
  they don't break or redirect the binding).

**How to verify a binding landed right:** after regen, grep
`generated/<grammar>_parser.rs` for
`directives_by_rule.insert("<rule>", …)` — the key is the rule the
directive actually bound to. Do this read-only check BEFORE trusting
a behavioral probe (a mis-bound directive shows up as a silent
no-op, not an error). See [[feedback_ebnf_consult_annotation_docs]]
and the SV-EXH-PROOF "never trust a stale/mis-built artifact" lesson.
