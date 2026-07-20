# The Speed Journey: 496 µs → ≈1.94 µs

> **Part II · Inside PGEN.** This chapter is the companion to
> [Inside the Parser: Termination & Performance](inside-parser-performance.md). That
> chapter is the *mechanics* record — what each landed lever is and how it works, kept
> current as the running scoreboard. This one tells the **campaign itself**: the method
> that drove it, the arc of what landed and what was refuted, and the lessons that now
> have names. Every number here is pinned to its primary record in the task tree
> (`docs/tasks/RGX-0078.md`) and `CHANGES.md`; nothing below is a recollection.

Between sessions #90 and #166 — nine calendar days — PGEN's regex parser went from a
**496 µs** geometric-mean parse over its eight-pattern benchmark to **≈1.94 µs** with a
mimalloc-class global allocator: about **256× faster**, with every single landing proven
**byte-identical** on its full oracle battery before its speed number was believed.
On the external PCRE2 corpus the same work put the **maximum** observed parse at ≈484 µs
and the corpus geomean at ≈1.26 µs.
Thirty-odd levers landed. More than a dozen others were refuted, rejected, or reverted —
most of them for the price of a document rather than a build. This chapter is the story of
both lists, because the refusals are as much the method as the landings.

## Why the journey happened

PGEN's regex parser is the compile front-end for RGX: when a tool compiles thousands of
regular expressions, PGEN's parse time per pattern *is* that tool's compile time per
pattern. The director elevated speed to a first-class, continuously tracked deliverable —
co-equal with accuracy, in strict order: **accuracy first and never regressed** (the
immovable floor), speed maximized on top.

The finish line moved once, deliberately. The campaign opened against a *relative* bound —
geomean within 5× of PCRE2's compiler on the same patterns. Mid-campaign the director
re-baselined it to a tighter *absolute* one: **a cold parse at roughly ≤1 µs, with the
EBNF grammar (plus annotations) remaining the sole source of truth**. That second clause
is the hard part and the whole point: the speed must come from *compiling the grammar
better*, never from hand-writing a parser next to it.

And one rule was constitutional from the first slice: **any optimization that regresses
parity is rejected without second thought**, no matter how fast it is. Every landing in
this story produced output byte-for-byte identical to its predecessor. That rule is not a
tax on the campaign — as the arc will show, it is what made the campaign's radical steps
*cheap to attempt*.

## The method — profile, scout, price, then land or refute

The campaign's real protagonist is a loop, applied without exception:

1. **Profile the real binary.** Every steering decision starts from a fresh sampling
   profile of the *current* release build (two 30-second windows covering the full
   benchmark, call-tree-only attribution, self-times reconciled against the root count).
   Never a guess, and never a stale profile: after every landing the landscape is
   re-measured, because each win reshapes where the remaining time lives.
2. **Scout the surface (step 0).** Before any emission code, a read-only instrument — a
   census, a counter, a strip-variant — measures exactly how much of the benchmark the
   proposed mechanism can even touch.
3. **Record a falsifiable price.** The scout's output is a written ceiling ("this pass is
   worth ≈−X–Y% on this basis"), recorded in the task tree *before* the build. The basis
   is named, because ceilings computed on the wrong basis were the campaign's most
   instructive failures.
4. **Land iff faster — or refute.** The land gate is a drift-controlled measurement:
   both binaries built fresh at release fat-LTO with distinct hashes, measured
   *alternately* (five rounds of 2,000 parses) so shared machine load cancels, judged on
   the geometric mean of per-pattern noise-floor minimums. Faster and byte-identical
   lands; anything else is reverted or never built, and the record says why.

