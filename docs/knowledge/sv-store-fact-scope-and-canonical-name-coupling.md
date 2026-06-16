---
id: sv-store-fact-scope-and-canonical-name-coupling
title: Two witness-construction facts — top-level binding facts are visible to later post-predicates, and construct-mode identifiers render canonically (free name-coupling)
answers:
  - "is a top-level / $unit variable_binding fact visible to has_fact later in the same source"
  - "does a compilation-unit-scope declaration satisfy has_fact at a deeper construct"
  - "are semantic-store facts global or scoped for has_fact"
  - "how do I witness a has_fact-gated rule like context_member_method_call"
  - "do the generator's producer and consumer identifiers render the same string"
  - "is name-coupling between an @emit_fact producer and a @predicate consumer free"
  - "what makes context_member_method_call witness in cert-coverage"
tags: [systemverilog, semantic-store, has_fact, scope, stimuli, witness, name-coupling, cert-coverage, grammar-wellformed]
date: 2026-06-16
status: current
evidence: "parseability_probe --parse-dump-ast-pretty systemverilog <s> --profile sv_2017 (witness matrix a/b/c/c2/d/e); PGEN_REACH_PATH_DUMP=1 + PGEN_CERT_COVERAGE_DEBUG_PROBES=1 cert-coverage seed 0; grammars/systemverilog.ebnf:2892/2906 (consumer), :5403/5404 (producer); docs/tasks/GRAMMAR-WELLFORMED-H125533421-context-member-prelude-whywhere-design.md"
reverify: "printf 'int \\\\foo ; (*\\\\foo =+\\\\foo .\\\\foo [0].\\\\foo ()*);' > /tmp/c2.sv && ./rust/target/debug/parseability_probe --parse-dump-ast-pretty systemverilog /tmp/c2.sv /tmp/c2.ast.json --profile sv_2017 && grep -c context_member_method /tmp/c2.ast.json"
---

Two durable facts established tools-first while designing the `has_fact` semantic-prelude witness
for `context_member_method_call` (GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.1.1). They govern how to
*construct* a witness for any `has_fact(K, $name)`-gated rule, not just this one.

## Fact 1 — a top-level (`$unit`/root-scope) `variable_binding` fact is visible to a later same-region `has_fact` post-predicate

A `variable_binding` emitted by a **top-level** `int \foo ;` data declaration satisfies
`has_fact(variable_binding, \foo)` evaluated inside a *later* top-level `description`-level
construct — even an `attribute_instance` `(* \foo = \foo.\foo[0].\foo() *)`. Empirically (parser
is the judge, `context_member_method` AST node = witness):

| sample | witnesses |
|---|---|
| `(*\foo =+\foo .\foo [0].\foo ()*);` (no decl) | no — gate fails |
| `int \foo ; (*\foo =+\foo .\foo [0].\foo ()*);` (top-level decl) | **YES** — gate satisfied across the decl→attr gap |

So for witness construction you do **not** have to re-select the reach path toward a module/program
body to host the binding: a top-level binding-producer prelude (a `source_text_item` data
declaration) injected on the existing reach path satisfies a deeper post-predicate. The semantic
store's `has_fact` index is `(kind, scope_depth, name)` (`semantic_runtime.rs:1030`), and the
compilation-unit/root-scope fact remains visible to the later same-region check. (Scope: this is
the *top-level → later top-level item* configuration; it is **not** a claim that any inner-scope
fact is globally visible — declarations inside a `class`/`module` open their own scope.)

## Fact 2 — construct-mode identifiers render canonically as `\foo`, so producer↔consumer name-coupling is free

Under the generator's **construct/minimal** mode (the plannable + reach passes), every
identifier-family terminal renders the same canonical escaped identifier `\foo`. Both
`context_member_method_call`'s head (`$head = $1.body`, an `identifier`) and
`variable_decl_assignment`'s emitted name (`$name.body`, a `variable_identifier`) derive from the
same identifier machinery, so a binding-producer prelude and the gated rule's head both render
`\foo` → `has_fact(variable_binding, \foo)` is satisfied **without any explicit name-coupling
machinery**. (Caveat: the *target-own-structure* pass re-rolls terminals to random identifiers —
`(*u6H=+P.Mp.Q6.DlM9*)` — so a witness that needs both the binding prelude AND a forced
distinguishing structure must keep them on the same canonical render, or couple the names
explicitly. The parser stays the judge — a name mismatch fails loudly, never false-witnesses.)

## Why it matters

These two facts are the foundation for extending the semantic-prelude reach
([[stimuli-generator-construction-path]], the regex `\NN` `fact_count_at_least` precedent in the
Grammar Well-Formedness book's "Reaching store-gated rules") from **count-gated** to
**`has_fact`/name-coupled** gates: one top-level binding declaration (iterations = 1, no numeric
capture, no count-prune bypass — the gate is a PARSE-time post-predicate) suffices, and coupling is
structural on the canonical path. See also [[branch-predicate-is-rule-wide]],
[[prove-rule-dead-or-reachable]], [[grammar-coverage-and-directed-generation]].
