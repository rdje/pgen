# docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_EXPANSION_CONTRACT.md

## Purpose
Contract the SystemVerilog **preprocessing service** — the EXPANDER that executes IEEE 1800 clause-22 directives over the
`systemverilog_preprocessor` (svpp) AST and hands a downstream project the preprocessed text, its source map, its event
stream and its diagnostics — as something a project integrates on its own terms: in-process (with an in-memory file
provider), in-memory, or as a tool; and as the same engine PGEN's own SystemVerilog parser front end runs on.

This is the contract the director asked for on 2026-10-19 (*"define the contract the downstream project on how to
integrate svpp efficiently, elegantly to their projects"*; *"another tool which can be use also in-memory or in-process
that will consume the AST from svpp and apply expansion"* — recorded verbatim in
`docs/decisions/project_svpp_expansion_is_a_service_over_the_svpp_ast.md`). It is the first row of the PGEN service
register (`docs/tasks/PGEN-SERVICES.md`, `docs/decisions/project_pgen_is_a_service_provider_shared_contracted_services.md`).

## Status — `1.0` (2026-10-20, `SVPP-EXPANSION.9.2`; drafted as `0.1` on 2026-10-19 by `.9.1`)
This document contracts a surface that is being BUILT in measured steps. Every statement below is tagged: **LIVE**
(true of the committed tree, re-verifiable by the command beside it, and — for every vocabulary and version this
document publishes — held equal to the code on every commit by `scripts/check_svpp_expansion_contract_currency.sh`,
the doctrine `SVPP-EXPANSION-CONTRACT-CURRENCY`) or **CONTRACTED FOR `.9.n`** (the step that lands it, after which the
tag is removed and the service version moves). `1.0` means: the typed Rust API, the file provider, the typed refusal,
the probe's emit flags and the currency gate exist (`.9.2`). A downstream project may code against a LIVE statement
today and may PLAN against a CONTRACTED one; it must not assume a contracted surface exists.

## Contract Identity (the service register's five columns)
- Service:
  - the **SystemVerilog preprocessing service** — layer 2 over the `systemverilog_preprocessor` family's AST (layer 1).
    Placement (a service of the `systemverilog_preprocessor` family, or a third family name) is an OPEN director
    question; the surfaces below do not depend on the answer.
- Contract (this document) version:
  - `1.0` (2026-10-20, `SVPP-EXPANSION.9.2`). `1.1` is reached with `.9.3` (the expander over the AST, schema `2`).
- Stability axes (the two lines below are the ones the currency gate reads; each is a constant in the code):
  - Service release version: `1.0.0` — `PREPROCESSING_SERVICE_VERSION` in
    `rust/src/families/systemverilog/preprocessing_service.rs`, declared to the engine as the stage's `service_version`
    (`pgen::preprocessing_stage::PreprocessingStage`) and written into the probe's `--emit-source-map` carrier. The
    runtime module's own line, independent of svpp's parser release (`1.0.9`) and of the SystemVerilog parser's: any
    change a consumer could observe in the five outputs for an unchanged input and context moves it.
  - Preprocessed-output schema version: `1` — `OUTPUT_SCHEMA_VERSION` in `rust/src/preprocessing_stage.rs`, re-exported
    as `PREPROCESSED_OUTPUT_SCHEMA_VERSION` by the service module. The shape of the five outputs: schema `1` is the
    serde form of `pgen::preprocessing_stage::StageOutput` below (as it has been since `ENGINE-UNIVERSAL-SERVICES.113`),
    carried on the wire by the probe's `--emit-source-map` carrier as `preprocessed_output_schema`; the tool's
    `--source-map-json` / `--event-log-json` / `--diagnostics-json` files are the bare arrays they always were.
    CONTRACTED FOR `.9.3`: schema `2`, the macro-expansion chain on every source-map entry.
- Neutrality tier:
  - the SOCKET is engine-neutral — `rust/src/preprocessing_stage.rs` (`PreprocessingStage`, `StageRequest`,
    `StageOutput`, `SourceMapEntry`, `StageEvent`, `StageDiagnostic`; `ENGINE-UNIVERSAL-SERVICES.113`); the IMPLEMENTATION
    is the SystemVerilog family's component (`rust/src/families/systemverilog/preprocessor.rs`, with its §22.14 keyword
    tables beside it). A second family implementing the socket changes nothing in this document's neutral parts.
