# Task Tree: EBNF-SOURCE-OF-TRUTH (the grammar is the single source of truth for the accepted language)

> **Status:** `active` (2026-06-07). **Family / slice-id prefix:** `PGEN-EBNF-SOT-<NNNN>`.
> **Frontier:** `.2` audit every grammar for out-of-band acceptance validation invisible to the generator.
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
- `.2` — **AUDIT (frontier):** enumerate every grammar's parse path (`parser_registry.rs`
  `parse_*_detail` / `parse_sample`) and list every hand-written acceptance check applied AFTER the
  structural parse that is NOT mirrored in the EBNF. `regex_compile_validation.rs` is the known instance;
  check SV/VHDL/SVPP/RTL/annotation parse paths for analogous out-of-band gates. Output: a per-grammar
  table {grammar → out-of-band checks → in-EBNF? → generator-aware?}.
- `.3` — **FIX regex** (the trigger): for each validator-rejected construct, decide encode-in-EBNF
  (e.g. gate `directive_name` to the recognized PCRE2 verb set via a keyword rule / `@predicate`; drop or
  gate `unicode_escape` since `\u` is unsupported, keeping `\x{…}`) vs relax-the-validator vs
  generator-consults-validator. Verify cert-coverage `sample_parse_failures → 0` for regex + RGX
  conformance unchanged. Each construct its own leaf, tools-first, one-at-a-time + measured.
- `.4` — **GENERALIZE:** apply `.3`'s resolutions to the `.2` audit's other grammars.
- `.5` — **ENFORCE:** a gate/lint so a new out-of-band acceptance check (a hand-written post-parse
  rejection not encoded in the EBNF) is flagged — keeping the EBNF the single source of truth.

## Current Frontier

- `.2` AUDIT — enumerate out-of-band acceptance validation per grammar.
- (then) `.3` FIX regex (`\u` + `(*verb)` — encode-in-EBNF vs relax-validator).

## Decisions

- The rule (above) is the binding principle. Recorded as decision
  [[project_ebnf_is_single_source_of_truth]] and KM `ebnf-single-source-of-truth`.
- Composes with — does NOT replace — `GRAMMAR-WELLFORMED` (the certifying-linter duality this protects).

## Verification log

- `.1` — tool-backed root cause (`parseability_probe`: `\u{b7a2}`→"unsupported regex escape \u";
  `(*xjDD)`→"unrecognized PCRE2 verb"; `\x{b7a2}`/`(?|a)`/`(?P>n)`/`(?(1)a)` parse). Source pinned:
  `rust/src/regex_compile_validation.rs`. Generator + EBNF confirmed to NOT reference it.

## Commit log

- `.1` → `PGEN-EBNF-SOT-0001` (this slice).

## Changelog

- 2026-06-07: tree created (`PGEN-EBNF-SOT-0001`), triggered by the regex cert-coverage `\u`/`(*verb)`
  finding routed from `GRAMMAR-WELLFORMED.H.1`/`.G.4.9`.
