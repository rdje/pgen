# V1 STEP-0 — the BUILD-VALUE census on the MAX cell (`PGEN-RGX-0078-0152`)

Session #159, 2026-07-19. The `-0151` re-steer's slice (1): population sizing per
build class + the clone-elision candidate mechanism + oracle obligations, BEFORE
any pricing (no number promised). The censused population is the `-0151`-named
MAX-cell top addressable: the build-value pass ~18%, concentrated
`cascade_build_value_group` ≈8.1% (`to_shaped_value` 485 + arena 194 cum).

## 0. Instrument (NEW, debug-only — the release floor is byte-untouched)

`shaped_conversion_census` (`rust/src/ast_pipeline/mod.rs`,
`#[cfg(debug_assertions)]` throughout — every counter is STRIPPED from release
binaries; the shipped floor probes cannot see this change):

- exact thread-local counts of every `ParseContent::to_shaped_value` call by
  INPUT variant + the element population of `Sequence`/`Quantified`
  conversions (each = one transient `Vec` collect + one arena `alloc_extend`);
- every `ParseContent::clone` by variant + cloned-Vec element totals
  (`Clone` is now hand-written IDENTICALLY to the former derive expansion —
  release codegen unchanged — so debug builds can attribute the clone
  population the `-0151` re-steer named);
- every arena shaped-slice / rendered-string allocation CALL (items were
  already visible via the `-0123` `NodeArena::census`).

Consumer: `regex_construction_census_probe --case-file` (NEW mode — censuses
arbitrary corpus cells, the `regex_perf_probe` corpus-case JSONL row shape,
instead of the built-in 8-pattern bench). Census cells:
`census_cases.jsonl` = the 5 worst-cell corpus members (725 / 2880 / 4674 /
6526 / 6538) + ladder `nest_80` + linear control `flat_160`. Determinism is
asserted across repeats INCLUDING the new counters (they join the probe's
`PartialEq` census record).

## 1. The mechanism, tool-verified at the emission sites (WHY + WHERE)

Three distinct build-value cost mechanisms exist in the artifact; the census
sizes each:

**(a) The `$N`-property-access CLONE artifact (`ast_return_transform.rs:413-471`,
`generate_value_extraction` → `parse_content_to_shaped_value` :109).** Every
emitted `$N` value extraction is
`let __pgen_content = match &BASE { Sequence(el) => el[N].content.clone(),
Alternative(node) => node.content.clone(), other => other.clone() };
__pgen_content.to_shaped_value(arena)`. The clone exists ONLY because the
emission routes through an owned temporary — `to_shaped_value` takes `&self`,
so every arm could convert THROUGH THE BORROW with zero clone (the owned
`_ => Terminal("<invalid_sequence_access>")` sentinel arm simply converts
in-arm). Cost per clone is variant-dependent: `Sequence`/`Quantified` = real
Vec malloc+memcpy, `TransformedTerminal` = String clone;
`Terminal`/`Shaped`/`Alternative` = pointer-cheap.

**(b) The protocol `$1`-peel WRAPPER-ALLOC artifact (e.g. the protocol
`pattern` fn, artifact :6898-6912).** The protocol zone builds
`ParseContent::Alternative(arena.alloc(child))` and the `-> $1` transform then
immediately PEELS that wrapper via `node.content.clone()` — the arena node
alloc AND the clone are both pure artifacts of composing the two emission
steps. On worst cells every eagerly-parsed nested group body crosses `pattern`
(a protocol boundary call-out), so this fires per group nesting level.

