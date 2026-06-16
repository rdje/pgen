# GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.1.2 — `has_fact` semantic-prelude FIX ATTEMPT → REFUTED (false-witness discovery)

Tools-first implementation of the `-0101` design for witnessing the `has_fact`-gated rule
`context_member_method_call`, and the tools-first verification the design itself mandated
("dump the armed sample to confirm decl + `.method()`"). That verification **REFUTES** the
prelude-alone approach: the implemented prelude makes the cert-coverage report a **FALSE
witness**, so it was **REVERTED**. This slice (`PGEN-GRAMMAR-WELLFORMED-0102`) is therefore a
WHY+WHERE refutation + a foundational cert-coverage soundness-gap discovery, **PURE-DOCS** in its
landed form (the code change was implemented, measured, proven a false witness, and reverted —
nothing code/grammar/generated/release/schema/ledger lands).

> Scope: layer-B detail behind the `H.12.5.5.3.3.4.2.1.2` row. Opened by `.4.2.1.1` (`-0101`),
> which locked a design and explicitly handed the FIX a composition question to settle tools-first
> ("dump the armed sample; confirm it carries decl + `.method()`"). Doing exactly that surfaced a
> deeper problem than the open question anticipated.
> ([[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]],
> [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_always_signoff_decisions]],
> [[feedback_be_alert_root_cause_fishy_immediately]].)

## Baseline (reproduced this session — deterministic ⇒ signal)

```
./target/debug/ast_pipeline ../grammars/systemverilog.ebnf --report-certificate-coverage \
  --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0
# total=1292 proof=1 witness=1203 UNKNOWN=88 fully_certified=false (sample_parse_failures=0)
```

(DEBUG `ast_pipeline` built `--features ebnf_dual_run,generated_parsers` — the SV grammar input
needs `ebnf_dual_run` and the cert-coverage witness side needs `generated_parsers`.) Byte-identical
to the post-`-0101` layer-A pointer; `context_member_method_call` is in the 88-rule `UNKNOWN` set.

## What was implemented (then reverted)

The `-0101` design, faithfully, in `rust/src/ast_pipeline/stimuli_generator.rs` (generator-only):

1. `gen_has_fact_gates: HashMap<String, Vec<(String,String)>>` (rule → `(kind, name-ref)`),
   computed by a new `compute_gen_has_fact_gates` from each `has_fact(K, $rule_ref)` **post**-predicate
   (the `has_fact` analogue of `gen_count_kinds`; does NOT activate `store_aware_gen`).
2. A `PreludeKind` discriminator on `ReachPrelude` (`Count` = the existing two-phase
   capture/replay; `Presence` = armed at `iterations = 1`, no capture, no replay), and `Count`-only
   guards on `reach_prelude_capture` / `reach_prelude_replay_text` / the count-prune bypass.
3. A `has_fact` branch in `compute_reach_prelude` (factored with the count branch into a shared
   `build_semantic_prelude`): when a `has_fact`-gated rule is on the reach path, build a prelude
   whose producer is the first sorted rule whose `@emit_fact` emits `K`, hosted at the innermost
   on-path quantifier whose body graph-reaches that producer, armed `iterations = 1`.

It **compiled clean** and the seed-0 metric moved exactly as the row predicted:
`UNKNOWN 88 → 87`, `witness 1203 → 1204`, `spf=0` — `context_member_method_call` removed from the
`UNKNOWN` set. **By the headline number alone the leaf looked done.** The design's own discipline
("dump the armed sample") is what caught that the number was a lie.

## WHY+WHERE — the witness is FALSE (the parser is the judge, tools-first)

The cert-coverage witnessed `context_member_method_call` from the **plannable** pass (not the
target-own pass the design expected). Captured the exact armed sample at seed 0:

```
(*\foo =+type(struct{struct{struct{struct{bit\foo ;}\foo ;}\foo ;}\foo ;})*)(*\foo =+\foo .\foo .\foo *);
```

It carries the prelude's binding (`bit \foo ;` inside the first attribute's type) — but the chain
the rule is supposed to witness is `\foo .\foo .\foo`, a **bare hierarchical reference with no
`()`**. `context_member_method_call := identifier ( dot identifier constant_bit_select &dot )+ dot
callable_method_call_body …` REQUIRES a real `callable_method_call_body` (a `built_in_method_call`
or `method_identifier … lparen … rparen`). The sample has no method call at all. Three independent
tools agree it is not a genuine witness:

1. **AST node count.** `parseability_probe --parse-dump-ast-pretty` on the armed sample →
   **0** `context_member_method` nodes (`call_primary := context_member_method_call -> {kind:
   "context_member_method", …}` sets that kind explicitly, so a committed match WOULD show it). The
   design's genuinely-witnessing **Test c2** (`int \foo ; (*\foo =+\foo .\foo [0].\foo ()*)`, a real
   `.\foo()`) → **1** node. So the armed sample does not demonstrate the construct; Test c2 does.
2. **Execution trace.** `--trace-rules context_member_method_call` on the armed sample shows EVERY
   event is a `Speculative parse failed / backtracked` (at positions 13, 20, 95, 101) or a failed
   inner `built_in_method_call` keyword (`unique`/`and`/`or`/`xor`/`randomize`/`with`); there is
   **no positive success-exit** anywhere. The rule never commits.
3. **The committed AST omits it.** `parse_full` passes via OTHER rules; `call_primary` never yields a
   `context_member_method` result, so the rule is not in the accepted parse.

So the `UNKNOWN 88→87` was a **false certification**: a rule credited as witnessed by a sample whose
accepted parse never enters it.

### WHERE the false witness comes from (the soundness gap)

