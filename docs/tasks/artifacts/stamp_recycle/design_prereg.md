# `PGEN-RGX-0078-0210` design pre-registration — the STAMP-VALIDATED parser-scratch recycling unit

Leaf `RGX-0078.5.j.4`, session #182, 2026-07-21. Registered BEFORE any code,
per the `-0197` ratchet contract. This is the banked `-0209` follow-up (the
step result's "stamp-validated recycling redesign") executed as this
session's ONE fix, still under the 2026-07-21 director GO (the
teardown/construction recycling fork LICENSED at the engineer's discretion).

## 0. The re-pricing this session rides on (the cheap re-read, per the `-0209` NEXT pointer)

Nothing landed in `-0209` (unconditional revert, artifacts byte-exact —
re-verified this session: the 11-artifact pre-regen bank reproduces the
`-0209` bank byte-for-byte; base probe `a406779314b9…` SHA-asserted). The
`-0208` prices therefore remain HONEST on the unchanged floor
**1037.804231341058 ns** (noise 23.657 ns) and are ADOPTED, not re-captured:

- the licensed teardown/construction population stays the dominant un-owned
  mechanism: teardown_drop 51.13 ns gross (2.16× noise; ≈40.7 in-scope after
  excluding the NodeArena 10.40) + `_mi_free` 36.28 +
  `SemanticRuntimeState::new` 6.10 + the inlined-`new()` construction mass
  inside the 46.08 harness bucket + the zeroed/aligned mi-alloc lanes;
- every in-target lane is individually sub-noise (input view 13.286 CLOSED,
  checkpoint 9.031, position 5.673, guard 4.928, tape 3.327); protocol-memo
  inserts 12.30 — all refused as solo fixes, unchanged;
- spine_other 203.0 stays research-class (no designed mechanism).

New evidence since `-0208`: the `-0209` refusal's per-band decomposition
(`parser_scratch_recycle/band_analysis.txt`) tool-pinned WHY the eager-clear
lease form failed — **hashbrown `clear()` is O(capacity)**: a recycled
`memo_fail` carries the largest prior capacity (the corpus-max cell reserves
≈21k entries ⇒ a ~32k-bucket table), so every small parse paid a ~32 KB ctrl
memset ≈ +290 ns flat exactly where the geomean's log-weight lives (sub-1 µs
band +20.18%). The **≥20 µs band IMPROVED −1.45%** — the warm-allocation
mechanism is real once the flat add is gone. Hence the design law
(`docs/decisions/`-recorded): pooled parser state needs **O(live-entries)
invalidation, never an O(high-water-capacity) clear**.

## 1. The mechanism being bought

Identical population to `-0209` §1 (the priced table is incorporated by
reference — conservative gross **≈30–55 ns ≈ 1.3–2.3× noise**), MINUS the
mechanism that refused it. What this redesign changes about the price:

- the O(capacity) reset is REMOVED for the two fail-memo containers
  (generation-stamp invalidation is one u64 increment);
- new replacement costs, priced honestly: (a) one extra u64 stamp
  compare/store per fail-memo hit/insert; (b) a locality tax — probes into a
  persistent high-water table instead of a fresh dense one (bounded by the
  hot-key-line reuse across back-to-back parses: (rule, position) keys repeat
  structurally across inputs, so the hot bucket lines stay cache-resident);
  (c) the state's take-side `recycle_reset` remains a second
  `reset_for_new_parse` per parse — O(empty-collections) on regex, priced as
  a small retained residual (see §3 DEVIATION); (d) four TLS take/return
  round-trips per parse (≈2–4 ns each), as in `-0209`.

Expected delivery at the established 30–70% whole-mechanism capture band:
**≈−9 to −38 ns ≈ −0.9% to −3.7%** corpus geomean, now WITHOUT the flat add
that inverted `-0209`'s sign. The ratchet adjudicates regardless; a
strict-decrease miss reverts unconditionally and closes the recycling lane
measured-exhausted (two design forms would then have been refused on
measurement).

## 2. The ONE unit (scope, exact)

The `-0209` lease architecture is reused VERBATIM where it was sound (its
correctness battery was fully green; the refusal was speed-only):

- `ParseScratchRecyclable` (`'static`-supertrait) + `ParseScratchLease`
  (active-flag `Default` leftover; take/return through per-type thread
  slots; `Deref`/`DerefMut`) — byte-identical to the banked
  `refused_change.diff`.
