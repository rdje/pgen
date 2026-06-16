# GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.1.2.1 — cert-coverage witness-SOUNDNESS gap: WHY+WHERE (the leak PINNED tools-first)

Tools-first WHY+WHERE for the foundational cert-coverage soundness gap opened by the `-0102`
refutation: **a rule whose body memoizes a structurally-degenerate success and whose post-predicate
passes can be credited as `witnessed` even though the committed parse never enters it.** This slice
(`PGEN-GRAMMAR-WELLFORMED-0103`) reproduces the gap on a **plain SV input** (no reverted prelude
needed) and pins the exact leak leg. It is **PURE-DOCS** — no code/grammar/generated/release/schema/
ledger change; the throwaway reproduction test + trace probes were reverted.

> Scope: layer-B detail behind the `H.12.5.5.3.3.4.2.1.2.1` row. Opened by `.4.2.1.2` (`-0102`),
> which discovered the gap but left the exact leak leg as an open question (the task file's own
> hedge: "a speculatively-entered-then-backtracked entry **/** a memoized 'success' coverage delta").
> This slice closes that `/`. ([[feedback_why_and_where_before_solution]],
> [[feedback_tools_first_no_guessing]], [[feedback_no_codebase_change_without_tool_backed_facts]],
> [[feedback_never_edit_generated_artifacts]].)

## Baseline (reproduced this session — deterministic ⇒ signal)

```
./target/debug/ast_pipeline ../grammars/systemverilog.ebnf --report-certificate-coverage \
  --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0
# total=1292 proof=1 witness=1203 UNKNOWN=88 fully_certified=false (sample_parse_failures=0)
```

Byte-identical to the layer-A pointer. `context_member_method_call` (rule id `547`) is in the
88-rule `UNKNOWN` set on the SHIPPED grammar — the gap is **latent** there (no `has_fact` prelude
exists, so the witness pass never generates a degenerate predicate-passing render for it).

## The gap, reproduced on a PLAIN input (doctrine-clean, no prelude)

The `-0102` false witness needed the reverted prelude to establish the `variable_binding` fact. This
slice removes that dependency: a **real declaration** `int \foo ;` emits the same fact, so the
`@predicate has_fact(variable_binding, $head)` post-gate of `context_member_method_call` passes
naturally. Two crafted inputs, exercised through the witness oracle
`parser_registry::parse_and_cover_systemverilog` (a throwaway source-level test, since reverted) and
the AST genuineness oracle (`parseability_probe --parse-dump-ast-pretty`, grep
`context_member_method`):

| input | parses | `context_member_method_call` ∈ `exercised_rule_names` | `context_member_method` AST nodes |
|---|---|---|---|
| **bare-ref** `int \foo ; (*\foo =+\foo .\foo .\foo *);` | ✅ | **TRUE — FALSE WITNESS** | **0** |
| method-call `int \foo ; (*\foo =+\foo .\foo [0].\foo ()*);` | ✅ | TRUE (genuine) | 1 |

The bare-ref input is a clean, prelude-independent reproducer (and the future regression lock): the
rule is credited as witnessed while the accepted parse contains **zero** of its nodes. This violates
the book's cert-coverage soundness invariant ("the entries that survive are **exactly the rules of
the accepted parse**").

## WHY — the mechanism (tools-first, `parseability_probe --trace-rules`)

`--trace-rules context_member_method_call` on the bare-ref input (pre-built binary, no edits) shows,
at the `\foo .\foo .\foo` position:

1. the body's `( dot identifier constant_bit_select &dot )+ dot callable_method_call_body` tail
   fails (`dot` cached-failure → `Speculative parse failed … rule=context_member_method_call`), yet
2. **`💾 Memoized successful result for rule 547 at position 36`** — the body nonetheless memoizes a
   **structurally-degenerate success** (a shorter render), then
3. **`has_fact(kind=variable_binding, name="\foo") → true`** — the POST-predicate passes (the binding
   from the real `int \foo ;`), so `context_member_method_call` returns `Ok`, and
4. the caller chain in the very next events moves on to **sibling** alternatives
   (`call_with_postfix_chain → direct_callable_method_call → …`) — the committed parse routes the
   bytes elsewhere, so `context_member_method_call` is absent from the AST.

The trace's `has_fact` caller chain is the exact reach path:
`systemverilog_file → source_text → source_text_item → description → … → attribute_instance →
attr_spec → constant_expression → constant_expression_operand → constant_primary →
constant_primary_sv_2017 → constant_function_call → call_primary → context_member_method_call`.

This is the **memoization × transactional-coverage composition class** (`H.10.2.2` /
`SV-EXH-PROOF.3.3.4.b.6.2.36.4`), in a **new trigger**: a predicate-gated rule whose result is
discarded by the committed parse but whose **coverage push outlives the discard**, leaking through an
ancestor's memo `coverage_delta`.

## WHERE — the leak legs in the ENGINE SOURCE (read, not guessed)

The transactional coverage record has exactly three legs in the codegen
(`rust/src/ast_pipeline/ast_based_generator.rs`, emitted verbatim into every generated parser):

