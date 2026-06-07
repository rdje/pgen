# REGEX-PCRE2-FIDELITY: regex accepts/rejects exactly what PCRE2 does (default), with a relaxed opt-out — EBNF-driven

## Metadata

- Tree ID: `REGEX-PCRE2-FIDELITY`
- Status: `active`
- Family / slice-id prefix: `PGEN-REGEX-PCRE2-<NNNN>`
- Roadmap lane: regex parser quality — PCRE2-faithful-by-default acceptance with an opt-out relaxed mode,
  encoded in the EBNF via semantic annotations (engine = last resort)
- Created: `2026-06-07`
- Last updated: `2026-06-07`
- Owner: repo-local workflow
- Director directive: [[project_regex_pcre2_faithful_by_default_relaxed_optout]]

## Goal

The `regex` parser SHALL, in its **default** mode, accept exactly what **PCRE2** accepts and reject
exactly what PCRE2 rejects (PCRE2 is the de-facto reference; the oracle is `pcre2test` via
`regex_pcre2_compile_oracle_gate`). A **`relaxed`** profile (opt-out) re-admits the broader, non-PCRE2
set. The PCRE2 acceptance rules currently in the out-of-band validator
(`rust/src/regex_compile_validation.rs::validate_regex_compile_contract`, 10 sub-checks) must be encoded
**IN `grammars/regex.ebnf`** as semantic annotations (profile-gating + value-constraint predicates), so
the stimuli generator honors them too; the validator is then removed (the EBNF becomes the single source
of truth — [[project_ebnf_is_single_source_of_truth]], and `scripts/check_ebnf_source_of_truth.sh` is
satisfied by construction). Engine changes ONLY as a last resort (the standing no-workarounds hierarchy).

## Non-Goals

- Re-implementing PCRE2's *matching* engine. This is about the **accepted language** (compile-time
  accept/reject) + the **AST shape**, not runtime match semantics.
- Forcing a relaxed-mode definition of "valid" — relaxed simply lifts the PCRE2-only restrictions.
- A regex-specific engine hack. Any new primitive must be GENERAL/parser-agnostic.

## Acceptance Criteria

- `regex.ebnf` carries a `relaxed` profile; **default mode = strict PCRE2**, `relaxed` = superset.
- Every PCRE2 accept/reject rule in `validate_regex_compile_contract` is encoded in the EBNF (profile-gate
  or value-constraint predicate); the validator is deleted.
- Default mode passes `regex_pcre2_compile_oracle_gate` (matches `pcre2test`) with no new divergence;
  relaxed mode is covered by its own tests.
- The stimuli generator, default mode, emits only PCRE2-valid samples (regex cert-coverage
  `sample_parse_failures` → its true floor; the `[]`/`\u`/`(*verb)` residuals resolve here).
- `check_ebnf_source_of_truth.sh` green with the validator gone (no out-of-band acceptance gate).
- Full released-parser lockstep per `COMMIT.md` (regen + RGX conformance + AST-shape manifest + regex
  book + integration contract + bug ledger + release/contract version) on each behaviour-affecting leaf.

## `.1` Scoping result (2026-06-07, `PGEN-REGEX-PCRE2-0001`) — tool-backed

**Mechanisms confirmed present (parser-agnostic):** `@profiles` profile-gating (codegen emits a profile
guard for any grammar, `ast_based_generator.rs:7969`; `set_grammar_profile`/`grammar_profile` plumbing);
annotation value-constraints `@predicate` + `len_bounds` + `numeric_bounds`; PCRE2 oracle
`regex_pcre2_compile_oracle_gate.sh` (`pcre2test`). `regex.ebnf` is currently single-profile → adding
`relaxed` is additive.

**The PCRE2 rules to migrate (the 10 `validate_regex_compile_contract` sub-checks) → EBNF encoding plan:**

| # | Validator check | PCRE2 rule (rejects) | EBNF encoding approach |
| --- | --- | --- | --- |
| 1 | `find_invalid_escape_i` | unsupported escapes (`\u`, …) | profile-gate `unicode_escape` etc. `@profiles:["relaxed"]` |
| 2 | `find_invalid_property_escape` | invalid `\p{…}` property names | `@predicate` on property name (valid set) / profile-gate |
| 3 | `find_invalid_named_escape_or_group_name` | group/name validity + length ≤ 128 | `@predicate` `len_bounds` + name charset |
| 4 | `find_invalid_counted_quantifier` | `{N,M}` bounds (N≤M, ≤ 65535) | `@predicate` `numeric_bounds` / N≤M |
| 5 | `find_invalid_numeric_callout` | `(?C N)` numeric range | `@predicate` `numeric_bounds` |
| 6 | `find_invalid_verb_construct` | unrecognized `(*VERB)` / start-option | gate `directive_name` to the known verb/option set (keyword rule); arbitrary names → `relaxed` |
| 7 | `find_invalid_char_class_construct` | empty `[]`, malformed class | require non-empty class (PCRE2 first-`]`-is-literal); empty `[]` → `relaxed` |
| 8 | `find_invalid_quantified_anchor` | quantified anchors (`^*`, …) | `@predicate` / structural; lift in `relaxed` |
| 9 | `find_invalid_scan_substring_capture_list` | malformed scan/substring capture list | structural / `@predicate` |
| 10 | `find_invalid_keep_out_escape_in_lookaround` | `\K` inside lookaround | **contextual** — likely the hard one; may need a new parser-agnostic annotation primitive (rung 3), engine only if proven necessary |

