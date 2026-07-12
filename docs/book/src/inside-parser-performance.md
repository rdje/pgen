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

---

## The latency campaign: closing the gap to PCRE2 (small-input speed)

The section above is about the *big* input (uvm, ~3 MB) staying **bounded**. This section
is about the opposite end — the *small* input being **fast**. They are different problems
with different profiles, and this is the story of the second one, told as a running
scoreboard so it can be watched as it improves.

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
the right next step, and now a measured one. The first code increment — the lockstep advance folded
together with the arena on the regex `atom` tournament — is what the scoreboard measures next.

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
