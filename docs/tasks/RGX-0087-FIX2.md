# RGX-0087-FIX2: scope the `[89]`-leading multi-digit hard-reject to NON-character-class context (regression from this session's own `PGEN-RGX-0087-0001` fix)

## Metadata

- Tree ID: `RGX-0087-FIX2`
- Status: `active`
- Roadmap lane: released-parser bug remediation (`regex` family; downstream `PGEN-RGX-0087`, still `status:open`)
- Created: `2026-05-18`
- Owner: repo-local workflow
- Priority: user-directed (**regression from THIS session's `PGEN-RGX-0087-0001` fix `a81d7317` / release 1.1.78** — I own it; takes precedence; `SV-EXH-PROOF.3.2` frontier paused). Extra rigor: regression-from-own-fix ⇒ no assumed fix, oracle-derive the full matrix before/after, prove no NEW regression.

## Goal

Fix `PGEN-RGX-0087` properly. The rel-1.1.78 attempt
(`PGEN-RGX-0087-0001`, tree `RGX-0087` — its `RGX-0087.md`) **closed
the two originally-reported cases** (`testinput2:4671` `\81`,
`:4674` `\80` now correctly REJECT) **but is OVER-BROAD** and netted
−4 on the RGX PCRE2 differential ratchet (12,805/5 → 12,801/9): the
two negative-lookahead guards also fire where they must not.
Re-scope so PGEN matches the authoritative PCRE2 10.47 oracle on the
**whole** matrix, introducing **zero** new divergence.

## Root cause (report + read-only grammar + PCRE2 10.47 oracle — grounded, to be empirically re-verified in `.1`)

`PGEN-RGX-0087-0001` added two `!"0"…!"9"` negative-lookahead guards
in `grammars/regex.ebnf`: on `numeric_backreference_single` (:273)
and on `simple_escape` (:546). Both are over-broad:

1. **Character-class over-rejection (the dominant regression — 5 of
   6 losses).** `class_escape = escape` (`regex.ebnf:414`) reuses the
   shared outer `escape → escape_unit → simple_escape`. With the
   `simple_escape` digit-guard, a single class-member `\8`/`\9`
   (`[\8]`, `[\9]`, `^[A\8B\9C]+$`) no longer matches → the class
   `E_PARSE_FAILURE`s. PCRE2 10.47: **ACCEPT** (a character class has
   no back-references; `\8`/`\9` there are octal/literal). The range
   path `class_range_escape` has its OWN unguarded
   `class_range_simple_escape` (:449) so `[\8-\9]` still ACCEPTs →
   the internal inconsistency the report flags.
2. **`[1-7]`-leading long octal run (`\6666666666`, testinput9:287
   `(?i:A{1,}\6666666666)`).** Pre-1.1.78 PGEN matched `\6` via the
   *unguarded* `numeric_backreference_single` → backref-6 → (0 groups)
   → REJECT, *accidentally* matching PCRE2's reject. The
   `numeric_backreference_single` guard removed that accident:
   `\6666666666` now backtracks to `octal_escape`
   (`/([0-7]{1,3})/` → `\666`) + literals → **ACCEPT**. PCRE2 10.47:
   **REJECT** error 151 "octal value is greater than \377 in 8-bit
   non-UTF-8 mode" — PGEN's `octal_escape` performs **no
   octal-range validation**. This is a distinct mechanism (octal
   overflow), exposed (not strictly *caused*) by the guard; treat as
   a sibling sub-leaf if a clean scoped fix isn't shared.

The originally-targeted non-class `[89]`-leading cases stay correctly
REJECTed (`\81`@8g, `\82`@8g, `\91`@8g, `\89`@0g, `\80`@0g,
`(x)\81`@1g, `a\8`, `\8a`) and `[1-7]`-led octal-degrade stays correct
(`\199`@0g → `\x01`+"99", `\10`@9g → octal) — those must remain
byte-identical.

## Authoritative PCRE2 10.47 oracle matrix (independent of fix/report — `feedback_corpus_expected_from_spec_not_fix` / `feedback_report_expected_verify_against_oracle`)

| input | PCRE2 10.47 | rel-1.1.78 | target |
| --- | --- | --- | --- |
| `[\8]` `[\9]` `^[A\8B\9C]+$` `[\88]` `[\89]` | **ACCEPT** | REJECT (WRONG) | ACCEPT |
| `[\8-\9]` `[\377]` | ACCEPT | ACCEPT | ACCEPT (consistent) |
| `(?i:A{1,}\6666666666)` | **REJECT** (err 151 octal>\377) | ACCEPT (WRONG) | REJECT |
| `((((((((x))))))))\81` `\82` `\91` | REJECT (err 115) | REJECT ✓ | REJECT |
| `\89`@0g `\80`@0g `(x)\81`@1g `a\8` `\8a` `\8`@0g | REJECT (err 115) | REJECT ✓ | REJECT |
| `\199`@0g | ACCEPT (`\x01`+"99") | ACCEPT ✓ | ACCEPT |
| `(((((((((x)))))))))\10` `\377` | ACCEPT | ACCEPT ✓ | ACCEPT |

## Non-Goals

- NOT reopening/altering `PGEN-RGX-0084` (`\10` forward-ref) or
  single-digit `\1`–`\9` (N<10) — byte-identical.
- NOT reverting `PGEN-RGX-0087-0001`'s correct non-class
  `[89]`-leading hard-reject (4671/4674 must stay closed) nor the
  `[1-7]`-led octal-degrade (`\199`/`\10`).
- NOT an RGX-side workaround (`feedback_no_pgen_workarounds` /
  `feedback_family_fix_doctrine`); PGEN owns the parse-time fix.

## Acceptance Criteria

- Full oracle matrix above verified via `parseability_probe
  --parse[-dump-ast-pretty] regex … --profile regex_default` AND the
  embedding entry `parse_grammar_profile_named` (report verification
  step 2), expecteds from `pcre2test` 10.47 — **zero** new divergence;
  the 5 class-context cases + testinput9:287 now match PCRE2; the
  4671/4674/`\89`/`\199`/`\10` set stays byte-identical.
- No regression: `regex` lib (RGX-0079..0086 pins + the new
  `regex_parser_pgen_rgx_0087_*` pin extended for class-context +
  octal-overflow), `regex_ast_shape_contract_holds_against_running
  _generated_parser`, `make -C rust regex_parser_integration_contract
  _gate`, cross-parser `ast_shape_contract`, regex/mdBook book gates.
  AST-dump schema unchanged if no new shape vocab.
- Released-parser-bug remediation end-to-end: update ledger row
  `REGEX-0086` (record the over-broad 1.1.78 attempt + the scoped
  fix), regex release/contract bump (RGX-0086 drift gate keeps the
  embedding consts synced), regex book changelog-index + escapes
  worked-family table + integration-contract Highlights, top-level
  `parser-families.md`, CHANGES/LIVE/memory — same-commit lockstep
  (binding). Standing push rule (push every 30).

## Task Tree

- ID: `RGX-0087-FIX2`
  Status: `active`
  Goal: `Scope the [89]-leading multi-digit hard-reject to non-character-class pattern-body context; cover the testinput9:287 octal-overflow; zero new divergence vs PCRE2 10.47; remediate PGEN-RGX-0087 end to end. RGX-0084 + the correct part of RGX-0087-0001 stay intact.`
  Children: `RGX-0087-FIX2.1`, `RGX-0087-FIX2.2`

- ID: `RGX-0087-FIX2.1`
  Status: `pending` (frontier; root cause grounded above — EMPIRICALLY VERIFY before+after, regression from own fix)
  Goal: `Empirically pin the rel-1.1.78 baseline on the full oracle matrix; design + implement the scoped regex.ebnf fix (class-member escape path must accept \8/\9 octal-literal; non-class [89]-led hard-reject preserved; testinput9:287 octal>\377 — scoped or sibling); regen; probe-verify the WHOLE matrix byte-identical-where-required + RGX-0084/0079..0086 no-regression + cross-parser + gates. Consult annotation docs + proven idiom + SEMREF-SHAPED contract FIRST (binding, hook-enforced).`
  Acceptance: `behavior half of Acceptance Criteria; zero new divergence; the 4671/4674/\89/\199/\10 set byte-identical.`
  Verification: `root cause grounded 2026-05-18 (report + regex.ebnf:273/414/443/449/485/490/546 + pcre2test 10.47 matrix); empirical verify + fix is .1`
  Commit: `pending`

- ID: `RGX-0087-FIX2.2`
  Status: `pending`
  Goal: `Released-parser-bug closure: ledger REGEX-0086 updated (over-broad 1.1.78 attempt + scoped fix), regex release+contract bump (drift gate synced), regex book + integration-contract + parser-families.md + CHANGES/LIVE/memory same-commit lockstep; tree closed; SV-EXH-PROOF.3.2 resumes.`
  Acceptance: `Ledger updated; release/contract cut; full books↔code lockstep; drift gate green at the new version; tree closed + Completed; SV-EXH-PROOF.3.2 the resumed frontier.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `RGX-0087-FIX2.1` | `pending` | Root cause grounded (class_escape reuses the digit-guarded shared simple_escape; numeric_backreference_single guard rerouted [1-7]-led long runs to unvalidated octal). Scoped fix + full oracle-matrix no-new-divergence verification. |
| 2 | `RGX-0087-FIX2.2` | `pending` | Ledger/release/book/contract lockstep + closeout (needs `.1`). Then `SV-EXH-PROOF.3.2` resumes. |

## Decisions

- `2026-05-18`: RGX reported `PGEN-RGX-0087` stays `open` — rel-1.1.78
  (`a81d7317`) over-broad (class-context over-rejection + testinput9
  :287). User-directed; family-linked regression from this session's
  own fix ⇒ extra rigor (oracle-derive the whole matrix; no assumed
  fix; prove zero NEW divergence; the correct part of 1.1.78 stays).
  Code-Change Doctrine: tree owns the forthcoming `.ebnf` edit.
  `SV-EXH-PROOF.3.2` frontier paused for this priority interrupt.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-18` | `RGX-0087-FIX2` (setup + root cause grounded) | Read updated `PGEN-RGX-0087.yaml` + artifacts (`pgen_ast_dump_classctx.json`, `pgen_parse_outcome_classctx.json`, `repro_input_classctx.txt`, `pgen_ast_dump_81.json`); read `regex.ebnf` escape/class architecture (`:273` numeric_backreference_single guard, `:414` class_escape=escape, `:443/:449` class_range_escape/class_range_simple_escape, `:485/:490/:546` escape/escape_unit/simple_escape guard); ran `pcre2test` 10.47 over the full matrix incl. the 5 class-context cases + testinput9:287 + `[\88]`/`[\89]`/`a\8`/`\8a` + the originals. | `pass — root cause grounded (report + grammar + independent oracle): (1) class_escape reuses the digit-guarded shared simple_escape ⇒ single class-member \8/\9 over-reject (range path separate+unguarded ⇒ inconsistency); (2) numeric_backreference_single guard rerouted [1-7]-led long runs off accidental single-backref-reject onto PGEN's unvalidated octal_escape (no >\377 check) ⇒ \6666666666 wrongly accepted. Oracle matrix pinned (independent of fix/report). NOT yet empirically reproduced on the running 1.1.78 parser (the regression is from my own fix ⇒ `.1` must probe before/after, no assumed fix). Tree created/activated (Code-Change Doctrine). No code. Standing push rule: push @30.` |

## Commit Log

| Slice | Commit ID | Note |
| --- | --- | --- |
| `RGX-0087-FIX2` setup + root cause grounded | `PGEN-RGX-0087-FIX2-0000` | tree created + activated; report + grammar + pcre2test-10.47 grounded root cause + oracle matrix; empirical verify + scoped fix deferred to `.1` (docs-only) |

## Changelog

- `2026-05-18`: Created + activated. `PGEN-RGX-0087` stays open —
  the rel-1.1.78 `PGEN-RGX-0087-0001` fix is over-broad
  (class-context over-rejection of `\8`/`\9`; testinput9:287
  octal-overflow now accepted) netting −4 on the RGX conformance
  ratchet. Root cause grounded (report + read-only `regex.ebnf` +
  independent `pcre2test` 10.47 matrix); scoped fix + before/after
  empirical proof is `.1`. `SV-EXH-PROOF.3.2` paused.