Under the loop sits the **safety net** that made it fast to run: the byte-identity oracle
battery. PGEN keeps an *interpreter twin* — a grammar-AST interpreter that executes the
same grammar the code generator compiles — and a differential-equivalence gate that holds
the two byte-identical (verdict, typed AST, and the furthest-position diagnostic) across
every grammar at three fixed seeds. Around it: the eight benchmark AST dumps compared
bit-for-bit, the certificate-coverage pins, the PCRE2 compile-oracle over the real
corpus, and the shape/duality gates. The consequence is easy to miss and worth stating
plainly: **rewriting the emitted code stopped being dangerous.** A slip anywhere in a
radical codegen change is caught by a cheap, deterministic diff long before it could
mislead a benchmark — so the campaign could afford to attempt things (collapse frames,
elide the memo, fold whole evaluators away) that would be reckless without the net.

## The arc

### Act I — the profile speaks

The opening profile of the 496 µs parse said two things: **59% of self-time is the
allocator**, and the dominant *source* of allocation is failed speculation — the
longest-match tournament tries every alternative and rolls most of them back, paying
clone-and-restore machinery each time. It also refuted the obvious guess on day one:
parser *construction*, which looked like an easy fixed cost, measured at ~0.7% and was
dismissed without a line of code.

The first levers followed the profile directly. Guarding the rollback path's
scope-restoration on "did scope state actually change?" — a pure engine change — landed
**−18.6%**. The free build flags came next, each measured rather than assumed: fat LTO
with one codegen unit landed **−6.7%**; `target-cpu=native`, commonly assumed free, made
the parse **4% slower** and was rejected; PGO measured about −1% and was declined as a
build-process burden a low-single-digit win does not justify. Swapping the per-rule
annotation-table lookups from SipHash to FxHash landed **−5.4%**.

### Act II — structure over buckets

The next tier stopped shaving buckets and changed what the generated code *does*.
**First-set predictive dispatch** — compute at codegen time a sound over-approximation of
each branch's admissible first bytes, and skip branches that provably cannot start at the
next input byte — landed **−75%**, a 4× step in one lever, byte-identical across the full
battery. The **reference arena** moved every parse-node allocation onto a per-parse
destructor-running arena (**−21.9%**), chosen over an index arena precisely to keep the
derived serializer — the byte-identity oracle itself — untouched. The **construction
cache** (build the grammar-constant annotation tables once per process) closed the act at
**−3.8%**.

The same act produced the campaign's first two named refusals. **Lazy winner-only
materialization** was implemented, proven byte-identical, and measured *neutral* — it
attacked the same losing-branch cost first-set dispatch had already removed. It was
reverted, and the lesson recorded: two levers can be *substitutes*, and a marginal gain
is measured against the current baseline, never the original one. Its queued sibling —
**memo subtree sharing** — was then killed *before* being built: a focused profile showed
the memo's clones at 1.27% of the parse. The discipline had started paying for itself.

### Act III — measuring the distance

With the profile flat (no bucket above ~12%), the campaign measured its actual distance:
**≈99×** from PCRE2's compiler, with every remaining named bucket summing to ~1.5× even
if it all vanished. Parsing a four-character literal made 45 rule entries at ~262 ns
each — roughly eleven entries per input character. The verdict was architectural: the
cost is the **rule-cascade-per-character execution model**, addressable only by executing
*fewer, cheaper entries*, not by shaving the machinery around each one.

Two research-grade roads were adjudicated at this altitude, both refuted by measurement
or literature before a build. **GLL with a graph-structured stack** — machinery whose
value is taming nondeterminism — was declined because the engine's measured backtracking
residue after first-set dispatch and memoization was ~2.2%, exactly the regime where the
literature shows adaptive top-down parsing wins by orders of magnitude. **The derived
scanner rung** (fuse token-shaped sub-languages into DFAs) died on its own step-0 census:
the killable share of rule entries capped the win at ~1.5–1.8× against a hoped-for 3–5×.
What survived was the agreed destination: an **optimizing middle-end** — "the planner" —
between the grammar AST and code emission, deriving every speed decision from grammar
analysis so grammar authors never think about speed.

### Act IV — the planner

