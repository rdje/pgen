# docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the downstream integration contract for PGEN's `regex` parser family.

This is the document downstream projects such as RGX should read first when deciding how to embed the PGEN regex parser.

## Contract Identity
- Contract version:
  - `1.1.17`
- Parser release version:
  - `1.1.16`
- Embedding API contract baseline:
  - `1.2.0`
- Regex AST-dump schema version:
  - `1`
- Last updated:
  - `2026-04-13`
- Current grammar family label:
  - `regex`
- Current stable host profile:
  - `regex_default`
- Current live status:
  - `Done` for the currently tracked grammar contract in `LIVE_ACHIEVEMENT_STATUS.md`

## Current Trust Statement
- PGEN currently treats the published regex flavor, when consumed through the stable `pgen::embedding_api` host surface, as closure-grade and fit for downstream parser consumption.
- That statement applies to the published regex parser contract documented here and in the regex-flavor section of `PGEN_USER_GUIDE.md`.
- It does not automatically cover every regex dialect or every future contract widening.

## Release 1.1.16 / Contract 1.1.17 Highlights
- `1.1.16` is a PCRE2-conformance directive-payload generalization over the `1.1.15` parser release.
- This specifically covers RGX PCRE2 conformance reports `PGEN-RGX-0031` and `PGEN-RGX-0032` and broadens the contract for the backtracking-control verb payload family instead of pinning only the concrete failing payload string.
- PCRE2's default verb-name rule is that `(*VERB:NAME)` payload text is a sequence of characters up to the verb-closing `)`: `MARK` requires a non-empty payload/name, while `PRUNE`, `SKIP`, and `THEN` accept optional payload/name text. PGEN now models that parser-side payload shape conceptually as `directive_payload_char = /([^)])/`: any character except `)` is payload text in default `regex_default` directive parsing.
- MARK shorthand still transports through `directive_verb` -> `directive_mark_shorthand` -> `directive_payload_simple`; named backtracking-control verb payloads transport through `directive_verb` -> `directive_named` -> `directive_payload_suffix` -> `directive_payload_simple`. Regex AST schema version stays `1`.
- PGEN does not yet model PCRE2 `PCRE2_ALT_VERBNAMES` escape-processing semantics for verb names; under the stable `regex_default` profile, an unescaped `)` terminates the directive payload.

## Release 1.1.15 / Contract 1.1.16 Highlights
- `1.1.15` is a PCRE2-conformance directive-payload patch over the `1.1.14` parser release.
- This specifically covers RGX PCRE2 conformance reports `PGEN-RGX-0029` and `PGEN-RGX-0030`.
- MARK shorthand directives such as `(*:m(m)` and named directives such as `(*PRUNE:m(m)` now accept literal `(` inside the directive payload, so full patterns like `(*:m(m)(?&y)(?(DEFINE)(?<y>b))` and `(*PRUNE:m(m)(?&y)(?(DEFINE)(?<y>b))` parse as a directive atom followed by the named subroutine call and the `DEFINE` conditional.
- MARK shorthand payloads transport through `directive_verb` -> `directive_mark_shorthand` -> `directive_payload_simple`; named directive payloads transport through `directive_verb` -> `directive_named` -> `directive_payload_suffix` -> `directive_payload_simple`; regex AST schema version stays `1`.

## Release 1.1.14 / Contract 1.1.15 Highlights
- `1.1.14` is a PCRE2-conformance quoted-literal patch over the `1.1.13` parser release; integration contract `1.1.15` pins the downstream AST shape.
- This specifically covers RGX PCRE2 conformance report `PGEN-RGX-0023`.
- `abc\Q(*+|\Eabc` now treats the metacharacters inside `\Q...\E` as quoted literal payload instead of re-reading `(`, `*`, `+`, and `|` as active regex syntax.
- The quoted segment transports as a first-class `quoted_literal` atom whose rule text includes the delimiters, for example `\Q(*+|\E`.
- Downstream AST adapters should handle `quoted_literal` alongside `literal` and `escape`; regex AST schema version stays `1`.

## Contract 1.1.14 Highlights
- `1.1.14` is a downstream AST-contract clarification over parser release `1.1.13`; it does not change the regex grammar, parser release version, or AST dump schema version.
- This specifically covers RGX PCRE2 conformance reports `PGEN-RGX-0021`, `PGEN-RGX-0022`, and the mixed literal/POSIX-class reports `PGEN-RGX-0027` and `PGEN-RGX-0028`.
- `[[:space:]]+`, `[[:blank:]]+`, `^[:a[:digit:]]+`, and `^[:a[:digit:]:b]+` emit a `class_item` containing the first-class `posix_class` variant, with `posix_name = "space"`, `posix_name = "blank"`, or `posix_name = "digit"` respectively.
- Downstream AST adapters that walk character classes must handle `posix_class` alongside `class_range`, `class_literal`, and `class_escape` instead of treating it as an unknown `class_item` shape.

