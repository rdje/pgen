# In-place semantic-runtime reset — design + pre-registration

Task: `PGEN-RGX-0078-0198` (leaf `RGX-0078.5.j.4`), session #173, 2026-07-20.
One implementation fix only, per the held-carrier execution contract
(`docs/tasks/artifacts/held_carrier_batch/execution_contract.md`) and the
standing directive
`docs/decisions/feedback_regex_fix_geomean_ratchet_and_fresh_session.md`.

## 1. The measured target (from `-0178`, custody-pinned)

The generated `prepare_parse_state` runs before every parse and performs a
whole-state reconstruction. The four cumulative call sites inside the closure
timer price at a conservative **22.563 ns** (1.7859% of the 1,263.4 ns corpus
geomean floor), a lower bound that excludes the inline clear/copy/assign
instructions of the same ceremony:

| mechanism | source |
|---|---|
| `facts().to_vec()` snapshot | generated `prepare_parse_state` |
| `SemanticRuntimeState::new()` replacement | same |
| `drop_in_place<SemanticRuntimeState>` of the old state | same |
| `clone_predicate_defs()` fresh clone | same |

Evidence: `docs/tasks/artifacts/g3_setup_teardown_pricing/` (binary custody on
the preserved floor probe `1d3fa0ee`, disassembly-pinned return offsets
+3184/+3192/+3200/+3420).

## 2. Exact-equivalence specification (WHY the shape of the fix)

`semantic_runtime_state_mut()` is public on every generated parser, so a
first-parse bypass is unsound: callers may have mutated any state category
before `parse()`. The compatible fix is an in-place reset that reproduces the
old ceremony's post-state **exactly**.

Source-derived post-ceremony state (old ceremony = snapshot facts → fresh
`new()` state → `push_fact_record` replay in order → `set_predicate_defs`
(fresh clone of the annotations table)):

| field | post-ceremony value |
|---|---|
| `scopes` | `[SemanticScopeFrame { kind: Global, name: None }]` |
| `facts` | same records, same order, every record re-stamped `scope_depth=0`, `scope_id=ScopeId::ROOT` (`push_fact_record` re-bases both on every replay — `semantic_runtime.rs:3643-3644`) |
| `fact_index` | fresh rebuild: one `insert(kind, 0, name, position)` per fact, positions ascending |
| `scope_arena` | `[root ScopeNode]` (id ROOT, no parent, Global, open, depth 0) |
| `active_chain` | `[ScopeId::ROOT]` |
| `chain_trail` | empty |
| `predicate_defs` | content-equal to the compiled annotations table (the old ceremony **overwrites** whatever was in the state — preserved semantics would be a behavior change) |
| `counters` | default except `facts_imported = facts.len()` (each replayed record bumps it — `:3649`) |
| `write_epoch` | `facts.len()` (each replayed record bumps it — `:3642`) |
| `current_rule_context_stack` | empty |
| `deferred_obligations` | empty |
| `memo_hit_counts` | empty |

The new primitive `SemanticRuntimeState::reset_for_new_parse(&mut self,
predicate_defs: &HashMap<String, PredicateDef>)` reproduces exactly this
table in place (allocation-reusing `clear()`/`clone_from`, no state
construction, no state drop, no facts snapshot). The generated
`prepare_parse_state` emission replaces the four-step ceremony with the single
call, passing `self.semantic_runtime_annotations.predicate_defs_map()` (new
borrow accessor). Additive API only — previously generated parsers keep
compiling against the lib (the `.5.i.1.t1` vintage rule).

`SemanticRuntimeState` derives `PartialEq`, so the focused tests assert
full-state equality between (a) the old ceremony replayed verbatim and (b)
the new primitive, after mutating every publicly reachable state category
(facts via emit/import, scopes open/close, checkpoints/rollbacks growing the
chain trail, rule-context pushes, memo-hit records, deferred obligations,
predicate-def drift via public `set_predicate_defs`), plus repeated resets
and the empty-state fast path.

## 3. Fix-hierarchy level

Engine/runtime + emitter (level 5): the ceremony is emitter-owned engine
interaction; no annotation/store/grammar lever can express an in-place reset.

## 4. Pre-registered measurement protocol

- Immediate baseline: probe built from the clean HEAD tree
  (`255809e7`, regex artifact `e4924024a4bf7a91…`) with
  `cargo build --release --features "generated_parsers mimalloc_perf" --bin
  regex_perf_probe` (fat LTO, codegen-units 1 — the standard closure-bench
  config). SHA recorded in `custody.txt` before any timed run.
- Candidate: same command after the lib+emitter change and the canonical
  all-11 regen train; embeds the regenerated regex artifact (SHA recorded in
  `custody.txt`).
- Corpus: `regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_cases.jsonl`
  (2,189 cells), probe defaults for adaptive sampling
  (`--slow-cell-threshold-ms 100`, `--slow-cell-samples 5`,
  `--giant-cell-threshold-ms 2000`), full per-cell JSONL banked.
- Bench: 3 base floor-validation rounds, then 5 rounds × 2,000 samples
  (200 warmup) per side, order alternated per round.
- Custody: memory guard (16384 MB budget, 10% floor, disk floor default),
  `caffeinate -i`, serialized single-heavy-runner, idle host.
- **Acceptance (binding, `-0197` ratchet):**
  1. unrounded candidate corpus geomean **strictly <** unrounded base corpus
     geomean from the same session;
  2. zero verdict flips across all 2,189 cells;
  3. candidate corpus MAX ≤ **483,583 ns**.
  Equal-or-higher geomean, any flip, or a MAX breach ⇒ unconditional product
  reversion; only the evidence/rejection record commits. Bench rounds are
  steering/reporting only — they cannot override the corpus verdict in either
  direction. Floor-validation sanity: base bench geomean within ±6% of the
  banked ≈1,937.4 ns, else the sweep is invalidated for custody (rerun), not
  adjudicated.

## 5. Reproduction

```sh
bash docs/tasks/artifacts/inplace_reset/regen_train.sh    # canonical all-11 regen
bash docs/tasks/artifacts/inplace_reset/battery.sh        # correctness battery
bash docs/tasks/artifacts/inplace_reset/run_ab.sh         # pre-registered A/B
python3 docs/tasks/artifacts/inplace_reset/analyze_ab.py  # adjudication
```
