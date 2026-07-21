# PGEN-RGX-0078-0216 — the `.5.j.1` LAST-ATTEMPT GO/NO-GO: STEP-0 paper gate

Session #187, 2026-07-21. Read-only paper adjudication (no code, no build, no
measurement — the banked records only). Executes the director's `-0216` ruling
block (tree, 2026-07-21): *"try this last attempt but first ensure if
implemented it will deliver. No need to change the code for nothing."*

Inputs (all banked, none re-measured):
- `-0123` census/design record (`docs/tasks/artifacts/repr_step0/census_report.txt`
  + the tree RESULTS block) — the road's own STEP-0.
- `-0126` corrected partition (barrier=96 / value_licensed=105 / node_locked=33
  / demoted=0; the 33 node_locked = the protocol-facing population).
- `-0171` caller-attribution facts (`docs/tasks/artifacts/g1c_attribution/`).
- `-0215` cluster decomposition (`docs/tasks/artifacts/content_work_decomposition/
  decomposition_content.txt`, custody REFUSE-gated to the `-0213` capture).
- `-0213` role re-price (`docs/tasks/artifacts/carrier48_reprice/role_reprice.txt`)
  for the masses outside the four `-0215` populations.

## 1. The gate criterion (fixed BEFORE the table was filled)

- Floor of record: corpus geomean **1,004.4 ns**. Margined call-off bar
  (#162 + #176): unrounded **≤ 950.0 ns** ⇒ required elimination
  **≥ 54.4 ns (−5.42%)**.
- The ruling's adjudicator: **the delivered-fraction precedent (30–70% of
  eliminable), not the gross surface.** Observed on the last three landed
  fixes: 49% (`-0205`), 42% (`-0207`), in-band (`-0212`); no campaign fix has
  ever delivered above its band top.
- Therefore **PASS ⇔ honest eliminable mass ≥ 54.4 / 0.70 ≈ 77.7 ns.**
  (Below that, even a best-in-history delivery cannot reach the bar, and any
  implementation would be exactly the "change the code for nothing" the
  ruling forbids.)
- Falsification bound for step (c), fixed here as the ruling requires even
  though (c) is only licensed on a paper PASS: the skip-value-build ceiling
  ablation must bound the road-claimed populations at **≥ 77.7 ns** under the
  hardened `-0212` custody protocol, labeled a BOUND (the `.5.i.1` V1–V6
  instrument class), for a GO.

## 2. The road, as recorded

`.5.j.1` protocol-zone value-ization (`-0123` design + `-0134` road
consequence): value forms for the **33 node_locked protocol-facing rules**
(the `-0126` partition: boundary sub-roots `pattern`, `class_zero_width`,
`capture_name`, … + their transparent descendants) + a **value-carrying
protocol memo** (replay-by-reference / value payloads instead of node-content
clones — 426 success + 232 fail clones corpus-wide at the `-0123` census) +
**boundary doomed-build elision** (44 dead pairs ≈10%) + the in-place
extension (reserve-uninit fill-in-place arena builds, eliding the
materialize-then-copy read half).

Not this road (excluded with reasons): re-fusing the match and build passes
(would relitigate the LANDED `-0118` thin-memo split); anything whose sole
effect is fewer allocations/frees (priced at ZERO by the measured
`-0209`/`-0210`/`-0214` law ×3).

## 3. The eliminate-vs-keep table

All ns are the banked current-floor prices. "Eliminable" is the honest
mechanism mass the road could remove; the keep-reason names why the remainder
survives in ANY value-ized design.

