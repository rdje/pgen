# STORE-AWARE-GEN — Design (`.2`)

> Tool-backed design for semantic-store-aware (context-valid) stimuli generation. Companion to
> [`STORE-AWARE-GEN.md`](STORE-AWARE-GEN.md). Pins the exact mechanism so the `.3`–`.5` implementation is
> turnkey. All file:line references verified 2026-06-08 (`PGEN-STORE-AWARE-GEN-0002`). **No code in this
> slice** — design only.

## 1. The invariant

`GEN-VALID`: in faithful mode, every sample the generator emits satisfies the SAME semantic
`@predicate` constraints the parser enforces. The generator becomes a SECOND consumer of the existing
`@emit_fact` / `@predicate` annotations — the parser and generator share one runtime
(`semantic_runtime.rs`) and one annotation vocabulary; no new annotation is introduced.

## 2. What the parser does (the thing to mirror) — tool-backed

The generated parser runs each rule body inside a semantic-runtime transaction. For a rule's annotations
(`Annotations.semantic_annotations[rule]` / `branch_semantic_annotations[rule][branch]`, in
`mod.rs:1125`), the codegen (`ast_based_generator.rs`) emits:

- **`@emit_fact`** → builds a `SemanticFactSpec { kind, name, attributes }` (`semantic_runtime.rs:131`),
  resolving `$ref` args against the parsed content, and calls
  `transaction.apply_directive(EmitFact(spec))` → `state.emit_fact(spec)` (`:1894`). The directive list
  is `effect_directives_for_rule(rule)` applied in the effects phase (`ast_based_generator.rs:1566`).
  *(regex's `@emit_fact: { kind: regex_capture_group, name: capture }` has only LITERAL scalar args — no
  `$ref` — so emission needs no content resolution.)*
- **`@predicate`** → builds a `SemanticPredicateSpec { name, args, phase, view }` (`:354`), resolves any
  `$ref` arg (e.g. `$index` → the rule's produced `index` field) against the parsed content via
  `resolve_semantic_predicate_spec_against_content`, then calls
  `state.evaluate_predicate(&spec) -> Option<bool>` (`:2010`): `Some(true)`=pass, `Some(false)`=fail,
  `None`=indeterminate (no-op). `phase: pre` is evaluated BEFORE the body (`ast_based_generator.rs:1496`,
  fail → `Backtrack`); `phase: post` AFTER the body (`:1600`, fail → `Backtrack`); `phase: branch`
  per-branch in the tournament (`:2981`, fail → prune that branch).
- **`fact_count_at_least`** (`semantic_runtime.rs:2178`): `args[0]` = kind identifier, `args[1]` = the
  COUNT THRESHOLD (a `usize`); returns `count(kind) >= threshold`. For
  `fact_count_at_least(regex_capture_group, $index)` the threshold is the rule's generated/parsed `index`
  value, so `\NN` is accepted only when ≥ `NN` capture groups already exist.

Speculation safety: every `try_parse` (branch alternative / optional / quantifier iter / lookahead)
snapshots + restores the store: `checkpoint()` (`:1586`) before, `rollback_to_named(cp, ctx)` (`:1699`)
on `Err` (`ast_based_generator.rs:5559`/`:5596`). The multi-branch tournament uses per-branch
`extract_delta_since` (`:1618`) + winner `apply_delta` (`:1651`) so only the winning branch's facts
survive (`ast_based_generator.rs:3152`–`3291`). **The generator must mirror exactly this discipline.**

## 3. The generator changes (turnkey plan)

### 3.1 State + activation gate
- Add `gen_semantic_state: SemanticRuntimeState` to the generator (`StimuliGenerator`, near
  `last_terminal_word_shaped`), `SemanticRuntimeState::new()` in the ctor.
- Add a one-time `grammar_has_generative_predicates: bool` computed from the annotations: true iff any
  rule/branch carries a `@predicate` whose primitive is one we honour generation-side AND the grammar
  also has the matching `@emit_fact` source. When false, the entire store-aware path is bypassed →
  **byte-identical generation** for predicate-free grammars (json/ebnf/most SV today). This is the no-op
  guarantee, enforced by a gate at every hook (`if !self.store_aware_active { /* old path */ }`).
- Reuse `parse_semantic_runtime_directives` (`semantic_runtime.rs`) to convert a rule's
  `Vec<SemanticAnnotation>` → the `SemanticRuntimeDirective`s (EmitFact / predicate) once, cached per
  rule (avoid re-parsing each generation). The generator already reads `semantic_annotations` (e.g.
  `semantic_hint_for_rule`, `:8288`), so the access path exists.

### 3.2 Emit hook (mirror the parser's effect phase)
In `generate_rule`, AFTER a rule body generates successfully, apply the rule's `@emit_fact` directives to
`gen_semantic_state` (resolving `$ref` args against the GENERATED text/structure the same way the parser
resolves against parsed content — reuse `resolve_semantic_runtime_value_against_content`). For
`regex_capture_group` the args are literal, so this is a direct `emit_fact(SemanticFactSpec{ kind:
"regex_capture_group", name: Identifier("capture"), attributes: [] })`. Generation is left-to-right, so
groups generated earlier in the pattern are in the store by the time a backreference is generated.

### 3.3 Predicate hook (two strategies; pick per predicate)
- **(A) Constraint-directed (preferred for invertible predicates — the `.3` MVP).** For a rule gated by
  `fact_count_at_least(K, $index)` where `$index` is produced by a numeric sub-rule, the constraint
  inverts cleanly: `$index ≤ count(K)`. Compute `n = gen_semantic_state.count(K)` and feed `n` as a
  numeric UPPER BOUND into the existing value machinery (`constraint_driven_candidate` /
  `rule_value_constraints`, `:7322`) for the sub-rule that produces `$index` (regex
  `backreference_digits`). If `n == 0` the multi-digit `numeric_backreference` branch is UNSATISFIABLE →
  PRUNE it from `generate_or`'s candidate set (mirrors branch-predicate pruning,
  `ast_based_generator.rs:3102`) so the generator naturally falls to `numeric_backreference_single`,
  `octal_escape`, or generating a capture group first. No wasted attempts; deterministic.
