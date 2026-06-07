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
- ID: `.2`  Status: `done` (`PGEN-REGEX-PCRE2-0003`)  Goal: profile scaffolding — establish the explicit
  `pcre2` default (parse side). DONE: `normalize_generated_grammar_profile` gained a `regex` arm
  (unspecified/`pcre2`/`strict` → `pcre2`; `relaxed` → `relaxed`; its cfg widened to
  `any(sv, regex)`); `parse_with_regex_detail` / `parse_with_regex_ast_json` take + thread a profile and
  `set_grammar_profile` (owned into the worker closure); `parse_and_cover_regex` honors its profile param;
  the 2 dispatch call sites pass `grammar_profile`. VERIFIED no-op: regex cert-coverage `sample_parse_failures`
  = 3 (default) AND 3 (`--grammar-profile relaxed`) — unchanged (no constructs gated yet); lib `--lib`
  614/614; regex-focused tests 110/0; clippy source 0 errors. Rust-only (NO regen). The generator-side
  default rides with `.3.1` (where it first matters for not emitting `\u`).
- ID: `.3`  Status: `pending`  Goal: encode-in-EBNF per construct (one sub-leaf per row above), each:
  encode (profile-gate / predicate) → migrate/refine the matching `validate_regex_compile_contract` check
  → verify PCRE2-oracle (default) + relaxed-mode + cert-coverage, tools-first, one at a time, measured.
  Children `.3.1`..`.3.10` map to rows 1–10 (start with the generator-tripping `\u`, then `(*verb)`, `[]`).
- ID: `.3.1`  Status: `pending` (DESIGN CORRECTED `PGEN-REGEX-PCRE2-0005`; the `-0004` "bounded" design
  was SUPERSEDED as flawed)  Goal: PCRE2-align `\u`.
  **⚠️ `-0004` correction (tool-backed): gating `unicode_escape` alone is INSUFFICIENT.** Empirically
  (`parseability_probe`): `\g`/`\a` parse via the `simple_escape` catch-all; `\u{41}`/`\uZ` are rejected
  by the VALIDATOR, not structurally — `simple_escape` (`escape_unit`'s last alt, `regex.ebnf:531/:543`,
  `!"o{" !"x{" !"p{" !"P{" any_char`) matches `\u` (any letter). So with `unicode_escape` gated out in
  default, `\u{…}` does NOT get rejected — it just shifts to `simple_escape` (parsed as `\u` + `{…}`), and
  the `-0004` "refine the validator to not reject `\u{`" tweak would make default *accept* it. WRONG.
  **Corrected design — the real lever is the `simple_escape` (+ `class_simple_escape`, `:449`) catch-all:**
  (1) profile-gate the 6 unsupported escape letters (`i F l L u U`): a strict (untagged, always-active)
  `simple_escape` variant that EXCLUDES them (`!"i" !"F" !"l" !"L" !"u" !"U"` added to the existing
  `!"o{" !"x{" !"p{" !"P{"` + RGX-0087 `\8`/`\9` guards) + a `@profiles:["relaxed"]` variant that admits
  them (`simple_escape = simple_escape_strict | simple_escape_relaxed`); same for `class_simple_escape`;
  (2) tag `unicode_escape` `@profiles:["relaxed"]` (braced shape relaxed-only); (3) generator-side `pcre2`
  default (the `.2` parse-side default's GENERATION twin — via `apply_grammar_profile_filter` defaulting
  regex `None`→`pcre2`, `main.rs:2196`); (4) then `find_invalid_escape_i` can be FULLY removed (grammar
  owns all 6). **MUST preserve** `simple_escape`'s existing guards (`\o{`/`\x{`/`\p{`/`\P{`, RGX-0087
  `[89]`) + the `{type:"escape",kind:"shorthand",char:$N}` AST shape in both variants — a core-rule
  restructure, higher-risk → careful released slice. Conformance-neutral on DEFAULT (default rejected the
  6 escapes before via the validator, after via the strict variant) → RGX conformance unchanged; relaxed
  GAINS them. Verify: `regex_pcre2_compile_oracle_gate` (default), relaxed probe, cert-coverage (no new
  fail), lib, RGX conformance; regen `generated/regex_parser.rs`; released lockstep (manifest `unicode_escape`
  now relaxed-only + book + contract + ledger + release/contract bump). Fresh context for the lockstep.
