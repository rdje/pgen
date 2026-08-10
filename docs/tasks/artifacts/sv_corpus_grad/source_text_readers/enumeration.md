# Source-text reader enumeration — `SV-CORPUS-GRAD.12c.1` deliverable 1

Measured 2026-08-11 at HEAD `d49ceeed`. Regenerate the raw census with:

```bash
grep -rn "read_to_string" rust/src/ --include=*.rs      # 45 sites
grep -rn "fs::read("      rust/src/ --include=*.rs      #  1 site
```

`.12c` required this enumeration **before** any code change, because
*"which sites read user source?"* is exactly the question a blanket find-and-replace over
`read_to_string` would answer wrongly: most of these sites read files **PGEN itself writes**, and
making those encoding-tolerant would hide genuine corruption in generated artifacts.

46 sites, three categories.

## Category A — USER SOURCE TEXT (6 readers / 7 call sites) — ⭐ in scope, fixed by `.12c.1`

Text that a human (or a vendor tool) authored and that PGEN hands to a parser or preprocessor.
These are the only sites where a non-UTF-8 byte is the *input's* property rather than a symptom of
a broken PGEN artifact.

| site | what it reads | before `.12c.1` | after |
|---|---|---|---|
| `rust/src/bin/parseability_probe.rs:504` | `--parse <file>` | `read_to_string` → hard refusal | `source_text::read_source_file` |
| `rust/src/bin/parseability_probe.rs:556` | `--parse-dump-ast[-pretty] <file>` | `read_to_string` → hard refusal | `source_text::read_source_file` |
| `rust/src/bin/generated_parse_probe.rs:46` | `--parse <file>` | `read_to_string` → hard refusal | `source_text::read_source_file` |
| `rust/src/parse_harness.rs:463` | the emitted standalone-probe template's input file | `read_to_string` → hard refusal | `pgen::source_text::read_source_file` |
| `rust/src/sv_preprocessor.rs:412` | the top file and every `` `include `` target | `fs::read` + **lossy** `from_utf8_lossy` (U+FFFD per bad byte) + `W_SVPP_NON_UTF8_SOURCE` | `source_text::decode_source_bytes` (lossless) + the same warning, now naming the encoding |
| `rust/src/main.rs:2797` / `:2802` | `--mimicry-corpus-file` / `--mimicry-corpus-lines` | `read_to_string` → hard refusal | `source_text::read_source_file` |

⭐⭐ **The enumeration paid for itself three times.**

1. `parseability_probe` and `sv_preprocessor` are both category-A readers and they had
   **contradictory** non-UTF-8 behaviour — one refuses the file, the other silently substitutes
   U+FFFD — with nothing in the repo saying so. A fix applied only at the site `.12c` named (the
   probe) would have left the mangling path in place.
2. `rust/src/bin/generated_parse_probe.rs` — one of the seven category-A sites — turned out to be
   **untracked in git and never to have been tracked**, swallowed by the depth-unanchored
   `.gitignore` pattern `generated_*.rs`. Measured: that pattern protects nothing (`generated/` is
   ignored at `.gitignore:24`, and none of its files start with `generated_`) and matches exactly
   this one file. Fixed with a targeted negation; the class — `BIN-BUILD-INTEGRITY.2`'s census is
   derived from `cargo metadata`, which reports the WORKING TREE, so it counts 19 binaries here and
   18 on a fresh clone, green both times — is routed to **`BIN-BUILD-INTEGRITY.6`**.
3. Wiring `sv_preprocessor` then exposed a **second, independent and previously unknown defect**:
   the preprocessor's line scanners re-emit with `bytes[i] as char`, so **every** non-ASCII
   character — including in files that are perfectly valid UTF-8 — is double-encoded in the
   preprocessed output (`©` → `Â©`, +1 byte per non-ASCII byte). Eight sites, all pre-existing.
   Owned by **`SV-CORPUS-GRAD.12c.2`** and pinned by two deliberately defect-asserting tests.

## Category B — GRAMMAR text (4 sites) — routed, NOT changed here

`.ebnf` source is user-authored, so it is subject to the same defect; but it is PGEN's *own* input
language, every grammar tracked in this repo is UTF-8 by construction, and no SV release claim
depends on it. Changing it is non-SV work under the standing SV lane lock, so it is **routed to a
created leaf**, per `DOCTRINE-GAP-OWNERSHIP` (*naming an owner is not routing*).

| site | what it reads |
|---|---|
| `rust/src/ebnf_frontend.rs:16` | the top-level `.ebnf` file |
| `rust/src/ebnf_frontend.rs:563` | an `@include`d `.ebnf` file |
| `rust/src/bin/ebnf_dual_run_diff.rs:288` | the `.ebnf` input of the dual-run differential |
| `rust/src/parser_registry.rs:3805` | every `grammars/*.ebnf` (standalone-annotation scan) |

Owner: **`EBNF-FRONTEND-SILENT-TRUNCATION.5`** (opened 2026-08-11 by this leaf). The fix is
mechanical once `pgen::source_text` exists — it is the same call swap, four times.

## Category C — files PGEN itself WRITES (35 sites) — ⛔ deliberately untouched

JSON manifests, generated Rust, reports, AST dumps, baselines, thresholds, contracts. These are
UTF-8 **by construction**: PGEN wrote them. A decode failure here is not "the input used another
encoding", it is *corruption or a truncated write*, and it must stay loud.

`rust/src/parse_harness.rs:333` · `rust/src/test_registry.rs:370` ·
`rust/src/parser_registry.rs:2164` · `rust/src/auto_return_annotation_shape_gate.rs:276` ·
`rust/src/ast_shape_contract.rs:260,456,531` · `rust/src/test_discovery.rs:56` ·
`rust/src/test_runner/round_trip_tests.rs:223` · `rust/src/ast_pipeline/library.rs:167,560` ·
`rust/src/main.rs:2178,4552,4558,5701,5850,6051,6075,6102,6133,6181` ·
`rust/src/ast_pipeline/grammar_wellformedness.rs:4171` ·
`rust/src/ast_pipeline/fusibility_census.rs:2439,2979,4656` ·
`rust/src/bin/coverage_gap_triage.rs:94,100,106` · `rust/src/bin/parseability_probe.rs:917` ·
`rust/src/bin/regex_construction_census_probe.rs:267` ·
`rust/src/bin/return_annotation_generated_audit.rs:30` · `rust/src/bin/perf_bench.rs:172` ·
`rust/src/bin/rtl_frontend_generated_contract_probe.rs:93` · `rust/src/bin/pgen_ast.rs:62` ·
`rust/src/bin/test_runner.rs:531`

## The arithmetic

7 (A) + 4 (B) + 35 (C) = 46 = 45 `read_to_string` + 1 `fs::read` (the `fs::read` is
`sv_preprocessor.rs:412`). Two of the six category-A **readers** — `parseability_probe` and
`main.rs` — hold two call sites each, which is why A counts 6 readers but 7 sites.