- Consumers:
  - PGEN's own SystemVerilog parser front end (`parseability_probe --parse systemverilog … --preprocess`; the pipeline
    corpus lanes, `docs/systemverilog_parser_book/src/preprocess-parse-pipeline.md`); downstream projects through the
    surfaces in §"Surfaces".
- Currency gate:
  - LIVE: `scripts/check_svpp_expansion_contract_currency.sh` (doctrine `SVPP-EXPANSION-CONTRACT-CURRENCY`, run by
    `scripts/check_doctrines.sh` at every commit and by the hosted `memory-architecture-gate`) re-derives from the code
    and compares as SETS in both directions: the two versions above (A), the declared option table (B), the event kinds
    (C), the diagnostic codes (D), the refusal codes (E), the named surfaces (F), and refuses a `CONTRACTED FOR
    `.9.2`` tag that outlives its step (G); `--self-test` drives six RED arms and a GREEN control. The stage's
    legacy-flag rewrites stay held by the SystemVerilog family's gates. Re-verify:
    `bash scripts/check_svpp_expansion_contract_currency.sh && bash scripts/check_svpp_expansion_contract_currency.sh --self-test`.
- Per-family mdBook chapter:
  - `docs/systemverilog_preprocessor_parser_book/src/preprocessing-service.md` (the director's window; definitions of
    every term used here).

## The two layers and the one boundary
1. **Layer 1 — svpp**: text → the directive AST. Contracted by
   `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` (release `1.0.9`, AST-dump schema `7`,
   per-edition profiles). It decides no liveness, substitutes nothing, resolves no include.
2. **Layer 2 — the expander**: the AST + a context → the preprocessed output. LIVE: the expansion stage reads the TEXT
   with its own hand-written directive recognizer (it does not consume svpp's AST); CONTRACTED FOR `.9.3`: it consumes
   svpp's AST — an `` `include `` is parsed by svpp and spliced as a subtree (the file provider supplies the bytes), a
   condition is evaluated on the tree's three shapes (identifier, macro usage, the 1800-2023 expression), macro text is
   the tree's fragments, and edition behaviour comes from the AST's profile.
3. **One engine**: the SystemVerilog parser's front end and this service are the SAME code path. LIVE today (both call
   the expansion stage); kept under `.9.3` / `.9.4` (both call the expander). Consequence: the text and the verdict a
   downstream project receives are the ones PGEN's corpus lanes measure.

## The three consumption modes (each first-class)
| Mode | Input | Output | Status |
|---|---|---|---|
| **AST only** — the project expands itself or never expands (a linter that must see both arms of a conditional; an editor) | source text | svpp's AST | LIVE — the embedding API's generic entry points with grammar `systemverilog_preprocessor` and an edition profile (`parse_grammar_profile`, `ast_dump_grammar_profile`; `rust/docs/EMBEDDING_API_CONTRACT.md` `2.2.0`) |
| **PGEN expands** — the project compiles against the preprocessed output | source text + context | the five outputs | LIVE — in-process through the typed API (`preprocess` over paths, `preprocess_str` over text, both over a `FileProvider`: the disk or the consumer's memory) and through the neutral stage socket; as a tool through `ast_pipeline` (every output as a file) and `parseability_probe … --preprocess --emit-preprocessed OUT.sv [--emit-source-map MAP.json]` (expand, emit, then parse) |
| **Intercept** — the project walks or edits the AST (injects a define, serves an include from its own store, drops a group) and hands it to the expander | svpp's AST + context | the five outputs | CONTRACTED FOR `.9.3` (`expand(ast, ctx)`) |

## Surfaces
### LIVE — in-process, the neutral stage socket
```rust
use pgen::preprocessing_stage::{stage_for, StageRequest, StageOutput};
let stage = stage_for("systemverilog").expect("declared");   // Option<&'static PreprocessingStage>
let outputs: Vec<StageOutput> = stage.execute(&StageRequest {
    unit: Vec<PathBuf>,                  // the compilation unit's files, earlier first — macro tables carry forward (§22.5.1)
    include_dirs: Vec<PathBuf>,          // the include search directories, in order
    symbols: Vec<String>,                // predefined macros, "NAME" or "NAME=VALUE", in order — as `define before line 1
    parse_profile: Option<String>,       // the edition the EXPANDED text will be parsed under: sv_2017 | sv_2023 | verilog_2005
    options: Vec<(String, String)>,      // the declared stage options below; an undeclared key is refused by name
})?;                                     // one StageOutput per unit file, in order; Err = a refusal (the parser never runs)
outputs[i].origin_of_position(byte)      // Option<&SourceLocation>
```
Re-verify: `grep -n 'pub fn stage_for\|pub fn execute\|pub fn origin_of_position' rust/src/preprocessing_stage.rs`.

### LIVE — in-process, the typed API (`.9.2`; `pgen::families::systemverilog::preprocessing_service`)
```rust
use pgen::families::systemverilog::preprocessing_service::{
    preprocess, preprocess_str, PreprocessContext, PreprocessRefusal, PreprocessedOutput,   // the service
    FileProvider, DiskFileProvider, MemoryFileProvider, ProviderError,                        // the engine's neutral provider, re-exported
    PREPROCESSING_SERVICE_VERSION, PREPROCESSED_OUTPUT_SCHEMA_VERSION, refusal_code,          // the two axes, the code table
};
use pgen::families::systemverilog::preprocessor::RefusalKind;                                 // the refusal's kind (18 variants, `RefusalKind::ALL`)

