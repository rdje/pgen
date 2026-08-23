# Grammar Certification Status

**One question, answered for every shipped grammar: is it certified, and if not, why.**

> This page is **DERIVED BY MEASUREMENT**. The table is produced by
> [`scripts/report_grammar_certification.sh`](../../../scripts/report_grammar_certification.sh),
> which **runs the certificate-coverage oracle** against the generated parsers in the tree. Do not
> hand-edit it — re-run with `--markdown` and republish;
> `--check docs/book/src/grammar-certification-status.md` refuses when the two disagree.

## The definition

PGEN's own, from `GRAMMAR-WELLFORMED.G.4` — the linter⟷generator duality capstone — quoted from the
oracle's own help text:

> For every rule, is it covered by a verified unreachability **PROOF** or a verified reachability
> **WITNESS** (a clean diverse `--count` sample that parses through the real parser and exercises
> it)? `UNKNOWN`=0 with no failures = the objective *"trustworthy on this grammar"* number.

So a grammar is **certified** iff, for every rule, either a verified proof says no input can reach
it, or a generated sample witnesses it **through the real generated parser** — with `UNKNOWN=0`,
`sample_parse_failures=0` and `proof_reverify_failures=0`.

⛔ **It is not a correctness claim about the language.** A certified grammar can still accept invalid
input or reject valid input; that is the *corpus* axis, tracked per family in
[Roadmap and Live Status](roadmap-and-live-status.md). SystemVerilog's corpus axis currently reads
**46.3 % adjudicated** with **275** known defects.

## Status

<!-- BEGIN DERIVED: scripts/report_grammar_certification.sh --markdown -->

Measured at seed 0, `--count 40`, against the generated parsers in the tree.

| grammar | certified | coverage | notes |
|---|---|---|---|
| `json` | ✅ **yes** | 9 rules · 9 witness · 0 proof · 0 unknown | every rule proven or witnessed |
| `regex` | ✅ **yes** | 269 rules · 260 witness · 9 proof · 0 unknown | every rule proven or witnessed |
| `return_annotation` | ⛔ **no** | 35 rules · 33 witness · 0 proof · 2 unknown | **2 rule(s) UNKNOWN** — no generated sample reaches them and no proof covers them |
| `rtl_const_expr` | ✅ **yes** | 48 rules · 48 witness · 0 proof · 0 unknown | every rule proven or witnessed |
| `rtl_frontend` | ✅ **yes** | 169 rules · 168 witness · 1 proof · 0 unknown | every rule proven or witnessed |
| `semantic_annotation` | ⛔ **no** | 119 rules · 90 witness · 0 proof · 29 unknown | **29 rule(s) UNKNOWN** — no generated sample reaches them and no proof covers them |
| `systemverilog_preprocessor` | ✅ **yes** | 74 rules · 74 witness · 0 proof · 0 unknown | every rule proven or witnessed |
| `vhdl` | ✅ **yes** | 225 rules · 225 witness · 0 proof · 0 unknown | every rule proven or witnessed |
| `systemverilog` | ⚠️ **unverified** | 1385 rules · 1378 witness · 7 proof · 0 unknown (canonical unknown 11) | from the tracked union contract, **not re-measured here** (~2 min/seed × 3 seeds × 4 configs). Proof freshness vs the parser in the tree: **STALE** |

**6 of 9 shipped grammars are certified** — every rule proven or witnessed, UNKNOWN=0.

<!-- END DERIVED -->

## The two that are not certified

- **`return_annotation` — 2 unknown of 35.** Two rules are neither witnessed by a generated sample
  nor covered by a proof.
- **`semantic_annotation` — 29 unknown of 119.** Note the total moved from 115 to 119 when
  `GRAMMAR-WELLFORMED.H.16.6b` added the `map_key` rules, and unknown fell 31 → 29 across the same
  work, so this number tracks live grammar development.

## SystemVerilog, stated precisely

Its accounting is a **union over four** entry/profile configurations
(`systemverilog_file:sv_2023`, `sv_multi_entry_root:sv_2017`, `library_text:sv_2017`,
`systemverilog_parseable_file:sv_2017`), not a single run. Under that union the last recorded run
reached **0 unknown**; under the **single canonical config, 11 rules remain unknown**, and **7 rules
are credited by PROOF rather than by a generated string**.

⚠️ That result is **not verified against the parser now in the tree**: the contract pins one
`systemverilog_parser.rs` digest and the tree holds another, because engine-universal codegen moved
under it (the grammar itself did not — 0 commits). Re-running the gate is owned by
`SV-CORPUS-GRAD.13c.2x.10`.

## ⛔ Correction, 2026-08-23

The first version of this page reported **`certified_and_fresh = 0/9`** with seven families as
`NO ORACLE`. **That was wrong.** It asked *"does a `*cert*contract*.json` file exist for this
family?"* — a filesystem question — when the certificate-coverage **report is** the oracle and a
tracked contract is only a pin on top of it. Five families that certify cleanly were reported as
never scored. ⇒ **ask the instrument, not the filesystem.** The table above is now produced by
running the oracle.

## ⛔ Correction, 2026-08-23 — the second one, on the same page

The fix above shipped with a defect of its own, found the same day and owned by
`GRAMMAR-CERT-STATUS.1b`: the rewrite deleted the `--check` **implementation** but left the **flag**
in the argument parser. `--check` went on being accepted, printed a fresh table to stdout and
**exited 0 without ever opening this page** — so the sentence at the top of this page,
*"refuses when the two disagree"*, described a capability that had been deleted. Four arms measured
at that commit — this page, a page with a corrupted table, a page with no derived block, and a path
that does not exist — **all exited 0 and were mutually indistinguishable.**

⇒ **deleting an implementation while leaving its flag is worse than deleting the flag too**: an
unknown-argument error would have been loud on the very next run. Now restored, with five arms
observed: in sync → `OK`; drifted → exit 1 with a unified diff naming the row; no derived block →
refuse; missing page → refuse; an empty derivation → refuse rather than compare against nothing.

## Re-derive it yourself

```bash
bash scripts/report_grammar_certification.sh              # human-readable
bash scripts/report_grammar_certification.sh --markdown   # the table above
bash scripts/report_grammar_certification.sh --check docs/book/src/grammar-certification-status.md
```
