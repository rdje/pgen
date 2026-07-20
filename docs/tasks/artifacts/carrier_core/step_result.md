# `PGEN-RGX-0078-0203` — result: **LANDED** (the sixth fix through the `-0197` strict-geomean ratchet; the carrier core closes the held program's ordered leaves)

## What landed

The pre-registered joint replacement/overlap contract (`design_prereg.md`) +
exactly one fix — **the unified packed derivation tape**:

- **LIB** (`mod.rs`): additive `TapeWord<'input>` — the 8-byte tagged address
  word (`Copy`, `needs_drop=false`, layout-pinned by 5 new tests incl. the
  `usize::MAX` wide-escape round-trip and the event-word-can-never-become-a-
  reference panic pin). Tag 0 = the aligned `&'input ParseNode` boundary
  pointer unchanged (provenance intact; the carrier's SOLE dereference is the
  hard-tag-checked engine accessor — emitted code carries no `unsafe`); tags
  1–6 = the `DerivEvent` variants (payload `<<3|tag`, `OptPresent` two
  payload-free tags); tag 7 = the wide escape (header + one raw `usize`
  word) — lossless, no input/count cap. Conditional
  `unsafe impl Send/Sync where &ParseNode: Send/Sync` preserves the parser's
  auto-trait surface bit for bit. `ThinTapeMemoEntry` (ONE inline-6-word
  segment) replaces `ThinDerivSegMemoEntry`, which is RETIRED in-slice after
  the all-11 fixed point proved no artifact references it (the C3 additive
  transient-migration precedent); its full taint-class doctrine moved onto
  the successor. `DerivEvent` stays as the decode-result vocabulary.
- **EMITTER** (`ast_based_generator.rs` + `cascade.rs`): the two-lane
  `deriv_events`/`deriv_boundary` + two cursors → ONE
  `deriv_tape: Vec<TapeWord>` + one cursor. Every speculation / lookahead /
  zero-length-iteration / tournament-candidate mark+truncate pair → ONE;
  tournament compaction → ONE `copy_within` + truncate (the `-0196` §2
  pairing proof); placeholder push/patch via `narrow_event` (patches proven
  statically narrow: branch index < branch count, count ≤ SAFETY_LIMIT);
  dynamic terminal events via `push_event` (1-or-2-word append); boundary
  call-outs push tag-0 words; thin-memo success insert = ONE
  `SmallVec::from_slice`, hit splice = ONE `extend_from_slice`; orchestrator
  = one mark/cursor-reset/assert/truncate. Build-pass logic untouched by
  construction (`deriv_next_event`/`deriv_next_boundary` signatures
  preserved; only their bodies decode the unified lane). Emitter text-pins
  re-anchored (4 sites) + doc lockstep.

Regex-artifact census delta (custody `c3d91959` → `c5e1b425`): the fix
deletes ~2,160 mark reads, ~2,170 truncates, and 401 of 802 range copies
from the fused region, halves the 56 memo `from_slice` copies, and shrinks
EVERY artifact (−0.6%…−2.1%; growth bar 5%).

## Adjudication (binding `-0197` ratchet — `adjudication.txt`)

- Corpus (2,189 cells): unrounded geomean **1094.942204364074 →
  1051.8818294596394 ns = −3.9327%** — strict same-session decrease ⇒
  **LAND**; ≈1.7× OUTSIDE the ≈2.3% noise span (the second
  individually-measurable win, after `-0202`).
- Verdict flips **0/2,189**; candidate MAX **405,084 ≤ 483,583 ns** settled
  bound (same worst cell `line_725`, itself −8.3%).
- Floorval custody: base bench geomean 1671.9 vs banked 1708.8 = **−2.16%**
  OK (session running cool; the base corpus re-read 1094.94 vs the banked
  1100.91 floor = −0.54% drift, cool side).
- Bench steering (secondary): geomean **−1.02%** with a mixed per-pattern
  spread (`email_basic` −7.97%, `anchor_complex` −6.50%, `url_simple`
  −4.20%, `digit_sequence` −3.18% vs `character_class` +6.68%,
  `capture_groups` +4.41% — single-pattern jitter both ways); the corpus
  geomean is the adjudicating instrument.
- Honest magnitude note: the strict-priced rows were **0.530 ns** (event
  second words) + a 1.45 ns gross boundary-metadata view; the delivered
  −43.1 ns is dominated by the deliberately UNPRICED width/lane class —
  halving the event element to 8 B, deleting the second mark/truncate/
  copy/cursor lane, and halving the memo copy lanes — the same
  memory-traffic-dominant-spine pattern `-0202` exposed (priced 4.4 ns,
  delivered −64.8 ns).

## Floor re-baseline

Floor of record moves to the candidate's same-session reading: corpus
geomean **1,051.9 ns** (unrounded 1051.8818294596394), bench ≈**1,646.1
ns**, settled MAX bound unchanged (483,583 ns). The margined <1 µs call-off
bar (unrounded geomean ≤ **950.0 ns**, director #176) now needs **−9.7%**.

## Custody

- Base = `preserved_probes/regex_perf_probe_cascerr_e88beee8` (SHA banked
  pre-regen, cmp-asserted in-run by `run_ab.sh`). Candidate probe preserved
  `preserved_probes/regex_perf_probe_unitape_6812d8e2` (cmp-verified).
- Regex artifact: pre `3814aea1…`→`c3d91959…` (the `-0202` vintage) → post
  **`c5e1b425…`**; ebnf **`e2ede238…`** (`artifacts_pre/post_regen.sha256`);
  all-11 regen train green incl. the ebnf fixed point; expected-delta review
  **`overall=0` over all 11** (`artifact_delta_review.txt`, with the
  strengthened move-cancelling symmetric-difference form); every artifact
  SHRANK (−0.6%…−2.1%).
- Battery (all green, `battery_summary.txt`): dual-feature lib **1006/0
  (29 ignored)** incl. the ALL-11 interpreter↔generated oracle + the 5 new
  `TapeWord` pins; cert ×3 seeds byte-exact `268/9/259/0 fully_certified`
  (spf=0); `ast_shape_contract_gate`, `duality_hunt_gate`,
  `regex_pcre2_compile_oracle_gate`, clippy source-strict.
- Every heavy step memory-guarded (16384 MB, floor 10%), serialized,
  `caffeinate -i`; alternated base/candidate rounds; peaks 12,225 /
  11,554 MB.

## Next

Per the one-fix-per-fresh-session directive this session stops after the
clean commit. The `-0197` ordered leaves are now EXHAUSTED (`-0198`…`-0203`
all adjudicated, all six LANDED; cumulative across the held program's
floor-of-record readings: **1,263.4 → 1,051.9 ns ≈ −16.7%** corpus geomean,
cross-session drift included). NEXT (brand-new session) =
a read-only RE-PRICING leaf: reprice the surviving gross/proxy lanes (G1-C
node width/arena residue, the memo's remaining copy lane, thin-lookup,
input-view, position-forwarding) against the LANDED unified-tape
representation (the `-0197` rule: gross/proxy lanes must be repriced after
landed representation changes) before selecting any further fix — the
margined bar needs −9.7%.
