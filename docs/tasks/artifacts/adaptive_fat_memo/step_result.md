# PGEN-RGX-0078-0214 — the adaptive fat-memo pre-size unit: REFUSED + REVERTED

Session #185, 2026-07-21. The `-0213`-selected ONE fix, executed exactly per
the banked design prereg (design_prereg.md, incl. its execution-time custody
addendum) and REFUSED by the `-0197` ratchet.

## What was executed (all banked in this dir)

- Design prereg BEFORE code; the MUST-SURFACE construct-LESS fork obligation
  satisfied by the durable `-0213` record.
- The one-line emitter change (`ast_based_generator.rs:1535`: fixed 256 →
  `(input.len() + 1).min(256)`) + the honest comment rewrite.
- All-11 regen train green (tool rebuilt first; ebnf FIXED POINT
  byte-identical; 8 focus targets; pre/post SHA banks). Expected-delta
  review: **2 changed lines per artifact × 11, nothing else**
  (`artifact_delta_review.txt`).
- Candidate probe `96b45c18…` (release fat-LTO + mimalloc), REFUSE-gated
  custody checks in `run_ab.sh`.
- The `-0212` hardened interleaved A/B — the campaign's CLEANEST custody:
  base sweeps +0.05%/−0.44% vs the 1,004.4 floor, twin +0.49%, no foreign
  load, flips 0/2,189, MAX PASS.

## The refusal

Pooled corpus geomean **993.55 → 994.82 ns = +0.1278%** — no strict
decrease; NO band improves (1.0009/1.0009/1.0033/1.0056); bench −1.01% =
the bench-only reversion case. The −1.0…−5.0% prediction band is REFUTED.

## ⭐ The design law learned (recorded for every future lever)

**An allocation-size/count census is NOT a price under mimalloc.** The
machine-pinned 160,264-byte alloc+free every parse — the purest-waste,
largest, cheapest-to-elide construction cost identified on this floor —
priced at ZERO when elided: mimalloc serves a repeated same-size pair from
its thread-local segment cache at near-zero cost, so the table was already
effectively pooled by the allocator, and the only forced content work (the
520-B hashbrown ctrl memset) is trivial. This is the third face of the same
`-0209`/`-0210` lesson: at this floor, mimalloc's paths are so cheap that
neither POOLING allocations (refused twice) nor ELIDING them (refused now)
pays — only removing real CONTENT work (copies, init loops, drop iteration,
per-entry compute) can move the geomean.

**Consequence — the construction-elision (construct-LESS) family is CLOSED
measured-refuted at this floor**: its largest, purest member priced at
zero; the smaller banked members (the `grammar_profile` String alloc, the
275-slot counter `Arc`) are strictly cheaper allocator work and cannot do
better. The teardown/construction population stays priced but is now
proven un-ownable by BOTH pooling and elision; its real content
(drop-iteration of used state, construction init of used state) is owned
by the state's USERS, not by its allocation pattern.

## Revert (executed + verified)

`refused_change.diff` banked; `rust/src` git-clean; ALL 11 artifacts
byte-restored from the SHA-verified pre-regen copies (shasum diff clean);
dual-feature `ast_pipeline` + debug `parseability_probe` + release probe
rebuilt at the reverted vintage; cert ×3 seeds 0/7/42 green post-revert.
Floor of record UNCHANGED (corpus 1,004.4 ns / bench ≈1,579.7 ns);
`carrier48_8d392176` remains the next session's A/B base.

## The honest post-`-0214` selection landscape (for the next session)

On the fresh `-0213` prices, with construction-elision and recycling both
closed and every in-target lane sub-noise: the remaining ≥noise masses are
spine_other 186.6 (mechanism-less ABI/frame ceremony; arena-ref ABI and
compiler tuning refused), allocator 122.6 (now proven fast-path ceremony —
un-ownable by allocation-pattern changes), external 113.4 (libsystem/
kernel + probe timing shell), build_value 72.7 + arena_alloc 69.4 +
semantic_runtime 49.4 (content work of USED state — the only populations
not yet decomposed on the slimmed-carrier floor), and the `-0170`
sub-noise batch (≈22.8 ns ≈ 1.0× noise). The margined −5.4% gap has NO
priced ≥noise designed mechanism left on the current evidence; the next
honest step is either a build_value/arena content-work decomposition
(research-class) or surfacing the exhaustion question to the director.
