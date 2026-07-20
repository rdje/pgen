# `PGEN-RGX-0078-0205` — result: **LANDED** (the seventh fix through the `-0197` strict-geomean ratchet; the first fix selected by a post-carrier re-pricing)

## What landed

The pre-registered design (`design_prereg.md`) + exactly one fix — **the
thin-memo direct-index carrier**:

- **LIB** (`mod.rs`, additive): `ThinMemoScratch` — the generation-stamped
  row table (slot = one u64 `gen<<32 | idx+1`, `0` = never written;
  `begin()` invalidates every prior parse's slots with ONE counter bump —
  the design fork's naive per-parse zeroed array was refused in-design
  because its alloc+memset is an ADDED fixed cost on the sub-1 µs band;
  u32-wrap hard-clear once per 2³² parses/thread) + `ThinMemoScratchLease` —
  the RAII thread-slot lease (take at parser construction, return on drop
  via the FIELD's own Drop — no `impl Drop` on the parser, no E0509
  hazard; a second live parser on a thread takes a fresh scratch,
  correct and merely unshared).
- **EMITTER** (`ast_based_generator.rs` + `cascade.rs`): the
  `thin_memo: FxHashMap<(RuleId, usize), ThinTapeMemoEntry>` container →
  `thin_scratch` + dense `thin_entries: Vec<ThinTapeMemoEntry>` +
  `thin_stride`; per-rule `THIN_ROW_<RULE>` consts assigned in SORTED rule
  order (28 rows in the regex artifact — the plan set is a HashSet, sorting
  keeps regeneration deterministic) + `THIN_RULE_COUNT`; the per-rule frame's
  probe = one mul/add + one slot load + one generation compare (vs FxHash
  mix + NEON group probe), insert = `Vec::push` + one slot store behind the
  u32-overflow skip-caching guard (a memo skip is always sound), stale
  eviction = one slot clear. The stamp/taint classes
  (PURE / STORE-READ / never-cache-MUTATING), the segment splice replay,
  and the `-0203` copy lanes (`from_slice`/`extend_from_slice`) are
  UNTOUCHED — the lane ledger holds, no double credit.

Entries pre-size carries the K3a formula (`((len+1)*6).min(32768)`); at
~96 B/entry the dense vec reserves LESS than the map's bucket array did.

## Adjudication (binding `-0197` ratchet — `adjudication.txt`)

- Corpus (2,189 cells): unrounded geomean **1068.450565996314 →
  1038.361702538721 ns = −2.8161%** — strict same-session decrease ⇒
  **LAND**; ≈1.2× outside the ≈2.3% noise span (the third
  individually-measurable win of the ratchet series). Base re-read
  1068.45 vs the banked 1051.88 floor = +1.58% drift (inside the span).
- Verdict flips **0/2,189**; candidate MAX **391,875 ≤ 425,000 ns**
  re-settled bound (director #178; same worst cell `line_725`, itself
  −5.93% vs the base re-read).
- Floorval custody: base bench geomean 1675.7 vs banked 1646.1 = **+1.80%**
  OK.
- Bench steering (secondary): geomean **−2.66%**; no pattern worse than
  +1.14% (`literal_simple` −8.40%, `alternation` −6.72%, `url_simple`
  −4.28%).
- Honest attribution: the `-0204` re-pricing put ≈60.97 ns of identifiable
  machinery behind this fix (insert self 48.13 + map teardown 11.46 +
  bookkeeping 1.38); delivered −30.1 ns on the base re-read ≈ 49% capture —
  the FIRST fix of the series whose delivery landed INSIDE its priced
  30–70% capture band (18.3–42.7 ns) rather than 10–60× above a sliver
  price: pricing whole-mechanism self-time works where instruction-row
  pricing understates.

## Floor re-baseline

Floor of record moves to the candidate's same-session reading: corpus
geomean **1,038.4 ns** (unrounded 1038.361702538721), bench ≈**1,609.1
ns** (≈308× from the 496 µs activation baseline), MAX observed 391,875 ns
(guardrail unchanged at 425,000). The margined <1 µs call-off bar
(unrounded geomean ≤ **950.0 ns**) now needs **−8.5%**. Cumulative across
the seven ratchet fixes: 1,263.4 → 1,038.4 ns ≈ **−17.8%**.

## Custody

- Base = `preserved_probes/regex_perf_probe_unitape_6812d8e2` (SHA
  cmp-asserted in-run by `run_ab.sh`). Candidate probe preserved
  `preserved_probes/regex_perf_probe_dindex_d21fab44` (cmp-verified) —
  **the NEXT session's immediate-parent A/B base.**
- Regex artifact `c5e1b425` → **`438bb931`**; ebnf `e2ede238` →
  **`47266fb0`** (`artifacts_pre/post_regen.sha256`); all-11 regen train
  green incl. the ebnf fixed point; expected-delta review **`overall=0`
  over all 11** (`artifact_delta_review.txt`; scratch byte-identical;
  growth ≤+0.23% vs the 5% bar).
- Battery (all green, `battery_summary.txt`): dual-feature lib **1006/0
  (29 ignored)** incl. the ALL-11 interpreter↔generated oracle; cert ×3
  seeds byte-exact `268/9/259/0 fully_certified` (spf=0);
  `ast_shape_contract_gate`, `duality_hunt_gate`,
  `regex_pcre2_compile_oracle_gate`, clippy source-strict.
- Every heavy step memory-guarded (16384 MB, floor 10%), serialized,
  `caffeinate -i`; alternated base/candidate bench rounds; probe-build
  peak 12,001 MB.

## Next

Per the one-fix-per-fresh-session directive this session stops after the
clean commit. NEXT (brand-new session) = re-price on the NEW floor per the
standing discipline, with the `-0204` bench already naming the leading
candidates: the semantic-store SipHash lane (sip write 9.128 + hash_one
5.79 ns on the OLD floor, band-rising, source-pinned to
`FactIndex.by_kind`/`by_scope_and_name`/`predicate_defs` std-RandomState
maps — an FxHash swap, parser-agnostic, determinism-increasing) and the
protocol-memo inserts (12.0 ns via the cyclic core); both must be re-priced
against the LANDED direct-index representation before one is selected.
