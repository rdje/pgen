# docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the downstream integration contract for PGEN's main `systemverilog` parser family.

This is the document downstream projects such as Nexsim should read first when deciding how to embed the PGEN systemverilog parser.

## Contract Identity
- Contract version:
  - `1.0.3`
- Parser release version:
  - `1.0.3`
- Embedding API contract baseline:
  - `1.2.0`
- SystemVerilog AST-dump schema version:
  - `1`
- Last updated:
  - `2026-05-04`
- Current grammar family label:
  - `systemverilog`
- Current stable host profiles:
  - `sv_2017`
  - `sv_2023`
- Current live status:
  - Tracked in `LIVE_ACHIEVEMENT_STATUS.md`

## Current Trust Statement
- The PGEN `systemverilog` parser is **closure-grade for the current Nexsim-facing scope** when consumed through the stable `pgen::embedding_api` host surface.
- Closure is established via the family status / contract / telemetry gates listed under "Validation / Release Gates" below.
- The current sign-off bar is Nexsim-facing SystemVerilog parsing, not an open-ended promise for every imaginable SystemVerilog dialect or tool ecosystem.
- The grammar covers IEEE 1800-2017 (`sv_2017` profile) and the IEEE 1800-2023 delta (`sv_2023` profile). Both profiles share `grammars/systemverilog.ebnf` as the single source of truth.

## Companion Documentation — SystemVerilog Parser Integration mdBook
- The systemverilog-parser integration mdBook lives at `docs/systemverilog_parser_book/` and is the **canonical AST reference** for downstream consumers (Nexsim in particular).
- The book documents: build recipe, public API, the AST envelope, every annotated/un-annotated rule shape (as the annotation campaign progresses), per-feature worked examples, schema versioning, glossary, and a release-by-release index.
- Build it with `make systemverilog_parser_book_gate` (uses `mdbook build docs/systemverilog_parser_book`).
- Where the book and this contract disagree, **the contract wins** for compliance — but please report the disagreement as a documentation bug.

## Release 1.0.3 / Contract 1.0.3 Highlights — SV-Slice-3: `source_text_item` per-branch typed (`kind:` discriminator on 8 branches)

- **Annotation:** 8 per-branch annotations on `source_text_item` (lines 210-217 of `grammars/systemverilog.ebnf`):

```ebnf
source_text_item := description                       -> {kind: "description", body: $1}
                  | local_parameter_declaration semi  -> {kind: "local_parameter_declaration", body: $1}
                  | parameter_declaration semi        -> {kind: "parameter_declaration", body: $1}
                  | package_import_declaration         -> {kind: "package_import_declaration", body: $1}
                  | timeunits_declaration              -> {kind: "timeunits_declaration", body: $1}
                  | compiler_directive                 -> {kind: "compiler_directive", body: $1}
                  | comment_only_source_region         -> {kind: "comment_only_source_region", body: $1}
                  | semi                               -> {kind: "semi"}
```