The planner opened with its own scout: six measurement-only strip-variants of the
generated parser, each byte-identity-proven, pricing every piece of per-entry machinery.
The headline was a surprise — **about a third of the parse was unconditional bookkeeping
waste**, dominated by heap-allocating trace-naming strings for a trace that is off. That
measurement fixed the pass order, and the passes landed one measured step at a time:

- **P0 — protocol hygiene** (build trace payloads only when trace consumes them):
  **−25.8%**, with the trace output under tracing proven byte-identical.
- **P2 — degenerate-tournament byte-switch** (where FIRST-set disjointness *proves* a
  tournament degenerate, emit one `match` on the next byte): **−5.3%**, predicted −3–7%
  by its census before any code.
- **P1a/P1b — cascade inlining, then memo elision at inlined frames**: **≈−4–7%** and
  **≈−7–12%**, under a measured code-size budget — the tighter budget variant was built,
  raced head-to-head, and *lost*, so the wider one stayed.
- **P3a/P3b — selective guard/memo machinery**: *refuted before a line was written*. The
  re-profile showed the entire packrat cache at ~3% and the recursion guard under 1% —
  the census ceilings had been priced on a cost model from a benchmark twice as slow.
- **P3c-i — the store's epoch fast path**: a four-counter census first proved 98.8% of
  rollbacks and 93.6% of delta extractions happen with a provably unchanged store; the
  O(1) epoch check landed **−11.2%**, above its own ceiling (an uncounted second
  population rode the same fast path).
- **P4-i — the `@constraint` constant-fold**: the profile's top named symbol turned out
  to be the parser *re-reading its own documentation* — grammar-constant prose
  annotations re-evaluated through the full expression machine on every rule exit, ≈1 µs
  each, to conclude every time that prose means "true". Folding the classification into
  codegen landed **−12.1%**, with the evaluation-heavy patterns gaining 26–31%.
- **P4-ii — memo-value ownership**: refuted by a sharper cut of the same profile — its
  surface lived almost entirely *after* the benchmark's stopwatch (parser teardown), in
  CPU the profiler sees but the reported metric never contains.
- **P4-iii — the pass-through reference fold**: the act's closing lesson. Implemented in
  full, it eliminated all ~6,000 pass-through deep copies, earned every byte-identity
  proof — and measured **−0.4%/−2.2%** against a 10–13% ceiling. The pre-registered
  falsifier fired; the fold was reverted, the restored parsers proven bit-identical by
  hash. The post-mortem named a permanent rule (see the lessons below): a sampled share
  prices a fixed-compute mechanism, but for allocator-churn surfaces it prices only the
  profile, not the min-based metric.

The planner closed at ≈27.4 µs — better than 18× — with five passes landed and four
refuted by measurement before or at the land gate.

### Act V — deep specialization

The final act opened with a scout whose first finding was about *evidence*: a host crash
had wiped the byte-identity reference dumps, and re-deriving them caught the on-disk
benchmark binary being a leftover *candidate* build from the reverted fold — a benchmark
number is only as trustworthy as the hash of the binary that produced it. The canonical
probe was rebuilt from the verified tree before any timing.

The measurement then reframed the residual: **three quarters of all rule entries were
discarded speculation**, and the single named cause was the FIRST-set analysis giving up
on regex-literal terminals ("unresolved, always try") — blinding the landed prune and
dispatch emissions exactly where the grammar is busiest. Three increments followed:

- **D0 — seeing through regex literals**: teach the analyzer to derive admissible first
  bytes from a regex terminal's own syntax. Pure analysis; the already-landed emissions
  did the rest at regeneration. **−10.0%**, with 346 discarded entries gone and committed
  counts *exactly* unchanged, pattern by pattern.
- **D1 — the second byte**: the per-branch guard learned to refuse on byte two, licensed
  branch-by-branch by a furthest-position-parity proof. The guard form turned out
  *stronger* than the sketched nested switch (no pairwise-disjointness needed), and
  over-delivered its ceiling: **−19.8%**, 541 more discards gone, committed unchanged.
