# Embedding and Downstream Integration

PGEN is built to be embedded into other projects, not only run as a standalone repo-local tool.

## Main Downstream Shapes

Downstream consumers interact with PGEN through a few stable kinds of surfaces:

- generated parser artifacts,
- parser registry integration,
- embedding API functions,
- parser-family integration contracts,
- issue reporting and release-support workflow.

## Embedding API

The embedding API exists to provide a versioned surface for selected parser families and annotation parsing without forcing downstream projects to understand the entire repository internals.

This matters especially for:

- parser availability,
- profile names,
- diagnostics shape,
- AST dump availability,
- release-version metadata.

## Linter-Oriented Downstream Surfaces

One downstream direction now tracked explicitly is linter enablement. The idea is not that PGEN should become a linter by itself. The idea is that PGEN should be able to hand downstream tools a stronger front-end product than "just a parse tree."

That means the longer-term downstream surface should include:

- grammar-shaped ASTs,
- provenance-carrying semantic seeds,
- scope and declaration facts,
- pragma and waiver capture,
- version and dialect gating,
- and stable machine-readable export for later attribution and rule engines.

The detailed platform plan for that lane lives in:

- `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`

## Compiler And Elaborator-Oriented Downstream Surfaces

Another explicit downstream direction is compiler and elaborator enablement.

Here the goal is broader than linting. The goal is to let downstream tools start from a real front-end bundle instead of only:

- parse success,
- plus an opaque tree,
- plus a lot of hand-written glue work.

The longer-term workbench surface should therefore include:

- lossless or source-faithful front-end structure where needed,
- shaped ASTs,
- stable node ids,
- traversal and query helpers,
- semantic-bundle export,
- source-map and diagnostic-anchor fidelity,
- and explicit handoff scaffolding into later binding, typing, lowering, or elaboration passes.

The detailed platform plan for that lane lives in:

- `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`

## Parser Registry

The parser registry is the in-process discovery and routing seam used by several internal and proof paths. It matters because many contract lanes and probes do not call generated parsers directly; they go through registry-backed surfaces that downstream consumers can rely on more easily.

## Contracts As Downstream Truth

For public downstream use, the contracts are the most important authority. They state:

- what is supported,
- what version or profile names exist,
- what proof lane backs the surface,
- what bug-report workflow downstream users should follow.

## Release And Support Model

PGEN treats downstream parser support as a maintained surface:

- released parser bug ledgers exist,
- issue reporting protocol is explicit,
- maintenance releases can widen or harden syntax support without pretending the support boundary is undocumented.

That model is especially visible in the regex parser track because RGX actively consumes it.

For the regex track, PGEN is intentionally transparent about how PCRE2 compatibility evidence is derived. PCRE2 does not provide a formal EBNF/PEG for the whole flavor, so PGEN treats `pcre2syntax(3)` and `pcre2pattern(3)` as the prose intent layer, `src/pcre2_compile.c` as the hand-written parser edge-case authority, and PCRE2 upstream `testdata/testinput*` plus expected outputs as the executable regression oracle. RGX conformance reports are then folded back into the regex integration contract and released-parser bug ledger, rather than fixed as one-off examples.

The 2026-04-14 regex audit makes that split concrete: syntax forms such as string callouts, non-atomic lookarounds, scan-substring/script-run groups, strict VERSION conditionals, and PCRE2 escape spellings live in `regex.ebnf`, while source-derived compile checks such as verb legality, `\K` lookaround restrictions, forbidden class escapes, POSIX class names, and scan-substring reference existence live in the generated-host validation layer. That is intentional transparency, not a hidden second grammar.

The 2026-04-15 regex maintenance release extends that same split for RGX `PGEN-RGX-0058` through `PGEN-RGX-0060`: bounded variable-length lookbehind and Unicode capture names are now modeled as accepted PCRE2 surface, while unbounded lookbehind, malformed named references, names that violate PCRE2's `128`-byte limit, and empty substantive character classes remain generated-host compile-contract rejections. Orphan `\E` inside a character class follows `pcre2_compile.c` rather than folklore: it is a zero-width scanner marker, not a literal `E` atom, so non-empty classes such as `^[\Eabc]` are valid while `[\E]` remains invalid.

The 2026-04-16 regex maintenance release continues that source-derived approach for RGX `PGEN-RGX-0061` and `PGEN-RGX-0062`. PCRE2 `\C` now transports as a dedicated `single_byte_escape` AST node because `pcre2_compile.c` treats it as `ESC_C`, not as a generic escaped `C`. Conditional assertion tests may also be preceded by an explicit callout, so patterns like `^(?(?C25)(?=abc)abcd|xyz)` and the string-callout variant `^(?(?C$abc$)(?=abc)abcd|xyz)` preserve both `condition_callout` and `condition_assertion` structure.

The current regex maintenance release also documents a resource-depth distinction that downstream embedders should understand: legal PCRE2 patterns can be syntactically deep enough to stress a generated parser even when they are small in bytes. PGEN therefore runs generated regex entrypoints on a larger bounded worker stack (`64 MiB`) and keeps the generated recursion guard bounded (`4096`) but high enough for the retained PCRE2 conformance samples.

PNR now also has a PGEN-side downstream contract addendum. It captures PNR's inbound request for future LEF, DEF, Liberty, SDC, structural Verilog netlist, and SPEF parser crates, while explicitly marking those parser-family releases as pending rather than shipped.

For that pending PNR/SDC lane, `docs/tcl/md/tcl.md` records a local Tcl syntax note so future work remembers that SDC is Tcl-shaped at the command/word/substitution layer. That note is reference input only; the authoritative SDC command semantics still need the Synopsys SDC source path recorded in the PNR contract before EBNF work starts.

## Primary Source Docs

- `rust/docs/EMBEDDING_API_CONTRACT.md`
- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`
- `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
- `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_PNR_PARSER_INTEGRATION_CONTRACT.md`
- `docs/reference/PGEN_LINTER_ENABLEMENT_ROADMAP.md`
- `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md`
