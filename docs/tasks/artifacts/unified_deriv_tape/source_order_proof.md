# Unified derivation-tape source-order proof

Scope: `PGEN-RGX-0078-0196`, read-only feasibility/pricing. The source is
SHA-256 pinned by `classify_unified_tape.py`; line references below describe
that pinned source rather than transferring current-vintage addresses into the
preserved C1 probe.

## 1. One total order exists

Both existing vectors are append-only logs during the match walk. A record is
produced at exactly the AST site whose build mirror later consumes it:

- a non-degenerate `Or` appends its `OrWinner` placeholder before entering the
  chosen branch; the build mirror consumes `OrWinner` before entering that
  branch;
- `QuantCount` is appended before its iterations and consumed before rebuilding
  those iterations;
- `OptPresent` is appended before the optional body and consumed before the
  build decides whether to enter that body;
- `TokStart`/`TokEnd` are appended at a terminal and consumed at that terminal;
- a boundary node is appended immediately after the eager call-out returns and
  consumed at the matching non-internal rule-reference atom.

Sequence emission preserves source order, while recursive internal calls emit
and consume their nested records in place. Therefore a preorder merge of the
two append streams is lossless: the build's identical AST walk consumes the
same merged order. No event or boundary record needs an absolute tape index.

## 2. Speculation and compaction remain word-range operations

Every paired speculation mark records the two current vector lengths. In one
carrier this becomes one word index. A failed attempt truncates to that index;
lookahead does the same on both success and failure. The one event-only
truncate is the failed `Or` placeholder, so it also becomes a single truncate.

Tournament code currently compacts the winning event range and then the
winning boundary range. All 22 outlined boundary copies and all 15 direct
boundary memmoves are paired one-for-one with the immediately preceding event
copy (machine gaps `0x30..0x50` and `0x3c..0x44`). A merged winner is already
one contiguous word range, so one `copy_within` and one `truncate` preserve it.
The sole unpaired event copy is valid because a winning segment can contain no
boundary records; a unified word-range copy still handles that case.

## 3. Placeholder patching is compatible with a wide escape

The ordinary one-word format has seven low-bit classes:

- `000`: an unmodified, aligned `ParseNode` pointer;
- six nonzero tags: the four `usize` event variants plus both `OptPresent`
  values;
- `111`: a wide-event header followed by one raw `usize` payload word.

Ordinary payloads up to `usize::MAX >> 3` remain one word. The escape preserves
arbitrary `usize` values rather than imposing a 61-bit semantic cap. `OrWinner`
and `QuantCount` are patched only after tournament compaction or loop completion.
If a patched value ever needs the escape, insertion of its second word shifts
the already-final segment once; no still-live inner mark exists, enclosing
marks precede the insertion, and records contain no absolute indices. Appended
terminal events can choose one or two words immediately.

## 4. Pointer lifetime and safety can be encapsulated

`ParseNode` contains `&str`/`usize`-aligned fields, so its alignment is at least
8 and a live pointer has three zero low bits. Tag zero stores that pointer
unchanged, preserving its provenance. Event words are created with
`ptr::without_provenance` and are never dereferenced. The boundary accessor
checks tag zero before the sole internal unsafe dereference; safe callers cannot
construct a word or request a reference without passing that check. A
`PhantomData<&'input ParseNode<'input>>` retains the arena lifetime without
widening the word. The compiled probe round-trips a real boundary pointer and
event payloads at 0, the narrow limit, the first wide value, and `usize::MAX`
for every payload-bearing tag.

A plain untagged union is rejected: selecting the wrong field after emitter
drift could construct an invalid reference and cause undefined behavior. A
safe Rust enum is also rejected for the performance premise: both a current
event/reference enum and a packed-word/reference enum are 16 bytes, widening
every boundary record and undoing the common event-width saving.

## 5. Memo and orchestrator segments remain contiguous

Thin memo currently snapshots two slices and replays them with two
`extend_from_slice` calls. The merged word interval is one contiguous slice;
the same stamp and end position apply, and a hit appends that slice verbatim.
The one-carrier representation can therefore use one inline-small segment.
This slice assigns no time to that change because the accepted thin-memo
success ranges are already owned.

The sub-root orchestrator similarly replaces two marks, two cursors, two
consumption assertions, and two truncates with one of each. Nested boundary
orchestrators remain stack-disciplined: an inner match/build/truncate completes
before the outer match appends the returned node as one boundary record.

## 6. Pricing boundary

The source proof establishes feasibility, not net speed. The preserved machine
profile contains 215 unowned direct boundary-Vec metadata PCs and 30/19/40
samples across the three bands, worth a gross 1.452693317 ns. But the unified
carrier adds a boundary kind test and pointer decode, and the preserved binary
cannot measure those not-yet-existing instructions. The strict contribution is
therefore zero.

Likewise, a unified packed tape copies the same `8*events + 8*boundaries` bytes
as separately packed tapes. It removes one call/setup lane, not the boundary
bytes; crediting the full 5.727016718 ns boundary-copy child is explicitly
impossible. One initial allocation can disappear, but crediting the entire
1.687336492 ns boundary allocator child is only an absolute ceiling because the
surviving allocation becomes larger.

Verdict: **FEASIBLE-HOLD**. The design is lossless and signoff-capable, but it
does not add a strict floor-clearing amount without a later owned implementation
and A/B measurement (preferably as part of a larger carrier batch).
