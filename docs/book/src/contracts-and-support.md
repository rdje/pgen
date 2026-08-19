# Contracts and Support

PGEN publishes downstream parser behavior through explicit contracts.

## Why Contracts Exist

They define:

- what parser families are available,
- how they should be built or consumed,
- what stability and support boundaries apply,
- how downstream projects should report bugs,
- how released parser issues are tracked.

## Contract Layers

### Integration overview

- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`

### Family-specific contracts

- `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_RETURN_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`
- `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`

### Downstream-specific contract addenda

- `docs/contracts/PGEN_PNR_PARSER_INTEGRATION_CONTRACT.md`

The PNR addendum is a pending-request contract, not a shipped parser release. It names future LEF, DEF, Liberty, SDC, structural Verilog netlist, and SPEF parser surfaces and points SDC work at Tcl-shaped tokenization/quoting/substitution behavior. The local Tcl syntax note lives at `docs/tcl/md/tcl.md`.

### Support and issue workflow

- `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`

## Release and Maintenance Model

PGEN uses maintenance releases, issue ledgers, and contract updates to keep parser-family promises explicit rather than implied.

The regex/RGX track is the most active example of that model: PCRE2 conformance reports are recorded in the released-parser ledger, fixed or rejected with rationale, and then reflected in the regex integration contract and user-facing docs when the public handoff changes.

The release policy reference lives in:

- `docs/reference/PGEN_RELEASE_POLICY.md`

### Keeping a contract true — the SystemVerilog grammar revision register

A contract is a promise about a parser, and a parser is generated from a grammar. So a contract is
only as true as the last time someone compared the two — and *"someone compares them"* is a habit,
not a mechanism.

Measured on 2026-08-19, the habit had failed. The SystemVerilog contract said release `1.0.183`,
the state it was left in on 2026-08-12. Between those two dates `grammars/systemverilog.ebnf` moved
in **nine** commits, **seven** of which changed what the code generator consumes. Nothing in the
repository related the two files: the check that would have caught it on day one was one `git log`
away and did not exist.

Two things about that week are worth stating plainly, because they shaped the fix:

- **Four of the seven replaced an AST shape a consumer was already reading** — a `cross` body's item
  list, a hierarchical call's path, the node type of every gate instantiation, and the key of an
  assignment pattern. A contract watched only for *accept-set* drift would have missed all four.
- **One of those four moved no verdict at all.** `bufif0 g(o, i, e);` parsed before the fix and
  parses after it; what changed is that it stopped arriving as a `udp_instantiation` and started
  arriving as a `gate_instantiation`. Twelve pinned reproducers changed shape and zero changed
  verdict, so every pass/fail oracle in the repository stayed green.

The remedy is `docs/contracts/PGEN_SV_GRAMMAR_REVISION_REGISTER.tsv` — one row
per revision of the SV grammar, each either a `RELEASE` (a contract section plus a bug-ledger row) or
`NEUTRAL` (comment-only). Its identity column is not a hash of the file's bytes; it is a hash of the
**EBNF frontend's own `raw_ast` envelope** — what the generator actually consumes — so a comment
rewrite cannot move it and nothing a generated parser can observe can hide from it.

That makes a `NEUTRAL` claim *checkable rather than believed*: its digest must equal its
predecessor's. The `SV-CONTRACT-CURRENCY` doctrine enforces the whole thing on every commit, in four
tiers — history, the staged diff, neutrality, and a re-derivation of the working tree's digest from
the producer. It is what turns *"remember to update the contract"* into something that cannot be
forgotten.
