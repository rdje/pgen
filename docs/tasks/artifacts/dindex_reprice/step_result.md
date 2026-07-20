# `PGEN-RGX-0078-0206` result — the post-direct-index RE-PRICING of the surviving lanes

Leaf `RGX-0078.5.j.4`, session #180, 2026-07-20. Read-only measurement over the
LANDED `-0205` thin-memo direct-index representation (preserved probe
`dindex_d21fab44`, SHA-asserted before every step). No parser, emitter,
generated artifact, corpus expectation, release floor, or settled MAX changed.

## Verdict

**Every in-target lane is individually sub-noise on the 1,038.4 ns floor, and
the landed direct-index carrier is structurally verified in the new binary
(the hashbrown thin-memo machinery is GONE; its residue is ≈2.1 ns of
row/entry access vs the replaced ≈61 ns).  The largest surviving un-owned
identifiable mechanism is the `-0204`-pre-named semantic-store SipHash lane:
≈17.36 ns of zero-overlap hashing self-time = 0.73× the noise floor, with a
mechanical, parser-agnostic, determinism-increasing FxHash swap as the
pre-specified replacement.  That is the ONE fix this session selects under
the `-0197` ratchet — priced honestly BELOW noise even at 100% capture, so
the ratchet's unconditional reversion is the adjudicator: a strict unrounded
same-session corpus-geomean decrease lands it; anything else reverts it and
closes the lane measured-exhausted (the `-0156` V1-M3 precedent).**

## Capture (fresh, on the landed representation)

The `-0182`/`-0204` recipe reused verbatim: v3 sampler (source SHA
`9bb51a53…`), 250 µs `ITIMER_PROF`, teardown qualification green before the
corpus was spent, three sequential bands under `caffeinate` + the 16384 MB
guard.  Bands re-derived from the `-0205` floor sweep itself
(`thin_memo_direct_index/corpus_candidate.jsonl`, SHA `d11ea9f8…`), whose
2,189 rows reproduce the floor geomean **1038.361702538721 ns exactly**; the
≥20 µs tail (11 cells, 0.78% log-share) stays excluded by design:

| band | cells | log-share (weight) | stored PCs | target PCs | verdict identity |
|---|---:|---:|---:|---:|---|
| sub-1 µs | 1,079 | 0.438358 | 102,401 | 11,746 | PASS |
| 1–2.5 µs | 720 | 0.347338 | 104,447 | 11,571 | PASS |
| 2.5–20 µs | 379 | 0.206509 | 110,328 | 13,198 | PASS |

Zero drops; 100% target-disassembly coverage; ~1.4–1.9× the `-0204` stored /
target statistics.  Noise floor carried onto the new floor:
28.8/1263.4 × 1038.36 = **23.670 ns**.

## The re-derived role map (this binary's own fingerprints)

The parser `self` register is split by symbol exactly as at `-0204` (`x19`
in `cascade_match_piece h6aaef5c8…`, `x20` in the atom closure `hd69dd0d3…`).
Both symbols expose an IDENTICAL 28-offset set, partitioned into 9 roles,
each qualified by a machine/dataflow fingerprint
(`classify_and_reprice.py`, REFUSE-on-unknown).  The landed `-0205` program
is directly visible in the layout:

- the old thin-memo FxHashMap RawTable offsets (0x4B0–0x4C8) are **GONE**,
  and the REFUSE-on-NEON-probe check confirms no ctrl-group machinery in the
  thin-memo windows;
- the direct-index carrier appears instead: the generation-stamped row table
  (data 0x280 / len 0x288, 8-byte slots, `lsr #32` generation extract vs the
  current-generation word 0x290) + the per-parse `(len+1)` row stride 0x538
  multiplied by immediate sorted `THIN_ROW_*` consts (`madd` / `add …lsl`);
- the dense 96-byte `thin_entries` vec (data 0x2A0 / len 0x2A8, `#0x60`
  umaddl addressing, epoch-validated against the checkpoint word 0x258);
- the unified tape moved to 0x2B0–0x2C0 (one Vec, 8-byte words, `grow_one`);
  input view 0x360/0x368; arena 0x370; position/furthest 0x4E8/0x530 — every
  prior role re-qualified at its new offset.

## Re-priced in-target lanes (weighted share × 1038.36 ns floor)

