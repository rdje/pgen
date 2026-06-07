---
name: project-regex-pcre2-faithful-by-default-relaxed-optout
description: The regex parser SHALL, by default, accept exactly what PCRE2 accepts and reject exactly what PCRE2 rejects (PCRE2 is the de-facto reference), with an opt-out RELAXED mode. The PCRE2 acceptance rules must be encoded IN the EBNF via semantic annotations (profile-gating + value-constraint predicates) so the stimuli generator honors them; the out-of-band validator (regex_compile_validation.rs) must migrate INTO the EBNF and be removed. Engine changes ONLY as a last resort. Director directive 2026-06-07.
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-06-07
  owning_tree: REGEX-PCRE2-FIDELITY
---

**THE DIRECTIVE (director, 2026-06-07, verbatim intent).** "I want the regex parser to accept what
PCRE2 accepts and reject what PCRE2 rejects, because PCRE2 has become the de-facto reference. But I
would like to be able to opt-out of this. By default we should try hard to match/reject what PCRE2
does, but also have the more relaxed behavior for whatever reason. We should try hard to have the fix
in the EBNF via semantic annotations to steer the parsing. Ideally we should touch the engine only as
a last resort."

**WHAT THIS MEANS.**

1. **PCRE2 is the default reference.** The regex parser's default-mode accepted language = PCRE2's
   (verified against the `pcre2test` oracle: `regex_pcre2_compile_oracle_gate`). What PCRE2 rejects
   (`\u`, unrecognized `(*verb)`, empty `[]`, group-name > 128, octal > `\377`, malformed counted
   quantifiers, …) the default-mode parser rejects, and the stimuli generator never emits.

2. **RELAXED is an opt-out mode.** A `relaxed` grammar profile (selected via `--grammar-profile
   relaxed` / the embedding API, exactly like SV's `sv_2017`/`sv_2023`) re-admits the broader,
   non-PCRE2 set "for whatever reason." Default = strict PCRE2; relaxed = superset.

3. **The mechanism is the EBNF + semantic annotations — NOT an out-of-band validator, NOT the engine
   by default.** This composes with [[project_ebnf_is_single_source_of_truth]]: the PCRE2 rules in
   `rust/src/regex_compile_validation.rs` (`validate_regex_compile_contract`, 10 sub-checks) are
   currently OUT-OF-BAND (invisible to the generator) — a defect. They must move INTO `grammars/regex.ebnf`
   as semantic annotations:
   - **Profile-gating** (`@profiles: ["relaxed"]`) for whole constructs PCRE2 forbids (e.g.
     `unicode_escape`, arbitrary `directive_name`/verbs, empty character class).
   - **Value-constraint predicates** (`@predicate` with `len_bounds` / `numeric_bounds`) for
     PCRE2's numeric/length rules (group-name length, octal range, counted-quantifier bounds).
   As each check is encoded, it is removed from `validate_regex_compile_contract`; when all are gone
   the validator is deleted and the `check_ebnf_source_of_truth.sh` gate is satisfied by construction.

4. **Fix hierarchy (the standing no-workarounds order, [[feedback_no_workarounds_fix_hierarchy]]):**
   (1) existing semantic annotations → (2) existing store → (3) NEW parser-agnostic annotation
   feature → (4) new store feature → (5) parser-agnostic engine enhancement. Engine ONLY as last
   resort, with a tool-backed case proving every lower rung insufficient. Any new primitive must be
   GENERAL/parser-agnostic, not regex-specific.

**FEASIBILITY (tool-backed, 2026-06-07).** All mechanisms exist: `@profiles` is parser-agnostic (codegen
emits a profile guard for any grammar, `ast_based_generator.rs:7969`; `grammar_profile` plumbing is
generic); the annotation language already has `@predicate` + `len_bounds` + `numeric_bounds`; the PCRE2
oracle gate (`regex_pcre2_compile_oracle_gate.sh`, `pcre2test`) is the verification surface; the 10
validator sub-checks are enumerated. `regex.ebnf` is currently single-profile, so adding `relaxed` is
additive.

**SCOPE.** A multi-leaf, released-parser campaign (RGX downstream → AST-shape/contract/ledger/release
lockstep). Owned by the `REGEX-PCRE2-FIDELITY` task tree. Subsumes `EBNF-SOURCE-OF-TRUTH.3` (the
`\u`/`(*verb)` consumer-path fix) and the regex `[]` empty-class residual. Composes with — does not
replace — [[project_ebnf_is_single_source_of_truth]] and the certifying-linter duality
([[project_grammar_wellformedness_contract]]).
