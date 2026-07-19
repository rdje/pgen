# RGX-0078.5.j.4 — BATCH-1 DESIGN RECORD

`PGEN-RGX-0078-0173` · session #170 · 2026-07-20 · **docs + read-only evidence only**

Executes the `-0172` NEXT pointer verbatim:

> NEXT = **BATCH-1 DESIGN SLICE** — the enlarged batch (G1-B + C2 + G1-C +
> memo-insert + semantic-runtime) needs ONE design record naming **each
> member's emission site, its mechanism, the combined acceptance band, the
> falsification bound, and a pre-adjudicated unit revert** BEFORE the single
> regen+A/B chain is spent.

---

## 0. What this slice is, and what it is not

**IS:** one design record for the enlarged BATCH-1, grounded in emission sites
counted in the shipped artifact and mechanisms read out of the actual source.

**IS NOT:** an emission. No code, no grammar, no artifact is touched. No chain
is spent. No floor number is banked. The LIVE tracker is unchanged.

**Instrument discipline.** Everything here is text matching over sources + the
shipped artifact, plus one standalone `rustc` layout probe on copied field
types. Nothing builds PGEN, runs a parser, or perturbs a gate ⇒ the **`-0163`
STANDING INSTRUMENT RULE holds BY CONSTRUCTION** (that rule forbids pricing a
fused-path lever with a runtime counter census, because observing counters
disables the fused path by design; this slice observes *text* and *type
layout*).

**Custody.** `generated/regex_parser.rs` asserted in-run at sha256 prefix
**`e4924024`** — the banked floor vintage. `site_census.sh` **refuses to
report** on mismatch (the `-0172` precedent). Floor unchanged: bench ≈1,937.4 ns
· corpus MAX 483,583 ns (settled, `-0161`) · corpus geomean 1,263.4 ns · floor
probe `1d3fa0ee`.

---

## 1. ⛔ Four corrections to banked facts

The design cannot be written on the record as it stands, because four banked
statements do not survive contact with the code. Each correction below is
reproducible from the two scripts in this directory.

### 1.1 `ParseError` is **80 bytes**, and boxing does **NOT** make it trivially droppable

`-0164` banked the G1-B mechanism as:

> `ParseError` … carries `String` + `Vec<&'static str>` + `String` … ⇒ ~80 bytes
> + drop glue … **FIX = box the cold payload ⇒ small + trivially droppable**

Measured (`carrier_layout_probe.rs`, field types copied verbatim from
`rust/src/ast_pipeline/mod.rs:654-681`):

| shape | `size_of` | `ParseResult<()>` | `needs_drop` |
|---|---|---|---|
| **A** current, inline `ContextualError` | **80** | **80** | **true** |
| **B** boxed cold payload — *the `-0164` proposal* | 32 | 32 | **true** |
| **C** index cold payload (parser-owned side table) | 32 | 32 | **false** |
| **D** index + interned `&'static str` → `u32` | **12** | **12** | **false** |

**The banked fix is half wrong.** Boxing narrows the carrier 80 → 32 but
**keeps drop glue**, because `Box` is itself an owner — so the
`drop_in_place::<Result<(),ParseError>>` call that `-0164` named as the thing to
remove **survives shape (B)**. Only shapes **(C)/(D)**, which move the cold
payload to a side table and leave the variant carrying an index, make the
carrier `Copy` and drop-free.

⇒ **G1-B's design changes from "box it" to "index it."** Shape (C) is the
recommendation: same 32-byte width as boxing, but `needs_drop = false`, which is
what the lever was actually for. Shape (D) is a further −20 bytes but requires
interning `&'static str` → `u32` across the error surface; it is recorded as a
follow-on, not part of BATCH-1.

### 1.2 The memo lever is **not** a hash lever — it is already `FxHashMap`

`-0170` listed the member as "**memo-insert (hashbrown)**", which reads as a
hash-table/hasher swap. But both memos are **already** `rustc_hash::FxHashMap`
(`ast_based_generator.rs:955` packrat, `:881` thin), and the census finds 6
`rustc_hash::FxHashMap` memo declarations. The FxHash slice **did** reach the
memos.