- **(B) Generate-check-backtrack (general fallback, mirrors the parser's post-predicate).** For a
  non-invertible predicate: checkpoint the store, generate the body, resolve+`evaluate_predicate`; on
  `Some(false)` rollback + retry a bounded number of times, else fail the rule (`Err`) so the caller's
  alternative/retry handles it. `None`/`Some(true)` → keep. This is the always-correct path; (A) is the
  optimization for the decidable cases.

### 3.4 Checkpoint/rollback discipline (the bulk of the work — mirror `try_parse`)
Every generator site that SPECULATIVELY generates then may discard must `checkpoint()` the store before
and `rollback_to_named(cp, ctx)` on discard, exactly like the parser:
- `generate_or` branch selection — when a chosen branch's predicate fails (strategy B) or it is pruned,
  the store must not retain its facts. Use the tournament pattern (`extract_delta_since` + winner
  `apply_delta`) if multiple branches are tried.
- `generate_quantified` iterations (`:6146`) — a discarded iteration's facts roll back.
- the relational-constraint retry loop (`generate_sequence`, `:5886`) — each discarded attempt rolls back.
- target/witness generation retries + the recovery/mutation replays.
Reuse the EXACT methods (`checkpoint`/`rollback_to_named`/`extract_delta_since`/`apply_delta`) — no new
runtime code; the generator just drives the same API the parser drives.

### 3.5 Determinism
The store is threaded deterministically (no RNG in the store path); strategy (A) prunes/bounds
deterministically. A fixed (seed, grammar) must still produce a byte-identical sample. Lock with a
determinism cmp (generate twice, `--output`, `cmp`) in the verification matrix.

## 4. `.3` MVP scope (the regex `.3.12` closer)

Minimal first cut = §3.1 + §3.2 (emit `regex_capture_group`) + §3.3(A) for `fact_count_at_least` +
the §3.4 checkpoints needed for regex generation. Outcome: `numeric_backreference` generates an index
≤ the live capture-group count (or is pruned when 0) → the generator never emits `\98495`-style
backrefs to non-existent groups → regex DEFAULT cert-coverage `sample_parse_failures` = 0 at seed 1 and
across a seed sweep. Closes `REGEX-PCRE2-FIDELITY.3.12`.

## 5. Verification matrix

| Check | Expectation |
| --- | --- |
| regex cert-cov, seed sweep (0,1,7,…) + count 500 | `sample_parse_failures` = 0 everywhere (no `\NN`-to-nonexistent-group) |
| minimal probes (`parseability_probe --parse regex`) | a generated `\NN` references only existing groups; `(?(R…))` likewise |
| determinism | generate ×2 at fixed seed → `cmp` identical |
| no-op grammars (json/ebnf/SV) | byte-identical generation vs pre-change (store-aware path bypassed) |
| `stimuli_cross_family_platform_gate` | green (regex + VHDL + SV-2017) |
| `regex_pcre2_compile_oracle_gate` | byte-identical (parser unchanged) |
| `cargo test --lib` | green + new locking tests (emit-then-count; predicate-prune; rollback-on-discard) |
| self-hosting gate | OK (no grammar change) |

## 6. Risks / mitigations

- **`generate_or` is pick-not-try.** Today it weighted-picks one branch; strategy (A) PRUNES unsatisfiable
  branches BEFORE the pick (cheap, deterministic, no retry storm). Strategy (B) is reserved for predicates
  (A) can't invert; bound its retries and fail-closed to a discard (never emit an unverified sample).
- **Perf.** The store path is gated off for predicate-free grammars (no cost). For regex, the store is
  tiny (a handful of `regex_capture_group` facts); `count` is O(1) via the secondary index
  (`fact_count_at_least` is already O(1), `semantic_runtime.rs:5219`).
- **Coverage.** Pruning the multi-digit backref when no groups exist is CORRECT (those samples are
  invalid), not a coverage loss — and the generator can still reach `\NN` by generating ≥N groups first
  (the diverse/witness passes will), so the construct stays reachable.
- **No fixed-bound guesses** (the `.3.12` non-goal): the bound is the LIVE count, derived from the store,
  not a magic constant.

## 7. No-workarounds-hierarchy placement

Level-3+ (a new general parser-agnostic generator capability), justified because the generator
structurally cannot otherwise honour a context-dependent predicate (Levels 1–2 — existing annotations /
store — are what the PARSER already uses; the gap is that the GENERATOR does not consume them). The
capability reuses the existing runtime + annotation vocabulary (no new primitive at the language level),
so it is the smallest general addition that closes the duality.
