---
id: sv-cert-coverage-predicate-gated-false-witness
title: The cert-coverage UNKNOWN count can be fooled by a predicate-gated rule — a prelude that satisfies the predicate on a structurally-degenerate render produces a FALSE witness; the genuineness oracle is the AST node, not the count
answers:
  - "is the cert-coverage UNKNOWN count sufficient proof that a rule is genuinely witnessed"
  - "how do I tell a genuine cert-coverage witness from a false one"
  - "why did a has_fact-prelude drop UNKNOWN without the rule actually parsing"
  - "can a semantic-prelude false-witness a rule in cert-coverage"
  - "what verifies a context_member_method_call witness is real"
  - "does a passing post-predicate plus a degenerate render count as a witness"
  - "where does a false witness leak from in the transactional coverage record"
tags: [systemverilog, cert-coverage, witness, has_fact, predicate, soundness, stimuli, grammar-wellformed, memoization, false-witness]
date: 2026-06-16
status: current
evidence: "PGEN-GRAMMAR-WELLFORMED-0102 (GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.1.2): a has_fact semantic-prelude made cert-coverage report UNKNOWN 88->87 for context_member_method_call, but parseability_probe --parse-dump-ast-pretty showed 0 context_member_method nodes in the armed sample (vs 1 in the genuine Test c2) and --trace-rules context_member_method_call showed only backtracks (no positive success-exit); RULE_CONTEXT_MEMBER_METHOD_CALL=547 (generated/systemverilog_parser.rs:634); coverage push at :449256; docs/tasks/GRAMMAR-WELLFORMED-H1255334212-context-member-prelude-fix-attempt-refuted.md"
reverify: "printf '(*\\\\foo =+\\\\foo .\\\\foo .\\\\foo *);' > /tmp/a.sv && ./rust/target/debug/parseability_probe --parse-dump-ast-pretty systemverilog /tmp/a.sv /tmp/a.ast.json --profile sv_2017 && echo bare-ref-nodes=$(grep -c context_member_method /tmp/a.ast.json); printf 'int \\\\foo ; (*\\\\foo =+\\\\foo .\\\\foo [0].\\\\foo ()*);' > /tmp/c2.sv && ./rust/target/debug/parseability_probe --parse-dump-ast-pretty systemverilog /tmp/c2.sv /tmp/c2.ast.json --profile sv_2017 && echo method-call-nodes=$(grep -c context_member_method /tmp/c2.ast.json)"
---

**Claim.** The cert-coverage `UNKNOWN` count is *necessary but not sufficient* proof that a rule is
genuinely witnessed when that rule carries a **semantic post-predicate** (`@predicate has_fact(...)`,
`fact_count_at_least`, …). A semantic-prelude that makes the predicate *pass* on a
**structurally-degenerate render** (one that satisfies the predicate but does not contain the rule's
distinguishing structure) can drive the count down via a **false witness** — the rule is credited
even though the accepted parse never enters it.

**Worked instance.** `context_member_method_call := identifier ( dot identifier constant_bit_select
&dot )+ dot callable_method_call_body …` is gated by `@predicate has_fact(variable_binding, $head)`
and needs a real `.method()` (a `callable_method_call_body`). The `H.12.5.5.3.3.4.2.1.2` `has_fact`
prelude injected a top-level `\foo` binding, which made the post-predicate pass on the plannable
pass's degenerate render `… \foo .\foo .\foo` (a bare hierarchical reference, **no `()`**).
Cert-coverage reported `UNKNOWN 88 → 87`. But:

- `parseability_probe --parse-dump-ast-pretty` on the armed sample → **0** `context_member_method`
  AST nodes; the genuine Test c2 (`int \foo ; (*\foo =+\foo .\foo [0].\foo ()*)`, a real `.\foo()`)
  → **1**. (`call_primary` sets `kind: "context_member_method"` explicitly, so a committed match
  shows it — for *this* rule the AST node is a reliable genuineness oracle.)
- `--trace-rules context_member_method_call` on the armed sample → only `Speculative parse failed /
  backtracked`; **no positive success-exit**. The rule never commits.

So the witness was false. The leak is a memoization × transactional-coverage composition gap (the
same class as `H.10.2.2`): the rule's coverage id is pushed at entry
(`generated/systemverilog_parser.rs:449256`), and on the **post-predicate-passes-then-structurally-
fails** path the contribution survives into the witness record (rule 547 shows a "Memoized successful
result" while absent from the committed AST). It is **latent in the shipped grammar** — without the
prelude the gate fails and the rule rejects early, so no degenerate predicate-passing render is ever
generated.

**How to apply.** When closing a cert-coverage `UNKNOWN` for a **predicate-gated** rule, do **not**
trust the `UNKNOWN`/`witness` delta alone. Verify genuineness with the construct's own oracle:

1. dump the armed witnessing sample (`PGEN_CERT_COVERAGE_DEBUG_PROBES=1`), and
2. confirm the construct is in the **committed AST** (`--parse-dump-ast-pretty`, grep the rule's
   distinguishing `kind`) **or** a positive success-exit in `--trace-rules <rule>` — not merely that
   the sample parses and the count dropped.

A passing post-predicate on a degenerate render is exactly the trap. The genuine witness must carry
the rule's distinguishing structure (here, a real `.method()`), which is why the fix needs the prelude
*composed with* mandatory child-structure forcing — and why the underlying coverage-record soundness
gap (a non-committing, predicate-passing rule must never be witnessed) deserves its own fix.

**Update (`PGEN-GRAMMAR-WELLFORMED-0103`, `H.12.5.5.3.3.4.2.1.2.1` WHY+WHERE).** The soundness gap is
reproducible WITHOUT the prelude — a real declaration emits the same fact:
`int \foo ; (*\foo =+\foo .\foo .\foo *);` false-witnesses `context_member_method_call` through
`parse_and_cover_systemverilog` (∈ `exercised_rule_names`) while its AST shows 0
`context_member_method` nodes. The leak is PINNED: the rule's body memoizes a structurally-degenerate
SUCCESS, the post-predicate passes, the committed parse routes the bytes through a sibling (rule absent
from the AST), and the rule's coverage push — **removed only by `try_parse` truncation, which the
discard never passes through** — is frozen into the `coverage_delta` of an ancestor memo entry
(`attribute_instance`) that is memo-hit on the committed path and replayed. The `H.10.2.2`
replay-transactionality fix cannot help: the captured delta was already wrong at capture (result node
omits the rule, `coverage_delta` includes it). Root: a success-then-reject (post-predicate-reject at
`with_semantic_runtime_rule_transaction` `:1835`/`:1874`, or a committed-then-abandoned ordered-choice
branch) leaves the coverage push because that reject path rolls back facts + rule-context but NOT
`coverage_stack`. Fix is its own slice. See
[GRAMMAR-WELLFORMED-H1255334212-1-cert-coverage-witness-soundness-whywhere.md](../tasks/GRAMMAR-WELLFORMED-H1255334212-1-cert-coverage-witness-soundness-whywhere.md).

Related: [[cert-coverage-measures-structural-not-validator]], [[memo-hit-transactional-replay]],
[[grammar-linter-trustworthiness]], [[sv-store-fact-scope-and-canonical-name-coupling]],
[[prove-rule-dead-or-reachable]].