- **Effect:** every item in the `systemverilog_file.source_text` array now carries an explicit `kind:` discriminator. Consumers walking `obj["source_text"]` can dispatch on `item["kind"]` instead of structural recursion to identify which top-level construct each item is.
- **`semi` branch carries no body** (it's just a stray `;` — no useful payload). The other 7 branches carry the matched sub-rule's raw envelope as `body`.
- **`local_parameter_declaration semi` and `parameter_declaration semi` branches drop the trailing `semi`** (annotation references `$1` only, not `$2`). The semicolon is a syntactic delimiter, not part of the typed shape.
- **Empirical pre/post on `module m; endmodule\n`:**

```text
# Pre-SV-Slice-3 — source_text[0] was the matched-branch shape directly:
"source_text": [
  [<description envelope — module_declaration shape>]
]

# Post-SV-Slice-3 — source_text[0] is a typed object with discriminator:
"source_text": [
  {
    "kind": "description",
    "body": [<description envelope — module_declaration shape>]
  }
]
```

- **Annotation inventory:** 11 entries (was 3). New: 8 per-branch entries on `source_text_item`. Existing: `source_text`, `systemverilog_file`, `systemverilog_parseable_file`.
- **Same accept set, same diagnostic codes.** Only the `source_text_item` shape changed.
- **`@branch_policy: priority_first` and `@priority: [24, 16, 16, 12, 10, 8, 6, 4]` preserved** — the branch-policy / priority metadata applies before annotation dispatch, no change needed.
- **Inner `description`, `local_parameter_declaration`, etc. shapes still raw envelope** — per-rule typing of those rules is a follow-up slice. The `kind:` discriminator gives consumers the dispatch hook to route to per-branch walkers; the walkers themselves currently descend the raw envelope.
- **Schema-version stays `1`** (additive — discriminator on existing branches, no new rules or accept-set change).
- **mdBook**: `changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.
- Public API surface unchanged.
- Annotation-language idiom note: per-branch `{kind: "<name>", body: $1}` is the canonical regex-campaign idiom for Or-rule per-branch typing (used in regex slices 7, 8, 14-21, etc.). Verified to work for SV grammar with metadata blocks (`@branch_policy`, `@priority`) preserved.

## Release 1.0.2 / Contract 1.0.2 Highlights — SV-Slice-2: `source_text` typed via `[$1**]` flatten-spread

- **Annotation:** `source_text := source_text_item* -> [$1**]` (line 2273 of `grammars/systemverilog.ebnf`).
- **Effect:** the `source_text` field of every typed `systemverilog_file` JSON object is now a **flat array** of `source_text_item` shapes. Pre-fix it carried the raw Quantified envelope wrapping the iteration — consumers walking `obj["source_text"]` had to descend through the Quantified wrap before reaching items. Post-fix the array is consumer-ready; iterate directly.
- **Empirical pre/post on `module m; endmodule\n`:**

```text
# Pre-SV-Slice-2 — source_text was a Quantified envelope:
{
  "type": "systemverilog_file",
  "source_text": [<Quantified-wrapped iteration of source_text_item>]
}

# Post-SV-Slice-2 — source_text is a flat array of items:
{
  "type": "systemverilog_file",
  "source_text": [<source_text_item shape>]   // length = 1 for minimal_module
}
```

- **`source_text_item` itself stays raw envelope** (Or of `description | local_parameter_declaration semi | parameter_declaration semi | package_import_declaration | bind_directive | ...`). Per-branch typing of source_text_item is a follow-up slice; this slice only flattens the parent.
- **Annotation inventory:** 3 entries (was 2). New: `source_text`. Existing: `systemverilog_file`, `systemverilog_parseable_file`.
- **Same accept set, same diagnostic codes.** Only the `source_text` array shape changed.
- **Same `expected_json_object_keys_present` and `expected_json_object_string_values`** in the manifest's `minimal_module` sample. The rule-under-test is `systemverilog_file`, whose top-level keys (`type`, `source_text`) and `type` value (`"systemverilog_file"`) are unchanged. The Slice-2 change is in the SHAPE of the `source_text` value, not its key presence — manifest's drift-status updated to `calibrated_2026_05_04_slice_2` to record the calibration.
- **Schema-version stays `1`** (additive — flat-array shape is strictly a clean-up of the raw envelope, no new keys or rules).
- **mdBook**: `changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, `rules-top-level.md` updated. `make systemverilog_parser_book_gate` green.
- Public API surface unchanged.
- Annotation-language idiom note: `[$1**]` is the canonical regex-campaign idiom for "flatten an array-shaped sub-rule reference into the enclosing array literal" — same idiom used in regex.ebnf for `concatenation = piece+ -> [$1**]` (slice 1 of the regex campaign). Verified to work for SV grammar's first array-shaped rule.

## Release 1.0.1 / Contract 1.0.1 Highlights — SV-Slice-1: `systemverilog_file` typed (dangling annotation rescued)

- **First effective annotation on the systemverilog parser.** Pre-fix `grammars/systemverilog.ebnf` carried two intended annotations that were both broken:
  1. Line 195's `-> {type: "systemverilog_file", source_text: $2}` was **dangling** — separated from its intended rule `systemverilog_file` (line 184) by a blank line + the `sv_multi_entry_root` helper rule (line 193) + another blank line. The annotation latched onto nothing effective; the parser registered 0 annotations for `systemverilog_file` and the rule's AST output stayed as the recursive-envelope `Sequence` shape.
  2. Line 200's `// -> {type: "systemverilog_parseable_file", items: $2}` had a `//` prefix (presumed by the grammar author to be a comment) but PGEN's EBNF dialect uses `#` for comments, not `//` — the `// ` was treated as literal noise and the rest of the line was parsed as a real annotation. So the `systemverilog_parseable_file` annotation was actually registered, but accidentally so.
- **Fix:** moved the dangling line-195 annotation up onto `systemverilog_file := trivia source_text trivia` (line 184) using the canonical multi-line continuation form. Removed the misleading `//` prefix from the line-200 annotation since it was effectively active anyway. Both annotations now register through the documented path.

```ebnf
# After SV-Slice-1:
systemverilog_file := trivia source_text trivia
                   -> {type: "systemverilog_file", source_text: $2}
...
systemverilog_parseable_file := trivia parseable_source_item* trivia
                             -> {type: "systemverilog_parseable_file", items: $2}
```

- **Empirical verification.** Generated the parser via `ast_pipeline grammars/systemverilog.ebnf --generate-parser --eliminate-left-recursion`, built `parseability_probe` with the `PGEN_SYSTEMVERILOG_PARSER_PATH` adapter, ran it on `module m; endmodule\n` with `--profile sv_2017`. AST root pre-fix: `{"content": {"Sequence": [...]}}` (recursive envelope). Post-fix: `{"content": {"Json": {"type": "systemverilog_file", "source_text": [...]}}}`. The annotation correctly latches and the typed shape lands.
- **Annotation inventory** (from `ast_pipeline`'s reporting): 2 entries — `systemverilog_file` and `systemverilog_parseable_file`. Was 1 entry pre-fix (only the accidentally-registered `systemverilog_parseable_file`).
- **Manifest update.** `rust/test_data/ast_shape_contract/systemverilog_v1.json` `current_content_kind` updated from placeholder `"sequence"` to calibrated `"json_object"`. `drift_status` flipped from `parser_unavailable_in_default_build_pending_first_run_calibration` to `calibrated_2026_05_04`. Layout note about line 195 dangling annotation removed (resolved by this slice). Calibration history added.
- **Annotation campaign starts here.** This is SV-Slice-1 — the first slice in the systematic return-annotation campaign on `grammars/systemverilog.ebnf`, mirroring the regex parser's 42-slice campaign. Subsequent slices will type rules one-by-one (`description`, `module_declaration`, `interface_declaration`, etc.). Each slice bumps parser release / contract version and appends a Highlights section here.
- **No accept-set change.** The grammar's accept set is unchanged — same inputs parse, same inputs reject. Only the AST shape for accepted inputs changes (recursive envelope → typed `{type, source_text}` object at the root).
- **Schema-version stays `1`.** Per the schema versioning policy, additive shape changes within a major version don't bump the schema number; the `current_content_kind` change is tracked in the per-rule manifest.
- **mdBook**: `docs/systemverilog_parser_book/src/changelog-index.md`, `schema-versioning.md`, `json-carrier.md`, and `rules-top-level.md` updated to reflect the typed shape. `make systemverilog_parser_book_gate` green.
- Public API surface unchanged.
- No SV-NNNN bug ledger entry (this is a foundation-slice annotation correctness fix, not a downstream-reported bug).

## Release 1.0.0 / Contract 1.0.0 Highlights — initial baseline (foundation deliverables landed)

- **Initial contract identity baseline.** The systemverilog parser has been part of PGEN for some time; this contract document is being upgraded from a thin "stable surface" pointer into the same release-tracked Highlights structure used by `PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`. Subsequent return-annotation slices on `grammars/systemverilog.ebnf` will each bump the parser-release / contract-version pair and append a Highlights section here.
- **mdBook scaffolded.** `docs/systemverilog_parser_book/` is the new canonical AST reference for downstream consumers. Initial chapters: welcome, build recipe, public API, AST envelope, schema versioning, changelog index, glossary. Per-rule and per-example chapters land as the annotation campaign progresses.
- **Build status.** The generated systemverilog parser is **NOT in the default `cargo test --features generated_parsers` build**. It is produced on-demand by `sv_stimuli_quality_gate` (and similar gates) into `rust/target/<gate>/work/systemverilog_parser.rs` and discarded after the gate run. Cfg `has_generated_systemverilog_parser` therefore stays off in default builds. When the parser is present (gate run or `PGEN_SYSTEMVERILOG_PARSER_PATH` override), the embedding API path is enabled and the per-family AST-shape contract test activates.
- **Schema baseline.** `1` — corresponds to the `version: 1` field in `rust/test_data/ast_shape_contract/systemverilog_v1.json`. The manifest currently carries one stub sample (`minimal_module: "module m; endmodule\n"`) calibrated against the placeholder `current_content_kind: "sequence"`. First post-foundation slice will run the parser, observe the actual content kind, and either confirm or update the manifest.
- **Annotation campaign — not yet started.** `grammars/systemverilog.ebnf` is currently un-annotated except for one commented-out trial annotation at line 200. Subsequent slices will systematically apply return annotations rule-by-rule, mirroring the regex parser campaign that produced typed shapes for 42+ regex rules. Schema-version bumps and contract Highlights sections will track each slice.
- **Public API surface unchanged.** No accept-set or diagnostic-code change in this baseline.
- **Bug ledger entries**: any released SV parser bug is tracked in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` under the `SV-NNNN` ID family. None blocking the baseline.

## Source Of Truth
- Grammar source:
  - `grammars/systemverilog.ebnf`
  - Companion (LRM-extracted reference): `grammars/systemverilog_2017_lrm_extracted.ebnf`, `grammars/systemverilog_2023_lrm_extracted.ebnf`
  - Companion (profiled wrappers): `grammars/systemverilog_lrm_profiled_generated.ebnf`, `grammars/systemverilog_lrm_profiled_wrapper.ebnf`
- Public host API:
  - `rust/src/embedding_api.rs`
- Public API contract:
  - `rust/docs/EMBEDDING_API_CONTRACT.md`
- AST shape contract manifest:
  - `rust/test_data/ast_shape_contract/systemverilog_v1.json`
- Build-time generated parser discovery:
  - `rust/build.rs`
  - `PGEN_SYSTEMVERILOG_PARSER_PATH`
- Live closure/status surface:
  - `LIVE_ACHIEVEMENT_STATUS.md`
- Reference IEEE 1800 LRM corpus (read-only):
  - `docs/systemverilog/2017/` (Annex A formal syntax, plus other annexes)
  - `docs/systemverilog/2023/` (delta + 2023-specific annexes)

## Stable Integration Surface
- Grammar family:
  - `systemverilog`
- Stable host profiles:
  - `sv_2017`
  - `sv_2023`
- Stable convenience entry points:
  - `parse_systemverilog_2017(...)`
  - `parse_systemverilog_2023(...)`
  - `parse_systemverilog_2017_ast_dump(...)`
  - `parse_systemverilog_2023_ast_dump(...)`
- Stable generic entry points:
  - `parse_grammar_profile(...)`
  - `parse_grammar_profile_result(...)`
  - `parse_grammar_profile_ast_dump(...)`
  - `parse_grammar_profile_named(...)` (string-name overload)
  - `parse_grammar_profile_named_with_limits(...)` (string-name overload with explicit limits)
- Stable diagnostics:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`
  - `E_UNSUPPORTED_PROFILE`

## Build / Availability Requirements
- Downstream consumers should treat the generated backend as required for real host integration.
- Startup should inspect `parser_embedding_api_contract().supports_systemverilog_generated_backend`.
- Build-time generated-parser discovery is mediated by `rust/build.rs`, not by direct use of internal parser modules.
- When local validation sets `PGEN_SYSTEMVERILOG_PARSER_PATH` while invoking `cargo ... --manifest-path rust/Cargo.toml`, use an absolute path or a path relative to `rust/`; `rust/build.rs` resolves that variable relative to the Rust manifest directory.
- The PGEN-side `sv_stimuli_quality_gate` make target produces the generated parser at `rust/target/sv_stimuli_quality_gate/work/systemverilog_parser.rs`. Downstream embedders that vendor this artifact should track its SHA256 against the parser-release version recorded in this contract.

## Validation / Release Gates
- Public host API stability:
  - `make -C rust SHELL=/bin/bash embedding_api_gate`
  - `make -C rust SHELL=/bin/bash nexsim_parser_embedding_contract_gate`
- Family closure / proof:
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_parser_family_status_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_parser_family_status_contract_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_combined_telemetry_contract_gate`
- Stimuli / corpus:
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_stimuli_quality_gate`
  - `make -C rust SHELL=/opt/homebrew/bin/bash sv_syntax_closure_gate`
- Documentation:
  - `make systemverilog_parser_book_gate` — builds the mdBook + verifies tracked HTML output.

## Scope / Non-Goals
- The stable contract is the host-oriented embedding surface in `pgen::embedding_api`, not internal generated parser types.
- Internal AST node types are not the downstream contract.
- The current tracked sign-off bar is Nexsim-facing SystemVerilog parsing, not an open-ended promise for every imaginable SystemVerilog dialect or tool ecosystem.
- When reporting downstream bugs, follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`; accepted released-parser bugs should then be logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.