What the memo actually pays at its 56 hot sites is **copying**, not hashing:
each `thin_memo.insert` is preceded by **two `SmallVec::from_slice` segment
copies** (`cascade.rs:690-695`) — and the census counts **56
`SmallVec::from_slice` sites against 56 `thin_memo.insert` sites**, i.e. exactly
one pair per site.

⇒ **The memo member's mechanism is segment-copy elision, not a hasher swap.**
This is consistent with `-0172`'s finding that memory traffic is the largest
static instruction class in the spine functions (34.7–40.4%).

### 1.3 The SipHash that `-0162` found is in **`FactIndex`**, not the memo

The banked note "the `FactIndex` maps still run std's DEFAULT SipHash, the
FxHash slice never reached them" (L6592) is **correct and still true** — the
census finds **3 std `HashMap` declarations** in `FactIndex`
(`semantic_runtime.rs:2106, 2145`) while `FxHashMap` sits **imported and unused
for them in the same file**. `-0170` attached the hash concern to the wrong
member (memo instead of C2).

⇒ **The hasher swap belongs to C2, and it is LIB-ONLY (no regen).**

### 1.4 ⭐ **`rule_context_path()` is NOT trace-guarded — a banked claim is falsified**

The record banks (L6677 / L7165 / L7369):

> every `rule_context_path` consumer sits inside `pgen_trace_high!` —
> **outcome-neutral**

**Sitting inside `pgen_trace_high!` is not outcome-neutral.** The macro expands
to a **plain function call**:

```
pgen_trace_high!(..) -> pgen_trace!(level, ..)
                     -> trace_log(level, file!(), line!(), module_path!(), format_args!(..))
```

and `trace_log`'s level guard is an **early return in the function body**
(`rust/src/ast_pipeline/mod.rs:304-306`) — which runs **after** Rust has
evaluated every argument. So `self.rule_context_path()` passed as a macro
argument is evaluated **unconditionally, at every trace level, including
`none`**.

And `rule_context_path()` **always allocates**
(`semantic_runtime.rs:2595-2601`): `self.current_rule_context_stack.join(" > ")`
on the non-empty path, `"<anonymous>".to_string()` otherwise.

Census: **10 call sites; exactly 1 is genuinely guarded** (line 3293, inside an
explicit `if … && trace_enabled(High) { … }` block — whose comment credits
`RGX-0078.5.i.2 (P0)` for precisely this hardening). **The other 9 are eager**
(3139, 3428, 3605, 3754, 3770, 3794, 3892, 3980, 4017), and they sit on the
`emit_fact` path and on **six predicate-query paths**.

⇒ This is a **pure-diagnostic allocation paid on the production parse path** —
the same defect class as the landed `P-env` lever (`-0099`, a per-parse `getenv`
worth ≈3% in-metric) and the landed `P0` hygiene lever (`-0104`, −25.8%). It is
**LIB-ONLY** and it is added to BATCH-1 as a sixth member.

---

## 2. The batch members — site, mechanism, class, risk

