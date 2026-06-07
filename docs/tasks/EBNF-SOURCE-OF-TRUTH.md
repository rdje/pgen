# Task Tree: EBNF-SOURCE-OF-TRUTH (the grammar is the single source of truth for the accepted language)

> **Status:** `active` (2026-06-07). **Family / slice-id prefix:** `PGEN-EBNF-SOT-<NNNN>`.
> **Frontier:** `.3` FIX regex (`.2` audit DONE — regex is the SOLE out-of-band-validator instance across all 12 parse paths).
> **Decision record:** [`project_ebnf_is_single_source_of_truth`](../decisions/project_ebnf_is_single_source_of_truth.md).
> **KM card:** [`docs/knowledge/ebnf-single-source-of-truth.md`](../knowledge/ebnf-single-source-of-truth.md).
> **Book chapter:** [`docs/book/src/quality-and-closure-model.md`](../book/src/quality-and-closure-model.md) (the loud rule).

## THE RULE (bold and loud)

> **THE EBNF — together with its `@predicate` / `@generate` / `@semantic` annotations — IS THE SINGLE
> SOURCE OF TRUTH FOR WHAT A PGEN PARSER ACCEPTS.** The stimuli generator derives samples from the EBNF
> (its structure + its semantic annotations) and NOTHING ELSE. Therefore **any acceptance constraint that
> lives OUTSIDE the EBNF — in a hand-written post-parse validation layer — is INVISIBLE to the generator,
> so the generator WILL emit structurally-valid samples the parser rejects, silently breaking the
> generator⟷parser duality** (and the certifying-linter's "generation constructively corroborates the
> grammar" guarantee). An out-of-band acceptance gate the generator cannot see is a **DEFECT**. Every
> acceptance constraint MUST be encoded IN the EBNF (as a semantic annotation, the one shared source of
> truth for both generation and parsing) — or it must be removed/relaxed. The grammar defines the
> language; a separate validator must never silently narrow it.

## The frame (why this tree exists)

The certifying-linter doctrine ([[GRAMMAR-WELLFORMED]]) rests on a DUALITY: the linter PROVES the grammar
well-formed (static side), and the stimuli generator CONSTRUCTIVELY CORROBORATES it by generating samples
that re-parse (constructive side). That duality is **only valid if the EBNF is the complete spec of the
accepted language.** This tree exists because we found a case where it is NOT.

### Trigger (the tool-backed root cause, 2026-06-07)

`GRAMMAR-WELLFORMED.H.1` enabled certificate-coverage for `regex`; it reported `sample_parse_failures=6`.
Investigation (`PGEN-GRAMMAR-WELLFORMED-0035` then this tree), tools-first via `parseability_probe`:

- The EBNF **structurally accepts** the failing constructs: `unicode_escape = "u{" hex_digits "}"`
  generates `\u{…}`; `directive_verb = "(*" directive_body ")"` (with `directive_body = directive_named
  | directive_mark_shorthand`, `directive_named = directive_name …`) generates `(*<any-name>)`.
- The rejection is **NOT** structural / PEG-ordering. `parseability_probe --parse regex` returns explicit
  SEMANTIC errors: `\u{b7a2}` → **"unsupported regex escape \u"**; `(*xjDD)` → **"unrecognized PCRE2 verb
  or start option"**. `\x{b7a2}` (the supported braced-hex form) PARSES; `(?|a)` / `(?P>n)` / `(?(1)a)`
  PARSE (so the earlier G.4.9 list over-claimed — only `\u` and unrecognized `(*verb)` actually fail).
- Source of the rejection: **`rust/src/regex_compile_validation.rs`** (`validate_regex_compile_contract`),
  a hand-written post-parse PCRE2-compile-contract check run by `parse_with_regex_detail`. It is **NOT**
  referenced by the stimuli generator (`grep` of `stimuli_generator.rs` = 0) and **NOT** encoded in the
  EBNF as `@predicate`/`@generate` (`grep` of `regex.ebnf` = 0).

⇒ The regex parser's accepted language = (structural `regex.ebnf`) ∩ (`regex_compile_validation.rs`). The
generator targets only the first set. So it generates `\u{…}` / `(*xjDD)` — valid per the EBNF, rejected
by the validator. **The EBNF is broader than the accepted language; the gap lives out-of-band.** This is
the general defect class this tree owns.

## Acceptance (tree-level)

- The rule above is documented LOUD in the book, a KM card, and a decision record (this slice).
- Every grammar is audited for out-of-band acceptance validation invisible to the generator (`.2`).
- For each gap, the constraint is either encoded in the EBNF (semantic annotation) or the out-of-band
  validation removed/relaxed — so `generation ⊆ accepted-language` holds and cert-coverage
  `sample_parse_failures → 0` from the general mechanism (`.3`/`.4`).
- An enforcement so a new out-of-band acceptance gate cannot silently reappear (`.5`).

## Non-goals

- Re-deriving PCRE2 semantics in the EBNF wholesale. The point is consistency (generator sees what the
  parser enforces), not moving every byte of validation into the grammar — relaxing/removing an
  over-strict out-of-band check is an equally valid resolution.

## Task tree

- `.1` — **DONE (this slice, `PGEN-EBNF-SOT-0001`):** name the rule + root-cause the regex trigger
  (above) + write it LOUD in the book + KM card + decision record + this tree. Correct the G.4.9
  over-claim (only `\u`/`(*verb)` fail; `(?|)`/`(?P>)`/`(?(…))` parse).
- `.2` — **AUDIT (DONE, `PGEN-EBNF-SOT-0002`):** enumerated every grammar's parse path
  (`parser_registry.rs` `parse_sample_detail_with_profile` dispatch → the 12 `parse_with_*_detail`
  functions) and inspected each for a hand-written acceptance check applied AFTER the structural parse
  that is NOT mirrored in the EBNF. **RESULT: `regex` is the SOLE instance** of the out-of-band
  acceptance-validator defect class across all 12 parse paths. See the audit table below.
- `.3` — **FIX regex** (the trigger): for each validator-rejected construct, decide encode-in-EBNF
  (e.g. gate `directive_name` to the recognized PCRE2 verb set via a keyword rule / `@predicate`; drop or
  gate `unicode_escape` since `\u` is unsupported, keeping `\x{…}`) vs relax-the-validator vs
  generator-consults-validator. Verify cert-coverage `sample_parse_failures → 0` for regex + RGX
  conformance unchanged. Each construct its own leaf, tools-first, one-at-a-time + measured.
- `.4` — **GENERALIZE:** apply `.3`'s resolutions to the `.2` audit's other grammars.
- `.5` — **ENFORCE:** a gate/lint so a new out-of-band acceptance check (a hand-written post-parse
  rejection not encoded in the EBNF) is flagged — keeping the EBNF the single source of truth.

## `.2` Audit result (2026-06-07, `PGEN-EBNF-SOT-0002`) — tools-first

**Method.** All parse paths route through `parser_registry::parse_sample_detail_with_profile`
(`parser_registry.rs:873`), which dispatches each grammar to a `parse_with_<grammar>_detail`
function. Read every one of the 12 bodies and checked for any Rust-level acceptance check applied
*after* the generated parser's `parse_full_*()` call that is not encoded in the EBNF / not seen by
the generator (`grep` of `stimuli_generator.rs` + the grammar `.ebnf`).

**Per-grammar table {grammar → out-of-band post-parse acceptance check → in EBNF? → generator-aware? → verdict}:**

| Grammar | `parse_*_detail` | Out-of-band acceptance check after parse | In EBNF? | Generator-aware? | Verdict |
| --- | --- | --- | --- | --- | --- |
| **regex** | `parse_with_regex_detail` (:301) | **YES — `validate_regex_compile_contract`** (`regex_compile_validation.rs`) | NO | NO | **DEFECT** (the trigger → `.3`) |
| return_annotation | `parse_with_return_annotation_detail` | none (only `parse_full_*` + map_err) | n/a | yes | clean |
| semantic_annotation | `parse_with_semantic_annotation_detail` | none | n/a | yes | clean |
| builtin_return_annotation | → `parse_with_return_annotation_detail` | none | n/a | yes | clean |
| builtin_semantic_annotation | `UnifiedSemanticAST::parse_bootstrap` | none | n/a | yes | clean |
| ebnf | `parse_with_ebnf_detail` | none | n/a | yes | clean |
| json | `parse_with_json_detail` | none | n/a | yes | clean |
| rtl_const_expr | `parse_with_rtl_const_expr_detail` | none | n/a | yes | clean |
| rtl_frontend | `parse_with_rtl_frontend_detail` | none | n/a | yes | clean |
| systemverilog | `parse_with_systemverilog_detail_profile` (:501) | none¹ | n/a | yes | clean |
| systemverilog_preprocessor | `parse_with_systemverilog_preprocessor_detail` | none | n/a | yes | clean |
| vhdl | `parse_with_vhdl_detail` | none | n/a | yes | clean |

¹ The SV path has two extra steps that are **not** out-of-band acceptance narrowing: (a)
`preload_systemverilog_stdlib` runs *before* the parse and is the in-EBNF `@import_from_library`
mechanism (generator-aware); (b) `furthest_position` only *enriches the error message* on an
already-failing parse — it never changes accept/reject. The `@import`/`@export` library-options path
(`parse_with_systemverilog_detail_profile_with_library`) is likewise in-EBNF semantic annotations.

**The regex out-of-band surface (full, feeds `.3`/`.4`).** `validate_regex_compile_contract`
(`regex_compile_validation.rs:18`) is broader than the `\u`/`(*verb)` subset found in `.1` — it runs
**10** PCRE2-compile sub-checks, each able to reject an EBNF-structurally-valid pattern:
`find_invalid_escape_i` (the `\u` case), `find_invalid_property_escape`,
`find_invalid_named_escape_or_group_name` (PCRE2 name ≤ 128), `find_invalid_counted_quantifier`,
`find_invalid_numeric_callout`, `find_invalid_verb_construct` (the unrecognized-`(*verb)` case),
`find_invalid_char_class_construct`, `find_invalid_quantified_anchor`,
`find_invalid_scan_substring_capture_list`, `find_invalid_keep_out_escape_in_lookaround`.

**Not in scope (confirmed):** `ast_pipeline/annotation_validator.rs` is a *build-time* validator of the
`@return`/`@semantic` annotation DSL during grammar compilation (in-band, generator-aware by
definition); it is not referenced by `parser_registry.rs` and is not a runtime sample-acceptance gate.
`parse_and_cover_regex` (the cert-coverage witness path) deliberately does NOT apply the validator
(`parser_registry.rs:324`) — it asks "did the grammar parse", not "is it PCRE2-valid".

**Consequence for the tree.** Because the audit found regex is the *only* instance, `.4` (generalize to
other grammars) has no other targets — it reduces to "no additional grammars affected; `.5` enforcement
prevents recurrence." The remaining substantive work is `.3` (fix regex) + `.5` (enforce).

## Current Frontier

- `.3` FIX regex — for each of the 10 validator sub-checks (starting with the two known generator-tripping
  cases, `\u` via `find_invalid_escape_i` and unrecognized `(*verb)` via `find_invalid_verb_construct`):
  decide encode-in-EBNF (semantic annotation / keyword-gated rule) vs relax/remove the out-of-band check;
  verify cert-coverage `sample_parse_failures → 0` + RGX conformance unchanged. Each construct its own
  leaf, tools-first, one-at-a-time + measured.
- (then) `.5` ENFORCE — a gate/lint flagging any new hand-written post-parse acceptance check not encoded
  in the EBNF.
- `.4` GENERALIZE — NO-OP per the `.2` audit (regex is the sole instance); kept as a checkpoint to
  re-run the audit after `.5` lands.

## Decisions

- The rule (above) is the binding principle. Recorded as decision
  [[project_ebnf_is_single_source_of_truth]] and KM `ebnf-single-source-of-truth`.
- Composes with — does NOT replace — `GRAMMAR-WELLFORMED` (the certifying-linter duality this protects).

## Verification log

- `.1` — tool-backed root cause (`parseability_probe`: `\u{b7a2}`→"unsupported regex escape \u";
  `(*xjDD)`→"unrecognized PCRE2 verb"; `\x{b7a2}`/`(?|a)`/`(?P>n)`/`(?(1)a)` parse). Source pinned:
  `rust/src/regex_compile_validation.rs`. Generator + EBNF confirmed to NOT reference it.
- `.2` — read all 12 `parse_with_*_detail` bodies in `parser_registry.rs`; only `parse_with_regex_detail`
  (:301) applies a post-parse check (`validate_regex_compile_contract`, :305). Enumerated its 10
  sub-validators (`grep find_invalid_*` over `regex_compile_validation.rs:18-52`). Confirmed
  `annotation_validator.rs` is build-time (not referenced by `parser_registry.rs`). Pure-docs audit; no
  code change. `mdbook_docs_gate` green; docpath + memarch guards green.

## Commit log

- `.1` → `PGEN-EBNF-SOT-0001`.
- `.2` → `PGEN-EBNF-SOT-0002` (this slice).

## Changelog

- 2026-06-07: tree created (`PGEN-EBNF-SOT-0001`), triggered by the regex cert-coverage `\u`/`(*verb)`
  finding routed from `GRAMMAR-WELLFORMED.H.1`/`.G.4.9`.
- 2026-06-07: `.2` audit DONE (`PGEN-EBNF-SOT-0002`) — regex is the sole out-of-band-validator instance
  across all 12 parse paths; full 10-check surface recorded; `.4` reduced to no-op; frontier → `.3`.
