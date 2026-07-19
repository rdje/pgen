# Entry-count fusion STEP-1 — the lever is ALREADY SPENT; it reduces to G1-C (`PGEN-RGX-0078-0169`)

Session #168, 2026-07-20. **Read-only census — zero code, grammar, or artifact
change; no build, no regen, no gate perturbed.** Executes the `-0168` NEXT
pointer: a static, fused-path-valid census identifying which wrapper chains are
genuinely fusible, before entry-count fusion earns a heavy chain.

⛔ **Result: it does not earn one.** The lever `-0167` called "the entry-count
lever the model prices at −21% [that] has never been attempted" **has in fact
already been attempted and landed** — twice. What remains of it is G1-C.

## Method + custody

- **Authoritative instrument, not a hand-rolled one.** `ast_pipeline
  --report-fusibility-census` (`rust/src/ast_pipeline/fusibility_census.rs`,
  TOOLBOX §5.3), joined to the banked `-0162` outcome counts
  (`../geomean_reprofile/outcome_*.json`, 8 quantile exemplars, 653 entries).
  Its `INLINE-CENSUS` / `INLINE-DECISIONS` lines are computed by
  `compute_inline_decisions` — **the same function codegen consumes**, so the
  report and the emitted parser cannot drift.
- Binary: `rust/target/debug/ast_pipeline`, dual-feature verified in-run
  (`AST-PIPELINE-FEATURE-SURFACE: ebnf_dual_run=true generated_parsers=true`).
- Read-only ⇒ satisfies the `-0163` STANDING INSTRUMENT RULE by construction.
- Full output banked at `regex_census.txt` (344 lines).
- ⚠️ A first attempt used a **hand-rolled EBNF classifier**; it was discarded.
  It had two defects (unstripped `#` comments; a `(*…*)` block-comment regex
  that ate whole grammar spans, because a *regex* grammar is full of literal
  `(` and `*`) and it was **redundant with an existing authoritative tool** —
  an out-of-band validator, which this project treats as a defect by doctrine.
  Recorded so the mistake is not repeated.

## 1. ⛔ The pure-wrapper population is 8 rules — and all 8 are ALREADY INLINED

```
INLINE-CENSUS: grammar=regex rules=275 inline_eligible=205
               (pass_through=8 alternation_leaf=31 shaped=166)
```

Of 275 rules, exactly **8** are pure pass-through wrappers. Every one of them is
already emitted inline by the landed P1a emission:

```
capture_name … class_range_endpoint … class_range_simple_escape …
directive_option_named … directive_payload_digits … reference_name …
scs_capture_number … whitespace_literal          -- all: INLINED
```

There is **no untapped pure-wrapper population** to fuse.

## 2. ⛔ The rule FRAME the lever would delete is already gone on the hot path

```
CASCADE-EXPOSURE: total_entries=653 | internal=592 (90.7%) committed=491
                  discarded=101 memo_hits=0 | roots=43 | residual=18
                  | committed_floor=49
```

**90.7% of measured entries are INTERNAL to fused cascade regions**, where (per
TOOLBOX §5.3) *"internal entries' per-entry protocol is eliminated"* — the D2-B
cyclic-spine emitter is landed and every generated parser carries its full fused
graph. Only **49 of 653 entries (7.5%)** still pay protocol at all
(`committed_floor`).

This is `-0163`'s own correction, now quantified: *"in the fused path there is no
frame to delete."* Entry-count fusion, understood as *delete the rule frame*, was
**landed as P1a wrapper inlining (≈−4…−7%) and completed by D2-B cascade
fusion.** `-0162` modelled a frame cost the architecture had already removed.

⭐ **This is the same fact `-0168` found statistically, seen structurally.** The
joint fit halved the per-entry coefficient (21.28 → 10.28 ns) because roughly
half of the banked "cost of an entry" was work that no longer exists per entry.
Two independent instruments, one conclusion.

## 3. ⇒ Entry-count fusion REDUCES TO G1-C — one lever remains, not two

What a wrapper entry still costs on the fused path is its **cascade body**:
the speculation save (`position` + `deriv_events.len()` + `deriv_boundary.len()`),
`arena.alloc(child)`, and `deriv_boundary.push(...)` (`-0163` §3b, reading
`cascade_match_literal_char`). Removing a chain level removes exactly one arena
alloc + one tape push + one speculation block.

