# Residual BATCH-1 re-price — HOLD before the regeneration chain

`PGEN-RGX-0078-0177`, leaf `RGX-0078.5.j.4`, session #170,
2026-07-20. Read-only over banked profiles; no source, emitter, generated
artifact, parser binary, or measurement was changed.

## Result

The residual emitter bundle has a mechanism-attributed target of **61.8 ns per
parse (4.89% of the 1,263.4 ns corpus geomean)**:

| member | attributed target | evidence |
|---|---:|---|
| G1-B index-side-table carrier | **4.42 ns** conservative full-corpus estimate | executed `drop_in_place::<Result<(), ParseError>>` self samples in the three real fused-path profiles |
| G1-C per-atom carrier removal | **26.7 ns** | caller-attributed `cascade_match_*` mass from `-0171` |
| memo segment-copy elision | **30.7 ns** | caller-attributed memo-insert mass from `-0171` |
| **total** | **61.8 ns** | **4.89% of the floor** |

At the campaign's standard 30/50/70% capture fractions, that target yields
**18.5 / 30.9 / 43.3 ns**, or **−1.47% / −2.45% / −3.43%**. Those savings are
**0.64× / 1.07× / 1.50×** the 28.8 ns same-binary noise span. The low case
misses; the midpoint clears the span by only **2.1 ns**, which is not practical
margin for an all-11 regeneration, two fat-LTO probes, and the full correctness
battery.

**Decision: HOLD the residual BATCH-1 implementation chain.** This does not
refuse or discard G1-B, G1-C, or memo copies: all three populations are real.
It applies the `-0170` batching rule literally—do not spend a heavy chain until
the combined estimate clears measurement noise *with margin*. The next step is
to price an independent neighbor and enlarge the measurable batch if its
mechanism survives attribution.

## Method and custody

`reprice.py` reads the three raw macOS `sample` call trees already banked by
`-0162`, asserts each file's full SHA-256, asserts its main-thread total, and
extracts the exact flat self row for the monomorphic
`drop_in_place<Result<(), ParseError>>` symbol:

| band | samples | drop self | self share | log weight |
|---|---:|---:|---:|---:|
| <1 µs | 11,875 | 43 | 0.3621% | 38.0% |
| 1–2.5 µs | 11,805 | 36 | 0.3050% | 35.3% |
| 2.5–20 µs | 10,673 | 43 | 0.4029% | 26.0% |

The covered-band normalized share is **0.3525% = 4.453 ns**. For the bundle,
the calculation deliberately uses the more conservative **0.3500% = 4.422 ns**
by leaving the excluded ≥20 µs tail unpriced at zero. The band weights are the
`-0162` corpus log weights; their 99.3% sum reflects that deliberate tail
exclusion plus one-decimal rounding.

This row is mechanism-valid in a way the collapsed generic rows rejected by
`-0171` were not: it names one concrete `Result<(), pgen::ast_pipeline::ParseError>`
monomorphization, and the samples sit directly in the generated fused match
call trees. The `-0173` layout probe independently proves why G1-B reaches it:
the current 80-byte carrier has `needs_drop=true`; boxing remains
`needs_drop=true`; only the parser-owned index-side-table shape tested there is
32 bytes and `needs_drop=false`. Making the result drop-free removes this exact
drop target. Any benefit from shrinking the 80-byte result carrier—register
pressure, spills, or moves—is deliberately **unpriced upside**, not added to
the model.

The input profiles use the preserved C1 floor probe `1d3fa0ee` and predate
G1-A. That custody remains valid for this mechanism: G1-A changed only the
successful literal helper's returned slice/panic scaffolding, left
`ParseError`, its variants, and this drop glue untouched, and did not bank a
new floor. This leaf derives a price from banked observations; it does not run
an instrument against the current fused graph, so the `-0163` observer rule
holds by construction.

## Next independently priced neighbor

Queue **G3: fixed per-parse setup/teardown** for a read-only caller/mechanism
audit. `-0162` measured an approximately 171 ns fixed intercept and separately
identified setup plus teardown at roughly 6–8% across the band profiles, but
that class is not yet a lever: it still needs concrete constructors, clones,
drops, and ownership mechanisms attributed before any nanoseconds can join a
batch. G3 is therefore the correct next pricing leaf—not a presumed win and
not permission to sum the whole fixed intercept.

## Reproduction

From the repository root:

```sh
python3 docs/tasks/artifacts/batch1_residual_reprice/reprice.py
```

The committed `reprice.txt` is that command's byte-for-byte output.
