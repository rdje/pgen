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

### Support and issue workflow

- `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
- `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`

## Release and Maintenance Model

PGEN uses maintenance releases, issue ledgers, and contract updates to keep parser-family promises explicit rather than implied.

The release policy reference lives in:

- `docs/reference/PGEN_RELEASE_POLICY.md`
