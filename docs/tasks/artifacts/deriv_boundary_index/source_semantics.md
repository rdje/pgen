# Derivation-boundary identity and replacement audit

## Current semantic contract

The match/build split stores `&'input ParseNode<'input>` in
`deriv_boundary`. Each boundary call-out first constructs a `ParseNode`, then
`NodeArena::alloc` returns the stable reference that is appended to the tape.
The build pass consumes the same reference in append order. Tournament
speculation may truncate or move tape segments; successful cyclic thin-memo
entries copy boundary segments into `SmallVec`s and replay those exact
references later in the same parse.

The pinned layout probe measures the current reference at **8 bytes**, aligned
to 8, with no drop. It is already one native word and carries no tag, spare
payload word, or other in-element redundancy. An equally lossless `usize`
ordinal is also 8 bytes, so it offers no width reduction.

## Why a direct `u32` replacement is not lossless

A `u32` can identify at most 2^32 simultaneously addressable values. Neither
the public parser contract nor `NodeArena` caps input length, node count, arena
capacity, or live boundary population at that limit. The observed corpus and
the current 8,192-element capacity *hint* cannot become correctness bounds:
`Vec` grows beyond its hint, and the arena accepts arbitrary further
allocations.

The arena is not one contiguous allocation. `NodeArena` wraps
`typed_arena::Arena<ParseNode>` 2.0.2, whose stable addresses live in multiple
non-growing chunks. The project wrapper exposes `alloc(node) -> &mut
ParseNode`, not an ordinal-to-node lookup. Therefore neither a 32-bit offset
from one base nor a dense ordinal can reconstruct today's reference without a
new identity structure.

Three complete alternatives were considered:

- `usize` ordinal: lossless, but still 8 bytes; reconstruction adds a lookup.
- `u32` into `Vec<&ParseNode>`: the table retains one full 8-byte pointer for
  each unique boundary node and the boundary tape adds the 4-byte handle. In
  tournament order, discarded earlier winners can also leave interior table
  entries that cannot be removed without invalidating later handles. This
  increases identity storage before charging lookup and bounds work. Counts
  beyond 2^32 still need an escape.
- Indexed chunk arena: replace the node arena with chunk descriptors and
  return a dense ordinal at allocation. A common `u32` tape word plus an
  explicit wide escape can preserve arbitrary counts, but replay must decode
  the escape, find the chunk (direct table, prefix search, or packed
  chunk/slot), bounds-check it, and load the final node address. This is a new
  arena architecture, not a carrier-only width change.

Raw pointer truncation, a presumed 4-GiB arena window, and a corpus-derived
maximum are refused. They either lose pointer identity/provenance or silently
turn a measurement population into a correctness cap.

## Complete preserved-machine census and ownership

The custody-pinned binary fixes the parser's boundary Vec at capacity/data/len
offsets `0x290/0x298/0x2a0`, its build cursor at `0x4f0`, and the element scale
at `lsl #3`:

- seven exact pointer stores and seven `RawVec::grow_one` calls occur in the
  already-owned G1-C boundary-push ranges;
- 138 exact build-side pointer replay loads are rediscovered across all regex
  `cascade_build_*` symbols; their address digest is pinned;
- the tournament carrier has 22 outlined `copy_within<&ParseNode>` calls and
  15 inlined direct `memmove` calls, distinct from the 16-byte event calls
  owned by `-0191`;
- constructor allocation at `parse_once_timed +1676` is qualified by
  `capacity << 3`; and
- the two cyclic thin-memo success sites contain both boundary-segment
  `SmallVec::from_slice` construction and the corresponding successful insert.
  Their four accepted instruction ranges were already isolated before this
  leaf, so they receive zero new credit.

The seven push/growth sites are likewise excluded as G1-C-owned. Tournament
copies, replay loads, and constructor allocation have zero overlap with all
accepted/held work through `-0194`, including the event-width, input-view,
checkpoint, and position carriers.

## Replacement-aware pricing

Narrowing a current pointer store or replay load changes its width but does
not eliminate the instruction. It also adds handle reconstruction. All direct
push and replay instructions happen to have zero samples in the three raw
captures, but the strict result is zero independently of that sampling fact.
The G1-C growth children also have zero caller-attributed samples.

For a deliberately optimistic upper view, pretending every tournament copy's
runtime scales linearly with bytes assigns half of each exact copy child to a
four-byte carrier: **2.863508358 ns**. Small-copy fixed overhead does not halve,
and no complete lossless candidate gets this credit without added identity and
lookup work, so this is explicitly a `u32` fantasy ceiling.

The constructor boundary allocation child contributes **1.687336492 ns** if
credited in full, even though every candidate still needs boundary storage and
an indexed design may allocate more metadata. This is an impossible absolute
ceiling, not measured saving.

Strict accumulated savings stay **102.138976561 ns**, whose 30% capture leaves
only **1.841692968 ns** above the 28.8 ns same-binary noise floor. Adding the
copy-linear fantasy raises that margin to **2.700745476 ns**; making the still
required constructor allocation magically free raises it to **3.206946424
ns**. Neither view charges the mandatory reconstruction or arena redesign.
Implementation remains HOLD.
