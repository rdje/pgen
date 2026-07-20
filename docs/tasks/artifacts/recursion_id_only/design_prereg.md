# Generated ID-only recursion-name path — design + pre-registration

Task: `PGEN-RGX-0078-0200` (leaf `RGX-0078.5.j.4`), session #175, 2026-07-20.
One implementation fix only, per the held-carrier execution contract
(`docs/tasks/artifacts/held_carrier_batch/execution_contract.md`) and the
standing directive
`docs/decisions/feedback_regex_fix_geomean_ratchet_and_fresh_session.md`.

## 1. The measured target (held-carrier composition, SHA-pinned)

Three SHA-pinned classifiers price the duplicate human-readable recursion-name
carrier as ONE representation unit (the `-0197` dependency map: "fixed
entry/exit + rollback + growth are ONE representation fix, measured TOGETHER
— never three code fixes"):

```
name_fixed     3.940409443  exact-current  ID-only generated path + reconstruction   (-0188)
name_rollback  0.149032382  exact-current  ID-only generated path + reconstruction   (-0189)
name_growth    4.005065577  exact-current  ID-only generated path + reconstruction   (-0190)
                8.094507402  ns unit total
```

(`docs/tasks/artifacts/held_carrier_batch/held_carrier_batch.txt`, banked
executable output, SHA-custodied by `compose_held_batch.py`; the underlying
site-level proofs are banked in `docs/tasks/artifacts/recursion_guard_expansion/`,
`docs/tasks/artifacts/try_parse_name_rollback/`, and
`docs/tasks/artifacts/recursion_name_growth/`.)

## 2. WHY the name stack is a duplicate (banked audits + this session's source verification)

C1 (`-0118`) made `RecursionGuard.rule_id_stack` (16-byte `(RuleId, position)`
frames) the complete modern cycle-check representation — `check_cycle_id`
scans it, positions travel with it, and a parser's `RuleId`↔rule-name mapping
is a bijection through the generated `RULE_NAMES` table (rule id = table
index, `ast_based_generator.rs:1191`). The parallel `parse_stack` (24-byte
`(&'static str, position)` frames) is maintained in lockstep at every push/
pop/truncate purely for name READERS, all of which are cold or trace-gated:

1. legacy `check_cycle` (bootstrap-emitter parsers only — out of scope);
2. `check_cycle_id`'s maximum-depth arm (`MutualRecursive.rules` payload —
   a once-per-parse ceiling event; the generated match arms bind `depth`
   only and never read `rules`);
3. generated `try_parse`'s rollback-label capture (`parse_stack.last()`),
   consumed ONLY inside a trace-enabled branch
   (`RollbackLabel::materialize`, `semantic_runtime.rs:2501-2503`);
4. the DBG rule-stack log (gated `trace_enabled(Debug)`) and
   `create_contextual_error`'s rule-name vector (`ast_based_generator.rs:8661`).

On the bare fused path (`bare_parse == true`, the corpus/bench population:
no coverage, no trace, no counters observer) every one of those readers is
dead or reconstructible, yet the path still pays: paired name pushes at the
cascade internal frame (`cascade.rs:657`), paired pops (`:663`), the
name-side snapshot/compare/truncate in every `try_parse` speculation
(`ast_based_generator.rs:8247/8352`, 39 monomorphized sites in the fused
symbols), and name-stack `RawVec::grow_one` allocator growth.

This session's source verification (HEAD `b9588e29`): all guard-traffic
emission sites are exactly (a) the protocol rule method
(`ast_based_generator.rs:3824/3871/3938`), (b) the cascade internal match
frame (`cascade.rs:598/657/663`), (c) the universal `try_parse`
(`ast_based_generator.rs:8242-8367`) with cascade bodies calling it from
exactly two emission sites (`cascade.rs:774/998`), and (d) the name readers
listed above. `ast_generator_direct.rs`, `scan.rs`, and the cascade
build-pass/leaf forms contain zero guard traffic (asserted by the existing
emitter self-check tests).

## 3. The fix (fix-hierarchy level: engine/runtime + emitter — ONE representation change)