pub fn preprocess(unit: &[&Path], ctx: &PreprocessContext) -> Result<Vec<PreprocessedOutput>, PreprocessRefusal>;
pub fn preprocess_str(name: &str, text: &str, ctx: &PreprocessContext) -> Result<PreprocessedOutput, PreprocessRefusal>;

pub struct PreprocessContext {              // Default: no profile, no defines, no include dirs, no options, the disk
    pub profile: Option<String>,            // sv_2017 | sv_2023 | verilog_2005 (and the svpp contract's aliases)
    pub defines: Vec<String>,               // "NAME" or "NAME=VALUE", in order
    pub include_dirs: Vec<PathBuf>,
    pub options: Vec<(String, String)>,     // the declared options below; an undeclared key is refused by name
    pub file_provider: Arc<dyn FileProvider>,
}
pub trait FileProvider: Send + Sync {       // path → bytes for the unit's files and every `include
    fn read(&self, path: &Path) -> Result<Vec<u8>, ProviderError>;
    fn exists(&self, path: &Path) -> bool;                               // default: read succeeds
    fn canonicalize(&self, path: &Path) -> Result<PathBuf, ProviderError>; // default: the path as given — the spelling every output names the file by
}
pub struct PreprocessRefusal { pub kind: RefusalKind, pub code: String, pub file: Option<String>, pub line: Option<usize>, pub message: String }
```
The context is a `PreprocessContext` VALUE (the table in "The context" below; `Default` is the disk and no options).
`preprocess` runs EXACTLY the code path the socket and the SystemVerilog parser's front end run — the same declared
option table, the same configuration derivation, the same per-file / per-unit entry — so its five outputs are
byte-identical to the socket's for the same unit and context (pinned by a test). `preprocess_str` is the in-memory
one-file form: `text` is served under `name` by a `MemoryFileProvider` layered over `ctx.file_provider`, so the
file's `` `include ``s come from the consumer's provider (or the disk by default), searched from `name`'s parent and
then `include_dirs`; every output names the file `name`, exactly as spelled. `Err` is a refusal: the parser never sees
the text; `message` is byte-for-byte what the stage has always printed (the probe's `preprocess_failed …` text, the
socket's `Err` chain), `kind` / `code` / `file` / `line` are attached AT THE REFUSING SITE (never parsed from the
message). Re-verify: `grep -n 'pub fn preprocess\b\|pub fn preprocess_str\|pub struct PreprocessContext\|pub struct PreprocessRefusal' rust/src/families/systemverilog/preprocessing_service.rs; grep -n 'pub trait FileProvider\|pub struct MemoryFileProvider' rust/src/preprocessing_stage.rs`.
The stage's own entries (`preprocess_systemverilog_file[_with]`, `preprocess_systemverilog_unit[_with]`, `pub` under
`preprocessor`) are the implementation the typed API and the socket share; a consumer codes to the typed API.

