# ENGINE-UNIVERSAL-SERVICES.67(b) — prediction #2, recorded BEFORE the owned-heap instrument is run

Written after `NodeArena::record_owned_heap` compiled (`cargo check --lib` clean) and
**before any owned-heap number was read**, so the comparison is a prediction and not a
description. This is the second pre-registration on this leaf; the first
(`PREDICTION.md`) was 2 legs falsified / 1 undecided, and the undecided leg is exactly
what this measurement closes.

## What is being measured

The heap a `ParseNode` owns OUTSIDE the four arenas — the omission whose absence forced
`-0122` to withdraw *"a substantial majority is provably NOT the AST"*:

- `ParseContent::Sequence(Vec<&ParseNode>)` and `Quantified(Vec<&ParseNode>, _)` — one
  separate allocation each, `capacity()` pointer slots of 8 B.
- `ParseContent::TransformedTerminal(String)` — the text behind the header.
- the `rendered_strings` arena's `String` payloads (declared omission #2 of the first cut).

## Predictions, on `iverilog/ivtest/ivltests/br_gh330.v` (36 673 B)

Published anchors this must be read against, all from `-0121` and unchanged:
1 409 813 arena nodes · 64.54 MB node headers · 79.59 MB arena item total · 314 408 960 B
peak RSS · 27.9 % of the floor-subtracted peak.

1. **The owned heap is LARGE relative to the headers — 25 %–75 % of the 79.59 MB arena
   item total (≈ 20–60 MB).** Reasoning: Rust's `RawVec` minimum non-zero capacity for an
   8-byte element is 4 slots = 32 B, which is two-thirds of a 48-byte `ParseNode` header.
   A grammar whose nodes are mostly short sequences therefore pays nearly as much for the
   child vectors as for the nodes that own them.
2. **Child `Vec`s dominate that heap**: `TransformedTerminal` payloads + rendered-string
   payloads together are **< 5 %** of the owned total. Reasoning: transforms are rare in
   the SV grammar relative to structural sequencing.
3. **`Sequence` + `Quantified` are 35 %–65 % of all nodes** — i.e. 490 000–920 000 child
   vectors on this file.
4. **The AST — headers + owned heap — is still a MINORITY of the floor-subtracted peak,
   landing at 35 %–50 %** (up from the 27.9 % headers-only figure).

## Falsifier — and it is the interesting outcome

If (4) comes back **above 50 %**, the withdrawn conclusion is not merely unproven but
**INVERTED**: the AST would be the majority leg, `-0121`'s original prediction #1 would be
retro-vindicated on its own terms, and the fix shape changes — slimming the node/child
representation becomes the lever on the ingestion bound instead of the allocator.

If (1) comes back **under 10 %**, the omission that forced the withdrawal was immaterial
to the number, and the withdrawal — while right about the METHOD — cost a correct
conclusion. Recording that possibility here so it cannot be quietly dropped.
