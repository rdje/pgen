# Suite-intended-invalid `must_accept` census — SV-CORPUS-GRAD.13x

> DERIVED. Re-run: `python3 docs/tasks/artifacts/sv_corpus_grad/answer_key_intended_invalid/census.py`
>
> ⛔ **A MEASUREMENT, NOT A RULING.** Every row's `expected` / `adjudication` is read back from the
> tracked manifest; the rulings live in `stimuli/sv/adjudicate_external_corpus.py`'s pin tables and
> move only on an LRM citation (`.13k`'s discipline). This table shows the population before and
> after each ruling slice.

**Population: 0 rows** = `_bad|/error/|should_fail|_fail` on a `divergence:unexplained_rejects_valid` + `must_accept` row (0) + named EXTRA (0).

| suite | rows |
|---|---:|

| adjudication (manifest, as of this run) | rows |
|---|---:|

Tracked `positions.tsv` agrees with the live probe on **0 of 0** rows.
Edition contrast (`sv_2017` fail, `sv_2023` pass): **0** rows.

## Rows

| # | suite | relpath | line:col | sv_2017 | sv_2023 | source at furthest | upstream testimony | expected | adjudication |
|---:|---|---|---|---|---|---|---|---|---|

## Instrument identity

Re-hash these inputs; if any hash differs from the row below, **this report no longer
describes your tree** and the honest act is to re-measure, not to quote.

| input | sha256 |
| --- | --- |
| `stimuli/sv/characterization/adjudication_manifest.tsv` | `4a5539b817d993d4eedaa4ba5db8b6fdac6e41bbfc606e1339af3bea28162d54` |
| `stimuli/sv/characterization/positions.tsv` | `c27ab38b73aab9f4ee8a04cf1361bbebe361724a8ee242ce7ec333247ef4eb9f` |
| `docs/tasks/artifacts/sv_corpus_grad/answer_key_intended_invalid/census.py` | `67f272648621ca328e0559d3d8087bf7fb8d3e4314036c092e50d44d091ad567` |
| `rust/target/release/parseability_probe` (parse binary) | `d7588dfadc1a07594afead99551ec038de07ea6d6e26fefe10926c6251fc6e2c` |