| # | member | emission site | mechanism | class | risk |
|---|---|---|---|---|---|
| 1 | **G1-B** carrier | lib `mod.rs:655` + **emitted** `ParseError::ContextualError` ×10 in `ast_based_generator.rs` | cold payload → parser-owned side table + `u32` index ⇒ carrier 80→32 B, `needs_drop` true→**false** | lib + emitter ⇒ **regen** | LOW–MED |
| 2 | **C2** fact-ops | lib `semantic_runtime.rs:2106,2145,2154,2167,2196,2243` | `FactIndex` std `HashMap`→`FxHashMap` (3 decls); kill the per-insert **and per-query** `to_ascii_lowercase()` String alloc (26 sites) via a pre-normalized interned kind id; kill `FactNameKey::from_value`'s clone | **LIB-ONLY** | LOW |
| 3 | **G1-C** per-atom | emitter `cascade.rs:1252` (`mtb_match_atom_logic`), `scan.rs:828` (`scan_value_sequence_logic`) | elide per-atom `ParseNode` materialization + 72-B arena bump-copy + tape push; **2,853** `arena.alloc` and **268** `deriv_boundary.push` sites in the artifact | emitter ⇒ **regen** | **HIGH** (build-pass carrier) |
| 4 | **memo-insert** | emitter `cascade.rs:690-706` (56 artifact sites), `ast_based_generator.rs:8621` (1 site) | elide the **two `SmallVec::from_slice` segment copies** per thin-memo store (§1.2) — *not* a hasher swap | emitter ⇒ **regen** | MED |
| 5 | **semantic-runtime** | emitted `push_rule_context_static` ×**215** vs **284** rule methods | the rule-context `Vec<Cow>` push/pop is emitted **unconditionally on every rule entry**, including rules with zero annotations; gate it on the grammar actually consuming a context path | emitter ⇒ **regen** | MED |
| 6 | ⭐ **trace-eagerness** *(new, §1.4)* | lib `semantic_runtime.rs` ×9 eager sites | sink each `self.rule_context_path()` behind an explicit `trace_enabled(..)` check (the line-3293 pattern), so the `join`/`to_string` allocation stops firing at trace level `none` | **LIB-ONLY** | **LOW** |

Members **2** and **6** are lib-only and need **no regen**; members **1, 3, 4,
5** are emitter changes and share the batch's single regen.

---

## 3. Combined acceptance band — and the honest problem with it

### 3.1 The only attributed mass we have

`-0171` attributed the `alloc_extend` + `_platform_memmove` mass to callers:

| mechanism | share | ns | in BATCH-1? |
|---|---|---|---|
| BUILD/VALUE pass | 5.15% | 65.1 | ⛔ **NO — V1 closed, measured-exhausted (`-0156`)** |
| MEMO insert | 2.43% | 30.7 | ✅ member 4 |
| G1-C addressable | 2.11% | 26.7 | ✅ member 3 |
| SEMANTIC RUNTIME | 1.90% | 24.1 | ✅ members 5 + 6 |
| other/spine | 1.02% | — | — |
| ⚠️ harness `parse_once_timed` | 0.71% | 9.0 | ⛔ not parser cost at all |

Attributed BATCH-1 mass = **30.7 + 26.7 + 24.1 = 81.5 ns = 6.45%** of the
1,263.4 ns geomean = **2.8× the 28.8 ns noise floor**.

⚠️ **81.5 ns is a LOWER bound on target mass and an UPPER bound on capture**
(`-0171` §7) — it is shares of **two symbols only**, so each mechanism's total
cost is larger, but no lever captures 100% of its mass.

### 3.2 The band

Applying the capture fractions this campaign has used throughout (30/50/70%) to
the attributed mass:

| capture | ns | % of geomean | × noise floor | clears −2.0% land bar? |
|---|---|---|---|---|
| 30% (LOW) | 24.5 | **−1.94%** | 0.85× | ❌ **no** |
| 50% (MID) | 40.8 | **−3.23%** | 1.42× | ✅ yes |
| 70% (HIGH) | 57.1 | **−4.52%** | 1.98× | ✅ yes |

**ACCEPTANCE BAND = −1.9% … −4.5%, MID −3.2%.**

Members **1 (G1-B)**, **2 (C2)** and **6 (trace-eagerness)** contribute **zero**
to this band — they are not priced by those two symbols. They are **unpriced
upside**, and per the `-0172` rule (*naming ≠ pricing; static ≠ dynamic ≠
nanoseconds*) **no band is derived for them here, and none may be.**

### 3.3 ⛔ The problem, stated plainly

`-0170`'s amended rule requires a batch whose **combined estimate clears the
noise floor with margin**. This band **does not clear it at the low end**: at
30% capture the batch delivers 0.85× noise and misses the −2.0% land bar.

The batch is therefore **not yet safe to spend the chain on** as priced. Two of
its six members (C2, trace-eagerness) are **lib-only and cheap to price without
a chain at all**, and pricing them is exactly what moves the low end above the
floor.