## Release 1.1.13 Highlights
- `1.1.13` is a PCRE2-conformance character-class recovery patch over the `1.1.12` downstream handoff.
- The headline change in `1.1.13` is accepting malformed POSIX-class opener text inside a character class when PCRE2 treats the second `[` as a literal fallback, such as `([[:]+)`.
- This specifically covers RGX PCRE2 conformance report `PGEN-RGX-0018`.
- The same bracket-literal fallback also covers the related malformed equivalence-opener spelling from `PGEN-RGX-0019`, `([[=]+)`, malformed collating-opener spelling from `PGEN-RGX-0020`, `([[.]+)`, nested literal-bracket class spelling from `PGEN-RGX-0024`, `[[,abc,]+]`, malformed POSIX-class body spelling from `PGEN-RGX-0025`, `[[:abcd:xyz]]`, and malformed POSIX-looking class text with an escaped `]` from `PGEN-RGX-0026`, `[abc[:x\]pqr]`.
- The fix is deliberately narrow:
  - `posix_class` now wins before literal fallback inside `class_item`
  - `[` is allowed as a `class_literal` fallback only after the stricter POSIX-class path fails
  - the compile-style validator now mirrors that fallback instead of reporting an unterminated POSIX class for the same malformed opener text
  - regex AST schema version stays `1`
- `1.1.13` carries forward the `1.1.12` control-escape validator hardening and all prior regex parser contract guarantees.

## Release 1.1.12 Highlights
- `1.1.12` is a PCRE2-conformance validator patch over the `1.1.11` downstream handoff.
- The headline change in `1.1.12` is accepting PCRE2 control escapes whose target byte looks like regex syntax, such as `^\ca\cA\c[;\c:`.
- This specifically covers RGX PCRE2 conformance report `PGEN-RGX-0017`.
- The fix is deliberately narrow:
  - the regex grammar already transports `\cX` forms through `control_escape`
  - the compile-style validator now skips the complete `\cX` escape instead of leaving the target byte behind for later class/quantifier scans
  - regex AST schema version stays `1`
- `1.1.12` carries forward the `1.1.11` malformed counted-quantifier compatibility and all prior regex parser contract guarantees.

## Release 1.1.11 Highlights
- `1.1.11` is a PCRE2-conformance syntax-compatibility patch over the `1.1.10` downstream handoff.
- The headline change in `1.1.11` is accepting malformed counted-quantifier spellings as ordinary literal text when PCRE2 treats them that way, rather than rejecting the pattern during PGEN parseability.
- This specifically covers the RGX PCRE2 conformance cluster `PGEN-RGX-0040` through `PGEN-RGX-0049`, plus `PGEN-RGX-0052`, including forms such as:
  - `a{1,2,3}b`
  - `a{65536`
  - `X{`
  - `X{}`
  - `X{12ABC}`
  - `X{,9]`
  - `a{(?#XYZ),2}`
- The grammar change is deliberately narrow:
  - valid counted quantifiers still bind through `quantifier` before literal fallback
  - malformed brace forms now fall back through `literal_char`, so downstream RGX receives the literal pattern surface PCRE2 accepts
  - the compile-style validator still rejects truly invalid closed counted quantifiers such as inverted ranges and closed bounds above `65535`
- `1.1.11` strengthens the upstream regression surface for that widening:
  - `regex_parser_integration_contract_v1.json` now declares representative malformed counted-quantifier literal samples
  - the generated-backend parseability adapter now covers the full fixed RGX cluster
  - the compile validator now has an explicit PCRE2 literal-malformed-counted-quantifier regression test
- `1.1.11` carries forward the `1.1.10` PCRE2 VERSION conditional support and all prior regex parser contract guarantees.

## Release 1.1.10 Highlights
- `1.1.10` is a syntax-widening patch over the `1.1.9` downstream handoff.
- The headline change in `1.1.10` is publishing PCRE2 VERSION conditionals requested in RGX bug report `PGEN-RGX-0016`.
- `1.1.10` adds support for conditional forms such as:
  - `(?(VERSION>=10.0)cat|dog)`
  - `(?(VERSION >= 10)cat|dog)`
