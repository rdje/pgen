# Glossary

Every term this project uses to describe its own work, with the **quantity** behind it — formula,
units, and the question it answers. A metric with no formula is a vibe.

⛔ **Two things may not share a name.** Where the repository currently violates that, the row says so
until one of them is renamed.

## Exit statuses

A parse run is reported by its **`rc`** — *return code*, the process exit status (`$?` in a shell).

| `rc` | meaning | produced by |
|---|---|---|
| `0` | the parse **accepted** | the probe |
| `1` | a **clean rejection** — the parser refused and emitted a diagnostic | the probe |
| `124` | the `timeout` cap expired | `timeout(1)`, **not** the program |
| `134` | killed by **SIGABRT** (`128 + 6`) — the process died with no answer | the OS |
| `143` | killed by **SIGTERM** (`128 + 15`) | whoever sent it |

⚠️ `rc 1` and `rc 134` are not degrees of the same thing. A rejection is the parser doing its job; an
abort takes the embedder's process down with it. ⛔ And `rc 124` is a statement about **your patience**,
never about the program — raising a cap has twice turned an apparent timeout into a measured abort.

## The parse stack

| term | quantity | what it is |
|---|---|---|
| **reservation** | `min(256 MiB, max(96 MiB, physical_memory / 64))` | the dedicated parse thread's stack size. DERIVED from the host since `ENGINE-UNIVERSAL-SERVICES.69`(a); overridden by `PGEN_PARSE_STACK_BYTES`; read it with `parseability_probe --stack-reservation` |
| **budget** | `reservation − reservation/16` (93.75 %) | the share the recursion guard is willing to spend |
| **reserve** | `reservation/16` (6.25 %) | headroom withheld to absorb whatever the parse spends past the budget |
| **reference reservation** | 256 MiB (`DEDICATED_PARSE_STACK_BYTES`) | a FIXED compile-time anchor with two jobs: the reservation every other reservation is expressed as a share OF (which is what makes `.79`'s first-sample margin invariant), and the fallback when the host's memory cannot be read. ⛔ Since `.69`(a) it is NOT the default — letting it track the derived default would restore a measured `rc 134` |
| **share divisor** | 64 (`PARSE_STACK_HOST_MEMORY_SHARE_DIVISOR`) | how much of a host's physical memory ONE parse's stack may claim. A POLICY, not a proof — chosen so `256 MiB × 64 = 16 GiB`, i.e. the law returns the historic literal exactly on the host class that literal was written for |
| **provenance** (of a reservation) | `operator_override` · `host_memory_derived` · `reference_fallback` | which of the three ways produced the resolved number. A single integer hid all three before `.69`(a) |
| **used_bytes** | measured | stack bytes spent from the thread's recorded base |
| **overshoot** | `used_bytes − budget` | how far past the spendable budget the parse got before the guard stopped it. **Signed** — a negative value is legitimate (`NoFrameAffordable`: headroom left, but not enough for one more frame) |
| **B/frame** | `used_bytes ÷ depth` | the **average** per-frame stack cost of this parse |
| **stack floor** | `base − budget` | the stack ADDRESS at which the budget is gone; the guard tests the real stack pointer against it |

## Depth

| term | what it is |
|---|---|
| **levels** | nesting levels in the **source text** — what a generator was asked for |
| **depth** / **frames** | live **parser frames**, the guard's own count |

⛔ **These are not the same number.** One source level costs many parser frames — ≈16.7 for nested
assignment patterns, ≈7.5 for nested parens, ≈1.6 for a ternary chain. A reader who takes a frame
count for source nesting concludes their file is nowhere near the limit when it is over it, which is
why the rejection diagnostic says so in its own words.

## Margin — ⚠️ two different quantities share this name

| used in | formula | question it answers |
|---|---|---|
| `ENGINE-UNIVERSAL-SERVICES.69`(a), `.85` | `reserve ÷ overshoot` | how much of the **headroom** did the parse eat? `1.0×` = all of it, one step from an abort |
| `ENGINE-UNIVERSAL-SERVICES.79`(a) | `budget ÷ (4 096 × B/frame)` | can the guard **reach** its 4 096-frame first sample before the budget runs out? |

Same word, same unit, different questions — **they are not comparable**, and prose that puts them
side by side is wrong. Renaming one is owed.

⚠️ **The first sense is censored at 1.0 by construction.** Its reading is printed *after* the refusal,
so a parse that overshoots past the whole reservation never reports a small margin — it hits the guard
page and dies. Sub-1 cases appear as `rc 134`, never as a number, so a measured minimum is the worst
case among the **survivors** and never a bound.

## Measurement vocabulary

| term | what it is |
|---|---|
| **shape** | a deep-nesting construct used as a probe (`paren`, `cast`, `concat`, `ternary`, `pattern`, `if_stmt`, …). Per-frame cost and overshoot are per-SHAPE quantities, not per-family ones |
| **cell** | one (family, profile, build-mode) triple in a census |
| **ladder** / **rung** | a doubling climb over input depth; a rung is one step. ⛔ A ladder that stops at its own last rung reports a **lower bound**, not a boundary |
| **seed** | the first rung, so a cell can start near its answer instead of climbing from the bottom |
| **binding case** | the measurement that constrains the design — the worst margin, the most expensive family, the slower build mode |
| **accept set** | the set of inputs a parser accepts. A fix that removes a crash by **narrowing** it has traded one defect for another |
| **censored** | a metric whose failing values cannot be observed by the instrument that reports it |
| **advisory vs binding** | a gate metric that is machine-dependent and does not block, versus a deterministic one that does |

## Refusal causes

`PGEN_REPORT_STACK_PROBE=1` names why the depth probe refused. Only two are facts about the **stack**;
the rest are statements about what the process could not **learn**.

| cause | meaning |
|---|---|
| `BudgetExhausted` | the spendable budget is used up — a real measurement |
| `NoFrameAffordable` | headroom remains but will not pay for one more frame |
| `ExtentUnknown` | a thread this module did not create; its stack size is unknowable here |
| `DirectionUnknown` | the stack did not grow downward as expected |
| `FrameCostUnmeasurable` | under one byte per live frame — the quotient is not honest |

⛔ **A zero byte field means NOT MEASURED, never measured-as-zero.** Which fields carry a real number
is a function of the cause alone: the probe refuses at the first thing it cannot establish, so a later
quantity was never computed rather than computed as zero.

## Reading a stack-probe row

The census tables in the task tree use these columns; see
[Diagnostic & Debug Toolbox](diagnosing-unknowns.md) for a worked example.

| column | what it is |
|---|---|
| `LEVELS` | nesting levels in the source file |
| `RC` | process exit status (above) |
| `DEPTH` | live parser frames when the probe refused |
| `CAUSE` | which refusal fired |
| `USED_BYTES` | stack bytes spent from the thread base |
| `OVERSHOOT` | `USED_BYTES − budget`, signed |
| `B_FRAME` | `USED_BYTES ÷ DEPTH`, an average |
| `MARGIN` | `reserve ÷ OVERSHOOT` |

⛔ **`RC 0` beside a recorded refusal is not a contradiction.** The columns describe the refusal
EVENT; `RC` describes the VERDICT. A parse can contact the ceiling, have the guard refuse, backtrack,
and still find an accepting derivation.