### LIVE — as a tool
```bash
ast_pipeline INPUT.sv --preprocess systemverilog [--profile P] [--include-dir DIR]... [--symbol NAME[=VALUE]]...
    [--unit-file FILE]... [--stage-opt KEY=VALUE]...
    --output OUT.sv --source-map-json MAP.json --event-log-json EVENTS.json --diagnostics-json DIAG.json
ast_pipeline INPUT.sv --preprocess systemverilog --list-stage-options     # the declared option table, with defaults
parseability_probe --parse systemverilog INPUT.sv --profile P --preprocess [the same context flags]   # expand THEN parse
```
```bash
parseability_probe --parse systemverilog INPUT.sv --profile P --preprocess [the same context flags] \
    --emit-preprocessed OUT.sv [--emit-source-map MAP.json]     # expand, EMIT, then parse (.9.2)
```
Exit codes of the probe: `0` accepted; `1` the parser rejected the expanded text (`furthest_position=N`,
`[origin=FILE:LINE:COL]` from the source map); `3` the expander refused (`preprocess_failed …`). `--emit-preprocessed`
writes the expanded text of `INPUT.sv` and `--emit-source-map` writes the versioned JSON carrier of the other four
outputs — `{"preprocessed_output_schema": 1, "service": "systemverilog", "service_version": "1.0.0", "source_map": […],
"events": […], "diagnostics": […], "included_files": […]}` — both BEFORE the parser runs, so a parse rejection (exit `1`)
still leaves them; a refusal (exit `3`) writes nothing; either flag without `--preprocess` is refused. The text is
byte-identical to `ast_pipeline --output`'s and the carrier's `source_map` to `--source-map-json`'s (measured 2026-10-20).
`--symbol` has the alias `--define`; the pre-`ENGINE-UNIVERSAL-SERVICES.113` spellings (`--preprocess-systemverilog`,
every `--sv-…` flag) are accepted through SystemVerilog release `1.0.247` with one deprecation line each.
Re-verify: `grep -n '"--emit-preprocessed"\|"--emit-source-map"' rust/src/bin/parseability_probe.rs`.

A by-name / JSON in-process form (`preprocess_by_name("systemverilog", json_request) -> json`) is a director question
(below); the module path of the typed API may gain a second, additive spelling with the placement answer — the names
above are the contract.

### CONTRACTED FOR `.9.3` — the expander over the AST
```rust
pub fn parse(text, profile) -> SvppAst;                              // layer 1 (the generated parser; exists today through the embedding API)
pub fn expand(ast: &SvppAst, ctx: &PreprocessContext) -> Result<PreprocessedOutput, PreprocessRefusal>;   // layer 2
// preprocess(text, ctx) == expand(&parse(text, ctx.profile), ctx)
```

