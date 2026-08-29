//! `ENGINE-UNIVERSAL-SERVICES.67`(b) leg 3 — the `typed_arena` CHUNK-CAPACITY model.
//!
//! # The question this closes
//!
//! `.67`(b) legs 1–2 measured arena ITEM bytes (`census() × size_of`) and the
//! out-of-arena AST-owned heap. Both are LOWER bounds on resident bytes for one
//! named reason: `typed_arena` allocates in chunks that grow and are never
//! freed until the arena drops, so resident bytes lie in `[items, ~2 × items]`.
//! That single exclusion is what keeps the arena's share of the parse footprint
//! anywhere in **28–71 %** — a band wide enough that the leaf's headline
//! question ("is the footprint the AST, or is it something else?") cannot be
//! answered at all.
//!
//! # Why a model, and why it is not a formula
//!
//! `-0125` refuted the *"chunk capacity is a pure function of item count and
//! item size"* claim: it holds for the `nodes` arena, which is fed only by
//! `alloc`, and it does **not** hold for `shaped_values`/`shaped_pairs`, which
//! are fed by `alloc_extend`. There the crate fills the current chunk
//! item-by-item and, on overflow, calls `reserve(i + 1)` and drains the
//! partially written items into the new chunk, so resident capacity depends on
//! the SEQUENCE of extend sizes and not on their total.
//!
//! This type is therefore driven by the real call sequence — one `alloc()` per
//! `Arena::alloc`, one `alloc_extend(min_hint, actual)` per `Arena::alloc_extend`
//! — and reproduces `typed-arena` 2.0.2's chunk arithmetic step for step. It is
//! an instrument fed by observed events, not a closed form evaluated at the end.
//!
//! # The transcription, line for line
//!
//! From `typed-arena-2.0.2/src/lib.rs`:
//!
//! - `INITIAL_SIZE = 1024`, `MIN_CAPACITY = 1` (`:83`, `:85`);
//! - `Arena::new` → `with_capacity(INITIAL_SIZE / max(1, size_of::<T>()))`,
//!   and `with_capacity` clamps to `max(MIN_CAPACITY, n)` (`:122`, `:136`);
//! - `ChunkList::reserve(additional)` retires `current` INTO `rest` keeping its
//!   capacity, and makes a new `current` of
//!   `max(2 × current.capacity(), additional.next_power_of_two())` (`:550-562`);
//! - `alloc` pushes into `current` while `len < capacity`, else falls into
//!   `alloc_extend(once(value))`, which sees `1 > 0` free slots and so takes the
//!   reserve path — i.e. plain `alloc` doubles and nothing else (`:187-206`);
//! - `alloc_extend` takes the reserve path when `iter_min_len > capacity - len`;
//!   otherwise it pushes item by item and, if the iterator outruns the chunk,
//!   reserves `i + 1` and drains the `i` already-written items forward
//!   (`:222-265`).
//!
//! ⛔ **The one thing a reader must not assume is that the transcription is
//! right because it is careful.** It is checked against the allocator itself:
//! `rust/src/bin/arena_chunk_oracle_probe.rs` drives a REAL `typed_arena::Arena`
//! under a counting `GlobalAlloc` and compares measured live bytes to
//! [`ArenaChunkModel::total_bytes`] — exact equality, over the shapes a parse
//! actually produces and over pseudo-random sequences. A model of an algorithm
//! is a claim about that algorithm, and this one is falsifiable by construction.
//!
//! # The `rest` spine
//!
//! `ChunkList::rest` is itself a `Vec<Vec<T>>`, so retiring chunks allocates a
//! little too. It is modelled here as well ([`spine_bytes`](ArenaChunkModel::spine_bytes))
//! and reported SEPARATELY, because its growth rule is `std`'s amortized
//! `Vec` growth rather than anything `typed-arena` promises. It is
//! sub-kilobyte at every size this repository parses (chunk count is
//! logarithmic), and separating it means a future `std` change falsifies one
//! named quantity instead of silently biasing the headline.