⇒ **RECOMMENDATION (§5): run a cheap read-only pricing pre-flight before the
chain**, not after — the `-0171` lesson ("caught BEFORE the chain rather than
after") applied one slice earlier.

---

## 4. Falsification bound and the pre-adjudicated UNIT revert

### 4.1 Falsification bound

**If the measured combined A/B delta is worse than −2.3%** (the `-0166` noise
floor, 2.28% peak-to-peak on 8 re-runs of the same binary), the batch is
**FALSIFIED**: the combined mass model is wrong, not merely the capture
fraction, and the batch is **reverted as a unit** with **no per-lever perf claim
banked**.

Between −2.3% and −2.0% the result is **inside noise and therefore not a
measurement** — it is also reverted, because `-0170` explicitly preserves
"nothing is claimed that was not measured."

### 4.2 Pre-adjudicated UNIT revert — decided NOW, before any measurement

Recorded in advance so no post-hoc reasoning can rescue a failed batch:

1. **The batch lands or reverts AS ONE UNIT.** No member is retained on the
   grounds that it "probably helped." No per-lever perf number is banked from a
   batch result — the A/B measures the *bundle*, so only the bundle has a
   measured delta.
2. **Revert trigger** = any of: combined delta worse than −2.0%; any
   verdict flip on the 2,189-cell corpus or 39-cell ladder; any battery item
   red; corpus MAX regressing above the settled 483,583 ns; artifact growth
   > 5%.
3. **Revert mechanics** = `git checkout` the source change set, then regenerate
   all 11 artifacts and **assert byte-identity against the banked floor
   vintages** (regex `e4924024`, ebnf `c2a4290f`, the other 9 per
   `post_regen_custody.txt`). ⚠️ **The `-0166` trap is armed**: `make focus_regex`
   **silently no-ops** because make is timestamp-driven and the gates leave
   artifacts newer than their prerequisites — **only the direct canonical `-o`
   spelling restores the vintage**. Hash-tripwire immediately after.
4. **Correctness is never traded.** A member that is byte-identical-breaking is
   removed from the batch at design time, not adjudicated at land time.
5. **A reverted batch is still banked as evidence** — the measured delta, the
   attribution that predicted it, and the gap between them go into the leaf, per
   the `-0094`/`-0134` precedent (both refused-and-reverted, both banked).

---

## 5. ⭐ Recommended sequencing — split the chain, price the cheap half first

**The `-0170`/`-0171`/`-0172` pointer says "ONE regen+A/B". I am recommending a
deviation, and flagging it loudly rather than drifting silently.**

**Why:** two of the six members (C2 §1.3, trace-eagerness §1.4) are **lib-only**
— they need **no regen, no artifact-growth bar, no all-11 byte-compare**. Their
chain cost is a fraction of the emitter members'. And the band (§3.3) says the
emitter chain is *not currently justified on its own numbers*.

**Recommended two-step:**

- **BATCH-1a — LIB-ONLY (cheap chain, no regen).** Members **2** (C2 FactIndex:
  FxHash + interned kind ids) and **6** (trace-eagerness: 9 eager
  `rule_context_path()` sites). Battery: full lib suite incl. all-11
  differential equivalence + combinator + semantic + cert 0/7/42 + PCRE2 tuple +
  shape/duality/clippy, then the alternated fat-LTO 5×2000 A/B. **This is the
  first measured number for the semantic-runtime population, and it is obtained
  without spending a regen.**
- **BATCH-1b — EMITTER (the full chain).** Members **1** (G1-B as *index*, per
  §1.1), **3** (G1-C), **4** (memo segment copies), **5** (rule-context gating)
  — re-priced on the floor BATCH-1a leaves, and **entered only if that re-price
  clears the noise floor with margin**.

**Cheaper still, and strictly first — a read-only pre-flight** (no build at
all): `--dump-rule-outcome-counts-json` (TOOLBOX 3.5) reports per-parse
`store_counters`, **including predicate evaluations and facts emitted**. That
count × one `join`/`to_string` allocation prices member 6 directly, and the
facts-emitted count prices member 2's insert path.

⚠️ **Why the `-0163` instrument rule does not bite here:** that rule forbids
pricing a *fused-path frame* with a counter census, because enabling counters
routes the parse to the protocol graph. But **semantic-store event counts are
graph-invariant** — the observability twin is pinned byte-identical on verdict
and typed AST, which entails the same facts emitted and the same predicates
evaluated on both graphs. The count is a property of (grammar × input), not of
which graph executed it. *This is an argument, not a measurement; the pre-flight
slice should state it and, if cheap, cross-check one cell against the protocol
and fused paths.*

---

## 6. Chain order and battery (when the emitter chain is entered)

Per the `-0165` mandated ordering, unchanged:

1. `git apply` the change set → **dual-feature `ast_pipeline` rebuild FIRST**
   (`ebnf_dual_run` + `generated_parsers`) — the BIT-TWICE trap.
2. **Regen ALL 11** via the canonical `make focus_*`; per-parser diff audit;
   artifact growth measured against the **< 5% bar**.
3. Fixed point proven — independent regen trains byte-identical 11/11.
4. Base-vintage custody proven in-slice (stashed-source regen reproduces the
   base artifact byte-exactly).
5. **Full lib suite incl. `certified_grammars_are_byte_identical`** (the all-11
   value-shape oracle) **BEFORE** the land decision; + combinator 27/27;
   + semantic 36/36.
6. Regex cert **×3 seeds 0/7/42**, `fully_certified=true`.
7. `regex_typed_differential_gate` 8/8 → ⚠️ **armed hash tripwire immediately
   after** (the silent-restore trap, fired 16×).
8. `regex_pcre2_compile_oracle_gate` — 2,189 cases / 48 skips.
9. `ast_shape_contract_gate` 18/0; `duality_hunt_gate` no novel / no vanished;
   clippy source-strict.
10. Verdict identity — **zero flips on 2,189 corpus + 39 ladder cells**.
11. Two fat-LTO + mimalloc probe builds (distinct sha256) → base floor-validated
    best-of-3 within the ±6% gate → **alternated 5×2000 geomean-of-mins A/B off
    `1d3fa0ee`**, under `caffeinate -ims`, **ONE runner**, guard
    `--budget-mb 16384`.

---

## 7. Honest bounds

1. **No band is claimed for members 1, 2, 5 or 6.** They are unpriced upside.
   The §3.2 band rests solely on the 81.5 ns attributed to members 3, 4 and the
   semantic-runtime allocation traffic.
2. **The band's low end misses the land bar** (§3.3). Stated, not hidden.
3. **The 81.5 ns is two symbols' allocation traffic only** — an upper bound on
   capture, a lower bound on mass.
4. **G1-C and entry-count fusion are ONE lever** (`-0169` identity); they are
   never summed.
5. **BUILD/VALUE (65.1 ns, the largest row) stays CLOSED** — V1 measured-exhausted
   at `-0156`. Finding mass there re-confirms V1; it does not license re-running
   refuted mechanisms.
6. **The layout probe is a standalone `rustc` measurement on copied field
   types**, not on the compiled PGEN crate. It is exact for layout (the field
   types are identical) but does not observe PGEN's own monomorphizations. A
   `static_assert`-style `size_of` test in the emission slice should pin it
   in-tree.
7. **The census counts TEXT sites, not dynamic executions.** 2,853 `arena.alloc`
   sites is a static population; the per-parse execution count is a different
   quantity. Per `-0166`/`-0172`, **no ns figure is derived from a static
   count here.**
8. **Campaign position unchanged and still short.** Honest inventory remains
   BATCH-1 + G3 ⇒ compounded ≈−10…−13% against the −20.8% the <1 µs bar needs.
   This slice does not close that gap; it makes the batch's own numbers honest
   and adds one new lib-only member.

---

## 8. Reproduce

```bash
# emission-site census (asserts artifact custody e4924024, refuses on mismatch)
bash docs/tasks/artifacts/batch1_design/site_census.sh

# carrier layout probe
cd docs/tasks/artifacts/batch1_design
rustc -O -o /tmp/clp carrier_layout_probe.rs && /tmp/clp && rm -f /tmp/clp
```
