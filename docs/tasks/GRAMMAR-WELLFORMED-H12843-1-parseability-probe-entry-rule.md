# GRAMMAR-WELLFORMED.H.12.8.4.3.1 — `parseability_probe --entry-rule` (the entry-relative debug tool)

The carve-out from `.8.4.3`: give `parseability_probe --parse` an `--entry-rule <RULE>` flag so an
entry-relative rule (rooted under an alternate LRM start symbol such as `library_text`) can be parsed
— and its `furthest_position` / `--trace-rules` diagnostics produced — IN ISOLATION. Today the probe
always parses from the registered default entry (`systemverilog_file`), so Step 3 of the `UNKNOWN`
protocol (the scoped predicate-rejection trace) is unavailable for these rules, and they cannot be
minimally reproduced. This closes the tool gap `-0140` surfaced.

> Slice `PGEN-GRAMMAR-WELLFORMED-0142` (**CODE** — tool-build: registry detail-parse path + probe CLI).
> Status: `done` (`PGEN-GRAMMAR-WELLFORMED-0142`).
> Reads with [[reference_pgen_debug_toolbox]], [[feedback_systematically_use_debug_toolbox]],
> [[feedback_tools_first_no_guessing]] (build the tool when the toolbox can't show WHY+WHERE),
> [[feedback_ast_pipeline_parser_agnostic]]. Parent: `GRAMMAR-WELLFORMED.H.12.8.4.3` (which landed the
> `parse_full_from(entry)` dispatch this slice reuses).

## Mechanism + plan (parser-AGNOSTIC; reuses the `.8.4.3` `parse_full_from`)

The generated parser already exposes `parse_full_from(entry)` (landed in `-0141`). The probe's
`--parse` path goes `command_parse` → `parser_registry::parse_sample_detail_with_options` → the
per-grammar `ParseDetailFn` (e.g. `parse_with_systemverilog_detail_profile`, which augments the error
with `furthest_position`). That detail fn is hardwired to `parse_full_systemverilog_file`. Plan:

1. **Registry** (`parser_registry.rs`): add `entry: Option<&str>` to `ParseDetailFn` (`:114`),
   `parse_error` (`:568`), `parse_sample_detail_with_options`, and the per-grammar detail fns; each
   parses via `parser.parse_full_from(entry)` when `Some`, else its existing `parse_full_<canonical>`
   (so `None` is byte-identical — the same inertness contract as `-0141`). Preserve the SV
   `furthest_position` augmentation.
2. **Probe** (`rust/src/bin/parseability_probe.rs`): parse `--entry-rule <RULE>` into `GlobalOptions`
   (mirroring `--profile`), thread it through `command_parse` to `parse_sample_detail_with_options`,
   and add it to the usage string.
3. Scope: `--parse` only (the `furthest_position` detail path — the Step-3 debugging value). The
   AST-dump path (`--parse-dump-ast* --entry-rule`) is a trivial follow-on, deferred unless needed.

## Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — `parseability_probe --parse systemverilog lib_include.sv --profile 2017` (file = `include file_path_spec;`) ⇒ `Parser did not consume full input at position 0 [furthest_position=7]` (rejects — `systemverilog_file` has no top-level `include`); the probe had no `--entry-rule`, so the rule could not be reproduced from its real entry.
- [x] **ROOT CAUSE (WHY + WHERE)** — `parse_with_systemverilog_detail_profile` (`parser_registry.rs:620`) hardwired the detail parse to `parse_full_systemverilog_file`; the probe had no `--entry-rule`, so an entry-relative rule (rooted under `library_text`) could not be reproduced/traced in isolation.
- [x] **FIX** — registry: extracted `parse_with_systemverilog_detail_profile_entry(…, entry)` (the 2-arg registered `ParseDetailFn` delegates with `None` ⇒ byte-identical, keeps `furthest_position`); added parser-agnostic `parse_sample_detail_from_entry` (dispatches each grammar via `parse_full_from`, regex on its worker stack, `None` for `parse_full_from`-less meta-grammars). Probe: `--entry-rule RULE` (GlobalOptions field + arg parse + `command_parse` entry path + usage). Tool-build; no grammar/release/schema change.
- [x] **ADDRESSED (verified)** — A/B with the freshly-built release probe: `--parse … lib_include.sv --profile 2017 --entry-rule library_text` ⇒ `parse_full passed` (rc 0); WITHOUT `--entry-rule` ⇒ REJECT; control `module m; endmodule` from the default entry ⇒ `parse_full passed` (the `None` path unchanged).
- [x] **NO REGRESSION** — without `--entry-rule`, all existing probe behavior byte-identical (`None` path = the unchanged `parse_full_<canonical>` call); the cert witness path (`parse_and_cover`), codegen, and grammars are untouched by this slice ⇒ all cert numbers byte-identical to `-0141` by construction. `cargo test --lib --features "generated_parsers ebnf_dual_run"` = **771 passed / 0 failed / 21 ignored** (56.19s); `clippy_on_rust_change` = **source-clean** (`clippy_source_all_targets` ok). (SV external corpus uses the default parse path — unaffected; not re-run for this probe-flag slice.)
- [x] **LOCKSTEP** — book `cli-and-workflows.md` (`--entry-rule` bullet) + `parseability-probe-debug.md` (Quick Reference row + "Parse from an alternate entry rule" subsection); tree `GRAMMAR-WELLFORMED.md` (`.8.4.3.1` row → done + Decisions); `MEMORY.md` / `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `LIVE_ACHIEVEMENT_STATUS.md`. NO grammar/release/schema/ledger/contract change.
