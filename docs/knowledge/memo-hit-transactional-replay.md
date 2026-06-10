---
id: memo-hit-transactional-replay
title: "A packrat memo HIT bypasses the rule body — EVERY transactional per-rule record must be delta-captured in the MemoEntry and REPLAYED on the hit (semantic store AND coverage record), or it silently goes incomplete"
answers:
  - "why is a rule UNKNOWN in cert-coverage when the accepted parse clearly contains its construct"
  - "probe parses and the AST shows the construct but witnessed_target=false — why"
  - "does a memo hit replay the coverage record / exercised_rule_names"
  - "why does exercised_rule_names miss rules the accepted parse exercised"
  - "memoization composes wrongly with a transactional record — what is the fix pattern"
  - "what is the memo-hit coverage-delta replay"
  - "what records does memoized_call replay on a cache hit"
  - "adding a new transactional per-rule record to generated parsers — what must I not forget"
date: 2026-06-10
status: current
tags: [packrat, memoization, coverage, cert-coverage, witness, engine, parser-agnostic, gotcha, grammar-wellformed]
evidence: "GRAMMAR-WELLFORMED.H.10.2.2 (tools-first): regex cert-coverage probe `\\Q\\A\\E*` reported parsed=true witnessed_target=false for letter_no_upper_e, yet its AST dump shows atom [\"\\\\\",\"A\"] — the accepted parse DOES route through quoted_literal_escaped_char → letter_no_upper_e. Mechanism: piece_quoted_run_quantified's quoted_run_inner_piece* speculation enters quoted_literal_char(pos 2)→…→letter_no_upper_e (coverage pushed AND memoized as successes), the !\"\\E\" lookahead fails → try_parse truncates the coverage stack; the trailing quoted_literal_char slot re-calls at the SAME (rule,position) key → memo HIT returns the cached node WITHOUT re-entering the body → the per-rule-entry coverage push (ast_based_generator.rs ~:2701) never fires. Same composition-gap class as SV-EXH-PROOF .b.6.2.36.3/.36.4 (semantic delta), now on the coverage stack. FIX: MemoEntry gains coverage_delta: Option<Vec<u32>> (mod.rs ~:790); memoized_call snapshots coverage_stack.len() before f(self), stores the pushed slice on success (None when coverage disabled — zero cost), and on every hit extends the live stack with the cached delta (inside the current speculation → later rollback still truncates → transactionality preserved). Live-fire lock: parser_registry::tests::regex_parse_and_cover_replays_coverage_on_memo_hits. Codegen lock: transactional_parse_coverage_wiring_is_emitted_at_codegen."
reverify: "grep -n 'coverage_delta' rust/src/ast_pipeline/mod.rs rust/src/ast_pipeline/ast_based_generator.rs; cd rust && cargo test --lib --features generated_parsers regex_parse_and_cover_replays_coverage_on_memo_hits"
---

## The gotcha

Generated parsers keep **transactional per-rule records** — state pushed on rule entry and
truncated by `try_parse` when a speculation fails. Two exist today: the semantic store
(`@emit_fact` / scopes) and the opt-in parse-coverage record (`enable_coverage` /
`exercised_rule_names`, the certifying-linter witness side).

Packrat memoization breaks BOTH the same way: a memo **hit** returns the cached result
**without re-entering the rule body**, so entry-time effects never fire on the hit path. A
subtree first parsed inside a speculation that later fails (its record entries rolled back)
and then memo-hit on the committed path is **silently absent** from the record — the record
stays *sound* (nothing rolled-back survives) but goes *incomplete*.

- The semantic-store instance was pinned and fixed by `SV-EXH-PROOF.3.3.4.b.6.2.36.3/.36.4`
  (`MemoEntry.semantic_delta`, replayed on hit).
- The coverage-record instance was pinned and fixed by `GRAMMAR-WELLFORMED.H.10.2.2`
  (`MemoEntry.coverage_delta`, replayed on hit) — found live as regex cert-coverage
  `UNKNOWN` rules whose probes parsed with the construct visibly in the AST.

## The fix pattern (apply to ANY future transactional record)

When adding a new per-rule transactional record to generated parsers, it is **not enough**
to push on rule entry and truncate in `try_parse`. You must also:

1. snapshot the record's extent in `memoized_call` **before** executing the rule body,
2. store the body's delta in the `MemoEntry` on success (gate the capture on the record's
   enable flag so disabled parsing pays nothing),
3. **replay the delta on every memo hit** — inside the current speculation, so a later
   rollback still truncates it (transactionality preserved).

The symptom of forgetting step 3: "the accepted parse clearly contains construct X, but the
record says X never happened" — e.g. a cert-coverage probe that is `parsed=true,
witnessed_target=false` while its AST dump visibly contains the target rule's production.

## See also

- [[prove-rule-dead-or-reachable]] — adjudicating UNKNOWN: dead rule vs generator-reach gap
  (this card adds the third cause: a *witness-record* gap, neither dead nor unreachable).
- [[grammar-linter-trustworthiness]] — the certificate/witness model this record feeds.
- The book's Grammar Well-Formedness chapter, "watch it hit the fragment" section, documents
  the memo-hit replay as part of the witness-record guarantee.
