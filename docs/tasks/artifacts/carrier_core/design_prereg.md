# `PGEN-RGX-0078-0203` — carrier core: the JOINT replacement/overlap contract (pre-registered BEFORE code)

Leaf `RGX-0078.5.j.4` ordered member 6 of the `-0197` held program
(`docs/tasks/artifacts/held_carrier_batch/execution_contract.md`), session
#178, 2026-07-20. The `-0197` dependency map: *"G1-C, memo segment-copy
elision, packed events, and the unified derivation tape share
representation/range/copy decisions. They require one design before
implementation so the same copy or metadata lane is never credited twice."*
This document IS that joint design. It is written and banked before any code
is touched; then exactly ONE fix is implemented and adjudicated by the
binding `-0197` strict same-session corpus-geomean ratchet.

## 1. The four coupled members and their evidence tiers

| member | priced | tier | source |
|---|---:|---|---|
| G1-C atom carrier (arena stride ×72 + `RefCell` borrow + memcpy + boundary push per atom) | 26.7 ns | attributed-region | `-0171` caller-attributed `cascade_match_*` mass |
| memo segment-copy (2 `SmallVec::from_slice` per thin-memo success insert) | 30.7 ns | attributed-region | `-0171` caller-attributed memo-insert mass |
| packed events (16 → 8 B `DerivEvent` word) | 0.530 ns strict (copy-linear ceiling 2.94) | exact-current | `-0191` HOLD (lossless word + wide escape designed) |
| unified derivation tape (merge `deriv_events` + `deriv_boundary`) | 0 strict (boundary-metadata gross 1.453) | gross | `-0196` FEASIBLE-HOLD (losslessness + order proof complete) |

All numbers are current-side targets, NOT banked savings (`-0197`
accounting); every candidate owes replacement cost; net is measured only by
the A/B.

## 2. The fork and the one licensed fix (recorded, then proceeded on)

Three implementable units exist inside the coupled set:

- **(A) the unified PACKED tape** — one `Vec<TapeWord>` of 8-byte tagged
  address words replacing BOTH lanes; implements `-0191` (event packing) and
  `-0196` (tape unification) as ONE representation change and halves the
  memo-segment copy LANES as a construction consequence;
- **(B) memo-only segment-copy elision** — blocked as a first move: its copy
  lanes are ranges OF the tape, so its design depends on which tape
  representation lands (the exact coupling the `-0197` map names);
- **(C) G1-C node-width/arena surgery** — independent of the tape word but
  shares the boundary-push lane; `-0195` already REFUSED boundary-identity
  narrowing (the 8-byte arena reference is the minimum lossless identity).

