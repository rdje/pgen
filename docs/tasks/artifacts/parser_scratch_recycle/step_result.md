# `PGEN-RGX-0078-0209` result — the parser-scratch TLS-lease recycling unit (REFUSED + REVERTED)

Leaf `RGX-0078.5.j.4`, session #181, 2026-07-21. The `-0208`-selected ONE fix
under the 2026-07-21 director GO, executed per `design_prereg.md` (registered
before code) and adjudicated by the binding `-0197` ratchet.

## Verdict — REVERT (the ratchet's unconditional product reversion)

| gate | result |
|---|---|
| corpus geomean (unrounded, same-session) | **1017.0150223841464 → 1156.7478806476497 ns = +13.7395%** — strict decrease **FAIL** |
| verdict flips | 0 / 2,189 |
| candidate corpus MAX | 383,125 ns ≤ 425,000 (PASS; worst cell `line_725`, itself −0.9%) |
| floorval custody (base bench vs banked 1,619.1) | +0.24% — OK |
| bench steering (alternated 5×2000) | 1557.0 → 1588.2 ns (+2.00%) |

The floor of record is UNCHANGED: **corpus geomean 1,037.8 ns / bench
≈1,619.1 ns (≈306×)**; base probe `fxstore_a4067793` remains the next
session's immediate-parent A/B base. (The base re-read 1,017.0 sat −2.0%
below the banked floor — inside the cross-session span, cool side.)

## Root cause of the regression (tool-pinned, not narrative)

The per-band decomposition (`band_analysis.txt`, computed from the banked
paired sweeps) shows a textbook FLAT-ADD signature:

| band | cells | delta |
|---|---|---|
| sub-1 µs | 1,098 | **+20.18%** |
| 1–2.5 µs | 715 | +9.36% |
| 2.5–20 µs | 365 | +4.52% |
| ≥20 µs | 11 | **−1.45%** (improves) |

Worst cells are 250–291 ns parses DOUBLING (+~290 ns flat). Mechanism:
**hashbrown `clear()` is O(capacity)** — it memsets the whole ctrl array.
A recycled `memo_fail` permanently carries the LARGEST capacity any prior
parse grew (the corpus MAX cell reserves min((len+1)·6, 32768) ≈ 21k entries
⇒ a ~32k-bucket table), so after one large parse EVERY subsequent small parse
pays a ~32 KB ctrl memset in `reset_recycled()` — a fixed cost that dwarfs
the recycled savings exactly where the geomean's log-weight lives (43.6% in
the sub-1 µs band). The ≥20 µs improvement (−1.45%) confirms the warm-
allocation mechanism is real — it wins only once the clear amortizes.
Secondary contributors, same direction: the state's double reset per parse
(`reset_recycled` at take + `reset_for_new_parse` in `prepare_parse_state`)
and four TLS take/return round-trips.

⛔ The design lesson (now twice-taught, and this time by a REVERT): the
`-0205` `ThinMemoScratch` avoided precisely this trap with a GENERATION
STAMP — invalidation is one counter bump, never an O(capacity) clear. The
prereg cited that lesson for pricing (§1) yet treated warm capacity as a
pure win (§5, "capacities may EXCEED today's") — the reset's O(capacity)
cost was the unpriced replacement cost. On a band-weighted geomean, a
recycling design is only admissible if its per-parse reset is O(live
entries), not O(high-water capacity).

## What was executed (all banked)

- Design prereg BEFORE code (`design_prereg.md`, incl. the post-regen
  addendum recording the bootstrap-emitter misprediction: the annotation
  pair is emitted by `ast_based_generator` in bootstrap mode, so the lease
  surfaces reached ALL 11 artifacts — uniform, 48 lines each, verified by
  `review_artifact_delta.sh` = `artifact_delta_review.txt`, growth ≤+0.324%).
- Lib: `ParseScratchRecyclable` (`'static`-supertrait-enforced
  lifetime-freeness) + `ParseScratchLease` (active-flag `Default` leftover
  keeping the emitted `mem::take` transaction doctrine cost-identical) +
  `SemanticRuntimeState::recycle_reset`/`recycle_shell` (the fact-leak
  hazard closed: `reset_for_new_parse` preserves facts by contract, so the
  pooled reset clears them first) + `RecursionGuard::set_max_depth`/Default
  + two test pins (`recycle_reset_matches_new`,
  `parse_scratch_lease_recycles_and_default_leftover_is_inert`).
- Emitter: the four field decls + `new()` take/reserve/set_max_depth blocks;
  self-check pins re-anchored (wrap-tolerant — prettyplease breaks the lease
  generic across lines; the first battery's only failures were the two new
  single-line pins, fixed test-side only).
- All-11 regen train green incl. the ebnf fixed point; full battery green at
  the candidate vintage: dual-feature lib **1008/0 (29 ignored)** incl. the
  ALL-11 byte-identical oracle + both new pins; cert ×3 seeds 0/7/42
  byte-exact `268/9/259/0 fully_certified` (spf=0); shape, duality, PCRE2
  compile oracle; clippy source-strict. Correctness was NEVER the problem —
  the fix was refused purely on the speed ratchet.
- Custody: base probe SHA asserted in-run (`custody.txt`); candidate probe
  SHA `94fdaa14…` recorded (NOT preserved — refused fixes bank numbers, not
  binaries; the `-0134`/`-0156` precedent); candidate regex artifact
  `f739ce88…` vs base `438bb931…`.
- REVERT executed: `rust/src` restored via git (tree clean); ALL 11
  artifacts restored BYTE-EXACT from the pre-regen copies (SHA-verified
  against `artifacts_pre_regen.sha256`); dual-feature tool + release probe
  rebuilt at the reverted vintage; cert ×3 re-verified green post-revert.
  ⭐ The rebuilt release probe is **BIT-IDENTICAL to the preserved base
  probe** (`a406779314b9…` — the fat-LTO build reproduced exactly), so the
  on-disk `target/release/regex_perf_probe` == the floor probe of record
  byte-for-byte: the strongest possible restore proof. The refused change's
  full source diff is banked as `refused_change.diff` (re-applied and
  captured after the coherence rebuild, then reverted again — verified
  clean both times).

## Banked follow-up (for a future re-pricing to adjudicate — NOT this session)

A **stamp-validated recycling** redesign: per-entry generation stamps (the
`-0205` trick applied to the fail-memo class) make cross-parse invalidation
O(1) with no capacity-proportional reset; the state side would need
take-side reset folded into the ONE existing `prepare_parse_state` reset
(eliminating the double reset) with the facts-clear preserved. The ≥20 µs
band's measured −1.45% shows the warm-allocation win is real when the flat
add is removed. Research-class until designed; the teardown/construction
population (51.13 ns) stays priced but UN-OWNED.

## Reproduction

```sh
R0209_SCRATCH=<scratch> bash docs/tasks/artifacts/parser_scratch_recycle/regen_train.sh
R0209_SCRATCH=<scratch> bash docs/tasks/artifacts/parser_scratch_recycle/battery.sh
R0209_SCRATCH=<scratch> bash docs/tasks/artifacts/parser_scratch_recycle/run_ab.sh
python3 docs/tasks/artifacts/parser_scratch_recycle/analyze_ab.py
```

(`analyze_ab.py` reproduces `adjudication.txt` byte-for-byte from the banked
round/corpus outputs; `band_analysis.txt` reproduces from the banked corpus
JSONLs.)