- **Q-guard — the quantifier frontier**: min-0 quantified sites skip their element's
  doomed attempt under an exact one-line furthest-position emulation. **−4.3%**, the
  character-class pattern (untouchable by D1) dropping 12.7% — and an honest ceiling
  miss, because these attempts were already cheap; the kill count was exactly right, the
  *exchange rate* belonged to a different population.

Then the ninth re-profile closed the act with the campaign's most important adjudication.
The residual discards — 715 of them — are now **broadly distributed** (the largest family
is 8.8% of the residual; every prior re-profile had a dominant family). The allocator's
~40% share is *shape-preserved* through three consecutive rounds of discard elimination:
the guards remove volume, never shape. And the endgame bound is decisive: killing **100%
of the remaining discarded speculation** — a perfect, unreachable endgame — prices out at
≈12.5–16 µs. The ≤1 µs bar sits an order of magnitude below that. **The guard/dispatch
program is exhausted**: what remains is not failed attempts but the execution model of
the *committed* path itself — which is where the next road, full cascade folding, begins.

## The refuted roads, told honestly

Refutation-by-measurement is a feature of this method, not a failure mode. The table is
the campaign's second scoreboard — what did *not* land, how far it got, and what it paid
for:

| Road | How far it got | Why it died | What it bought |
|---|---|---|---|
| `target-cpu=native` | built + measured | **+4% slower** — nothing to vectorize; code bloat hurts I-cache | "a flag's reputation is not a measurement" |
| PGO | built + measured | ~−1%, a build-process burden with a bounded ceiling (layout can't fix allocation) | the ceiling argument itself |
| Lazy winner-only materialization | implemented, byte-identical | measured **neutral** — a *substitute* for the already-landed first-set prune | the substitutes lesson; the design returns inside any future lockstep engine |
| Memo subtree sharing | design + focused profile | the memo's clones measured **1.27%** of the parse | killed *before* invasive lifetime surgery |
| GLL / graph-structured stack | literature-first design spike | backtracking residue ~2.2% — GLL taxes every entry to remove almost nothing | a research rewrite refuted for the cost of a document |
| Derived-scanner rung | step-0 fusibility census | killable entry share caps the win at ~1.5–1.8× vs the hoped 3–5× | the census instrument itself, reused by every later pass |
| P3a guard elision | census + re-profile | recursion guard measured **<1%** — the paper ceiling used a stale cost basis | "re-profile before pricing" made mandatory |
| P3b memo elision at method frames | census + re-profile | the entire packrat cache measured ~3% | same |
| P4-ii memo-value ownership | design-spike attribution cut | its surface lives **after the stopwatch** (teardown) — the metric never contained it | the stale-window rule |
| P4-iii pass-through reference fold | fully built, all oracles green | **−0.4%/−2.2%** vs a 10–13% ceiling — allocator churn lives above the min-iteration | the sampled-share vs min-metric rule; two compile-affordance arms kept |
| D3 memo-necessity analysis | priced at the re-profiles | memo self-time 0.2% — dead | a clean park, twice re-confirmed |
| D1-ii / mixed-frontier v2 | priced on the residual census | ≈−1–3% combined — thin against their build cost | the per-population re-price in action |

Half of these died before a build; the other half died at a measurement gate that was
written down before the attempt. That is the intended shape: **the expensive way to be
wrong is to land; every other way is cheap.**

## The lessons, named

These are the campaign's transferable rules — each earned by a specific failure or
surprise, each now applied as standing discipline:

1. **Profile before you reshape, and re-profile after every landing.** Two whole passes
   (P3a/P3b) died because their ceilings were priced on a cost model from a benchmark
   twice as slow. A stale profile steers the next lever wrong by construction.
2. **Substitutes, not complements.** Two levers attacking the same observable cost do
   not add. Measure the *marginal* gain against the *current* baseline — the decisive
   before/after is what caught a lever that "obviously should help" delivering nothing.
3. **A sampled share prices fixed compute — not a min-based metric.** A sampling
   profiler charges a mechanism with its share of *all* CPU; a geomean-of-minimums
   metric barely contains allocator churn, which concentrates above the minimum
   iteration. The passes that over-delivered (constant-fold, epoch fast path) were fixed
   compute; the one that under-delivered 5× (the reference fold) was churn.
4. **The stale window.** Know what your stopwatch contains. Teardown after the timed
   region is real CPU the profiler sees and the metric never will; an increment priced
   against it is priced against nothing.
5. **Kills price at a per-population exchange rate.** Eliminating N discarded attempts
   is worth N × *that population's* per-attempt cost — never the campaign's historical
   average. The Q-guard killed exactly the predicted attempts and still missed its
   ceiling, because those attempts were already cheap.
6. **A clean instrument needs a settled tree.** A staleness or identity claim is only
   meaningful from a proven fixpoint (build idempotence first, then touch one thing).
   The corollary incident: an order-dependent FIRST-cache poisoning that made a census
   disagree with emission by traversal order — found and fixed because the instrument
   was held to the same identity bar as the product.
7. **Census and emission must share one implementation.** Any analysis that both
   *reports* a surface and *gates* an emission lives in one shared function, or the two
   silently drift apart.
8. **A binary's identity is a hash, not a recollection.** The on-disk benchmark probe
   was once a reverted candidate build; only a recorded SHA caught it.
9. **Record the ceiling before the build, and let it falsify.** A pre-registered price
   is what lets a fully-built, fully-proven change be reverted without argument when the
   stopwatch says no.
10. **Byte-identity first, speed second.** Prove a candidate identical on the cheap
    deterministic gates before spending on measurement. The identity floor is also what
    makes aggressive rewrites affordable at all — and it caught real incidents on the
    way (a regeneration-path drift that silently degraded annotation literals; the
    equivalence gate was exactly the tripwire).

## Where it stands, and the road ahead

At the ninth re-profile the scoreboard read **496 µs → ≈18.2 µs — about 27×** — with
every landing byte-identical and every refutation recorded. Against the ≤1 µs bar, that
re-profile was unambiguous: no amount of further discard elimination gets there, because
even the perfect endgame lands an order of magnitude short. The remaining gap lives in what the
**committed path** executes — the per-entry rule protocol and its speculation-shaped
allocation, ~11 rule entries per input character where a hand-written parser would run
straight-line code.

The road opened next is **D2 — full cascade folding**: teach the generator to emit, for
provably simple regions of the grammar, the fused direct-coded matcher a performance
engineer would write by hand — collapsing the committed descent itself rather than
pruning failed attempts around it. It began the way every road in this chapter began:
with a docs-and-instrument-only scout and a falsifiable price recorded before any build,
under the same hard constraint as everything before it — measurably faster and
byte-identical under the full battery, or it does not land.

**Both increments have landed.** The scout priced the exposure, the design record
answered the honesty problem with the *observability twin* — every foldable region is
emitted twice, a fused bare-parse graph beside the untouched protocol methods, with any
diagnostic consumer routed to the protocol graph so every counter, witness, and trace
line stays the exact machinery it always was. The acyclic-sub-region increment (D2-A)
measured **−13.6%**, faster in all five alternated rounds, and the cyclic-spine
increment (D2-B) — recursive fused functions through the grammar's core, with a
taint-classed thin memo preserving packrat protection on the cycle — measured a further
**−11.4%**, faster in every round and every pattern: **496 µs → ≈14.0 µs, about 35×**,
byte-identical under the full battery, landed platform-wide across every generated
parser. Both measurements re-taught lesson 6 in a new costume (the acyclic fold's
frames were already cheap; the spine fold's price had treated the cycle-mandated guard
and memo machinery as killable when soundness requires keeping it), and the spine
landing added a sharper lesson of its own: its first cut validated thin-memo entries
against a *global* store-unchanged check, and the benchmark immediately regressed the
two fact-writing patterns — the fix was to give each entry the protocol memo's own
per-entry taint class, and a controlled A/B in which only that rule changed turned both
regressions into wins. The remaining road to the ≤1 µs bar runs through the named
bar-complete program: root/boundary protocol thinning (D2-C) and the match-then-build
value model, each to be re-priced on a fresh census and profile the way every increment
before them was.

## The allocator, priced and banked

One increment did not need the compiler at all. Every re-profile of the campaign found
the same shape: roughly two-fifths of the parse burns inside the platform allocator —
`malloc` for the short-lived value nodes the parse builds, and `free` for the far larger
number it immediately throws away as failed speculation. Sampled profiles say "allocator
≈41% + `_xzm_free` ≈21%", but the campaign had learned (lesson: *sampled share ≠
min-metric*) not to trust a sampled share on the geomean-of-minimums metric. So it built a
measurement, not an argument: a never-free bump `#[global_allocator]` swapped into the
bench — every `malloc` a pointer bump, every `free` a no-op, the arena reset between
samples so page faults never enter the timed region. That prices the *falsifiable ceiling*
of the whole allocation axis. It measured **−40.0%**: the value/alloc cost is real on the
min metric, not a sampling artifact.