**Recommendation, proceeded on (the `-0196` verdict verbatim: the unified
carrier is "best batched with the already-designed event-width carrier
rather than charged as an isolated lane"): (A), the unified packed tape.**
(B) re-prices against the landed representation in a later leaf; (C)'s
node-width/arena part is untouched and re-prices likewise.

## 3. Current-vintage census (custody `c3d91959`, `site_census.sh` → `site_census.txt`)

Whole-file `perl -0777` counts (the `-0173` standing rule). Regex artifact:

- Match-pass writes: **204 event pushes** (OrWinner 83 / QuantCount 69 /
  OptPresent 52 / TokStart 0 / TokEnd 0 — the regex artifact carries ZERO
  terminal events, the banked `-0129` fact) + **204 patch sites** +
  **268 boundary pushes**.
- The SECOND-LANE bookkeeping this fix deletes: **4,321 `len()` mark reads**
  (2,221 ev + 2,100 b), **4,343 truncates** (2,213 ev + 2,130 b), **802
  `copy_within` compactions** (401 + 401) — each pair becomes ONE operation.
- Thin memo (28 cyclic fused rules): success insert = **2×28 = 56
  `SmallVec::from_slice`** copies; hit splice = **2×28 = 56
  `extend_from_slice`** (28 ev + 28 b) — each pair becomes ONE.
- Build reads: 188 `deriv_next_event()` + 256 `deriv_next_boundary()`
  (signatures preserved — value-build logic untouched by construction).
- Region shape: 235 match/build fn pairs, 19 sub-root orchestrators, 30
  scanners.
- All-11 presence: every artifact carries the tape (SV: 1,643 ev pushes /
  2,479 b pushes / 219 thin-memo uses; json/rtl/vhdl/semantic: tok-event
  populations 56–1,432) ⇒ all-11 regen + expected-delta review is owed.

## 4. The representation (the `-0196` proven design, engine-owned)

**LIB (`rust/src/ast_pipeline/mod.rs`, additive):** `TapeWord<'input>` — an
8-byte tagged address word (`Copy`, `needs_drop=false`, layout-pinned by a
new test):

- internal storage: one `*const ParseNode<'input>` +
  `PhantomData<&'input ParseNode<'input>>` (provenance-preserving; size 8);
- **tag 0b000** — an unmodified, aligned `&'input ParseNode` pointer
  (`ParseNode` alignment ≥ 8 ⇒ 3 zero low bits; provenance kept intact);
- **tags 1–6** — the event variants: `OrWinner`/`QuantCount`/`TokStart`/
  `TokEnd` (payload `<< 3 | tag`, narrow limit `usize::MAX >> 3`) +
  `OptPresent(false)`/`OptPresent(true)` (payload-free tags); event words are
  built with `ptr::without_provenance` and are NEVER dereferenced;
- **tag 0b111** — the wide escape: the header word's payload bits name the
  variant; ONE following raw `usize` word carries the full payload (so no
  input/count/index cap exists — `usize::MAX` round-trips);
- the ONLY unsafe is inside the engine accessor
  `TapeWord::boundary() -> &'input ParseNode<'input>`, which HARD-checks tag
  0 before the encapsulated dereference (drift ⇒ loud panic, the MTB-A
  stance; emitted code contains no `unsafe`);
- decode: `TapeWord::event_len()` (1 or 2 words) + a total decoder
  reconstructing `DerivEvent` (which STAYS as the decode-result type — the
  build logic keeps matching on it);
- auto-trait surface preserved exactly:
  `unsafe impl Send/Sync for TapeWord<'input> where &'input ParseNode<'input>: Send/Sync`
  (a `TapeWord` is semantically either an integer word — no provenance,
  never dereferenced — or exactly that reference);
- **statically-narrow patch proof:** every PATCHED placeholder is narrow by
  construction — `OrWinner(i)` has `i <` the site's branch count (a small
  compile-time constant), `QuantCount(n)` has `n ≤ SAFETY_LIMIT = 10,000`,
  `OptPresent` is payload-free — so a patch NEVER needs the wide escape and
  in-place `tape[mark] = word` is total (debug-asserted in the encoder).
  Only appended `TokStart`/`TokEnd` positions can be wide, and appends
  choose 1 or 2 words at push time (no shifting, the `-0196` §3 license).

**Memo entry (`mod.rs`, additive):** `ThinTapeMemoEntry<'input>` — the
`ThinDerivSegMemoEntry` twin with ONE segment:
`outcome: Option<(usize, SmallVec<[TapeWord<'input>; 6]>)>` (inline 6 words
= 48 B covers the census common segment: 1 placeholder + 0–2 events + 0–1
boundary; spill semantics unchanged). Same `stamp` taint classes verbatim.
The old `ThinDerivSegMemoEntry` is retired in-slice AFTER the all-11
regen fixed point proves no artifact references it (the C3 additive
transient-migration precedent); if any friction appears it is instead
retained one slice with a retirement note — pre-adjudicated fallback.

## 5. The emitter replacement map (all sites, cascade-plan-gated, parser-agnostic)

1. Struct fields: `deriv_events`/`deriv_boundary`/`deriv_ev_cursor`/
   `deriv_b_cursor` → `deriv_tape: Vec<TapeWord<'input>>` +
   `deriv_cursor: usize` (`deriv_pos` unchanged). Init capacity
   `((input.len() + 1) * 5).clamp(80, 40960)` (the old two lanes' record
   population in one vec); `clear()` sites merge to one.
2. Helper bodies (`deriv_next_event`/`deriv_next_boundary`): decode the next
   word(s) at `deriv_cursor` — signatures unchanged, so the ENTIRE build
   pass (`value.rs` emission) is untouched by construction.
3. Terminal events (non-regex artifacts): `TokEnd`/`TokStart` pushes →
   `TapeWord::push_event(&mut tape, ev)` (1-or-2-word append).
