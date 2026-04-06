# PGEN_RELEASED_PARSER_BUG_LEDGER.md

## Purpose
Track every bug reported against a released PGEN parser family until it is either fixed, proven invalid, or explicitly deferred with a documented reason.

This is a live operational ledger, not an archival narrative.

GitHub is optional. This ledger should be the canonical parser-side tracker inside the PGEN git repo while any number of downstream consumer repos may keep their own local tracking references.

## Tracking Rule
- Every downstream bug report against a released parser family must receive a stable report ID.
- Every accepted report must link back to a reproducible artifact bundle captured using `PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.
- A bug is not considered fully closed until:
  - the root cause is identified,
  - the fix is landed,
  - executable proof exists,
  - and the ledger entry is updated with the fix provenance.

## Per-Parser Indexing Rule
- The primary index for this ledger is `Parser Family/Profile`.
- Report IDs should carry a parser-family prefix whenever practical, for example:
  - `REGEX-0001`
  - `VHDL-0003`
  - `SV-0012`
- If multiple downstream consumers report the same parser-side root cause, prefer one canonical ledger row with multiple downstream tracking refs instead of losing that linkage across separate isolated notes.
- If two reports look similar but differ in parser profile, reproducer class, or root cause, keep separate rows and cross-reference them in `Notes`.

## Required Fields
- `Report ID`
- `Parser Family/Profile`
- `Reported Against Parser Release`
- `Reported Against Contract Version`
- `Downstream Consumer(s)`
- `First Reported`
- `Current State`
- `Downstream Tracking Refs`
- `Reproducer Bundle`
- `Root Cause`
- `Fix Proof`
- `Fixed In`
- `Notes`

## State Meanings
- `Reported`
  - issue has been received but not yet reproduced locally
- `Reproduced`
  - PGEN can reproduce it from the supplied bundle
- `Root Caused`
  - the mechanism is understood, but the fix is not landed yet
- `Fix In Progress`
  - an implementation is underway
- `Fixed Pending Release`
  - a fix is landed and validated, but downstream release/adoption is still pending
- `Released`
  - the fix is landed, validated, and available to downstream consumers
- `Rejected`
  - not actually a PGEN bug, duplicate, or outside the contracted parser surface
- `Deferred`
  - acknowledged but intentionally postponed with a documented reason

## Closure Rule
Each `Released` row should point at:
- the reproducer artifact bundle
- the parser release version containing the fix
- and the commit containing the fix when that extra provenance helps downstream consumers map local adoption timing
- the regression test or gate proving the bug stays closed

## Live Ledger

| Report ID | Parser Family/Profile | Reported Against Parser Release | Reported Against Contract Version | Downstream Consumer(s) | First Reported | Current State | Downstream Tracking Refs | Reproducer Bundle | Root Cause | Fix Proof | Fixed In | Notes |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `REGEX-0001` | `regex` / `regex_default` | `1.1.0` | `1.1.0` | `RGX` | `2026-03-29` | `Released` | [`PGEN-RGX-0001`](../rgx/pgen-issues/PGEN-RGX-0001.yaml) | [`PGEN-RGX-0001/`](../rgx/pgen-issues/artifacts/PGEN-RGX-0001/) | `(?R)` was shadowed by `inline_modifiers` because `atom` tried `inline_modifiers` before `subroutine_call`, and `modifier_char` accepted `R`. | `regex_parser_integration_contract_classifies_whole_pattern_recursion_as_subroutine_call` plus `make -C rust regex_parser_integration_contract_gate` | `regex parser release 1.1.1` | Transport fix only; schema version stays `1`. |
| `REGEX-0002` | `regex` / `regex_default` | `1.1.0` | `1.1.0` | `RGX` | `2026-03-29` | `Released` | [`PGEN-RGX-0002`](../rgx/pgen-issues/PGEN-RGX-0002.yaml) | [`PGEN-RGX-0002/`](../rgx/pgen-issues/artifacts/PGEN-RGX-0002/) | Numeric backreferences were shadowed by generic `escape` parsing; `atom` tried `escape` before `backreference`, and the numeric backreference path needed an explicit nonzero-digit split so `\0`-style octal handling stayed distinct. | `regex_parser_integration_contract_classifies_numeric_backreferences` plus `make -C rust regex_parser_integration_contract_gate` | `regex parser release 1.1.1` | The published contract already claimed `\1`; `1.1.1` makes the accepted tree match that claim. |
| `REGEX-0003` | `regex` / `regex_default` | `1.1.0` | `1.1.0` | `RGX` | `2026-03-29` | `Released` | [`PGEN-RGX-0003`](../rgx/pgen-issues/PGEN-RGX-0003.yaml) | [`PGEN-RGX-0003/`](../rgx/pgen-issues/artifacts/PGEN-RGX-0003/) | Conditional branches used `pattern?`, so `yes_branch` greedily consumed the full `a|b` alternation and suppressed the explicit false branch. | `regex_parser_integration_contract_preserves_conditional_false_branch` plus `make -C rust regex_parser_integration_contract_gate` | `regex parser release 1.1.1` | `yes_branch` / `no_branch` transport is now explicit for false-branch conditionals. |
| `REGEX-0004` | `regex` / `regex_default` | `1.1.0` | `1.1.0` | `RGX` | `2026-03-29` | `Released` | [`PGEN-RGX-0004`](../rgx/pgen-issues/PGEN-RGX-0004.yaml) | [`PGEN-RGX-0004/`](../rgx/pgen-issues/artifacts/PGEN-RGX-0004/) | `literal = literal_char+` let trailing quantifiers bind to a whole literal run, so `ab+` transported like `(ab)+` instead of `a` then `b+`. | `regex_parser_integration_contract_binds_quantifier_to_final_literal_atom` plus `make -C rust regex_parser_integration_contract_gate` | `regex parser release 1.1.1` | This is an accepted-tree semantic fix; downstream AST consumers should repin to `1.1.1`. |
| `REGEX-0005` | `regex` / `regex_default` | `1.1.1` | `1.1.1` | `RGX` | `2026-04-01` | `Released` | [`PGEN-RGX-0005`](../rgx/pgen-issues/PGEN-RGX-0005.yaml) | [`PGEN-RGX-0005/`](../rgx/pgen-issues/artifacts/PGEN-RGX-0005/) | `condition` only admitted `recursion_condition = "R" digits?`, so named recursion conditions like `R&word` were rejected instead of entering the conditional transport path. | `regex_parser_integration_contract_accepts_named_recursion_conditionals` plus `make -C rust regex_parser_integration_contract_gate` | `regex parser release 1.1.2` | This was a real downstream blocker for RGX and closes the published named recursion-condition gap without changing regex AST schema version `1`. |
| `REGEX-0006` | `regex` / `regex_default` | `1.1.2` | `1.1.2` | `RGX` | `2026-04-05` | `Released` | [`PGEN-RGX-0006`](../rgx/pgen-issues/PGEN-RGX-0006.yaml) | [`PGEN-RGX-0006/`](../rgx/pgen-issues/artifacts/PGEN-RGX-0006/) | `regex.ebnf` did not model braced octal escapes, so `\o{101}` was accepted as generic `simple_escape("o")` followed by counted quantifier `{101}` instead of a real octal escape. The post-parse compile validator also skipped only two-byte escapes, so brace-style numeric escapes could be re-read as counted quantifiers. | `regex_parser_integration_contract_classifies_braced_octal_escape`, `allows_braced_octal_escape_without_counted_quantifier_rejection`, `regex_parser_integration_contract_accepts_declared_success_samples`, and `make -C rust regex_parser_integration_contract_gate` | `regex parser release 1.1.3` | Accepted-tree fix plus host-validation hardening only; regex AST schema version stays `1`. |
| `REGEX-0007` | `regex` / `regex_default` | `1.1.3` | `1.1.3` | `RGX` | `2026-04-05` | `Released` | [`PGEN-RGX-0007`](../rgx/pgen-issues/PGEN-RGX-0007.yaml) | [`PGEN-RGX-0007/`](../rgx/pgen-issues/artifacts/PGEN-RGX-0007/) | `subroutine_ref` only admitted `name` inside `<...>` / `'...'`, so numeric forms like `\g<1>` fell through the `backreference` path and were misclassified as `simple_escape("g")` plus literal `<`, `1`, `>`. | `regex_parser_integration_contract_classifies_numeric_angle_subroutine_ref`, `regex_parser_integration_contract_accepts_declared_success_samples`, and `make -C rust regex_parser_integration_contract_gate` | `regex parser release 1.1.4` | Accepted-tree fix only; regex AST schema version stays `1`. |
| `REGEX-0008` | `regex` / `regex_default` | `1.1.3` | `1.1.3` | `RGX` | `2026-04-06` | `Released` | [`PGEN-RGX-0008`](../rgx/pgen-issues/PGEN-RGX-0008.yaml) | [`PGEN-RGX-0008/`](../rgx/pgen-issues/artifacts/PGEN-RGX-0008/) | `code_block` tried `code_block_plain` before `code_block_lang`, so tagged payloads like `(?{lua:return true})` were always consumed as plain opaque blocks. `code_lang` also omitted `rhai`, `native`, and `wasm`, which left real RGX tag forms outside the structured tagged path entirely. | `regex_parser_integration_contract_classifies_language_tagged_code_blocks`, `regex_parser_integration_contract_preserves_plain_code_blocks_as_plain`, `make -C rust regex_embedded_code_block_contract_gate`, and `make -C rust regex_parser_integration_contract_gate` | `regex parser release 1.1.5` | Accepted-tree fix plus tagged syntax widening only; regex AST schema version stays `1`. |
| `REGEX-0009` | `regex` / `regex_default` | `1.1.5` | `1.1.5` | `RGX` | `2026-04-06` | `Released` | [`PGEN-RGX-0009`](../rgx/pgen-issues/PGEN-RGX-0009.yaml) | [`PGEN-RGX-0009/`](../rgx/pgen-issues/artifacts/PGEN-RGX-0009/) | The regex grammar surface was correct, but the Rust EBNF frontend decoded `'\v'` as literal `v` instead of vertical tab when adapting quoted terminals. That widened the generated regex whitespace rule, so the optional `ws?` inside `code_block_lang` could consume the first payload byte of tagged forms such as `(?{native:validate_word})`, shifting `code_content` one byte late. | `regex_parser_integration_contract_enforces_declared_ast_shape_for_success_samples`, `regex_parser_integration_contract_classifies_language_tagged_code_blocks`, `cargo test --features ebnf_dual_run tokenizes_quoted_literals_with_decoded_escape_sequences --lib`, `cargo test --features ebnf_dual_run parses_ebnf_text_into_raw_ast_with_decoded_terminal_escapes --lib`, `make -C rust regex_embedded_code_block_contract_gate`, and `make -C rust regex_parser_integration_contract_gate` | `regex parser release 1.1.6` | Accepted-tree span fix plus stronger rule-text oracle coverage; regex AST schema version stays `1`. |
| `REGEX-0010` | `regex` / `regex_default` | `1.1.6` | `1.1.6` | `RGX` | `2026-04-06` | `Released` | [`PGEN-RGX-0010`](../rgx/pgen-issues/PGEN-RGX-0010.yaml) | [`PGEN-RGX-0010/`](../rgx/pgen-issues/artifacts/PGEN-RGX-0010/) | `condition` used PEG ordered choice with `name` before `recursion_condition`, so numeric recursion tests like `(?(R1)b|c)` were accepted but transported as `name("R1")` instead of `recursion_condition("R1")`. | `regex_parser_integration_contract_accepts_bare_recursion_conditionals`, `regex_parser_integration_contract_accepts_numeric_recursion_conditionals`, `regex_parser_integration_contract_enforces_declared_ast_shape_for_success_samples`, and `make -C rust regex_parser_integration_contract_gate` | `regex parser release 1.1.7` | Accepted-tree fix plus stronger conditional-shape oracle coverage; named recursion forms like `R&word` still legitimately preserve nested `name` under `recursion_condition`, and regex AST schema version stays `1`. |
