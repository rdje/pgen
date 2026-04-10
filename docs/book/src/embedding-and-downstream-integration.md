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

PNR now also has a PGEN-side downstream contract addendum. It captures PNR's inbound request for future LEF, DEF, Liberty, SDC, structural Verilog netlist, and SPEF parser crates, while explicitly marking those parser-family releases as pending rather than shipped.

## Primary Source Docs

- `rust/docs/EMBEDDING_API_CONTRACT.md`
- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`
- `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
- `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_PNR_PARSER_INTEGRATION_CONTRACT.md`