Tier-1+2 = exact parser-direct offsets + window-qualified derived accesses:

| lane | new gross ns | 30% | 50% | 70% | vs 23.670 noise |
|---|---:|---:|---:|---:|---|
| input view (12.25 at `-0204`) | **12.916** | 3.87 | 6.46 | 9.04 | sub-noise at EVERY fraction (CLOSED: irreducible transport) |
| checkpoint state | 9.330 | 2.80 | 4.67 | 6.53 | sub-noise |
| position forwarding | 5.903 | 1.77 | 2.95 | 4.13 | sub-noise |
| recursion guard | 4.175 | 1.25 | 2.09 | 2.92 | sub-noise |
| unified tape residue | 3.607 | 1.08 | 1.80 | 2.52 | sub-noise |
| **thin-memo rows (landed carrier)** | **1.291** | — | — | — | the `-0205` probe machinery: sub-noise BY DESIGN |
| **thin-memo entries (landed carrier)** | **0.779** | — | — | — | ditto |
| G1-C arena residue (tier-2) | 0.892 | — | — | — | sub-noise |
| memo taint signal | 0.013 | — | — | — | sub-noise |

**The `-0205` fix verified in-profile:** the replaced container machinery
(insert 48.13 + teardown 11.46 + bookkeeping 1.38 ≈ 61.0 ns at `-0204`) is
now rows 1.29 + entries 0.78 + the outlined `ThinMemoScratch::lookup`
1.43 ns; the surviving `thin_entry_push` population (16.71 ns, bl-sites =
exactly the fused thin-memo stores) is the entry PAYLOAD copy owned by the
`-0203` ledger — no double credit, not a lever here.

## Where the mass sits now (out-of-target populations, self-time)

| population | weighted ns | note |
|---|---:|---|
| spine_other (protocol/cascade dispatch self) | 203.0 | still the largest; no designed mechanism |
| allocator (mimalloc self, in-binary) | 125.0 | perfect-allocator ceiling banked −15.2% |
| external_or_injected (libsystem: memmove, timer) | 109.1 | not addressable by parser change |
| other_text (outlined tails, clones, hashing) | 98.4 | includes the sip lane below |
| build_value (`to_shaped_value` + `cascade_build_*`) | 74.6 | V1 closed measured-exhausted; re-confirmed |
| arena_alloc (`alloc_extend` ×6 monomorphs) | 72.9 | the BUILD-pass arenas; V1 stays closed |
| semantic_runtime (band-rising store work) | 51.5 | contains the selected sip users |
| teardown_drop (per-parse drops) | 51.4 | G3-class tail: RegexParser 14.39 + NodeArena 10.61 + FactIndex 2.60 + store-vec drops; public-boundary design required (see below) |
| harness (`parse_once_timed` 44.11 + main) | 49.5 | measurement cost, not parser cost — scope honesty |
| vec_growth / tape_helpers | 27.7 / 19.0 | tape decode 17.50 is build-pass transport |
| **thin_entry_push** (`push_mut h2d2239…`) | **16.7** | the `-0203`-owned payload copy (see above) |
| hashbrown_map (protocol-memo inserts) | 13.5 | h50f4 8.23 + hccf7 4.02 — out of scope (below) |

## ⭐ The selected mechanism — semantic-store SipHash → FxHash (ONE fix)

Caller-pinned, zero-overlap identifiable hashing self-time on the geomean
band, source-pinned to the THREE std-`RandomState` maps the `-0204` census
already named (verified unchanged at HEAD):

- `core::hash::sip::Hasher::write ha755b248…` self — **10.274 ns** —
  band-rising 75/821/3656, called by the four `hash_one` monomorphs, of
  which only the two semantic-store ones are dynamically hot;
- `BuildHasher::hash_one hc6b44c1d…` — **4.030 ns** — bl-sites:
  `FactIndex::insert`, `rollback_to_labeled_slow`,
  `any_with_name_at_scope`, `reserve_rehash` — the
  `by_scope_and_name: HashMap<(usize, FactNameKey), Vec<usize>>` map
  (`semantic_runtime.rs:2166`);
- `BuildHasher::hash_one hafc51c93…` — **3.054 ns** — bl-sites:
  `rollback_to_labeled_slow`, `evaluate_predicate`, `count_for_kind`,
  `positions_for_name`, `reserve_rehash` — the
  `by_kind: HashMap<String, FactKindIndex>` map (`semantic_runtime.rs:2127`);