- ID: `.4`  Status: `pending`  Goal: capstone — once all 10 checks are encoded, delete
  `validate_regex_compile_contract` + its module; `check_ebnf_source_of_truth.sh` green with no validator;
  EBNF is the sole source of truth.
- ID: `.5`  Status: `pending`  Goal: verification — full `regex_pcre2_compile_oracle_gate` parity
  (default), relaxed-mode test suite, cert-coverage at floor, RGX conformance ratchet, lockstep.

## `.2` DESIGN — the explicit `pcre2` default (uncovered scoping `.3.1`, 2026-06-07)

**The wrinkle (tool-backed):** the codegen profile guard `rule_profile_is_enabled`
(`ast_based_generator.rs:3806`) returns `true` when `grammar_profile` is `None`
(`None => true` = permissive: ALL gated rules active). And `normalize_generated_grammar_profile`
(`parser_registry.rs:110`) returns `None` for an unspecified/empty profile. So if regex's default stays
`None`, a `@profiles:["relaxed"]` rule (e.g. `unicode_escape`) would be ENABLED by default → `\u` accepted
by default — the OPPOSITE of strict. ⇒ the default-strict design REQUIRES regex to default to an explicit
named profile **`pcre2`** (not `None`), so relaxed-tagged rules are excluded by default
(`"pcre2" ∉ {"relaxed"}`). Changing the `None => true` engine semantics is OUT (it would break SV's
"None = all editions" + is an engine change; last resort only).

The regex generated parser ALREADY carries the profile machinery (`set_grammar_profile` /
`grammar_profile` / `rule_profile_is_enabled` present), so **no regen is needed for `.2`** — it is Rust
wiring + the `normalize` default.

**`.2` implementation plan (precise sites):**
1. `normalize_generated_grammar_profile` (`parser_registry.rs:110`): add a `"regex"` arm — unspecified/
   empty/`pcre2`/`strict`/`default` → `Some("pcre2")`; `relaxed` → `Some("relaxed")`.
2. Thread a profile into the regex parse paths (they currently take no profile): give
   `parse_with_regex_detail` / `parse_with_regex_ast_json` a `grammar_profile: Option<&str>` param,
   normalize it (`"regex"`), and `parser.set_grammar_profile(...)`. Update the dispatch
   (`parse_sample_detail_with_profile` regex arm, `parse_with_regex_ast_json` caller) to pass the
   incoming profile. `parse_and_cover_regex` already takes `_grammar_profile` — normalize + set it.
3. Generation side: default the regex generator profile to `pcre2` when none is specified (the
   cert-coverage + `--generate-stimuli` paths), so default-mode generation also excludes relaxed
   constructs once `.3.x` gates them.
4. Verify NO behaviour change (no constructs gated yet): regex cert-coverage identical (still 3), lib
   614/614, RGX conformance unchanged. Then `.3.1` gates `unicode_escape` and the default flips to reject.

## Current Frontier

- `.3.1` — gate `unicode_escape` (`\u`) `@profiles:["relaxed"]` + add the generator-side `pcre2` default +
  drop the `\u` arm of `find_invalid_escape_i` → default rejects `\u` (matches PCRE2) + generator stops
  emitting it; `relaxed` accepts. Then `.3.2` `(*verb)`, `.3.7` empty-`[]`. Each is a released-parser
  slice: regen `generated/regex_parser.rs` + `regex_pcre2_compile_oracle_gate` (PCRE2 oracle) +
  cert-coverage + relaxed tests + AST-shape manifest + regex book + integration contract + ledger +
  release/contract bump. (`.2` default scaffolding DONE.)

