# `PGEN-RGX-0078-0207` design pre-registration — the semantic-store SipHash → FxHash swap

Leaf `RGX-0078.5.j.4`, session #180, 2026-07-20. Registered BEFORE any code
change, per the `-0197`/`-0203`/`-0205` design-first discipline. The selected
ONE fix of the `-0206` re-pricing (`docs/tasks/artifacts/dindex_reprice/`).

## The priced mechanism (from `-0206`, on the 1,038.4 ns floor)

Zero-overlap hashing self-time, caller-pinned to the semantic store's three
std-`RandomState` maps (noise floor 23.670 ns):

| row | ns | attribution |
|---|---:|---|
| `core::hash::sip::Hasher::write ha755b248…` | 10.274 | band-rising 75/821/3656; called by the four `hash_one` monomorphs, of which only the two store-side ones are dynamically hot |
| `BuildHasher::hash_one hc6b44c1d…` | 4.030 | bl-sites: `FactIndex::insert`, `rollback_to_labeled_slow`, `any_with_name_at_scope`, rehash — the `by_scope_and_name` map |
| `BuildHasher::hash_one hafc51c93…` | 3.054 | bl-sites: `rollback_to_labeled_slow`, `evaluate_predicate`, `count_for_kind`, `positions_for_name`, rehash — the `by_kind` map |
| **total identifiable** | **17.359** | **= 0.733× noise GROSS; 30/50/70% capture = 5.21/8.68/12.15 ns — sub-noise at every fraction** |

NOT credited (stays regardless): `FactIndex::insert` container self 6.758 ns
(RawTable work), `SemanticRuntimeState::new` 9.230 ns (setup), the
protocol-memo inserts 12.26 ns (out of scope — shared observed-parse
surface).

## The honest license (recorded, not inflated)

This unit is **sub-noise even at 100% capture**. It proceeds as a `-0170`
small solo unit because: (a) it is the largest un-owned zero-overlap
mechanism left on the floor; (b) `-0204` pre-adjudicated it BY NAME
("parser-agnostic, mechanical, determinism-increasing FxHash swap"); (c) it
is fix-hierarchy level 2 (store), benefiting every parser; (d) the binding
`-0197` ratchet adjudicates with unconditional reversion — a miss costs
nothing and CLOSES the lane measured-exhausted (the `-0156` V1-M3
precedent). A REVERT verdict is a live possibility and is a legitimate,
informative outcome of this design.

## The change (ONE unit, lib-only, `rust/src/ast_pipeline/semantic_runtime.rs`)

All three fields are PRIVATE; the crate already imports
`rustc_hash::FxHashMap` (line 8) and the store's `directives_by_rule` pair
is already Fx (the `.5.a` fix — this completes the store's hashing story):

1. `FactIndex.by_kind: HashMap<String, FactKindIndex>` (≈:2127) →
   `FxHashMap<String, FactKindIndex>`.
2. `FactKindIndex.by_scope_and_name: HashMap<(usize, FactNameKey), Vec<usize>>`
   (≈:2166) → `FxHashMap<(usize, FactNameKey), Vec<usize>>`.
3. `SemanticRuntimeState.predicate_defs: HashMap<String, PredicateDef>`
   (≈:2408) → `FxHashMap<String, PredicateDef>`, with:
   - `new()` init `HashMap::new()` → `FxHashMap::default()`;
   - `set_predicate_defs(defs: HashMap<…>)` keeps its PUBLIC std signature,
     converts at the boundary (`defs.into_iter().collect()` — one-time at
     parser construction);
   - `reset_for_new_parse(&HashMap<…>)` keeps its PUBLIC std signature; the
     `clone_from` becomes clear + extend-with-clone (same content, reused
     allocation).

NO emitter change, NO generated-artifact change (expected: all 11 artifacts
byte-identical through the regen train), NO public API change, NO grammar
change, NO protocol-memo change.

## Replacement cost, priced per band (the `-0205` band law)

- Per-operation: FxHash replaces SipHash-1-3 on the same key traffic — a
  short word-mix loop vs full sip rounds; strictly cheaper per hash on every
  band; NO new fixed per-parse cost is introduced (no allocation, no table,
  no reset ceremony).
- The ONE boundary delta: `reset_for_new_parse` re-inserts (re-hashes) the
  predicate defs per parse instead of `clone_from`'s topology copy. On the
  measured corpus this is exactly ZERO work: the regex grammar has **no
  `@predicate_def`** (`grep -c predicate_def grammars/regex.ebnf` = 0), the
  map is empty, and `clear()`+`extend([])` on an empty map does nothing. For
  grammars WITH defs the count is a handful and the same swap removes sip
  from every store query on their paths — net favorable, and outside this
  fix's measured corpus anyway.
- Strict net banked pre-A/B: **0** (the `-0197` accounting rule).

## Soundness / determinism audit (done pre-code, banked in `-0206`)

- Iteration-order consumers: `by_kind` — never iterated; the state's
  `predicate_defs` — never iterated (get/len only); `by_scope_and_name` —
  ONE iteration site (`positions_for_name` :2256) whose only consumers are
  `.count()` (:2720) and `.any(…)` (:3985/:4004/:4022) — order-independent
  aggregates. No output, dump, or gate observes map order. (Empirical
  backstop: today's RandomState order is random per process, so any real
  order-dependence would already have made the byte-exact gates flake.)
- Hasher swap changes NO map semantics (same hashbrown container, same
  entry/get/remove behavior). Determinism strictly INCREASES (FxHash is
  fixed-seed; RandomState is per-process random).
- `fact_kinds` stays std intentionally: empty on the generated-parser path
  (hashbrown's empty-map short-circuit never hashes) — not part of the
  priced mechanism.

## Acceptance (binding, the `-0197` ratchet — pre-registered)

- Full battery at the changed vintage (dual-feature lib suite incl. the
  ALL-11 interpreter↔generated oracle; cert ×3 seeds 0/7/42 expecting
  byte-exact `268/9/259/0 fully_certified`; `ast_shape_contract_gate`;
  `duality_hunt_gate`; `regex_pcre2_compile_oracle_gate`; clippy
  source-strict).
- All-11 regen train with the ebnf fixed point; artifact-delta expectation
  for this LIB-ONLY unit: **byte-identical pre/post** (a changed artifact
  REFUSES the land gate — inverted vs the `-0205` emitter-unit check).
- Canonical corpus A/B vs base
  `preserved_probes/regex_perf_probe_dindex_d21fab44` (SHA `d21fab44…`),
  alternated 5×2000 bench (steering only) + full corpus sweeps: land ONLY on
  a strict unrounded same-session corpus-geomean decrease with flips
  0/2,189 and MAX ≤ the settled **425,000 ns**; floor-validation custody
  band ±6% vs the banked bench floor **1,609.1 ns**.
- Otherwise: unconditional product reversion; the design + rejection
  evidence stay banked; the lane closes measured-exhausted.

## Commit/session boundary

Exactly this one design+fix; after the accept/revert commit the session
proves clean and STOPS (the `-0197` fresh-session contract).
