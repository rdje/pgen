# G3 setup/teardown pricing — teardown is outside the metric; one virgin-reset mechanism survives

`PGEN-RGX-0078-0178`, leaf `RGX-0078.5.j.4`, session #170,
2026-07-20. Read-only over the preserved C1 probe and its three custody-pinned
band profiles. No source, emitter, generated artifact, parser binary, or
measurement changed.

## Result

The earlier “setup + teardown ≈6–8%” G3 class does **not** describe addressable
time in the closure metric:

- `regex_perf_probe::parse_once_timed` evaluates `start.elapsed()` first;
- only after the end timestamp does normal-return destruction call
  `drop_in_place<RegexParser>` and `drop_in_place<NodeArena>`;
- the three profiles spend a log-weighted **8.5216%** in those two cumulative
  teardown call sites, a false **107.662 ns** target if profile share were
  multiplied by the 1,263.4 ns floor;
- none of those destructor samples can change the duration already returned by
  the timer, so **zero teardown nanoseconds enter G3 or BATCH-1**.

One concrete in-metric setup mechanism survives: **the virgin parser is reset
as if it had already parsed**. `RegexParser::new` constructs and seeds a fresh
`SemanticRuntimeState`; the immediately following `parse()` calls
`prepare_parse_state`, snapshots its empty/preloaded facts, constructs a second
state, drops the first, and clones the same predicate-definition table a second
time. On the first parse, retaining the already-fresh state preserves both
preloaded facts and predicate definitions; subsequent parses still require the
reset.

The four exact cumulative call sites removed by a first-parse reset bypass are:

| return offset in `parse_once_timed` | call | source mechanism |
|---:|---|---|
| +3184 | `facts().to_vec()` | snapshot facts before reset |
| +3192 | `SemanticRuntimeState::new()` | build replacement state |
| +3200 | `drop_in_place<SemanticRuntimeState>` | destroy virgin state |
| +3420 | `clone_predicate_defs()` | seed replacement state |

They price at a conservative **1.7859% = 22.563 ns** over the full corpus
(excluded ≥20 µs tail assigned zero), or **1.7985% = 22.722 ns** normalized over
the covered 99.3% log weight. This is a lower bound: inline empty-map clear
checks, field copies, and state assignment instructions from the same ceremony
remain unpriced.

Adding that target to residual BATCH-1 yields **84.385 ns = 6.6792%**. At
30/50/70% capture, savings are **25.316 / 42.193 / 59.070 ns**, or
**−2.004% / −3.340% / −4.675%** and **0.879× / 1.465× / 2.051×** the 28.8 ns
noise span.

**Decision: HOLD before implementation.** The midpoint now has real margin,
but the conservative 30% case still misses the noise span by **3.484 ns**. The
`-0173` rule refused a heavy all-11 chain when the low capture case missed;
this audit applies the same rule. G1-B, G1-C, memo copies, and the G3 virgin
reset all remain real queued populations—none is discarded.

## Binary custody and metric boundary

`price_g3.py` asserts the preserved probe's full SHA-256
`1d3fa0eebb4de19ff003ace38721d80d71bd94653f8ea3801c0f29ca9581f9b8`,
the exact `parse_once_timed` symbol start, and these disassembly instructions:

- calls at `0x100002408`, `0x100002410`, `0x100002418`, and `0x1000024f4`
  map to return offsets +3184/+3192/+3200/+3420;
- `Timespec::now` at `0x10000399c` reads the end clock;
- parser and arena drops follow at `0x1000039d4` and `0x1000039f0`.

It then asserts every profile's full SHA-256, main-thread sample total, and
exact cumulative counts at all six in-metric/out-of-metric return offsets.
Profile ASLR changes absolute addresses but not offsets within the custody-pinned
binary.

The source says the same thing at a higher level:
`rust/src/bin/regex_perf_probe.rs:381-390` computes `start.elapsed()` in the
return expression while `parser` and `node_arena` are still local variables;
Rust drops those locals only after evaluating that expression. The preserved
binary proves the optimized artifact retained that ordering.

## Next independently priced neighbor

Queue the dynamic fused-spine memory audit that `-0172` explicitly required.
That slice found the fused `cascade_match_*` regions are 39.9–40.4% memory
instructions statically but correctly refused to turn static instruction mix
into time. The next leaf must attribute **time-weighted sampled instruction
addresses** to concrete loads/stores/copies and then to removable ownership or
carrier mechanisms. It must not sum the whole memory class, must exclude G1-C
and memo-copy overlap already in BATCH-1, and must produce a new neighbor only
where a named mechanism survives.

## Reproduction

From the repository root on macOS:

```sh
python3 docs/tasks/artifacts/g3_setup_teardown_pricing/price_g3.py
```

The committed `price_g3.txt` is that command's byte-for-byte output.
