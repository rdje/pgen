# Suite-intended-invalid `must_accept` census — SV-CORPUS-GRAD.13x

> DERIVED. Re-run: `python3 docs/tasks/artifacts/sv_corpus_grad/answer_key_intended_invalid/census.py`
>
> ⛔ **A MEASUREMENT, NOT A RULING.** Every row's `expected` / `adjudication` is read back from the
> tracked manifest; the rulings live in `stimuli/sv/adjudicate_external_corpus.py`'s pin tables and
> move only on an LRM citation (`.13k`'s discipline). This table shows the population before and
> after each ruling slice.

**Population: 1 rows** = `_bad|/error/|should_fail|_fail` on a `divergence:unexplained_rejects_valid` + `must_accept` row (1) + named EXTRA (0).

| suite | rows |
|---|---:|
| `verilator` | 1 |

| adjudication (manifest, as of this run) | rows |
|---|---:|
| `divergence:unexplained_rejects_valid` | 1 |

Tracked `positions.tsv` agrees with the live probe on **1 of 1** rows.
Edition contrast (`sv_2017` fail, `sv_2023` pass): **0** rows.

## Rows

| # | suite | relpath | line:col | sv_2017 | sv_2023 | source at furthest | upstream testimony | expected | adjudication |
|---:|---|---|---|---|---|---|---|---|---|
| 1 | `verilator` | `test_regress/t/t_assign_automatic_bad.v` | 60:5 | fail@2126 | fail@2126 | `].s_dyn <= '1;  // OK: direct nonblocking assignment to dynamically-si` | %Error: t/t_assign_automatic_bad.v:36:10: Dynamically-sized variable not allowed in continuous assignment (IEE | `must_accept` | `divergence:unexplained_rejects_valid` |

## Instrument identity

Re-hash these inputs; if any hash differs from the row below, **this report no longer
describes your tree** and the honest act is to re-measure, not to quote.

| input | sha256 |
| --- | --- |
| `stimuli/sv/characterization/adjudication_manifest.tsv` | `f50235b3a7879580dcb8bfe6a6c4beca13c8392645fb187a8d66bbb9d66e02e5` |
| `stimuli/sv/characterization/positions.tsv` | `75dd16941ff3de8113bd0f9144031e5fa8e00980384a6c4c2633636af199538d` |
| `docs/tasks/artifacts/sv_corpus_grad/answer_key_intended_invalid/census.py` | `23f68af1ded2b07a8137cbd94a445d776b6e0ecf739494a07701a53d0330e48b` |
| `rust/target/release/parseability_probe` (parse binary) | `5f670862bad6f80de1f10402ea7c85be80e23a362f73e472cf2abbe76a6c6bac` |

