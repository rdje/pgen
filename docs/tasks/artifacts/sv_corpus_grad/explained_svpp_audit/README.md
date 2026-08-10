# `explained_svpp_*` audit — banked before/after (SV-CORPUS-GRAD `.12` → `.12a`)

Produced by `stimuli/sv/audit_explained_svpp.py`. Both snapshots are a **full census** of the
`divergence:explained_svpp_*` population at their respective HEAD — never a sample.

| dir | HEAD | rows audited | disproven | what it shows |
|---|---|---:|---:|---|
| `before/` | `de9003ce` (`PGEN-SV-CORPUS-GRAD-0200`) | 1459 | **26** | the label decided by a whole-file existence test |
| `after/` | this commit (`.12a`) | 1433 | **0** | the same label decided positionally |

**Read the pair, not either half.** `before/` is the finding: 26 rows carried a preprocessor label
while the parser demonstrably stopped somewhere svpp cannot reach or alter. `after/` is the proof
that the fix landed and is complete — the disproven class is empty, and the row count fell by
exactly the 26 that moved to `divergence:unexplained_rejects_valid`.

⛔ **`after/` being clean is not a claim that every remaining row is verified.** 1170 rows are
positively corroborated (the parse stops *on* a macro use, an `` `include ``, or a conditional);
263 are undecidable from position alone and are owned by `.12b`, which is blocked by construction
on `SVPP-EXPANSION` — the only instrument that can settle them. The honest axis-2 statement is a
floor of 319 and a ceiling of 582, not a point.

A re-run with the tool's default `--outdir` writes `audit.tsv`/`summary.md` **here at the parent**,
which is the live/scratch run; these two subdirectories are the banked evidence and are not
overwritten by it. Each `summary.md` carries its own instrument-identity triple — re-hash before
quoting either.
