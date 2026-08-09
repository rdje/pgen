---
id: a-bare-rule-reference-alternative-is-ast-transparent
title: Splitting alternatives into a `@profiles`-gated sibling does NOT reshape the AST — a bare rule-reference alternative carrying no return annotation adds no wrapper node
answers:
  - "does moving alternatives into a sibling rule bump the AST schema"
  - "how do I gate SOME alternatives of a rule to a profile when @profiles is rule-level"
  - "will a sibling split break downstream consumers of this rule's JSON"
  - "does a bare rule reference in an Or branch add a level of nesting to the AST"
  - "how do I prove a grammar restructuring is shape-neutral instead of assuming it"
tags: [grammar-authoring, profiles, ast-shape, schema-versioning, systemverilog, codegen]
date: 2026-08-09
status: current
evidence: docs/tasks/artifacts/sv_corpus_grad/config_use_param_override/ast_identity_3_20.txt (15/15 cmp-identical ASTs across the use_clause split, plus the 6 still-legal verilog_2005 cases); ir_split_verification_3_20.txt (the IR diff — one rule added, one body changed, rule_order otherwise identical); rust/src/ast_shape_contract.rs:639 (the observed-vs-manifest content_kind regression lock, GREEN across the split); grammars/systemverilog.ebnf use_clause / use_clause_param_override_sv_only, and the house pattern always_keyword / always_keyword_sv_only; docs/tasks/SV-CORPUS-GRAD.md leaf .3.20
reverify: "./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --dump-rule-profiles /tmp/rp.json >/dev/null && python3 -c \"import json;d=json.load(open('/tmp/rp.json'));print(d['rules']['use_clause_param_override_sv_only'])\""
---

**`@profiles` is a RULE-level directive.** So gating *some* of a rule's alternatives to a dialect
cannot be done in place — the alternatives must move into their own rule, which the original then
references. The house pattern in `grammars/systemverilog.ebnf` is:

```ebnf
@profiles: ["sv_2017", "sv_2023"]
always_keyword_sv_only := kw_always_comb -> {kind: "always_comb"} | …

always_keyword := kw_always -> {kind: "always"}
                | always_keyword_sv_only
```

The obvious worry is that this is a **restructuring**, and restructurings are the classic silent
schema bump: an extra rule frame looks like it should add an extra level of nesting to the emitted
tree.

**It does not.** A branch whose entire body is one rule reference, and which carries no return
annotation of its own, is *transparent*: the referenced rule's value is the branch's value. The
emitted AST is byte-identical before and after the split.

## Measured, not reasoned

`SV-CORPUS-GRAD.3.20` split four alternatives out of `use_clause` and checked two independent ways:

| instrument | result |
|---|---|
| `--parse-dump-ast` on every accepting repro case, pre vs post, `cmp` | **15/15 byte-identical** under `sv_2017`; the 6 still-legal cases identical under `verilog_2005` |
| `ast_shape_contract` regression lock (`ast_shape_contract.rs:639`) — compares the OBSERVED `content_kind` of a live parse against the manifest | unmoved; gate GREEN |

⭐ The second one matters more than it looks: it is an **oracle re-run on every gate invocation**,
not a note. If a future emission change makes rule-reference branches non-transparent, that lock
goes red by name.

## The two traps the split must still clear

Shape-neutrality is not the same as behaviour-neutrality. Two things DO change and must be handled:

1. **Ordering is relocated, not relaxed.** If the extracted alternatives had to precede a
   more-general sibling (PEG specific-before-general), the *reference* must sit in that position.
   Referencing the new rule last silently re-introduces the original defect — and that class shows
   up as a cert-coverage witness residual, not as a parse failure, so it is easy to miss.
2. **`$N` indices are per-ALTERNATIVE, so they do not shift.** A return annotation on an alternative
   that stays behind keeps its indices exactly. Verify it in the IR
   (`--dump-gen-ast` → `annotations.branch_return_annotations[<rule>]`) rather than by counting
   elements in the source.

And the standing one: **any comment block that moves into the new rule's body must be INDENTED** —
see [[a-column-0-comment-inside-a-rule-body-deletes-the-following-alternatives]].

## The general rule

*"I restructured the grammar"* is not by itself a schema bump, and *"I only added a rule"* is not by
itself a non-bump. **The AST is the arbiter and it costs one `--parse-dump-ast` per case to ask.**
The census will move (`defined_rule_count` +1, and the per-profile counts asymmetrically — see
[[an-instrument-that-prints-an-unjudged-column-reports-a-defect-as-green]] for the sibling lesson on
predicting that move before running the gates), but a census move is a contract re-baseline, not a
consumer-visible shape change.
