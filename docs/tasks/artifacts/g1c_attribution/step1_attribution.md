# G1-C ATTRIBUTION AUDIT — the `-0168` addressable mass is 6.3× too large

`PGEN-RGX-0078-0171`, leaf `RGX-0078.5.j.4`, session #169, 2026-07-20.
Read-only over banked artifacts. No build, no regen, no measurement, no gate
perturbed ⇒ the `-0163` STANDING INSTRUMENT RULE holds **by construction**.

Executes the `-0170` NEXT pointer (G1-C emission, step 1 of the stacked
program) — and **stops before the chain**, because the pre-flight that the
`-0166`/`-0167` standing rule demands (a written per-parse ns estimate) does not
survive contact with the raw profile.

---

## 0. What `-0168` banked

`-0168` §3 priced G1-C (per-atom `ParseNode` materialization + arena bump-copy +
tape push) from two rows of `geomean_reprofile/selfcum_tables.txt`:

| population (self%) | sub-1µs | 1–2.5µs | 2.5–20µs |
|---|---|---|---|
| `typed_arena::ArenaT::alloc_extend` | 8.6% | 6.7% | 5.3% |
| `_platform_memmove` | 5.3% | 6.5% | 7.4% |
| **log-weighted** | | | **13.3% = 168 ns/parse** |

It was deliberately careful — it explicitly rejected the broader 27–31% alloc
cluster as containing setup/teardown/memo mass a per-atom change cannot touch.
It narrowed to two symbols. **It did not check whose callers those two symbols
have.**

## 1. WHY + WHERE the figure is wrong

Two independent defects, both of the same family: an **aggregate row read as a
mechanism**.

**(a) `ArenaT` is SEVEN monomorphizations collapsed into one row.**
`alloc_extend` is generic; the demangled summary table erases `T`. The raw call
trees carry seven distinct instantiations:

```
h0f20dcea3f658c24  h88670e1bf10cf0ff  h916724f9b9255e27  h611a89fc10d7af8d
hf241ce72116737e7  h85fa14adae63667b  h6ff79074481ee1ea
```

**(b) `alloc_extend` is not on the per-atom path at all.** Verified repo-wide:
`alloc_extend` has **exactly two call sites**, both in `NodeArena`
(`rust/src/ast_pipeline/mod.rs:790, 806`) —
`alloc_shaped_values` and `alloc_shaped_pairs`, i.e. the **shaped-VALUE arenas**
(the build pass). There are **zero** occurrences in any generated parser. The
node arena reaches it only through `typed_arena::Arena::alloc`'s
**amortized-rare slow path** (`alloc_slow_path` → `alloc_extend(iter::once(v))`,
typed-arena-2.0.2 `src/lib.rs:207`), which fires roughly once per chunk, not per
atom.

The call trees confirm it directly: **not one `cascade_match_*` frame appears as
a caller of `alloc_extend` anywhere in any band.** `cascade_match_*` is the
per-atom match pass — precisely and only what G1-C changes.

## 2. Method — caller attribution

`attrib_alloc_extend.py` / `reprice_g1c.py` parse the banked macOS `sample`
call trees (indent depth = call depth), attribute every sample of both symbols
to its **immediate caller**, and bucket callers by mechanism. Inputs are the
already-banked `geomean_reprofile/sample_band_*.txt`; band log-shares
(38.0 / 35.3 / 26.0) and self-percentages are the `-0162` banked constants.

**Self-check:** the buckets re-sum to **13.34%** against the `-0168` banked
**13.3%** — the same mass, partitioned rather than re-measured.

## 3. Where the 168 ns actually lives

| mechanism | % parse | ns/parse | status in the `-0170` stack |
|---|---|---|---|
| **BUILD/VALUE pass** (`cascade_build_*`, `to_shaped_value`) | 5.15% | 65.1 | step 4 — **but V1 is CLOSED measured-exhausted** |
| **MEMO insert** (`memoized_call`, `hashbrown::insert`) | 2.43% | 30.7 | inside BATCH-1 |
| **G1-C addressable** (`cascade_match_*`) | **2.11%** | **26.7** | **step 1 — the solo heavy chain** |
| **SEMANTIC RUNTIME** (effect directive, rule transaction, `rule_context_path`, `FactIndex`) | 1.90% | 24.1 | ⭐ **never named by any slice** |
| other / spine / misc | 1.02% | 12.9 | — |
| harness `parse_once_timed` | 0.71% | 9.0 | ⚠️ **not parser cost at all** |
| **TOTAL** | **13.34%** | **168.5** | = the `-0168` figure |

## 4. Re-priced G1-C

| | mass | vs noise floor (28.8 ns) |
|---|---|---|
| `-0168` banked | 13.30% = 168.0 ns | — |
| **`-0171` corrected** | **2.11% = 26.7 ns** | **6.3× smaller** |

| capture | saving | % geomean | vs noise |
|---|---|---|---|
| 30% | −8.0 ns | −0.63% | 0.28× |
| 50% | −13.4 ns | −1.06% | 0.46× |
| 70% | −18.7 ns | −1.48% | 0.65× |

⛔ **G1-C is BELOW the noise floor at every capture fraction.** Its banked band
of **−4.0…−9.3%** was an artifact of the mis-attribution; the honest band is
**−0.6…−1.5%**.

⚠️ Note the `-0168` "addressable" figure also contained **9.0 ns of the
measurement harness's own `memmove`** (`regex_perf_probe::parse_once_timed`) —
mass no parser change of any kind can remove.

