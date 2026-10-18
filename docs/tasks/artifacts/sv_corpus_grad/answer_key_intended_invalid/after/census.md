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
| `stimuli/sv/characterization/adjudication_manifest.tsv` | `abb165841cf83cf66c077e46b1817f52283d6d762786a153a135ba25fd825b2a` |
| `stimuli/sv/characterization/positions.tsv` | `2570f719f50b930057e0bfa79e39ba64d104ff8571939837e675a43cb801d0ad` |
| `docs/tasks/artifacts/sv_corpus_grad/answer_key_intended_invalid/census.py` | `67f272648621ca328e0559d3d8087bf7fb8d3e4314036c092e50d44d091ad567` |
| `rust/target/release/parseability_probe` (parse binary) | `a666007464bccafdb6d12a8baa96d2ee2ad01bff7744f14bafa808c6c664b83d` |