1. **entry push** (`:2701-2702`) — `if self.coverage_enabled { coverage_stack.push(RULE) }` at rule
   entry, gated on `coverage_enabled`.
2. **`try_parse` truncate** (`:6128` save `saved_coverage_len`, `:6209` truncate on speculation
   failure) — the only leg that REMOVES entries.
3. **`memoized_call` capture + replay** (`:6374` `memo_coverage_checkpoint`, `:6383-6384` capture
   `coverage_stack[checkpoint..]` into `MemoEntry.coverage_delta` on body success, `:6354-6356`
   replay `extend_from_slice` on a memo hit) — the `H.10.2.2` fix that keeps the record complete.

`with_semantic_runtime_rule_transaction` (`:1538-1885`, also emitted verbatim) manages **only**
`semantic_runtime_state`: its post-predicate-reject path (`:1835` `Err(Backtrack)`) and its error
restore (`:1874-1878` `rollback_to_named` + `:1883` `pop_rule_context`) roll back facts + the rule
context but **never touch `coverage_stack`**. So a success-then-reject inside a rule body relies
entirely on an ancestor `try_parse` to truncate the leaked coverage push.

**The pinned leak.** `context_member_method_call`'s coverage push (leg 1) is captured into the frozen
`coverage_delta` of an ancestor memo entry (leg 3 capture) while it is still on the live stack — i.e.
the ancestor's body succeeded and memoized *before* the discard of `context_member_method_call`'s
result truncated its push. The carrier is **`attribute_instance`** (the innermost ancestor on the
reach chain that memoizes a success and is later **memo-hit on the committed path**): its replay
(leg 3 replay) re-introduces the stale `context_member_method_call` id onto the live stack, where it
survives to parse-end and into `exercised_rule_names`. The asymmetry is the bug: the ancestor's
**result node does not contain `context_member_method_call`** (it was discarded), but its
**`coverage_delta` does** (the push was never truncated). The `H.10.2.2` replay-transactionality fix
faithfully reproduces a delta that was already wrong **at capture time** — so it cannot save us here.

### Root cause, stated precisely

A coverage push is removed ONLY by `try_parse` truncation (leg 2). When a rule's *result* is rejected
**after** a structural success — via the post-predicate-reject path, or by being a committed
ordered-choice branch that a higher mandatory context then abandons — and the discard does not pass
through a `try_parse` that truncates *before* an enclosing rule memoizes, the push is frozen into that
enclosing rule's `coverage_delta` and replayed on the committed path. **A non-committing,
predicate-passing (or otherwise discarded) rule must never be witnessed; today it can be.** This is a
soundness gap that protects EVERY witness number, not just `context_member_method_call` — it is
latent on the shipped grammars only because none currently generate a degenerate predicate-passing
render in the witness pass.

## FIX direction (handed to `.4.2.1.2.1` FIX — the next slice; NOT done here)

Close the truncation gap so a discarded rule's coverage push cannot survive into an ancestor memo
delta. Candidate (to be settled tools-first, change ONE thing, measure GLOBAL cert + spf at seeds
0/7/42 for EVERY wired grammar, fully-certified roster byte-identical):

- **(a)** truncate `coverage_stack` to the rule's entry length on the
  `with_semantic_runtime_rule_transaction` reject/error path (the post-predicate-reject and IIFE-`Err`
  legs at `:1835`/`:1874`), symmetric with `try_parse` — so a predicate-rejected success cannot leak;
  and/or
- **(b)** at memo capture (`:6383`), exclude from `coverage_delta` any coverage id whose contributing
  rule's result is not in the memo's output node (a deeper, structural fix).

(a) directly closes the post-predicate-reject leak; whether it fully closes the bare-ref case
(where `context_member_method_call`'s OWN post-predicate PASSES and the discard happens in an ANCESTOR)
must be verified — the FIX slice will re-derive the exact discard via **codegen** instrumentation
(`ast_based_generator.rs` + `make focus_systemverilog`), never by editing `generated/*.rs`
([[feedback_never_edit_generated_artifacts]]).

## Genuineness oracle (durable)

For a predicate-gated rule, the cert-coverage `UNKNOWN`/`witness` delta is necessary but NOT
sufficient. The genuineness oracle is the construct's **committed AST node**
(`--parse-dump-ast-pretty`, grep the rule's distinguishing `kind`) or a positive success-exit in
`--trace-rules <rule>` — never the count alone. KM:
[sv-cert-coverage-predicate-gated-false-witness](../knowledge/sv-cert-coverage-predicate-gated-false-witness.md),
[memo-hit-transactional-replay](../knowledge/memo-hit-transactional-replay.md).

## Artifacts (this session, scratch — not tracked)

`rust/target/generated_logs/h1255334_42121/` — `sv_bare.sv` / `sv_bare.ast.json` (the false-witness
reproducer, 0 `context_member_method` nodes), `sv_method.sv` / `sv_method.ast.json` (genuine, 1 node),
`cmmc_trace.txt` (`--trace-rules context_member_method_call` showing the memoized-success +
post-predicate-pass + sibling re-route).