| # | population (banked source) | gross ns | eliminable | keep-reason for the remainder |
|---|---|---|---|---|
| E1 | `ParseContent::to_shaped_value` (`-0215` build_value; head cluster 14.53 incl. the 8.5 ns pinned discriminant pointer-chase) | 21.71 | **11.5** | the value WRITES relocate to direct builds, they do not vanish — `-0129` measured: value builders still pay staging (`cascade_build_piece` 22.14 IS a post-`-0129` value/mixed builder; `cascade_build_value_literal` 4.20 is a PURE value fn); eliminable = the re-walk/double-handling half |
| E2 | node-form/barrier build fns on the protocol chain (`cascade_build_alternative` 10.83 + `_char_class` 1.31 + `_conditional` 0.27 + misc ≈0.2; `-0215`) | 12.6 | **5.0** | same `-0129` law — a value form keeps staging + tape dispatch; only wrapper-node scaffolding dies (≈40%) |
| E3a | `ParseContent::clone` — the protocol `memoized_call` insert/hit clones (the `-0123` #16 corrected label; `-0213` role_reprice) | 8.08 | **8.08** | granted in FULL — conditional on replay-by-reference soundness, an OPEN `-0123` question never adjudicated (risk, not credit) |
| E3b | `Vec::clone` 3.25 + `to_vec` 4.61 + `SmallVec::clone` 1.78 (`-0213`; caller-UNATTRIBUTED — the raw capture has no stacks) | 9.63 | **4.8** | semantic-runtime snapshots + guard also clone; granting all would repeat the `-0171` aggregate-row error |
| E3c | hashbrown fat-memo inserts (8.90 + 3.96 + rehash 0.85 + misc; `-0213`) | 14.49 | **3.0** | map machinery (hash/probe/bucket/growth) exists in ANY memo design; `-0212` already took the payload-size elasticity win (72→48 B); a value payload shrinks only the payload-memcpy share |
| E3d | external-memops share of memo-clone memcpy (directional bound: `-0171` memo-insert 30.7 ns on the 1,263.4 floor ⇒ ×0.795 ≈ 24.4 scaled; that mass = alloc_extend+memmove shares only, partly already counted in arena_alloc/E3c; `-0212` slimmed payloads since) | ≤24.4 | **6.0** | no current-floor stack data exists to attribute more; see robustness R1 — even granting the FULL 24.4 the gate fails |
| E4 | reserve-uninit fill-in-place read-half of the `alloc_extend` copy loops (the `-0215` arena_alloc adjudication's own banked price) | 8–12 | **10.0** | the staging writes are content work in ANY contiguous-slice representation — the `mod.rs:1155` re-entrancy constraint governs WHERE bytes are written, not WHETHER |
| E5 | boundary doomed builds — 44/435 dead pairs ≈10.1% of the 40-B pair monomorph 23.91 (`-0123` census × `-0215` price) | 2.4 | **2.4** | granted in full (≤0.5 ns overlap with E4's pair share, noted) |
| E6 | `smallvec::from_slice` (`-0213`; caller-unattributed) | 10.91 | **1.5** | most plausibly match-pass children staging — the road does not touch the match pass; no evidence licenses more |
| — | **TOTAL honest eliminable** | | **≈52.3** | |

KEEP (unaffected, with the governing record): spine_other 186.6 (ABI/frame;
protocol orchestration frames are unchanged by payload type; compiler-tuning
+ arena-ref ABI refused), allocator 122.6 + teardown_drop 50.9 (the
`-0209`/`-0210`/`-0214` measured law ×3 — the road's fewer-allocations
promise prices at ZERO), external/kernel non-clone remainder ≈100+, other_text
remainder ≈60, harness 50.4, semantic_runtime 49.4 (REQUIRED fact content —
value representation orthogonal), in-target lanes ≈39.3, vec_growth 28.6
(match pass), build_value staging remainder ≈56, arena_alloc remainder ≈57,
tape_helpers 20.3 (the tape replay drives value construction in ANY variant
of this road), thin_entry_push 16.4 (the thin memo is already value-free).

Draw-down consistency: every eliminable line is drawn from within its parent
mass (E1+E2 ⊂ build_value 72.66; E4+E5 ⊂ arena_alloc 69.42; E3a/E3b/E6 ⊂
other_text 83.0; E3c ⊂ hashbrown_map 14.5; E3d ⊂ external 113.4) — no
double-counting across rows.

## 4. Adjudication

- Honest eliminable ≈ **52.3 ns** < the required **77.7 ns** ⇒ **the paper
  gate FAILS.**
- Equivalent statement: at the adjudicating 30–70% band the road delivers
  **15.7–36.6 ns = −1.6…−3.6%** against the needed **−5.42%**. Even
  IMPOSSIBLE 100% delivery of the honest eliminable (52.3 < 54.4) does not
  reach the bar.
- **⇒ NO-GO.** Step (c), the ceiling ablation, is NOT licensed ("if and only
  if the paper gate passes"). Per the ruling: **the exhaustion record is
  complete INCLUDING the last road, and the call-off proceeds** (the #162
  closing wave, honest-floor form per the `-0215` recommendation).

## 5. Robustness (why the verdict cannot be granted away)

Single-premise maximal grantings, each replacing an honest line with its full
gross mass:

| granting | total eliminable | gate (≥77.7) |
|---|---|---|
| R1: E3d full 24.4 (the whole scaled `-0171` bound) | 70.7 | FAIL |
| R2: E1 full 21.71 (conversion vanishes entirely) | 62.5 | FAIL |
| R3: E3c full 14.49 (hash-map machinery free) | 63.8 | FAIL |
| R4: E6 full 10.91 (from_slice owned by the road) | 61.7 | FAIL |
| R5: E3b full 9.63 (every clone is a memo clone) | 57.1 | FAIL |
| R6: E2 full 12.6 (node builds vanish entirely) | 59.9 | FAIL |

**Every single maximal granting still fails.** Crossing 77.7 requires
stacking two or more (e.g. R1+R2 = 80.9), where each is individually
contradicted by a measured record (R2 vs the `-0129` staging persistence;
R1 double-counts E3a/E3c against the `-0171` symbol set; R3 vs the memo
machinery's existence in any design) — and even then GO would rest on 70%
delivery, never observed (best ever: 49%). At the OBSERVED ≈42–49% delivery,
even the absolute everything-dies ceiling (≈116 ns, granting every line in
full simultaneously) delivers ≈49–57 ns — still at-or-under the 54.4 ns gap.

Measured cross-check: `-0134` built, fully proved, and measured a
mechanically-similar clone+wrapper elision (the carrying chain) — it
delivered −0.96% corpus (≈29 ns on the 2,991 ns floor), in its priced band
and sub-bar, and was reverted. The protocol zone's populations on today's
floor are comparable or smaller in gross mass.

## 6. What this record is

The `.5.j.1` road's delivery question is answered on paper, from banked
measurements, without touching the code — exactly the assurance the director
gated on. The road is not "untried" anymore: it is **priced, and the price
cannot pay the bar**. With it, every road on the 1,004.4 ns floor carries a
closed / refused / un-ownable / required / sub-noise / **priced-below-bar**
verdict, and the exhaustion record is complete.
