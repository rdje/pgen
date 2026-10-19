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

## Status — DRAFT `0.1` (2026-10-19, `SVPP-EXPANSION.9.1`)
⛔ This document contracts a surface that is being BUILT in three measured steps. Every statement below is tagged:
**LIVE** (true of the committed tree on the date above, re-verifiable by the command beside it) or **CONTRACTED FOR `.9.n`**
(the step that lands it, after which the tag is removed and the service version moves). A downstream project may code
against a LIVE statement today and may PLAN against a CONTRACTED one; it must not assume a contracted surface exists.

## Contract Identity (the service register's five columns)
- Service:
  - the **SystemVerilog preprocessing service** — layer 2 over the `systemverilog_preprocessor` family's AST (layer 1).
    Placement (a service of the `systemverilog_preprocessor` family, or a third family name) is an OPEN director
    question; the surfaces below do not depend on the answer.
- Contract (this document) version:
  - `0.1` (draft). `1.0` is reached with `.9.2`, when the typed Rust API and the probe's emit flag exist and a currency
    gate holds this document to them.
- Stability axes:
  - **service release version** — the runtime module's own line, independent of svpp's parser release (`1.0.9`) and of
    the SystemVerilog parser's. LIVE: the module carries no version of its own yet (it ships under the SystemVerilog
    family's release stream, `rust/docs/EMBEDDING_API_CONTRACT.md`); CONTRACTED FOR `.9.2`: `1.0.0`.
  - **preprocessed-output schema version** — the shape of the five outputs. LIVE: schema `1` = the serde form of
    `pgen::preprocessing_stage::StageOutput` below, unversioned on the wire today; CONTRACTED FOR `.9.2`: the version is
    carried in the JSON carrier; CONTRACTED FOR `.9.3`: schema `2`, the macro-expansion chain on every source-map entry.
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
  - LIVE: none holds this document to the code (it is a draft); the stage's declared option list and its legacy-flag
    rewrites are held by the SystemVerilog family's gates. CONTRACTED FOR `.9.2`: a gate that re-derives the option
    table, the event kinds, the diagnostic codes and the schema version from the code and refuses on drift
    (`PGEN-SERVICES.2`'s doctrine shape).
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
| **PGEN expands** — the project compiles against the preprocessed output | source text + context | the five outputs | LIVE in-process (the stage socket) and as a tool (`ast_pipeline`); CONTRACTED FOR `.9.2`: the typed API, the file provider, the probe's `--emit-preprocessed` |
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
The runtime module's own functions (`preprocess_systemverilog_file(path, &SvPreprocessorConfig)`,
`preprocess_systemverilog_unit(&[PathBuf], …)` → `PreprocessedOutput`) are the socket's implementation; their crate
visibility and their stability are what `.9.2` publishes — until then a consumer codes to the socket.

### LIVE — as a tool
```bash
ast_pipeline INPUT.sv --preprocess systemverilog [--profile P] [--include-dir DIR]... [--symbol NAME[=VALUE]]...
    [--unit-file FILE]... [--stage-opt KEY=VALUE]...
    --output OUT.sv --source-map-json MAP.json --event-log-json EVENTS.json --diagnostics-json DIAG.json
ast_pipeline INPUT.sv --preprocess systemverilog --list-stage-options     # the declared option table, with defaults
parseability_probe --parse systemverilog INPUT.sv --profile P --preprocess [the same context flags]   # expand THEN parse
```
Exit codes of the probe: `0` accepted; `1` the parser rejected the expanded text (`furthest_position=N`,
`[origin=FILE:LINE:COL]` from the source map); `3` the expander refused (`preprocess_failed …`). The probe does NOT emit
the expanded text today. `--symbol` has the alias `--define`; the pre-`ENGINE-UNIVERSAL-SERVICES.113` spellings
(`--preprocess-systemverilog`, every `--sv-…` flag) are accepted through SystemVerilog release `1.0.247` with one
deprecation line each.

### CONTRACTED FOR `.9.2` — the typed API over the existing stage
```rust
// placement per the director's answer; names are the contract, the module path is not yet
pub fn preprocess(unit: &[&Path], ctx: &PreprocessContext) -> Result<Vec<PreprocessedOutput>, PreprocessRefusal>;
pub fn preprocess_str(name: &str, text: &str, ctx: &PreprocessContext) -> Result<PreprocessedOutput, PreprocessRefusal>;
pub trait FileProvider { fn read(&self, path: &Path) -> Result<Vec<u8>, ProviderError>; }   // default: the disk
```
and `parseability_probe … --preprocess --emit-preprocessed OUT.sv [--emit-source-map MAP.json]`. A by-name / JSON form
(`preprocess_by_name("systemverilog", json_request) -> json`) ships with `.9.2` or after it — a director question.

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
| file provider | — | CONTRACTED FOR `.9.2`: `path → bytes` for the unit's files and every include; the default reads the disk, a consumer's serves memory |