The bankable half followed immediately. A mature production allocator (mimalloc) captured
**−29.2%** — 73% of that ceiling — from a one-line global-allocator swap, deterministic
across rounds, every pattern faster, and correctness-neutral by construction (an allocator
cannot change program semantics for a program that never depends on an allocation's
address — the same reasoning that made the fat-LTO build profile safe). Because a *library*
must never impose a global allocator on the binaries that embed it, this is not a lib code
change: it is banked as the **recommended production configuration** — documented in the
regex integration contract, adopted as the campaign's baseline — and it takes the
scoreboard to **496 µs → ≈9.77 µs, about 51×**. The honest arithmetic it leaves behind is
sharp: even a *perfect* allocator (the never-free floor, ≈8.3 µs) sits an order of
magnitude above the ≤1 µs bar, so the road from here must also cheapen what the *committed*
path computes — the match-then-build value model and the committed-value representation
itself.

The match-then-build fold is now landed (session #132, after a getenv hoist trimmed
another −2.3%): no fused rule constructs a value during matching — the match pass records
a tape of POD *decisions*, failed speculation truncates its segment, and only the
committed derivation is built, once. Even the cyclic spine's memo keeps its protection
with derivation *segments* as the payload. It measured **−8.8%** (every round, every
pattern) against a directional model that said −45…−53% — the fourth reminder that a
sampled profile share is a reason to attempt, never a price. Its most valuable output
wasn't the number: the differential-equivalence oracle caught a subtle build-side `$text`
slicing defect that had been latent since the fold's first (reverted) attempt — proof
that the battery, not the tripwire, is what a landing means.

