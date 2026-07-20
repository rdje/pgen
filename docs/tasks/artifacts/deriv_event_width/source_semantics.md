# Derivation-event width feasibility and ownership audit

## Current representation and semantic payloads

At the four pinned source files, `DerivEvent` is a five-variant `Copy` enum:
`OrWinner(usize)`, `QuantCount(usize)`, `OptPresent(bool)`, `TokStart(usize)`,
and `TokEnd(usize)`. The standalone probe reproduces that exact declaration and
measures **16-byte size / 8-byte alignment / no drop** on the floor machine.
The event tape is distinct from the 8-byte `deriv_boundary` reference tape.

The payload audit refuses a position, count, or branch cap:

- `TokStart` and `TokEnd` are arbitrary input byte positions;
- `QuantCount` is the completed iteration count;
- `OrWinner` is an emitted grammar branch index; and
- `OptPresent` is one bit.

No one of those facts licenses narrowing an arbitrary `usize` payload.

## Lossless one-word common carrier

A feasible carrier is `Vec<usize>`, interpreted as packed words. Low three bits
hold one of the five variant tags. An ordinary payload no greater than
`usize::MAX >> 3` occupies the remaining bits, so the common event is one word.
Tag 6 is an explicit in-band escape: the escape header records the original
variant tag, and the immediately following word stores the complete, unmodified
`usize` payload. This is lossless on both 32- and 64-bit targets and silently
caps nothing.

Tape marks, cursors, memo segment bounds, truncation marks, and tournament-copy
ranges become **word** indices rather than logical-event indices. That is
compatible with every current lifecycle operation: clear, len/mark, truncate,
slice, extend, `copy_within`, and sequential build replay. The current events
contain no absolute tape indices, so moving a complete word range stays safe.
`deriv_next_event` consumes one ordinary word or an escape header plus its full
payload word and reconstructs the source enum for the existing build matches.

The placeholder writers need an explicit wide-value case. `OrWinner` and
`QuantCount` start as one-word zero placeholders; if the final payload needs an
escape, patching inserts the payload word immediately after the header. That
shifts the placeholder's trailing segment but not an outer mark before it.
`OptPresent` is always ordinary. Start/end events know their payload at append
time. This is implementation work and a replacement debit, not free work.

The thin memo would hold/copy packed words as its event segment. Those segment
copies are deliberately excluded from this leaf because they belong to the
separate thin-memo copy mechanism. The input-scaled constructor hint remains a
capacity hint, now in words; an underestimated escape-heavy tape grows normally
and does not affect correctness.

## Exact machine ownership

The full-SHA-pinned floor probe proves the event Vec as parser offsets
`0x278/0x280/0x288`, separate from boundary offsets
`0x290/0x298/0x2a0`. Machine dataflow finds:

- 22 exact match-side second-word stores (`str x*, [event_ptr, #0x8]`);
- 134 exact build-side second payload loads paired with a tag load and a
  16-byte event-tape index; the address-list digest is pinned;
- 16 exact event `RawVec::grow_one` callers qualified by `parser + 0x278`;
- 23 outlined event `copy_within<DerivEvent>` callers and 15 inlined direct
  event memmoves; and
- constructor event allocation at `parse_once_timed +1628`, qualified by the
  `capacity << 4` byte calculation, with the boundary allocation at +1676 and
  `capacity << 3` as the control.

The direct instruction sets are mechanically disjoint from all accepted
three-band mechanisms and held expansions `-0185` through `-0189`. Caller sets
are disjoint from `-0190`. Boundary traffic, G1-C, thin-memo segment copies,
post-stopwatch teardown, and all other fields receive zero credit.

## Why the three prices differ

The strict floor admits only the second store/load operations that a packed
word can replace: **0.530179151 ns**. All 16 event growth callers have zero
samples on all three profiles, so growth contributes zero.

Halving copied bytes does not halve a small libc copy's fixed overhead.
Nevertheless, assigning half of every exact copy child gives a deliberately
optimistic **2.412143705 ns** copy ceiling. Likewise, the event allocation still
exists at half the byte request; assigning its *entire* exact allocator child as
savings is an impossible-but-useful absolute ceiling of **2.496999283 ns**.

Thus event width contributes 0.530 ns strictly, 2.942 ns under linear copy
scaling, and at most 5.439 ns if the still-required allocation were magically
free. These views do not subtract tag masking, payload shifting, escape
branching, or rare wide-placeholder insertion. Even the impossible ceiling
leaves the accumulated batch's 30%-capture margin only 3.057 ns above the
28.8 ns same-binary noise floor. The implementation remains HOLD.
