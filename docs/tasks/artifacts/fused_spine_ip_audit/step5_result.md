# Fused-spine IP audit — the banked sampler compressed away the required evidence

`PGEN-RGX-0078-0179`, leaf `RGX-0078.5.j.4`, session #170,
2026-07-20. Read-only over the preserved C1 probe and its three custody-pinned
band profiles. No parser, emitter, generated artifact, probe, or measurement
changed.

## Result

The banked macOS `sample` reports cannot answer the dynamic instruction-address
question posed by `-0172`.

Their flat profiles correctly retain self totals, and `filtercalltree
-invertCallTree -pruneCount 1` reproduces those totals exactly for the two
largest fused regions:

| band | `piece + atom` self | one-PC self | multi-PC collapsed | address coverage |
|---|---:|---:|---:|---:|
| below 1 us | 1,740 | 17 | 1,723 | **0.9770%** |
| 1–2.5 us | 1,659 | 28 | 1,631 | **1.6878%** |
| 2.5–20 us | 1,673 | 91 | 1,582 | **5.4393%** |

But a normal `sample` call-tree node may contain an aggregate count followed by
multiple PCs and an ellipsis, for example `+ 1152,1356,...`. It does not retain
the count belonging to each listed PC. Inverting the tree exposes every top
frame but cannot reconstruct information already collapsed in the report.

Log-weighting the three bands, `piece + atom` account for a conservative
**14.6044%** of full-corpus dynamic samples. Only **0.3598% of the corpus**, or
**2.4637% of that target population**, still has a unique PC. The unresolved
target mass is **14.2445%**. Assigning the unique rows alone to a lever would
produce an invalid 4.546 ns target; distributing the other 97.5% among their
listed PCs would be invented data.

## What the small exact residue says—and does not say

The 136 uniquely addressed samples map to 111 memory, 7 control, and 18 other
instructions when pooled across bands. That agrees directionally with `-0172`'s
static classification, but it is a severely selected 2.46% residue. It is also
mechanistically mixed:

- function epilogues restore registers;
- input/grammar tables are read;
- parser fields and stack slots carry live state;
- a few compares and address calculations remain.

No statistically or semantically valid extrapolation follows. In particular,
the exact residue does not isolate speculation save/restore, node/tape traffic,
or another removable carrier, and it cannot be used to subtract the already
priced G1-C and memo-copy overlap.

**Decision: INSTRUMENT-INCOMPLETE.** No new mechanism and no nanoseconds are
admitted. The residual BATCH-1 + G3 target therefore remains **84.385 ns** and
the implementation chain remains **HOLD**.

## Custody and reproduction

`audit_sample_ips.py` asserts:

- the preserved probe's full SHA-256 and exact target-symbol starts;
- all three profiles' full SHA-256 values, main-thread totals, and exact flat
  self totals for `cascade_match_piece` and `cascade_match_atom{closure}`;
- each retained unique PC after ASLR normalization against the pinned binary;
- the exact `otool` instruction at every retained PC.

From the repository root on macOS:

```sh
python3 docs/tasks/artifacts/fused_spine_ip_audit/audit_sample_ips.py
```

The committed `audit_sample_ips.txt` is byte-for-byte output from that command.

## Next measurement leaf

`PGEN-RGX-0078-0180` will validate a raw/timeline sampler before another
full-band capture is spent. The local `spindump` surface is the first candidate:
it supports target-only microsecond intervals, timeline order, leaf-frame
timestamps, retained binary output, and re-rendering a raw capture with `-i`.
Those advertised capabilities are not yet evidence that it preserves one PC
per sample, so the next leaf must test that property rather than assume it.

Pre-registered validation:

1. Use the unchanged preserved probe and verify its full SHA and live executable
   identity before capture.
2. Run one short target-only timeline capture; retain the raw artifact and a
   deterministic text/JSON rendering if the tool permits it.
3. Require at least **95%** of `piece + atom` self samples to expose one
   ASLR-normalizable PC. If the format still groups PCs without counts, refuse
   it and test an in-process signal/Mach-PC sampler in a separately owned leaf.
4. Only after the instrument passes may a later leaf capture all three geomean
   bands and price named mechanisms with G1-C/memo overlap subtracted.

This sequencing prevents a second long profile from producing another report
that is structurally incapable of answering the question.
