# `ENGINE-UNIVERSAL-SERVICES.69`(a) — the overshoot census, and the SIGABRT it found

`.69`(d) closed by measuring the stack spend at a refusal, and left `(a)` an acceptance criterion in
its own words: the overshoot past the spendable budget is **SHAPE-driven** (6 800 B to 597 280 B, an
88× spread at ONE reservation), so the 6.25 % headroom reserve must be sized against the **WORST**
shape — and *nothing measures that distribution*. Two shapes is not a distribution. This directory
measures it, and the measurement found a hard process abort at the shipped default reservation.

## Files

| file | what it is |
|---|---|
| `gen_shapes.py` | twelve deep-nesting SystemVerilog shapes, each driving recursion through a different chain of the expression/statement hierarchy |
| `validate_shapes.sh` | does the SHIPPED `sv_2017` profile accept each shape at all? Runs BEFORE the census |
| `overshoot_census.sh` | per shape, climb until the depth probe refuses on stack grounds, then report `used − budget` against the reserve |
| `overshoot_depth_sweep.sh` | hold the reservation FIXED and move the input depth — the sweep that located the abort |
| `census_debug_96MiB_shapes.txt` | the recorded shape census |
| `depth_sweep_pattern.txt` | the recorded sweeps, including the shipped-default rows |
| `preregistration_reservation_scaling.md` | the reservation-scaling prediction, written before the deciding points ran — and REFUTED |

## The three things this directory exists to say

**1. The reserve is far closer to the edge than the record said.** Over ten LRM-valid shapes the
margin `reserve ÷ overshoot` spans **716.24×** (`concat`) to **1.10×** (`pattern`, nested assignment
patterns `'{'{'{1}}}`) — an overshoot spread of **649×**, against the 88× on record, and a worst case
**10× worse** than the *"worst margin measured 11×"* `.69`(d) published.

**2. It is not a near-miss. It aborts.** Holding the reservation fixed and moving the depth, the
`pattern` margin degrades smoothly — 3.20× → 1.64× → 1.10× — and the next step is `rc 134`. At the
**shipped 256 MiB default** a debug build aborts at **768 nesting levels, a 2 344-byte file**.

**3. ⛔⛔ THE CENSUS IS CENSORED AT 1.0 BY CONSTRUCTION, AND THAT IS WHY THE SPREAD LOOKED SAFE.** The
refusal fires when `used >= budget` and the reading is printed by the process *afterwards*. A parse
that overshoots past the whole RESERVATION never reports a margin below 1 — it hits the guard page
and dies. So sub-1 cases appear only as `rc 134`, and **the measured minimum of 1.10× is the worst
case among the SURVIVORS, not a bound.** Any census of this quantity that reports only margins is
reading a truncated distribution.

## Why the shapes differ — the mechanism

`RecursionGuard::extend_depth_probe` (`rust/src/ast_pipeline/mod.rs`) sets the next sample point from
the **average** per-frame cost:

```rust
let bytes_per_frame = used / depth;                  // AVERAGE over every frame so far
let affordable = (budget - used) / bytes_per_frame;
self.max_depth = depth.saturating_add((affordable / STACK_HEADROOM_STRIDE_DIVISOR).max(1));
```

Between two samples the parse may push `affordable / 4` frames. If the frames it is *about* to push
cost more than the running average, it spends more than the stride was priced for — and nothing
re-checks until `max_depth`. ⇒ a shape with UNIFORM frames (`paren`: 15 488 B overshoot, 406× margin)
is priced correctly by an average, and a shape with HIGHLY VARIABLE frames (`pattern`: 5 701 568 B,
1.10× margin, then abort) is not. The reserve must absorb
`stride × (worst frame cost − average frame cost)`, and nothing in the derivation bounds that term.

## Running it

```bash
bash .../validate_shapes.sh ./rust/target/debug/parseability_probe          # gate first
bash .../overshoot_census.sh ./rust/target/debug/parseability_probe          # 96 MiB floor
bash .../overshoot_census.sh ./rust/target/debug/parseability_probe 268435456
PGEN_OVERSHOOT_LEVELS="512 640 704 768" \
  bash .../overshoot_depth_sweep.sh ./rust/target/debug/parseability_probe pattern 268435456
```

⚠️ Run in DEBUG deliberately: `.79`(a) slice 2b measured that debug is the binding build mode by
7.83×, per-frame cost is ~8× larger there, and the overshoot is frames-past-budget × per-frame cost —
so the margin is smallest in the mode a developer actually runs.

⛔ Two shapes (`unary_not`, `unary_neg`) are REJECTED by the shipped profile and are excluded by
`validate_shapes.sh` rather than silently carried. That is correct behaviour, not a gap: IEEE 1800
A.8.3 makes a unary operator's operand a `primary`, and `!x` is not one, so `!!x` is not LRM-valid.
⚠️ The shapes are SYNTACTIC probes. `pattern` uses an untyped `localparam` with an assignment
pattern, which is syntactically accepted and whose semantic legality is not what is being measured.