## Decisions

- `2026-06-07`: PCRE2-faithful-by-default + relaxed opt-out, EBNF-driven, engine last resort
  ([[project_regex_pcre2_faithful_by_default_relaxed_optout]]). Subsumes `EBNF-SOURCE-OF-TRUTH.3` (the
  `\u`/`(*verb)` consumer-path fix) and the regex `[]` empty-class residual; composes with
  [[project_ebnf_is_single_source_of_truth]] (the validator migrates IN and is removed).
- `2026-06-07`: default profile name is the strict/PCRE2 base (relaxed is the additive opt-out), matching
  the PGEN convention that the base grammar is the default and a profile re-admits extras.
- `2026-06-07`: **PCRE2 verification surfaces.** Primary in-repo oracle: `regex_pcre2_compile_oracle_gate`
  (`pcre2test`) — directly proves default-mode accept/reject == PCRE2. Director hint: the external RGX
  bug reports `~/Documents/github/rgx/pgen-issues/PGEN-RGX-0017..0028.yaml` carry the broader PCRE2
  conformance corpus + how-to-run for the RGX PCRE2 conformance test (consult them for the full corpus +
  methodology when ratcheting conformance). The regex corpus bundle (`regex_corpus_bundle/`,
  `make -C rust regex_pcre2_compile_oracle_gate` / `regex_pcre2_textsafe_corpus_gate`) is the tracked
  in-repo PCRE2 corpus surface.

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
| `2026-06-07` | `.2` | regex-focused build + cert-coverage (default 3, relaxed 3 = no-op) + lib 614/614 + regex tests 110/0 + clippy source 0 | DONE |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `PGEN-REGEX-PCRE2-0001` | scoping |
| `.2` design | `PGEN-REGEX-PCRE2-0002` | pinned the explicit-`pcre2`-default wiring plan |
| `.2` | `PGEN-REGEX-PCRE2-0003` | parse-side `pcre2` default (Rust-only, no-op verified) |
| `.3.1` design | `PGEN-REGEX-PCRE2-0004` | bounded `\u{…}` design — SUPERSEDED (flawed: simple_escape catches `\u`) |
| `.3.1` design correction | `PGEN-REGEX-PCRE2-0005` | the real lever is the simple_escape catch-all restructure |

## Changelog

- `2026-06-07`: tree created + `.1` scoping done (`PGEN-REGEX-PCRE2-0001`), per the director directive to
  make regex PCRE2-faithful by default with a relaxed opt-out, EBNF-driven.
- `2026-06-07`: `.2` design (`PGEN-REGEX-PCRE2-0002`) pinned the explicit-`pcre2`-default wiring (the
  `None => true` permissive-guard wrinkle); `.2` DONE (`PGEN-REGEX-PCRE2-0003`) — parse-side `pcre2`
  default wired (Rust-only, no-op verified). Frontier → `.3.1` (`\u`).
- `2026-06-07`: `.3.1` DESIGN pinned (`PGEN-REGEX-PCRE2-0004`) — bounded to braced `\u{…}`. **SUPERSEDED
  by `-0005` (DESIGN CORRECTION):** empirically (`\g`/`\a` parse; `\u{41}`/`\uZ` validator-rejected) the
  `simple_escape` catch-all matches `\u`, so gating `unicode_escape` alone is INSUFFICIENT (shifts `\u{…}`
  to `simple_escape`). The real lever is the `simple_escape`/`class_simple_escape` catch-all restructure
  (profile-gate the 6 letters `i F l L u U`: strict-untagged excludes them, `@profiles:["relaxed"]`
  admits them) + `unicode_escape` gate + generator-side `pcre2` default, then remove
  `find_invalid_escape_i`. A core-rule restructure (preserve existing guards + AST shape) → higher-risk
  released slice; careful fresh-context execution.