`RULE_CONTEXT_MEMBER_METHOD_CALL = 547` (`generated/systemverilog_parser.rs:634`). The generated
`parse_context_member_method_call` pushes its coverage id at rule ENTRY
(`self.coverage_stack.push(RULE_CONTEXT_MEMBER_METHOD_CALL)` @ `:449256`), then runs the body inside
`with_semantic_runtime_rule_transaction → memoized_call`. The trace shows
`Memoized successful result for rule 547 at position 101` together with
`has_fact(variable_binding, "\foo") → true`, even though the rule structurally fails (no method
call) and is not in the committed AST. The mechanism:

- WITHOUT the prelude (baseline / shipped state) the gate `has_fact(variable_binding, $head)`
  **fails** (no binding in scope), so `context_member_method_call` rejects at the post-predicate and
  is correctly NOT witnessed — the latent gap is never triggered.
- WITH the prelude the binding exists, so the post-predicate **passes** on the plannable pass's
  **degenerate (no-`.method()`) render**; the rule then fails structurally — but its coverage
  contribution (a speculatively-entered-then-backtracked entry / a memoized "success" coverage delta
  on a speculation the parent discards) **survives into the witness record**. This is the same
  memoization × transactional-coverage composition class as `H.10.2.2`/`.b.6.2.36.4`, here on a
  **post-predicate-passes-then-structurally-fails** path the existing truncation/replay does not
  cover.

This violates the cert-coverage soundness contract the whole sign-off model rests on (the book's
"trusting the linter: certificates, not faith" — *surviving coverage entries = rules of the accepted
parse*). The prelude exposes it; it is latent in the shipped grammar (no `has_fact` prelude exists
today, so no degenerate predicate-passing render is ever generated for a `has_fact`-gated rule).

## Decision — REVERT, do not land a false witness

Per [[feedback_always_signoff_decisions]] ("when a result can't be verified [as genuine], the
signoff decision is to STOP + checkpoint, NOT proceed") and the cert-coverage soundness contract, the
prelude code was **reverted** so the SV cert number stays honest at `UNKNOWN=88`. Verified by rebuild:
post-revert seed-0 = `total=1292 proof=1 witness=1203 UNKNOWN=88 spf=0` (byte-identical to baseline).

Net measured effect of the reverted change had been exactly **one false witness**
(`context_member_method_call`); no other `has_fact`-gated rule flipped (the broad `gen_has_fact_gates`
map is parser-agnostic but only this rule's reach path + degenerate render hit the gap at seed 0).

## REFUTATION of the `-0101` premise

`-0101` concluded "a top-level binding-producer prelude satisfies the gate (Test c2), name-coupling
is free" and handed the FIX an open *composition* question. The tools-first FIX verification shows the
premise is **too optimistic for the plannable route**: a binding prelude WITHOUT the mandatory
`.method()` structure does not merely "fail to witness" (as Test a does WITHOUT a binding) — once the
binding makes the post-predicate pass, it **false-witnesses** the degenerate render. The genuine
witness (Test c2) requires the method-call structure AND the binding in ONE sample, and the
cert-coverage must not credit the degenerate render. So the leaf cannot land as a simple generator
prelude; it needs the deeper work below.

## Re-scope (children of `.4.2.1.2`, each leaf-first, tools-first)

- **`.4.2.1.2.1` — cert-coverage witness-soundness gap (ENGINE, foundational, higher priority).**
  A rule whose post-predicate passes but whose body then structurally fails (and which the committed
  parse does not enter) must NOT be witnessed. Investigate the exact leak (the `memoized_call`
  coverage-delta vs the `try_parse` truncation on the post-predicate-pass-then-fail path; rule 547's
  "Memoized successful result at position 101" while absent from the committed AST). This protects the
  trustworthiness of EVERY witness number, not just this rule. WHY+WHERE first; change ONE thing;
  measure GLOBAL cert + spf at seeds 0/7/42 for every wired grammar; the fully-certified roster must
  stay byte-identical.
- **`.4.2.1.2.2` — `has_fact` prelude composed with mandatory method-call-structure forcing
  (GENERATOR), after `.2.1`.** Re-introduce the prelude AND force the gated rule's mandatory
  `callable_method_call_body` child (the design's option (b) / the `.3.3.1` mandatory-child lineage)
  so the witnessing sample is Test-c2-shaped (`int \foo ; … \foo.\foo[0].\foo()`) — a GENUINE witness
  (AST node present, rule committed). Dump the armed sample AND verify a `context_member_method` AST
  node BEFORE trusting the `UNKNOWN` delta — the AST-node check is the genuineness oracle the headline
  count cannot provide.

## Lesson (durable)

The cert-coverage `UNKNOWN` count is necessary but **not sufficient** evidence that a rule is
genuinely witnessed when a **predicate-gated** rule is involved: a prelude that satisfies the
predicate on a structurally-degenerate render can drop the count via a false witness. The genuineness
oracle is the **AST node** for the construct (or a positive success-exit in the trace), not the
count. New KM card [sv-cert-coverage-predicate-gated-false-witness](../knowledge/sv-cert-coverage-predicate-gated-false-witness.md).

## Artifacts (this session, scratch — not tracked)

`rust/target/generated_logs/h1255334_4212/` — `witness.sv` (the armed false-witness sample),
`witness.ast.json` (0 `context_member_method` nodes), `testc2.sv`/`testc2.ast.json` (Test c2, 1
node), `trace.txt`/`trace2.txt` (`--trace-rules context_member_method_call`), `probes.txt`
(`PGEN_CERT_COVERAGE_DEBUG_PROBES` dump).
