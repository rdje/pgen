# `PGEN-RGX-0078-0210` result — the STAMP-VALIDATED parser-scratch recycling unit (REFUSED + REVERTED; the recycling lane CLOSED measured-exhausted)

Leaf `RGX-0078.5.j.4`, session #182, 2026-07-21. The banked `-0209` follow-up
executed per `design_prereg.md` (registered before code) and adjudicated by
the binding `-0197` ratchet.

## Verdict — REVERT (the ratchet's unconditional product reversion)

| gate | result |
|---|---|
| corpus geomean (unrounded, same-session) | **1024.1479897839465 → 1031.0056940861866 ns = +0.6696%** — strict decrease **FAIL** |
| verdict flips | 0 / 2,189 |
| candidate corpus MAX | 379,416 ns ≤ 425,000 (PASS; worst cell `line_725`, itself −1.9%) |
| floorval custody (base bench vs banked 1,619.1) | −0.06% — OK |
| bench steering (alternated 5×2000) | 1601.9 → 1589.4 ns (**−0.78%**) — the contract's "bench-only improvement ⇒ reversion" case, realized |

The floor of record is UNCHANGED: **corpus geomean 1,037.8 ns / bench
≈1,619.1 ns (≈306×)**; base probe `fxstore_a4067793` remains the next
session's immediate-parent A/B base. (The base re-read 1024.1 sat −1.3%
below the banked floor — inside the cross-session span, cool side.)

## What the measurement says (tool-pinned, banked in `band_analysis.txt`)

The stamp redesign did EXACTLY what it was designed to do — and the lane
still lost:

| band | cells | `-0210` delta | `-0209` delta (eager clear) |
|---|---|---|---|
| sub-1 µs | 1,090 | **+1.87%** | +20.18% |
| 1–2.5 µs | 721 | **−0.47%** | +9.36% |
| 2.5–20 µs | 368 | **−0.56%** | +4.52% |
| ≥20 µs | 10 | **−1.26%** | −1.45% |

- The O(capacity) flat add is GONE: worst cells 1.40× (+83 ns) vs `-0209`'s
  2.16× (+290 ns); every band ≥1 µs now IMPROVES — the warm-allocation win
  is real and the generation-stamp mechanism works as specified.
