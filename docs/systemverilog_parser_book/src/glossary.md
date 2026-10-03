# Glossary

Terms used throughout this book.

## AST envelope
The JSON tree that `parse_grammar_profile_ast_dump_named` returns. See [AST Envelope Structure](ast-envelope.md).

## AST shape contract manifest
The file `rust/test_data/ast_shape_contract/systemverilog_v1.json`. Records per-rule expected JSON shape for each entry in the test corpus. Drift in the AST dump fails a `cargo test` regression-lock test, surfacing the change.

## Annotation campaign
The systematic effort to add return annotations (`-> ...`) to grammar rules in `grammars/systemverilog.ebnf`. Each slice annotates one rule (or a small family) and bumps the schema version. The regex parser campaign (42+ slices) is the model for the SV campaign.

## ParseContent
The internal AST-pipeline representation used by PGEN's parser hooks and codegen. Variants include `Terminal`, `Sequence`, `Quantified`, `Or`, `Alternative`, `Json`. See [Parse Content Variants](parse-content-variants.md).

## parseability_probe
The CLI wrapper around `pgen::embedding_api` used for terminal-side verification, AST inspection, and bug-report reproducers. 4 sub-commands: `--supports`, `--parse`, `--parse-dump-ast`, `--parse-dump-ast-pretty`. See the [`parseability_probe` CLI Reference](../../reference/PARSEABILITY_PROBE.md) for the full flag set, exit codes, output formats, and registered grammars.

## Profile
A named configuration of the grammar that selects which top-level entry rule to start parsing from, and which profile-gated rules are active. SV profiles: `sv_2017`, `sv_2023`, `verilog_2005`, and — opt-in, since `1.0.249` — `sv_2023_tolerant` (`sv_2023` plus a closed list of four simulator-tolerated spellings; see *Tool tolerance*). The three edition profiles are strict; nothing selects the tolerant one unless the caller names it. The accepted request *spellings* on top of the canonical names (`2017`/`ieee1800-2017`/`ieee_1800_2017` → `sv_2017`, the `2023` group → `sv_2023`, the `1364-2005` group → `verilog_2005`) are declared by the grammar itself via `@profile_alias` in `grammars/systemverilog.ebnf` (since `PROFILE-ALIAS.2`, 2026-07-07) — the generated parser carries them in its `GRAMMAR_PROFILE_ALIASES` constant and resolves them case-insensitively inside `set_grammar_profile`, so every entry point (CLI, registry, embedding) accepts the same spellings. An undeclared spelling passes through un-coerced and simply matches no `@profiles` list.

`verilog_2005` (aliases `1364-2005` / `ieee1364-2005` / `ieee_1364_2005`; introduced parser-side by `VERILOG-2005-PROFILE.2`, embedding API `1.3.0`) is a strict IEEE 1364-2005 (Verilog) parsing profile on the SystemVerilog grammar: shared core constructs ride the `sv_2017` baseline, and SystemVerilog-only constructs are rejected. It is realized purely by `@profiles: [...]` gating on the one grammar — no separate parser. The profile is built to wellformedness coherence (`--lint-grammar` reports `0` `verilog_2005` profile-orphans): the Verilog-2005 core is admitted to the `sv_2017` baseline variants, the SV-only surface (classes, packages, interfaces, programs, assertions, covergroups, constraints, clocking, checkers, DPI, `typedef`, `++`/`--`, jump statements, `final`, SV-only integer atom types, `unique`/`priority`, …) is whole-rule gated out, and keyword reservation is profile-faithful (the IEEE 1364-2005 Annex B keyword set is reserved under `verilog_2005`, while SV-only reserved words such as `logic`/`class`/`typedef`/`int` parse as ordinary identifiers). So `module m; endmodule` and `wire logic;` (a net named `logic`) parse under `--profile verilog_2005` while `class C; endclass`, `int x;`, `logic x;`, `initial assert (1);`, `always_comb q = 1'b0;`, `do … while (…);`, `foreach (a[i]) …`, `shortreal r;`, `interconnect w;`, `wait fork;`, `task t (ref integer a);`, and `-> #5 e;` are rejected (each still accepted under `sv_2017`/`sv_2023`). The bare-keyword SV-only alternatives inside shared core rules are gated via shape-preserving named lifts (`always_keyword_sv_only`, `loop_statement_sv_only`, `integer_vector_type_sv_only`, `non_integer_type_sv_only`, `net_port_type_interconnect_sv_only`, `interconnect_net_declaration_sv_only`, `port_direction_sv_only`, `tf_port_direction_const_ref_sv_only`, `wait_statement_sv_only`, `event_trigger_control_sv_only`), plus a whole-rule gate on `interface_port_header` — the `sv_2017`/`sv_2023` ASTs are byte-identical to before, so no release/schema bump. The profile's **repo-standard conformance surface** (landed by `VERILOG-2005-PROFILE.4.3`) is `make -C rust SHELL=/bin/bash verilog_2005_conformance_gate`: against the tracked contract `rust/test_data/grammar_quality/verilog_2005_conformance_contract_v0.json` it asserts the `--lint-grammar` 0-profile-orphan lock, the curated 12-accept + 34-reject corpus (`rust/test_data/grammar_quality/verilog_2005_conformance/`) as a full per-file × per-profile matrix (138 checks across all three profiles + 2 alias-normalization probes), and the profiled certificate-coverage baseline (`--grammar-profile verilog_2005`: `total=1138 proof=2 witness=826 UNKNOWN=310`, `sample_parse_failures=0`, deterministic at seeds 0/7/42 — a baseline pin, not a closure claim: 277 of the 310 are NO-reach-path candidates under the profile, i.e. the gated SV-only surface being profile-unreachable by design). The downstream write-up is § "Dialect Profile — `verilog_2005`" in `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`; tracker status `Mostly Done`. Two pre-existing all-profile grammar defects bounding today's reject surface are ledgered as `SV-0023` (event-trigger `->>`/delay-form LRM fidelity) and `SV-0024` (un-braced multi-identifier port expressions — the bare `module m (interconnect w);` form still accepts under `verilog_2005` until it lands; the corpus pins the ANSI form).