**Lib (`rust/src/ast_pipeline/mod.rs`, additive-API discipline):**
- `RecursionGuard::enter_id_bare(rule_id, position)` — push ONLY
  `rule_id_stack` (the bare fused frame's complete representation).
- `RecursionGuard::exit_bare()` — pop ONLY `rule_id_stack`.
- `RecursionGuard::truncate_stacks(name_len, id_len)` — per-stack restore
  for mixed-depth stacks.
- `check_cycle_id`: the maximum-depth arm's ceiling test and `depth` payload
  re-pointed from `parse_stack.len()` to `rule_id_stack.len()`. Byte-identical
  for every existing caller (the stacks are lockstep wherever bare frames do
  not exist — i.e., everywhere today), and EXACT under mixed stacks: the ID
  stack counts ALL live frames (protocol frames push both stacks), so the
  recursion ceiling keeps firing at the true total depth. The
  `MutualRecursive.rules` payload still reads `parse_stack` (complete in
  every protocol parse; possibly partial during a bare parse, where no
  generated consumer reads it — the generated arms bind `depth` only).
- Public legacy surfaces preserved verbatim: `enter`, `enter_id`, `exit`,
  `truncate_stack`, `check_cycle`, both public fields.

**Emitter (`rust/src/ast_pipeline/ast_based_generator.rs`):**
- Universal protocol `try_parse`: TWO independent snapshots
  (`saved_name_len` = `parse_stack.len()`, `saved_id_len` =
  `rule_id_stack.len()`) restored per-stack via `truncate_stacks`.
  Behavior-identical in every all-paired parse (both saved values equal ⇒
  both truncates identical to today) and CORRECT when a protocol
  speculation spans bare ID-only frames (today's single-length
  `truncate_stack(saved_parse_stack_len)` would destroy live deeper ID
  frames — the mixed-mode hazard this design closes). Label capture stays
  `parse_stack.last()` verbatim (accurate wherever it is consumed: the
  label materializes only under trace, trace ⇒ protocol ⇒ paired).
- New `try_parse_bare` (emitted only when the cascade plan is active):
  ID-only snapshot/truncate; rollback label = `rule_id_stack.last()`
  mapped through `Self::RULE_NAMES` (the `-0189` replacement obligation —
  the same innermost frame, hence the same label value, since today's
  paired push makes `parse_stack.last()` and `rule_id_stack.last()` the
  same frame); position/coverage/semantic checkpoint + `rollback_to_labeled`
  identical to the protocol wrapper; no trace branches
  (`bare_parse ⇒ !logger_enabled ⇒ !trace_enabled`, the documented
  invariant at the match-regex helper).
- `create_contextual_error`: the rule-name vector built from
  `rule_id_stack` mapped through `Self::RULE_NAMES` instead of
  `parse_stack`. Byte-identical today (lockstep + total bijection: modern
  generated parsers call `enter_id` exclusively, no `RuleId::MAX` frames)
  and COMPLETE during a bare parse (the name stack would be shallow there).

**Cascade emitter (`rust/src/ast_pipeline/ast_based_generator/cascade.rs`):**
- Internal match frame: `enter_id` → `enter_id_bare`, `exit` → `exit_bare`
  (the fixed entry/exit unit; `check_cycle_id` call unchanged — it already
  scans the ID stack and now counts ID depth).
- Both cascade `try_parse` call sites (`:774`, `:998`) → `try_parse_bare`
  (the rollback unit; growth follows: no bare name pushes ⇒ no name-stack
  `RawVec::grow_one` on the bare path).
- Emitter self-check tests updated to pin the bare names.

**What is deliberately NOT changed:** the protocol rule method's paired
`enter_id`/`exit` (observability keeps full name frames), the legacy
name-scan path, the rule-context stack (the promoted `-0174` member-5 unit
— a LATER leaf), diagnostics text, `ParseError` shapes, public telemetry,
memo/carrier representation, and all grammars.

## 4. Behavior-identity argument (why verdicts cannot move)

- Bare parses: trace/coverage/counters are off by definition of
  `bare_parse`; the only observable outputs are the parse verdict/value and
  error payloads. Cycle verdicts are unchanged (`check_cycle_id` scans the
  same ID frames as today; the ceiling counts the same total depth as
  today's lockstep length). `RecursionDepthExceeded { position, depth }`
  carries the same depth. Contextual errors reconstruct the identical
  name vector through the bijection. The rollback label maps to the
  identical value (and is dead under bare anyway).
- Protocol parses: no bare frames exist (bare routing requires
  `bare_parse`), so the stacks remain lockstep and every changed surface
  degenerates to today's behavior (two equal snapshots, same truncates,
  same payloads).
- The ALL-11 interpreter↔generated byte-identical oracle, cert ×3, shape,
  duality, PCRE2 compile oracle, and typed differential gates adjudicate
  this claim mechanically in the battery.

## 5. Pre-registered measurement protocol

- Immediate baseline: the current release probe, built from the clean
  `-0199` HEAD tree (`b9588e29`), byte-identical to the preserved
  `preserved_probes/regex_perf_probe_ckptword_8aef8541` (verified by `cmp`
  this session). Copied to scratch + SHA banked BEFORE any source change
  (`base_probe.sha256` = `8aef8541071b7d95…`); pre-regen artifact SHAs
  banked (`artifacts_pre_regen.sha256`, regex = `f85f2121…`).
- Candidate: the emitter/lib change + the canonical all-11 regen train
  (tool rebuilt FIRST, ebnf + fixed-point check, the 8 focus targets), then
  `cargo build --release --features "generated_parsers mimalloc_perf"
  --bin regex_perf_probe` (fat LTO, codegen-units 1 — the standard
  closure-bench config).
- Corpus: `regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_cases.jsonl`
  (2,189 cells), probe-default adaptive sampling, full per-cell JSONL banked.
- Bench: 3 base floor-validation rounds, then 5 rounds × 2,000 samples
  (200 warmup) per side, order alternated per round. Steering only.
- Custody: memory guard (16384 MB budget, 10% floor, disk floor default),
  `caffeinate -i`, serialized single-heavy-runner, idle host; custody
  asserts the base probe SHA, the CHANGED candidate regex artifact
  identity, and candidate-probe mtime newer than both the lib and the
  regenerated artifact.
- **Acceptance (binding, `-0197` ratchet):**
  1. unrounded candidate corpus geomean **strictly <** unrounded base
     corpus geomean from the same session;
  2. zero verdict flips across all 2,189 cells;
  3. candidate corpus MAX ≤ **483,583 ns**.
  Equal-or-higher geomean, any flip, or a MAX breach ⇒ unconditional
  product reversion (source + regenerated artifacts restored); only the
  evidence/rejection record commits. Bench rounds are steering/reporting
  only. Floor-validation sanity: base bench geomean within ±6% of the
  banked ≈1,937.4 ns, else the sweep is invalidated for custody (rerun),
  not adjudicated.
- Honest pre-registration note: the priced unit (≈8.09 ns on a ≈1,193 ns
  geomean ≈ −0.68%) is inside the observed session noise span (≈2.3%), and
  the candidate owes unpriced replacement work (the ID-map label capture on
  the Err path, the second snapshot word in the protocol wrapper) plus
  cross-site codegen effects of narrower bare frames. Per the directive,
  noise affects confidence reporting, never direction: the same-session
  strict compare adjudicates, and an unfavorable valid measurement REVERTS
  the product change.

## 5b. Session custody finding — the regex canonical artifact is the HOOKS-form emit

Recorded mid-session (before any A/B run), root-caused with tools per the
alertness directive:

- The banked pre-change regex artifact `f85f2121…` contains **769**
  `parse_*_typed` occurrences. `focus_regex` (the documented canonical regen
  path) emits the **no-hooks default**, which contains **0** — the first
  train pass reproduced every OTHER artifact's expected delta but produced a
  typed-less regex artifact (`1a5f7018…`).
- The typed surface is emitted only by `--enable-parser-hooks` — the
  `regex_typed_differential_gate` step-1 command. That gate's step-3
  "restore to default emit" is the ALREADY-TRACKED silent-failure trap
  (`2>/dev/null || true`), and the `-0198`/`-0199` post-battery hash-checks
  pinned `f85f2121…` — the hooks form — as the verified canonical. The
  `-0198` train's own post-regen bank shows the same phenomenon
  (`7ee95ce2…` no-hooks after `focus_regex`, `f85f2121…` on disk at
  session end).
- Proof of form: regenerating with the step-1 hooks command at the `-0200`
  vintage reproduces the 769 typed occurrences, and its diff against
  `f85f2121…` (23,830 changed lines) contains ONLY the `-0200` surfaces
  plus the artifact's embedded own-output-path strings (the same
  path-dependent bytes the ebnf fixed-point check normalizes).
- Custody consequence: the `-0199` floor probe embeds the hooks form, so
  apples-to-apples A/B REQUIRES the candidate artifact in the hooks form.
  The train's step 5b now re-emits the regex artifact with
  `--enable-parser-hooks` at the canonical path (candidate identity
  `b535be2c…`), and the expected-delta review passes over all 11 artifacts
  (`artifact_delta_review.txt`, `overall=0`).
- Durable repo consequence (surfaced to the director): for the regex
  family, "regen is canonical ONLY via `make focus_*`" is INCOMPLETE — the
  on-disk canonical additionally requires the hooks re-emit. Until the
  Makefile grows an explicit `focus_regex` hooks step (a separate leaf —
  this one owns exactly the ID-only fix), the banked `regen_train.sh` step
  5b is the reproducible recipe.

## 6. Reproduction

```sh
bash docs/tasks/artifacts/recursion_id_only/regen_train.sh   # all-11 regen + fixed point
bash docs/tasks/artifacts/recursion_id_only/battery.sh       # correctness battery
bash docs/tasks/artifacts/recursion_id_only/run_ab.sh        # pre-registered A/B
python3 docs/tasks/artifacts/recursion_id_only/analyze_ab.py # adjudication
```
