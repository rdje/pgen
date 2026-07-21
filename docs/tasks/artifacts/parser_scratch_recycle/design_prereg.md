# `PGEN-RGX-0078-0209` design pre-registration — the parser-scratch TLS-lease recycling unit

Leaf `RGX-0078.5.j.4`, session #181, 2026-07-21. Registered BEFORE any code,
per the `-0197` ratchet contract. Executes the `-0208` adjudication under the
2026-07-21 director GO (the teardown/construction recycling fork LICENSED at
the engineer's discretion; the one-fix-per-fresh-session contract intact).

## 1. The mechanism being bought (the `-0208` prices, floor 1037.804231341058 ns, noise 23.657 ns)

Every parse constructs and tears down the full parser component set.
In-scope shares (zero-overlap where exact):

| lane | ns | scope note |
|---|---|---|
| state-family teardown drops (`FactIndex` 2.19 + `SemanticFa` 2.23 + `SemanticSc` 2.78 + `ScopeNode` 2.79 + sub-threshold state drops) | ≈10.0 | exact, fully in-scope |
| `SemanticRuntimeState::new` (construction twin) | 6.10 | exact, fully in-scope |
| `RegexParser` drop_in_place (inline container frees) | ≤13.80 | partial — the `memo_fail`/tainted/guard frees are in-scope; `memo` + arena-adjacent stay |
| `_mi_free` | ≤36.28 | partial — recycled-container frees die; arena-chunk/`memo`/node frees stay |
| construction allocs (`_mi_theap_malloc_zero_aligned*` 24.2 + `_mi_malloc_aligned` 18.49 + `__mi_malloc_generic` 24.37 shares; the hashbrown `with_capacity` ctrl-byte zeroing is exactly the memo_fail class) + `__mi_page_retire` 11.47 churn share | partial | the recycled containers' `with_capacity` allocs die; warm capacity also removes in-parse growth reallocs |
| the inlined-`new()` mass inside `parse_once_timed` 46.08 [harness] | partial | the probe's per-parse fn inlines `RegexParser::new` — its recycled-component share dies |

Conservative gross **≈30–55 ns ≈ 1.3–2.3× noise**. Expected delivery at the
established 30–70% whole-mechanism capture band: **≈−9 to −38 ns ≈ −0.9% to
−3.7%** corpus geomean. The ratchet adjudicates regardless; a strict-decrease
miss reverts unconditionally and closes the lane measured-exhausted.

## 2. The ONE unit (scope, exact)

Recycle the parser's **lifetime-free** construction/teardown component set
across parses through per-component thread-local leases:

- `semantic_runtime_state: SemanticRuntimeState`
- `memo_fail: FxHashSet<(RuleId, usize)>`
- `memo_fail_tainted: FxHashMap<(RuleId, usize), u64>`
- `recursion_guard: RecursionGuard`

REFUSED / DEFERRED (recorded, not silently dropped):

- `memo: FxHashMap<(RuleId,usize), MemoEntry<'input>>`, `thin_entries:
  Vec<ThinTapeMemoEntry<'input>>`, `deriv_tape: Vec<TapeWord<'input>>` —
  `'input`-parameterized; TLS reuse would need lifetime-erasing `unsafe`
  (refused for this unit; a future fork may design it).
- `NodeArena` — wraps four `typed_arena::Arena`s (drop-only, no reset API) +
  `'input`-bound. Its 10.40 ns drop + ≈72 ns `alloc_extend` family stay the
  named residual.
- `rule_call_counts: Arc<Vec<AtomicU64>>` — recyclable only via
  `Arc::get_mut` uniqueness protocol; Arc-escape semantics (dashboard clones)
  add risk disproportionate to its share. Deferred.
- `grammar_profile: Option<String>` — one small alloc; below pricing floor.

## 3. The lease design (the `-0205` pattern generalized)

Lib (`rust/src/ast_pipeline/mod.rs`), parser-agnostic:

```rust
pub trait ParseScratchRecyclable: Default {
    fn scratch_slot() -> &'static std::thread::LocalKey<Cell<Option<Self>>>;
    /// Re-establish the CONSTRUCTOR's postcondition on a recycled instance
    /// (allocations kept). Runs on every take().
    fn reset_recycled(&mut self);
    /// Alloc-free placeholder swapped into a returning lease (never used as
    /// a live component; immediately dropped by the lease's drop glue).
    fn empty_shell() -> Self;
}

pub struct ParseScratchLease<T: ParseScratchRecyclable> {
    inner: T,
    /// Construction-time leases return their component to the thread slot;
    /// `Default` leftovers (the emitted `mem::take` pattern) do NOT.
    active: bool,
}
```

- `take()` → slot-take or `T::default()`, then `reset_recycled()`, `active:
  true`. `Deref`/`DerefMut` → `inner` (zero-cost, no branch).
- `Drop`: if `active`, `mem::replace(&mut self.inner, T::empty_shell())` and
  store the real component in the slot (`Cell::set` — last return wins the
  slot, the `-0205` semantics; the shell frees nothing).
- `Default` (⛔ load-bearing): `{ inner: T::default(), active: false }`. The
  emitted semantic-transaction path `mem::take`s the state FIELD per
  annotation-bearing rule entry and restores it by assignment; its leftover
  must (a) deref to a VALID state — the SV-EXH-PROOF.3.3.3 doctrine that a
  panic between take and restore leaves a valid empty state — and (b) cost
  exactly what today's leftover costs (`SemanticRuntimeState::default() ==
  new()`, two small allocs, dropped on restore). An `active: false` leftover
  drops PLAINLY — no TLS traffic — so the transaction path is cost- and
  semantics-identical to today.

Per-type impls:

- `SemanticRuntimeState` (in `semantic_runtime.rs`): `reset_recycled` =
  `self.facts.clear(); self.reset_for_new_parse(&HashMap::new())` — riding
  the `-0198` proven primitive. ⛔ THE FACT-LEAK HAZARD, resolved by
  construction: `reset_for_new_parse` PRESERVES facts by contract (the
  library-import flow), which is correct within one parser instance and a
  cross-parse leak across recycled instances; clearing `facts` first makes
  the replay loop empty, so the result is field-for-field `new()`-equivalent
  (`counters.facts_imported = 0`, `write_epoch = 0`, root scope/arena/chain,
  empty `predicate_defs`). Pinned by a new test:
  used-state → `reset_recycled` → `PartialEq`-equal to
  `SemanticRuntimeState::new()`. `empty_shell` = all-fields-empty private
  constructor (no root pushes — alloc-free).
- `FxHashSet<(RuleId,usize)>` / `FxHashMap<(RuleId,usize),u64>`:
  `reset_recycled` = `clear()` (capacity kept); `empty_shell` = `default()`.
- `RecursionGuard`: new `Default` (empty stacks/cache, `max_depth: 0`) +
  `reset_recycled` = clear stacks + `cycle_cache` (capacity kept; max_depth
  left for the setter) + new `pub fn set_max_depth(&mut self, usize)`;
  `empty_shell` = `default()`.

## 4. The emitter change (`ast_based_generator.rs`, protocol emitter ONLY)

1. Four field decls → `crate::ast_pipeline::ParseScratchLease<…>`.
2. `new()`: the state block becomes `ParseScratchLease::take()` +
   `set_predicate_defs(...)` (take's reset supplies the `new()` postcondition);
   `memo_fail`/`memo_fail_tainted` become take + `reserve(<the exact K3a/C2
   sizing formulas, verbatim>)` (empty + `reserve(n)` ⇒ capacity ≥ n — the
   with_capacity invariant preserved, warm capacity retained);
   `recursion_guard` becomes take + `set_max_depth(#recursion_guard_max_depth)`.
3. The rendered-contract self-check pins re-anchored to the new spellings.
4. Every other access site is UNTOUCHED — `Deref`/`DerefMut` auto-deref keeps
   method calls, `&`/`&mut` coercions, and place expressions compiling
   verbatim; the whole-field sites (`mem::take` at the transaction wrapper,
   the restore assignment) type-check as lease moves with the `Default`
   contract of §3. The bootstrap emitter (`ast_code_generator.rs`) is NOT
   touched — its two annotation artifacts must regenerate BYTE-IDENTICAL.

## 5. Behavior-visibility audit (pre-registered)

- Between `new()` and `parse()`: today the state is `new()` + predicate defs;
  recycled it is `reset_recycled` (== `new()`-equivalent, test-pinned) +
  predicate defs — observably identical through the public accessors.
- `parse()`'s `prepare_parse_state` → `reset_for_new_parse` ceremony:
  unchanged (auto-deref).
- Container capacities may EXCEED today's (warm reuse). Iteration order over
  the fail sets exists only in the env-gated `report_memo_stats` debug
  surface (never in product output); Fx is fixed-seed, so equal capacity ⇒
  equal order, larger capacity ⇒ possibly different debug-row order. The
  byte-identical oracle battery + cert gates adjudicate the product surface.
- Two live parsers on one thread: the second takes a fresh component
  (slot empty) — correct, merely unshared; last return wins the slot.
- Cross-parser-type recycling (e.g. regex → ebnf on one thread): sound —
  `reset_recycled` re-establishes the constructor postcondition; the
  component types are the shared lib types.
- Panic/unwind: an `active` lease returns its (dirty) component on unwind;
  the NEXT take resets it. The transaction-leftover doctrine of §3 holds.
- Memory: each thread retains at most one component set, sized by the
  largest parse seen (worst corpus cell class — tens of KB). The 16384 MB
  guard supervises every run.
- Determinism: unchanged (no hashing/order semantics touched; Fx already
  fixed-seed).

## 6. Custody / land gate (pre-registered)

- Base = `preserved_probes/regex_perf_probe_fxstore_a4067793`
  (SHA `a406779314b971305da58b6eb7688be233342efd78348c3c62a456dda24b5ee4`),
  copied to scratch + SHA-banked BEFORE any regeneration.
- Pre-regen artifact SHAs banked; after the all-11 regen train (tool rebuilt
  FIRST; ebnf fixed-point check): the regex artifact MUST differ (an emitter
  unit that didn't regenerate is a broken run — the `-0205` polarity) and the
  two bootstrap annotation artifacts MUST be byte-identical (the emitter
  split of §4). Expected-delta review over all 11 (the `-0205`
  `review_artifact_delta.sh` shape): every protocol-artifact delta must be
  exactly the four field types + the `new()` init block; growth bar ≤5%.
- Battery: dual-feature lib suite (incl. the ALL-11 interpreter↔generated
  byte-identical oracle + the new `reset_recycled` equivalence pin), cert ×3
  seeds 0/7/42 (banked `268/9/259/0 fully_certified`, spf=0),
  `ast_shape_contract_gate`, `duality_hunt_gate`,
  `regex_pcre2_compile_oracle_gate`, clippy source-strict.
- Candidate probe: `cargo build --release --features "generated_parsers
  mimalloc_perf" --bin regex_perf_probe` (fat-LTO profile), newer-than-source
  asserted.
- A/B (the `-0202` harness): 3 base floor-validation rounds (±6% custody band
  vs the banked 1,619.1 ns bench floor), 5 alternated bench rounds
  (steering), full-corpus paired sweep both sides.
- **Binding acceptance (the `-0197` ratchet):** unrounded candidate corpus
  geomean STRICTLY below the same-session base sweep geomean; verdict flips
  0/2,189; candidate corpus MAX ≤ 425,000 ns (director #178). Anything else
  ⇒ unconditional product reversion (evidence banked either way).
- After the accept/revert commit the session proves clean and STOPS (the
  fresh-session contract).

## 7. POST-REGEN ADDENDUM (dated 2026-07-21, recorded after the regen train — a prereg misprediction, NOT a silent rewrite)

§4/§6 predicted the two annotation artifacts byte-identical on the
assumption they come from the separate legacy bootstrap emitter
(`ast_code_generator.rs`, which carries its own plain `recursion_guard`
templates). The regen train showed otherwise: the annotation pair is
emitted by `ast_based_generator` in bootstrap MODE, so the `-0209` surfaces
reach all 11 artifacts, and their deltas are exactly the four lease
fields + construction blocks (verified by the corrected
`review_artifact_delta.sh`, which now requires the `-0209`-surface-only
delta for ALL 11 and refuses an unchanged artifact). Effect on the unit:
none adverse — recycling is uniform across every parser, which is the
parser-agnostic doctrine's preferred shape; the custody polarity (regex
MUST change) still held. The misprediction is recorded here and in the
step result per the honesty discipline.