## The context — a value, not a filesystem
| Member | LIVE spelling (socket / tool) | Meaning |
|---|---|---|
| edition profile | `parse_profile` / `--profile` | the edition the expanded text is parsed under (`sv_2017`, `sv_2023`, `verilog_2005`, and the aliases the svpp contract's "Editions and Profiles" lists); selects the keywords a `` `begin_keywords `` region is applied against and the three readings where IEEE 1800-2023 says more than 1800-2017 |
| predefined macros | `symbols` / `--symbol NAME[=VALUE]` (alias `--define`) | seeded as `` `define NAME VALUE `` before the first line; the redefinition policy, `` `undef `` and `` `ifdef `` see an ordinary definition |
| include directories | `include_dirs` / `--include-dir DIR` | searched in order for a relative `` `include `` |
| the unit's earlier files | `unit` / `--unit-file FILE` | preprocessed first, in order; their macro tables and open keyword regions carry into the later files; their text is not parsed by the pipeline |
| declared options | `options` / `--stage-opt KEY=VALUE` | the eight keys below; an undeclared key is refused by name |
| file provider | `file_provider` / — (every CLI reads the disk) | `path → bytes` for the unit's files and every include, plus `exists` (the include search) and `canonicalize` (the spelling every output names a file by); `DiskFileProvider` is the default, `MemoryFileProvider` serves memory over an optional fallback, a consumer's own implementation serves anything |

The declared options (LIVE; re-verify with `ast_pipeline --preprocess systemverilog --list-stage-options <file>`; the
table below is held equal to the stage's `STAGE_OPTIONS` by the currency gate, key, values and default):

| Key | Values | Default |
|---|---|---|
| `include_max_depth` | `N` | `64` |
| `include_path_policy` | `allow_absolute\|relative_only` | `allow_absolute` |
| `macro_redefine_policy` | `allow\|warn\|error` | `allow` |
| `conditional_symbol_policy` | `assume_false_silent\|assume_false_warn\|error` | `assume_false_silent` |
| `conditional_expr_policy` | `identifier_only\|identifier_or_defined` | `identifier_or_defined` |
| `plain_string_substitution` | `standard\|paste_delimited\|all` | `standard` |
| `strict_warning_codes` | `all\|none\|CODE[,CODE…]` | `none (or PGEN_SVPP_STRICT_WARNING_CODES)` |
| `undefined_macro_policy` | `error\|keep` | `error` |

`plain_string_substitution` is `SVPP-EXPANSION.5`(k), `undefined_macro_policy` is `SVPP-EXPANSION.6f`. A new key is
ADDITIVE; a default never moves without a service release bump.

## The outputs (preprocessed-output schema `1`, LIVE)
The serde form of `StageOutput` — the same bytes the tool's `--output` / `--*-json` files hold:
- `text: String` — the expanded compilation-unit file. Under `undefined_macro_policy=error` no backtick survives
  outside a comment or a string: an undefined macro, a function-like macro without its actuals, a stray macro-string
  token or an unknown directive is a REFUSAL with its clause (the parser never sees the text); under `keep` it is passed
  through for a consumer that resolves the rest itself.
- `source_map: Vec<{output_start, output_end, source: {file, line, column_start, column_end}}>` — sorted by
  `output_start`, byte offsets into `text`; `origin_of_position(byte)` is the binary search. Rule: every expanded byte
  maps to the location it came from; a token a `` `begin_keywords `` region rendered as an ESCAPED identifier maps to
  the unescaped bytes the author wrote. CONTRACTED FOR `.9.3` (schema `2`): `expansion_chain: Vec<{macro, file, line}>`
  on every entry — outermost usage first — so a diagnostic can read *"in the expansion of `M` at f.sv:12"*.
- `events: Vec<{kind, file, line, active, detail}>` — one per directive outcome; `active` says whether the directive
  fired in a live group. A new kind is ADDITIVE; a kind is never renamed within a schema. The kinds (the serde
  spellings of `PreprocessorEventKind`, pinned by a test over every variant and held by the currency gate):
  - **Event kinds** (schema `1`): `define`, `undef`, `include`, `ifdef`, `ifndef`, `elsif`, `else`, `endif`,
    `passthrough_line`, `skipped_line`, `macro_expand`, `line_directive`, `protected_envelope`, `keywords_region`.
- `diagnostics: Vec<{code, severity, file, line, message, detail}>` — `severity` is `warning` or `error`;
  `strict_warning_codes` promotes a warning to an error. A code is never reused with a different meaning; a new code is
  ADDITIVE. The codes (re-verify: `grep -oE '"[EW]_SVPP_[A-Z_0-9]+"' rust/src/families/systemverilog/preprocessor.rs | sort -u`;
  held by the currency gate):
  - **Diagnostic codes**: `E_SVPP_EXTRA_MACRO_ARGUMENT`, `E_SVPP_MACRO_REDEFINED`, `E_SVPP_MISSING_MACRO_ARGUMENT`,
    `E_SVPP_RECURSIVE_MACRO`, `E_SVPP_UNDEFINED_CONDITIONAL_SYMBOL`, `E_SVPP_UNTERMINATED_PROTECTED_ENVELOPE`,
    `W_SVPP_ABSOLUTE_INCLUDE_PATH`, `W_SVPP_INVALID_CONDITIONAL_SYMBOL`, `W_SVPP_KEYWORDS_VERSION_ABOVE_PROFILE`,
    `W_SVPP_KEYWORDS_VERSION_NOT_IN_EDITION`, `W_SVPP_MACRO_REDEFINED`, `W_SVPP_MALFORMED_KEYWORDS_DIRECTIVE`,
    `W_SVPP_NON_UTF8_SOURCE`, `W_SVPP_UNDEFINED_CONDITIONAL_SYMBOL`, `W_SVPP_UNKNOWN_KEYWORDS_VERSION`,
    `W_SVPP_UNSUPPORTED_CONDITIONAL_EXPR`.
- A **refusal** — the expansion did not complete, the parser never sees the text — is the `Err` of the typed API
  (`PreprocessRefusal`), the `Err` of the socket (an `anyhow` chain) and exit `3` of the probe (`preprocess_failed …`),
  with ONE message in all three, naming the clause. The typed refusal carries its `kind` (`RefusalKind`, 18 variants,
  `RefusalKind::ALL`), its `code` (`refusal_code(kind)`, the same `E_SVPP_…` vocabulary as the diagnostics — a refusal
  that IS a coded diagnostic reuses that diagnostic's code, and a promoted warning reports its own `W_SVPP_…` code), and
  the `file` and `line` the refusing site knew. A code is never reused with a different meaning; a new code is ADDITIVE;
  `E_SVPP_REFUSED` (`RefusalKind::Other`) is the kind of a refusal no site has classified — meeting one is a defect of
  the classification, to be named (re-verify: `grep -c 'RefusalKind::[A-Za-z]* => "E_SVPP_' rust/src/families/systemverilog/preprocessing_service.rs` → 18;
  held by the currency gate):
  - **Refusal codes** (`refusal_code`, one per `RefusalKind`): `E_SVPP_INCLUDE_NOT_FOUND`, `E_SVPP_INCLUDE_NESTING`,
    `E_SVPP_INCLUDE_MALFORMED`, `E_SVPP_SOURCE_UNREADABLE`, `E_SVPP_UNDEFINED_MACRO`, `E_SVPP_UNRESOLVED_BACKTICK`,
    `E_SVPP_MISSING_MACRO_ARGUMENT`, `E_SVPP_EXTRA_MACRO_ARGUMENT`, `E_SVPP_RECURSIVE_MACRO`, `E_SVPP_MACRO_REDEFINED`,
    `E_SVPP_DEFINE_MALFORMED`, `E_SVPP_UNDEFINED_CONDITIONAL_SYMBOL`, `E_SVPP_UNTERMINATED_CONDITIONAL`,
    `E_SVPP_UNTERMINATED_PROTECTED_ENVELOPE`, `E_SVPP_KEYWORDS_REGION`, `E_SVPP_PROMOTED_WARNING`,
    `E_SVPP_INVALID_CONTEXT`, `E_SVPP_REFUSED`.
- `included_files: Vec<String>` — every file the expansion read, the unit file first, then each include in order of
  first inclusion; each spelled as the file provider canonicalizes it (the disk's canonical path; an in-memory file as
  given).

## Semantics a consumer may rely on (LIVE; each with its clause and a corpus reproducer in `docs/tasks/SVPP-EXPANSION.md`)
Object and function-like macros with actuals and defaults (§22.5.1), a missing or extra actual refused
(`E_SVPP_MISSING_MACRO_ARGUMENT` / `E_SVPP_EXTRA_MACRO_ARGUMENT`), recursion refused by provenance; the ```` `` ````
paste with the order §22.5.1 implies (`.5`(i)), the `` `" `` region rendered once in the rescan (`.5`(j)), a plain
`"…"` string in macro text opaque by default (`.5`(k)); `//` comments stripped from macro text, block comments spanning
lines; a well-formed `` `line `` consumed (`line_directive`), a `` `pragma protect `` envelope consumed
(`protected_envelope`); `$ident` never substituted; the fifteen `` `SV_COV_* `` macros predefined (§40.3.1);
`` `begin_keywords `` regions applied as text (§22.14, `.8`) — a token the profile reserves and the region does not is
emitted as an escaped identifier; the unit's macro tables and keyword regions carried across `--unit-file` boundaries.
The SystemVerilog integration contract's *"Front End — the preprocess→parse pipeline"* section is the authoritative
statement of these rules for the PIPELINE and of how the corpus lane derives its context; this document does not
restate them with a different meaning, and a disagreement between the two is a documentation bug.

## Honest boundary (ledgered; each row is a leaf with its clause)
- The two readers of the directive layer — svpp's grammar and the expansion stage's recognizer — disagree in measured
  places, each adjudicated: `SVPP-EXPANSION.5`(w) `` `line 0 `` consumed (§22.12 wants a positive integer); (x) an
  ignored group not held to the lexical conventions (§22.6); (y) a parenthesized conditional passing under `sv_2017` /
  `verilog_2005` (1800-2023-only); (z) a formal default and `` `" `` passing under `verilog_2005` (1364-2005 clause 19
  has neither); (aa) a directive over several lines read inconsistently and edition-blind; (ab) an empty formal list
  passing (Syntax 22-2 has at least one formal); (ac) an actual list closed by parenthesis depth alone (§22.5.1). Each
  is an EXPECTED MOVER of `.9.3`'s reading — the pipeline re-key on the same corpus must move exactly these rows and
  `AST-IDENTITY-SWEEP ast_moved=0` elsewhere — and none is a documented limit: each is fixed, by `.9.3` or before it.
- No expansion chain in the source map (`.9.3`, schema `2`); no by-name / JSON in-process form (a director question);
  the intercept mode (`expand(ast, ctx)`) is `.9.3` — today a consumer intercepts through the context (its own
  `FileProvider`, its predefines), not through the AST.
- Parse cost: the expander must not cost the parser — the SystemVerilog parse-cost ratchet (`PARSE-COST-RATCHET`) is
  held unmoved across `.9.3` / `.9.4`; a cost is REJECTED, not traded (north star).

## Versioning rules
- The service release version and the output schema version are axes of THIS document, independent of svpp's parser
  release / AST-dump schema and of the SystemVerilog parser's; a change to either of those never moves these.
- Schema: a new field, event kind, diagnostic code or option key is ADDITIVE within a schema; a removal, a rename, a
  changed default or a changed meaning is a new schema version, announced in this document's identity block and in the
  svpp book's `schema-versioning.md` before it ships.
- Service release: any change a consumer could observe in the five outputs for an unchanged input and context is a
  release; the pipeline re-key (`REKEY-MOVERS`) is its measurement, with every mover adjudicated.
- Held, not promised: the two versions, the option table, the event kinds, the diagnostic codes, the refusal codes and
  the named surfaces are re-derived from the code at every commit by `scripts/check_svpp_expansion_contract_currency.sh`;
  an addition that does not land in this document in the same commit is refused there.

## Acceptance harness (what `.9.2` – `.9.4` must show before a tag is removed)
- A RED-first test per consumption mode: AST-only (a both-arms walk), PGEN-expands, intercept (an in-memory include
  served by a provider; a define injected into the tree). `.9.2` (2026-10-20, `rust/src/families/systemverilog/preprocessing_service.rs`
  tests): PGEN-expands through the typed API equals the socket byte-for-byte; the in-memory include is a typed
  `E_SVPP_INCLUDE_NOT_FOUND` refusal without the provider entry and the served text with it (RED first); the typed
  refusal's kind, code, file and line for an undefined macro, a redefinition under `error`, a promoted warning, an
  undeclared option, an absent unit file and an unterminated conditional in the second file of a unit; the probe's two
  emit flags refused without `--preprocess`. The AST-level intercept and the both-arms walk stay `.9.3`.
- Source-map round trip: every expanded byte maps to a location; after `.9.3` a macro-expansion diagnostic names the chain.
- The pipeline re-key on the SAME corpus: `REKEY-MOVERS` identical on the three readings except the adjudicated
  (w) – (ac) rows; the svpp census unchanged (the AST side does not move); the parse-cost ratchet unmoved.

## Open for the director (not decided here)
- The name and placement of layer 2 (a service of the `systemverilog_preprocessor` family, or a third family name).
- Whether the by-name / JSON surface ships with `.9.2` or after the typed Rust API.

## Companion documentation and precedence
| Surface | Path | Authoritative for |
|---|---|---|
| **This contract** | `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_EXPANSION_CONTRACT.md` | the service's surfaces, context, outputs, consumption modes and stability axes |
| **The svpp book chapter** | `docs/systemverilog_preprocessor_parser_book/src/preprocessing-service.md` | the teaching text and the definitions (the director's window) |
| **The svpp integration contract** | `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md` | layer 1 — the directive AST, its profiles, its schema |
| **The SystemVerilog integration contract**, *"Front End — the preprocess→parse pipeline"* | `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` | the pipeline command's verdicts and the corpus lane's context derivation |
| **The SystemVerilog book chapter** | `docs/systemverilog_parser_book/src/preprocess-parse-pipeline.md` | the live measurement of the expansion on the corpus |
| **The embedding-API contract** | `rust/docs/EMBEDDING_API_CONTRACT.md` | the host-API truth for layer 1's entry points |
| **The service register** | `docs/tasks/PGEN-SERVICES.md` | the five-column row this document fills |
| **The currency gate** | `scripts/check_svpp_expansion_contract_currency.sh` (`SVPP-EXPANSION-CONTRACT-CURRENCY`) | that this document's vocabularies and versions equal the code's, at every commit |

Precedence when surfaces disagree (highest first): the embedding-API contract for layer 1's host API; this contract
for the service's surfaces and outputs; the SystemVerilog integration contract for the pipeline's verdicts; the books
for teaching text. Report a disagreement as a documentation bug rather than code to the lower-precedence surface.