4. Placeholder push-then-patch (`OrWinner`/`QuantCount`/`OptPresent`): one
   mark, `push(TapeWord::narrow_event(...))`, patch
   `tape[mark] = TapeWord::narrow_event(...)` (total by §4's proof).
5. Speculation scopes / lookahead / zero-length-iteration guard: ONE mark +
   ONE truncate (was 2 + 2).
6. Tournament: one candidate mark; compaction = ONE
   `copy_within(cand_start.., or_mark + 1)` + ONE truncate (the winner's
   interleaved segment is contiguous — the `-0196` §2 pairing proof: all
   22 outlined + 15 direct boundary copies pair with the preceding event
   copy; the one unpaired event copy is the boundary-free winner case,
   which a unified range copy also handles).
7. Boundary call-outs / scan sites: `deriv_boundary.push(arena.alloc(v))` →
   `deriv_tape.push(TapeWord::boundary(arena.alloc(v)))`.
8. Thin memo: ONE `SmallVec::from_slice(&tape[mark..])` at success insert;
   ONE `extend_from_slice` at hit splice; entry type `ThinTapeMemoEntry`.
9. Orchestrator: one mark, one cursor reset, one unconsumed-tape
   debug-assert, one truncate (was 2/2/2/2).
10. NOT touched: protocol graph, scan signatures, twin dispatch, error
    carrier (`-0202`), recursion guard, semantic runtime, grammar,
    public API (`DerivEvent` stays; additions are additive).

## 6. The lane-ownership / no-double-credit ledger

**THIS fix owns (and its A/B measures):**
- the event element width 16 → 8 B (`-0191`'s strict 0.530 ns second-word
  rows + its copy-linear view — the same byte population, counted once);
- the boundary lane's Vec/cursor metadata (`-0196`'s gross 1.453 ns);
- ONE of the two mark/truncate/copy/cursor lanes at every site (census §3:
  ~2,160 marks, ~2,170 truncates, 401 copies, 1 cursor word + its
  maintenance);
- the memo's SECOND `from_slice` + SECOND `extend_from_slice` lane and the
  entry-width reduction (a PART of the 30.7 ns memo region — the copy-lane
  half, delivered by construction);
- the merged single allocation (one lane's allocator traffic; the surviving
  allocation grows — the `-0196` honest bound).

**NOT this fix (later units re-price against the landed representation, so
the same lane is never credited twice):**
- G1-C's node width / arena stride / `RefCell` borrow / node memcpy (the
  26.7 ns region MINUS the boundary-push bookkeeping absorbed above);
- the memo's FIRST copy lane (a future full elision — e.g. tape-range
  references instead of owned segments — starts from THIS representation);
- boundary-identity narrowing (`-0195` REFUSED — tag-0 keeps the full
  pointer);
- public telemetry (contract-blocked, 0.227 ns);
- thin-lookup / input-view / position-forwarding gross lanes (`-0197`:
  re-price after this lands).

**Replacement costs owed (debited by the A/B, never pre-netted):** the tag
encode (shl+or) at every event push/patch; the tag test+shift decode at
every build event read; the tag-0 hard check at every boundary read; the
wide-escape branch at dynamic-terminal pushes/decodes (statically absent in
the regex artifact — zero terminal events); the 1-or-2-word cursor step.

## 7. Expected effect (honest, per the `-0197` accounting)

Strict-priced rows total well under noise (0.530 + gross 1.453). The thesis
of this fix is the SAME unpriced-width/lane class that delivered `-0202`'s
−5.56% from a 4.4 ns priced row: the fused spine is memory-traffic-dominant
(`-0172`: 34.7–40.4% of spine instructions), and this unit halves the event
element width, deletes ~2,160 mark reads + ~2,170 truncates + 401 range
copies + one cursor lane from the regex artifact's hot region, halves the
memo copy lanes, and shrinks every thin-memo entry. **No net is banked
before measurement**; the strict same-session ratchet adjudicates, exactly
as `-0198`…`-0202`.

## 8. Acceptance (binding, pre-registered)

The `-0197` ratchet verbatim: canonical 2,189-cell PCRE2 corpus A/B vs base
probe `preserved_probes/regex_perf_probe_cascerr_e88beee8` (same-session
re-read), land only if the unrounded candidate corpus geomean is **strictly
below** the same-session base, verdict flips are **0/2,189**, and candidate
MAX ≤ **483,583 ns**; battery green at the changed vintage (dual-feature lib
suite incl. the ALL-11 interpreter↔generated byte-identical oracle + the new
`TapeWord` layout/round-trip pins, cert ×3 seeds 0/7/42,
`ast_shape_contract_gate`, `duality_hunt_gate`,
`regex_pcre2_compile_oracle_gate`, clippy source-strict, all-11 regen train
with the ebnf fixed point + expected-delta review `overall=0` outside the
`-0203` surfaces + artifact growth ≤ 5%). Otherwise: unconditional product
reversion; only design + rejection evidence commits. Heavy builds under
`scripts/run_with_memory_guard.sh --budget-mb 16384`; ONE heavy job at a
time; `caffeinate` custody for the A/B; alternated base/candidate rounds;
SHA custody banked pre-regen (base probe, pre/post artifact hashes).
