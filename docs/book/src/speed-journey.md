# The Speed Journey: 496 µs → 18 µs

> **Part II · Inside PGEN.** This chapter is the companion to
> [Inside the Parser: Termination & Performance](inside-parser-performance.md). That
> chapter is the *mechanics* record — what each landed lever is and how it works, kept
> current as the running scoreboard. This one tells the **campaign itself**: the method
> that drove it, the arc of what landed and what was refuted, and the lessons that now
> have names. Every number here is pinned to its primary record in the task tree
> (`docs/tasks/RGX-0078.md`) and `CHANGES.md`; nothing below is a recollection.

Between sessions #90 and #121 — five calendar days — PGEN's regex parser went from a
**496 µs** geometric-mean parse over its eight-pattern benchmark to **≈18.2 µs**: about
**27× faster**, with every single landing proven **byte-identical** on its full oracle
battery before its speed number was believed. Fifteen levers landed. A dozen more were
refuted, rejected, or reverted — most of them for the price of a document rather than a
build. This chapter is the story of both lists, because the refusals are as much the
method as the landings.

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

The scoreboard reads **496 µs → ≈18.2 µs — about 27×** — with every landing
byte-identical and every refutation recorded. Against the ≤1 µs bar, the ninth re-profile
was unambiguous: no amount of further discard elimination gets there, because even the
perfect endgame lands an order of magnitude short. The remaining gap lives in what the
**committed path** executes — the per-entry rule protocol and its speculation-shaped
allocation, ~11 rule entries per input character where a hand-written parser would run
straight-line code.

The road opened next is **D2 — full cascade folding**: teach the generator to emit, for
provably simple regions of the grammar, the fused direct-coded matcher a performance
engineer would write by hand — collapsing the committed descent itself rather than
pruning failed attempts around it. It begins the way every road in this chapter began:
with a docs-and-instrument-only scout and a falsifiable price recorded before any build,
under the same hard constraint as everything before it — measurably faster and
byte-identical under the full battery, or it does not land. It may land; it may join the
refuted-roads table. Either way, the method decides — and the story will be told here.