## 5. Adjudication

**G1-C is NOT refused.** Under the `-0170` AMENDED rule, a lever is refused only
when its POPULATION does not exist. G1-C's population is real —
`cascade_match_piece` alone is 21.2% of all `memmove` samples. What is refused is
its **sequencing**: a sub-noise lever cannot earn a **solo HIGH-risk heavy
chain** (a build-pass carrier change, per `-0164`). G1-C is demoted from **step 1
of the stack** to a **BATCH-1 member**.

Had the chain run as pointed, it would have spent a full regen + two fat-LTO
probes + the complete battery on a lever whose honest ceiling is 0.65× the noise
floor — i.e. a guaranteed-unmeasurable result. That is the `-0166` outcome for
the **third** time (`-0166` paper-count → `-0168` confounded coefficient →
`-0171` collapsed aggregate), and the first time it has been caught *before* the
chain rather than after.

⛔ **BUILD/VALUE holds the largest share — and it stays CLOSED.** This audit does
NOT re-open V1. `-0156` closed it **measured-exhausted** with all three
mechanisms resolved against real A/Bs: M1 falsified (≈1 value-level clone per
parse; landed as a simplification), M2 folded into K4b, M3 falsified and
REVERTED (the transient-`Vec` round-trips are ~free under mimalloc). The banked
road fact stands: *the build-value pass's mass is legitimate conversion+arena
work already at its constant floor*, short of protocol-zone value-ization, which
is out of scope on the `.5.j.1` road. Finding mass there re-confirms V1's own
conclusion; it does not license a re-run of refuted mechanisms.

⭐ **A genuinely new population is named: the SEMANTIC RUNTIME** (24.1 ns of this
mass — comparable to G1-C's own share, and larger once its non-alloc cost is
counted). No slice in the campaign has ever priced
`apply_semantic_runtime_effect_directive` / `with_semantic_runtime_rule_transaction`
/ `rule_context_path` / `FactIndex::insert`. It is a batch candidate at minimum.

### BATCH-1, re-grounded

Batching is what the `-0170` amendment exists for, and it works here:

| batch member | caller-attributed mass |
|---|---|
| memo insert | 30.7 ns |
| G1-C per-atom | 26.7 ns |
| semantic-runtime constants | 24.1 ns |
| G1-B (`ParseError` boxing) | unpriced by these two symbols |
| C2 fact-op constants | unpriced by these two symbols |
| **combined (lower bound)** | **81.5 ns = 6.45% = 2.8× noise** |

Three individually-sub-noise levers **clear the floor at 2.8× when batched** —
exactly the compounding tail the `-0170` amendment was written to stop
discarding, now with attributed numbers behind it.

## 6. ▶️ Corrected stack

1. **SPINE-DISPATCH STEP-0** — cheap pricing slice. At 22–25% self across all
   three bands it is the largest population in the profile and still unpriced;
   this correction promotes it from step 3 to step 1, since every per-atom
   alternative has now been priced sub-noise.
2. **BATCH-1 (enlarged)** — G1-B + C2 + G1-C + memo-insert + semantic-runtime as
   ONE regen+A/B, combined estimate ≥81.5 ns (2.8× noise), landed or reverted
   **as a unit**, no per-lever perf claim banked.
3. **G3** (per-parse setup+teardown) — re-priced on whatever floor BATCH-1 leaves.
4. **BUILD-VALUE** — stays closed; re-opens only on a genuinely new mechanism.

## 7. Scope honesty

- These are shares of **two symbols only** (`alloc_extend` + `memmove`). Each
  mechanism's TOTAL cost is larger — the build pass, the memo and the semantic
  runtime all cost more than their allocation traffic. The table **re-ranks
  levers within the mass `-0168` claimed for G1-C**; it is not a full cost
  model of each mechanism.
- Consequently the BATCH-1 figure is a **lower bound** on the batch's target
  mass, and simultaneously an **upper bound on capture** (no lever captures
  100% of its population).
- The shares come from the banked 8-exemplar / 3-band sample. They are
  directional proportions, not precision figures. The structural facts (two
  `alloc_extend` call sites, zero in generated parsers, zero `cascade_match_*`
  callers) are sample-INDEPENDENT and decisive on their own.
- Floor and custody byte-untouched: bench ≈1,937.4 ns / corpus MAX 483,583 ns /
  corpus geomean 1,263.4 ns; regex `e4924024`, probe `1d3fa0ee`. No floor number
  banked. LIVE tracker unchanged.

## 8. ⛔ Standing rule added

> **No profile row may be converted into a lever's addressable mass until its
> samples have been attributed to their CALLERS.** A demangled symbol row is an
> aggregate over call sites and (for generics) over monomorphizations; a
> mechanism is a call site. `-0166` mistook instruction count for executed work,
> `-0168` mistook a collinear regressor for a marginal cost, `-0171` mistook an
> aggregate symbol for a mechanism — the same error in three costumes. Caller
> attribution is cheap, read-only, and settles it before a chain is spent.

## 9. Reproduce

```bash
python3 docs/tasks/artifacts/g1c_attribution/attrib_alloc_extend.py   # caller census
python3 docs/tasks/artifacts/g1c_attribution/reprice_g1c.py           # corrected pricing
```

Outputs banked alongside as `caller_attribution.txt` and `reprice_output.txt`.
