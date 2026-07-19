# SPINE-DISPATCH STEP-0 — the "22–25% spine dispatch" population is not dispatch

`PGEN-RGX-0078-0172`, leaf `RGX-0078.5.j.4`, session #169, 2026-07-20.
Read-only over the custody-verified release floor probe. No build, no regen, no
measurement, no gate perturbed ⇒ the `-0163` STANDING INSTRUMENT RULE holds **by
construction** (reading a binary cannot flip the observability twin).

Executes the `-0171` NEXT pointer — step 1 of the corrected stack.

---

## 0. The claim under test

`-0170` named the campaign's "biggest miss":

> `spine dispatch self` at 22–25% self across ALL THREE BANDS is the LARGEST
> single population in the `-0162` profile and NO slice has ever priced it
> (`piece` + `atom` + `parse_pattern` + `alternative` + `memoized_call` +
> `entry_concatenation`).

The population is real — those rows do sum to ≈23% self in the sub-1µs band. The
word under test is **"dispatch"**, which entered the record by reading FUNCTION
NAMES at the top of a self-time table, never by inspecting the functions.

## 1. Instrument

`otool -tV` on `preserved_probes/regex_perf_probe_c1_1d3fa0ee`, custody asserted
in-run (`sha256` prefix `1d3fa0ee`, byte-identical to the banked floor probe) —
the `-0164` instrument. Two structural tests, both decisive against the label:

- **size** — a dispatch function is tens of instructions;
- **instruction mix** — a dispatch function is dominated by compare + branch.

## 2. Result

| function | `-0162` self | bytes | instrs | mem | ctl | call | other |
|---|---|---|---|---|---|---|---|
| `cascade_match_piece` | 8.0% | 27,380 | 6,845 | **39.9%** | 25.1% | 4.8% | 30.2% |
| `cascade_match_atom{closure}` | 6.7% | 76,860 | **19,215** | **40.4%** | 22.5% | 4.4% | 32.7% |
| `memoized_call` | 5.2% | 15,664 | 3,916 | **34.7%** | 18.7% | 5.7% | 40.9% |
| `cascade_match_entry_concatenation` | 3.0% | 10,776 | 2,694 | **40.1%** | 25.2% | 4.8% | 29.9% |
| `parse_pattern` | — | 15,080 | 3,770 | **39.4%** | 18.9% | 5.0% | 36.6% |

⛔ **The label is refuted.** These are **2,694 to 19,215 instructions** each —
`cascade_match_atom{closure}` alone is a **77 KB** function. Control flow is a
**minority in every one** (18.7–25.2%), and **memory traffic is the largest
class throughout** (34.7–40.4%).

⭐ **What the population actually is:** the **fused cascade regions**. The D2-B
cyclic-spine emitter fused the rule graph, and fat-LTO then inlined the callees
into a handful of giant functions. Their "self" time is therefore **the matching
work itself** — reading input bytes, saving and restoring speculation state,
writing nodes — **not a dispatch overhead sitting on top of it.**

This is the same structural fact `-0169` found from the other side
(`CASCADE-EXPOSURE internal=592 (90.7%)`, `committed_floor=49`): 90.7% of entries
are internal to fused regions. Of course the spine functions are enormous — the
architecture deliberately put the whole parse inside them.

## 3. Adjudication

**There is no dispatch lever here**, because there is no dispatch to remove: a
parser must compare and branch, and even a physically-impossible removal of
*every* compare and branch in these functions caps at roughly
`0.22 × 23% ≈ 5%` of parse time. Any realistic capture is sub-noise.

⚠️ **This slice does NOT convert the memory-traffic share into a lever.** The
largest instruction class being loads and stores is a genuine and interesting
structural fact — it is consistent with the per-cascade-level speculation
bookkeeping (`position`, `deriv_events.len()`, `deriv_boundary.len()` save and
restore, `-0163` §3b) — but naming it and pricing it are different acts, and this
slice performs only the first.

## 4. ⚠️ Scope honesty — the `-0166` lesson applied to my own numbers

A **static instruction mix over a function body is not the dynamic retired
instruction mix, and neither one is nanoseconds.** A cold error path and a hot
inner loop weigh the same in the table above. Consequently:

- this slice **refutes a label**; it does **not** price a lever;
- **no acceptance band is derived** from these numbers, and none may be;
- pricing the spine's memory traffic requires a **dynamic** instrument
  (sampled or counted retired work attributed to the emitting construct), which
  is a separate slice with its own design.

Stating this is the whole point: `-0166` failed by converting a static
instruction count into an expected speed-up. The same conversion is available
here and is explicitly refused.

## 5. Consequence for the stack

The corrected stack loses its step 1 and is re-ordered again:

1. ~~SPINE-DISPATCH STEP-0~~ — **executed; the lever does not exist.** The
   population is real but it is the parse, not an overhead on it.
2. **BATCH-1 (enlarged)** — G1-B + C2 + G1-C + memo-insert + semantic-runtime as
   ONE regen+A/B, combined ≥81.5 ns = 2.8× noise (`-0171` §5), landed or
   reverted **as a unit**. **This is now step 1** — and it is the only remaining
   step with an attributed, floor-clearing estimate behind it.
3. **G3** (per-parse setup+teardown, ≈171–306 ns fixed cost) — re-priced on the
   floor BATCH-1 leaves; its relative value **grows** as the floor falls.
4. **BUILD-VALUE** — stays closed (V1 measured-exhausted, `-0156`).

⛔ **Campaign-level consequence, stated plainly.** Three consecutive pricing
audits (`-0168`, `-0171`, `-0172`) have each REMOVED a lever the record believed
in, and none has added one. The honest remaining inventory is **BATCH-1
(≥81.5 ns ≈ −6.4%)** and **G3 (≈−4…−7%, unpriced)** — compounded, roughly
**−10…−13%**, against the **−20.8%** the <1 µs bar needs. Per the `-0170`
mandate every gain is still to be taken and the stack still gets built; but the
identified inventory now falls **short of the bar by about half**, and closing it
requires a mechanism class that is not yet on the books. This is reported as a
measured fact for director visibility, **not** as a call-off question — the
`-0169` error is not being repeated.

## 6. Reproduce

```bash
python3 docs/tasks/artifacts/spine_dispatch_pricing/spine_instr_mix.py
```

Output banked alongside as `spine_instr_mix.txt`. Custody is asserted in-run;
the script refuses to report on a probe hash mismatch.
