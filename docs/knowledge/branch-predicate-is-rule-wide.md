---
id: branch-predicate-is-rule-wide
title: "An inline `@predicate … phase: branch` is applied RULE-WIDE (flattened across every branch), not just its own branch — for a branch-LOCAL gate use `phase: post` on a dedicated helper rule"
answers:
  - "why does my phase: branch predicate reject a sibling branch / the wrong branch"
  - "my fact_count_at_least phase: branch predicate blocks the plain `wire a;` branch"
  - "how do I gate a SINGLE branch of a multi-branch rule with a content-free predicate"
  - "is an inline phase: branch predicate branch-local or rule-wide in pgen"
  - "branch-local vs rule-wide semantics of @predicate phase: branch"
  - "how do I make a per-branch semantic gate that does not leak to other branches"
  - "content-free branch predicate gates every branch — how to scope it to one branch"
  - "phase: post helper rule vs inline phase: branch predicate"
date: 2026-06-09
status: current
tags: [semantic-store, predicate, branch, phase, ebnf, parser-agnostic, sv-parse-strict, gotcha]
evidence: "CompiledSemanticRuntimeAnnotations::branch_predicates_for_rule(rule) (rust/src/ast_pipeline/semantic_runtime.rs:735) chains directives_for_rule(rule) with branch_directives_for_rule(rule).flat_map(...) (ALL branches flattened) and filters is_branch_predicate. The codegen branch-attempt loop (ast_based_generator.rs ~2979) iterates branch_predicates_for_rule(rule).chain(branch_predicates_for_rule_branch(rule, current_branch_index)) for EVERY branch — so a phase: branch predicate declared on ANY branch is evaluated against EVERY branch. A CONTENT-FREE predicate (no $ref, e.g. fact_count_at_least(K, N)) always resolves+evaluates, so it gates all branches uniformly. Tools-proven (SV-PARSE-STRICT.2): an inline `@predicate fact_count_at_least(wildcard_import_open,1) phase: branch` on net_declaration_sv_2017's wildcard-escape branch made `module m; wire a; endmodule` REJECT — trace: `✅ Leaving branch 1/4 ... (success)` then `🚫 Branch 1/4 ... rejected by branch predicate 'fact_count_at_least[wildcard_import_open, 1]'`. Fix: move the gate to `@predicate ... phase: post` on a dedicated helper rule (wildcard_escape_nettype_identifier := declaration_identifier -> {body:$1.body}) referenced by ONLY that branch — a post-predicate is evaluated on its OWN rule, so it stays branch-local."
reverify: "grep -n 'fn branch_predicates_for_rule\\b' rust/src/ast_pipeline/semantic_runtime.rs; sed -n '735,757p' rust/src/ast_pipeline/semantic_runtime.rs; grep -n 'branch_predicates_for_rule(' rust/src/ast_pipeline/ast_based_generator.rs"
---

## The gotcha

In a multi-branch rule, an inline branch-start predicate

```ebnf
my_rule := alt_a_items                                              -> { ... }
         | @predicate: { name: P, args: [...], phase: branch } alt_b_items  -> { ... }
```

does **not** gate only `alt_b`. `CompiledSemanticRuntimeAnnotations::branch_predicates_for_rule`
flattens **every** branch's branch-predicates into one set, and the generated branch-attempt
loop evaluates that whole set against **each** branch. So predicate `P` is checked when the
parser tries `alt_a` too.

For a predicate that references the branch's matched content (`$scope.name.body`, …) this is
often self-limiting: the `$ref` fails to resolve against a non-matching branch, which blocks
that branch — sometimes harmlessly, sometimes not. But for a **content-free** predicate
(`fact_count_at_least(K, N)`, which takes no `$ref`) the predicate **always** evaluates, so it
gates *every* branch identically. That is the bug behind SV-PARSE-STRICT.2's first attempt:
`fact_count_at_least(wildcard_import_open, 1)` placed inline on the wildcard-escape branch
rejected the sibling `wire a;` branch whenever no wildcard import was in scope.

## The fix pattern — branch-local gating via a `phase: post` helper rule

A `phase: post` predicate is evaluated against the rule it is attached to, so putting the gate
on a **dedicated helper rule used by only one branch** keeps it branch-local:

```ebnf
@predicate: { name: fact_count_at_least, args: [wildcard_import_open, 1], phase: post }
wildcard_escape_nettype_identifier := declaration_identifier -> { body: $1.body }

net_declaration_sv_2017 := net_type ... semi                       -> { kind: "wire",  ... }
                         | checked_nettype_identifier ... semi     -> { kind: "alias", ... }
                         | wildcard_escape_nettype_identifier ... semi -> { kind: "alias", ... }
                         | kw_interconnect ... semi                 -> { kind: "interconnect", ... }
```

Now the gate fires only when the parser actually enters the helper (i.e. only on that branch).
This is the same idiom as `checked_type_identifier` / `checked_nettype_identifier` (a helper
rule carrying a `phase: post has_fact` gate). Use the inline `phase: branch` form only when you
genuinely want a **rule-wide** gate, or when the predicate's `$ref` is naturally branch-selective.

## See also

- [[ebnf-single-source-of-truth]] — the EBNF (+ its annotations) is the sole acceptance authority.
- The semantic-store book chapter (`docs/book/src/semantic-store.md`) for the predicate phases
  (`pre` / `branch` / `post`) and the emit/query lifecycle.
