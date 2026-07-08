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
- ID: `.3.7`  Status: `done` (root-cause `PGEN-REGEX-PCRE2-0009`; (a) DONE `PGEN-REGEX-PCRE2-0010`;
  (b)+(c) DONE `PGEN-LEXICAL-ANNOTATIONS-0024` via the LEXICAL-ANNOTATIONS tree)
  Goal: drive regex DEFAULT-profile cert-coverage `sample_parse_failures` → 0. Root-caused (2026-06-08) into
  3 generator↔parser round-trip causes (each minimal-repro'd via `parseability_probe --parse regex`).
  **ALL THREE CLOSED:** regex DEFAULT cert-coverage `sample_parse_failures` **16→0** (count 200/seed 0,
  deterministic), seed 7 17→0, count 500/seed 0 0 — the empty-class + spacing categories are gone. A
  DISTINCT residual surfaced at seed 1 (18→1): `\98495*`, a **numeric-backreference over-generation** (the
  generator emits `\98495` = a backref to a non-existent group, which the parser correctly REJECTS
  PCRE2-faithfully). It is NOT spacing — the (b)/(c) fix UNMASKED it (the spacing bug had been emitting
  `\9 8495`, which parsed as `\9`+literals). NEW leaf `.3.12` owns it (see below):
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
  - **(b) `(?(R N)` spacing** + **(c) name/condition spacing** — **DONE (`PGEN-LEXICAL-ANNOTATIONS-0024`,
    2026-06-08)** via the LEXICAL-ANNOTATIONS tree (leaf `.6`, lexical-token cohesion). Fix = generator-only
    declarative atomicity: a rule whose return is `$text`/`$0` (`MatchedText`) or which carries a
    `@transform` directive is ONE lexical token, so (intra-rule) its internal word-boundary joins are
    suppressed and (cross-rule) a preceding word char fuses with a following atomic-token-rule segment.
    Fixes `name` (`abc`), `recursion_condition` (`R1`), `hex_escape` (`xAB`), `hex_digits`/`octal_digits`,
    `prop_name`, `directive_name_relaxed`, the comment/callout payloads. Surface-neutral (no grammar/regen/
    AST/version change). VERIFIED: cert-cov 16→0; repros pass; lib 618/0; cross-family + oracle + self-host
    gates green. CONFIRMED root-cause (2026-06-08, repros + grammar read): the word-boundary spacing's concat
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
- ID: `.3.12`  Status: **`done`** (DISCOVERED `PGEN-LEXICAL-ANNOTATIONS-0024`; CLOSED `PGEN-STORE-AWARE-GEN-0003`, 2026-06-08, via the `STORE-AWARE-GEN` tree)
  Goal: **stop the generator emitting PCRE2-INVALID numeric backreferences.** CLOSED: the
  semantic-store-aware generation MVP (`STORE-AWARE-GEN.3`) emits `regex_capture_group` facts during
  generation and prunes `numeric_backreference` when no groups exist (the sound `count==0` necessary
  condition), so the generator no longer emits `\98495`-style backrefs to non-existent groups. regex
  DEFAULT cert-coverage `sample_parse_failures` = 0 across the seed sweep (seed 0/1/7/13 + count 500).
  Surface-neutral (generator-only, no regen/version bump). The tight `$index ≤ count` bound (count ≥ 1
  over-value) is the `STORE-AWARE-GEN.4` refinement (did not arise in the sweep). Surfaced when the `.3.7`
  (b)/(c) spacing fix UNMASKED it (it had been hidden because the spacing emitted `\9 8495` = `\9`+literals,
  which parsed; the faithful generator now emits `\98495` which the parser correctly rejects). Tool-backed
  (`parseability_probe --parse regex`): `\9` PASSES, `\98`/`\984`/`\98495*` REJECT (PCRE2-faithful: `\98…`
  with no group 98 is err 115, the documented RGX-0087/0088 family). Root: `numeric_backreference =
  "\\" backreference_digits` with `backreference_digits = nonzero_digit digit+` over-generates multi-digit
  backreferences to non-existent groups. Fix direction (director-decided 2026-06-08): the grammar already
  gates this PCRE2-faithfully via `@predicate fact_count_at_least(regex_capture_group, $index)` on
  `numeric_backreference`; the defect is that the GENERATOR is predicate-blind. A fixed `numeric_bounds`
  cap would be a guess (the valid bound is the context-dependent capture-group count). The principled fix
  is the new **semantic-store-aware generation** capability — owned by the **`STORE-AWARE-GEN`** tree
  (director directive 2026-06-08, [[project_store_aware_generation]]): a generation-time semantic store
  that emits `regex_capture_group` facts as it generates and gates the backreference index by
  `fact_count_at_least`. `.3.12` is the concrete driver/closer of `STORE-AWARE-GEN.3`; verify cert-cov
  seed 1 → 0 + seed sweep + RGX conformance + oracle. Low-frequency (seed-dependent); not a spacing
  concern. Composes with `.3.11` (escape whitelist).
- ID: `.3.13`  Status: **`done`** (`PGEN-REGEX-PCRE2-0011`, 2026-07-07 session #60; regex release
  `1.1.81`→**`1.1.82`**, contract `1.1.83`→**`1.1.84`**, AST-dump schema stays `1`, ledger **`REGEX-0088`**;
  EVIDENCED by `PGEN-STIMULI-SIGNOFF-0015` — the STIMULI-SIGNOFF.13 duality-break hunt + differential oracle
  probe; design §3/§4 of `docs/tasks/STIMULI-SIGNOFF-13-post-parse-contract-design.md`)
  Goal: encode row 8 (quantified anchors) STRUCTURALLY — split `piece` so the full non-quantifiable anchor
  set `{^, $, \A, \b, \B, \G, \z, \Z, \K}` cannot take a quantifier (POSIX aliases `[[:<:]]`/`[[:>:]]` STAY
  quantifiable — oracle-verified parity); AST shape preserved; migrate/remove
  `find_invalid_quantified_anchor` same-slice. **This leaf ALSO fixed a REAL released-parser
  accepts-invalid DIVERGENCE:** PGEN accepted `\A*` `\b*` `\B?` `\G+` `\z*` `\Z*` `\K*`, PCRE2 10.47
  rejects all seven (err 109) — the validator checked only `^`/`$`; latent because the oracle corpus lacks
  the forms.
  Verification: `done — full evidence in the Acceptance Checklist below. LANDED: (1) piece gains the anchor
  branch (anchor !quantifier -> {type:"piece", atom:$1, quantifier:[]}; anchor removed from atom) — the
  !quantifier lookahead gives the ORACLE-EXACT boundary (counted forms \A{2}/\A{2,}/\A{2,3}/\A{,2} reject;
  non-quantifier braces ${/\A{a}/\A{2/\A{}/^{a} stay literal-accepted); (2) simple_escape restructured to a
  POSITIVE letter enumeration (simple_escape_tail/letter + strict 39 letters + @profiles:["relaxed"] extra
  6) excluding the 7 anchor letters in BOTH profiles — generation-faithful (the generator is
  lookahead-blind, so the old guard idiom was invisible to generation; also retro-fixes the .3.1 six-letter
  guards the same way); the o{/x{/p{/P{ 2-char guards stay (no positive spelling; generation-safe since the
  brace forms belong to the dedicated octal/hex/property rules); (3) find_invalid_quantified_anchor DELETED
  (call + fn + test); new full-stack pin regex_quantified_anchors_reject_at_the_grammar_layer_pcre2_faithfully
  in parser_registry (18 rejects / 22 accepts / both-profile tightening via parse_sample_detail_with_profile
  — NOTE: parse_sample_with_profile threads profiles only for SV — / relaxed \u regression guard); manifest
  inventory re-derived from generated/regex_return_annotations.json (5-entry diff: piece branch insert +
  shift, simple_escape_strict/relaxed → simple_escape@$5).`
  Commit: `PGEN-REGEX-PCRE2-0011`

### REGEX-PCRE2-FIDELITY.3.13 — Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — duality class: hunter `--directed-generation-goal duality_break` on regex seeds 0/7/42 emitted `DUALITY-BREAK: signature="quantifier cannot be applied directly to an anchor" … shrunk_reproducer="$*"` at every seed. Divergence: `printf '\A*' | parseability_probe --parse regex` → ACCEPT while `pcre2test` 10.47 rejects `/\A*/` err 109 (same for `\b* \B? \G+ \z* \Z* \K*` + counted forms — the full differential matrix in the STIMULI-SIGNOFF-13 design §3).
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: `piece = atom quantifier?` with `atom = … | anchor | …` (regex.ebnf:54/:149) structurally permits any anchor+quantifier, and the PCRE2 rule lived OUT-OF-BAND in `find_invalid_quantified_anchor` (regex_compile_validation.rs:1356), invisible to generation ([[project_ebnf_is_single_source_of_truth]]). WHERE the divergence: that validator matched only bytes `^`/`$` (`b'^' | b'$'` arm) — the 7 escape-anchor spellings were never checked; latent because the PCRE2-oracle corpus contains no quantified escape-anchors. Probe evidence: every reproducer failed with the CONTRACT message (not a grammar backtrack) pre-change ⇒ grammar-accepted; `\A*` failed NOWHERE ⇒ parser-accepted (the divergence).
- [x] **FIX** — fix-hierarchy GRAMMAR tier (no engine change): the anchor `piece` branch + `!quantifier` + the positive `simple_escape` letter enumeration (both profiles); validator branch deleted same-slice (the `.3.2` migration precedent).
- [x] **ADDRESSED (verified)** — verdict matrix pre→post: EXACTLY the 8 target flips (`\A*` `\A{2}` `\b*` `\B?` `\G+` `\z*` `\Z*` `\K*` ACCEPT→REJECT), zero collateral changes across the 49-pattern matrix; `$*`/`^*`/`${2}` now reject at the GRAMMAR layer (`Parser did not consume full input`, not the contract message); hunter re-run seeds 0/7/42: the quantified-anchor signature GONE (directed rejections 6/7/11 → 4/1/1 per 100; residual classes = the `.3.14`/`.3.15`/`.3.17` classes + the predicted latent start-option-position class, exactly per the design); new full-stack pin green.
- [x] **NO REGRESSION** — 34/34 still-accepted matrix samples' ASTs cmp BYTE-IDENTICAL pre/post (anchors, groups, quoted-runs, escapes, POSIX aliases, `${`); regex cert `CERTIFICATE-COVERAGE: grammar='regex' … total=200 proof=0 witness=200 UNKNOWN=0 fully_certified=true (sample_parse_failures=0 …)` at seeds 0/7/42, byte-identical re-runs (total 198→200 = the 2 net new rules, both witnessed); `--lint-grammar` 0 errors / 0 profile-orphans (200 rules); lib suites dual **880/0** (+1 pin), `generated_parsers` **838/0**, no-features **763/0**; `parse_harness_equivalence_gate` ✅ (regex stays differential-CERTIFIED); `ebnf_frontend_dual_run_gate` ✅; `regex_pcre2_compile_oracle_gate` ✅; `regex_broader_corpus_proof_gate` ✅ (0 parse failures); svpp cross-guard cert unchanged (`total=74 witness=74 UNKNOWN=0 fully_certified=true spf=0`); clippy source-strict clean.
- [x] **LOCKSTEP** — ledger `REGEX-0088` row (drift-gate authoritative Fixed-in pair) + embedding consts + tracked contract JSON bumped to 1.1.82/1.1.84; integration contract Identity + "Release 1.1.82 / Contract 1.1.84 Highlights — REGEX-0088"; regex book: changelog entry, anchors-chapter "Quantified anchors reject" section, piece/atom/escape chapters updated, tracked HTML regenerated; top-book `parser-families.md` handoff pair + `stimuli-and-quality.md` closure-progress note; ast_shape_contract manifest inventory synced; latent `\E*`/`\Q\E*` spun out to `.3.19`.
- ID: `.3.14`  Status: **`done`** (`PGEN-REGEX-PCRE2-0012`, 2026-07-08 session #61; regex release
  `1.1.82`→**`1.1.83`**, contract `1.1.84`→**`1.1.85`**, AST-dump schema stays `1`, ledger **`REGEX-0089`**
  + **`REGEX-0090`**; EVIDENCED `-0015`)
  Goal: encode row 6's STRUCTURAL residual
  (names were `.3.2`): name-class-conditional directive shapes — MARK-shorthand payload REQUIRED non-empty
  (`(*:)` rejects, err 166); verbs (`MARK ACCEPT F FAIL COMMIT PRUNE SKIP THEN`) take `:`-suffix only
  (`(*SKIP=)`/`(*PRUNE=)` reject, err 160; `(*PRUNE:)` stays accepted — oracle-verified); `=`-suffix only for
  the numeric-value start options (digits payload). Honest bounds: the start-option POSITION check (`a(*UTF)`)
  is contextual — stays in the validator until the capstone finds a shape; quantified-verb (`(*:x)+` rejects,
  `(*ACCEPT)+` accepts) adjudicated during design. Migrate the matched validator branches same-slice.
  Verification: `done — full evidence in the Acceptance Checklist below. LANDED: (1) directive_named split
  into name-class-conditional branches (directive_mark_named "MARK"+required :-payload / directive_verb_named
  7 verbs+optional :-payload / directive_limit_named 4 LIMIT_*+required =digit+ / directive_option_named 21
  bare-only options with literal payload:[] / @profiles:["relaxed"] directive_relaxed_named with an INLINE
  recognized-name-at-boundary negative lookahead so strict shapes bind in BOTH profiles — inlined over the
  positively-witnessed name rules on purpose: a helper referenced only under a negative lookahead can never
  be coverage-witnessed); directive_mark_shorthand requires the payload (the duality-class kill). (2) The
  ORACLE MATRIX (96 patterns, pcre2test 10.47) found and this slice FIXED 3 real accepts-invalid divergence
  spellings beyond the migration set: =digits on non-LIMIT options ((*UTF=5)/(*CR=5)/(*TURKISH_CASING=5)) +
  bare (*LIMIT_HEAP) [REGEX-0089, grammar-encoded] and mid-pattern =-form options a(*LIMIT_HEAP=500) /
  (*FAIL)(*LIMIT_HEAP=5)a [REGEX-0090 — the validator position check ran only in the bare-')' else-branch;
  now unconditional for recognized option names; position stays validator-owned per the honest bound]. (3)
  Validator migration: MARK-shorthand/MARK-required/verb-'='/option-shape branches DELETED;
  pcre2_verb_argument_rule → is_pcre2_verb_name; quantified-verb checks KEPT (→ .3.20). (4) CERT REGRESSION
  root-caused TOOLS-FIRST and fixed in-slice: 4 UNKNOWNs = the stimuli generator has NO generation arm for
  builtin_any_char (mini-grammar isolation: ( !")" builtin_any_char )+ errors "Missing rule"; latent because
  every prior use sat under * where EMPTY generation sufficed — exactly how (*:) was ever emitted) → fix =
  directive_payload_required pairs a positively-enumerated generatable core (directive_payload_core) with the
  superset branch (parse-identical union under longest_match; engine debt → NEW STIMULI-SIGNOFF.14); 1
  UNKNOWN (directive_payload_suffix "NO reach path") = an UNGATED rule referenced only from a PROFILE-GATED
  rule stays in the default cert universe with no reach path (scratch-slot V6 reproduction; the gated parent
  itself is excluded — the 210=200+11-1 arithmetic) → fix = @profiles:["relaxed"] on directive_payload_suffix
  (the directive_name_relaxed precedent; it IS relaxed-only now). (5) In-slice adjudications: quantified-verb
  encoding SPUN to .3.20 (different mechanism — piece-level quantifiability — + needs a relaxed-semantics
  decision; hunter then OBSERVED it at seed 42: (*F)+); LIMIT value RANGE spun to .3.21 (oracle: overflow
  rejects err 160; value-constraint class blocked on 13.2 like .3.16). New full-stack pin
  regex_verb_argument_shapes_reject_at_the_grammar_layer_pcre2_faithfully (25 rejects / 37 accepts /
  both-profile tightening / relaxed unrecognized-name regression). Manifest inventory re-derived (12-entry
  diff: directive_named out, 11 per-class entries in).`
  Commit: `PGEN-REGEX-PCRE2-0012`

### REGEX-PCRE2-FIDELITY.3.14 — Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — duality classes: the `.13.1` hunter emitted `DUALITY-BREAK: signature="MARK shorthand verb requires a non-empty argument" … shrunk_reproducer="(*:)"` (seeds 0/7/42) and `signature="PCRE2 verb is malformed" … shrunk_reproducer="(*PRUNE=)"` (seed 7). Divergences: pre-change probe matrix vs `pcre2test` 10.47 — `printf '(*UTF=5)' | parseability_probe --parse regex` → ACCEPT / PCRE2 err 160; same for `(*CR=5)` `(*TURKISH_CASING=5)` `(*LIMIT_HEAP)` `a(*LIMIT_HEAP=500)` `(*FAIL)(*LIMIT_HEAP=5)a`.
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: `directive_mark_shorthand = ":" directive_payload_simple?` (payload OPTIONAL) + `directive_payload_suffix` attaching `:`/`=` to ANY `directive_name` (regex.ebnf pre-change :1143-1149) structurally admit every arg shape; the PCRE2 per-name-class rules lived OUT-OF-BAND in `find_invalid_verb_construct` (regex_compile_validation.rs :436-514), invisible to generation ([[project_ebnf_is_single_source_of_truth]]). WHERE the divergences: the validator's start-option arm accepted `=<digits>)` for EVERY option name and bare `)` unconditionally (no LIMIT-vs-plain distinction, :455-470) — REGEX-0089; and its POSITION check sat in the bare-`)` `else`-branch so `=`-forms skipped it (:456 vs :476) — REGEX-0090. Cert-regression root causes: `Error: Missing rule 'builtin_any_char'` (mini-grammar `( !")" builtin_any_char )+` isolation; zero generation arms in stimuli_generator.rs vs the native matcher at ast_based_generator.rs:1022) and the scratch-slot V6 reproduction of the gated-parent/ungated-child cert-universe signature (`name_part`/`suffix_part` UNKNOWN "NO reach path" exactly mirroring `directive_payload_suffix`).
- [x] **FIX** — fix-hierarchy GRAMMAR tier (no engine change): name-class-conditional `directive_named` branches + required/equals/bare payload rules + the guarded relaxed catch-all + `directive_payload_core` generatable-core pairing + `@profiles` gate on the now-relaxed-only `directive_payload_suffix`; validator shape-branches deleted same-slice (the `.3.2`/`.3.13` precedent), position check made unconditional (REGEX-0090, validator-owned per the honest bound).
- [x] **ADDRESSED (verified)** — verdict matrix pre→post over 96 oracle-pinned patterns: EXACTLY the 6 divergence flips (`(*UTF=5)` `(*CR=5)` `(*TURKISH_CASING=5)` `(*LIMIT_HEAP)` `a(*LIMIT_HEAP=500)` `(*FAIL)(*LIMIT_HEAP=5)a` ACCEPT→REJECT), zero collateral changes; `(*:)`/`(*MARK)`/`(*SKIP=)`-class forms now reject at the GRAMMAR layer; hunter re-run seeds 0/7/42: BOTH `.3.14` signatures GONE (directed rejections 4/1/1 → **1/1/2** per 100; residuals = the tracked `.3.15` `[\E]`, `.3.20` `(*F)+` — now OBSERVED — and the position class, exactly per the design); relaxed-profile matrix: parity preserved (invalid recognized-name shapes reject in relaxed too; `(*FOO)`/`(*FOO=x)`/`(*SKIPX)`/`(*LIMIT_HEAPX=5)` stay relaxed-accepted); new full-stack pin green in-suite.
- [x] **NO REGRESSION** — 30/30 still-accepted directive/verb matrix ASTs cmp BYTE-IDENTICAL pre/post (`--parse-dump-ast-pretty`); regex cert `CERTIFICATE-COVERAGE: grammar='regex' … total=210 proof=0 witness=210 UNKNOWN=0 fully_certified=true (sample_parse_failures=0 …)` at seeds 0/7/42 (total 200→210 = the 11 net-new rules witnessed, `directive_relaxed_named` profile-excluded by construction); `--lint-grammar` 0 errors / 0 profile-orphans (210 rules); lib suites dual **879/0** (= 880 − 2 migrated validator tests + 1 pin), `generated_parsers` **837/0**, no-features **761/0**; `parse_harness_equivalence_gate` ✅ (regex stays differential-CERTIFIED); `ebnf_frontend_dual_run_gate` ✅; `regex_pcre2_compile_oracle_gate` ✅; `regex_broader_corpus_proof_gate` ✅; svpp cross-guard cert `total=74 witness=74 UNKNOWN=0 fully_certified=true spf=0` ×3 seeds unchanged; `mdbook_docs_gate` + `regex_parser_book_gate` ✅; clippy source-strict clean (generated stage = the known pre-existing 178 `eq_op` debt, non-strict by design).
- [x] **LOCKSTEP** — ledger `REGEX-0089`+`REGEX-0090` rows (drift-gate authoritative Fixed-in pair) + embedding consts + tracked contract JSON bumped to 1.1.83/1.1.85; integration contract Identity + "Release 1.1.83 / Contract 1.1.85 Highlights — REGEX-0089/0090" + a supersession pointer on the `.3.2` historical note; regex book: changelog entry, `rules-misc.md` directive section REWRITTEN to the current per-class truth (also purging the stale pre-typed `directive_payload_char` era text), `json-carrier.md` inventory rows re-derived, tracked HTML regenerated; top book `parser-families.md` handoff pair + `stimuli-and-quality.md` closure-progress note (incl. the STIMULI-SIGNOFF.14 finding); ast_shape_contract manifest inventory synced (12-entry diff); new leaves `.3.20`/`.3.21` + `STIMULI-SIGNOFF.14` recorded; `docs/TASK_TREE.md` index updated.
- ID: `.3.15`  Status: **`done`** (`PGEN-REGEX-PCRE2-0013`, 2026-07-08 session #62; regex release
  `1.1.83`→**`1.1.84`**, contract `1.1.85`→**`1.1.86`**, AST-dump schema stays `1`, ledger **`REGEX-0091`**;
  EVIDENCED `-0015`)  Goal: encode row 7's residual (empty-`[]` was `.3.7(a)`): class-member VISIBILITY —
  `stray_class_end_quote` (`\E`), empty `quoted_class_literal` (`\Q\E`) and `empty_quoted_class_literal`
  are PCRE2-INVISIBLE members; the non-empty-class requirement must count only VISIBLE members, allowing
  invisible prefixes before the first-`]`-literal form (`[\E]` / `[\Q]` / `[\E\E]` reject err 106;
  `[\E]x]` stays ACCEPTED — oracle-verified). Migrate the matched class-analyzer paths.
  IN-SLICE ADJUDICATION (design, session #62): the pre-encode oracle matrix (84 cells + 4 range cells,
  pcre2test 10.47) widened the class — PCRE2's class-open model recognizes the NEGATION caret THROUGH
  invisibles (`[\E^]` rejects 106 = the `^` negated, `]` became the initial literal, unterminated) and a
  caret after the negation is an ordinary MEMBER (`[^^]` ACCEPTS — PGEN wrongly rejected, previously
  UNTRACKED). The caret and visibility mechanisms are ONE entangled model in both layers (the validator's
  `^`-skip at `regex_compile_validation.rs:769` approximated negation-through-invisibles), so splitting
  would leave either new accepts-invalid (`[\E^]`) or the flips unfixed → ONE slice encoded the full model.
  17 rejects-valid verdict flips; 3 accepted inputs (`[\E^a]` `[\Q\E^a]` `[\E^-z]`) got the SEMANTIC AST
  correction (negation was mis-parsed as members).
  Verification: `done — full evidence in the Acceptance Checklist below. LANDED: (1) char_class → 3 alts
  ("[" class_negated_open? class_zero_width* class_initial_close class_body "]" / "[" class_negated_open
  class_body_nonempty "]" / "[" class_body_nonempty_nocaret "]"); class_negated_open = class_zero_width*
  negation class_zero_width* -> $2 (negation-through-invisibles; bare-true slot convention);
  class_body_nonempty(_nocaret) = class_zero_width* class_item_visible(_nocaret) class_item*
  -> [$1*, $2, $3*] (mixed-position spread VERIFIED; body list byte-identical);
  class_safe_special_nocaret (the 29-set minus ^) keeps the no-negation first-visible slot caret-free
  STRUCTURALLY (generation-sound, no lookahead); quoted_class_literal_nonempty (char+) fills the visible
  slot; empty_quoted_class_literal -> {type:"class_quoted_literal", body:[]} (zw-position byte-parity);
  !"Q" !"E" guards on BOTH class_simple_escape_strict ($11→$13) / _relaxed ($5→$7) — \Q always the
  quote-opener in classes, so [\Q]/[\Qa]/[a\Q] reject at the grammar. (2) Validator migration:
  scan_char_class's open phase rewritten to the PCRE2 model via new skip_invisible_class_items
  (invisibles → optional ^ → invisibles → optional ]-literal that can seed a RANGE — [\E]-z] accepts /
  [\E]-A] rejects err-108-faithfully, post-land oracle-pinned); the has_substantive_item machinery + the
  no-substantive-] "unterminated character class" error + the ^-skip DELETED (4th compile-contract
  migration); range/POSIX/escape-letter/\N checks unchanged. (3) Scratch-slot design validation BEFORE
  landing: control run (unmodified grammar) byte-identical modulo rule_name; candidate 80/80
  oracle-expected verdicts + still-accepted ASTs byte-identical; forced char_class duality probe
  37/129→7/130 rejects (residual = the pre-existing generation-truncation class, STIMULI-SIGNOFF-tracked).
  One landing bug caught by the post-land matrix re-run: a python-heredoc escaping slip wrote "\Q" for
  "\\Q" in quoted_class_literal_nonempty — fixed, full matrix re-verified. Matrix + baselines + pre/post
  ASTs archived in session scratchpad cls_matrix/. New full-stack pin
  regex_class_member_visibility_rejects_at_the_grammar_layer_pcre2_faithfully (18 rejects / 40 accepts /
  negated-AST assertion / both-profile agreement / relaxed class-escape-letter regression). Manifest
  inventory re-derived (196→202: +6 new annotations, 3 renumbered).`
  Commit: `PGEN-REGEX-PCRE2-0013`

### REGEX-PCRE2-FIDELITY.3.15 — Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — duality class: the `.13.1` hunter emitted `DUALITY-BREAK: signature="unterminated character class" … shrunk_reproducer="[\E]"` at EVERY seed (the standing residual). Divergences: pre-change probe matrix vs `pcre2test` 10.47 — `printf '[\E]x]' | parseability_probe --parse regex` → REJECT ("unterminated character class") while PCRE2 ACCEPTS `/[\E]x]/`; same rejects-valid for 16 more spellings incl. `[^^]` (PCRE2 ACCEPT — negated class of a literal `^`; PGEN REJECT, previously untracked).
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: `class_body_nonempty = class_item+` (regex.ebnf) counted the PCRE2-INVISIBLE items (stray `\E`, empty `\Q\E`) toward class non-emptiness, and the `class_simple_escape_*` `any_char` catch-alls admitted `\Q`/`\E` as shorthand escapes — so the generator could emit invisible-only classes ([[project_ebnf_is_single_source_of_truth]]). WHERE the rejects-valid: `scan_char_class` (regex_compile_validation.rs:744) applied the initial-`]`-literal rule only IMMEDIATELY after `[`/`[^` (never after invisibles), errored "unterminated character class" on `]`-with-no-substantive-item (:759-767), and silently SKIPPED a pre-member `^` (:769) instead of modeling negation-through-invisibles — so `[\E]x]`-family and `[^^]`-family valid patterns rejected, and `[\E^a]`-family accepted with a PCRE2-unfaithful non-negated AST.
- [x] **FIX** — fix-hierarchy GRAMMAR tier (no engine change): the full PCRE2 class-open model encoded (3-alt `char_class`, `class_negated_open`, visibility-led bodies with `-> [$1*, $2, $3*]`, structural nocaret first-visible slot, `!"Q"`/`!"E"` guards both profiles); matched validator class-open paths deleted same-slice, scanner aligned via `skip_invisible_class_items` (the `.3.2`/`.3.13`/`.3.14` migration precedent).
- [x] **ADDRESSED (verified)** — verdict matrix pre→post over 84 oracle-pinned cells (80 strict + 4 relaxed): EXACTLY the 17 divergence flips (all REJECT→ACCEPT: `[\E]x]` `[\Q\E]x]` `[\E\E]x]` `[\E\Q\E]x]` `[^\E]x]` `[^\Q\E]x]` `[\E]]` `[\Q\E]]` `[^\E]]` `[^^]` `[^^]x]` `[^\E^]` `[^\Q\E^]` `[\E^]x]` `[\E^\E]x]` `[\E^^]` `[\E\E^]y]`), zero collateral, BOTH profiles oracle-faithful; invisible-only/unterminated-quote/caret-negated-empty families now reject at the GRAMMAR layer; post-land range pins `[\E]-z]`/`[]-z]` ACCEPT + `[\E]-A]`/`[]-A]` REJECT (err-108-faithful); hunter re-run seeds 0/7/42: the `[\E]` signature GONE (directed rejections 1/1/2 → **1/0/0** per 100; sole residual = the tracked `.3.17` scs class); new full-stack pin green in-suite.
- [x] **NO REGRESSION** — still-accepted matrix ASTs cmp BYTE-IDENTICAL pre/post in BOTH profiles except EXACTLY the 3 documented semantic corrections (`[\E^a]` `[\Q\E^a]` `[\E^-z]` → `negated:true`, invisibles dropped — the fix's intended effect, ledger-documented); regex cert `CERTIFICATE-COVERAGE: grammar='regex' … total=217 proof=0 witness=217 UNKNOWN=0 fully_certified=true (sample_parse_failures=0 …)` at seeds 0/7/42, byte-identical re-runs (total 210→217 = the 7 net-new rules, all witnessed); `--lint-grammar` 0 errors / 0 profile-orphans (217 rules); lib suites dual **880/0** (+1 pin), `generated_parsers` **838/0**, no-features **761/0**; validator unit tests 51/51 pre AND post (no stale expectations); `parse_harness_equivalence_gate` ✅ (regex stays differential-CERTIFIED); `ebnf_frontend_dual_run_gate` ✅; `regex_pcre2_compile_oracle_gate` ✅; `regex_broader_corpus_proof_gate` ✅ (0 parse failures); svpp cross-guard cert `total=74 witness=74 UNKNOWN=0 fully_certified=true spf=0` ×3 seeds unchanged; `mdbook_docs_gate` + `regex_parser_book_gate` ✅; clippy: zero findings in touched regions (generated stage = the known pre-existing debt, non-strict by design).
- [x] **LOCKSTEP** — ledger `REGEX-0091` row (drift-gate authoritative Fixed-in pair) + embedding consts + tracked contract JSON bumped to 1.1.84/1.1.86; integration contract Identity + "Release 1.1.84 / Contract 1.1.86 Highlights — REGEX-0091" (verdict table + AST-correction + rejection-layer + duality-closure notes); regex book: changelog entry, `rules-char-class.md` REWRITTEN to the current typed truth (the chapter was raw-envelope-era stale — pre-.3.7 rule text, wrong walking code), `examples-char-class.md` stale "future direction" tail replaced with the class-open model examples (verified shapes), `json-carrier.md` inventory rows (+6 new / char_class updated), tracked HTML regenerated; top book `parser-families.md` handoff pair + `stimuli-and-quality.md` closure-progress note; ast_shape_contract manifest inventory re-derived (196→202); `docs/TASK_TREE.md` index updated.
- ID: `.3.16`  Status: **`done`** (`PGEN-REGEX-PCRE2-0014`, 2026-07-08 session #64; SURFACE-NEUTRAL —
  release stays `1.1.84`, contract stays `1.1.86`, schema stays `1`, NO ledger row; EVIDENCED `-0015`;
  was UNBLOCKED by `STIMULI-SIGNOFF.13.2`)
  Goal (as declared): encode row 5 — `@range: [0, 255]` on a dedicated `callout_number` rule; migrate
  `find_invalid_numeric_callout`.
  **IN-SLICE DESIGN ADJUDICATION (tools-first): the declared `@range` design was REFUTED and the bound
  encoded STRUCTURALLY instead.** The SC-08 value-constraint machinery is ATOM-scoped on BOTH sides —
  the parse guard is spliced only after `match_string`/`match_regex` atoms of the constraint-bearing
  rule (`semantic_value_constraint_tokens` call sites `ast_based_generator.rs:3993/:4054/:4090/:4126/:4142`;
  the `.13.2` interpreter mirror doc pins it: "`rule_reference` delegates to the referenced rule's own
  guards", `parse_harness_interpreter.rs:2505`) and generation sampling lives in `generate_regex_sample`
  (`stimuli_generator.rs:11545`, regex atoms only) — and the self-hosted regex grammar HAS no regex/single
  literal atom spanning a multi-digit number. Generation-probed on a mini grammar: `@range: [0, 255]` on a
  native `digits` body emitted `7562`, `3245`, `0935`, `504`, … (out-of-range) ⇒ `@range` would be INERT on
  both sides and deleting the validator would have opened an accepts-invalid hole. The `.13.2` unblock was
  real only for `/regex/`-bodied rules (the suite shape). SECOND probed hazard: an Or-ROOTED rule gets NO
  rule-level `@transform` span fallback (`semantic_span_transform_tokens` excludes `ASTNode::Or`,
  `ast_based_generator.rs` — emitted-parser probes: choice-root and inline-group-root minis have ZERO
  `__pgen_span_text` blocks; the wrapper shape has it) ⇒ the typed-int carrier needs the wrapper
  indirection. Oracle pinned FIRST (`pcre2test` 10.47): the bound is the VALUE with ARBITRARY leading
  zeros — `(?C255)`/`(?C0255)`/`(?C00)`/`(?C000000000255)`/`(?C010)` ACCEPT; `(?C256)`/`(?C262)`/
  `(?C000000000256)`/`(?C999999999999999999999)` REJECT err 138; condition-callout site identical.
  Verification: `done — full evidence in the Acceptance Checklist below. LANDED: (1) grammar (structural
  tier): callout_arg = callout_number | callout_string; @transform-span callout_number = callout_number_body
  (wrapper — Or-root span-transform hazard); callout_number_body = "0"+ callout_number_core? |
  callout_number_core; callout_number_core = the 5-branch nonzero-led ≤255 encode (250-255 / 200-249 /
  100-199 / 10-99 / 1-9) — out-of-range runs have no fully-consuming parse (the trailing ")" fails) and
  generation is in-range BY CONSTRUCTION (120/120 across seeds 0/7/42 from entry callout_number).
  (2) find_invalid_numeric_callout DELETED (fn + call + 2 unit tests; 5th compile-contract migration).
  (3) New full-stack pin regex_numeric_callout_range_rejects_at_the_grammar_layer_pcre2_faithfully
  (10 rejects / 19 accepts / typed-int AST assertion "arg":255 for (?C0255) / both-profile agreement).
  (4) In-slice incident root-caused tools-first: a mid-verification cert read UNKNOWN=3
  (parsed=true witnessed_target=false on the 3 new rules) — mtime proof: debug ast_pipeline (08:19)
  predated the regen (09:04), so the WITNESS side ran the STALE embedded parser (no callout_number rules
  to witness) while the planner read the new .ebnf; dual-feature rebuild → 220/220 clean ×3 seeds.
  Reinforces [[feedback_verify_sv_parser_regen_mtime]].`
  Commit: `PGEN-REGEX-PCRE2-0014`

### REGEX-PCRE2-FIDELITY.3.16 — Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — duality class (EVIDENCED `-0015`): `printf '(?C262)' | parseability_probe --parse regex` → `Error: parse_full rejected sample … : numeric callout argument exceeds PCRE2 compile limit 255` (the CONTRACT message ⇒ grammar-ACCEPTED, validator-layer reject; same for the condition site `(?(?C262)(?=y)x|z)`); generation duality: `ast_pipeline grammars/regex.ebnf --generate-stimuli --count 60 --seed 0 --entry-rule callout` → **5/60 parser-rejected** (`(?C4135)` `(?C4612)` `(?C359)` `(?C4601)` `(?C3742)`) — the generator is validator-blind ([[project_ebnf_is_single_source_of_truth]]).
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: the PCRE2 bound lived OUT-OF-BAND in `find_invalid_numeric_callout` (`regex_compile_validation.rs:364`, call `:34`) while the grammar's `callout_arg = digits | callout_string` admits ANY digit run — invisible to generation. WHERE the declared design fails: the `@range` guard + sampling are ATOM-scoped (codegen call sites `:3993/:4054/:4090/:4126/:4142`; interpreter mirror "rule_reference delegates" `parse_harness_interpreter.rs:2505`; generation `stimuli_generator.rs:11545`) — mini-grammar probe: `@range` on a native `digits` body emits `7562`/`3245`/`0935`/… (inert both sides); plus the Or-root `@transform` span-fallback exclusion (`semantic_span_transform_tokens`, emitted-parser probes: 0 `__pgen_span_text` on choice/group roots vs present on the wrapper shape).
- [x] **FIX** — fix-hierarchy GRAMMAR tier (structural; no engine change, no new annotation): the `callout_number`/`callout_number_body`/`callout_number_core` structural [0, 255] encode with the `@transform`-span wrapper preserving the exact typed-int carrier; validator check deleted same-slice (the `.3.2`/`.3.13`/`.3.14`/`.3.15` migration precedent).
- [x] **ADDRESSED (verified)** — post-land matrix **31/31 oracle-exact** (12 rejects incl. `(?C0256)`/`(?C000000000256)`/`(?C999999999999999999999)`/both sites + 19 accepts incl. `(?C0255)`/`(?C00)`/`(?C010)`/string forms), BOTH profiles agree (pin test); rejection LAYER flipped ((?C262): contract message → `Parser did not consume full input at position 0` = grammar layer, code `E_PARSE_FAILURE` unchanged); duality probe **5/60 → 0/60** (seed 0); generation from `callout_number` **120/120 in-range** ×seeds 0/7/42; new full-stack pin green in-suite.
- [x] **NO REGRESSION** — accepted-callout ASTs **19/19 cmp BYTE-IDENTICAL** pre/post (`--parse-dump-ast-pretty`, incl. `(?C0255)`→`"arg": 255`, `(?C)`→`"arg": []`, string forms, condition site); regex cert `CERTIFICATE-COVERAGE: grammar='regex' … total=220 proof=0 witness=220 UNKNOWN=0 fully_certified=true (sample_parse_failures=0, proof_reverify_failures=0)` ×seeds 0/7/42 (217→220 = exactly the 3 net-new rules, all witnessed); `--lint-grammar` 0 errors / 0 profile-orphans (220 rules); svpp cross-guard cert `total=74 witness=74 UNKNOWN=0 fully_certified=true spf=0` ×3 seeds unchanged; `regex_pcre2_compile_oracle_gate` ✅ **byte-identical baseline** (1857 matches / 46 false-rejects / 292 false-accepts / 338 mismatches = conformance-NEUTRAL); `regex_broader_corpus_proof_gate` ✅ (0 parse failures); `parse_harness_equivalence_gate` ✅ 4/4 (regex stays differential-CERTIFIED over the new rules); `ebnf_frontend_dual_run_gate` ✅; lib suites dual **880/0** / `generated_parsers` **838/0** / no-features **760/0** (= baselines −2 migrated validator tests +1 pin); `check_regex_self_hosting.sh` OK (new rules are literal-only); clippy `clippy_source_all_targets: ok` (generated stage = the known pre-existing 178 `eq_op` debt, non-strict by design); `mdbook_docs_gate` + `regex_parser_book_gate` ✅.
- [x] **LOCKSTEP** — NO version bump (conformance- & surface-neutral, the `.3.1`/`.3.2` precedent; embedding consts untouched; no ledger row — no released divergence existed); integration contract **"Maintenance Update 2026-07-08"** (before/after table: identical language, byte-identical ASTs, code-not-message guidance); regex book: `rules-misc.md` new § `callout_number`, `json-carrier.md` callout row, `changelog-index.md` maintenance entry, tracked HTML regenerated; top book `stimuli-and-quality.md` duality-closure note (incl. the @range-inert adjudication); AST shape-contract manifest UNCHANGED (inventory stays 202 entries — no return-annotation change; shape gate green in-suite); tree + `docs/TASK_TREE.md` index + live docs updated.
- ID: `.3.17`  Status: **`done`** (`PGEN-STIMULI-SIGNOFF-0017`, 2026-07-08 session #65, paired with
  `STIMULI-SIGNOFF.13.4` which owns the general capability; conformance- & surface-NEUTRAL — NO version
  bump: release `1.1.84`/contract `1.1.86`/schema `1` stay, no ledger row; full evidence in the
  Acceptance Checklist below. EVIDENCED `-0015`, re-reproduced at HEAD `ed1861be`: hunter seed 0
  `(*scs:('_'))` + entry-probe 54/57 scs samples contract-rejected — 34 numeric "unavailable capture" +
  20 named "unknown named capture")  Goal: row 9
  generation-side — scs capture-list references. Parse-time predicate is UNSOUND (forward refs LEGAL:
  `(*scs:('a'))(?<a>x)` oracle-accepted), so the parse-side check STAYS in the validator; the generator-side
  fix = store-aware generation draws `name_ref` from generation-emitted capture-name facts (the sound
  already-generated subset, the `.3.12` precedent) via a grammar-declared generation-side gate.
  REGEX APPLICATION of the `.13.4` capability (parse-language + AST byte-identical, generator-side only —
  NO version bump): (a) `capture_name = name -> $1` wrapper in the three named-open markers
  carrying `@gen_emit_fact {kind: regex_capture_name, name: $name}` (whole-render resolution registers the
  REAL generated name, the instant the name renders — maximizing the sound prefix); (b) the scs list split
  `returned_capture_group = scs_capture_number | scs_capture_name_ref` with
  `scs_capture_number = signed_digits -> $1` gated `@gen_predicate fact_count_at_least(regex_capture_group,
  $value)` (index draw 1..=count — the `.3.12` tight-bound refinement, needed here because scs digits ARE
  reachable) and `scs_capture_name = name -> $1` gated `@gen_predicate has_fact(regex_capture_name, $text)`
  (name draw from live facts). Also narrows the (unvalidated) subroutine-call capture-list generation to the
  same sound subset — harmless, parse-side unchanged.

### REGEX-PCRE2-FIDELITY.3.17 — Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — see `STIMULI-SIGNOFF.13.4`'s REPRODUCE box (same slice): hunter seed 0 at HEAD `ed1861be` → `DUALITY-BREAK: … shrunk_reproducer="(*scs:('_'))"`; scs entry-probe **54/57** samples parser-rejected (34 numeric-unavailable + 20 unknown-named).
- [x] **ROOT CAUSE (WHY + WHERE)** — the `.13.1` design §2 row 6: `returned_capture_group = signed_digits | name_ref` (regex.ebnf:1016 pre-change) unconstrained; validator full-inventory check `validate_scan_substring_capture_refs` (regex_compile_validation.rs:1240); forward refs LEGAL (pcre2test 10.47: `(*scs:('a'))(?<a>x)` compiles — re-verified) ⇒ parse-side encode UNSOUND; the generator never consults the contract.
- [x] **FIX** — the `.13.4` capability applied to `grammars/regex.ebnf`: `capture_name` carrier (+`@gen_emit_fact`) in the 3 named-open markers; `returned_capture_group = scs_capture_number | scs_capture_name_ref` (+`@gen_predicate` draws on the two value rules); parse language + AST shape identical by construction (`-> $1`/`-> $2` pass-throughs mirroring `name_ref`).
- [x] **ADDRESSED (verified)** — hunter re-run seeds 0/7/42: the scs signature **GONE** (seed 0: 2 breaks → 1, the survivor = the separately-tracked start-option-position class `E(*UTF16)` — now OBSERVED, was latent; seeds 7/42: 0 breaks); entry-relative scs generation now fails HONESTLY at the count-prune (`STORE-AWARE-GEN: rule 'scs_capture_number' fact_count_at_least predicate unsatisfiable`) — the documented forward-only-position bound; full-entry witness samples draw live values: `()(*scs:(1))` / `(?<A>)(*scs:(<A>))` (DEBUG_PROBES).
- [x] **NO REGRESSION** — accepted-pattern ASTs **21/21 cmp BYTE-IDENTICAL** pre/post (`--parse-dump-ast-pretty` matrix: named/quote/python groups, scs numeric/named/relative `+0`/`-1`/forward forms, `\k` both spellings, `(?P=…)`, conditional `(?(<a>)y|z)`, subroutine-call captures `(x)(?1(1))`); regex cert `CERTIFICATE-COVERAGE: grammar='regex' … total=224 proof=0 witness=224 UNKNOWN=0 fully_certified=true (sample_parse_failures=0, proof_reverify_failures=0)` ×seeds 0/7/42 (220→224 = exactly the 4 net-new rules, all witnessed — the initial `UNKNOWN=1` on `scs_capture_name_ref` was root-caused via DEBUG_PROBES and fixed by the `.13.4` count-gate mandatory descent); `--lint-grammar` 0 errors (224 rules); `regex_pcre2_compile_oracle_gate` ✅ **byte-identical baseline** (1857/46/292/338); `regex_broader_corpus_proof_gate` ✅; `parse_harness_equivalence_gate` ✅ (regex stays differential-CERTIFIED over the new grammar); `ebnf_frontend_dual_run_gate` ✅; svpp cert byte-identical ×3 seeds (stash A/B); lib suites 766/844/886 all green; `check_regex_self_hosting.sh` OK; clippy source ok; `mdbook_docs_gate` + `regex_parser_book_gate` + `ebnf_parser_book_gate` ✅.
- [x] **LOCKSTEP** — NO version bump (conformance- & surface-neutral, the `.3.16` precedent; embedding consts untouched; no ledger row — the parse surface is unchanged and the parse-side check stays in the validator until capstone `.4`); integration contract **"Maintenance Update 2026-07-08 — REGEX-PCRE2-FIDELITY.3.17"** (before/after table: identical language, 21/21 byte-identical ASTs, unchanged codes/messages); regex book: `changelog-index.md` maintenance entry, `json-carrier.md` +5 inventory rows (`capture_name`/`scs_*`), tracked HTML regenerated; AST shape-contract manifest `regex_v1.json` inventory 202→207 (exactly the 5 new declared annotations, multiset-verified vs HEAD; gate green); top book + matrix + spec + ebnf book in the `.13.4` checklist; trees + `docs/TASK_TREE.md` + live docs updated.
- ID: `.3.22`  Status: `pending` (🔎 NEW FINDING 2026-07-08 session #65, oracle-verified while scoping
  `.3.17`)  Goal: the NAMED-REFERENCE UNKNOWN-NAME family — PGEN ACCEPTS `\k<zzz>` `(?P=zzz)` `(?&zzz)`
  `\g{zzz}` (no group named `zzz` anywhere) which PCRE2 10.47 REJECTS (err 115 "reference to non-existent
  subpattern"); control `(?'aa'x)\k<aa>` parity-ACCEPTED by both. The validator has NO named-reference
  inventory check (only malformed-escape shape checks — `regex_compile_validation.rs:188/:194`), so this is
  a latent accepts-invalid divergence in the RELEASED parser, invisible to the duality hunter (generator AND
  parser agree — only oracle-differential coverage can see it; the `.3.19` discovery class). Fix shape:
  parse-time inventory encoding is the same two-pass problem as scs (forward refs LEGAL: `\k<a>(?<a>x)`
  oracle-accepted) ⇒ parse-side stays validator-tier until the capstone `.4` two-pass design; the
  GENERATION side can meanwhile draw named-backref/subroutine names from the `.3.17`
  `regex_capture_name` facts (the same `@gen_predicate` idiom) if the hunter ever observes over-generation
  there (today the generator's named-ref sites are rare enough that no break was observed at the
  100-sample budget). Ledger row + oracle-matrix + encode design = this leaf, sequenced after `.3.18`–`.3.21`.
- ID: `.3.18`  Status: `pending` (LATENT class, oracle-verified `-0015`)  Goal: row 4 — counted-quantifier
  bounds (`a{5,2}` rejects err 104; `{,>65535}` limits); `@predicate`/structural per the `.1` table; the
  generator currently CAN emit out-of-order bounds (not yet observed at the 100-sample hunter budget).
- ID: `.3.19`  Status: `pending` (LATENT accepts-invalid divergences, oracle-verified 2026-07-07 during the
  `.3.13` implementation)  Goal: the ZERO-WIDTH-QUANTIFIED family beyond anchors — PGEN ACCEPTS `\E*`
  (stray `\E` is zero-width) and `\Q\E*` (empty quoted literal) which PCRE2 10.47 REJECTS (err 109);
  `\Qa\E*` correctly accepted by both. Both sides of PGEN (generator AND parser) agree today, so the
  duality hunter can NEVER surface these — only oracle-differential coverage can; candidate encode =
  the stray-`\E`/empty-`\Q\E` atoms joining the anchor treatment (non-quantifiable piece forms). Distinct
  from `.3.13`'s anchor-rule scope; spun out to keep that slice bounded.
- ID: `.3.20`  Status: `pending` (spun out of `.3.14`'s in-slice adjudication, 2026-07-08; oracle matrix
  already pinned; NO LONGER latent-only — the post-`.3.14` hunter re-run OBSERVED it at seed 42:
  `DUALITY-BREAK: signature="only ACCEPT verb may be quantified…" shrunk_reproducer="(*F)+"`)
  Goal: QUANTIFIED-VERB encoding — only `(*ACCEPT)` may take a quantifier (oracle
  `pcre2test` 10.47: `(*ACCEPT)+` AND `(*ACCEPT:x)+` ACCEPT; `(*:x)+` `(*PRUNE)+` `(*FAIL)*` `(*MARK:x)+`
  all err 109). Candidate mechanism = the `.3.13` piece-level split (non-ACCEPT `directive_verb` forms get
  a `!quantifier` piece branch; the ACCEPT-named form stays quantifiable in `atom`). Adjudicated OUT of
  `.3.14` to keep that slice bounded to arg shapes AND because it needs its own relaxed-semantics decision:
  quantified UNKNOWN-name verbs (`(*foo)+`) are relaxed-ACCEPTED today and a piece-level `!quantifier` on
  the relaxed catch-all would newly reject them (a relaxed-surface behavior change to adjudicate) plus the
  lookahead-blind generation interplay. The validator's quantified-verb branches (both the empty-name and
  named arms of `find_invalid_verb_construct`) stay until this leaf or capstone `.4`.
- ID: `.3.21`  Status: `pending` (LATENT accepts-invalid divergence, oracle-verified 2026-07-08 during
  `.3.14`; UNBLOCKED like `.3.16` on 2026-07-08 — `STIMULI-SIGNOFF.13.2` landed the interpreter
  value-constraint mirror)  Goal: LIMIT `=value` RANGE — PGEN accepts `(*LIMIT_HEAP=99999999999999999999)` (the `.3.14`
  grammar requires `digit+` but bounds no value) where PCRE2 10.47 REJECTS (err 160; u32-range family).
  Candidate encode = `@range` on `directive_payload_digits` (or a width-bounded digits shape if the
  boundary proves digit-count-exact — pin the exact boundary with pcre2test first: `4294967295` vs
  `4294967296`). Same class as `.3.16` (callout `@range`): parse-time value constraints must land AFTER
  the interpreter mirror or they open a latent differential-equivalence divergence.
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

- **(2026-07-08, session #65)** `.3.13`/`.3.14`/`.3.15`/`.3.16`/**`.3.17`** are DONE (see the leaves +
  enforced checklists above — the live frontier is tracked in `MEMORY.md` + `docs/TASK_TREE.md`). `.3.17`
  closed the LAST hunter-visible scs class via the `STIMULI-SIGNOFF.13.4` generation-side store gates;
  the hunter's sole residual is now the START-OPTION-POSITION class (`E(*UTF16)`, observed seed 0 —
  previously latent, validator-owned position check). 🔎 NEW leaf `.3.22` records the named-reference
  unknown-name family (`\k<zzz>`/`(?P=zzz)`/`(?&zzz)`/`\g{zzz}` PGEN-accepts / PCRE2-err-115) —
  oracle-verified accepts-invalid divergences, hunter-invisible (both PGEN sides agree). Next per the
  standing PNT order: `STIMULI-SIGNOFF.13.3` (gate lane + scaled residual enumeration), then
  `.3.18`–`.3.22` (`.3.20` needs its relaxed-semantics decision; `.3.21` note: the interpreter
  value-constraint mirror is landed, but `.3.16` PROVED the SC-08 `@range` machinery is ATOM-scoped —
  inert on native-body rules of the self-hosted grammar — so `.3.21` needs either a
  digit-width-bounded structural shape or the rule-span value-constraint extension FIRST; pin the u32
  boundary with pcre2test before designing).
- *(historical, 2026-06-08)* `.3.1` (`\u`-family), `.3.2` (`(*verb)` names), and **`.3.7`
  (cert-coverage clean — all 3 causes closed)** are DONE. After `.3.7`, regex DEFAULT cert-coverage `sample_parse_failures` is **0 at most
  seeds** (16→0/seed 0, 17→0/seed 7, 0/count 500); the only remaining residual is the seed-1 `\98495`
  numeric-backreference over-generation, owned by the spun-out **`.3.12`** (a distinct, low-frequency
  generator-fidelity gap unmasked by the spacing fix, NOT spacing). Next candidates, **pending a director
  priority decision**:
  - `.3.12` — numeric-backreference over-generation (drive seed-1 cert-cov → 0). Then the cert-coverage
    lever is fully closed; proceed to GRAMMAR-WELLFORMED Phase H (wire cert-coverage for the other 10
    grammars) toward the director's standing "all parsers cert-coverage clean" ask.
  - `.3.11` (full unrecognized-escape whitelist) and `.4`/`.5` capstone (delete `validate_regex_compile_contract`
    + expose `relaxed` via the embedding API) remain.
  - (historical) `.3.7` — **the cert-coverage lever** (was the frontier; REGEX-SELF-HOSTING done 2026-06-08).
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
| `2026-06-08` | `.3.7` (b)+(c) impl (`PGEN-LEXICAL-ANNOTATIONS-0024`) | generator-only `$text`/`@transform` atomicity (intra-rule join suppression + cross-rule cohesion); cert-cov `sample_parse_failures` 16→**0** (count 200/seed 0 ×2), seed 7 17→0, count 500/seed 0 0, seed 1 18→**1** (the distinct `\98495` numeric-backref, routed to `.3.12`); repros `(?(R1)x)`/`(?P=abc)`/`(?P>vx)` pass; lib `--lib` 618/0 (+3 locks); `stimuli_cross_family_platform_gate` ✅; `regex_pcre2_compile_oracle_gate` byte-identical; self-hosting gate OK; strict source clippy clean | **(b)+(c) DONE** — spacing category eliminated; surface-neutral (no version bump). `.3.7` CLOSED; spun out `.3.12` (numeric-backref over-generation, unmasked) |
| `2026-07-08` | `.3.16` impl (`PGEN-REGEX-PCRE2-0014`) | oracle matrix (pcre2test 10.47, value-based + leading zeros); @range-inert adjudication probes (mini-grammar generation + emitted-parser span-transform greps); post-land matrix 31/31 oracle-exact both profiles; ASTs 19/19 byte-identical; duality 5/60→0/60; cert 220/220/0 fully_certified ×3 seeds (spf=0); svpp 74/74/0 ×3; oracle gate byte-identical (1857/46/292/338); broader corpus 0 fails; equivalence gate 4/4; dual-run gate ✅; suites 880/838/760 all /0; self-hosting OK; clippy source ok; both book gates ✅ | **DONE** — structural [0,255] encode (the declared @range design REFUTED tools-first: SC-08 is atom-scoped ⇒ inert on the self-hosted native body); validator check DELETED (5th migration); surface-neutral, no version bump; stale-binary cert incident root-caused (mtime) in-slice |

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
| `.3.7` (b)+(c) spacing | `PGEN-LEXICAL-ANNOTATIONS-0024` (LEXICAL-ANNOTATIONS `.6`) | lexical-token cohesion (generator-only `$text`/`@transform` atomicity); cert-cov 16→0; spacing category eliminated; surface-neutral; spun out `.3.12` (numeric-backref over-generation, unmasked) |
| `.3.16` | `PGEN-REGEX-PCRE2-0014` | numeric-callout [0,255] STRUCTURAL encode (`callout_number` cluster + @transform-span wrapper); `find_invalid_numeric_callout` deleted (5th migration); @range design refuted tools-first (SC-08 atom-scoped); conformance- & surface-neutral (no version bump); duality 5/60→0/60 |

## Changelog

- `2026-07-08`: `.3.16` IMPLEMENTATION DONE (`PGEN-REGEX-PCRE2-0014`, session #64). The numeric-callout
  [0, 255] bound migrated into the grammar — but NOT via the declared `@range` design: the pre-implementation
  mechanism probes REFUTED it (SC-08 value constraints are ATOM-scoped in parse-guard AND generation-sampling;
  the self-hosted grammar has no spanning atom, so `@range` on a native `digits` body is inert on both sides
  — generation-probed). Encoded STRUCTURALLY instead (`callout_number` = leading zeros + optional nonzero-led
  core ≤ 255, `@transform`-span wrapper for the typed-int carrier — the Or-root span-transform exclusion was
  also probe-confirmed and designed around). `find_invalid_numeric_callout` DELETED (5th compile-contract
  migration). Conformance-NEUTRAL (oracle byte-identical 1857/46/292/338) + surface-neutral (19/19 ASTs
  byte-identical) → NO version bump. Duality class closed at the source (generated-callout parser-rejections
  5/60 → 0/60). Cert 217→220/220/0 `fully_certified` ×3 seeds. IMPORTANT downstream design fact: `.3.21`
  (LIMIT `=value` u32 range) is in the SAME class and CANNOT use `@range` as-is either — it needs a
  structural/width-bounded shape or the rule-span value-constraint extension first. In-slice incident:
  a stale debug `ast_pipeline` (predating the regen) made cert read UNKNOWN=3 (`parsed=true
  witnessed_target=false` — the witness side ran the OLD embedded parser); mtime-proven, dual rebuild fixed;
  reinforces [[feedback_verify_sv_parser_regen_mtime]].
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
