# Inside the Parser: Termination & Performance

> **Part II · Inside PGEN.** This chapter is for contributors who want to understand
> how a PGEN parser stays fast and bounded before they change the engine. It explains
> mechanisms in prose; it does not paste Rust. For the last 100% of detail, follow the
> [Source Map](source-map.md) into the code.

## The promise

A generated PGEN parser must **terminate in bounded, near-linear time and bounded
memory on every valid input**. A parser that hangs, or that exhausts the host's RAM,
is treated as a *correctness defect* — not as "slow." Everything in this chapter
exists to make that promise true, and — just as importantly — to make it *checkable*.

The running stress case throughout is `uvm_pkg` (the Accellera UVM library,
~3 MB of preprocessed SystemVerilog). It is the hardest input PGEN parses today, so
its numbers are the honest upper bound.

## Why a PEG needs a memo — and why that is the whole tension

PGEN parsers are **PEGs** (ordered-choice grammars). A naive PEG can backtrack
exponentially: the same rule can be re-attempted at the same position an exponential
number of times across different alternatives. **Packrat** parsing fixes this by
*memoizing* every `(rule, position)` outcome, so each is computed at most once —
turning exponential time into linear time.

That memo is therefore the source of both PGEN's speed **and** its appetite for
memory. Most of this chapter is about keeping the memo (and the work around it)
bounded without giving up the linear-time guarantee.

## The stateful twist (why "packrat = linear" is not free here)

Textbook packrat assumes a **pure** parse function. PGEN is not pure: its
[semantic store](semantic-store.md) lets the grammar consult parse-time state
(*"is this identifier a type?"*) to make decisions. That makes the parse function
*stateful*, and a stateful packrat parser is **not** automatically linear
(Chida & Kawakoya, CC 2020).

We did not assume — we measured. A scaling probe over store-gated inputs of growing
size showed parse time trending **quadratic (~N¹·⁶ and rising)**, while a stateless
baseline stayed linear. So the statefulness was a real, measured super-linearity that
had to be cured, not hand-waved.

## Four mechanisms that keep it bounded

### 1. O(1) state snapshot/restore instead of cloning

Every speculative step (a branch, an optional, a quantifier) must be able to *undo*
the semantic-store changes it made if it fails. The original engine did this by
**cloning the entire semantic state** on every rule transaction and restoring the
clone on failure. That is O(state) per rule call × O(rule calls) = **O(N²)** — and on
uvm it ballooned the parse to ~26 GB and an apparent hang.

The cure is an **efficient snapshot/restore**: a *checkpoint* records only where the
state currently ends (a few integers, O(1)); a *rollback* truncates back to that point
and undoes exactly the changes made since (O(changes)). This is the published-correct
technique (Laurent & Mens, SLE 2016). Result: the O(N²) clone disappeared and the uvm
peak dropped from ~26 GB to ~12–14 GB.

### 2. Anchored terminal matching

Terminals are matched with regular expressions. The original matcher searched the
*whole remaining input* for the pattern and then checked it started at the cursor —
so every *failing* terminal attempt (the common case in ordered choice) scanned up to
the entire 3 MB tail. That is O(remaining input) per attempt = another O(N²).

The cure: compile each terminal pattern **anchored to the cursor**, so a match attempt
only ever looks at the current position — O(match length), never a haystack scan. This
was the single biggest *time* win: uvm parse time fell **~2.5×** (≈181 s → ≈71 s) with
identical output.

### 3. A bounded, outcome-split memo

On uvm the memo records ~26 million `(rule, position)` probes, and **~81% of them are
failures** — "this rule does not start here." A failure needs to remember nothing
except *that* it failed at that position (so the parser can backtrack immediately);
the position is already the key. So the memo is **split by outcome**: failures live in
a lean set (no per-entry payload), and only successes carry a full cached result.

A cautionary note that lives here on purpose: two *other* memo reshapes were tried
first — sharing subtrees by reference, and boxing the entry's optional fields — and
**both were rejected because the measurement said so**, not because they sounded wrong.
Profiling (below) showed the memo's per-entry *shape* was never the memory bottleneck;
the split is kept because it is a cleaner representation and ~20% faster, not because it
saved memory. Which leads to the most important lesson in this chapter.

### 4. A resource guard that fails loud, never silent

No bound is perfect, so a runaway must become a *classified error*, never a silent
hang or a host-OOM. The corpus harness runs every parse under a memory cap and a
timeout (auto-sized to the host), so a pathological input is reported as a bounded,
named failure and the machine stays usable. The longer-term plan adds the same budget
*inside* the parser as an always-emitted severity (a hang is an error, regardless of
how quiet the logs are set).

## Where the memory actually goes (profile, not guess)

For a long time the residual ~12 GB on uvm was *assumed* to be the memo. It is not.
A `vmmap` profile of the uncapped parse tells the real story:

| Quantity | Value |
|---|---|
| Live (allocated) data | ~13.7 GB |
| Number of live allocations | ~71.8 million |
| Allocator fragmentation | ~13.4 GB (≈50%) |
| Physical footprint (uncapped peak) | ~27 GB |

So the cost is **(a)** ~13.7 GB of live data spread over ~72 million *small*
allocations, and **(b)** roughly as much again in allocator fragmentation that so many
small allocations induce. Shrinking the memo's entry shape cannot move either number —
which is exactly why the boxing and split experiments left peak RSS unchanged. (Under
the resource guard's cap the parse is forced tight to ~14.7 GB and still succeeds, so
uvm already fits a safe host; this is a peak/headroom concern, not a "it doesn't fit"
one.)

The two levers that *do* target this, each its own future work:

- **A different global allocator.** Allocators such as mimalloc or jemalloc are built
  for many-small-allocation workloads and fragment far less than the platform default;
  this is a parser-agnostic, build-level change aimed squarely at the ~50% fragmentation.
- **Fewer allocations.** The ~72 M live allocations are the real floor. Driving them
  down (arena/bump allocation for parse nodes, fewer transient clones per rule) is the
  architectural path below ~14 GB.

## How we keep it honest

Three habits make the numbers above trustworthy, and they are the rules a contributor
should follow before claiming any performance win:

1. **Measure the global metric, not a hypothesis.** Peak RSS and wall-clock on the real
   stress input decide — not "this struct looks smaller." The boxing experiment *looked*
   like a 5 GB win and was a small *regression*; only the measurement caught that.
2. **Profile before you reshape.** The memo was "obviously" the memory hog for months;
   one `vmmap` run showed it never was. Build the instrument (PGEN has an opt-in memo
   census and a complexity-scaling probe) and read it first.
3. **Change one thing, then re-measure.** Determinism makes deltas meaningful: the same
   input must produce byte-identical output before and after, so any change in time or
   memory is signal, not noise.

These mechanisms are parser-agnostic: they live in the engine and the code generator,
so every grammar PGEN compiles — SystemVerilog, VHDL, regex, the RTL front-ends —
inherits the same bounded-time, bounded-memory behavior for free.