- `1.1.10` transports those condition bodies structurally as:
  - `version_condition`
  - `version_operator`
  - `version_number`
- This keeps RGX's parse-time short-circuit path simple: downstream can slice the complete `condition` body and evaluate it against its fixed PCRE2 compatibility version before constructing a runtime conditional node.
- `1.1.10` strengthens the upstream regression surface for that widening:
  - the published regex integration manifest now explicitly includes:
    - a compact VERSION comparison sample
    - a whitespace-bearing VERSION comparison sample with a missing minor component
  - the generated-backend integration tests now assert:
    - correct `version_condition`, `version_operator`, and `version_number` text preservation
    - absence of bare-name fallback on VERSION condition bodies
- `1.1.10` carries forward the `1.1.9` returned-capture subroutine widening:
  - `1.1.9` published PCRE2 `10.47+` returned-capture subroutine syntax requested in RGX feature request `PGEN-RGX-0015`.
  - `1.1.9` adds support for parenthesized subroutine-return forms such as:
    - `(?1(1))`
    - `(?&callee(+1,<cap>,'alt'))`
  - `1.1.9` transports those forms structurally as:
    - `subroutine_call`
    - `returned_capture_subroutine`
    - `subroutine_target`
    - `returned_capture_group_list`
    - `returned_capture_group`
  - `1.1.9` strengthens the upstream regression surface for that widening:
    - the published regex integration manifest now explicitly includes:
      - a numeric returned-capture subroutine sample
      - a named-target returned-capture subroutine sample with mixed numeric/named grouplist entries
    - the generated-backend integration tests now assert:
      - correct returned-capture spans for `(?1(1))`
      - absence of `inline_modifiers` misclassification on the widened subroutine surface
- `1.1.10` carries forward the `1.1.8` syntax and depth-resilience fixes:
  - non-ASCII literal atoms such as `🎉` now parse as real `literal` nodes instead of rejecting at byte `0`
  - mixed ASCII/UTF-8 literal runs such as `café` now preserve `literal = ["c", "a", "f", "é"]` instead of stopping at the first multibyte codepoint
  - nested capturing groups now accept at least `50` levels cleanly instead of tripping the generated parser's overly conservative recursion guard around depth `12`
- `1.1.10` also carries forward the public regex host hardening from `1.1.8`:
  - generated regex entrypoints now execute on a dedicated larger-stack worker thread
  - the generated recursion guard is widened but still bounded (`512`)
- `1.1.10` also carries forward the `1.1.8` regression-surface strengthening:
  - the published regex integration manifest now explicitly includes:
    - a pure Unicode literal sample (`🎉`)
    - a mixed ASCII/Unicode literal sample (`café`)
    - a `50`-level nested capturing-group sample
  - the generated-backend integration tests now assert:
    - exact literal text preservation for Unicode samples
    - exact nested capturing-group count for the `50`-level sample
- `1.1.10` carries forward the `1.1.7` accepted-tree disambiguation fix:
  - `(?(R)a|b)` and `(a)(?(R1)b|c)` now transport `condition` through `recursion_condition` instead of falling back to bare `name`
- `1.1.10` carries forward the `1.1.6` accepted-tree span-integrity fix:
  - tagged payloads such as `(?{native:validate_word})` now preserve `code_content = "validate_word"` instead of dropping the first payload byte and transporting `"alidate_word"`
- `1.1.10` carries forward the `1.1.5` accepted-tree fix and tagged-syntax widening:
  - tagged payloads such as `(?{lua:return true})` now transport as `code_block_lang` containing `code_lang` and `code_content` instead of being shadowed by `code_block_plain`
  - `rhai` is now published alongside `lua`, `js`, and `javascript` as a structurally preserved tagged source-body form
  - `native` and `wasm` are now published as structurally preserved tagged payload forms, while runtime/reference validation remains downstream-owned
- `1.1.10` carries forward the `1.1.4` accepted-tree fix:
  - numeric angle forms such as `\g<1>` now transport as `backreference` containing `subroutine_ref` / `signed_digits` instead of `simple_escape("g")` plus literal `<`, `1`, `>`
- `1.1.10` carries forward the `1.1.3` accepted-tree and host-validation fixes:
  - braced octal escapes such as `\o{101}` now transport as `escape` containing `octal_escape` / `octal_digits` instead of `simple_escape` plus counted quantifier
  - brace-style numeric escapes are skipped atomically during post-parse validation, so they are no longer re-read as counted quantifiers