/// `typed-arena-2.0.2/src/lib.rs:83` — initial chunk size in BYTES.
const INITIAL_SIZE: usize = 1024;
/// `typed-arena-2.0.2/src/lib.rs:85` — minimum chunk capacity in ITEMS.
const MIN_CAPACITY: usize = 1;
/// `Vec<T>` is a pointer plus two `usize` for every `T`, so the `rest` spine's
/// element size is the same for all four arenas.
const SPINE_ELEM_BYTES: usize = std::mem::size_of::<Vec<u8>>();
/// `std`'s `RawVec::MIN_NON_ZERO_CAP` for an element of 24 bytes (`> 1` and
/// `<= 1024`). Only the spine uses it; chunk growth never does.
const SPINE_MIN_NON_ZERO_CAP: usize = 4;

/// One `typed_arena::Arena<T>`'s chunk list, modelled item-exactly.
///
/// Construct with [`new`](Self::new) using the arena's `size_of::<T>()`, then
/// mirror every allocation call. Every field is in ITEMS unless its name says
/// bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArenaChunkModel {
    item_size: usize,
    current_cap: usize,
    current_len: usize,
    /// Summed capacity of the chunks already moved into `rest`.
    retired_cap: usize,
    /// Summed length of those chunks (after any `alloc_extend` drain).
    retired_len: usize,
    /// `rest.len() + 1` — the current chunk always exists.
    chunks: usize,
    /// The highest `len` the CURRENT chunk has reached.
    ///
    /// ⛔ **NOT the same as `current_len`, and the difference is what makes this
    /// field necessary**: `alloc_extend`'s overflow path DRAINS already-written
    /// items out of a chunk, lowering its `len` while leaving those pages
    /// written.
    current_high_water: usize,
    /// Summed high-water mark of the retired chunks — the item slots that were
    /// actually WRITTEN, as opposed to merely reserved.
    retired_touched: usize,
    /// Capacity of the `rest` vector itself, in elements.
    spine_cap: usize,
    /// Times an `alloc_extend` produced MORE items than its `size_hint` lower
    /// bound had reserved room for, i.e. the reserve path under-reserved and
    /// `Vec::extend` had to grow the chunk itself.
    ///
    /// ⛔ Every such event makes the chunk-byte figure a MODEL of `std`'s `Vec`
    /// growth rather than of `typed_arena`'s, so it is counted and reported
    /// rather than absorbed. PGEN's own two `alloc_extend` call sites pass
    /// `ExactSizeIterator`s, so the expected count is 0 and a non-zero one is a
    /// finding.
    extend_regrow_events: usize,
}

impl ArenaChunkModel {
    /// Mirror `Arena::<T>::new()` for an item of `item_size` bytes.
    pub fn new(item_size: usize) -> Self {
        let size = item_size.max(1);
        let cap = (INITIAL_SIZE / size).max(MIN_CAPACITY);
        ArenaChunkModel {
            item_size,
            current_cap: cap,
            current_len: 0,
            retired_cap: 0,
            retired_len: 0,
            chunks: 1,
            current_high_water: 0,
            retired_touched: 0,
            spine_cap: 0,
            extend_regrow_events: 0,
        }
    }

    /// Mirror `ChunkList::reserve` (`typed-arena-2.0.2/src/lib.rs:550-562`):
    /// the live chunk is moved into `rest` KEEPING its capacity, and a new one
    /// is created at `max(2 × current_capacity, next_power_of_two(additional))`.
    fn reserve(&mut self, additional: usize) {
        let double_cap = self.current_cap.saturating_mul(2);
        let required_cap = additional.checked_next_power_of_two().unwrap_or(usize::MAX);
        let new_cap = double_cap.max(required_cap);
        self.retired_cap += self.current_cap;
        self.retired_len += self.current_len;
        self.retired_touched += self.current_high_water;
        self.push_spine();
        self.chunks += 1;
        self.current_cap = new_cap;
        self.current_len = 0;
        self.current_high_water = 0;
    }

