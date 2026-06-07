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
- ID: `.3.1`  Status: **`done`** (`PGEN-REGEX-PCRE2-0007`; implemented per **DESIGN COMPLETE
  `PGEN-REGEX-PCRE2-0006`**, which superseded `-0004`/`-0005`) — see the "`.3.1` DESIGN COMPLETION"
  section below for the executed plan and the "`.3.1` IMPLEMENTATION OUTCOME" note.
  Goal: PCRE2-align `\u` (and its 5 validator siblings `\U \F \l \L \i`).
  **Design history (provenance):** `-0004` "bounded `\u{…}`" was flawed; `-0005` corrected to the
  `simple_escape`/`class_simple_escape` catch-all restructure but was STILL INCOMPLETE — it omitted the
  THIRD catch-all `class_range_literal_escape_letter` (it explicitly lists `F L U I i l u`), which would
  leak `\u` etc. in class-range position once `find_invalid_escape_i` is deleted. `-0006` (tool-backed:
  `pcre2test` oracle + grammar read) completes the design across all three catch-alls and pins exact
  positional-ref recomputations.
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
- ID: `.3.2`  Status: **`done`** (`PGEN-REGEX-PCRE2-0008`) — landed: `directive_name` split strict (8 verbs +
  26 start-options, longest-first, case-sensitive) | `@profiles:["relaxed"]` catch-all; validator's
  unrecognized-name reject removed (structural checks kept), so default rejects unrecognized verb names by
  grammar (conformance-neutral: oracle 46/292/338 byte-identical) + relaxed re-admits arbitrary names.
  VERIFIED: matrix (default rejects `(*FOO)`/`(*MARKX)`/`(*accept)`, accepts recognized; relaxed accepts
  arbitrary; `(*MARK)`/`a(*UTF)` structural-reject both) + lib 612/0 + 651/0 + oracle GREEN + clippy strict
  + cert-cov default 4→1 (only empty-`[]` residual left; relaxed 5 all empty-class residuals, no verb
  failures) + both book gates GREEN. Surface-neutral, no version bump. Lockstep: regex book `directive_name`
  + changelog + HTML, contract Maintenance-Update addendum, parser-families. Goal: PCRE2-align `(*verb)` NAMES — validator
  check #6 `find_invalid_verb_construct` (NOTE: leaf numbers follow priority/work-order `\u`→`(*verb)`→`[]`,
  NOT the validator row index; this is row 6 of the `.1` table). The grammar's `directive_name`
  (`regex.ebnf`, `/([A-Za-z][A-Za-z0-9_\-]*)/`) accepts ANY name, so the generator emits `(*FOO)` that the
  out-of-band validator rejects (the EBNF-SOT duality). **Design (tool-backed `pcre2test` 10.47):** split
  `directive_name` into a strict (default `pcre2`) keyword ordered-choice of EXACTLY the recognized set —
  8 verbs (`MARK ACCEPT F FAIL COMMIT PRUNE SKIP THEN`) + 26 start-options (`UTF UTF8 UTF16 UTF32 UCP
  NOTEMPTY NOTEMPTY_ATSTART NO_AUTO_POSSESS NO_DOTSTAR_ANCHOR NO_JIT NO_START_OPT CASELESS_RESTRICT
  TURKISH_CASING LIMIT_HEAP LIMIT_MATCH LIMIT_DEPTH LIMIT_RECURSION CR LF CRLF ANY NUL ANYCRLF
  BSR_ANYCRLF BSR_UNICODE`), ordered **longest-first** for prefix overlaps (`FAIL`>`F`,
  `UTF32/16/8`>`UTF`, `NOTEMPTY_ATSTART`>`NOTEMPTY`, `ANYCRLF`>`ANY`, `CRLF`>`CR`) — and a
  `@profiles:["relaxed"]` catch-all variant = the original regex (`directive_name = directive_name_strict |
  directive_name_relaxed`). Case-sensitive (`(*accept)`/`(*Skip)` → REJECT, matches PCRE2). Default then
  rejects unrecognized verb names BY GRAMMAR (conformance-neutral: rejected before via validator); `relaxed`
  re-admits arbitrary names. **No AST-shape/manifest change** (`directive_name` has no return annotation;
  the strict/relaxed variants are passthrough). The finer STRUCTURAL checks (MARK-arg-required,
  start-option position, `=value` numeric, quantified-ACCEPT-only) STAY in `find_invalid_verb_construct`
  for now (harder to express in grammar; the validator's unrecognized-name branch becomes grammar-shadowed
  but is removed at capstone `.4`). Verify: matrix (default rejects `(*FOO)`/`(*MARKX)`/`(*accept)`,
  accepts the recognized set; relaxed accepts arbitrary) + `regex_pcre2_compile_oracle_gate` (no new
  divergence) + cert-coverage + lib + clippy; regen LOCAL. Lockstep IFF surface changes (assess —
  likely surface-neutral like `.3.1`, so book note only, no version bump).