- `SemanticRuntimeState`: `recycle_reset` (facts-clear THEN
  `reset_for_new_parse` — the fact-leak hazard closed) + `recycle_shell` +
  the `recycle_reset_matches_new` pin — byte-identical to the banked diff.
- `RecursionGuard`: `Default` + `set_max_depth` + O(live) `reset_recycled`
  (two `Vec::clear`s = O(live frames); `cycle_cache.clear()` — that field is
  written by NOTHING in the generated-parser path (neither `check_cycle` nor
  `check_cycle_id` touches it; verified by exhaustive grep this session), so
  its capacity is permanently 0 and the clear is a no-op, not an O(capacity)
  hazard) — byte-identical to the banked diff.

What CHANGES vs `-0209` — the two fail-memo components become stamped lib
types (parser-agnostic, `rust/src/ast_pipeline/mod.rs`):

```rust
pub struct StampedFailSet {
    entries: FxHashMap<(RuleId, usize), u64>, // key -> generation stamp
    generation: u64,
}
// contains(&key) -> entry stamp == current generation
// insert(key)    -> entries.insert(key, generation)  (overwrite = revalidate)
// reset_recycled -> generation += 1  (O(1); NO clear)
// ensure_capacity(target) -> grow only if capacity < target
// iter()/len()/is_empty() -> current-generation view (debug-stats only)

pub struct StampedTaintedFailMap {
    entries: FxHashMap<(RuleId, usize), (u64, u64)>, // key -> (generation, epoch)
    generation: u64,
}
// get(&key) -> Option<&epoch> iff stamp == current generation
// insert(key, epoch) / remove(&key) / iter() / len() / ensure_capacity
// reset_recycled -> generation += 1
```

A stale-generation entry is semantically ABSENT: `contains`/`get` refuse it,
`insert` overwrites it in place (no growth on revalidation). This preserves
the two memo soundness contracts exactly:

- PARSE-TERMINATION.6 (fail set): a cached failure is replayable only within
  the parse that recorded it — the stamp binds every entry to one take().
- MEMO-STORE-SOUNDNESS.2 (tainted map): a tainted failure additionally
  carries its write-epoch and is evicted (`remove`) when the epoch has moved
  within the SAME generation; across generations the stamp already kills it
  (a recycled state resets `write_epoch` to 0, so without the stamp a stale
  entry from a prior parse could collide with an equal epoch value — the
  stamp closes that soundness hazard BY CONSTRUCTION, it is not just a
  performance device).

Memory backstop (pre-registered, not tunable later): `reset_recycled` clears
the table (capacity kept) only when `entries.len()` has exceeded
`1 << 18` (fail set) / `1 << 12` (tainted) — ≥12× the largest corpus cell's
live population (≈21k), so the O(capacity) memset amortizes to sub-0.1 ns
per parse and per-thread memory stays bounded (≈12 MB worst-case fail-set
table) under the 16384 MB guard. Union accumulation across parses is
otherwise bounded by the distinct-(rule, position) union, which the
overwrite-on-insert semantics keep from growing past the corpus's structural
key population.

REFUSED / DEFERRED (unchanged from `-0209` §2): the `'input`-parameterized
containers (`memo`, `thin_entries`, `deriv_tape`), `NodeArena`,
`rule_call_counts`, `grammar_profile`.

## 3. DEVIATION from the banked follow-up note (recorded, with the reason)

The `-0209` step result banked "take-side reset folded into the one
`prepare_parse_state` reset". Re-examining the emitted contract this session
REFUSES that fold: `prepare_parse_state` documents that facts pushed onto
the parser BETWEEN `new()` and `parse()` are intentional preloads (the SV
stdlib flow, `SV-EXH-PROOF.3.3.4.b.6.2.37.2`) and MUST survive the per-parse
reset. A stale-facts clear deferred into `prepare_parse_state` cannot
distinguish a prior instance's leftovers from preloads pushed after take —
it would wipe the preloads (unsound) — and a dirty-until-parse state would
also break pre-parse observable identity through the public accessors. The
facts-clear therefore stays at take() (`recycle_reset`), and the state keeps
the `-0209` double-reset shape, priced in §1(c) as an O(empty) residual on
regex. The fold idea is CLOSED design-refused for the state; the stamp
mechanism (§2) is where the refusal's mass actually was.

## 4. The emitter change (`ast_based_generator.rs`, protocol emitter only — but reaching ALL 11 artifacts per the `-0209` addendum)