- the protocol-memo-side monomorph `h158160dd…` is dynamically cold (<20
  samples in every band) — the sip row above is store-driven;
- adjacent but NOT credited: `FactIndex::insert` container self 6.758 ns
  (RawTable work that stays), `SemanticRuntimeState::new` 9.230 ns (setup).

**Total identifiable: ≈17.359 ns = 0.733× noise (gross).** Capture cases:
30% = 5.21, 50% = 8.68, 70% = 12.15 ns — all sub-noise; even 100% capture
is sub-noise.  The replacement (`FxHashMap` for `by_kind`,
`by_scope_and_name`, and the state's `predicate_defs` — the third
`-0204`-named map, `semantic_runtime.rs:2408`, empty on the regex path so
its boundary conversion is free) eliminates every sip compression round on
the store's parse path; FxHash's replacement cost is a short word-mix loop,
priced per band by construction.  Public API unchanged (all three are
private fields; `set_predicate_defs`/`reset_for_new_parse` keep their std
signatures and convert at the boundary).  The store's own
`directives_by_rule` maps are already FxHash (the `.5.a` fix) and
`fact_kinds` is empty on the generated-parser path (hashbrown's empty-map
short-circuit — never hashed).

**License (recorded honestly):** this is a `-0170` SMALL SOLO UNIT whose
price is sub-noise — selected anyway because (a) it is the largest un-owned
zero-overlap mechanism left on the floor, (b) the `-0204` session
pre-adjudicated it BY NAME as a licensed later-round unit ("parser-agnostic,
mechanical, determinism-increasing"), (c) it is a fix-hierarchy level-2
(store) change benefiting every parser, and (d) the binding `-0197` ratchet
adjudicates with unconditional reversion, so a sub-noise miss costs nothing
and is recorded as the lane's measured exhaustion.  The honest risk: a
revert outcome is a live possibility; that outcome is itself informative
(it would leave NO sub-30 ns lever standing and force the population-class
forks below to the director).

## 💡 Banked for LATER rounds (not this one's fix)

- **The protocol-memo inserts** (h50f4 8.23 + hccf7 4.02 ns): still running
  on the geomean band via the cyclic core; deliberately out of scope again —
  shared with observed parses (a container change there is a different
  soundness surface; a hasher-only change would need its own design fork).
- **The teardown population** (51.4 ns): the largest identifiable G3-class
  machinery — but killing per-parse drops (parser/arena/store-vec recycling
  à la the `-0205` TLS-lease pattern) reaches the embedding/public boundary
  and is therefore a MUST-SURFACE design fork BEFORE code (process law
  #175), flagged to the director in this session's findings, not
  unilaterally implemented.
- **spine_other** (203.0 ns): the standing `-0162` population; no designed
  mechanism exists; any attack is a representation-class research fork.

## Reproduction

```sh
python3 docs/tasks/artifacts/dindex_reprice/derive_band_inputs.py --output-dir /tmp/bands
bash -n docs/tasks/artifacts/dindex_reprice/run_capture.sh
python3 docs/tasks/artifacts/dindex_reprice/analyze_capture.py
python3 docs/tasks/artifacts/dindex_reprice/classify_and_reprice.py
```

The first and last two commands reproduce `input_derivation.txt`,
`capture_analysis.txt`, and `role_reprice.txt` byte-for-byte from the banked
raw captures (`raw_custody.json` pins their SHAs; raw captures themselves are
stochastic per run — the `-0182` custody convention).

## Scope

- Runtime/emitter/generated artifacts/product behavior: unchanged.
- PCRE2 verdicts, the accepted 1,038.4 ns floor, and the settled 425,000 ns
  MAX bound: unchanged.
- mdBook/contracts/LIVE tracker: unchanged (campaign-internal measurement —
  the `-0162`…`-0204` precedent).
- NEXT (this session, per the fresh-session contract): the ONE fix —
  design-prereg + implementation of the semantic-store FxHash swap,
  adjudicated by the binding `-0197` ratchet (strict unrounded same-session
  corpus-geomean decrease vs base `dindex_d21fab44`, flips 0/2,189,
  MAX ≤ 425,000 ns settled), full battery + all-11 regen train,
  unconditional reversion on a miss.