- `1.1.10` carries forward the `1.1.2` syntax unblock:
  - named recursion conditions such as `(?(R&word)a|b)` now parse and transport as `conditional` plus `recursion_condition`
- `1.1.10` also carries forward the `1.1.1` accepted-tree correctness fixes:
  - whole-pattern recursion `(?R)` now classifies as `subroutine_call` / `subroutine_target` instead of `inline_modifiers`
  - numeric backreferences such as `\1` now classify as `backreference` instead of generic `escape`
  - explicit conditional false branches such as `(?(1)a|b)` now preserve separate `yes_branch` and `no_branch` spans
  - trailing quantifiers now bind to the final literal atom instead of an entire preceding literal run, so `ab+` now transports as `a` plus `b+`, not `(ab)+`-style grouping
- `1.1.10` also carries forward the `1.1.0` published syntax coverage:
  - negated POSIX classes such as `[[:^alnum:]]`
  - braced named backreferences such as `\k{name}`
  - bare-name and signed numeric conditional references
  - named recursion conditions such as `R&name` inside conditionals
  - left-open counted quantifiers such as `{,4}` and comma-only counted form `{,}`
- The generated regex host path in `1.1.10` also continues to enforce the compile-style validation contract added in `1.1.0`, so obvious compile-invalid forms no longer slip through as successful parses.
- The release is additionally backed by the maintained PCRE2 compile-oracle lane documented in `PGEN_USER_GUIDE.md`.

## Supporting Documents
- Public host API:
  - `rust/src/embedding_api.rs`
- Public API contract:
  - `rust/docs/EMBEDDING_API_CONTRACT.md`
- Published regex flavor and operator-facing guidance:
  - `PGEN_USER_GUIDE.md`
- Shared issue-reporting protocol:
  - `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
- Canonical released-parser bug ledger:
  - `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
- Family proof/status surfaces:
  - `LIVE_ACHIEVEMENT_STATUS.md`
  - `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`

## Stable Integration Surface
- Grammar family:
  - `regex`
- Stable host profile:
  - `regex_default`
- Stable metadata calls:
  - `embedding_api_contract()`
  - `parser_embedding_api_contract()`
- Stable regex convenience parse entry points:
  - `parse_regex_default(...)`
  - `parse_regex_default_with_limits(...)`
  - `parse_regex_default_result(...)`
  - `parse_regex_default_with_limits_result(...)`
- Stable regex convenience AST-dump entry points:
  - `parse_regex_default_ast_dump(...)`
  - `parse_regex_default_ast_dump_with_limits(...)`
- Stable generic grammar parse entry points:
  - `parse_grammar_profile(...)`
  - `parse_grammar_profile_with_limits(...)`
  - `parse_grammar_profile_result(...)`
  - `parse_grammar_profile_with_limits_result(...)`
- Stable generic grammar AST-dump entry points:
  - `parse_grammar_profile_ast_dump(...)`
  - `parse_grammar_profile_ast_dump_with_limits(...)`
  - `parse_grammar_profile_ast_dump_result(...)`
  - `parse_grammar_profile_ast_dump_with_limits_result(...)`
- Stable named-string entry points for bindings and adapters:
  - `parse_grammar_profile_named(...)`
  - `parse_grammar_profile_named_with_limits(...)`
  - `parse_grammar_profile_named_with_limits_result(...)`
  - `parse_grammar_profile_ast_dump_named(...)`
  - `parse_grammar_profile_ast_dump_named_with_limits(...)`
  - `parse_grammar_profile_ast_dump_named_with_limits_result(...)`
- Stable regex contract metadata available through `parser_embedding_api_contract()`:
  - `supports_regex_generated_backend`
  - `regex_integration_contract_version`
  - `regex_parser_release_version`
  - `regex_ast_dump_schema_version`
- Stable integration invariants:
  - `input_ownership_model=borrowed_str`
  - `parse_session_model=stateless_per_call`
    - this is a host-surface guarantee, not a claim that the generated parser implementation is internally stateless
    - the generated Rust parser remains stateful per parser instance during a call, carrying parse cursor, memoization, recursion-guard, and semantic-runtime state
    - downstream consumers should treat each public parse API call as an independent session and should not rely on cross-call parser state
  - `zero_copy_input_boundary=true`
  - deterministic by default

## Build / Availability Requirements
- Real downstream use should require the generated regex backend.
- Startup or build validation should inspect:
  - `parser_embedding_api_contract().supports_regex_generated_backend`