**That is G1-C's mass, by definition** — the 168 ns/parse of
`typed_arena::alloc_extend` + `_platform_memmove` priced in `-0168` §3.

⇒ The `-0168` **overlap warning upgrades to an IDENTITY.** "Entry-count fusion"
and "G1-C" are not two levers whose savings must not be summed; on the current
architecture they are **one lever approached from two directions**. Any future
slice treating them as separate is double-counting.

## 4. The residual inline surface is a code-SIZE trade, and buys the same mass

```
INLINE-DECISIONS:        decided_under_budget=128 over_budget=77
                         (expansion_cap=12 duplication_cap=192)
INLINE-EXPOSURE:         eligible_entries=389 (59.6% of total)
INLINE-EXPOSURE-DECIDED: decided_entries=216 (33.1% of total)
```

**26.5% of entries** (59.6% − 33.1%) sit on rules that are inline-*eligible* but
**over the code-size budget** (77 rules). Raising `expansion_cap`/
`duplication_cap` is the only remaining "more fusion" move — and it is a
**code-size trade against the >5% artifact-growth stop bar** (`-0164`), on a
43.7 MB artifact. Critically, those frames are *already frameless* on the fused
path (§2), so inlining them buys the cascade-body cost only — **again G1-C's
mass**, at real artifact cost. Not recommended as a distinct lever.

## 5. ▶️ ADJUDICATION

| lever | status | verdict |
|---|---|---|
| entry-count fusion (frame deletion) | **LANDED** — P1a (≈−4…−7%) + D2-B cascade fold; 8/8 pass-throughs inlined; 90.7% of entries already frameless | ⛔ **SPENT — no chain** |
| raise the inline budget | 26.5% of entries eligible-but-over-budget | ⛔ code-size trade vs the 5% bar, buys G1-C's mass ⇒ **not a distinct lever** |
| **G1-C** (per-atom node/arena/tape) | the sole surviving per-entry lever | ✅ **the only remaining chain candidate** — −50…−118 ns (1.8–4.1× noise, `-0168` §3) |
| G1-B | ≪ 1× noise | ⛔ unrunnable (`-0168`) |

**Recommendation (engineer):** G1-C is now the *only* identified per-entry lever,
and `-0168` prices it at **−4.0…−9.3% (1.8–4.1× the 2.28% noise floor)**. It is
worth one chain — but it must be entered with the honest expectation that it
**cannot close the <1 µs bar alone**, and with G1-C's `-0164` HIGH risk rating
(a build-pass carrier change) intact.

⛔ **CAMPAIGN-LEVEL CONSEQUENCE (director judgment invited).** Combining `-0168`
and this slice, the identified-lever inventory is now:

- entry-count fusion — **spent**
- G1-A — landed, measured 0
- G1-B — below noise
- G1-C — −4…−9%
- G3 fixed-cost trim (setup/teardown ≈171–306 ns) — ≈−6…−8% ceiling, unpriced

Best case, G1-C and G3 stack to roughly **−12…−17%**, against the **−20.9%**
needed to reach geomean < 1 µs. **No combination of identified levers closes the
bar.** Crossing it needs either a new mechanism class not currently on the books,
or a re-scoped closure criterion. This is a call-off-relevant finding and is
surfaced rather than absorbed.

## 6. Scope honesty

- Structural facts (275 rules, `pass_through=8`, all INLINED, budget split) are
  sample-independent — they are properties of the grammar and the emission plan.
- Entry shares (90.7% internal, 59.6% eligible, 33.1% decided) come from the
  banked 8-exemplar outcome sample (653 entries), not the 60-cell census; they
  are proportions, and the 60-cell census is not in the `3.4`/`3.5` format the
  join requires. Directionally robust, not precision figures.
- Census run for `regex` only. The all-11 breadth sweep is deliberately NOT run:
  the adjudication turns on regex-internal structure (the campaign's metric
  grammar), and `-0164` already established that static counts measure breadth,
  never per-win cost. A breadth sweep would add no adjudicative value here.
- No floor number banked; no tracker row changes.