The committed-value **representation** change followed (session #136): the values a parse
*commits* are no longer eagerly-built `serde_json::Value` trees (`BTreeMap` nodes, cloned
`String` keys, a double deep-clone per nested reference) but arena-slice `PgenValue`s —
`Copy` machine words whose serialized bytes are identical by construction, with template
object keys sorted at *codegen* time so an emitted object is one arena bump of a
stack-built array. It measured **−32.0%** — every round, every pattern — the largest
single landing since pass zero, and this time the sampled-share model (≈25–30%) slightly
*under*-priced the win: the removed machinery had been feeding the allocator and the
teardown path too.

The next landing attacked what the re-profile of that 6.11 µs artifact named dominant:
the **protocol-boundary frames** around the token-shaped leaf rules the fused cascade
kept calling as full methods. A census cost model priced a boundary entry at roughly
four times a fused-internal one and — before a line of emission — recorded a falsifiable
expectation band (−9…−15%) with a refutation bar. The D3 **boundary scanners** landed
direct-coded frameless `scan_<rule>` functions for the census-qualified population
(19 rules in the regex artifact), reached only on the bare path while every diagnostic
consumer keeps the untouched protocol twin. It measured **−15.5%** — every round, every
pattern — squarely inside the recorded band: the first landing whose census model was
CONFIRMED rather than surprised. And the all-11 equivalence oracle earned its keep
again, catching a span-vs-token value fold divergence in SystemVerilog before the land
decision — a defect the regex-only tripwire could never see.

The next profile (#15) then found the single hottest remaining primitive hiding in
plain sight: every constant terminal literal — `|`, `(`, `\`, a digit — went through
`match_string`, whose runtime-`&str` parameter forces a real libc `memcmp` call plus
two UTF-8 boundary checks per attempt, even though codegen knows the literal at
emission time. A census over the artifacts measured the population (2862 sites in the
regex parser, 78.5% one-byte, 100% ASCII), a bench-minima-fitted model priced it
honestly at −4…−7% bench-wide (the profile window's 14.5% headline was
digit-pattern-weighted — the census caught the over-price *before* emission this
time), and the landed `match_lit_ascii` fast path measured **−4.8%**, inside the band.
Then the **thin-memo spine machinery** (lever 5.i.14) landed three compounding
elisions on the fused cascade's cycle-participating spine — an integer-compare
recursion guard scan (a parallel dense id stack, refined mid-slice from the design's
tuple-widening after the mandated compile check caught three emitted sites reading the
name stack's shape directly), adaptive memo pre-sizing, and inline-small `SmallVec`
segment storage that elides the per-committed-sub-derivation malloc pair — measured
**−12.0%**, which honestly *exceeded* the census-fit HIGH ceiling of −9.5%: the first
over-delivery of the campaign, most plausibly a super-additive alloc/cache compound the
additive census under-priced, named for a future re-profile rather than smoothed over.
Then came the **direct-value build** (−17.6%: the fused cascade constructs committed
`PgenValue`s directly, no dead node scaffolding), and a change of instrument: the campaign's
steering metric widened from the 8-pattern bench to the FULL PCRE2 external corpus — every
one of 2,189 real-world patterns timed, with the MAXIMUM observable parse the primary
target. That reframing found the worst cells and their mechanisms one by one: the
winner-in-place tournament commit (K1, corpus MAX −38%), a grammar left-factoring of the
`piece` alternation (K1c, the nesting ladder −80%), memo pre-sizing, an inline
checkpoint-chain, rule-id annotation tables, a root-elided snapshot, and a WAM-style
undo-trail checkpoint (K4a) that made checkpoints seven `Copy` words. Two levers were
built, measured below their falsification bands, and REVERTED whole — the discipline that
every claimed percent is a measured percent cuts both ways.

The latest structural find generalized the oldest one: the level-1 FIRST-byte guard and its
global second-byte refinement became a **bounded per-path prefix trie** (FIRSTₖ, k ≤ 4) —
each alternation branch carries a small byte-trie of every prefix its matches can have, the
guard walks it, and a branch whose walk falls off is refuted without ever being attempted,
with an exact one-write emulation preserving the parser's furthest-position diagnostics.
That single lever cut the corpus MAXIMUM by a third in one slice (722 µs → 484 µs),
crossing the campaign's first milestone (corpus MAX ≤ 500 µs), and took the bench floor
below 2 µs for the first time. The full-battery oracle earned its keep once more: it caught
a real over-prune (a version conditional the trie wrongly refused) before any measurement,
tracing to a truncation-stickiness invariant now unit-pinned.

After the trie, the campaign paused to put its own books in order: a held-carrier
composition re-derived every queued candidate from first principles, corrected an
accounting conflation (attributed regions and gross ceilings are not banked savings), and
locked a stricter bar — from here, one fix per session, accepted only when the *full
external-corpus geomean* strictly drops with identical verdicts. The first fix through
that gate was the smallest kind of lever: the parser had been **rebuilding its semantic
runtime state from scratch before every parse** — snapshot the facts, construct a fresh
state, drop the old one, replay the facts, re-clone the predicate registry — a ceremony
whose four call sites priced at a conservative ≈23 ns. Replacing it with an exact
**in-place reset** (facts and their indices survive re-stamped to the root scope,
everything else returns to fresh-state semantics, proven field-for-field equal to the old
ceremony by a whole-state equality oracle) cut the corpus geomean by **−5.1%** — roughly
2.9× the conservative pricing, the unpriced upside being the per-parse allocation traffic
of building and dropping a populated state.

The second fix through the gate was smaller still: the rollback checkpoint — the K4a-era
seven-word `Copy` value constructed at every speculation — carried **two words that were
provably always equal** (the legacy scope-stack depth and the active-chain depth, kept in
lockstep by every mutation site in the runtime). Deleting the duplicate made the
checkpoint six words, with the public accessor preserved over the surviving twin and the
debug asserts retargeted so the lockstep invariant is now re-proven mechanically on every
debug-build rollback. The corpus geomean dropped **−1.4%** — with the recorded caveat
that a change this size sits inside the measurement's noise span, so the strict
same-session direction gate (not the magnitude) is what adjudicated the land.

The third fix through the gate retired a whole duplicate data structure from the hot
path. Ever since the rule-ID stack became the recursion guard's real cycle-check
representation, the parallel **human-readable name stack** — a 24-byte frame beside every
16-byte ID frame — was maintained purely for readers that are cold, trace-gated, or
reconstructible: error context, rollback labels, the depth-ceiling payload. Three
independent audits had priced its bare-path cost (ordinary push/pop traffic, the
speculation snapshot/truncate tail, allocator growth) at a conservative ≈8 ns and each
returned HOLD alone; fused as one representation fix, the generated bare path now pushes
**ID-only frames** and reconstructs a name through the parser's own `RuleId`→name table
only when an error or diagnostic actually needs one — while protocol parses keep full
name frames verbatim, and every speculation restores each stack to its own snapshot so
mixed-depth stacks stay sound. The corpus geomean dropped **−3.8%** with zero verdict
flips — every bench pattern faster, and for once the magnitude sat clearly outside the
noise span.

The fourth fix closed the bare path's last dead diagnostic work — behind a design
decision rather than a code trick. The generated speculation wrapper still snapshotted
the transactional-coverage stack length on every bare attempt (provably a no-op there:
every coverage push is gated on the coverage opt-in, and a covered parse never runs the
fused graph), and the hot rollback still maintained one purely-diagnostic
classification counter. Removing the counter touched a public surface — the store's
cumulative counters are readable after any parse — so the fix first wrote the
**observed-parse boundary** down as a documented rule (diagnostic classifications are
exact on observed parses; a bare parse may skip them; soundness-bearing counters are
exact always — the same rule the per-rule entry counters already follow), then landed
the elision under it. The corpus geomean dropped **−1.1%** with zero verdict flips —
inside the noise span, accepted by the same strict same-session rule that adjudicated
the second fix, and in line with its own sub-noise pre-registered pricing. An honest bookkeeping
note travels with it: the recorded floor number *rose* slightly because the measurement
session ran ≈2% hot (a proven-neutral rebuild of the baseline plus the very same binary
re-read moved +2% across sessions with no accepted performance change); the fix's own
same-session comparison is what the gate adjudicates.

The scoreboard now reads **496 µs → ≈1.81 µs, about 274×** on the bench geomean, with the
corpus maximum at ≈484 µs and the corpus geomean at ≈1.16 µs. The method decides — and
the story continues here.
