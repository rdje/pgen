# Checkpoint duplicate-word removal — design + pre-registration

Task: `PGEN-RGX-0078-0199` (leaf `RGX-0078.5.j.4`), session #174, 2026-07-20.
One implementation fix only, per the held-carrier execution contract
(`docs/tasks/artifacts/held_carrier_batch/execution_contract.md`) and the
standing directive
`docs/decisions/feedback_regex_fix_geomean_ratchet_and_fresh_session.md`.

## 1. The measured target (held-carrier composition, SHA-pinned)

`SemanticRuntimeCheckpoint` is a 7-word `Copy` value constructed on every
`checkpoint()` and copied by value into every rollback/extract call. The
`-0148` profile pinned the per-attempt construct/copy/drop of the checkpoint
family at 8.1% of the corpus-MAX cell (~20,490 rollbacks/parse, 100% taking
the fast path); the K4a slice (`-0142`-era) already shrank it from the ~88-B
non-`Copy` struct to 7 words. The held-carrier composition prices the residual
duplicate word at:

```
checkpoint_duplicate  0.859310862  exact-current  six-word checkpoint
```

(`docs/tasks/artifacts/held_carrier_batch/held_carrier_batch.txt`, banked
executable output, SHA-custodied by `compose_held_batch.py`.)

## 2. WHY the word is a duplicate (mutation-site audit, this session)

`scopes` and `active_chain` are maintained in lockstep at EVERY mutation
site, so `scopes.len() == active_chain.len()` is a state invariant and the
checkpoint words `scope_len` and `chain_len` are always equal:

| site | scopes | active_chain |
|---|---|---|
| `new()` / `reset_for_new_parse` (`semantic_runtime.rs:2733-2748`) | clear + push root frame (len 1) | clear + push `ScopeId::ROOT` (len 1) |
| `open_scope` (`:3593`/`:3596`) | push | push |
| `close_scope` (`:3619`/`:3626`, shared root guard `scopes.len() <= 1`) | pop | pop |
| rollback slow path (`:3432-3466`) | rebuilt as exactly one frame per chain node | trail-unwound to `chain_len` (asserted `:3445`) |
| `apply_delta` (`:3197-3201`) | `delta.final_scopes` | `delta.final_active_chain` — both cloned from one state instant in `extract_delta_since` |

The lockstep is also documented invariant prose at `:3402-3415` ("`open_scope`
pushes BOTH; `close_scope` pops BOTH") and the paired
`debug_assert_eq!(self.scopes.len(), checkpoint.scope_len)` /
`debug_assert_eq!(self.active_chain.len(), checkpoint.chain_len)` lines
re-verify it on every fast-path rollback across the whole debug test battery.

`scope_len` has ZERO occurrences outside `semantic_runtime.rs` — no other
Rust source, no generated artifact (`grep -rn scope_len rust/ generated/`).

## 3. The fix (LIB-ONLY; fix-hierarchy level: engine/runtime)

- Remove the `scope_len` field; the checkpoint becomes 6 words.
- The PUBLIC accessor `scope_len()` is preserved and returns `self.chain_len`
  — the same value by the invariant (additive-API discipline; zero external
  callers today, but public surface).
- The two `scopes`-side debug asserts and `commit()` compare against
  `checkpoint.chain_len` (same value; the asserts now double as the
  mechanical lockstep tripwire).
- 7-word doc comments re-pinned to 6 words.
- One focused test pins `scope_len()`/`chain-depth` lockstep across
  open/close/checkpoint/rollback rounds.

No emitter change, no regen train: both A/B probes embed the SAME generated
regex artifact (`f85f2121…`, the `-0198` vintage).

## 4. Pre-registered measurement protocol

- Immediate baseline: the current release probe, built from the clean
  `-0198` HEAD tree (`ec13e1d1`), byte-identical to the preserved
  `preserved_probes/regex_perf_probe_g3reset_0a7346ce`
  (`0a7346ce3560635a…`, verified by `cmp` this session). Copied to scratch +
  SHA banked BEFORE any source change.
- Candidate: `cargo build --release --features "generated_parsers
  mimalloc_perf" --bin regex_perf_probe` (fat LTO, codegen-units 1 — the
  standard closure-bench config) after the lib change.
- Corpus: `regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_cases.jsonl`
  (2,189 cells), probe-default adaptive sampling, full per-cell JSONL banked.
- Bench: 3 base floor-validation rounds, then 5 rounds × 2,000 samples
  (200 warmup) per side, order alternated per round. Steering only.
- Custody: memory guard (16384 MB budget, 10% floor, disk floor default),
  `caffeinate -i`, serialized single-heavy-runner, idle host; custody asserts
  the base probe SHA, the UNCHANGED regex artifact SHA on both sides, and
  candidate-probe mtime newer than `semantic_runtime.rs`.
- **Acceptance (binding, `-0197` ratchet):**
  1. unrounded candidate corpus geomean **strictly <** unrounded base corpus
     geomean from the same session;
  2. zero verdict flips across all 2,189 cells;
  3. candidate corpus MAX ≤ **483,583 ns**.
  Equal-or-higher geomean, any flip, or a MAX breach ⇒ unconditional product
  reversion; only the evidence/rejection record commits. Bench rounds are
  steering/reporting only. Floor-validation sanity: base bench geomean within
  ±6% of the banked ≈1,937.4 ns, else the sweep is invalidated for custody
  (rerun), not adjudicated.
- Honest pre-registration note: the priced effect (≈0.86 ns on a ≈1,228 ns
  geomean ≈ −0.07%) is far inside the observed session noise span (≈2.3%).
  Per the directive, noise affects confidence reporting, never direction: the
  same-session strict compare adjudicates, and an unfavorable valid
  measurement REVERTS the product change.

## 5. Reproduction

```sh
bash docs/tasks/artifacts/checkpoint_word/battery.sh        # correctness battery
bash docs/tasks/artifacts/checkpoint_word/run_ab.sh         # pre-registered A/B
python3 docs/tasks/artifacts/checkpoint_word/analyze_ab.py  # adjudication
```
