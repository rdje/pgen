# `ENGINE-UNIVERSAL-SERVICES.65` — corpus A/B, both released families

Produced by `verdict_census.sh` (this directory) run twice with the SAME harness and the
same file lists, differing only in the parser binary:

* **before** — `rust/target/release/parseability_probe` copied to
  `tmp/eus65/parseability_probe.before` BEFORE the first source edit of the slice.
* **after** — the same target rebuilt from the post-`.65` sources and all eleven
  regenerated artifacts.

The census sets no environment variables, so it measures the FUSED graph that actually
ships. (`.61`(b)'s tracked census sets `PGEN_REPORT_MEMO_STATS`, which is a disjunct of the
`bare_parse` selector and therefore routes to the PROTOCOL graph — a different graph.)

| family | files | before | after | **differences** |
|---|---:|---|---|---:|
| systemverilog (`--profile sv_2017`) | 16 336 | 9 794 accept / 6 542 reject | 9 794 / 6 542 | **0** |
| vhdl | 13 720 | 4 370 accept / 9 350 reject | 4 370 / 9 350 | **0** |
| **total** | **30 056** | | | **0** |

`diff` over the per-file TSVs reports **0** changed lines in both families, so the zero is a
per-file identity and not a coincidence of equal totals.

⭐ **Run TWICE on the after side, and the second run is the one that counts.** The first after-census
predates a `Display`-only change to the ceiling diagnostic's wording; rather than argue that a
`Display` impl cannot move a verdict, the whole census was re-run against the FINAL shipped binary.
Both runs read 0 differences, and the boundary spot-check agrees across the two binaries (4 813
accept / 4 814 reject on each). The evidence and the artifact that ships are therefore the same
thing.

⭐ Cross-check that costs nothing and is worth recording: the BEFORE side reproduces
`.61`(b)'s tracked per-file census EXACTLY (9 794 / 6 542) although that census was taken on
the PROTOCOL graph and this one on the FUSED graph — two different graphs agreeing
file-for-file on 16 336 real files.

⛔ `--profile sv_2017` throughout, per `.62`: an unnamed SystemVerilog profile parses the
UNION of every dialect, so a census taken across profiles measures the profile rather than
the change.

## Reproduce

```bash
PGEN_PARSE_PROBE_BIN=rust/target/release/parseability_probe \
  bash docs/tasks/artifacts/eus65/verdict_census.sh systemverilog --profile sv_2017 \
  -- <file-list> out_sv.tsv
PGEN_PARSE_PROBE_BIN=rust/target/release/parseability_probe \
  bash docs/tasks/artifacts/eus65/verdict_census.sh vhdl -- <file-list> out_vhdl.tsv
```

The file lists are the third column of
`../engine_universal_services/es61_ceiling_label/contact_census_{sv_2017,vhdl}.tsv`.
