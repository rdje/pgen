# PGEN-RGX-0078-0196 result

The two derivation tapes have one lossless total order. Match-side appends and
build-side consumes occur at the same AST sites; placeholders precede nested
segments; speculation and lookahead truncate whole suffixes; tournament
compaction moves only relative segments; thin memo and sub-root orchestrators
already use contiguous stack-disciplined segments. All **22** outlined and
**15** direct boundary copies pair one-for-one with event copies; only one
outlined event copy is unpaired.

A normal safe enum does not realize the saving: both
`enum { Event(DerivEvent), Boundary(&ParseNode) }` and
`enum { Event(usize), Boundary(&ParseNode) }` measure **16 bytes**, widening
boundary entries and, in the second case, undoing the common event-width gain.
An untagged union is rejected because emitter/build drift can select an invalid
reference and cause undefined behavior.

A signoff-capable **8-byte** design does exist. Tag zero stores an aligned arena
pointer unchanged; six nonzero low-bit tags represent the event variants
(`OptPresent` uses two tags); tag seven is a header followed by one raw `usize`
payload word. This escape preserves values above `usize::MAX >> 3`, including
`usize::MAX`, so the design adds no input/count/index cap. Event words have no
provenance and are never dereferenced; the boundary accessor checks tag zero
before its sole encapsulated unsafe dereference. The pinned rustc probe
round-trips the boundary and narrow/wide/max payload cases and measures the
carrier at 8 bytes.

The exact unowned direct metadata census is **291 event PCs + 215 boundary
PCs**. Dynamic counts are event/boundary **104/30**, **74/19**, and **78/40**,
re-summing the prior tape row to **134/93/118**. Removing the boundary Vec and
cursor makes the boundary row structurally addressable, but the new boundary
tag test/mask and variable-width decoder do not exist in the preserved binary.
Its **1.452693317 ns** value is therefore a gross ceiling, not strict credit.

After the independent event-width design, a unified tape copies the same
`8*events + 8*boundaries` bytes. Only one call/setup lane disappears; the
boundary bytes remain. Crediting the whole boundary-copy child is an explicitly
impossible **5.727016718 ns** ceiling. Removing one constructor allocation has
an absolute **1.687336492 ns** ceiling because the surviving allocation grows.
G1-C, event-width, and accepted thin-memo ranges remain excluded.

Verdict: **FEASIBLE-HOLD**. New strict contribution is **0**; accumulated strict
stays **102.138976561 ns**, whose 30% capture leaves **1.841692968 ns** above the
28.8 ns noise floor. The unique direct-metadata gross view leaves
**2.277500963 ns**. Implementation requires a later owned A/B leaf and is best
batched with the already-designed event-width carrier rather than charged as an
isolated lane.

No parser/runtime/emitter/generated artifact, behavior, accepted floor/MAX,
mdBook, contract, reference architecture, or LIVE row changed.