- If building directly from a PGEN checkout, enable the generated parser surface rather than relying on bootstrap-only builds.
- If the generated backend is unavailable, the stable failure mode is:
  - `E_BACKEND_UNAVAILABLE`

## Stable Diagnostics Contract
- Stable diagnostic codes:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`
  - `E_UNSUPPORTED_PROFILE`
- Parse diagnostics now expose a stable optional machine-localizable location object:
  - `location.byte_offset`
  - `location.line`
  - `location.column`
- The location object is emitted when the selected parser backend can localize the parse failure precisely.
- Regex parse failures through the generated regex backend are expected to populate this location object.

## Stable AST-Dump Schema Contract
- Regex AST-dump JSON is transport-stable and schema-stable at schema version `1`.
- Schema version `1` stabilizes the recursive node envelope and variant encoding:
  - top-level node object fields:
    - `rule_name`
    - `span`
    - `content`
  - `span` fields:
    - `start`
    - `end`
  - `content` is an externally tagged JSON object with exactly one active variant:
    - `Terminal`
    - `TransformedTerminal`
    - `Sequence`
    - `Alternative`
    - `Quantified`
- Representative shape:

```json
{
  "rule_name": "regex",
  "span": {
    "start": 0,
    "end": 3
  },
  "content": {
    "Sequence": [
      {
        "rule_name": "atom",
        "span": {
          "start": 0,
          "end": 1
        },
        "content": {
          "Terminal": "a"
        }
      }
    ]
  }
}
```

- This schema contract is about JSON shape, field names, and variant encoding.
- Integration contract `1.1.17` explicitly guarantees the default PCRE2 backtracking-control directive payload shape, including MARK/PRUNE/SKIP/THEN:
  - directive payload characters are any characters except the verb-closing `)` under the stable `regex_default` profile
  - `(*MARK:m'm)(*PRUNE:p"p)(*SKIP:s(s)` transports the three payloads as `directive_payload_simple` values `m'm`, `p"p`, and `s(s`
  - `(*THEN:m(m)(?&y)(?(DEFINE)(?<y>b))` transports the leading directive as `directive_name = "THEN"` and `directive_payload_simple = "m(m"`
  - MARK shorthand still uses `(*:NAME)`; named backtracking-control verbs use `(*VERB:NAME)` and transport the name through `directive_payload_suffix`
  - `PCRE2_ALT_VERBNAMES` escape-processing semantics are not modeled by `regex_default`
- Integration contract `1.1.16` explicitly guarantees PCRE2 MARK shorthand directive payloads with literal `(`:
  - `(*:m(m)(?&y)(?(DEFINE)(?<y>b))` transports the leading directive as `directive_verb` with `directive_mark_shorthand` payload `m(m`
  - `(*PRUNE:m(m)(?&y)(?(DEFINE)(?<y>b))` transports the leading directive as `directive_verb` with `directive_name = "PRUNE"` and `directive_payload_simple = "m(m"`
  - downstream consumers should treat literal `(` as payload text inside directive verbs until the verb-closing `)` is reached
- Integration contract `1.1.15` explicitly guarantees PCRE2 quoted literals as first-class `quoted_literal` atoms:
  - `abc\Q(*+|\Eabc` transports the quoted section through `quoted_literal` with rule text `\Q(*+|\E`
  - downstream consumers should treat `quoted_literal` as an atom-level payload sibling of `literal` and `escape`
- Integration contract `1.1.14` explicitly guarantees that valid POSIX character classes inside character classes remain transported as `class_item` -> `posix_class` rather than being degraded to literal text:
  - `[[:space:]]+` transports the POSIX class through `posix_class` with `posix_name = "space"`
  - `[[:blank:]]+` transports the POSIX class through `posix_class` with `posix_name = "blank"`
  - `^[:a[:digit:]]+` transports the leading `:` and `a` as `class_literal` items and the embedded POSIX class through `posix_class` with `posix_name = "digit"`
  - `^[:a[:digit:]:b]+` transports the surrounding `:`, `a`, `:`, and `b` as `class_literal` items while preserving the embedded `digit` POSIX class through `posix_class`
  - downstream consumers should treat `posix_class` as a first-class `class_item` variant alongside `class_range`, `class_literal`, and `class_escape`
- Parser release `1.1.16` specifically generalizes PCRE2-compatible directive payload transport to the default non-`)` verb-name shape while carrying forward parser release `1.1.15` literal-`(` directive payload support, parser release `1.1.14` PCRE2-compatible `\Q...\E` quoted literal transport, parser release `1.1.13` PCRE2-compatible fallback for malformed POSIX-class opener text inside character classes, control-escape validator hardening, malformed counted-quantifier literal spellings, VERSION conditionals, returned-capture subroutine syntax, Unicode literal support, and deeper nested-group headroom, all while keeping this JSON schema version stable:
  - `(*:m(m)(?&y)(?(DEFINE)(?<y>b))` now parses the leading `(*:m(m)` directive before the following subroutine call and `DEFINE` conditional instead of rejecting at byte `0`
  - `(*PRUNE:m(m)(?&y)(?(DEFINE)(?<y>b))` now parses the leading `(*PRUNE:m(m)` directive through the same payload-character widening
  - `abc\Q(*+|\Eabc` now transports the quoted metacharacter segment through `quoted_literal` instead of treating `(` as active group syntax
  - `([[:]+)` now treats the inner `[` and `:` as ordinary character-class literals once the stricter POSIX-class form fails
  - `([[=]+)` now likewise treats the inner `[` and `=` as ordinary character-class literals
  - `([[.]+)` now likewise treats the inner `[` and `.` as ordinary character-class literals
  - `[[,abc,]+]` now treats the inner `[` and comma-separated payload as ordinary character-class literals, then parses the trailing `]` as a literal atom after the class quantifier
  - `[[:abcd:xyz]]` now treats the malformed POSIX-class body as ordinary class literals, then parses the trailing `]` as a literal atom after the class
  - `^\ca\cA\c[;\c:` now treats `\c[` as a complete control escape instead of re-reading `[` as an unterminated character class opener
  - `a{1,2,3}b` now transports the malformed counted-quantifier body as literal text instead of rejecting after `a`
  - `X{`, `X{A`, `X{1234`, and `X{1,` now preserve the unterminated brace spellings as literals
  - `X{12ABC}`, `X{,9`, and `X{,9]` now preserve malformed alphanumeric/left-open brace forms as literals
  - `a{(?#XYZ),2}` now preserves the surrounding brace/comma/digit text while still transporting `(?#XYZ)` through `comment_group`
  - `(?(VERSION>=10.0)cat|dog)` now transports the condition as `version_condition` with `version_operator = ">="` and `version_number = "10.0"`
  - `(?(VERSION >= 10)cat|dog)` now preserves whitespace around the comparison operator and accepts a missing minor component
  - `(?1(1))` now transports as `subroutine_call` containing `returned_capture_subroutine`, `subroutine_target`, `returned_capture_group_list`, and `returned_capture_group`
  - `(?&callee(+1,<cap>,'alt'))` now preserves mixed numeric and named returned-capture grouplist entries without falling back to `inline_modifiers`
  - `🎉` now transports as a single `literal` node spanning the full UTF-8 codepoint
  - `café` now transports as four `literal` nodes, preserving `é` as the final multibyte atom
  - nested capturing groups remain accepted at least through `50` levels
  - `(?(R)a|b)` now transports `condition` through `recursion_condition`
  - `(a)(?(R1)b|c)` now transports `condition` through `recursion_condition` instead of `name`
  - `(?{native:validate_word})` now preserves `code_content = "validate_word"` instead of starting one byte late
  - `(?{lua:return true})` now transports as `code_block_lang` plus `code_lang`, not `code_block_plain`
  - `(?{rhai:...})`, `(?{native:...})`, and `(?{wasm:...})` are now accepted through the same tagged code-block structure
  - `\g<1>` now transports as outer `backreference` plus inner `subroutine_ref` / `signed_digits`, not `simple_escape("g")` plus literal `<`, `1`, `>`
  - `\o{101}` now transports as outer `escape` plus inner `octal_escape` / `octal_digits`, not `simple_escape` plus counted `quantifier`
  - `(?R)` now transports as `subroutine_call`
  - `\1` now transports as `backreference`
  - `(?(1)a|b)` now transports with separate `yes_branch` / `no_branch`
  - `(?(R&word)a|b)` now parses and transports `R&word` as `recursion_condition` while preserving nested `name = "word"`
  - `ab+` now transports with final-atom quantifier binding
  - brace-style numeric escapes no longer trip counted-quantifier validation during post-parse compile checks
- Downstream consumers that interpret specific `rule_name` values should pin to a parser release version and rerun their own AST compatibility suite on upgrade.
- This document does not promise stable internal Rust AST node types.

## Current External Hardening Baseline
- The maintained PCRE2 compile-oracle lane is now part of the downstream trust story for the regex release.
- Current tracked baseline:
  - `cases_executed=2195`
  - `expected_parse_ok_total=1613`
  - `expected_parse_fail_total=582`
  - `parse_expectation_match_total=1668`
  - `parse_expectation_mismatch_total=527`
  - `false_accept_total=325`
  - `false_reject_total=202`
- This does not reopen the closed `regex` family row by itself, but it is the main maintained future hardening lane for downstream trust widening.

## Published Regex Flavor Summary
- The currently published regex parser accepts:
  - empty regex
  - raw regex bodies, not host-language delimiter wrappers
  - alternation and concatenation
  - whole-pattern recursion `(?R)`
  - returned-capture subroutine calls such as `(?1(1))` and `(?&name(+1,<cap>))`
  - capturing, noncapturing, named, and atomic groups
  - lookahead and lookbehind assertions
  - greedy, lazy, and possessive quantifiers
  - counted quantifier forms such as `{3}`, `{2,}`, `{2,4}`, `{,4}`, and `{,}`
  - final-atom quantifier binding for literal runs, so `ab+` means literal `a` followed by quantified `b`
  - char classes, negated char classes, ranges, and POSIX classes
  - negated POSIX classes such as `[[:^alnum:]]`
  - anchors including `^`, `$`, `\A`, `\Z`, `\z`, `\b`, `\B`, and `\G`
  - backreferences including `\1`, `\k<name>`, `\k'name'`, and `\k{name}`, with numeric forms preserved as backreference constructs rather than generic escapes
  - subroutine-reference forms such as `\g{1}` and `\g<1>`, with numeric angle form preserved as `backreference` plus `subroutine_ref`
  - parenthesized returned-capture subroutine forms that preserve a comma-separated return grouplist
  - inline modifiers and scoped modifiers
  - conditional regex forms whose condition may be:
    - a lookaround assertion
    - a PCRE2 VERSION comparison such as `VERSION>=10.0` or `VERSION >= 10`
    - a bare name
    - an explicit name reference
    - digits
    - signed digits
    - a recursion condition such as `R`, `R1`, or `R&name`
  - explicit conditional false-branch transport, so `(?(1)a|b)` preserves distinct `yes_branch` and `no_branch` spans
  - named recursion conditions such as `(?(R&word)a|b)`
  - embedded code-block syntax such as `(?{...})` and language-tagged variants
- Embedded code-block parser-layer contract:
  - plain `(?{...})` is preserved as opaque generic payload
  - `lua`, `js`, `javascript`, and `rhai` payloads are preserved as opaque source-body payloads
  - `native` and `wasm` payloads are preserved as tagged opaque/reference-style payloads
  - parser-layer structural handling currently guarantees:
    - balanced braces
    - single-quoted strings
    - double-quoted strings
    - escaped characters
- Generated-host compile-contract safeguards:
  - the generated regex host path additionally rejects several obvious compile-invalid forms even if the raw grammar shape parsed successfully
  - currently enforced rejections include:
    - unsupported `\i`
    - counted quantifier minimum/maximum inversions such as `{5,4}`
    - counted quantifier bounds above `65535`
    - forbidden character-class escapes such as `[\B]`, `[\R]`, and `[\X]`
    - descending character-class ranges such as `[z-a]`
    - quantified anchors such as `^+`
    - variable-length lookbehind such as `(?<=a+)b`
- Character-class AST adapter contract:
  - `class_item` variants currently include `class_range`, `class_literal`, `class_escape`, and `posix_class`
  - `posix_class` carries the POSIX class spelling through `posix_name`, including names such as `space`, `blank`, `digit`, `alnum`, and `xdigit`
  - valid POSIX classes are intentionally not flattened into literal text, because downstream engines need to preserve their range semantics
- The current detailed flavor description and measured operational baseline live in `PGEN_USER_GUIDE.md`.
- Representative accepted examples for the current published flavor include:
  - `ab+`
  - `\o{101}`
  - `\g<1>`
  - `(?1(1))`
  - `(?&callee(+1,<cap>,'alt'))`
  - `(?(VERSION>=10.0)cat|dog)`
  - `(?(VERSION >= 10)cat|dog)`
  - `(?{lua:return true})`
  - `(?{rhai:let x = 1;})`
  - `(?{native:callback_name})`
  - `(?{wasm:module:function})`
  - `(?R)`
  - `(a)\1`
  - `(?(1)a|b)`
  - `(?<A>foo)-\k{A}`
- The current parser contract does not promise:
  - slash-delimited host literal parsing such as `/pattern/flags` as a dedicated wrapper syntax
  - arbitrary valid Lua, JavaScript, or Rhai source acceptance beyond the structural forms listed above
  - JavaScript comment/template-literal shielding or Lua long-bracket shielding as part of the current published parser contract
  - parser-owned validation of `native` / `wasm` reference payload formats
  - runtime execution semantics for embedded code blocks
  - semantic equivalence with every regex engine on earth
  - a stable typed Rust AST API beyond the JSON schema described above

## Downstream Integration Checklist
1. Depend on the stable host module `pgen::embedding_api`.
2. Use profile `regex_default`.
3. Check `parser_embedding_api_contract().supports_regex_generated_backend` at startup or build validation time.
4. Record both:
   - `parser_embedding_api_contract().regex_parser_release_version`
   - `parser_embedding_api_contract().regex_integration_contract_version`
   - and copy those exact values into bug reports instead of inferring them from memory or stale docs
5. Use `parse_regex_default_result(...)` for accept/reject + diagnostics flows.
6. Use `parse_regex_default_ast_dump(...)` only if JSON AST transport is genuinely needed.
7. Treat `E_BACKEND_UNAVAILABLE`, `E_PARSE_FAILURE`, and `E_INPUT_TOO_LARGE` as first-class expected error modes.
8. Keep a downstream-owned regex acceptance/rejection corpus and run it alongside PGEN's own gate stack.
9. When reporting a bug, follow the quick path below and the full protocol in `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.
10. Once a real bug is confirmed, expect it to be tracked under the regex family/profile in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` until the fix is released.

## Issue Reporting Quick Path
- Every actionable regex parser bug report should include:
  - PGEN commit ID or released crate version
  - regex parser release version
  - regex integration contract version
  - parser family/profile:
    - `regex`
    - `regex_default`
  - exact failing input file
  - expected vs actual behavior
  - pretty-printed `parser_embedding_api_contract()` JSON
  - pretty-printed parse outcome JSON
  - pretty-printed AST dump JSON when the parse succeeds but the structure is wrong
- If the downstream project can run a PGEN checkout, also capture parser trace artifacts:

```bash
export PGEN_TRACE_VERBOSITY=debug

cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe -- \
  --parse regex repro_input.txt \
  --profile regex_default \
  --trace \
  --trace-log-file pgen_trace.log

cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe -- \
  --parse-dump-ast-pretty regex repro_input.txt pgen_ast_dump.json \
  --profile regex_default \
  --trace \
  --trace-log-file pgen_trace.log
```

- If the downstream project only has the Rust embedding API, capture:
  - `parser_embedding_api_contract()`
  - `parse_regex_default(...)` or `parse_regex_default_result(...)`
  - `parse_regex_default_ast_dump(...)` when AST structure is relevant
- For version fields, copy directly from:
  - `parser_embedding_api_contract().regex_parser_release_version`
  - `parser_embedding_api_contract().regex_integration_contract_version`
- For a broader description of what the published regex parser is expected to accept, consult the regex-flavor section in `PGEN_USER_GUIDE.md`.
- Full reporting procedure:
  - `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`

## Minimal Rust Example
```rust
use pgen::embedding_api::{
    parse_regex_default_result, parser_embedding_api_contract,
};

let contract = parser_embedding_api_contract();
assert!(contract.supports_regex_generated_backend);
assert_eq!(contract.regex_parser_release_version, "1.1.11");

parse_regex_default_result(r"https?://[^\s]+")?;
```

## Validation / Release Gates
- Public host API stability:
  - `make -C rust embedding_api_gate`
  - `make -C rust regex_parser_integration_contract_gate`
- Embedded code-block structural contract:
  - `make -C rust regex_embedded_code_block_contract_gate`
- External corpus hardening:
  - `make -C rust regex_pcre2_textsafe_corpus_gate`
  - `make -C rust regex_pcre2_compile_oracle_gate`
- Family proof/closure:
  - `make -C rust regex_parser_family_status_gate`
  - `make -C rust regex_parser_family_status_contract_gate`
  - `make -C rust regex_combined_telemetry_contract_gate`

## What This Does Not Promise
- It does not promise stable internal generated parser types.
- It does not promise runtime execution semantics for embedded code blocks such as `(?{...})`.
- It does not promise arbitrary valid Lua/JavaScript/Rhai payload acceptance beyond the explicitly published structural forms.
- It does not promise parser-owned validation of `native` / `wasm` reference payload formats.
- It does not promise every regex dialect already supported elsewhere in the ecosystem.
- It does not promise that downstream consumers can ignore parser release versioning when depending on AST details.
