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
  For the regex *speed* campaign this has now been measured and banked: a mimalloc-class
  global allocator is **−29.2%** on regex cold-parse (RGX-0078·5.i.7, scoreboard row
  *mimalloc*), and a never-free bump-arena ceiling probe priced the whole `malloc`/`free`
  axis at **−40.0%**. Because a library must not impose a `#[global_allocator]` on its
  consumers, it is recommended to embedders (the regex integration contract) rather than
  forced in the lib.
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

---

## The latency campaign: closing the gap to PCRE2 (small-input speed)

The section above is about the *big* input (uvm, ~3 MB) staying **bounded**. This section
is about the opposite end — the *small* input being **fast**. They are different problems
with different profiles, and this is the story of the second one, told as a running
scoreboard so it can be watched as it improves.

> The campaign's full narrative — the method, the refuted roads, and the named lessons —
> is told in its own chapter, [The Speed Journey: 496 µs → ≈9.5 µs](speed-journey.md).
> This section is the *mechanics* record: what each lever is, how it works, and what it
> measured.

### Why it matters, and what "fast enough" means

PGEN's regex parser is the compile front-end for [RGX](parser-families.md). When a tool
compiles thousands of regular expressions, PGEN's *parse time per pattern* is that tool's
*compile time per pattern* — so it is measured not against "is it linear?" but against the
industrial baseline for the same job: **PCRE2's compiler**. The closure target is a
**relative** one:

> geomean( PGEN-parse-time / PCRE2-no-JIT-compile-time ) **< 5×** over the PCRE2 test corpus.

Correctness is the floor, never the trade: in this campaign **any** optimization that flips
a single conformance verdict or breaks parity is rejected outright, no matter how much
faster it is. Speed is earned only on top of a parser that stays exactly as accurate as it
was. (This is the operational form of *correctness before speed*.)

### The systemic thesis

The slowness is **not** a regex quirk — it is hypothesized to be inherent to the *shared*
parse machinery every PGEN grammar uses, so it slows JSON, VHDL and SystemVerilog too. That
thesis is testable, and we test it rather than assume it: every profile below is taken
across **regex + JSON + VHDL**, and a lever only counts as a real win if the cost it removes
lives in **shared engine code** (so all parsers benefit), measured globally.

### Where the small-input time actually goes (profile, not guess)

Same discipline as the memory story: profile first. A release-build `sample` profile of a
hot regex parse (macOS, 1 ms) attributes the self-time like this:

| Cost centre | Share of self-time | What it is |
|---|---|---|
| `libsystem_malloc` (alloc/free/memmove) | **~59%** | the *symptom* — the parser allocates a great deal |
| Backtrack/rollback path | ~24% (call-graph) | work done on **every failed speculation** |
| Per-rule annotation-table lookups (SipHash) | ~6% | a hash lookup per rule entry, even when the table is empty |
| String formatting | ~4% | trace-context strings built and discarded |

The headline is that the dominant cost is **allocation**, and the dominant *source* of that
allocation is the **backtracking itself**. PGEN's `|` is a longest-match tournament: at each
position it tries *every* viable alternative and keeps the longest. Parsing a 4-character
literal like `"test"` therefore tries ~30 `atom` alternatives per character, and **most of
them fail and backtrack** — so the per-backtrack overhead is multiplied by a very large
number. That is why a trivial pattern still costs ~130–160 µs: it is not the matching, it is
the machinery around each speculative attempt.