- ID: `.3.7`  Status: `in-progress` (root-cause `PGEN-REGEX-PCRE2-0009`; cause (a) DONE `PGEN-REGEX-PCRE2-0010`)
  Goal: drive regex DEFAULT-profile cert-coverage `sample_parse_failures` → 0. Root-caused (2026-06-08) into
  3 generator↔parser round-trip causes (each minimal-repro'd via `parseability_probe --parse regex`):
  - **(a) empty char class `[]`/`[^]`** — DONE (`-0010`). PCRE2: the first `]` after `[`/`[^` is a LITERAL
    member, so `[]`/`[^]` are unterminated, not empty classes. The former `char_class = "[" negation?
    class_initial_close? class_body "]"` (`class_body = class_item*`) let the generator emit `[]`/`[^]`
    (no-initial-close AND empty-body). FIX = split `char_class` into two alts so a class always has ≥1
    member: alt 1 `"[" negation? class_initial_close class_body "]"` (the `[]…]` form, `initial_close:true`)
    | alt 2 `"[" negation? class_body_nonempty "]"` (`class_body_nonempty = class_item+`, `initial_close:[]`).
    AST shape PRESERVED exactly (`[]]`→initial_close:true,body:[]; `[x]`→initial_close:[],body:[x];
    `[^x]`→negated:true). VERIFIED: `[]`/`[^]` reject, `[]]`/`[^]]`/`[a-z]`/`[]x]`/`[\QxY\E]` pass; oracle
    BYTE-IDENTICAL; manifest synced (172→173, both char_class branches); self-hosting intact (0 `/.../`, 0
    `match_regex`); lib 615/0, generated_parsers 654/0, clippy ✓. The generator can no longer emit `[]`/`[^]`.
    (Note: the cert-cov COUNT is sample-dependent — at count 200/seed 0 it read 14→16 across the grammar
    change, NOT a regression: the empty-class CATEGORY is structurally gone; the residual is now spacing.)
  - **(b) `(?(R N)` spacing** + **(c) name/condition spacing** — PENDING, owned by **LEXICAL-ANNOTATIONS**
    (re-open). CONFIRMED root-cause (2026-06-08, repros + grammar read): the word-boundary spacing's concat
    join (`stimuli_generator.rs::append_generated_segment`, the choke point for sequences + quantifier
    repetition) works on ACCUMULATED characters and CROSSES rule boundaries by design (so it separates
    `module` + `automatic`). It inserts a space whenever the tail is word-shaped and the next char is a word
    char — which is WRONG inside a single lexical token. **(c)** `name = (letter|'_'|unicode_char)(letter|
    digit|'_'|unicode_char)* -> $text` is a CHAR-SEQUENCE (self-hosting; was a `/.../` single unit before),
    so the join separates its chars → `(?P=a b c)`/`(?P<a b>x)` reject (vs `(?P=abc)` pass). **(b)**
    `recursion_condition = "R" digits?` → the join separates `R` from `digits` → `(?(R 1)x)` rejects (vs
    `(?(R1)x)` pass). SELF-HOSTING-INDUCED: the `/.../`→char-sequence/`$text` conversions exposed the join.
    The hard part = the heuristic can't distinguish intra-token fusion (`R1`, name chars) from legitimate
    inter-token separation (`module automatic`) by word-shape alone. FIX DIRECTION (LEXICAL-ANNOTATIONS,
    careful, cross-grammar — must keep `stimuli_cross_family_platform_gate` green): a rule that IS one lexical
    token must generate its body as ONE fused unit (no internal word-boundary join). For (c) the `$text`
    annotation already declares "one token" → suppress the intra-rule join when generating a `$text` rule.
    For (b) `recursion_condition` returns a STRUCTURED object (not `$text`) yet `R`+digits is one token → needs
    a token-cohesion signal (e.g. a no-internal-spacing annotation, or treating a `literal + $text-rule`
    adjacency as cohesive). THEN re-measure → regex default `sample_parse_failures` → 0. (cert-cov count is
    sample-dependent — judge by the category being gone, not the raw number.)
- ID: `.3.11`  Status: `pending` (DISCOVERED `PGEN-REGEX-PCRE2-0006`, tool-backed)  Goal: full PCRE2
  **escape whitelist** — strict/default mode accepts ONLY PCRE2's recognized escape letters; the broad
  `simple_escape`/`class_simple_escape`/`class_range_literal_escape_letter` catch-alls become the `relaxed`
  superset. **Why a new leaf:** the `pcre2test` oracle shows PCRE2 rejects EVERY unrecognized `\<letter>`
  (e.g. `\I \J \m \M` → error 103), not just `find_invalid_escape_i`'s 6 (`i F l L u U`). So `.3.1`
  (six-letter migration) is a conformance-NEUTRAL stepping stone that only re-homes the validator into
  the grammar; this leaf is the eventual full-fidelity end-state and would SUBSUME `.3.1`'s six
  exclusions (strict variant = recognized-only). Bigger/riskier restructure → its own design + released
  slice. Oracle facts: [[reference_pcre2_unsupported_escape_oracle]].
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

## `.3.1` DESIGN COMPLETION (2026-06-07, `PGEN-REGEX-PCRE2-0006`) — tool-backed, implementable

Supersedes `-0004`/`-0005`. Established with `pcre2test` 10.47 (oracle) + a full read of the regex
grammar, validator, and profile-gating codegen. Durable oracle facts:
[[reference_pcre2_unsupported_escape_oracle]].

**Mechanism verified (no assumptions):**
- The profile guard is RULE-level and emits a clean backtrack:
  `if !self.rule_profile_is_enabled(&["relaxed"]) { return Err(ParseError::Backtrack { position }); }`
  (`ast_based_generator.rs:2376`). So a `@profiles:["relaxed"]` rule cleanly fails under any non-relaxed
  profile.
- `rule_profile_is_enabled` (`:3806`) returns `true` for `None` active profile (permissive). ⇒ regex MUST
  default GENERATION to an explicit `pcre2` profile (the parse side was already defaulted in `.2`).
- A pure-alternation rule with no `->` (e.g. the existing `escape_unit`) passes the matched branch
  through and carries NO shape-contract manifest entry — the template for the strict/relaxed splits below.

**The validator being migrated:** `find_invalid_escape_i` (`regex_compile_validation.rs:118`, called at
`:19`) rejects EXACTLY `\i \F \l \L \u \U` context-insensitively. PCRE2 rejects these in atom, class, AND
class-range (oracle), so the grammar must reject them in ALL THREE catch-alls. **Conformance-neutral on
default** (default rejected the 6 via the validator before; via the grammar after) — RGX conformance
unchanged; relaxed GAINS the 6.

**The three catch-alls + their exact edits (positional refs recomputed; AST shape `{type:"escape",
kind:"shorthand", char:$N}` preserved in every variant):**

1. **`simple_escape`** (atom, `regex.ebnf:581`; current 14 lookaheads → `$15`). Split:
   - `simple_escape = simple_escape_strict | simple_escape_relaxed` (no `->`, passthrough).
   - `simple_escape_strict` = existing guards **+** `!"i" !"F" !"l" !"L" !"u" !"U"` (now 20 lookaheads) +
     `any_char` → `char: $21`.
   - `@profiles:["relaxed"]` `simple_escape_relaxed` = the ORIGINAL body (14 lookaheads) → `char: $15`.
2. **`class_simple_escape`** (class, `:449`; current 4 lookaheads → `$5`; NO digit guards by RGX-0088). Split:
   - `class_simple_escape = class_simple_escape_strict | class_simple_escape_relaxed`.
   - `class_simple_escape_strict` = `!"o{" !"x{" !"p{" !"P{" !"i" !"F" !"l" !"L" !"u" !"U"` (10 lookaheads)
     + `any_char` → `char: $11`.
   - `@profiles:["relaxed"]` `class_simple_escape_relaxed` = ORIGINAL (4 lookaheads) → `char: $5`.
3. **`class_range_literal_escape_letter`** (class-range, `:486-487`; explicit letter list, NO `->`, NOT in
   manifest). Split (remove exactly the 6 validator letters from strict; KEEP `I` — see below):
   - `class_range_literal_escape_letter = class_range_literal_escape_letter_strict | ..._relaxed`.
   - `_strict` = the current list MINUS `F L U i l u` (keep `A G I J M O T V Y` / `a b e f g j k m n o q r
     t v x y z`).
   - `@profiles:["relaxed"]` `..._relaxed` = `'F' | 'L' | 'U' | 'i' | 'l' | 'u'`.

**Plus:**
4. `unicode_escape` (`:606`) gets `@profiles:["relaxed"]` (braced `\u{…}` is relaxed-only). It is referenced
   by `escape_unit`/`class_escape_unit`/`class_range_escape_unit` — all three gate off together in default.
   Shape unchanged → manifest entry unchanged.
5. Generator-side `pcre2` default in `apply_grammar_profile_filter` (`main.rs:2196`) — the GENERATION twin
   of `.2`'s parse-side default (pin the exact line during implementation).
6. Delete `find_invalid_escape_i` + its call site (`:19`); `validate_regex_compile_contract` keeps its
   other 9 checks (module stays allowlisted until `.4`).

**`\I` decision (resolves an Open Question):** `pcre2test` REJECTS `\I` (error 103), but `\I` is NOT in
`find_invalid_escape_i`'s set, so PGEN-default currently ACCEPTS `\I` (a pre-existing divergence). `.3.1`
is validator-parity (conformance-neutral) → it LEAVES `\I` accepted (keep `I` in the strict list). The
broader unrecognized-escape divergence (every `\<unrecognized-letter>`) is owned by the new leaf `.3.11`.

**Shape-contract manifest (`regex_v1.json`) churn (alphabetical insertion per
[[feedback_manifest_alphabetical_order]]):** remove `simple_escape`/`class_simple_escape` direct entries;
add `simple_escape_strict`($21)/`simple_escape_relaxed`($15) and
`class_simple_escape_strict`($11)/`class_simple_escape_relaxed`($5); `unicode_escape` unchanged;
`class_range_literal_escape_letter*` carry no entries.

**Verification plan (implementation slice):** regen `generated/regex_parser.rs` locally; `pcre2test`
oracle gate (`regex_pcre2_compile_oracle_gate`) default-mode no new divergence; `--grammar-profile
relaxed` probe accepts the 6; cert-coverage no new `sample_parse_failures`; `cargo test --lib`; RGX
conformance ratchet. Released lockstep: manifest + regex book + integration contract + bug ledger +
release/contract bump. Build: `ast_pipeline --features generated_parsers,ebnf_dual_run` +
`PGEN_<big>_PARSER_PATH=/nonexistent`.

## `.3.1` IMPLEMENTATION OUTCOME (2026-06-07, `PGEN-REGEX-PCRE2-0007`)

Executed the DESIGN COMPLETION plan exactly. Landed (source only; `generated/regex_parser.rs` regenerated
locally, gitignored): `regex.ebnf` — `simple_escape`/`class_simple_escape`/`class_range_literal_escape_letter`
each split into strict/`@profiles:["relaxed"]` variants (recomputed refs `$21`/`$11`; `class_range` strict
drops `F L U i l u`, keeps `I`) + `unicode_escape` tagged `@profiles:["relaxed"]`; `main.rs` — regex
generation default `pcre2` in `apply_grammar_profile_filter` **and** an explicit exemption so the
`generate_parser` codegen path always emits the FULL grammar (the default would otherwise strip the
relaxed rules → unresolved-reference fallback stubs — a bug caught + fixed in-slice); `regex_compile_validation.rs`
— `find_invalid_escape_i` + call site DELETED, two obsolete validator unit tests removed; `embedding_api.rs` —
both regex parse paths now `set_grammar_profile(Some("pcre2"))` (the embedding default was permissive `None`
→ leaked the 6 in default; caught by the contract failure-sample tests); manifest `regex_v1.json` (4
strict/relaxed entries, alphabetical); oracle baseline env 45→46 (PRE-EXISTING drift, stash-baseline
proven — see below).

**Verified (tools-first):** behaviour matrix (default REJECTS `\i \F \l \L \u \U`/`\u{…}` in atom+class+class-range,
ACCEPTS `\d \w \I`; `relaxed` ACCEPTS the 6) 15/15; AST shapes preserved (shorthand/unicode); `cargo test
--lib` 612/0; `cargo test --lib --features generated_parsers` (heavy parsers skipped) 651/0 incl. regex
shape-contract + integration-contract failure-sample tests; clippy strict source ✓ (generated `useless_vec`
non-strict, pre-existing); **`regex_pcre2_compile_oracle_gate` GREEN** — and proven CONFORMANCE-NEUTRAL via a
stash-baseline: the OLD parser (changes reverted) produced an IDENTICAL 46-element false-reject set (+292
false-accept/+338 mismatch unchanged), so the gate's 45→46 was a PRE-EXISTING stale baseline, not a `.3.1`
regression (all 46 are documented strict-default divergence classes); cert-coverage default 4 (all known
structural residuals — empty `[]`/`[^]` ×3 + `(?(R 1)` ×1, none escape-related) / relaxed 3 (= prior floor).
Released-parser lockstep: handoff contract "Maintenance Update 2026-06-07" (SURFACE-NEUTRAL — no version
bump; same accepted language + AST shape + `E_PARSE_FAILURE` code; message text changed → match on code),
regex book escapes chapter + Profiles section + changelog-index + regenerated tracked HTML, platform book
`parser-families.md`. **No version bump** (release stays 1.1.81 / contract 1.1.83) — the embedding surface is
unchanged; the `relaxed` profile is CLI-only (embedding-API exposure is a tracked follow-on).

## Current Frontier

- `.3.1` (`\u`-family) and `.3.2` (`(*verb)` names) are DONE. Next candidates, **pending a director
  priority decision** (2026-06-07 the director surfaced two cross-cutting goals — see below):
  - `.3.7` — **the cert-coverage lever** (NOW THE FRONTIER, REGEX-SELF-HOSTING done 2026-06-08).
    RE-MEASURED 2026-06-08 (post-self-hosting): `ast_pipeline grammars/regex.ebnf
    --report-certificate-coverage --entry-rule regex --count 200 --seed 0` → `sample_parse_failures=14`
    (up from the pre-self-hosting 3 — NOT an accepted-language regression: oracle byte-identical; the
    self-hosting changed the grammar STRUCTURE so the generator explores different paths and surfaces more
    generator↔parser round-trip gaps). **ROOT-CAUSED into 3 isolated causes (each minimal-repro'd via
    `parseability_probe --parse regex`):** (a) **empty char class** `[]`/`[^]` REJECT (PCRE2: the first `]`
    after `[`/`[^]` is a literal member, so an empty class is unterminated) while `[]]`/`[^]]` PASS — the
    generator emits `[]`/`[^]` because `char_class = "[" negation? class_initial_close? class_body "]"` with
    `class_body = class_item*` allows the (no-initial-close AND empty-body) combination; FIX = a `regex.ebnf`
    char_class change that forbids that combination (require ≥1 member: an initial-close `]` OR ≥1
    class_item), preserving the `{negated, initial_close, body}` AST shape (`[]]` dumps
    `initial_close:true, negated:[], body:[]`); (b) **`(?(R N)` spacing** — `(?(R 1))` REJECTS, `(?(R1))`
    PASSES → word-boundary spacing inserts a space between `R` and the recursion digit; (c) **name spacing**
    — `(?P=a b)`/`(?P=_ _ _)` REJECT, `(?P=ab)` PASSES → word-boundary spacing inserts a space INSIDE a
    `name`. (The `\Q…\E`-with-`]` samples in the failure list are RED HERRINGS — they parse fine.) Causes
    (b)+(c) are LEXICAL-ANNOTATIONS spacing-generator residuals (the `apply_word_boundary_spacing`
    over-insertion, like the earlier `(*VERB )` family); (a) is THIS leaf's `regex.ebnf` fix. Closing all
    three drives regex default `sample_parse_failures` → 0. Highest-value for the "all parsers cert-coverage
    clean" goal.
  - **REGEX-SELF-HOSTING** (NEW, director 2026-06-07): make `regex.ebnf` use ONLY literal `"..."`
    terminals — eliminate every `/.../` regex-literal so the generated regex parser does not depend on
    Rust's `regex` engine (self-hosting; no regex-to-parse-regex circularity). Big refactor (replace
    `digit`/`letter`/`hex` classes / `unicode_char` / `special_char` / `directive_name_relaxed` / etc. with
    literal-built rules). Composes with this tree (`.3.2`'s strict `directive_name` is already
    `/.../`-free). Likely its own task tree once scoped.
  - `.3.11` (full unrecognized-escape whitelist) and `.4`/`.5` capstone (delete `validate_regex_compile_contract`
    + expose `relaxed` via the embedding API) remain.

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
- `2026-06-07` (`-0006`): **`.3.1` is scoped to validator-parity (the 6 letters), NOT full escape
  fidelity.** Tool-backed oracle shows PCRE2 rejects every unrecognized `\<letter>`, but `.3.1` only
  migrates `find_invalid_escape_i` (conformance-neutral). The full recognized-escape WHITELIST is the
  separate, broader leaf `.3.11` (it would subsume `.3.1`). `\I` (PCRE2-rejected but outside the
  validator's set) stays accepted in `.3.1` and is owned by `.3.11`. Per [[feedback_no_workarounds_fix_hierarchy]]
  + targeted-fix discipline ([[feedback_tools_first_no_guessing]]).

## Open Questions

- ~~Profile naming: is the default literally `pcre2` (explicit) or the un-profiled base?~~ **RESOLVED
  (`.2`/`-0006`): explicit `pcre2`** — the codegen profile guard treats `None` as PERMISSIVE
  (`None => true`, enables relaxed-tagged rules), so the default MUST be an explicit named `pcre2`
  profile, not the un-profiled base. Parse side wired in `.2`; generation side rides with `.3.1`.
- #10 `\K`-in-lookaround: can it be expressed with a new parser-agnostic annotation, or is it the rare
  genuine engine case? (Resolve when `.3` reaches it; tools-first.)

## Blockers

- None. Each behaviour-affecting leaf is released-parser work (lockstep + RGX conformance), to be paced.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-07` | `.1` | mechanism greps (profiles/predicates/oracle), validator-check enumeration | confirmed feasible |
| `2026-06-07` | `.2` | regex-focused build + cert-coverage (default 3, relaxed 3 = no-op) + lib 614/614 + regex tests 110/0 + clippy source 0 | DONE |
| `2026-06-07` | `.3.1` design (`-0006`) | `pcre2test` 10.47 oracle matrix (6 letters reject in atom/class/class-range; broad unrecognized-escape reject); grammar read (3 catch-alls); profile-guard codegen read (clean backtrack, `None`=permissive); manifest read | DESIGN COMPLETE — found+fixed the omitted 3rd catch-all; spun out `.3.11` (full escape whitelist) |
| `2026-06-07` | `.3.1` impl (`-0007`) | behaviour matrix 15/15 (default rejects 6 / relaxed accepts 6 / controls unchanged); AST shapes preserved; `cargo test --lib` 612/0; `--features generated_parsers` 651/0 (incl. shape-contract + contract failure-samples); clippy strict source ✓; `regex_pcre2_compile_oracle_gate` GREEN + stash-baseline CONFORMANCE-NEUTRAL (OLD≡NEW 46/292/338); cert-coverage default 4 (known residuals)/relaxed 3 | **DONE** — landed; baseline 45→46 (pre-existing drift); surface-neutral, no version bump |
| `2026-06-07` | `.3.2` impl (`-0008`) | verb matrix (default rejects `(*FOO)`/`(*MARKX)`/`(*accept)`, accepts 8 verbs + start-options; relaxed accepts arbitrary; `(*MARK)`/`a(*UTF)` structural-reject both); `cargo test --lib` 612/0; `--features generated_parsers` 651/0; clippy strict source ✓; `regex_pcre2_compile_oracle_gate` GREEN (46/292/338 byte-identical = conformance-neutral); cert-cov default 4→1 (only empty-`[]`; relaxed 5 = empty-class residuals, no verb failures); both book gates GREEN | **DONE** — surface-neutral, no version bump |
| `2026-06-08` | `.3.7` root-cause (`-0009`) | `--report-certificate-coverage --entry-rule regex --count 200 --seed 0` → `sample_parse_failures=14`; `parseability_probe --parse regex` minimal repros isolated 3 causes (empty `[]`/`[^]`; `(?(R N)` spacing; name spacing); `\Q…\E`-with-`]` ruled out as red herrings | ROOT-CAUSE DONE (docs); 14 vs pre-self-hosting 3 = sample-set shift, oracle byte-identical (not a regression) |
| `2026-06-08` | `.3.7` (a) impl (`-0010`) | `char_class` two-alt split (always ≥1 member); `[]`/`[^]` reject + `[]]`/`[^]]`/`[a-z]`/`[]x]`/`[\QxY\E]` pass; AST shapes preserved (`[]]`→initial_close:true; `[x]`→initial_close:[]); `regex_pcre2_compile_oracle_gate` BYTE-IDENTICAL; manifest synced 172→173; self-hosting gate OK (0 `/.../`, 0 `match_regex`); lib 615/0, generated_parsers 654/0, clippy ✓ | **(a) DONE** — empty-class category structurally eliminated; surface-neutral (no version bump); (b)/(c) spacing next |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `PGEN-REGEX-PCRE2-0001` | scoping |
| `.2` design | `PGEN-REGEX-PCRE2-0002` | pinned the explicit-`pcre2`-default wiring plan |
| `.2` | `PGEN-REGEX-PCRE2-0003` | parse-side `pcre2` default (Rust-only, no-op verified) |
| `.3.1` design | `PGEN-REGEX-PCRE2-0004` | bounded `\u{…}` design — SUPERSEDED (flawed: simple_escape catches `\u`) |
| `.3.1` design correction | `PGEN-REGEX-PCRE2-0005` | the real lever is the simple_escape catch-all restructure (still incomplete — 2 catch-alls) |
| `.3.1` design completion | `PGEN-REGEX-PCRE2-0006` | tool-backed (pcre2test oracle); completes the design across ALL 3 catch-alls + recomputed positional refs; spun out `.3.11` (full escape whitelist) |
| `.3.1` implementation | `PGEN-REGEX-PCRE2-0007` | grammar 3-catch-all strict/relaxed split + `unicode_escape` relaxed + generator `pcre2` default (+ codegen-path exemption) + `find_invalid_escape_i` deleted + embedding `pcre2` default + manifest + oracle baseline 45→46 + handoff/book lockstep; conformance-neutral (stash-proven); no version bump |
| `.3.2` implementation | `PGEN-REGEX-PCRE2-0008` | `directive_name` strict (verbs+start-options, longest-first, case-sensitive) / `relaxed` split + validator unrecognized-name reject removed (structural checks kept) + book/contract/parser-families lockstep; conformance-neutral (oracle byte-identical); no version bump; no manifest change |
| `.3.7` root-cause | `PGEN-REGEX-PCRE2-0009` | regex default cert-coverage = 14, root-caused into 3 causes (empty `[]`/`[^]`; `(?(R N)` spacing; name spacing); docs-only |
| `.3.7` (a) char_class | `PGEN-REGEX-PCRE2-0010` | empty-class fix: `char_class` two-alt split (always ≥1 member) → generator can't emit `[]`/`[^]`; oracle byte-identical; AST shapes preserved; manifest 172→173; surface-neutral |

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
- `2026-06-07`: `.3.2` IMPLEMENTATION DONE (`PGEN-REGEX-PCRE2-0008`). PCRE2-align `(*verb)` NAMES: split
  `directive_name` into a strict (default `pcre2`) ordered choice of the recognized 8 verbs + 26
  start-options (longest-first for prefix overlaps; case-sensitive) and a `@profiles:["relaxed"]` catch-all;
  removed the validator's unrecognized-name reject (kept the structural checks — MARK-arg, start-option
  position, `=value`, quantified-ACCEPT). Default rejects unrecognized verb names by grammar
  (conformance-neutral — oracle 46/292/338 byte-identical); `relaxed` re-admits arbitrary names. cert-cov
  default 4→1. Surface-neutral → no version bump. Lockstep: regex book `directive_name` + changelog + HTML,
  contract Maintenance-Update addendum, parser-families. DIRECTOR surfaced two cross-cutting goals
  (cert-coverage-clean across all parsers; regex.ebnf `"..."`-only / self-hosting, no Rust regex engine) —
  frontier now pending a priority decision (see Current Frontier).
- `2026-06-07`: `.3.1` IMPLEMENTATION DONE (`PGEN-REGEX-PCRE2-0007`). Executed the completed design: 3
  escape catch-alls split strict/`relaxed`, `unicode_escape` relaxed-gated, generator `pcre2` default
  (+ a `generate_parser` codegen-path exemption so the FULL grammar is always emitted — the default would
  otherwise strip the relaxed rules into fallback stubs, a bug caught + fixed in-slice), `find_invalid_escape_i`
  deleted, embedding regex paths set `pcre2` (was permissive `None`). Conformance-NEUTRAL on the PCRE2
  oracle (stash-baseline: OLD≡NEW 46/292/338) → no version bump (release 1.1.81 / contract 1.1.83 stay);
  oracle baseline corrected 45→46 (pre-existing drift). Lockstep: handoff contract maintenance section,
  regex book escapes/profiles/changelog + regenerated HTML, platform book. Frontier → `.3.2` `(*verb)`.
- `2026-06-07`: `.3.1` DESIGN COMPLETION (`PGEN-REGEX-PCRE2-0006`, docs-only, tool-backed). Ran the
  `pcre2test` 10.47 oracle + read the grammar/validator/profile-gating codegen. **Found the `-0005`
  design was STILL incomplete:** it named only `simple_escape` + `class_simple_escape`, but there is a
  THIRD catch-all — `class_range_literal_escape_letter` (`:486-487`, explicitly lists `F L U I i l u`) —
  that would leak `\u`/`\U`/`\F`/`\l`/`\L`/`\i` in class-range position once `find_invalid_escape_i` is
  deleted. Completed the design across all three catch-alls with recomputed positional refs
  (`simple_escape_strict` `$15`→`$21`, `class_simple_escape_strict` `$5`→`$11`). Also surfaced a broader
  tool-backed fact: PCRE2 rejects EVERY unrecognized `\<letter>` (`\I \J …` → error 103), not just the
  validator's 6 — so `.3.1` is a conformance-NEUTRAL stepping stone and full escape fidelity is a
  WHITELIST (new leaf `.3.11`). Profile naming resolved to explicit `pcre2` (per `.2`); `\K`-in-lookaround
  (#10) still open. Oracle facts recorded as [[reference_pcre2_unsupported_escape_oracle]]. Frontier →
  `.3.1` IMPLEMENTATION.