The declared options (LIVE; re-verify with `--list-stage-options`): `include_max_depth=N` (64);
`include_path_policy=allow_absolute / relative_only` (allow_absolute); `macro_redefine_policy=allow / warn / error`
(allow); `conditional_symbol_policy=assume_false_silent / assume_false_warn / error` (assume_false_silent);
`conditional_expr_policy=identifier_only / identifier_or_defined` (identifier_or_defined);
`plain_string_substitution=standard / paste_delimited / all` (standard — `SVPP-EXPANSION.5`(k));
`strict_warning_codes=all / none / CODE[,CODE…]` (none, or `PGEN_SVPP_STRICT_WARNING_CODES`);
`undefined_macro_policy=error / keep` (error — `SVPP-EXPANSION.6f`). A new key is ADDITIVE; a default never moves
without a service release bump.

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
- `events: Vec<{kind, file, line, active, detail}>` — one per directive outcome; `kind` is one of `define`, `undef`,
  `include`, `ifdef`, `ifndef`, `elsif`, `else`, `endif`, `passthrough_line`, `skipped_line`, `macro_expand`,
  `line_directive`, `protected_envelope`, `keywords_region` (the serde spellings of
  `PreprocessorEventKind`, pinned by a test over every variant); `active` says whether the directive fired in a live
  group. A new kind is ADDITIVE; a kind is never renamed within a schema.
- `diagnostics: Vec<{code, severity, file, line, message, detail}>` — `severity` is `warning` or `error`. LIVE codes
  (re-verify: `grep -oE '"[EW]_SVPP_[A-Z_0-9]+"' rust/src/families/systemverilog/preprocessor.rs | sort -u`):
  `E_SVPP_EXTRA_MACRO_ARGUMENT`, `E_SVPP_MACRO_REDEFINED`, `E_SVPP_MISSING_MACRO_ARGUMENT`, `E_SVPP_RECURSIVE_MACRO`,
  `E_SVPP_UNDEFINED_CONDITIONAL_SYMBOL`, `E_SVPP_UNTERMINATED_PROTECTED_ENVELOPE`, `W_SVPP_ABSOLUTE_INCLUDE_PATH`,
  `W_SVPP_INVALID_CONDITIONAL_SYMBOL`, `W_SVPP_KEYWORDS_VERSION_ABOVE_PROFILE`, `W_SVPP_KEYWORDS_VERSION_NOT_IN_EDITION`,
  `W_SVPP_MACRO_REDEFINED`, `W_SVPP_MALFORMED_KEYWORDS_DIRECTIVE`, `W_SVPP_NON_UTF8_SOURCE`,
  `W_SVPP_UNDEFINED_CONDITIONAL_SYMBOL`, `W_SVPP_UNKNOWN_KEYWORDS_VERSION`, `W_SVPP_UNSUPPORTED_CONDITIONAL_EXPR`.
  `strict_warning_codes` promotes a warning to an error. A code is never reused with a different meaning; a new code is
  ADDITIVE. (Refusals that are not diagnostics — a missing include, a depth overrun, the `undefined_macro_policy=error`
  refusals, `E_SVPP_RECURSIVE_MACRO` as a refusal — surface as the `Err` of the call / exit `3` of the probe with a message
  that names the clause; `.9.2` gives them a typed `PreprocessRefusal` carrying the same code vocabulary.)
- `included_files: Vec<String>` — every file the expansion read, in order of first inclusion.

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
- The probe parses the expanded text and does not emit it (`.9.2`); no file provider (`.9.2`); no typed refusal
  (`.9.2`); no expansion chain in the source map (`.9.3`, schema `2`); no by-name / JSON in-process form (a director
  question).
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

## Acceptance harness (what `.9.2` – `.9.4` must show before a tag is removed)
- A RED-first test per consumption mode: AST-only (a both-arms walk), PGEN-expands, intercept (an in-memory include
  served by a provider; a define injected into the tree).
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

Precedence when surfaces disagree (highest first): the embedding-API contract for layer 1's host API; this contract
for the service's surfaces and outputs; the SystemVerilog integration contract for the pipeline's verdicts; the books
for teaching text. Report a disagreement as a documentation bug rather than code to the lower-precedence surface.