A red herring worth recording (the chapter's recurring lesson): the parser rebuilds a
per-grammar directive table in its constructor, and that *looked* like an obvious ~160 µs
fixed cost. The profile refuted it — construction is **~0.7%** of the time; the parse
dominates. We did not act on the guess.

The systemic check held: JSON (a predicate-free grammar) and VHDL (a predicate-carrying one)
show the *same* shape — the same allocator dominance, the same shared `RecursionGuard` and
backtrack-path costs — confirming the cost is in the engine, not any one grammar.

### The scoreboard

Regex 8-pattern micro-bench, release build, geomean of per-pattern parse time. Because the
bench machine is often shared, the **noise-floor minimum** per pattern is the reported
statistic (it is immune to unrelated CPU contention and reproduces exactly run-to-run);
every delta is proven by a *decisive baseline* — stash only the change, rebuild, re-measure
back-to-back, so the difference is caused by the change and nothing else.

| # | Lever | Layer | Regex geomean | Δ | Status |
|---|---|---|---|---|---|
| RGX-0073 | Optim #1–#16 (rule-names `&'static`, `FxHashMap` memo, anchored terminals, borrow-in-place, predicate-free fast-paths, worker cache, …) | engine + codegen | — | ~2–3× under the pre-optim reference | landed |
| 6.0 | Memo split by outcome | engine | — | ~20% faster (uvm) | landed |
| RGX-0078 · 1 | **Rollback scope-restoration guard** — skip the active-chain clone + `scopes` rebuild on the backtrack path when no scope state changed | engine (shared runtime) | 422 µs → **344 µs** | **−18.6%** | **landed** ✓ |
| RGX-0078 · 4.a | **Free AOT build flags** — `[profile.release] lto="fat" + codegen-units=1` (was cargo defaults: no LTO, 16 codegen units) | build config (parser-agnostic) | 332 µs → **310 µs** | **−6.7%** | **landed** ✓ |
| RGX-0078 · 4.b | `target-cpu=native` (on top of 4.a) — commonly assumed a free win | build flag (machine-specific) | 313 µs → 324–327 µs | **+4% (worse)** | **rejected** ✗ |
| RGX-0078 · 4.c | PGO (profile-guided optimization, on top of 4.a) | build process | 308.6 µs → 305.4 µs | −1% (lower bound) | **not landed** ✗ |
| RGX-0078 · 5.a | **FxHash the per-rule annotation-table lookups** — swap the two per-rule-entry directive maps (`directives_by_rule` + `branch_directives_by_rule`) from the std SipHash to `FxHashMap` | engine (shared runtime) | 320 µs → **303 µs** | **−5.4%** | **landed** ✓ |
| RGX-0078 · 5.c | **First-set predictive dispatch** — before a top-level branch tournament evaluates a branch, peek the next input byte and SKIP any non-nullable branch whose sound FIRST-set can't begin a match there | codegen (parser-agnostic) | 303 µs → **76 µs** | **−75% (~4×)** | **landed** ✓ |
| RGX-0078 · 5.d.2 | **Lazy winner-only materialization** — defer each branch's return-annotation transform + clone and run it once, for the winner only (explore in lockstep, don't re-do work per candidate) | codegen (parser-agnostic) | 75.5 µs ≈ 75.9 µs | **~0% (neutral)** | **reverted** ✗ (idea preserved) |
| RGX-0078 · 5.d.4 | **Node arena** — allocate every child `ParseNode` in a per-parse bump arena (`typed-arena`) and hold children as `&'input` references instead of `Box`/`Vec<ParseNode>`, so the profiled ~55% construction-`malloc` collapses to a handful of arena growths freed in one shot | codegen + engine (parser-agnostic) | 75.0 µs → **58.6 µs** | **−21.9%** | **landed** ✓ |
| RGX-0078 · 5.g | **Construction cache** — the post-arena re-profile found ~12% of every parse was spent REBUILDING the grammar-constant compiled annotation tables (std-map SipHash inserts, ~92 small strings, every key hashed twice, full drop at parse end); they are now built once per process and every parser instance shares the one table | codegen + engine (parser-agnostic) | 58.6 µs → **56.8 µs** | **−3.8%** | **landed** ✓ |
| RGX-0078 · 5.e | **GLL + graph-structured stack** (the research-grade general lockstep form) — adjudicated by a literature-first design spike instead of a build: the engine is already a memoized, first-set-pruned, near-deterministic recursive-descent parser (backtrack residue ~2.2%), exactly the regime where the literature shows GLL's descriptor/GSS/SPPF bookkeeping costs orders of magnitude more than adaptive top-down parsing buys | (not built — design spike only) | — | predicted net-negative | **refuted** ✗ |
| RGX-0078 · 5.i.1 | **Cost-decomposition census** (the planner rung's step 0) — six measurement-only strip-variants of the generated parser, each byte-identity-proven, pricing every piece of per-entry machinery; discovered that ≈32% of the parse is unconditional bookkeeping waste (trace-naming strings, tournament allocs, context strings) and fixed the planner pass order (see the census section below) | (measurement only — nothing landed) | 56.7 µs → 38.7 µs with all strips applied | **−31.7% measured ceiling** | **measured** — P0 landed as 5.i.2 |
| RGX-0078 · 5.i.2 | **P0 — lazy/no-alloc protocol hygiene** (the census's V1+V2+V3 surfaces made permanent, observability-preserving): rollback labels travel as a cheap `Copy` enum materialized into text only inside the trace-enabled branch (was: two `String`s per failed speculation + a `format!` per successful tournament branch, consumed only under trace); the branch tournament iterates its rotated order as `(step + offset) % n` instead of collecting a `Vec` per execution, and the partition-group string is built only when partitioning is enabled; the rule-context stack stores `Cow<'static, str>` pushed borrow-only from rule-name literals (generated parsers) and interned names (interpreter). Trace output with tracing ON is byte-for-byte unchanged — proven by a 923-line trace-payload diff | engine + codegen (parser-agnostic) | 56.7 µs → **42.0 µs** | **−25.8%** | **landed** ✓ |
| RGX-0078 · 5.i.3 | **P2 — degenerate-tournament byte-switch dispatch**: where FIRST-set analysis PROVES a rule's top-level branch tournament degenerate (every branch's admissible first bytes decided and pairwise DISJOINT, terminals whitespace-sensitive, no branch predicates or branch-start effects), the generated rule dispatches with ONE `match` on the next byte straight to the only branch that could match — eliding the per-branch guard-scan loop, the tournament semantic checkpoint, the winner's delta-extract/rollback/replay round-trip, and the `should_take` cascade (the sole candidate still runs under `try_parse`, so failure restores state exactly as before). 41 of regex's 112 top-level choice sites qualify (the single-char alternation leaves — `letter`'s 52-arm tournament becomes one byte switch); a new DEGENERACY census (`--report-fusibility-census`) measured the surface and predicted −3–7% before any code | codegen (parser-agnostic; census-verified gate) | 42.0 µs → **39.7 µs** | **−5.3%** | **landed** ✓ |
| RGX-0078 · 5.i.4 | **P1a — cascade/wrapper inlining, memo preserved**: a call site of a provably collapsible wrapper rule (on no reference cycle, no semantic directive in any phase, not the entry rule, not dialect-gated — the inline census's gates) receives the rule's BODY inline under a new emitted `inlined_frame_call` engine helper instead of a method call. The helper preserves the per-frame observability verbatim (entry counter, transactional coverage push, furthest-position, `memoized_call` with the memo intact, the method-identical exit trace lines), so rule-entry counters, outcome dumps, ASTs, and certification pins stay byte-identical BY MEASUREMENT; elided per frame: recursion-guard enter/exit, rule-context push/pop, the `--trace-rules` scope probe, the two needs-raw annotation probes (statically folded for every directive-free rule — a ride-along that applies to rule methods too), and the call frame itself. A measured code-size budget (capped-transitive body weight ≤ 12 gen-AST nodes, weight × reference-sites ≤ 192, shared with the census's `INLINE-DECISIONS` report) bounds the duplication: 128 of regex's 204 eligible rules are inlined; a tighter budget variant was built and measured — and lost | codegen + one emitted helper (parser-agnostic; census-shared gate + budget) | ≈39.7 µs → **≈36 µs** | **≈−4–7%** (alternated sessions −3.5/−3.9/−6.7%) | **landed** ✓ |
| RGX-0078 · 5.i.4 | **P1b — memo elision at inlined frames**: the inlined-frame helper runs the body DIRECTLY instead of through `memoized_call` — eliding, per inlined entry, the packrat probe cascade (fail-set, tainted-map, success-map) and, per inlined success, the memo insert (`node.clone()` + semantic-delta/coverage extraction + map insert). Result-neutral by the memo's own soundness contract (a pure, taint-gated cache — replay ≡ re-execution wherever a replay was legal; the inlined subgraph is additionally acyclic and directive-free by the census gates); a former cached hit re-executes the budget-capped body, whose non-inlined children keep their own memoized methods. ASTs, per-rule COMMITTED counts, and certification pins stay byte-identical BY MEASUREMENT; raw-entry counters change *truthfully* where former hits re-execute (the census priced 324 lost hits at the budget, ceiling ≈−6–18%, recorded before emission) | codegen (one emitted-helper hunk; parser-agnostic) | ≈36 µs → **≈32 µs** | **≈−7–12%** (five alternated rounds −7.2…−12.2%, all 8 patterns faster; best-mins −10.3%) | **landed** ✓ |
| RGX-0078 · 5.i.5 | **P3c-i — the store's epoch fast path**: a re-profile of the harvested benchmark refuted the remaining "selective machinery" paper ceilings (the whole packrat cache now costs ~3%, the recursion guard ~0.5%) and named the real residue — the C3-B tournament's checkpoint/delta/rollback protocol, paid in full even when the semantic store never changed (98.8% of all rollbacks and 93.6% of all delta extractions, measured by a four-counter census). The fix is one engine-side O(1) proof: the checkpoint stamps the store's monotone write epoch, and when the epoch and the deferred-obligation count are both unchanged, the delta extraction returns a canonical empty delta (skipping its two unconditional clones) and the rollback returns immediately (skipping the index walk, truncations, and chain compares) — counters, traces, and every parse output byte-identical | engine (`semantic_runtime.rs` only; every parser + the interpreter inherit it, no regeneration) | ≈35 µs-era → **≈31 µs** | **−11.2%** (five alternated rounds 0.870–0.899, all 8 patterns faster; best-mins 34.51 → 30.63 µs) | **landed** ✓ |
| RGX-0078 · 5.i.6 | **P4-i — the `@constraint` constant-fold**: the fresh profile's top named symbol (6–8% of the parse) was the generated rule exits re-evaluating grammar-constant `@constraint` strings — every live one descriptive prose like "produces control character" — through the full relational-expression machine (~9 byte-walks + ~9 allocations per evaluation, ≈1 µs each) to conclude, every time, that non-empty prose means "true". The generator now runs the same classification once, at code-generation time, under a gate strictly narrower than the runtime evaluator (every banned character maps to one evaluator feature; bare identifier chains are excluded because a `$`-less reference resolves against parse content), and emits *nothing* for a provably constant-true check — dead code by construction, no observable surface; real relational expressions keep the runtime path verbatim | codegen only (`ast_based_generator.rs`; regenerated parsers, engine + grammars untouched) | ≈31 µs-era → **≈27.4 µs** | **−12.1%** (five alternated rounds 0.868–0.887, all faster; best-mins 31.30 → 27.38 µs; the evaluation-heavy patterns −26…−31%) | **landed** ✓ |
| RGX-0078 · 5.i.7 D0 | **FIRST-set resolution through regex-literal terminals** — the shared FIRST analysis learns to derive a regex terminal's admissible first bytes from the pattern's own syntax tree (character classes — negated included — map to UTF-8 lead-byte ranges, literals contribute their first byte, an exactness-licensed subtraction handles the negative-lookahead idiom, anything undecidable keeps the conservative "always try"); pure analysis — the two already-landed emissions (prune guards + byte-switch dispatch) simply see more at the next regeneration (byte-switch sites 41 → 60, the literal-matching spine guarded) | analysis only (`first_set.rs`; parsers regenerate, engine untouched) | ≈27.4 µs → **≈24.7 µs** | **−10.0%** (all 5 rounds, all 8 patterns faster; 346 discarded entries killed, committed counts exactly unchanged) | **landed** ✓ |
| RGX-0078 · 5.i.7 D1 | **Second-byte (FIRST₂) prune guards** — the per-branch prune guard learns to refuse on the *second* byte too (a `\Q` branch skipped outright when the input reads `\b`), licensed branch-by-branch by a furthest-position-parity proof (only branches that provably enter no rule past offset 0 earn the sharper guard); the guard form needs no pairwise disjointness, so it fires beyond the sketched nested-switch model | codegen analysis + guard emission (parser-agnostic) | ≈23.6 µs → **≈18.9 µs** | **−19.8%** (all 5 rounds; 541 of 1,540 residual discards killed, committed exactly unchanged — over-delivered the −6–12% ceiling) | **landed** ✓ |
| RGX-0078 · 5.i.7 Q | **Quantifier attempt-elision guards** — min-0 quantified/optional sites skip their element's doomed attempt when the next byte cannot start it, under an exact one-line furthest-position emulation emitted only where the counterfactual is decidable (bare-reference elements; terminal-only sites need none; mixed frontiers stay unguarded rather than guess) | codegen (parser-agnostic; census-priced) | ≈19.0 µs → **≈18.2 µs** | **−4.3%** (all 5 rounds; 285 of 999 residual discards killed, `class_zero_width` 134 → 0; character_class −12.7%) | **landed** ✓ |
| RGX-0078 · 5.i.7 D2-A | **The observability twin & the fused cascade graph** — every acyclic, provably effect-free grammar region is emitted twice: the untouched protocol methods (memoization, counters, coverage, trace — everything the diagnostic surfaces read), and a compact fused `cascade_` function per region rule with the per-rule protocol frame elided; a parse with no diagnostic consumer runs the fused graph, and any consumer routes to the protocol graph automatically, so every observable pin stays byte-exact by construction (see the twin section below) | codegen (parser-agnostic; census-planned, all 11 parsers) | ≈17.8 µs → **≈15.4 µs** | **−13.6%** (all 5 rounds; every pattern faster; character_class −27.6% = the planned anchor; fused execution proven by live-stack profile) | **landed** ✓ |
| RGX-0078 · 5.i.7 D2-B | **The cyclic-spine fold + the thin memo** — the fused graph extends through the grammar's recursive core (for regex: pattern/alternation/concatenation/piece/atom and the group families), so a bare parse descends the whole spine in fused code; cycle-participating rules keep exactly the protocol parts that are load-bearing on a cycle, in lean form: the recursion-guard check (infinite/left-recursion detection is exact only if every cyclic rule participates) and a thin memo whose entries carry the protocol memo's own taint classes — a pure entry (no store read, no store write) replays at any store state, a store-reading entry replays only while the store is provably unchanged (the write-epoch license), and a store-mutating body is never cached, re-executing honestly instead (a value-only replay would skip effects the protocol re-applies from its stored delta) | codegen + one engine type (parser-agnostic; census-planned at the cyclic-spine increment, all 11 parsers) | ≈15.8 µs → **≈14.0 µs** | **−11.4%** (all 5 rounds; every pattern faster; the first cut validated entries against a global store-unchanged check and *regressed* the two fact-writing patterns +3/+15% — the per-entry taint classes are what fixed it, proven by a controlled A/B) | **landed** ✓ |
| RGX-0078 · 5.i.7 mimalloc | **A mimalloc-class global allocator** — the parse builds and drops many short-lived value nodes, so the platform allocator (`malloc`/`free`) is on the hot path. A measurement-only ceiling probe (a never-free bump `#[global_allocator]` — every `malloc` a pointer bump, every `free` a no-op) priced killing that cost at **−40.0%** of the min-metric time; a mimalloc swap captures 73% of it. The **library** deliberately sets no global allocator (that stays the embedder's choice), so this is banked as the *recommended production configuration* (documented in the regex integration contract) and the campaign's baseline — not a lib code change. Correctness-neutral by construction (an allocator cannot change program semantics for a program with no allocation-address dependence — the 4.a build-profile precedent) | allocator / build config (embedder-selected; recommended) | ≈13.8 µs → **≈9.77 µs** | **−29.2%** (fat-LTO alternated 5×2000 geomean-of-mins, deterministic, every pattern faster; the never-free-arena ceiling that priced it was **−40.0%**) | **landed** ✓ (recommended) |
| RGX-0078 · 5.i.7 P-env | **Hoist the per-parse `getenv` to a process-once cache** — every generated `parse()` read `std::env::var("PGEN_REPORT_MEMO_STATS")` *twice per parse* (the observability-twin fused-vs-protocol routing compute + the post-parse memo-report gate), and each call is a `getenv`/`__findenv_locked` locked linear scan of the whole `environ`. A `sample` profile of the malloc-free build (RE-PROFILE #12) priced it at ~3% of the parse; because the read is codegen-emitted, *every* generated parser paid it *every* parse. The flag is a process-launch diagnostic switch (nothing sets it mid-process), so it is now read once into a process `OnceLock<bool>` (`report_memo_stats_enabled()`) and the per-parse syscall is gone. Correctness-neutral by construction — the fused-vs-protocol routing decision is a process-level constant, so every parse output is byte-identical (proven: the differential-equivalence gate stays green over all 11 regenerated parsers) | engine helper + codegen (parser-agnostic) | ≈9.77 µs → **≈9.54 µs** | **−2.3%** (fat-LTO alternated 5×2000 geomean-of-mins, every round *and* every pattern faster; `literal_simple` −5.4% — the cheapest pattern, where the fixed syscall is the largest share) | **landed** ✓ |
| RGX-0078 · 5.i.7 MTB-B | **Match-then-build: the derivation-tape fold** — no fused rule constructs a value during matching anymore. The match pass runs the same control flow (guards, speculation, tournament islands) but appends POD *events* to a derivation tape (which branch won, how many iterations committed, where a dynamic terminal ended); failed speculation truncates its tape segment, and only the committed derivation gets one value-construction walk at the end — so the doomed values that dominated allocation traffic (≈91% of in-metric alloc events, ≈96% of bytes) are truncated un-built. The cyclic spine joins the fold with memo protection intact: the cycle-participant thin memo's payload becomes the derivation segment itself, spliced back onto the live tape on a hit. Byte-identical by the full oracle battery — including the differential-equivalence gate, which caught (and forced the fix of) a subtle build-side `$text` slicing defect before landing | codegen (parser-agnostic; the fused bare-parse graph) | ≈9.54 µs → **≈8.90 µs** | **−8.8%** (fat-LTO alternated 5×2000 geomean-of-mins, mimalloc, every round and every pattern faster; the directional ceiling said −45…−53% — the honest record is that sampled-share models keep over-pricing, and the land gate is the measurement) | **landed** ✓ |
| RGX-0078 · 5.i.7 REPRESENTATION | **The committed-value representation: serde `Value`/`BTreeMap` → the arena-`Copy` `PgenValue`** — the values a parse *commits* (return-annotation objects, arrays, property accesses, typed literals) were eagerly-built `serde_json::Value` trees: a `BTreeMap` node per object, a cloned `String` per template key, and a DOUBLE deep-clone per nested reference (`content.clone()` then `.to_json_value()` cloning the subtree again — 912 such sites in the regex artifact; RE-PROFILE #13 priced the machinery at CONSTRUCT ≈23.3% + TEARDOWN ≈13.3% of the sampled window). Now every committed value is a `PgenValue` — a `Copy` enum over arena slices owned by the same `NodeArena` as the parse nodes: putting a child under a key copies machine words; template object keys are sorted at *codegen* time so an emitted object is ONE arena bump of a stack-built array (no runtime map, no key allocs, no binary search); property access is a sorted-slice binary search (the byte-exact `Map::get` equivalent); teardown is the arena's mass free. The wire format is untouched: `Shaped` serializes under the same `"Json"` tag through a `Serialize` mirror of `Value`'s formatter, proven byte-identical by a dedicated oracle suite, the all-11 differential-equivalence gate (run BEFORE the land decision), a 16/16 dump tripwire, and the typed-hook differential gate | engine value type + codegen (parser-agnostic; all 11 parsers regenerated) | ≈8.90 µs → **≈6.11 µs** | **−32.0%** (fat-LTO alternated 5×2000 geomean-of-mins, mimalloc, every round 0.673–0.681 and every pattern 0.598–0.749 faster; the sampled-share model said ≈25–30% — the first slight *under*-price of the class) | **landed** ✓ |

**Lever RGX-0078·4.a in plain terms.** The release build was using cargo's *defaults* — link-time
optimization off, and the crate split into sixteen independently-optimized units. That fragments the
generated parser and the shared engine (which span many modules) into pieces the optimizer never
inlines across. Turning on **fat LTO** and collapsing to a **single codegen unit** lets the whole
program optimize together — a classic win on branchy, compute-heavy native code. It is the cheapest
possible lever: *zero* source or grammar change, it applies to every release binary and every grammar
(parser-agnostic), and it is reproducible on any machine (unlike the machine-specific `target-cpu` and
PGO levers that follow it). Measured decisive delta: **−6.7%** geomean at the noise floor, every one
of the eight patterns improved, proven by a drift-controlled back-to-back (both binaries built, then
measured alternately so shared CPU load cancels out). A build-profile setting cannot change program
semantics, so the correctness floor is even more robust here than for an engine change — and it was
still proven: the fat-LTO release binary returns **byte-identical** PCRE2-compile-oracle verdicts to
the unchanged reference across all 2 188 corpus cells (`1878/310/262/48`, `diff` empty — not a single
new false-accept or false-reject), and certificate-coverage stays `fully_certified` (UNKNOWN=0).

**Lever RGX-0078·4.b (`target-cpu=native`) — measured and rejected.** The obvious next build flag is
`target-cpu=native`, which lets the compiler use the host CPU's full instruction set. It is *commonly
assumed* to be a free win — so it is worth stating plainly that, measured decisively (built on top of 4.a,
compared alternately against the fat-LTO-only binary), it made the regex parse **~4% slower**, uniformly
across seven of the eight patterns. The reason is the shape of the workload: PGEN's parser is branchy,
control-flow-bound, and runs on tiny inputs — there is nothing data-parallel to vectorize, so the
native-codegen SIMD setup only bloats the code and hurts instruction-cache locality. It is a good reminder
that a build flag's reputation is no substitute for a measurement: this one was rejected, and it is also
machine-specific (a `native` binary is not portable), so it is left off entirely — downstream consumers
should not assume `target-cpu` helps their workload without measuring it.

**Lever RGX-0078·4.c (PGO) — measured modest, not landed.** The last build-flag lever is profile-guided
optimization: build an instrumented binary, run it on a training workload to record which branches are hot,
then rebuild using that profile so the compiler lays out and inlines for the real execution pattern. It is
the closest AOT analog to what a JIT does. Measured on top of fat-LTO, it came in at about **−1%** (a
conservative lower bound — the instrument and optimized builds inlined differently, so the profile only
partially applied). It is a small, real win, but it is *not landed*, for two reasons that hold regardless of
the exact number. First, its ceiling is bounded: the profile shows the dominant cost is allocation (59%),
and PGO improves code layout, not allocation volume — so it structurally cannot be a large win here. Second,
unlike a drop-in flag, PGO is a build *process* — it would require baking an instrument→train→rebuild
pipeline (and a representative, reproducible training corpus) into how the parser ships, a real complexity
cost that a low-single-digit gain does not justify when the engine levers below attack the actual bottleneck
directly.

**Lever RGX-0078·1 in plain terms.** Every failed speculation calls the semantic runtime's
*rollback*, which restored scope bookkeeping by cloning a vector and rebuilding another —
*unconditionally*. But the overwhelmingly common backtrack changes no scope at all, and the
restored state is provably already correct in that case (the bookkeeping is kept in lockstep
as scopes open and close). Guarding the restoration on "did scope state actually change?"
removes that per-backtrack work on the hot path. It is a pure engine change (no grammar, no
regeneration), it benefits every parser and the interpreter identically, and it changes no
observable output — a correctness-neutral skip. Measured decisive delta: **−18.6%** at the
noise floor across all eight patterns.

That "changes no observable output" claim is not asserted — it is *proven*, gate by gate, at
the change's pre-optimization value. The PCRE2 compile-oracle over the real corpus returns
**byte-identical** verdicts with and without the change (a decisive fix-vs-no-fix diff:
`1878/310/262/48` either way — not a single new false-accept or false-reject); the
certificate-coverage generator↔parser duality is unchanged (identical sample-parse-failure
counts with and without); the duality-hunt gate finds no new break signature; the interpreter
and generated parser stay byte-identical to each other; and the generated-AST shape contract
still holds against the running parser. Correctness is the floor here, never traded for the
latency — so a speed lever only lands once every one of those oracles is re-proven green.

**Lever RGX-0078·5.a in plain terms.** The parser keeps its semantic-annotation directives in a
table keyed by rule name, and it consults that table on *every* rule entry — "does this rule have a
pre-predicate? a post-predicate that needs the raw text? any branch directives?" — and on every
backtrack, of which the per-character alternative tournament generates a great many. Those tables were
std hash maps, which default to **SipHash**: a cryptographically strong, deliberately slow hash chosen
to resist denial-of-service attacks on hash tables exposed to untrusted keys. But these keys are the
grammar's own fixed rule names — there is no adversary — so the DoS resistance buys nothing and the
hashing cost is pure overhead (the profile put it at ~6% of parse self-time). Switching the two
per-rule-entry tables to **FxHash** (`rustc_hash`, the same fast non-cryptographic hash already used
for the packrat memo) removes it. The public constructors still take a std map and convert once at
construction, so the *generated* parser is untouched — no regeneration. And because those two tables
are only ever read by point lookup (never iterated in a way that reaches the output), swapping the
hash function cannot change what the parser produces: it is byte-identical *by construction*, and then
proven so by the same full oracle battery — PCRE2 compile-oracle `diff`-empty (`1878/310/262/48`),
certificate-coverage `fully_certified`, interpreter↔parser equivalence, the semantic suite, the
duality-hunt gate, and the AST-shape contract all green at their pre-optimization values. Measured
decisive delta: **−5.4%** geomean of the noise-floor minimums (drift-controlled, both binaries built
and measured alternately), every one of the eight patterns faster in every round.

**Lever RGX-0078·5.c (first-set predictive dispatch) — the deepest lever, and the biggest single
win.** The bulk of regex parse time is one rule: `atom`, a ~24-way branch tournament tried at every
input character. Under longest-match semantics the engine evaluates *every* alternative and keeps the
longest — so at a plain `a` it still sets up, tries, and rolls back the `.`-branch, the `[`-branch,
the `(`-branches, the `\`-branch, and twenty more, each paying a try + delta-extract + a rollback
label `format!` + the per-attempt allocation. That is the profiled 59%-malloc / 6%-string cost seen
twice: *"evaluate every branch, then discard most."* The fix is classic predictive parsing (LL(1)
director sets, re2c/flex first-char dispatch): compute, at codegen time, a sound **over-approximation
of the first bytes** that can begin each branch, and emit a one-byte guard — if the next input byte is
not in a non-nullable branch's first-set, skip that branch's whole body. At `a`, twenty-three of the
twenty-four `atom` branches are pruned; only the branches that could actually match are tried.

It is a **prune, never a commit**: a skipped branch is one that would have failed at character 1, so
it contributes nothing to the tournament — the longest-match winner is *identical*. Soundness rests
entirely on the first-set being a superset (when uncertain — a regex token, an unresolved reference, a
nullable branch, a `.`/lookahead/anchor — the branch is *always* tried, never pruned). It is
parser-agnostic (a codegen primitive, gated only on the grammar's declared `@whitespace_sensitive`
policy so the raw next-byte peek is sound, and restricted to a rule's top-level tournament so pruning
is `furthest_position`-neutral), so every whitespace-sensitive grammar inherits it. Measured decisive
delta: **−75%** geomean (≈303 µs → 76 µs, a ~4× speedup), stable across three alternated rounds, with
the parse output proven **byte-identical** — the interpreter↔generated differential-equivalence gate
(verdict + AST + `furthest_position`, every grammar, seeds 0/7/42), the 27-combinator suite (including
the `a|ab` longest-match discrimination), the 36-construct semantic suite, certificate coverage
(`UNKNOWN=0`, unchanged), and the byte-identical regen of all nine non-regex parsers all green at their
pre-optimization values. This is the sound *first mile* of a fuller lockstep-simulation road
(Thompson-NFA / RE2 / GLL — advance all live branches together, no backtracking, no per-branch
allocation); it ships the big regex win now while that engine work is scoped.

### What is left, and the honest gap

After the −75% first-set lever landed, the profile was *re-taken* — because pruning 23-of-24
branches per character changes the shape, and a stale profile would steer the next lever wrong.
The re-profile is decisive: **allocation is still the dominant cost.** malloc holds at **~56%**
(barely moved from 59% — the win removed losing-branch work *proportionally*), and together with
the parse-structure *clone* (~7%) and *drop* (~8%) it is roughly **70% of self-time spent building,
cloning and freeing parse structures** (`ParseNode`, `ParseContent`, the JSON value, the small
maps). The per-character matching work of the *surviving* branches is only ~14%; the string
formatting that was 6% has collapsed to ~2% (its losing-branch `format!`s are no longer evaluated,
exactly as predicted). The measurement points straight at the planned destination — so the ranked
work ahead is:

1. **The lockstep-simulation automaton — the agreed next direction, and where parity is reached.**
   Advance all live branches together, dead ones drop in place, the survivor moves the cursor
   forward: no backtracking, and the winner's AST built *once* and shared — so the ~70%
   allocate/clone/free is *eliminated at the root*, not merely pooled. This is exactly how the
   hand-tuned C engines this campaign races (re2c, Hyperscan) work, which is why it is the road whose
   ceiling reaches PCRE2 parity. It is built by composing three pieces the engine *already holds*:
   the packrat memo as a shared parse forest (build-once, share by the `(rule, start, end)` key), a
   copy-on-write overlay for the semantic store (per-branch deltas, not per-branch clones), and a
   single arena freed en masse. The tractable-now first step is the **lexical DFA** for the regular
   sub-language (the `atom` choice, keyword sets, any terminal choice), which captures the bulk of
   the regex win; the fully general recursive form (a GLL graph-structured stack) follows as a
   research-grade engine effort. Each step starts with a design spike and lands only once the
   `parse_harness_*` differential proves it byte-identical.
2. **Nested-tournament first-set pruning.** Extends the landed top-level prune (lever 5.c) into
   *nested* alternations. Deferred deliberately: it attacks only the ~14% matching bucket, and needs
   the `furthest_position` bookkeeping worked out first — a smaller, later increment than the
   lockstep road.
3. **The per-rule annotation-table lookups (remaining half).** The faster-hasher half is landed
   (lever 5.a, FxHash instead of SipHash); *generating the lookup away entirely* for annotation-free
   rules is still open as a codegen change — small (a few percent), a cheap parallel slice.

The standalone "arena / bump allocation" idea from the technique menu is **not** a separate lever
here: on its own it would only make each allocation cheaper while leaving the backtracking and the
build-then-clone pattern in place — a bounded win. It is instead one of the three composing pieces of
the lockstep road above, which is why the allocation is *eliminated* rather than pooled.

### An elegant tool that didn't pay off yet — and why (lever 5.d.2, the *substitutes* lesson)

Lever **5.d.2 (lazy winner-only materialization)** is worth recording *because* it was reverted — it
is a clean example of a discipline this campaign keeps. The idea is elegant: in a branch tournament,
the naive code builds each successful branch's return value (a JSON fold + a clone) and then throws
all but the winner's away — *"fold twenty-four, keep one."* But the winner is chosen from cheap
metadata alone (how far it matched, its priority), never from the built value — so you can **decide
the winner first and build only the survivor's value**, once. It is the *parallel-not-sequential*
principle in miniature: explore the candidates in lockstep, defer each one's expensive work, and
materialize once for the one that wins. It was implemented, and proven **100% byte-identical** (the
interpreter-vs-generated differential across all eleven grammars, the 27 structural combinators
including the `a|ab` longest-match tie-break, the 36 semantic constructs, and a PCRE2 per-case diff
that came back *empty* over 2 188 corpus cells).

And it made **no measurable difference** — a decisive seven-round, drift-controlled measurement put
it at ~75.5 µs against the ~75.9 µs baseline: heavy overlap, no separation. The reason is the useful
lesson: **first-set dispatch (lever 5.c) and lazy materialization are two different solutions to the
*same* observable problem — the cost of the losing branches — so they are *substitutes*, not
complements.** First-set removes a losing branch *before* it is ever tried; lazy removes its build
work *after* it is tried. Since the −75% first-set lever already deletes those branches up front,
lazy materialization arrives to find almost nothing left to defer. The durable rule: *before landing
a speed lever, ask whether an already-landed lever attacks the same cost — and measure the marginal
gain against the current baseline, not the original one.* The decisive before-vs-after measurement is
exactly what caught a lever that "obviously should help" delivering nothing.

The tool is **kept, not thrown away.** Its worked design and the full lesson live in the decision
record *lazy winner-only materialization* and the `RGX-0078.5.d.2` task leaf, and the tool is in fact
*structurally required* by the lockstep DFA advance above (item 1) — a lockstep automaton **cannot**
build per-branch values as it advances all branches together, so it *must* materialize the winner
only. Lazy materialization returns there, where it composes with the DFA advance into a real win
rather than standing alone as a substitute for a lever already landed.

### The same discipline, applied one step earlier (lever 5.d.3, the memo-clone premise)

The very next queued lever — **5.d.3 (memo subtree sharing)** — was to replace the packrat memo's
deep clone (it copies a rule's whole parse subtree both when it caches a result and when it replays
one) with a cheap shared handle, on the theory that *those* clones were the "~70% allocate/clone/free"
the re-profile saw. That theory was an **inference**, and the lesson from 5.d.2 is precisely *don't
infer — measure the marginal gain against the current baseline first.* So before building the change
(which is genuinely invasive — it entangles a shared-ownership handle with the parser's input
lifetime), the memo-clone cost was profiled *specifically*.

It does not hold up. A memo-footprint report shows the cache is **tiny and flat** — the busiest
pattern caches under four hundred subtree-nodes across the whole parse, averaging under two nodes per
entry, and most of those are string-slice leaves whose "clone" is a pointer copy that allocates
nothing. And the sampled self-time puts the two clone operations the lever targets at **1.27% of the
total** — the same speed-neutral territory 5.d.2 turned out to occupy. The biggest *clone* in the
profile is not the memo at all; it is the winner's JSON return value (a `serde_json` object, which is
a small map, cloned and dropped) — which this lever does not touch. The real ~55% is broad
*construction* allocation spread across every parse structure, and the only thing that removes it is
the arena-plus-lockstep advance (item 1 above), not sharing the memo's node.

So 5.d.3, like 5.d.2, is a **composing piece of the lockstep road, not a standalone win** — and this
time the discipline caught it *before* a day of invasive lifetime surgery rather than after. The
profile did surface two genuinely separable levers to weigh, though: the arena/lockstep advance
itself (the only lever that moves the 55%), and a smaller, byte-identical one outside that road —
**precompiling the predicate expression.** A semantic predicate like `a == b || c != d` is today
re-parsed from its source string on *every* evaluation (split on `||`, then `&&`, then each
comparison operator, recursively); parsing it once into a small tree and evaluating that tree instead
would reclaim the ~3% the string-splitting costs, with the same result byte-for-byte. Which of these
comes next was a sequencing decision for the director, since it touches the agreed lockstep plan — and
the call was to **advance to the lockstep road itself**, the only lever that moves the dominant ~55%,
folding the memo-sharing and the lazy-materialization tool in as the pieces they turned out to be.

The design of that lockstep advance then cleared its first and hardest correctness hurdle. The worry
with advancing all branches together and dropping the dead ones in place is that the parser's
"furthest position" — the deepest byte any branch reached, which is what turns a parse failure into a
precise error locus rather than a shrug — might come out different from the honest serial version that
tries every branch to its end. It does not: that value is written in exactly one place in the whole
generated parser (the moment a rule is entered), it only ever moves forward, and it is never rewound.
So a terminal branch can only ever push it to the choice's own start, which is already recorded, and a
grouped branch hands off to ordinary recursion the instant its opening marker is recognized — which
records the same positions it always did. The lockstep advance is therefore *furthest-neutral by
construction*, for the same reason the earlier first-set prune was, and the byte-identical differential
gate checks that position explicitly as a backstop.

The second and last design question was then worked out too: how a build-once-and-share arena coexists
with the borrow the parse tree already holds on the input. The key observation is that the borrowed
parse tree never leaves the parser — every public entry point returns *owned* output (a JSON value, or
its serialized string), produced by walking the tree once at the boundary — so an arena is a purely
*internal* detail that no consumer can observe. That frees the design to pick the cheapest shape rather
than the one that appeases the public API. Three were weighed: reference-counted shared handles (cheap
to share, but still one heap allocation per node — so it does not touch the dominant *construction*
cost); a bump arena of self-referential borrows (moves the cost, but at the price of a second lifetime
threaded through the internals); and a single growable vector of nodes addressed by small integer
indices (moves the same cost with *no* second lifetime and no new dependency, paying instead a
mechanical "pass the node vector to the tree-walkers" refactor). The third is the recommended shape: it
allocates all nodes in one contiguous, cache-friendly block freed en masse, lets the memo share a
subtree by handing back an index instead of deep-copying it, and keeps the single input lifetime the
tree already has.

One more measurement gated the invasive surgery before it began — the same discipline that caught the
two earlier levers. The concern: that "construction malloc" bucket is *all* parse-structure
allocation, and it includes the return-annotation's serde_json output — which an arena over *parse
nodes* does not touch (the parser builds JSON objects during the parse, at a couple of dozen sites,
and those live in the standard allocator regardless). So before a change that threads a node-index
through more than a dozen files, the allocation was profiled *by caller* to split the arena-movable
part from the irreducible part. The result is decisive: of all the allocation time, **~44% is parse-node
and vector construction (arena-movable), ~26% is serde_json output, ~20% is the semantic-runtime
backtrack path, and the rest is hashing and formatting.** Since allocation is ~55% of the total, the
arena's real target is about **a quarter of the whole parse** — comfortably the biggest single lever
left, so the surgery is justified. It also draws the map for *after* the arena: the serde_json output
(a quarter of allocation) and the semantic-runtime path (a fifth) are the next two levers, because the
arena, by design, leaves them untouched. The arena will not reach PCRE2 parity by itself — but it is
the biggest single lever left, and now a measured one. It landed as a **standalone** step (the
reference-arena migration recorded below), decoupled from the lockstep advance it was designed to
compose with — that advance is the step that follows.

A final pre-code scoping pass then settled the ownership shape — and, along the way, corrected the
first instinct. The tempting shape is an *index* arena (a single vector of nodes, children referenced
by small integers): it needs no second lifetime and no dependency, which is exactly why large,
long-lived syntax-tree codebases (rust-analyzer among them) favour it. But scoping surfaced a decider
that outranks that convenience. The parser's *authoritative* typed-AST — the exact bytes every
equivalence check compares — is produced by serialising the node tree **directly** through the
standard derive. An index arena would make that derive emit bare *integers* where the subtree used to
be (the serialiser has no way to reach the arena and resolve them), which would force the team to
re-implement the serialiser — the very oracle correctness is judged against — **by hand**, byte for
byte. That is the single most dangerous kind of change to attempt against a strict byte-identity floor.
A *reference* arena avoids it entirely: a borrowed child serialises identically to an owned one, so the
derive keeps producing the same bytes for free, and the most correctness-sensitive generated code (the
return-annotation transform, which moves child nodes between accumulators and builds fresh wrappers
mid-transform) barely changes because a borrowed child is still directly usable. Its price is a second
lifetime threaded through the internals — but that is *mechanical, compiler-checked* churn (a slip is a
build error, never a silent output difference) and it stays purely internal, since every public entry
point already returns owned output. The last constraint — the tree's leaves carry heap-owned values
(transformed strings, typed JSON), so the arena **must** run their destructors when it is freed — rules
out the naïve bump allocator (which never runs destructors and would leak on every parse) in favour of
a destructor-running reference arena. That is the shape chosen: a small, standard reference arena that
frees its whole block (and runs every leaf's destructor) at the boundary, lets the memo share a subtree
by handing back a cheap borrowed handle, and keeps the derived serialiser byte-identical by
construction. The whole change is proved byte-identical by the cheap differential gates *before* any
expensive optimized build, so a slip is caught early and cheaply.

### The arena landed — −21.9%, byte-identical

The reference arena is **in**. Every child `ParseNode` is now allocated in a per-parse
destructor-running arena and held as a borrowed `&'input` reference (`Sequence`, `Alternative`,
`Quantified` all carry references, not owned boxes/vectors); the arena is threaded through the
generated parser's constructor and freed in one shot at the parse boundary. The single feared risk —
a *viral second lifetime* — did not materialize: reborrowing collapses the arena's lifetime onto the
input's, so the tree keeps the **single** `'input` lifetime it always had. And the decisive design
bet paid off exactly as scoped: because a borrowed child serialises identically to an owned one, the
`#[derive(Serialize)]` that produces the authoritative typed-AST kept emitting the same bytes **for
free** — no hand-written serialiser, no oracle to re-prove by hand.

The measurement is decisive and drift-controlled — both binaries built at release fat-LTO with
distinct hashes, measured alternately five rounds of 2 000 parses so shared CPU load cancels out. The
arena parses the 8-pattern corpus at a **58.6 µs** geomean against the owned baseline's **75.0 µs** —
a **−21.9%** win, arena faster in *every* round (per-round ratios `0.79 / 0.78 / 0.78 / 0.78 / 0.78`,
clean separation with no overlap). That makes it the single biggest lever since first-set dispatch,
and it validates the by-caller profile above: moving just the parse-node construction allocation off
the general allocator reclaimed almost exactly the share the profile predicted.

Correctness held to the byte: the full ⛔ hard-constraint battery is green — the interpreter-vs-generated
equivalence gate byte-identical across every grammar and seed, combinator and semantic differential
gates `2/0`, the regex certificate `fully_certified` with `UNKNOWN=0` and an identical spurious-failure
profile, the PCRE2 compile-oracle verdicts unchanged, and the AST-shape contract `18/0`. The arena is a
*parser-agnostic* codegen-plus-engine primitive, so SystemVerilog, VHDL, JSON and every other grammar
inherit the same allocation win from the same change — the founding doctrine, once more: tune the
compiler, and every language it compiles gets faster.

### After the arena: the re-profile that found the construction floor

The discipline after every landed lever is the same: **re-profile on the new baseline before
choosing the next one** — each big win reshapes the cost landscape, and yesterday's "too small to
matter" can become today's dominant bucket. The post-arena re-profile confirmed the arena did
exactly what the by-caller profile predicted (allocation fell from ~55% of self-time to **under
18%**) — and then surfaced something genuinely new.

The single largest coherent cost is now **parser construction**: ~12% of the whole timed parse is
spent *before the first input byte is examined*, rebuilding tables that never change. The generated
constructor assembles the grammar's compiled annotation tables — scope directives, fact emissions,
predicate specs — into hash maps, allocating nearly a hundred small strings, hashing every key
*twice* (once into the standard library's map, then again while converting to the faster map the
engine actually uses), and then drops the whole structure at the end of the parse. All of it is
**grammar-constant**: every parse of every input builds and destroys the identical tables. At the
496 µs baseline this was ~0.7% and correctly dismissed; after an 8.5× speedup, the same fixed cost
is the top item on the board — a textbook illustration of why the re-profile step is not optional.

The indicated lever — agreed with the director as the next slice, ahead of the research-grade
lockstep/GLL rung — was a **construction cache**: build the compiled annotation tables *once per
process* and hand every parser instance a cheap shared handle. Same values, same lookups,
byte-identical by construction; just built once instead of on every parse. It landed the same
day: **−3.8%** (58.6 µs → 56.8 µs), faster in every measured round, with the full byte-identity
battery green — the differential-equivalence gate is a particularly pointed oracle here, since the
interpreter side still builds its tables fresh per parse while the generated side shares one, and
the outputs must (and do) match to the byte. The honest note: the win is smaller than the profile
bucket, because the constructor also does genuinely per-parse work (memo pre-sizing, runtime-state
init) that the cache correctly leaves alone. The remaining map: the distributed rule-call
machinery (~12%, the lockstep/GLL target), the semantic-runtime residue (~8%), and the
return-annotation JSON output (~6%).

### The measured distance — and why the next lever is architectural, not another bucket

With the profile now flat (no bucket above ~12%), the campaign measured, for the first time, the
actual distance to its destination. On the same machine and the same eight patterns, PCRE2 10.47
compiles each pattern in **0.34–0.86 µs** (`pcre2test -t`); PGEN parses them in 11.8–150.8 µs.
Geometric-mean ratio: **≈99×**, against the campaign's closure bound of **<5×**. Five landed levers
have already closed 8.76× (496 µs → 56.8 µs); reaching the bound needs roughly **20× more** — and
every named bucket left on the board sums to ~32%, worth at most ~1.5× even if it all vanished.
Bucket-shaving is over as a road to the destination.

Where the 20× actually lives is visible with one small tool run: parsing the four-character
pattern `test` makes **45 rule entries** (the `piece → atom → literal → literal_char → letter`
cascade plus the per-character quantifier/escape probes — about eleven entries per input
character), at ≈262 ns per entry. PCRE2 compiles the *entire pattern* in roughly the cost of
*one* PGEN rule entry. The dominant cost is the **rule-cascade-per-character execution model**
itself, and it is addressable only by executing *fewer rule entries*, not cheaper buckets:
either don't parse at all (a persistent pattern→AST **parse cache** — real regex workloads
recompile the same patterns constantly, so amortized cost collapses), or make the generator emit
*fused* code for the cascades and token-shaped sublanguages (collapse eleven entries per
character toward one or two — the direct-coded-scanner form that re2c and Ragel emit).

This measurement is also what retired the long-queued **GLL rung** (the scoreboard's `5.e`).
GLL generalizes recursive descent to arbitrary context-free grammars with a graph-structured
stack and a shared parse forest — machinery whose value is taming *nondeterminism*. But after
first-set dispatch and packrat memoization, this engine's measured backtracking residue is
~2.2%, and the parsing literature's own comparison is unambiguous: adaptive top-down prediction
(ANTLR's ALL(\*)) outperforms GLL and GLR *by orders of magnitude* on real, near-deterministic
grammars. Adopting GLL here would add descriptor and stack bookkeeping to every one of those
~262 ns rule entries in exchange for removing almost nothing — a predicted net *slowdown*,
adjudicated and refuted by a design spike for the cost of a document instead of a rewrite. That
is the land-iff-faster discipline working exactly as intended: the same gate that reverted a
byte-identical-but-neutral optimization also refuses a plausible-sounding research rewrite that
the numbers do not support.

### The gap is generator maturity, not "generated vs hand-tuned"

It is tempting to frame this as "a *generated* parser can never catch *hand-tuned* C." That
framing is wrong, and it is worth being precise about why, because it sets the ceiling for the
whole campaign.

**Code generation is a *superset* of hand-writing, not a weaker cousin.** Anything a
performance-obsessed engineer would write, a generator can emit — there is no expressiveness
barrier. And it can go *further*: no human will hand-specialize hundreds of parse rules into
individually inlined, unrolled, allocation-free functions and then *maintain* them — a
generator emits exactly that, uniformly, without tiring. This is not theory. The fastest
scanners and parsers in the wild are *generated* — Ragel, re2c, flex, protobuf/Cap'n Proto
codegen — and they routinely match or beat hand-written equivalents. **Generated is not slow;
generated-but-naive is slow.**

So the real gap is not "generated vs hand-tuned." It is **"the generator currently emits a
naive pattern vs the fast pattern it could emit."** Today codegen emits generic
recursive-descent packrat with per-backtrack allocation and try-every-alternative tournaments.
Teach the generator to emit the *fast* pattern — arena allocation, first-set dispatch,
specialized per-rule code, no per-speculation overhead — and the **output is the same native
code a hand-tuner would write**. That is a *maturity* gap, and maturity gaps close.

**A correction worth stating plainly: we are not racing a JIT.** PCRE2's JIT is its *matching*
engine. What this campaign races is PCRE2's **compile** step (RGX's compile time = our parse
time), and PCRE2's compiler is *just hand-written C* — no JIT on the other side of the parse-time
race. The only thing genuinely reserved for hand-code-plus-JIT is *runtime, data-dependent*
specialization to a specific pattern, and that is a *matching* concern — irrelevant to a
one-shot parse. So for the goal that actually matters here, **there is no intrinsic barrier.**

This reframes the campaign and — not by accident — aligns it with PGEN's founding doctrine:
**every speed lever should land as a codegen or shared-engine-primitive capability** (teach the
generator to emit the fast pattern), so every win is *parser-agnostic* — SystemVerilog, VHDL,
JSON and the RTL front-ends all get faster from the same change. That is strictly better than
hand-tuning one parser: we tune the *compiler*, and every language it compiles inherits the win.

Honest about effort, not about ceilings: reaching hand-tuned quality is real work — arena
allocation, predictive dispatch and per-rule specialization are non-trivial codegen changes,
landed one measured step at a time. But the ceiling is *hand-tuned-C parse speed*, not "5× is
the best a generator can do." The scoreboard exists so the distance to that ceiling is always
visible, and so every step toward it is a *measured* step, not a hopeful one.

### The optimizing-compiler rung: where the 236 ns per rule entry actually goes

After the derived-scanner rung's speed claim was refuted by its own step-0 measurement (the
fusibility census showed the killable share of rule entries caps the win at ~1.5–1.8× — far from
the estimated 3–5×), the campaign's road converged on its real destination: PGEN today has a
full-strength *spec* half (the EBNF and annotations — the sole source of truth) and a
full-strength *proof* half (byte-identity oracles, certificate seeds, the interpreter as
reference), but its *compiler* half is a **transliterator** — it maps every construct 1:1 onto
maximally-defensive generic machinery (every rule a guarded, memoized method; every `|` a full
tournament; every `?`/`*` a snapshot-speculation) regardless of what the grammar needs at that
site. The agreed next rung is an **optimizing middle-end** ("the planner") between the grammar's
AST and code emission — mixed-mode and capability-gated per region, deriving every speed decision
from grammar analysis so grammar authors never think about speed.

Its step 0 decomposed the measured ~236 ns per rule entry into named buckets — not by guessing,
but by building six *measurement-only* variants of the generated parser, each with exactly one
piece of machinery stripped, each proven byte-identical on the bench before its time was trusted
(none of them landed; they exist to price the machinery). The result was a genuine surprise:

**About a third of the entire parse is unconditional bookkeeping waste.** The single biggest
bucket (−14.7% when stripped) is *naming strings for a trace that is off*: every failed
speculation heap-allocates two strings ("which rule failed, for the rollback trace"), and every
*successful* tournament branch allocates a formatted cleanup label — payloads that are only ever
read when high-verbosity tracing is enabled. Next come the tournament's per-choice bookkeeping
allocations (an evaluation-order `Vec` built just to iterate `0..n`, plus a partition-group
string built even when partitioning is disabled — −6.5%), and a per-entry rule-context string
pushed for error context (−4.3%). Stripping those plus the observability counters (−3.4%) and
the recursion guard (−3.8%) all at once measures **−31.7%** — more than the five individually,
because relieving allocator pressure compounds. The remaining two-thirds decompose into memo
machinery (~17–22%: four hash probes per entry plus per-success node/delta capture), the
return-annotation JSON output (~5–6%), annotation-table probes (~4–6%), parse-time re-parsing of
constraint expressions (~2–4% — the annotation *table* is cached since lever 5.g, but expression
*payloads* are still interpreted from source text on every evaluation), node construction, and
finally the structural parse work itself.

That measurement fixes the planner's build order. A new pass zero — **lazy, allocation-free
protocol hygiene** — comes before all the analysis-driven passes: build the trace-naming payloads
*only when trace consumes them*, iterate branch orders without materializing them. It needs no
grammar analysis at all, preserves every observability feature, applies to every parser, and its
measured ceiling is ≈−25% of the whole parse.

**Pass zero has now landed (scoreboard lever 5.i.2): measured −25.8%, ≈56.7 µs → ≈42.0 µs —
the census's prediction confirmed almost exactly.** The rollback-owner labels now travel as a
cheap copyable value (a couple of pointers and integers) that is turned into text only inside the
trace-enabled branch; the tournament iterates its rotated branch order arithmetically instead of
building a `Vec` per choice execution (and computes the partition-group string only when
partitioning is actually enabled); and the rule-context stack borrows the rule-name literals the
generated parser already carries (`Cow<'static, str>`) instead of copying them on every entry.
Nothing observable changed: the typed ASTs are byte-identical, and with tracing *enabled* the
rollback/backtrack trace lines are byte-for-byte the same as before (proven by diffing 923
normalized trace-payload lines across the old and new builds) — the strings are simply no longer
built for the overwhelmingly common case where nobody reads them. Every parser inherits the win.

**Predictive dispatch — the first analysis-gated pass — has now landed too (scoreboard lever
5.i.3): measured −5.3%, ≈42.0 µs → ≈39.7 µs.** Its step 0 extended the census with a
*degeneracy* classification: for every rule-top-level choice site, can the compiler PROVE the
longest-match tournament degenerate — every branch's admissible first bytes decided by the same
sound FIRST-set analysis the prune guards trust, the byte sets pairwise *disjoint* (so at most
one branch can begin a match at any next byte), terminals whitespace-sensitive (so peeking the
raw byte is sound), and no branch predicates or branch-start effects (which would need the
tournament's rollback-and-continue machinery)? Where the proof holds — 41 of regex's 112
top-level sites, the single-character alternation leaves like `letter`'s 52-way tournament — the
generated rule is now a single `match` on the next input byte that jumps straight to the only
branch that could match. The tournament *protocol* vanishes at those sites: no per-branch
guard-scan loop, no tournament checkpoint, no delta-extract/rollback/replay round-trip for a
winner that provably has no competitors (the candidate still runs under the speculation wrapper,
so a failed parse restores state exactly as before). An honest modeling note the census forced:
a first-byte switch skips exactly the branches the landed prune guards already skip, so this
pass kills *protocol*, not rule entries — the census measured the exposure (12.8% of bench
entries, 36.9% of all tournament loop iterations) and predicted −3–7% before a line of emission
code was written; the measurement landed at −5.3%, with the typed ASTs, rule-entry counters, and
certification pins all byte-identical, and every non-regex parser regenerating byte-identically
(regex is today's only terminal-whitespace-sensitive grammar; the gate is a declared capability,
never a grammar name).

Next in the fixed order: cascade inlining, then selective machinery (emit memo/guard/snapshot
wrappers only where analysis says they can matter), and compile-time value folding (pre-compile
constraint expressions, fold `$text`-class shapes). Each lands under the same hard constraint as
every lever before it: measurably faster *and* byte-identical under the full oracle battery, or
it does not land.

**Cascade/wrapper inlining's step 0 — the inline census — has now been measured.** The census
gained a per-rule *inline-eligibility* verdict: a rule's call frame is provably collapsible into
its call sites when the rule sits on no reference cycle (so its recursion guard can never fire),
carries no semantic directive in any phase (so its frame's transactional role is trace-naming
only), is not the entry rule, and is not dialect-gated. On regex, **204 of 274 rules qualify,
and 70% of all bench rule entries (1 672 of 2 389) are entries into such collapsible wrapper
frames** — each paying the full guard/context/annotation-probe/memo-dispatch/call protocol to do
essentially nothing. The blockers on the other 70 rules are named per rule (38 sit on the
recursive grammar spine; the rest carry real directives, dialect gates, or transforms). The
outcome dump also gained a per-rule *memo-hit* counter (recorded only under the coverage opt-in,
so ordinary parsing pays nothing): 568 of the bench's entries are answered from the packrat
cache, 399 of them on eligible rules — the honest price tag for the variant of inlining that
also drops the memo at inlined frames, since each such hit would become a body re-execution.
(Direct counting also corrected an earlier derived figure: the cost census's "613 memo hits" was
an upper bound that silently included 45 recursion-machinery cycle-break entries on the two
group-bearing patterns; the true hit count is 568, reconciled per-pattern against the memo-stats
occupancy.) The recorded falsifiable ceilings, priced from the measured cost buckets before any
emission code: ≈−9–11% for the counter-preserving frame collapse, a further ≈−10–18% if the memo
is elided at inlined frames — and unlike predictive dispatch, this surface is *cross-grammar*
(SystemVerilog: 844 of 1 466 rules eligible; VHDL 186/216; every family has one), so a landed
emission is a platform-wide primitive. The emission itself is the next slice; the ceiling dies
or survives by measurement, like every number on the scoreboard.

**The memo-preserving emission (P1a) has now landed (scoreboard lever 5.i.4): measured ≈−4–7%
across three alternated benchmark sessions, ≈39.7 µs → ≈36 µs.** At a call site of a *decided*
rule, the generated code no longer calls the rule's method: the rule's body is emitted inline
under a new `inlined_frame_call` engine helper that performs, verbatim, exactly the per-frame
work that must survive — the rule-entry counter, the transactional coverage push, the
furthest-position update, the memoized dispatch (the packrat memo is fully preserved in this
increment), and the method-identical exit trace lines. Because the helper exists once per
generated parser and the inlined body is produced by the *same* body generator the rule method
uses, the collapse is identical-by-construction — and the identity was then verified by
measurement, not asserted: per-pattern outcome dumps (raw, committed, and memo-hit counts),
all eight typed-AST dumps, and the certification pins at three seeds are byte-identical between
the baseline and the inlined parser. What each collapsed frame stops paying: the recursion-guard
push/pop (the census only admits rules that can never re-enter themselves), the rule-context
push/pop and the `--trace-rules` scope probe (both proven to feed trace output only), the two
needs-raw annotation-table probes (statically folded to a constant for every directive-free
rule — a fold that legally rides along in every rule *method* too), and the call frame itself.
Two honest structural notes, recorded as documented deltas: inside an inlined frame, a rollback
trace label and the recursion-guard error stack name the nearest *enclosing* method frame (the
wrapper no longer appears — trace-payload only), and `--trace-rules <wrapper>` no longer
activates scoped tracing at inlined sites of that wrapper. Code size is governed by a measured
budget shared with the census's `INLINE-DECISIONS` report (capped-transitive body weight ≤ 12
gen-AST nodes and weight × reference-sites ≤ 192): 128 of regex's 204 eligible rules land under
it (the emitted source grows ×1.86), the 76 over-budget rules are logged by name, and the budget
itself was chosen by measurement — a tighter variant (105 rules, ×1.28) was built, benchmarked
head-to-head in the same alternated session, and *lost* (−3.8% vs −6.7%), so the wider budget
stayed. The memo-eliding variant (P1b), whose counters change truthfully where the 399 cached
hits would re-execute, was landed as its own separately-priced increment — next paragraph.

**The memo-eliding increment (P1b) has now landed too: measured ≈−7–12% across five alternated
rounds, ≈36 µs → ≈32 µs — cumulatively 496 µs → ≈32 µs, ≈15.5× since the campaign opened.**
First the price was re-measured at the *landed* budget rather than the eligibility ceiling: the
census exposure join gained the budget-decided subset (`INLINE-EXPOSURE-DECIDED`), which showed
the emission plan covers 1 179 of the bench's 2 389 rule entries (49.4%) and that 324 of the 568
packrat-cache hits sit on inlined frames — the honest lost-hit population, recorded with a
falsifiable ≈−6–18% ceiling *before* any emission code. The change itself is one hunk in the
emitted helper: the inlined frame runs the rule body directly instead of through the memoized
dispatch, shedding the three-way cache probe on every inlined entry and the success-path insert
(a node clone plus semantic-delta and coverage extraction) on every inlined success. Dropping a
cache can never change a correct parse — the packrat memo is a pure, taint-gated cache whose own
soundness contract guarantees replay and re-execution agree — and the identity was verified by
measurement anyway: all eight typed-AST dumps and the certification pins at three seeds are
byte-identical, and the per-rule *committed* counts are unchanged on every pattern (the coverage
system already counted replayed children under P1a, so live re-execution produces the identical
committed record). What changes, truthfully: raw entry counters grow where former hits now
re-execute (+114 on the bench, concentrated under the hit-heavy character-class leaves), the
cache-hit counter at inlined frames drops to zero by construction, and re-executed bodies now
probe their children's still-memoized methods. The measured result — every round faster, every
pattern faster at best-mins — confirmed the census's leaf-dominated hit population re-executes
cheaper than the replay clone it replaced. A post-landing re-run of the degeneracy census also
answered the standing question of whether collapsing wrappers would open new predictive-dispatch
surface: it does not (the 41 qualified sites and their committed exposure are unchanged).

One incidental find from the same session is worth recording for transparency: the census's
byte-identity oracle caught a *regeneration-path* divergence — parsers regenerated through a
bootstrap-mode shortcut had silently degraded annotation `null` literals to the *string*
`"null"` (the bootstrap annotation parser predates the `null` literal). The affected parsers
were regenerated through the canonical path, the equivalence gate — whose interpreter side always
uses the canonical annotation pipeline — is exactly the tripwire for this class, and a loud-refusal
fix for the bootstrap parser is queued. The incident is a small, useful proof of why every
measurement here insists on the byte-identity check first.

### The re-profile that killed a pass before it was written

With P0, P2, and both P1 increments landed, the planner's next pass on paper was **P3 —
selective machinery at method frames**: stop paying the recursion guard at rules that provably
cannot re-enter themselves, and stop paying the packrat cache at rules where a census shows the
cache no longer earns its keep. A census over the existing instruments had priced those two
increments at roughly −2% and −4–8% — but every number in that estimate was a *per-event cost
model* (nanoseconds per probe, per insert, per guard push) carried forward from microbenchmarks
taken when the benchmark was almost twice as slow. So before any code, the discipline that opened
the campaign was repeated: profile the real binary and let the wall clock adjudicate.

The setup made drift impossible to hide: the freshly rebuilt release probe reproduced
**byte-identical** (same SHA-256) to the binary that had measured the P1b landing, and its sanity
rounds reproduced the pinned ≈32 µs geomean. Two 30-second sampling windows — the benchmark runs
its eight patterns sequentially, so a single window cannot see them all — covered the full corpus
between them, agreed on every headline bucket, and put 99.5% of samples inside the timed parse.

The profile refuted both paper increments outright. The *entire* memoization machinery — every
cache probe, every insert, the hit-replay path, and all its allocation traffic, on every rule
including the recursive spine — now costs 2.8–3.2% of the parse. The recursion guard costs
0.4–0.5%. The addressable slices of those numbers sit well below the benchmark's own ±1–3% round
noise, so both increments were closed *refuted by measurement*, with no emission built and
nothing to revert. This is the planner's scout protocol doing precisely what it exists to do: a
pass priced on a stale cost model dies at the profile, not after a week of emission work.

What the profile found instead is where the next real levers live. Half of all samples are still
allocator traffic — but no longer parse-node traffic (the arena already moved that): it is
**JSON value traffic**. Building, cloning, and dropping the `serde_json::Value` trees that
return annotations shape costs ≈26–29% of the parse, and the single largest site is the output
boundary itself, which *re-clones* the winner's value tree rather than moving it; the
string-splitting re-parse of value-constraint expressions, priced at 2–4% by an earlier census,
is now a named 1.9% leaf symbol of its own. That is the **P4 — value folding** surface, and it
is now the dominant addressable bucket. The second find is the **C3-B tournament delta
protocol** at ≈8–10%: every multi-branch choice that succeeds pays a store-delta
extract → rollback → re-apply round-trip so branches compete from identical semantic state — paid
even when the store is untouched, which on this benchmark is six of the eight patterns (the
protocol's rollback routine is the profile's single hottest named symbol). That is a real
*selective-machinery* target — the P3 idea, aimed at the piece of machinery the profile says
actually costs something — and it can be gated two ways, both to be priced before any emission:
statically (elide the protocol where analysis proves the subtree cannot touch the store) or
dynamically (an O(1) "store unchanged since checkpoint" fast path, the same write-epoch idea the
memo already uses for its soundness validation). The scoreboard's next rows are those two
pricings — each of which will land, or die, by measurement, like everything else in this
chapter.

The first pricing is already in: a four-counter classification added to the store's own
operation counters (recorded inside the rollback routine from values it already computes, and
reported through the same outcome-dump JSON the earlier censuses used) measured, on the
benchmark, that **98.8% of all speculation rollbacks and 93.6% of all tournament delta
extractions happen with a provably unchanged store** — every one of those extractions paid two
unconditional vector clones and a delta drop to describe *nothing* — and that every checkpoint
clones a non-empty scope chain, because the root scope is always open. The instrument's totals
reproduce the previous landing's counter pins byte-exactly, and a unit test pins the one case an
epoch check alone would miss (a deferred `phase: final` obligation deliberately does not bump
the write epoch, so the fast path must also compare the obligation count — both checks are
O(1)). The priced increment, recorded before any code: an engine-only fast path with a
falsifiable ceiling of ≈−5–8%, the checkpoint-side clone explicitly logged as the margin a later
static elision could still claim.

**The fast path has now landed (scoreboard lever 5.i.5): measured −11.2%, ≈35 µs-era → ≈31 µs —
cumulatively 496 µs → ≈31 µs, ≈16× since the campaign opened.** The change is confined to three
places in the engine's semantic-runtime module — the checkpoint records the epoch, and the two
protocol entry points check it — so every generated parser and the interpreter inherited it with
no regeneration. It measured *above* its own ceiling, and the record explains why honestly: the
pricing had counted only the tournament's delta extractions, but the packrat cache's
success-insert path extracts a delta for every memoized entry too, and that second — uncounted —
population rides the same fast path; the rest is the familiar compounding of relieved allocator
pressure. Every proof obligation was discharged before the number was trusted: the full library
suite, all eight per-pattern outcome dumps byte-identical between the fast and slow paths (with
the fast path's own state-equality assertions live in the debug build), the certification pins
at three seeds, the shape/duality/PCRE2-oracle gates, and — because the benchmark baseline was
deliberately the *pre-instrument* binary — the measurement also proved the scout's counters cost
nothing that the fast path did not pay back many times over.

### The re-profile after the harvest — and the claim that did not survive it

The discipline repeats after every landing: profile the new binary before pricing the next
pass. The fresh profile (two sampling windows over the exact binary that measured the −11.2%
landing — provenance by SHA-256 identity, so build drift is impossible rather than merely
controlled) confirmed the harvest precisely where the fast path predicted it: the
checkpoint/delta/rollback bucket fell from ≈10% to ≈4.4%, the delta-extraction routine from
≈4.5% to ≈1%, and the rollback routine lost its place as the profile's hottest named symbol.

The same pass also caught — and corrected — a defect in the *measurement method itself*. The
sampling report ends with flat summary sections after the call tree; the analysis script had
been parsing those as tree nodes, inflating its totals by half and mis-attributing about 15% of
allocation samples to the timing harness. Re-run with the call tree alone, the script reproduces
the previous profile's headline table *exactly* — so every bucket number and every steering
decision taken from it stands — but one subsidiary claim does not survive: the output-boundary
value conversion, previously described as "the largest single site" on the strength of a clone
count taken over the whole file, actually bounds at ≈4% of the parse. The correction matters
because it re-orders the value-folding pass's targets. What the clean profile names, in order:
**dropping `serde_json::Value` trees** (16–18% — the parser teardown cascading the memo's stored
values, the end-of-parse result teardown, and discarded speculation trees), **cloning value
trees during the parse** (≈11–14%, spread class-wide across the shaped-view capture closures
with no dominant site), and the **value-constraint expression re-parse** (6–8%, now the
profile's top named symbol — the cleanest single-mechanism increment, since the expressions are
grammar constants that can be compiled once). The residual store protocol prices at ≈4.4% and
is parked until value folding lands; the guard and the packrat cache remain refuted at under 1%
and ≈3%. The next scoreboard row will be the value-folding pricing scout — ceilings recorded
before code, as always.

That scout is now in, and its headline find is almost comic: the profile's top named symbol —
six to eight percent of the whole parse — turns out to be the parser *re-reading its own
documentation*. Grammar authors can attach a `@constraint:` annotation to a rule; its payload
may be a real relational expression, but every live use in the regex grammar is descriptive
prose like "produces control character". The generated rule exit cannot know that, so on every
successful parse of such a rule it feeds the constant string through the full expression
machine — two structural splits, up to six operator probes (each a fresh byte-walk and a vector
allocation), a reference probe, a number probe, and a lowercasing pass that allocates a string —
all to conclude, every time, that non-empty prose means "true". Measured: eighteen evaluations
per benchmark pass at roughly a microsecond each — a quarter of an entire `literal_simple`
parse, per evaluation, spent re-deriving a compile-time constant. The fix is the planner's
cleanest kind: the constraint string is a grammar constant, so the *generator* can run the same
classification once, at code-generation time, and simply emit nothing for a provably
constant-true check (it has no observable surface — no trace line, no store effect, an error
branch that can never fire). The pricing, recorded before any code: ≈−5–8% for that fold; the
deeper value-ownership work behind the remaining drop/clone churn (≈−9–13% for capture-clone
folding, ≈−4–6% for memo-value ownership, ≈−3% at the boundary) queues behind it, pending a
design spike. The fold is next on the scoreboard.

**The fold has now landed (scoreboard lever 5.i.6): measured −12.1%, ≈31 µs-era → ≈27.4 µs —
cumulatively 496 µs → ≈27.4 µs, over 18× since the campaign opened.** The change never touches
a grammar or the engine: the generator classifies each `@constraint` string once, under a gate
deliberately narrower than the runtime evaluator it mirrors, and simply declines to emit a
check that cannot fail. One subtlety earned its place in the gate during verification: the
runtime evaluator accepts a *bare* identifier (no `$` sigil) as a capture reference, so a
one-word "constraint" like `validated` is not documentation — it resolves against parse
content and can reject the rule; the gate therefore excludes anything shaped like an
identifier chain, and the truth-table test pins both directions. The proof obligations were
discharged the same way as every landing before it: the regenerated parser diff touched the
regex family alone, the full differential suite held the folded parser byte-identical to the
interpreter, the eight benchmark ASTs and outcome dumps reproduced bit-for-bit against the
previous landing's preserved binaries, and certification stayed pinned at all three seeds. The
per-pattern deltas read like the population census predicted: the escape-heavy patterns gained
26–31%, and the two patterns that never evaluate a constraint moved only within noise. What
remains of the value-folding surface — the drop churn and capture-clone class behind the
remaining ~30% — now waits on the next re-profile and the value-ownership design spike.

The post-landing re-profile is in, and it closes the loop cleanly: the folded mechanism
registers **zero** samples in both windows — not merely cheaper, *gone* — and what remains is
now a one-item list. Building, cloning, and dropping the JSON value trees that return
annotations shape is the only cost bucket left above five percent of the parse; the store
protocol's residue, the packrat cache, and the recursion guard all sit in low single digits,
exactly where the previous landings left them. The next step is therefore not another point
fix but a design question — who *owns* a shaped value as it crosses capture boundaries, lives
in the memo, and leaves through the output boundary — and it will be priced the way every
pass in this chapter was: a falsifiable ceiling against the fresh profile, recorded before any
code is written.

### The design spike: cutting one bucket by who-pays-when

The ownership question was answered not by choosing a clever value representation but by
re-cutting the existing profile along two axes nobody had separated yet: *which code path*
holds each value operation, and *whether the benchmark's stopwatch is still running when it
happens*. The second axis sounds pedantic and is anything but. The benchmark times exactly the
parse call; the parser and its node arena are torn down after the clock stops. Re-attributed
that way, nearly all of the profile's headline "dropping value trees" cost splits three ways:
about six percent is the benchmark discarding its own finished output (real, timed, and
untouchable without changing what the benchmark means), seven to ten percent is teardown after
the clock — CPU the profiler sees but the reported number never contains — and under two
percent is genuine in-parse churn. That one cut refuted an entire planned increment before a
line of it was written: re-homing the memo's stored values had been priced at four to six
percent, but its surface lives almost wholly after the clock. The same mistake the profile
killed twice before — pricing against a bucket the metric doesn't actually contain — now has a
name in this campaign: the stale-window ceiling.

What survived the cut is a single, precisely named mechanism, and it is the best kind of
finding: a structural one. When a rule's result passes through unchanged — an explicit `-> $1`,
or the implicit default every bare alternation gets — the generated code does not *pass it
through*: it deep-copies the child's entire shaped JSON subtree and makes the copy this rule's
own content. Every level of the grammar's spine repeats this on the whole accumulated tree,
and in a tournament every *losing* branch that parsed successfully performs its copy before
being discarded. The copies then outlive their usefulness inside the arena and the memo, which
is exactly the teardown the profiler was pointing at after the clock. Nine to twelve percent
of the timed parse goes to these copies alone.

The priced fix is the oldest idea in the functional-compilation literature — deforestation,
eliminating intermediate trees between a producer and its consumer — expressed in one line of
the generator: where extraction means "the child's value, unchanged," emit a *reference* to
the child node instead of a copy of its content. The output boundary already knows how to
follow such references, so the final JSON is equal by definition; a value gets deeply built
once per genuine object or array template rather than once per level passed through. The
ceiling, recorded before any code: ten to thirteen percent, with the teardown shrink as an
unclaimed bonus. The heavier designs the spike was expected to choose between — reference-
counted values, copy-on-write, an arena tape à la simdjson — were surveyed and declined with
reasons: their surfaces are mostly after the clock here, at many times the blast radius.

### The fold that was right about everything except the stopwatch

The fold was built exactly as designed — a borrowed value carrier that serializes
indistinguishably from the owned one, so a dumped tree cannot tell whether a copy or a
reference sits inside it — and it earned every proof the campaign demands: all eight
benchmark ASTs and outcome dumps reproduced byte-for-byte, all nine-hundred-odd differential
and certification oracles stayed pinned, and the generated parsers shed every one of the six
thousand pass-through deep copies they used to perform. Landing it even surfaced and fixed a
real piece of technical debt: two generated helper functions match exhaustively over the
engine's content type, which means *any* future addition to that enum would have broken the
regeneration toolchain against its own previous output; the emitted code now carries an
unreachable fallback arm, so the next such change will not need the staged two-pass
regeneration this one did.

And then the stopwatch said no: two independent five-round benchmark sets measured the fold
at under half a percent and just over two — against a ten-to-thirteen percent ceiling. The
pre-registered falsifier fired, and the campaign's constitution is unambiguous about what
happens next: no landing, full revert, and the restored parsers were proven bit-identical to
the pre-fold generation by hash. What makes the episode worth its place in this chapter is
*why* the ceiling was wrong, because the reason is now a permanent rule of the pricing
discipline. A sampling profiler charges each mechanism with its share of *all* CPU time; the
campaign's metric is the geometric mean of per-pattern *minimums*. Allocator churn — the
malloc-and-free traffic of building and dropping copies — concentrates in the iterations
*above* the minimum, where the allocator hits its slow paths; the minimum iteration, the one
the metric keeps, barely contains it. So a mechanism that is nine to twelve percent of
sampled time can be worth one to two percent of the reported number. The passes that
*over*-delivered — the constant-fold and the checkpoint fast path — were the mirror image:
fixed compute paid identically in every iteration, minimums included. The rule, recorded
where every future scout will trip over it: a sampled share prices a fixed-compute mechanism;
for allocator-churn surfaces it prices only the profile, not the metric.

With that, the optimizing-compiler rung is complete. Five passes landed (protocol hygiene,
predictive dispatch, two rounds of wrapper inlining, the checkpoint fast path, the
constant-fold), four were refuted by measurement before or at the land gate — and the
scoreboard reads 496 µs to about 27.4 µs, better than eighteen times, with every landing
byte-identical to its predecessor. What remains between here and the cold-parse target is
not another bucket to shave but a change of altitude: the deep-specialization extensions,
where the generator stops trimming the interpreter-shaped machinery and starts emitting
code specialized to what each rule actually is.

## The deep-specialization scout: three quarters of the work is thrown away

The deep-specialization campaign opened, as every rung before it, with a scout — and the
scout's first finding was about evidence, not speed. A host crash had wiped the previous
sessions' scratch directories, taking the byte-identity reference dumps with them, so the
first order of business was re-deriving all of them from the canonical artifacts and
transcribing their hashes into the durable task tree. In the process, a hash check caught
the on-disk benchmark binary claiming to be something it wasn't: it was the reverted fold's
*candidate* build, left behind when the session that meant to rebuild it was killed. The
canonical probe was rebuilt from the verified tree, and it reproduced the era on cue. The
lesson is worth a sentence in a book about performance: a benchmark number is only as
trustworthy as the identity of the binary that produced it, and identity is a hash, not a
recollection.

The measurement itself reframed the residual. Counting every rule entry across the
benchmark and splitting it into work that survived into the final parse versus work that was
attempted and rolled back, **just over three quarters of all rule entries — 1,886 of 2,503 —
are discarded speculation**, and the share is highest in exactly the patterns that parse
slowest. The parser is not slow because the surviving work is expensive; it is slow because
for every entry that contributes to the answer, three are tried and thrown away — along with
their share of checkpointing, rollback, and allocator traffic.

And the reason so much is tried turned out to have a single, named address. The shared
FIRST-set analysis — the one that already powers the landed predictive prune and the
byte-switch dispatch — deliberately gives up on any branch that begins with a regex-literal
terminal, classifying it as "unresolved, always try." That is sound, but it blinds both
consumers to precisely the branches that matter: the literal-matching spine of the regex
grammar reaches such a terminal, so the busiest choice site in the grammar attempts its
literal branch at every metacharacter position, knowing nothing about what bytes could
possibly begin it. A regex literal's admissible first bytes are perfectly computable from
the pattern itself. Teaching the analyzer that one thing — with the conservative fallback
intact for genuinely undecidable patterns — lets the two already-landed emissions prune and
dispatch dozens of additional sites at the next regeneration, with no new machinery at all.
That analysis increment, followed by a re-census and then a subset-dispatch emission for the
sites whose branches genuinely share a first byte (the parenthesis and backslash families,
where the *second* byte discriminates), is the priced, ordered road; region fusion and
memo-necessity analysis were priced honestly against the fresh numbers and parked. Every
ceiling was recorded against the metric's own basis — the discipline the refuted fold paid
for — and every one is falsifiable by the same alternated benchmark that will judge the
emissions.

## Seeing through regex literals: another 10% from pure analysis

The first deep-specialization increment landed exactly where the scout pointed, and it is
the campaign's purest example of analysis paying for emission that already exists. Nothing
about the generated parser's *machinery* changed: the FIRST-set analyzer simply learned to
derive the admissible first bytes of a regex-literal terminal from the pattern's own
syntax tree — character classes (negated ones included) map to ranges of UTF-8 lead bytes,
literals contribute their first byte, and anything genuinely undecidable keeps the old
conservative "always try" answer. References to the built-in single-character matchers
resolve the same way, and one carefully licensed subtraction handles the grammar's
negated-class idiom: a negative lookahead whose inner matcher is *exactly decided by one
byte* may subtract its bytes from what follows, which is how "not an ASCII character,
then any character" resolves to precisely the non-ASCII lead bytes. Exactness is the
license — subtracting an over-approximation would prune branches that could match, so
anything not provably byte-decided keeps the loose union. At the next regeneration the
two already-landed emissions did the rest by themselves: the predictive prune began
guarding the literal-matching spine (the always-tried literal branch at metacharacter
positions simply stopped being entered), and the byte-switch dispatch flipped nineteen
more choice sites — 41 to 60 — because their branches' first bytes were suddenly known
and disjoint.

The counters tell the soundness story better than any argument: across the benchmark, 346
of the 1,886 discarded speculation entries vanished while the *committed* entry counts
stayed exactly identical, pattern by pattern — the analysis removed only work that was
always going to fail. The alternated benchmark judged the result at **−10.0%**, faster in
all five rounds and on all eight patterns, inside the priced ceiling: roughly 27.4µs to
**24.7µs**, about **20× faster** than where the campaign began. Just as valuable is what
the landing reclassified: most of the discards that survived turn out to begin with a
backslash — escape-family branches that legitimately *can* start at every escape position
and only fail on their second byte. Those were never analysis-blindness; they are the
subset-dispatch family the next increment exists for, and the re-census now knows their
names in advance.

## The second byte: guards instead of nested switches

The follow-up increment attacked exactly that reclassified family, and its final shape is
a small lesson in letting a soundness fact pick the emission form. The plan sketched a
nested two-level `match` — dispatch on byte one to the subset of branches that admit it,
then on byte two within the subset. What landed instead is simpler: the per-branch prune
guard that already checks the first byte learned to check the *second* byte too, so a
branch like `\Q` in a tournament full of `\`-escapes is skipped outright when the input
reads `\b`. The tournament itself stays byte-for-byte the machinery it always was; the
guard is just a sharper admission test.

What forced the refinement is observable behavior on *rejected* inputs. PGEN reports the
deepest byte position any attempt reached (`furthest_position`), and that value is written
exactly once per rule entry. A branch shaped like `"x"` followed by a payload *rule* enters
that rule at offset one even when the attempt is doomed — so pruning it on byte two would
silently change the failure diagnostics a caller sees. The analysis therefore carries one
more fact per branch: whether a two-byte-refuted attempt could enter any rule past offset
zero. Only branches provably free of that side effect earn a second-byte guard; the rest
keep their first-byte guard, no worse than before. The census instrument classifies those
members exactly like byte-2 wildcards, so the diagnostic surface and the emitted code keep
telling the same story.

The guard form also turned out to be *stronger* than the sketched dispatch, not weaker: a
nested switch needs the subset's second bytes to be pairwise disjoint, but a guard only
needs each branch's own second-byte set, so it fires even at sites where two escapes
collide on a digit. The counters made that visible at the first regeneration — 541 of the
remaining 1,540 discarded speculation entries vanished, half again more than the census's
site-based model had priced, with committed counts once more exactly unchanged. The
alternated benchmark measured **−19.8%** — roughly 23.6µs to **18.9µs**, faster in all
five rounds, with the two capture-heavy patterns dropping by a third and a half — putting
the campaign at about **26× faster** than its starting point on analysis alone.

## The quantifier frontier: attempts no guard sees yet

Profiling the 18.9µs parser re-told a familiar story — allocator traffic still tracks
discarded speculation, and the 296 new guards themselves are invisible in the profile —
but the *census* of what still gets discarded named something new. The largest surviving
families are not alternation branches at all. They are rule entries made by **min-0
quantified sites**: shapes like `class_zero_width*` inside a character-class range, where
the loop dutifully *attempts* its element once at every position, enters the rule, fails
on the first byte, and rolls back. Every guard landed so far lives at a rule's top-level
branch dispatch, so these attempts sail past all of them by construction — the
character-class benchmark pattern, the one pattern the second-byte increment left
untouched, turns out to be made of exactly this class.

The same FIRST-set machinery prices the fix before any emission, through a new lane in
the census (`QUANT-SITE-CENSUS` / `QUANT-EXPOSURE` in `--report-fusibility-census`): every
quantified site in the grammar, classified for a *guarded attempt elision* — skip the
element's attempt outright when the next byte cannot start it. The soundness argument is
the branch-guard one at offset zero, plus one fact the alternation case lacks: a min-0
quantifier *always* attempts its element exactly once at the current position, so the
skipped attempt's only observable trace — the furthest-position diagnostic — can be
emulated exactly. On the regex grammar the lane reports 118 min-0 sites of which **102
are guardable** (the 16 blocked ones all wrap nullable optional payloads, correctly
refused), and joining the benchmark's outcome dumps attributes **192 discarded entries**
to rules that live *only* under guardable sites — 134 of them the character-class family
the profile named. The census states its own honest bounds: a byte-1-admitted attempt
that dies later survives the guard, and terminal-only sites never show up in per-rule
counters at all. On a whitespace-skipping grammar like full SystemVerilog the lane
reports its 1,113 min-0 sites and zero guardable — the raw-byte peek is unsound under an
implicit layout skip, and the census says so rather than over-promising.

The guarded emission followed the census immediately, with one exactness rule doing the
soundness work. Skipping an attempt must not change what a failed parse *reports*: PGEN's
deepest-position diagnostic is written once per rule entry, so the guard emits a one-line
emulation of the entry the skipped attempt would have made — but only where that
counterfactual is *decidable*. A site whose element is a bare rule reference always
enters that rule, so the emulation is exact (even against memoized failures, which imply
an earlier real entry already recorded the position); a site whose element is pure
terminal matching enters nothing, so no emulation is exact; anything in between stays
unguarded rather than guess. That classification splits the 102 guardable sites into 78
bare-reference and 2 terminal-only guarded sites, with 22 mixed sites left for a possible
later refinement. The regenerated parser carries the guards at every emission instance —
the optional-element fast path included — and the counters told the usual story at first
regeneration: 285 of the 999 remaining discarded speculation entries vanished
(`class_zero_width` all 134 of them), committed counts exactly unchanged, every abstract
syntax tree byte-identical. The alternated benchmark measured **−4.3%** — roughly 19.0µs
to **18.2µs**, faster in all five rounds, with the character-class pattern (the one the
previous increment could not touch) dropping **12.7%**. One honest note the numbers
force: the priced ceiling had assumed each killed attempt cost what the earlier
increments' kills cost, but these attempts were already cheap — refuted at their first
branch guard — so the per-kill payoff was smaller. The census counted the kills exactly
right; the exchange rate belonged to a different population.

## The endgame of discard elimination — and where the road goes next

The ninth re-profile, taken on the 18.2 µs parser, adjudicated the whole guard/dispatch
program rather than pricing another increment. Three findings close it. First, the
allocator's ≈40% share of self-time is **shape-preserved for the third consecutive
re-profile** — every round of discard elimination removed allocation *volume* without
changing its shape, because the shape belongs to the speculation protocol itself, not to
any family of failed attempts. Second, the residual discarded speculation (715 entries
bench-wide) is now **broadly distributed**: the largest family holds 8.8% of the
residual, where every earlier re-profile had a dominant family to aim at — the byte-1
and byte-2 guard machinery has consumed every concentrated surface, and each parked
follow-up (byte-3 dispatch, the mixed quantifier frontiers, memo-necessity analysis)
re-prices at a few percent or less on its own population's exchange rate. Third — the
decisive bound — pricing a **perfect endgame** in which every last residual discard is
eliminated lands the parse at ≈12.5–16 µs: an order of magnitude short of the ≤1 µs
closure bar even at 100% success.

The conclusion is architectural, and it is the second time this campaign has reached it
from independent evidence (the measured-distance section above drew it at 99×; this
re-profile re-draws it at 27×): the remaining gap does not live in failed attempts — it
lives in what the **committed** path executes. Roughly eleven rule entries per input
character, each paying the per-entry protocol and its speculation-shaped allocation,
where a hand-written parser would run straight-line code. The road that attacks exactly
that — **full cascade folding**, emitting one fused direct-coded matcher for a provably
simple grammar region with its value build folded to the annotated result — opened as the
next scout under the same discipline as every row above it: a falsifiable price recorded
before any build, and a land gate that only passes measurably faster *and* byte-identical.
Its first increment has now landed; the next section tells how.

## The observability twin — running the parser without its instruments, without losing them

Every lever above made the *protocol* cheaper. The cascade-folding road asks a different
question: what if a bare parse — no coverage, no trace, no counter reader, no memo
statistics — simply did not execute the per-rule protocol at all?

The obstacle was never speed; it was honesty. The per-rule protocol frame *is* the
observability contract: the entry counters feed the diagnostic dashboards, the
transactional coverage stack is what certificate witnesses are made of, the memoization
table is what the memo-statistics report describes. Deleting any of it would make those
surfaces lie. The landed answer is the **observability twin**: the generator now emits
each provably-foldable grammar region twice. The protocol methods stay byte-for-byte as
they were. Alongside them, each region rule gains a compact fused `cascade_` function —
the same parse decisions, the same guards the earlier levers landed (the byte-switch
dispatch, the two-byte prune guards, the quantifier attempt elision, all reused through
the same shared analysis), and the same annotated value construction, with the frame
elided: no counter increment, no coverage push, no trace scope, no transaction wrapper,
and speculation that restores only the input position wherever the region provably
cannot touch the semantic store. An *acyclic* fused rule also drops the recursion-guard
bookkeeping and the memoization probe entirely (it can never re-enter itself, and a
repeated probe at the same position is bounded by the grammar's own shape).

The second increment extended the fold through the grammar's **cyclic spine** — for the
regex grammar, the pattern/alternation/concatenation/piece/atom core that every parse
descends — so a bare parse now runs fused code end to end. A cycle-participating rule
keeps exactly the two protocol parts that are load-bearing on a cycle, in lean form.
First, the recursion-guard check: infinite-recursion and left-recursion detection scan
the parse stack for that same rule's in-flight frames, which is exact only if every
cyclic rule pushes a frame in both graphs — so each cyclic fused function carries the
protocol's own check, minus the trace lines a bare parse can never enable. Second, a
**thin memo** (the packrat protection a recursive descent cannot lose — an earlier
campaign incident measured a 117× collapse without it): entries are only `position →
(end, value)`, with no stored effect deltas, so an entry is cached only when value-only
replay is provably equivalent to re-execution. Each entry carries the protocol memo's
own taint classes, measured across the body by three engine counters: a **pure** body
(no store read, no store write) replays at any later store state; a **store-reading**
body replays only while the store is provably unchanged since (the same write-epoch
license the protocol's taint-gated memo uses); a **store-mutating** body is never
cached — a value-only replay would silently skip the facts it wrote, so every re-probe
honestly re-executes. That last distinction was not theoretical: the first cut validated
every entry against a global "store unchanged" check, and the alternated benchmark
promptly *regressed* the two fact-writing patterns (+3% and +15%) because every capture
fact evicted the whole spine's entries — the per-entry classes recovered them to 9% and
6% wins, a controlled A/B in which only the validity rule changed.

Which graph runs is decided once, at parse start. A parse with no diagnostic consumer
takes the fused graph. The moment anything asks to observe — certificate coverage, a
trace flag, the memo-statistics switch, or any reader of the per-rule counters (taking
the counter handle *is* the request, so a future diagnostic surface cannot forget to
ask) — the parse runs the protocol graph, and every counter, witness record, and trace
line is the exact machinery it always was, not an emulation. That is why every
diagnostic pin in this chapter survives the landing byte-exact.

Two store-soundness rules survive into the fused code, both computed statically from the
same census plan the report prints (one implementation, so the plan and the emission
cannot drift). A fused speculation that could reach a fact-writing rule keeps the full
semantic snapshot; an alternation with such a branch keeps the protocol's tournament —
winners replayed, losers rolled back — as an island inside the fused code, because plain
backtracking there would visibly change what the semantic store remembers.

Both landings followed the standing discipline. Each time, the fused graph produced
byte-identical syntax trees on the full benchmark before any speed was measured; the
interpreter-equivalence oracle re-proved every registered grammar byte-identical through
its fused graph; the 2,189-case PCRE2 conformance corpus ran through the fused code and
reproduced its pinned verdicts; and a live stack profile showed the `cascade_` functions
actually carrying the parse. The acyclic increment measured **−13.6%** — roughly 17.8µs
to **15.4µs**, faster in all five rounds, with the character-class pattern (the region
the plan had identified as the anchor, nearly three-quarters of its entries foldable)
dropping **27.6%**. The cyclic-spine increment then measured **−11.4%** — to roughly
**14.0µs**, again faster in every round and every pattern. One honest note each: the
acyclic increment priced below its −18–25% band because the folded population's frames
were already cheap (the protocol-hygiene and inlining levers had stripped them first);
the spine increment priced below its band because the machinery a cycle must *keep* —
the guard check, the thin-memo probe and insert — is real per-entry cost the price
model had treated as killed. The same emission landed platform-wide both times: every
generated parser carries its own fused regions, sized by its own census plan at the
cyclic-spine increment — the VHDL parser folds into a single region rooted at its entry
rule, 216 rules of fused descent.

The campaign's story so far, including every refuted road,
is told in [The Speed Journey](speed-journey.md).