    /// Every mutation of `current_len` ends here, because the high-water mark is
    /// the only thing a DRAIN cannot lower.
    fn touch(&mut self) {
        if self.current_len > self.current_high_water {
            self.current_high_water = self.current_len;
        }
    }

    /// `rest.push(chunk)` — `std`'s amortized `Vec` growth, modelled because
    /// the retired chunks have to live somewhere.
    fn push_spine(&mut self) {
        let len = self.chunks - 1; // elements already in `rest`
        if len == self.spine_cap {
            let doubled = self.spine_cap.saturating_mul(2);
            self.spine_cap = doubled.max(len + 1).max(SPINE_MIN_NON_ZERO_CAP);
        }
    }

    /// Mirror `Arena::alloc`.
    pub fn alloc(&mut self) {
        if self.current_len < self.current_cap {
            self.current_len += 1;
            self.touch();
        } else {
            // `alloc_slow_path` -> `alloc_extend(iter::once(value))`, whose
            // `iter_min_len` of 1 always exceeds the 0 free slots that got us
            // here, so it is the reserve path with `additional = 1`.
            self.reserve(1);
            self.current_len = 1;
            self.touch();
        }
    }

    /// Mirror `Arena::alloc_extend`.
    ///
    /// `min_hint` is the iterator's `size_hint().0` as `typed_arena` sees it;
    /// `actual` is how many items it yielded (the length of the slice
    /// `alloc_extend` hands back). Passing both is what makes this an
    /// instrument rather than an assumption: a lying `size_hint` takes a
    /// different branch, and the difference is visible here.
    pub fn alloc_extend(&mut self, min_hint: usize, actual: usize) {
        let free = self.current_cap - self.current_len;
        if min_hint > free {
            self.reserve(min_hint);
            if actual > self.current_cap {
                // `chunks.current.extend(iter)` outran the reserved chunk, so
                // `Vec` itself grew: `max(2 × cap, required)`.
                self.extend_regrow_events += 1;
                self.current_cap = self.current_cap.saturating_mul(2).max(actual);
            }
            self.current_len = actual;
            self.touch();
        } else if actual <= free {
            self.current_len += actual;
            self.touch();
        } else {
            // The iterator outran its own `size_hint`: the crate pushed `free`
            // items, hit `len == capacity`, reserved `free + 1`, drained those
            // `free` items forward and put the whole remainder in the new chunk.
            let drained = free;
            self.current_len += drained;
            // The drained items WERE written into the old chunk before being
            // moved, so its high-water mark records them even though `reserve`
            // is about to lower its `len` again.
            self.touch();
            self.reserve(drained + 1);
            self.retired_len -= drained;
            if actual > self.current_cap {
                self.extend_regrow_events += 1;
                self.current_cap = self.current_cap.saturating_mul(2).max(actual);
            }
            self.current_len = actual;
            self.touch();
        }
    }

    /// Items the arena holds — mirrors `Arena::len()`, and exists so a caller
    /// can CHECK the model against the arena instead of trusting it.
    pub fn len_items(&self) -> usize {
        self.retired_len + self.current_len
    }

    /// Item slots the arena has actually allocated, live and retired.
    pub fn capacity_items(&self) -> usize {
        self.retired_cap + self.current_cap
    }

    /// Chunks allocated so far (`rest.len() + 1`).
    pub fn chunks(&self) -> usize {
        self.chunks
    }