**Risk flags:** #10 (`\K`-in-lookaround) is contextual (depends on enclosing construct) — the current
annotation vocabulary may not express it; that is the first candidate for a NEW parser-agnostic
annotation feature (NOT an engine change unless proven necessary). #6 (verb set) needs the full
recognized PCRE2 verb + start-option list (from `pcre2_verb_argument_rule` / `is_pcre2_start_option_name`).

## Task Tree

- ID: `REGEX-PCRE2-FIDELITY`  Status: `active`  Children: `.1`..`.5`
- ID: `.1`  Status: `done`  Goal: scope — confirm mechanisms, enumerate PCRE2 rules, map each to its
  EBNF encoding, flag risks. Acceptance: the table above + the decision record. Commit: `PGEN-REGEX-PCRE2-0001`.
- ID: `.2`  Status: `pending`  Goal: profile scaffolding — add the `relaxed` profile to `regex.ebnf`
  (default = strict PCRE2) + opt-out plumbing (`--grammar-profile relaxed` / embedding API) + a baseline
  test that default==current-strict and `relaxed` is selectable. Acceptance: profile selectable, no
  behaviour change yet (no constructs gated), gates green.
- ID: `.3`  Status: `pending`  Goal: encode-in-EBNF per construct (one sub-leaf per row above), each:
  encode (profile-gate / predicate) → remove that check from `validate_regex_compile_contract` → verify
  PCRE2-oracle (default) + relaxed-mode + cert-coverage, tools-first, one at a time, measured. Children
  `.3.1`..`.3.10` map to rows 1–10 (start with the generator-tripping ones: `\u`, `(*verb)`, empty `[]`).
- ID: `.4`  Status: `pending`  Goal: capstone — once all 10 checks are encoded, delete
  `validate_regex_compile_contract` + its module; `check_ebnf_source_of_truth.sh` green with no validator;
  EBNF is the sole source of truth.
- ID: `.5`  Status: `pending`  Goal: verification — full `regex_pcre2_compile_oracle_gate` parity
  (default), relaxed-mode test suite, cert-coverage at floor, RGX conformance ratchet, lockstep.

## Current Frontier

- `.2` profile scaffolding (default `pcre2` / `relaxed` opt-out) — then `.3.1`/`.3.2`/`.3.7` (the
  generator-tripping `\u` / `(*verb)` / empty-`[]` first, since those also close the cert-coverage
  residual). Each behaviour-affecting leaf is a released-parser slice (full lockstep).

## Decisions

- `2026-06-07`: PCRE2-faithful-by-default + relaxed opt-out, EBNF-driven, engine last resort
  ([[project_regex_pcre2_faithful_by_default_relaxed_optout]]). Subsumes `EBNF-SOURCE-OF-TRUTH.3` (the
  `\u`/`(*verb)` consumer-path fix) and the regex `[]` empty-class residual; composes with
  [[project_ebnf_is_single_source_of_truth]] (the validator migrates IN and is removed).
- `2026-06-07`: default profile name is the strict/PCRE2 base (relaxed is the additive opt-out), matching
  the PGEN convention that the base grammar is the default and a profile re-admits extras.

## Open Questions

- Profile naming: is the default literally `pcre2` (explicit) or the un-profiled base with only `relaxed`
  as a named profile? (Resolve in `.2` — leaning un-profiled base = strict, `relaxed` named.)
- #10 `\K`-in-lookaround: can it be expressed with a new parser-agnostic annotation, or is it the rare
  genuine engine case? (Resolve when `.3` reaches it; tools-first.)

## Blockers

- None. Each behaviour-affecting leaf is released-parser work (lockstep + RGX conformance), to be paced.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-07` | `.1` | mechanism greps (profiles/predicates/oracle), validator-check enumeration | confirmed feasible |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `PGEN-REGEX-PCRE2-0001` | scoping (this slice) |

## Changelog

- `2026-06-07`: tree created + `.1` scoping done (`PGEN-REGEX-PCRE2-0001`), per the director directive to
  make regex PCRE2-faithful by default with a relaxed opt-out, EBNF-driven.