- The residual regression is the FIXED per-parse recycling ceremony itself:
  four TLS take/return round-trips, the take-side `recycle_reset` (the
  second `reset_for_new_parse`, kept sound by the preload-facts contract —
  prereg §3), the stamp compare, and the persistent-table locality tax.
  On sub-1 µs parses (1,090 cells ≈ half the geomean's log-weight) that
  ceremony exceeds the construct+teardown cost it replaces — mimalloc's
  small-alloc path is simply too fast to beat with pooling ceremony at this
  floor. Net +0.67% ⇒ refused.

## ⛔ Lane closure (the prereg's own criterion, now triggered)

TWO design forms of parser-scratch recycling are refused on measurement:
eager-clear (`-0209`, +13.74%) and stamp-validated (`-0210`, +0.67%). The
recycling/pooling lane over the lifetime-free component set is **CLOSED
measured-exhausted** at this floor. Design law addendum (the second lesson):
pooling amortizes only while its fixed ceremony is ≪ the construction cost
it elides; under mimalloc on a sub-µs floor the component set's
construct+teardown (≈30–55 ns gross) leaves no room. The
teardown/construction population stays PRICED but is un-ownable by pooling —
any future attack on it must come from a DIFFERENT mechanism class
(constructing less, e.g. a lazily-materialized or shrunken state for
annotation-light grammars — a MUST-SURFACE design fork if pursued).

## What was executed (all banked in this dir)

- Design prereg BEFORE code (`design_prereg.md`), incl. the recorded
  DEVIATION refusing the banked "fold into `prepare_parse_state`" idea
  (unsound against the preload-facts contract) — no silent drift.
- Lib: `StampedFailSet` + `StampedTaintedFailMap` (generation-stamp O(1)
  invalidation; pre-registered memory backstops 2^18/2^12; the tainted
  stamp also closes a cross-generation epoch-collision soundness hazard) +
  the `-0209` lease architecture verbatim; 3 new pins + the 2 recycle pins
  green. Emitter: the four lease fields + take/ensure_capacity/set_max_depth
  blocks; self-check pins re-anchored wrap-tolerant.
- All-11 regen train green incl. the ebnf fixed point; expected-delta review
  GREEN: uniform 48-line deltas, all inside the `-0210` surfaces
  (`artifact_delta_review.txt`), manual diff read confirmed.
- Full battery green at the candidate vintage: dual-feature lib **1011/0
  (29 ignored)** incl. the ALL-11 byte-identical oracle; cert ×3 seeds
  0/7/42 byte-exact `268/9/259/0 fully_certified` (spf=0); shape, duality,
  PCRE2 compile oracle; clippy source-strict. Correctness was NEVER the
  problem — the refusal is speed-only.
- Custody: base probe SHA asserted in-run (`custody.txt`); candidate probe
  SHA `1879da6c…` recorded (NOT preserved — refused fixes bank numbers, not
  binaries); candidate regex artifact `0f2041b5…` vs base `438bb931…`.
- REVERT executed: `rust/src` restored via git; ALL 11 artifacts restored
  BYTE-EXACT from the pre-regen copies (SHA-verified against
  `artifacts_pre_regen.sha256`); dual-feature tool + debug probe + release
  probe rebuilt at the reverted vintage; post-revert cert ×3 verified
  byte-exact green. ⭐ Restore identity: the rebuilt release probe differs
  from the preserved base in exactly **87 of 9,368,992 bytes — all within
  the Mach-O `LC_UUID` (offsets 2025–2040, `dwarfdump`-confirmed distinct
  UUIDs) and the dependent ad-hoc code-signature tail (≈8,663,406+)**; the
  code/data sections are byte-identical. (Unlike `-0209`, full-file
  bit-identity did not reproduce: the mid-session `cargo sweep` forced
  dependency rebuilds whose input ordering perturbs the linker's UUID hash
  — build metadata only, no functional delta; the preserved
  `fxstore_a4067793` remains the custody anchor either way.) The full
  refused source diff is banked as `refused_change.diff`.

## Session-operational note (disk)

Mid-unit, on director order, a proactive §8 cleanup ran: 32 GB
`target/debug/incremental` + deps `.bin`/`.log` + `cargo sweep --time 1`
(1.63 GiB) engineer-side, plus the director-fired purge of 60 dead session
scratchpads — disk 27 → ~118 GB free. The A/B custody copies (current
session scratch) were verified intact before and after; the paired sweep ran
entirely after the cleanup, so no measurement spans the disturbance.

## Reproduction

```sh
R0210_SCRATCH=<scratch> bash docs/tasks/artifacts/stamp_recycle/regen_train.sh
R0210_SCRATCH=<scratch> bash docs/tasks/artifacts/stamp_recycle/battery.sh
R0210_SCRATCH=<scratch> bash docs/tasks/artifacts/stamp_recycle/run_ab.sh
python3 docs/tasks/artifacts/stamp_recycle/analyze_ab.py
python3 docs/tasks/artifacts/stamp_recycle/band_analysis.py
```

(`analyze_ab.py` reproduces `adjudication.txt` byte-for-byte from the banked
round/corpus outputs; `band_analysis.py` reproduces `band_analysis.txt` from
the banked corpus JSONLs.)

## Banked follow-ups (for the NEXT session's selection — NOT this one)

With recycling closed, the surviving candidates on the unchanged floor
(the `-0208` prices stay honest — nothing landed):

1. **spine_other decomposition** (203.0 ns, the largest unowned mass, no
   designed mechanism) — a research-class re-profiling slice to split that
   bucket into ownable lanes.
2. **Construct-less design fork** — attack the teardown/construction
   population by constructing LESS (lazily-materialized/shrunken state for
   annotation-light grammars). Extension-point-shaped ⇒ MUST-SURFACE fork
   BEFORE code (director process law #175).
3. **The `-0170` amended sub-noise batching adjudication** over the
   remaining sub-noise lanes (checkpoint 9.031 + position 5.673 + guard
   4.928 + tape 3.327), if a coherent single-mechanism batch can be argued.
