# REGEX-CODE-HOOKS: the RGX embedded-code-hook family — `(?{...})` (landed) + `(??{...})` (to add via the same mechanism)

## Metadata

- Tree ID: `REGEX-CODE-HOOKS`
- Status: `active` (created 2026-07-09 session #73; `.1` documents already-landed `(?{...})`; `.2` `(??{...})` is the near-term frontier)
- Roadmap lane: regex feature-completeness (RGX-hook extensions) — a prerequisite for `RGX-0078` (speed)
- Created: `2026-07-09`
- Owner: repo-local workflow
- Priority: director-directed (2026-07-09, session #73): PGEN regex already delegates execution of
  `(?{...})` hooks to backend languages (RGX executes them); `(??{...})` is "what's left" — use the
  SAME language-indicator mechanism.
- Related: [[project_regex_perl5_feature_gap_direction]] (the broader feature roadmap) · [[project_regex_pcre2_faithful_by_default]]

## Goal

Complete the PGEN/RGX **embedded-code-hook** family. PGEN's regex already supports `(?{...})` and the
language-tagged `(?{lang: ...})` form as a PGEN EXTENSION (NOT PCRE2, and broader than Perl's
Perl-only `(?{...})` — PGEN tags a BACKEND language and RGX executes it). The postponed / dynamic
sibling `(??{...})` (Perl's "postponed subexpression" — the code returns a subpattern that is then
matched) is not yet in the grammar. Add it via the SAME mechanism.

## Existing mechanism (tool-verified in `grammars/regex.ebnf`, session #73)

```ebnf
code_block       = code_block_lang | code_block_plain
code_block_plain = "(?{" code_content "})"                   -> {type:"atom", kind:"code_block", lang:null, content:$2}
code_block_lang  = "(?{" code_lang ":" ws? code_content "})" -> {type:"atom", kind:"code_block", lang:$2, content:$5}
code_lang        = "lua" | "js" | "javascript" | "rhai" | "native" | "wasm"
```

- `code_content` is a brace-balanced, string-literal-aware payload (`code_element*`), so nested `{…}`
  and quoted strings inside the hook are handled.
- **Parser-layer contract (`regex.ebnf:1659-1663`):** plain `(?{...})` = opaque generic payload;
  `lua`/`js`/`javascript`/`rhai` = tagged source bodies; `native`/`wasm` = tagged reference-style
  payloads; **runtime execution semantics are OUT OF SCOPE for PGEN** — RGX (the consumer) executes
  the tagged payload via the named backend. The AST carrier is `{kind:"code_block", lang, content}`.

## Task Tree

- ID: `REGEX-CODE-HOOKS`  Status: `active`  Children: `.1`..`.2`
- ID: `.1`  Status: `done` (pre-existing, documented here 2026-07-09)  Goal: `(?{...})` / `(?{lang:...})`
  embedded code hooks with the `code_lang` backend set (`lua`/`js`/`javascript`/`rhai`/`native`/`wasm`).
  Landed in `regex.ebnf` (`code_block` family). This leaf records the mechanism as the template for `.2`.
- ID: `.2`  Status: `pending`  Goal: add `(??{...})` / `(??{lang: ...})` — the POSTPONED / dynamic sibling
  — mirroring `code_block`. Proposed shape: a `dynamic_code_block` rule (`"(??{" …` / `"(??{" code_lang
  ":" ws? code_content "})"`) reusing `code_lang` + `code_content` verbatim, emitting a DISTINCT carrier
  so consumers can dispatch on postponed-vs-immediate — e.g. `{type:"atom", kind:"dynamic_code_block",
  lang, content}` (the `python_named_group` / `python_named_backreference` precedent for a distinct
  `kind`). Wire it into the atom/group alternation next to `code_block`. Same "runtime execution out of
  scope" contract — RGX executes the postponed hook and matches its returned subpattern. Released regex
  slice: regen + full lockstep (manifest / book / contract / ledger / release bump) + cert re-baseline +
  a pin test + the `pcre2test` note (PCRE2 does NOT support `(??{...})`, so it is a PGEN-extension accept,
  relaxed-or-default per the profile decision — assess against the `@profiles` model in `.2` design).
  Owns the acceptance-checklist (root-cause is a feature ADD, so REPRODUCE = "grammar rejects `(??{...})`
  today"; ADDRESSED = "accepts + shapes it"; NO-REGRESSION = the standard regex gate suite).

## Acceptance Criteria

- `(??{...})` and `(??{lang: ...})` parse and emit a `dynamic_code_block` (or agreed) carrier with
  `lang` ∈ the `code_lang` set (or `null`) and a brace-balanced `content`; byte-shape pinned by a
  manifest entry + a `parser_registry` pin test.
- No regression: regex cert `fully_certified` at seeds 0/7/42; `regex_pcre2_compile_oracle_gate`
  byte-identical (the new form is not in the PCRE2 corpus); equivalence/duality/ast_shape gates green.
- Lockstep: regex book (a code-hooks section covering BOTH `(?{...})` and `(??{...})`), contract +
  ledger + release version, top-book parser-families chain.

## Notes

- This family is a **PGEN/RGX extension**, distinct from PCRE2 parity (`REGEX-PCRE2-FIDELITY`) and from
  the Perl5-only matrix ([[project_regex_perl5_feature_gap_direction]]). It contributes to "regex is
  feature-complete", which is the PRECONDITION for `RGX-0078` (speed).