**(c) The LEGITIMATE deep conversion at Shaped-barrier boundaries (e.g.
`cascade_build_capturing_group`, artifact :423216: `body` =
`boundary_node.content.to_shaped_value(arena)`).** The group body's
protocol-built node tree is converted to a `PgenValue` exactly once at the
barrier — recursion + per-`Sequence` transient `Vec` + `alloc_extend`. No
clone; this is the conversion the representation REQUIRES unless the boundary
rules themselves become value-form (the protocol-zone value-ization road, out
of this slice's scope).

Static site inventory (`static_site_inventory.py` → `.txt`, per emitted fn):
totals `seq_peel_clone=2,119`, `alt_hop_clone=1,848`, `other_clone=1,848`,
`raw_use_clone=1,698`, `raw_best_clone=667`, `direct_fold=342` (no-clone),
`peel_walk/peel_fn=16/16`. The clone-first match is emitted in BOTH the
protocol zone and the island's per-branch `-> $1` forms (e.g.
`cascade_build_letter` 52 sites — but a `Terminal` receiver makes those
pointer-cheap); static counts size the SURFACE, the dynamic census below sizes
the POPULATION.

## 2. Dynamic populations (census output: `census_output.txt`, `census_bench.txt`;
deterministic across 3 repeats INCLUDING the counters, every cell)

Per single bare parse (the exact construction the release probe times):

| cell | to_shaped calls | shaped (Copy) | seq+quant convs | conv items | REAL Vec clones | cloned items | cheap clones | value slices | pair slices |
|---|---|---|---|---|---|---|---|---|---|
| **line_725 (MAX)** | **7,558** | 2,702 | **2,967** | 3,774 | **1,889** | 3,238 | 2,429 | 3,241 | 3,514 |
| line_2880 | 487 | 173 | 102 | 172 | 110 | 362 | 242 | 138 | 187 |
| line_4674 | 646 | 163 | 320 | 320 | 240 | 400 | 242 | 325 | 244 |
| line_6526 | 799 | 258 | 399 | 398 | 270 | 644 | 388 | 404 | 388 |
| line_6538 | 69 | 28 | 20 | 25 | 18 | 45 | 32 | 33 | 39 |
| nest_80 | 482 | 161 | 320 | 320 | 240 | 400 | 241 | 325 | 242 |
| flat_160 (control) | 320 | 160 | **0** | 0 | **0** | 0 | 1 | 5 | 161 |

(REAL Vec clones = `clone_sequence + clone_quantified` — each is a Vec
malloc+memcpy; cheap clones = `Terminal`/`Shaped`/`Alternative` pointer
copies, still a call+branch each. Full per-variant rows in the raw output.)

Findings:

1. **The MAX cell's build-value pass = ~2,967 Sequence/Quantified conversions
   averaging only 1.27 elements each** — the per-conversion CONSTANT (transient
   `Vec` malloc+collect+free + `alloc_extend` arena call) dominates the
   per-element work. Plus ~3,514 object builds (avg ~2.85 pairs — the
   `{type,kind,body}` shapes) and 7,558 conversion calls total.
2. **The clone-artifact population on the MAX cell = 1,889 real Vec clones
   (3,238 elements copied) + 2,429 cheap clones per parse.** flat_160 (linear
   control, all-barrier builds): ZERO — the population is STRUCTURE-driven
   (groups/alternations/quantifiers), exactly the worst-cell class shape.
3. **Dead build-value work is modest** (725: dead values 272/4,321 = 6.3%,
   dead pairs 1,084/10,001 = 10.8%): the pass is mostly CONSUMED work paid
   inefficiently, NOT doomed work — clone/constant elision, not doomed-build
   elision, is the lever class here.
4. Committed nodes = **0 on every cell** (`co_nod` column; refs=1 = the
   by-value root only) — corpus-wide confirmation of the `-0123`/`-0129`
   pure-value-tree fact on the worst-cell class.