## Reading (of the pipeline lane)
One run of the whole preprocess→parse corpus lane — its library pass and every row — under one edition profile. There are two: the primary `sv_2017` reading, over which the verdict-coverage tuple is defined, and the `sv_2023` reading (`PGEN_PIPELINE_PROFILE=sv_2023`). Each run leaves a *provenance sidecar* beside its dump: the profile, the parse binary's digest and the dumps' digests. See [The Preprocess→Parse Pipeline](preprocess-parse-pipeline.md).

## Recursive envelope
The default JSON shape produced by un-annotated rules — a recursive composition of arrays (for sequences and iterations), strings (for terminals), and matched-branch passthroughs (for alternations). See [AST Envelope Structure](ast-envelope.md).

## Return annotation
A `-> ...` clause appended to a grammar rule definition that overrides the default envelope shape with a typed JSON value. Example: `simple_identifier -> {type: "identifier", name: $1}`. See `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`.

## Schema version
Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to. Currently `1`. See [Schema Versioning](schema-versioning.md).

## Slice
The unit of work in the annotation campaign. One slice = one rule (or a small family of related rules) annotated, the manifest updated, the contract bumped, the book amended, and a regression-lock test landed.

## Stable surface
The set of public entry points downstream consumers can rely on across patch / minor releases. Defined in `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` § "Stable Integration Surface".

## Stress sample
A test input designed to exercise edge cases of the grammar — e.g. deeply nested structures, large modules, full LRM Annex-A coverage. Lives in `rust/test_data/grammar_quality/systemverilog_*` corpora.

## Tool tolerance
A spelling the commercial simulators accept that neither IEEE 1800-2017 nor 1800-2023 derives and whose clause text speaks against it (for example `foreach (a[i][j])`, or a lifetime in a method prototype). The edition profiles `sv_2017`, `sv_2023` and `verilog_2005` reject it, by the strict-LRM default. Since `1.0.249` the OPT-IN profile `sv_2023_tolerant` accepts a closed list of four such spellings and nothing else beyond `sv_2023` (a fifth, a name of three or more parts inside `binsof(…)`, is ruled a tool tolerance by `SV-CORPUS-GRAD.44` and is NOT in the list); each is pinned from both sides (rejected under the strict profiles, accepted under the tolerant one) and yields its own AST `kind`, so a consumer can tell a tolerated spelling from a standard one. The list, the standard's rewrite for each, and the kinds are in [The Json Carrier](json-carrier.md) (*The Opt-In Tolerance Profile*); a spelling joins the list only by a ruling, never because a file failed to parse.

## Typed shape
The JSON shape produced by an annotated rule. Always a JSON object with at least a `type` field and usually a `kind` discriminator. Consumers walking typed shapes dispatch on `obj["type"]` / `obj["kind"]`.
