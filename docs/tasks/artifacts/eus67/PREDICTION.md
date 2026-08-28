# ENGINE-UNIVERSAL-SERVICES.67(b) — prediction, recorded BEFORE the instrument is run

Written after the instrument compiled and before any footprint number was read, so the
comparison below is a prediction rather than a description.

On `iverilog/ivtest/ivltests/br_gh330.v` (36 673 B, 299.9 MB peak RSS on the fused graph):

1. The ARENA is the MAJORITY leg — I predict > 50 % of peak RSS, and `ParseNode`
   allocations (48 B each) dominate the arena over the three shaped/string arenas.
2. Reasoning: the memo is the other candidate, and its size is bounded by the entry counts
   `PGEN_REPORT_MEMO_STATS` already reports — 220 697 success + 651 246 cached failures =
   871 943 entries. Even at a generous ~96 B per entry including hash-table overhead that
   is ~80 MB, about a quarter of 299.9 MB. (⚠️ Those counts are PROTOCOL-graph counts and
   the fused graph skips the memo lane for fused rules, so the fused memo is SMALLER —
   which only strengthens the direction of the prediction.)
3. Therefore I expect roughly 4-5 million arena nodes for this 36 KB file, i.e. of order
   100+ ParseNode allocations per input byte.

Falsifier: if the arena reports well under half of peak RSS, the prediction is wrong and
the residual is the finding — the footprint would then live somewhere neither the arena
census nor the memo counts can see.