    /// Bytes the allocator handed this arena — the quantity `census() × size_of`
    /// was a lower bound for.
    ///
    /// ⛔ **THIS IS ALLOCATED, NOT RESIDENT, AND THE TWO ARE DIFFERENT ON EVERY
    /// LAZY-COMMIT OS.** A freshly allocated chunk is untouched anonymous memory;
    /// pages enter the process's RSS when they are first WRITTEN. Use
    /// [`touched_bytes`](Self::touched_bytes) as the RSS-relevant figure and this
    /// as the upper bound — `.67`(b) exists to attribute peak RSS, and reporting
    /// an allocation figure against an RSS denominator would inflate the share by
    /// exactly the arena's unused slack.
    pub fn chunk_bytes(&self) -> usize {
        self.capacity_items() * self.item_size
    }

    /// Item slots that were actually WRITTEN across every chunk.
    ///
    /// ⭐ This is not `len_items()`: a chunk is usually retired FULL, so its
    /// whole capacity is written, and `alloc_extend`'s drain path lowers a
    /// retired chunk's `len` without un-writing its pages. It is not
    /// `capacity_items()` either: the CURRENT chunk's tail is typically
    /// untouched, and after a large `alloc_extend` a partly filled chunk can be
    /// retired with slack that was never written.
    pub fn touched_items(&self) -> usize {
        self.retired_touched + self.current_high_water
    }

    /// `touched_items × size_of::<T>()` — the bytes this arena can actually be
    /// holding in RSS. ⚠️ Still an approximation in one named direction: the OS
    /// commits whole PAGES, so a partly written page is fully resident and this
    /// rounds DOWN by at most one page per chunk (chunk counts are logarithmic,
    /// so that is kilobytes).
    pub fn touched_bytes(&self) -> usize {
        self.touched_items() * self.item_size
    }

    /// Resident bytes in the `rest` spine (see the module docs).
    pub fn spine_bytes(&self) -> usize {
        self.spine_cap * SPINE_ELEM_BYTES
    }

    /// Every byte this arena has taken from the allocator (chunks + spine).
    /// ⛔ ALLOCATED, not resident — see [`chunk_bytes`](Self::chunk_bytes).
    pub fn total_bytes(&self) -> usize {
        self.chunk_bytes() + self.spine_bytes()
    }

    /// The RSS-relevant twin of [`total_bytes`](Self::total_bytes): written chunk
    /// bytes plus the spine, which is written in full as it grows.
    pub fn touched_total_bytes(&self) -> usize {
        self.touched_bytes() + self.spine_bytes()
    }

    /// See [`extend_regrow_events`](Self::extend_regrow_events) — expected 0.
    pub fn extend_regrow_events(&self) -> usize {
        self.extend_regrow_events
    }