1. Field decls: `memo_fail: ParseScratchLease<StampedFailSet>`,
   `memo_fail_tainted: ParseScratchLease<StampedTaintedFailMap>`,
   `recursion_guard: ParseScratchLease<RecursionGuard>`,
   `semantic_runtime_state: ParseScratchLease<SemanticRuntimeState>`.
2. `new()`: state/guard blocks verbatim from the banked diff; the two
   fail-memo blocks become take + `ensure_capacity(<the K3a/C2 sizing
   formulas, verbatim>)` — on the warm path this is a no-op unless the new
   input needs more than the recycled high-water capacity.
3. `memoized_call` and the memo-stats surface compile VERBATIM UNCHANGED —
   the stamped types expose the same method spellings the raw containers did
   (`contains`/`insert`/`get`/`remove`/`iter`/`len`), with stamp logic
   inside. Zero body-site rewrites (audited: the only access sites are the
   decl, the `new()` init, `memoized_call`'s 6 sites, and the env-gated
   debug-stats iteration).
4. The rendered-contract self-check pins re-anchored to the new spellings
   (wrap-tolerant, as `-0209` learned).
5. The bootstrap-mode annotation pair regenerates with the same surfaces
   (ALL 11 artifacts change uniformly — the `-0209` addendum's corrected
   expectation, enforced by the adapted `review_artifact_delta.sh`).

## 5. Behavior-visibility audit (pre-registered)

- Pre-parse observable identity: identical to `-0209` §5 (take-side full
  reset — no dirty window; preload flow sound per §3).
- Fail-memo semantics: within one parse, stamp-hit ≡ today's hit exactly;
  across parses the containers were per-instance before, and the stamp makes
  a recycled container behave as-if per-instance. The epoch-collision hazard
  of §2 is CLOSED by the stamp (strictly safer than `-0209`'s cleared form,
  which was itself sound via emptiness).
- Debug stats (`report_memo_stats`, env-gated, never in product output): row
  content is the current-generation view — identical populations to today;
  iteration order may differ with capacity history (Fx fixed-seed; same
  caveat as `-0209` §5). The byte-identical oracle battery + cert gates
  adjudicate the product surface.
- Memory: bounded per §2's backstop; guard-supervised.
- Determinism/panic/two-parsers-per-thread/cross-parser-type: unchanged from
  `-0209` §5 (same lease semantics).

## 6. Custody / land gate (pre-registered — the `-0209` gate re-homed verbatim)

- Base = `preserved_probes/regex_perf_probe_fxstore_a4067793`
  (SHA `a406779314b971305da58b6eb7688be233342efd78348c3c62a456dda24b5ee4`),
  copied to scratch + SHA-banked BEFORE any regeneration (done; this
  session's pre-regen 11-artifact bank verified byte-identical to the
  `-0209` bank — the revert continuity proof).
- All-11 regen train (tool rebuilt FIRST; ebnf fixed-point check); the regex
  artifact MUST differ; expected-delta review over ALL 11: every delta must
  be exactly the four lease field decls + the `new()` construction blocks;
  growth bar ≤5%; an unchanged artifact refuses the run.
- Battery: dual-feature lib suite (incl. the ALL-11 interpreter↔generated
  byte-identical oracle + the recycle pins + the NEW stamp pins:
  `stamped_fail_set_generation_invalidates_prior_parse`,
  `stamped_tainted_map_stamps_and_epochs_gate_replay`,
  `stamped_backstop_clears_only_past_cap`), cert ×3 seeds 0/7/42 (banked
  `268/9/259/0 fully_certified`, spf=0), `ast_shape_contract_gate`,
  `duality_hunt_gate`, `regex_pcre2_compile_oracle_gate`, clippy
  source-strict.
- Candidate probe: fat-LTO `--features "generated_parsers mimalloc_perf"`,
  newer-than-source asserted.
- A/B (the `-0202` harness): 3 base floor-validation rounds (±6% vs banked
  bench 1,619.1 ns), 5 alternated bench rounds (steering only), full-corpus
  paired sweep both sides, per-band decomposition banked either way.
- **Binding acceptance (the `-0197` ratchet):** unrounded candidate corpus
  geomean STRICTLY below the same-session base sweep geomean; verdict flips
  0/2,189; candidate corpus MAX ≤ 425,000 ns. Anything else ⇒ unconditional
  product reversion (evidence banked either way).
- After the accept/revert commit the session proves clean and STOPS (the
  fresh-session contract).
