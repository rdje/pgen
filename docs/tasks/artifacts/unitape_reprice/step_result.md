# `PGEN-RGX-0078-0204` result — the post-carrier RE-PRICING of the surviving gross/proxy lanes

Leaf `RGX-0078.5.j.4`, session #179, 2026-07-20. Read-only measurement over the
LANDED `-0203` unified-tape representation (preserved probe
`unitape_6812d8e2`, SHA-asserted before every step). No parser, emitter,
generated artifact, corpus expectation, release floor, or settled MAX changed.

## Verdict

**Every `-0197` surviving gross/proxy lane is individually sub-noise on the
new floor — none is a solo fix.  The re-pricing surfaces a NEW largest
identifiable mechanism instead: the thin-memo INSERT machinery, ≈61.0 ns of
zero-overlap self-time = 2.5× the noise floor, with a concrete pre-designed
replacement (the `-0186` compact direct-index carrier).  That is the ONE fix
this session selects for the `-0197` ratchet.**

## Capture (fresh, on the landed representation)

The `-0182` recipe reused verbatim: v3 sampler (source SHA `9bb51a53…`),
250 µs `ITIMER_PROF`, teardown qualification green before the corpus was
spent, three sequential bands under `caffeinate` + the 16384 MB guard.  Bands
re-derived from the `-0203` floor sweep itself
(`carrier_core/corpus_candidate.jsonl`, SHA `7b307550…`), whose 2,189 rows
reproduce the floor geomean **1051.8818294596394 ns exactly**; the ≥20 µs
tail (11 cells, 0.78% log-share) stays excluded by design:

| band | cells | log-share (weight) | stored PCs | target PCs | verdict identity |
|---|---:|---:|---:|---:|---|
| sub-1 µs | 1,072 | 0.435780 | 72,987 | 8,203 | PASS |
| 1–2.5 µs | 725 | 0.349170 | 72,204 | 8,003 | PASS |
| 2.5–20 µs | 381 | 0.207263 | 57,794 | 7,128 | PASS |

Zero drops; 100% target-disassembly coverage; ~2.7–2.8× the `-0182` target
statistics.  Noise floor carried onto the new floor:
28.8/1263.4 × 1051.88 = **23.978 ns**.

## The re-derived role map (this binary's own fingerprints)

The parser `self` register is **split by symbol at this vintage** (`x19` in
`cascade_match_piece`, `x20` in the atom closure) — the old x20-only map
would silently halve the census.  Both symbols expose an IDENTICAL 26-offset
set, partitioned into 8 roles, each qualified by a machine/dataflow
fingerprint (`classify_and_reprice.py`, REFUSE-on-unknown).  The landed
program is directly visible in the layout: the two tape lanes are ONE Vec
(0x278/0x280/0x288, 8-byte elements), the `-0199` duplicate checkpoint word
is gone (7 co-captured sources), and THREE old roles have vanished from the
fused region entirely (trace gate 0x504, coverage snapshot 0x270,
rollback-diagnostic 0x248 — the landed `-0201`/`-0202` observability
boundary).

## Re-priced surviving lanes (weighted share × 1051.88 ns floor)

Tier-1+2 = exact parser-direct offsets + window-qualified derived accesses:

| lane (old banked price) | new gross ns | 30% | 50% | 70% | vs 23.978 noise |
|---|---:|---:|---:|---:|---|
| input view (12.316 `-0192`) | **12.247** | 3.67 | 6.12 | 8.57 | sub-noise at EVERY fraction |
| checkpoint state (in `-0197` regions) | **10.285** | 3.09 | 5.14 | 7.20 | sub-noise |
| recursion guard (`-0200` landed residue) | 5.308 | 1.59 | 2.65 | 3.72 | sub-noise |
| position forwarding (0.828 `-0194`) | 4.800 | 1.44 | 2.40 | 3.36 | sub-noise |
| unified tape residue (`-0203`'s own lane) | 3.188 | 0.96 | 1.59 | 2.23 | sub-noise |
| thin-memo lookup (3.825 `-0186`) | 1.384 | 0.42 | 0.69 | 0.97 | sub-noise (shrank 2.8×) |
| G1-C arena residue, in-target (26.7 `-0171` region) | 0.881 | 0.26 | 0.44 | 0.62 | sub-noise |
| memo taint signal | 0.280 | — | — | — | sub-noise |

**Adjudication of the `-0197` question:** thin lookup, input view, and
position forwarding — and every other in-target proxy lane — are refused as
solo implementation leaves on the landed representation.  The input view is
the only lane that did NOT shrink (12.32 → 12.25 ns across six landed fixes:
byte-fetch is irreducible transport), and even its 70% fantasy capture is
0.36× noise.

## Where the mass actually sits now (out-of-target populations, self-time)

| population | weighted ns | note |
|---|---:|---|
| spine_other (protocol/cascade dispatch self) | 191.4 | the `-0162` spine population, still the largest; no designed mechanism |
| allocator (mimalloc self, in-binary) | 126.7 | perfect-allocator ceiling banked −15.2% (mimalloc→nfa) |
| external_or_injected (libsystem: memmove, timer) | 114.9 | not addressable by parser change |
| other_text (outlined tails, clones, hashing) | 83.4 | includes the sip lane below |
| arena_alloc (`alloc_extend` ×5 monomorphs) | 73.3 | the `-0171` BUILD-pass arenas; V1 stays closed |
| build_value (`to_shaped_value` + `cascade_build_*`) | 71.5 | V1 closed measured-exhausted; re-confirmed |
| **hashbrown_map (memo INSERT machinery)** | **62.4** | **the selected fix — breakdown below** |
| teardown_drop (per-parse drops) | 56.2 | G3-class; RegexParser 9.5 + NodeArena 9.5 + memo map 11.5 + … |
| harness (`parse_once_timed` self + main) | 49.7 | measurement cost, not parser cost — scope honesty |
| semantic_runtime (band-rising store work) | 49.9 | + the sip-hash lane; the `-0171` named population |
| vec_growth / tape_helpers | 26.7 / 18.4 | tape decode is build-pass transport |

## ⭐ The selected mechanism — thin-memo direct-index carrier (ONE fix)

Caller-pinned, zero-overlap identifiable machinery on the geomean band:

- `HashMap::insert h6884…` self — **48.127 ns** — its bl-sites are EXACTLY
  the fused thin-memo stores (`cascade_match_atom` closure ×14,
  `cascade_match_conditional` ×6, `piece`/`alternative`/
  `extended_class_content`/`code_content` ×3 each, + `try_parse_bare`
  wrappers) ⇒ this monomorph IS the
  `FxHashMap<(RuleId,usize), ThinTapeMemoEntry>` insert;
- the map's teardown `drop_in_place` (two monomorphs) — **11.463 ns**;
- in-target RawTable bookkeeping (tier-1 `thin_memo_table`) — **1.384 ns**;
- lookup-side hash+NEON-probe work sits inside the same replaced
  representation (priced 1.38 in-target; more in `other_text` hash_one rows).

**Total identifiable: ≈60.97 ns = 2.54× noise (gross).** Capture cases:
30% = 18.29 (0.76× noise), 50% = 30.49 (1.27×), 70% = 42.68 (1.78×).
Replacement (the `-0186` pre-specified compact direct-index: one u32 row per
thin-memoized fused rule × (len+1) + a dense entry vec) eliminates hashing,
group probing, ctrl-byte maintenance, key storage, amortized growth/rehash,
and the bucket-walking drop; it keeps the entry payload copy (owned by the
`-0203` ledger — no double credit) and adds one small per-parse row
allocation + reset (O(len), sentinel fill).  Strict net banked before A/B:
**0** (the `-0197` accounting rule); the binding adjudicator is the corpus
ratchet vs base `unitape_6812d8e2` with unconditional reversion.

Under the `-0170` amended rule this lever's population is real and large; its
conservative 30% case alone is sub-noise, so the license to spend the chain
rests on (a) the 2.54× gross magnitude — bigger than any machinery the six
landed fixes had in hand — and (b) the landed program's systematic
width/lane-class understatement (`-0202` priced 4.4 → delivered −64.8;
`-0203` priced 0.53 → delivered −43.1), with (c) the strict-geomean ratchet
guaranteeing the floor cannot regress on a miss.

## 💡 Named for the NEXT session (not this one's fix)

**The semantic-store SipHash lane:** `core::hash::sip::Hasher::write` 9.128 ns
+ `BuildHasher::hash_one` rows 5.79 ns, band-rising (44/776/1300), source-pinned
to std-`RandomState` maps in the semantic store — `FactIndex.by_kind:
HashMap<String,…>`, `by_scope_and_name: HashMap<(usize,FactNameKey),…>`,
`predicate_defs: HashMap<String,…>` (semantic_runtime.rs:2127/2166/2408) —
while the store's OWN `directives_by_rule` maps are already FxHashMap.  An
FxHash swap there is parser-agnostic, mechanical, and determinism-increasing
(FxHash is fixed-seed; RandomState is per-process random).  Gross 9.1–14.9 ns;
a `-0170` batch member or a small solo unit for a later ratchet round.

Also banked for the record: the protocol-graph memo inserts (h50f4 8.23 +
hccf7 3.80 ns) still run on the geomean band via the cyclic core
(`parse_pattern` 18.70 ns band-rising) — the direct-index mechanism
deliberately does NOT touch the protocol memo this round (shared with
observed parses; different soundness surface).

## Reproduction

```sh
python3 docs/tasks/artifacts/unitape_reprice/derive_band_inputs.py --output-dir /tmp/bands
bash -n docs/tasks/artifacts/unitape_reprice/run_capture.sh
python3 docs/tasks/artifacts/unitape_reprice/analyze_capture.py
python3 docs/tasks/artifacts/unitape_reprice/classify_and_reprice.py
```

The first and last two commands reproduce `input_derivation.txt`,
`capture_analysis.txt`, and `role_reprice.txt` byte-for-byte from the banked
raw captures (`raw_custody.json` pins their SHAs; raw captures themselves are
stochastic per run — the `-0182` custody convention).

## Scope

- Runtime/emitter/generated artifacts/product behavior: unchanged.
- PCRE2 verdicts, the accepted 1,051.9 ns floor, and the re-settled
  425,000 ns MAX bound: unchanged.
- mdBook/contracts/LIVE tracker: unchanged (campaign-internal measurement —
  the `-0162`…`-0196` precedent).
- NEXT (this session, per the fresh-session contract): the ONE fix —
  design-prereg + implementation of the thin-memo direct-index carrier,
  adjudicated by the binding `-0197` ratchet (strict unrounded same-session
  corpus-geomean decrease, flips 0/2,189, MAX ≤ 425,000 ns settled), full
  battery + all-11 regen train, unconditional reversion on a miss.