    /// `capacity_items / len_items`, the waste factor this leg exists to
    /// measure. `None` for an arena nothing was allocated in, because a ratio
    /// with a zero denominator is not "1.0".
    pub fn overhead_ratio(&self) -> Option<f64> {
        let items = self.len_items();
        (items > 0).then(|| self.capacity_items() as f64 / items as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `ParseNode` is 48 bytes, so the `nodes` arena starts at 1024/48 = 21 and
    /// doubles — the one arena for which `-0125` conceded a closed form.
    #[test]
    fn alloc_only_arena_doubles_from_the_initial_item_capacity() {
        let mut m = ArenaChunkModel::new(48);
        assert_eq!(m.capacity_items(), 21);
        assert_eq!(m.chunks(), 1);
        for _ in 0..21 {
            m.alloc();
        }
        // Still one chunk: the 21st item fills it exactly.
        assert_eq!(m.capacity_items(), 21);
        assert_eq!(m.chunks(), 1);
        m.alloc();
        // 22nd item retires the 21-slot chunk and opens a 42-slot one.
        assert_eq!(m.capacity_items(), 63);
        assert_eq!(m.chunks(), 2);
        assert_eq!(m.len_items(), 22);
        for _ in 0..41 {
            m.alloc();
        }
        assert_eq!(m.len_items(), 63);
        assert_eq!(m.capacity_items(), 63);
        m.alloc();
        assert_eq!(m.capacity_items(), 21 + 42 + 84);
        assert_eq!(m.chunks(), 3);
    }

    /// ⭐ The `footprint()` doc-comment asserted resident chunk bytes could
    /// exceed live item bytes "by up to ~2×". Made exact and checked at every
    /// item count: `capacity <= 2 × items + initial_capacity`, because the
    /// retired chunks sum to `cap0 × (2^k − 1)` while the items that forced the
    /// last grow already exceed that same sum.
    ///
    /// ⛔ The ratio ITSELF is unbounded — at one live item it is 21 — so a test
    /// asserting "under 2×" would be false for a reason that has nothing to do
    /// with the doubling. The additive form is the true statement.
    #[test]
    fn alloc_only_capacity_stays_within_twice_the_items_plus_one_chunk() {
        let mut m = ArenaChunkModel::new(48);
        let initial = m.capacity_items();
        let mut worst_tail: f64 = 0.0;
        for _ in 0..100_000 {
            m.alloc();
            assert!(
                m.capacity_items() >= m.len_items(),
                "capacity can never be below the live items"
            );
            assert!(
                m.capacity_items() <= 2 * m.len_items() + initial,
                "capacity {} exceeded 2x items {} + {initial}",
                m.capacity_items(),
                m.len_items()
            );
            if m.len_items() > 1_000 {
                worst_tail = worst_tail.max(m.overhead_ratio().unwrap());
            }
        }
        // Above a thousand items the bound is genuinely approached, so the
        // "~2x" is tight rather than merely satisfied.
        assert!(
            (2.0..2.05).contains(&worst_tail),
            "worst tail overhead {worst_tail} should sit just above 2x"
        );
    }

    /// `-0125`'s qualification, made executable: two arenas with the SAME item
    /// total and item size land on DIFFERENT resident capacities when the
    /// extend sequence differs. A closed form in `(count, size)` cannot do this.
    #[test]
    fn extend_capacity_depends_on_the_sequence_not_the_total() {
        let mut one_big = ArenaChunkModel::new(56);
        one_big.alloc_extend(1000, 1000);

        let mut many_small = ArenaChunkModel::new(56);
        for _ in 0..1000 {
            many_small.alloc_extend(1, 1);
        }

        assert_eq!(one_big.len_items(), many_small.len_items());
        assert_ne!(
            one_big.capacity_items(),
            many_small.capacity_items(),
            "if these agreed, leg 3's re-pricing would have been unnecessary"
        );
        // 18-slot initial chunk retired empty, then next_power_of_two(1000).
        assert_eq!(one_big.capacity_items(), 18 + 1024);
        assert_eq!(one_big.chunks(), 2);
    }

    /// An `alloc_extend` bigger than the free space retires the live chunk even
    /// when that chunk is EMPTY — capacity is lost, not carried over.
    #[test]
    fn a_large_extend_retires_a_partly_used_chunk_and_loses_its_free_slots() {
        let mut m = ArenaChunkModel::new(64); // 1024/64 = 16
        m.alloc();
        m.alloc();
        assert_eq!(m.capacity_items(), 16);
        m.alloc_extend(100, 100);
        // 16-slot chunk retired holding 2 items; new chunk is
        // max(2*16, next_power_of_two(100)) = 128.
        assert_eq!(m.capacity_items(), 16 + 128);
        assert_eq!(m.len_items(), 102);
        assert_eq!(m.chunks(), 2);
    }

    /// A `size_hint` that lies takes the OTHER branch, and the model has to
    /// follow it there — including the drain that moves already-written items
    /// forward, which is why `len_items` is not simply additive.
    #[test]
    fn an_understated_size_hint_takes_the_item_by_item_overflow_path() {
        let mut m = ArenaChunkModel::new(64); // capacity 16
        m.alloc_extend(0, 20);
        // Pushed 16, overflowed, reserved 17 -> max(32, 32) = 32, drained the
        // 16 forward, so the retired chunk holds NOTHING and the new one holds
        // all 20.
        assert_eq!(m.chunks(), 2);
        assert_eq!(m.capacity_items(), 16 + 32);
        assert_eq!(m.len_items(), 20);
        assert_eq!(m.extend_regrow_events(), 0);
    }

    /// The spine costs something, it is counted, and it stays negligible.
    #[test]
    fn the_rest_spine_is_counted_and_stays_sub_kilobyte() {
        let mut m = ArenaChunkModel::new(48);
        assert_eq!(m.spine_bytes(), 0, "an arena that never retired a chunk has no spine");
        for _ in 0..10_000_000 {
            m.alloc();
        }
        assert!(m.chunks() > 18, "10M items must have grown many chunks: {}", m.chunks());
        assert!(
            m.spine_bytes() <= 1024,
            "the spine must stay sub-kilobyte even at 10M items: {} B",
            m.spine_bytes()
        );
        assert!(m.spine_bytes() > 0);
    }

    /// ⛔ THE DISTINCTION THE FOOTPRINT SHARE TURNS ON: an `alloc`-only arena
    /// retires every chunk FULL, so `touched` is capacity minus the current
    /// chunk's unused tail — never `len`, and never capacity.
    #[test]
    fn touched_is_between_the_items_and_the_capacity_and_equals_neither() {
        let mut m = ArenaChunkModel::new(48);
        for _ in 0..22 {
            m.alloc();
        }
        // Chunks: a FULL retired 21 + a 42-slot chunk holding 1.
        assert_eq!(m.capacity_items(), 63);
        assert_eq!(m.len_items(), 22);
        assert_eq!(m.touched_items(), 22, "21 written + 1 written");
        for _ in 0..41 {
            m.alloc();
        }
        assert_eq!(m.len_items(), 63);
        assert_eq!(m.touched_items(), 63, "both chunks now written in full");
        m.alloc();
        assert_eq!(m.capacity_items(), 147, "21 + 42 + 84");
        assert_eq!(m.touched_items(), 64, "the 84-slot chunk holds ONE written item");
        assert!(m.touched_bytes() < m.chunk_bytes());
    }

    /// A big `alloc_extend` retires a chunk that was never filled, so `touched`
    /// must NOT be "capacity minus the current tail" either.
    #[test]
    fn a_partly_filled_retired_chunk_contributes_only_what_was_written() {
        let mut m = ArenaChunkModel::new(64); // capacity 16
        m.alloc();
        m.alloc();
        m.alloc_extend(100, 100);
        assert_eq!(m.capacity_items(), 16 + 128);
        assert_eq!(m.len_items(), 102);
        assert_eq!(
            m.touched_items(),
            102,
            "2 written in the retired 16-slot chunk, 100 in the new one"
        );
    }

    /// ⭐ The drain path is the reason `touched` cannot be derived from `len`:
    /// it moves already-WRITTEN items out of a chunk, lowering that chunk's len
    /// while its pages stay resident.
    #[test]
    fn a_drained_chunk_keeps_the_pages_it_wrote() {
        let mut m = ArenaChunkModel::new(64); // capacity 16
        m.alloc_extend(0, 20);
        assert_eq!(m.len_items(), 20, "the retired chunk was drained empty");
        assert_eq!(
            m.touched_items(),
            36,
            "16 written into the retired chunk before the drain, 20 into the new one"
        );
        assert!(m.touched_items() > m.len_items());
    }

    /// A zero-sized item cannot divide by zero and cannot report a zero-byte
    /// capacity as "measured" — `Arena::new` clamps the size to 1.
    #[test]
    fn a_zero_sized_item_type_clamps_instead_of_dividing_by_zero() {
        let mut m = ArenaChunkModel::new(0);
        assert_eq!(m.capacity_items(), 1024);
        m.alloc();
        assert_eq!(m.chunk_bytes(), 0, "zero-sized items occupy no bytes, and that is not an error");
    }
}