5. HONEST attribution bound: the dynamic clone totals are the UNION of the
   `$N`-site clones (mechanism (a)) and the protocol memo-path clones
   (`MemoEntry` insert `Some(node.clone())` / hit `node.clone()` — same
   `ParseContent::clone`). A per-site dynamic split needs an artifact-side
   counter (a regen — out of a census slice's scope); the memo-stats lane
   (release probe, `PGEN_REPORT_MEMO_STATS=1`, banked
   `memo_stats_725.txt`) bounds the protocol boundary populations
   (`pattern`/`group`/`alternation` ≈540 successes each per sweep on 725).
   The M1 emission's own before→after re-census resolves the split
   mechanically (after M1, the residual = memo-path exactly).

## 3. Candidate mechanisms (sized, unpriced — pricing is the design slice's job)

**M1 — borrow-not-clone at every `$N` value extraction**
(`ast_return_transform.rs::generate_value_extraction` + the same match shape
in the island's per-branch `-> $1` forms). Convert THROUGH the borrow:
`match &BASE { Sequence(el) => el[N].content.to_shaped_value(arena),
Alternative(node) => node.content.to_shaped_value(arena),
other => other.to_shaped_value(arena) }` — the owned-temporary
`__pgen_content` and its clone vanish; the sentinel `_ =>` arm converts its
owned sentinel in-arm. Population killed (UPPER bound incl. memo-path, see
§2.5): up to 1,889 Vec clones + 3,238 element copies + up to 2,429 cheap
clone calls per MAX-cell parse. Semantics: `to_shaped_value` is `&self` —
value-identical BY CONSTRUCTION; artifact-changing (all 11 regenerate).

**M2 — the protocol `$1`-peel wrapper-alloc elision** (protocol `pattern`
class: `ParseContent::Alternative(arena.alloc(child))` immediately peeled by
the `-> $1` match). When the transform is a bare `$N` peel, emit the peel on
the CHILD directly — no wrapper arena node, no peel match. Population:
protocol boundary successes (≈540 `pattern` entries on 725); part of the
10,526 all-dead arena nodes. Sequenced WITH or after M1 (same emitter file,
independent mechanisms).

**M3 — Sequence-conversion constant reduction** (the transient
`Vec`-materialize-then-`alloc_extend` per conversion; `to_shaped_value`'s own
re-entrancy comment documents why the Vec exists). 2,967 conversions × avg
1.27 elements on the MAX cell — an exact-size arena reservation or an
inline-small (≤2) stack path would kill most transient Vecs. Runtime-only
(lib) BUT touches the shared conversion — needs its own design care.
RECORDED, not the first lever.

**Recommendation (one lever per slice, corpus-MAX primary):** V1 STEP-1 =
**M1 + M2 together** — one emitter surface (`ast_return_transform.rs` +
the island twin), zero-semantic-change by construction, kills the named
`-0151` population (`cascade_build_value_group`'s clone→convert chain) at
its source. M3 parked behind the re-census.

## 4. Oracle obligations for the V1 emission (stated BEFORE the design slice)

M1+M2 are ARTIFACT-CHANGING (all 11 parsers regenerate) ⇒ the FULL regen
battery: dual-feature rebuild first; lib suite (incl. ALL-11 differential
equivalence byte-identical + combinator + semantic C3-B pins); cert ×3 seeds
0/7/42 byte-exact vs pre-slice; typed-diff 8/8 (+ the silent-restore
hash-tripwire, 12 firings and counting); PCRE2 oracle tuple EXACT
`2189/1879/262/48`; shape + duality + clippy source-strict; canonical `-o`
regen spelling from `rust/` (the `-0134` path-embedding trap); corpus
verdict-identity 2,189/2,189 + ladder 39/39 on the A/B. Acceptance instrument:
corpus-MAX primary on `line_725` + bench ±2% guard + the byte-band sweep;
falsification bound to be set by the design slice from the re-profile shares
(the census deliberately promises NO number — populations ≠ nanoseconds; the
`-0142`/`-0150` frequency-priced over-delivery class cuts BOTH ways).

## 5. Verification of this slice (instrument-only, release floor untouched)

- Release binaries: every counter is `#[cfg(debug_assertions)]`-stripped; the
  hand-written `Clone` compiles to the exact derived body in release. The
  preserved floor probes are files on disk — physically untouched.
- Artifact custody: `generated/regex_parser.rs` = `ee3a3cb6` before AND after
  (nothing regenerated this slice).
- Battery: dual-feature lib suite green (see the leaf record for counts).
- Determinism: the probe asserts census+counter equality across repeats — all
  7 cells + all 8 bench patterns identical ×3.
