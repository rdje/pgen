# PGEN-RGX-0078-0195 result

Verdict: **HOLD; no implementation licensed.**

`deriv_boundary` already stores the minimum direct lossless identity: one
8-byte stable arena reference. A `usize` handle is the same width. A direct
`u32` handle imposes an unlicensed 2^32 population cap; a `u32` side table
retains the original full pointer and adds storage; and a genuinely lossless
common-width design requires a new indexed chunk arena, wide escape, and a
dependent reconstruction lookup on every build read.

The complete preserved census finds seven G1-C-owned pointer pushes/growth
calls, 138 build replay loads, 22 outlined and 15 direct tournament copies,
the constructor allocation, and two already-owned thin-memo success sites.
Prior overlap is excluded. Replacing a pointer load/store with a handle does
not remove the instruction, so strict saving is **0 ns**.

Crediting half of every exact tournament-copy child gives an explicitly
unrealistic **2.863508358 ns** byte-linear ceiling. Crediting the *entire*
still-required constructor allocation adds an impossible **1.687336492 ns**.
The accumulated strict bundle therefore stays **102.138976561 ns** with only
**1.841692968 ns** practical margin at 30% capture. Even the copy and absolute
fantasies leave only **2.700745476 ns** and **3.206946424 ns** before mandatory
lookup, bounds/escape, chunk metadata, and arena-rewrite costs. No arena,
carrier, emitter, generated artifact, or behavior changed.
