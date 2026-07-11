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
- ID: `.3.22`  Status: **`done`** (`PGEN-REGEX-PCRE2-0020`, session #70 2026-07-08 — PURE-DOCS scoping per the
  frozen plan: oracle matrix + ledger `REGEX-0098` (Deferred) + encode design; NO code change, the parse-side
  fix is deferred to capstone `.4`. Was: `pending`, 🔎 NEW FINDING 2026-07-08 session #65, oracle-verified while
  scoping `.3.17`)  Goal: the NAMED-REFERENCE UNKNOWN-NAME family — PGEN ACCEPTS `\k<zzz>` `(?P=zzz)` `(?&zzz)`
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

  **SCOPING LOG (session #70, `PGEN-REGEX-PCRE2-0020`, PURE-DOCS).** Tools-first re-verification (both oracles
  re-run this session, not trusted from the #65 note):
  - **ORACLE (`pcre2test` 10.47).** 9 named-reference spellings for an UNKNOWN name `zzz` all → **err 115**
    "reference to non-existent subpattern": `\k<zzz>` `\k'zzz'` `\k{zzz}` (named backrefs), `(?P=zzz)` (Python
    named backref), `\g{zzz}` (braced named backref), `(?&zzz)` `(?P>zzz)` `\g<zzz>` `\g'zzz'` (named subroutine
    calls). CONTROL parity ACCEPTS (both sides): `(?'aa'x)\k<aa>`, `\k<aa>(?'aa'x)` (**FORWARD ref LEGAL**),
    `(?<a>x)(?&a)`, `(?&a)(?<a>x)`, `\g{a}(?<a>x)`. (Also `\g1`@0-groups → err 115, but that is the numeric
    single-digit N<10 Non-Goal — REGEX-0083/0086, `numeric_backreference_single` ungated — NOT this named family;
    excluded from this leaf's inventory.)
  - **RELEASED PROBE (`parseability_probe` `1.1.88`).** All 10 unknown-name spellings ACCEPT (accepts-invalid vs
    err 115); all 4 named controls ACCEPT (parity). Divergence CONFIRMED in the released parser.
  - **WHY+WHERE (grammar + validator).** No named-reference inventory exists anywhere. `grammars/regex.ebnf`
    reference rules carry NO `@predicate`: `\k…`→`backreference` (`:366`/`:367`), `\g<…>`/`\g'…'`→`subroutine_named`
    (`:368`/`:370`), `\g{…}`→`named_braced` (`:372`), `(?&…)`/`(?P>…)`→`subroutine_call` (`:1135`/`:1136`),
    `(?P=…)`→`python_named_backreference` (`:285`). The validator `regex_compile_validation.rs::find_invalid_named_escape_or_group_name`
    (`:177-228`) only SHAPE-checks names (`is_pcre2_capture_name`), never inventories definitions. The name fact
    `regex_capture_name` is only `@gen_emit_fact` (`:1045`, generation-side) — no parse-side inventory. Contrast:
    `numeric_backreference` IS gated (`@predicate fact_count_at_least[regex_capture_group,$index] phase:post`, `:415`).
  - **WHY DEFERRED (two-pass necessity, proven).** Forward references are LEGAL, so at the point a reference is
    parsed the target group may be defined LATER; a left-to-right `has_fact`/`post`-predicate on the reference rule
    fires before the forward name is emitted and would reject the legal `\k<aa>(?'aa'x)` (rejects-valid regression).
    The `fact_count_at_least` numeric trick works only because it tests a MONOTONE count; "does name X exist
    anywhere" is a for-all set-inclusion across two fact-kinds, outside the current per-call predicate vocabulary.
    ⇒ inherently WHOLE-PATTERN two-pass ⇒ owned by capstone `.4` (same class as start-option POSITION).
  - **ENCODE DESIGN (frozen for `.4`).** Pass 1: parse-side `@emit_fact` a `regex_capture_name` inventory per
    group definition + a reference-name fact per use-site. Pass 2 (whole-pattern post-parse): assert every
    reference name ∈ the definition set (order-independent → forward refs pass). Requires a GENERAL
    parser-agnostic primitive — a whole-pattern post-parse verification hook OR a quantified/set-inclusion
    predicate — NOT a regex special-case. GENERATION: no over-generation break observed at 100-sample or scaled
    22k `.13.3`; if the hunter ever sees one, gate generation via the `.3.17` `regex_capture_name` facts
    (`scs_capture_name` `@gen_predicate` idiom).
  - **NO-REGRESSION (docs-only slice).** No grammar/Rust/codegen/generated/manifest change ⇒ no parser, cert,
    or gate impact; release stays `1.1.88`, contract stays `1.1.90`, schema `1`. Lockstep: ledger `REGEX-0098`
    (Deferred), integration contract "Known deferred (`.3.22`)" note, regex book (changelog-index consolidated
    deferred-divergences note + `rules-groups.md` callouts on `subroutine_call`/`python_named_backreference`),
    CHANGES / DEVELOPMENT_NOTES / LIVE_ACHIEVEMENT_STATUS / MEMORY / TASK_TREE.
- ID: `.3.18`  Status: `done` (session #67, 2026-07-08, `PGEN-REGEX-PCRE2-0016` — IMPLEMENTED per the frozen
  `-0015` scoping; regex release `1.1.84`→`1.1.85`, contract `1.1.86`→`1.1.87`, schema stays `1`, ledger
  `REGEX-0092`/`0093`/`0094`; acceptance checklist below) — **SCOPED tools-first 2026-07-08 session #66**
  (oracle matrix + released-parser
  probes run; 🔎 FOUR NEW accepts-invalid divergences CONFIRMED, surfaced to the director)  Goal: row 4 —
  counted-quantifier bounds + the brace tokenization model. THE PCRE2 10.47 MODEL (oracle-pinned, corrected
  matrix with proper pcre2test blank-line separation — an earlier unseparated run silently treated patterns as
  subjects): a syntactically-valid quantifier brace (digits + SPACES/TABS anywhere inside — `{n}` `{n,}` `{n,m}`
  `{,m}`; tab-form `a{\t2\t,\t5\t}` oracle-verified a QUANTIFIER) is ALWAYS a quantifier, then three ordered
  checks: (1) any bound VALUE > 65535 → err 105 (value-based: `a{0000000000065535}` ACCEPTS as quantifier;
  `a{065536}`/`a{ 65536 }`/`a{4294967296}` all err 105), (2) min > max → err 104 (`a{5,2}`, `a{ 5 , 2 }`,
  `(){5,2}`, and POSITION-INDEPENDENT `^{5,2}$` → 104), (3) non-repeatable position → err 109 (`^{2,5}$`,
  `x|{2,5}`, `a{2}{3}`). Non-quantifier syntax (`{}`, `{,}`, `{a}`) = literal (`^a{}$` matches "a{}").
  🔎 CONFIRMED RELEASED-PARSER DIVERGENCES (release probe vs oracle, 2026-07-08): PGEN ACCEPTS
  `a{4294967296}` (the validator's `parse::<u32>().ok()?` silently returns None on overflow —
  regex_compile_validation.rs:604-610 — so >u32 bounds skip ALL checks; PCRE2 err 105) + `{2,5}` at pattern
  start + `x|{2,5}` + `a{2}{3}` (PGEN parses literal braces; PCRE2 err 109 position class). Parity cells
  confirmed: `a{5,2}`/`a{65536}`/`a{ 5 , 2 }`/`a{,65536}`/`(){5,2}` R/R; `a{2,5}`/`a{ 3 }`/`a{0000000000065535}`
  A/A. ENCODE DESIGN (the `.3.15`-guard + `.3.16`-value idiom composed): (i) value bound STRUCTURAL — a
  `quant_bound_number` wrapper (the `callout_number` idiom scaled to ≤65535: `"0"* core?`, core = 1-4 digits
  nonzero-led | 5-digit boundary cascade) replacing `digits` in all four `counted_quantifier_body` branches;
  (ii) the literal-`{` atom gains a NEGATIVE LOOKAHEAD guard on quantifier-SYNTAX (digits/spaces/tabs shape,
  value-UNBOUNDED) so a valid-syntax-but-bad brace can neither parse as quantifier NOR fall back to literal ⇒
  grammar-REJECT — this ONE guard makes err-105 grammar-faithful AND closes the err-109 position class
  (`{2,5}`-at-start / `x|{2,5}` / `a{2}{3}` all reject: no preceding atom ⇒ no quantifier slot, literal blocked)
  AND fixes the u32-overflow hole (syntax-valid ⇒ guard blocks literal; value-encode fails quantifier);
  (iii) `ws`-in-braces parity: the guard's syntax shape AND `counted_quantifier`'s `ws` must match PCRE2's
  exact allowed set (space + tab — verify regex.ebnf `ws` covers tab, else `a{\t2\t}` would newly REJECT while
  PCRE2 accepts, a rejects-valid regression; oracle newline/formfeed cells before freezing the set);
  (iv) validator: DELETE the two 65535-limit branches + the overflow-holed parse path; KEEP the min>max order
  branch (out-of-order needs cross-number value comparison — grammar-hostile with leading zeros; stays
  validator-owned until capstone `.4` or a rule-span value-constraint extension, the `.3.21` sibling need);
  (v) generation: bounds in-range BY CONSTRUCTION via the structural rule; out-of-order {n,m} draws remain a
  rare residual (never observed at the 22k `.13.3` budget; the duality gate watches). Ledger rows for the 4
  divergences + version bump (parse surface: new rejects) + oracle-matrix pin test + cert re-baseline (new
  rules) + possible duality-gate contract re-baseline (grammar change shifts distributions — the gate will
  say) + books/contract/manifest lockstep. Full slice — needs a fresh-session context budget.

  **IMPLEMENTATION LOG (session #67, `PGEN-REGEX-PCRE2-0016`).** Landed exactly the scoped design (i)–(v):
  `quant_bound_number`/`quant_bound_number_body`/`quant_bound_core` (the `.3.16` idiom scaled to ≤65535, 9-branch
  nonzero-led cascade, `@transform` typed-int carrier preserved) replacing `digits` in all four
  `counted_quantifier_body` branches with `$N` positions unchanged (manifest annotations byte-stable);
  `literal_open_brace = !( "{" brace_ws? ( digit+ brace_ws? ( "," brace_ws? ( digit+ brace_ws? )? )? | ","
  brace_ws? digit+ brace_ws? ) "}" ) "{" -> $2` — the guard INLINE (a named guard rule would be a permanently
  never-committing cert `UNKNOWN`; the `directive_relaxed_named` inline-group precedent), `'{'` removed from
  `literal_char`; quantifier-internal whitespace `ws?`→`brace_ws?` everywhere. ORACLE ADDITIONS beyond the
  `-0015` matrix (all pcre2test 10.47): the ws set FROZEN via hex-pattern 65536-discriminator cells — space+tab
  → err 105 (quantifier ws), `\n`/`\f`/`\r`/`\v` leading AND trailing → clean compile (literal brace);
  `a{ ,65536}`/`a{, 65536}` → err 105 (ws-comma forms ARE quantifier syntax → the guard's `{,m}` form carries
  ws slots); `a{ }`/`a{ ,}`/`a{\t}` compile clean (ws-only braces are literals — the guard requires digits).
  🔎 TWO MORE in-class divergences confirmed during implementation (both fixed same-slice, folded into ledger
  `REGEX-0094`): (5th) `a{\t5\t,\t2\t}` PGEN-ACCEPT vs PCRE2 err 104 — the validator's recognizer byte-set was
  space-only (`regex_compile_validation.rs:594` pre-fix), so tab-spaced order violations were never checked;
  (6th, REJECTS-VALID) `a{\n5,2\n}` PGEN-REJECT ("minimum cannot exceed") vs PCRE2 clean — the validator's
  whole-body `str::trim()` stripped ALL whitespace, order-checking a brace PCRE2 treats as literal. Validator
  narrowed to ORDER-only per (iv): both 65535-limit branches + the `parse::<u32>().ok()?` overflow path
  DELETED; recognizer gained `b'\t'`; `trim_matches([' ', '\t'])` per part; bounds parse as saturating u64.
  DUALITY-GATE RE-BASELINE (the scoped "the gate will say"): the same TWO adjudicated classes only
  (quantified-verb → `.3.20`, start-option-position → `.4`) shifted lanes with the new RNG stream positions
  (canonical seed0 now quantified-verb, seed42 now start-option; scaled seed7 emptied, seed42 gained
  start-option) — NO novel class at 6,300 gate samples; the literal-brace generation-adjacency concern did NOT
  materialize; contract re-baselined same-commit, owners unchanged, gate GREEN ×2. ADJACENT DEFECT fixed
  same-slice (surfaced by the oracle-gate ratchet): the corpus normalizer ingested pcre2test `expand`-modifier
  lines as raw patterns (`/\[AB]{6000000000000000000000}/expand` → expectation "ok" while the raw text is a
  plain err-105 reject — pcre2test macro-expands `\[...]{n}` BEFORE compiling, so raw-text↔verdict pairing is
  unsound by construction); `"expand"` added to `UNSUPPORTED_SUFFIX_TOKENS`, 6 cells skipped (2195→2189),
  canonical corpus artifacts regenerated, baseline env → v11 with the false-reject ratchet EXACT at 46.

  ## Acceptance Checklist (enforced)
  - [x] **REPRODUCE / ISSUE** — release probe (pre-change, `1.1.84` binary): `a{4294967296}` / `{2,5}` /
    `x|{2,5}` / `a{2}{3}` / `a{\t5\t,\t2\t}` all ACCEPT and `a{\n5,2\n}` REJECTS, vs pcre2test 10.47
    err 105 / 109 / 109 / 109 / 104 / clean-compile respectively (parity cells `a{5,2}`/`a{65536}` REJECT,
    `a{2,5}`/`a{}`/`a{0000000000065535}` ACCEPT — matrix in this leaf + the `-0015` scoping).
  - [x] **ROOT CAUSE (WHY + WHERE)** — (a) value hole: `validate_counted_quantifier_body::parse_bound` used
    `raw.parse::<u32>().ok()` and the single-bound arm consumed it with `?` (`regex_compile_validation.rs:604-621`
    pre-fix) ⇒ >u32 bounds returned `None` from the whole check; (b) position class: `literal_char` carried an
    UNGUARDED `'{'` (`regex.ebnf:211` pre-fix) ⇒ any brace failing `counted_quantifier` fell back to literal
    pieces; (c) ws set: grammar used `ws?` (6-char set) while the validator recognizer was space-only + all-ws
    `trim()` — each the opposite of PCRE2's oracle-frozen space+tab set (hex-cell matrix in this leaf).
  - [x] **FIX** — grammar tier (fix hierarchy: no engine change; 6th validator→grammar migration): the
    value-STRUCTURAL `quant_bound_number` + the inline `literal_open_brace` guard + `brace_ws` everywhere;
    validator narrowed to tab-aware order-only; corpus normalizer `expand` skip (adjacent defect, corpus tier).
  - [x] **ADDRESSED (verified)** — release probe (rebuilt `1.1.85`): all six divergence cells flipped to the
    oracle verdict (ACCEPT→REJECT ×5, REJECT→ACCEPT ×1); full 47-cell oracle matrix green on the release
    binary (12 value-reject + 5 order-reject + 6 position-reject + 16 quantifier-accept + 13 literal-accept
    spot-checks minus overlaps); literal-`{` AST byte-identical (`a{}` JSON cmp); typed min/max ints preserved
    (`a{ 2 , 5 }` → `{min:2,max:5}`; `a{065535}` → 65535); new pin
    `parser_registry::tests::regex_counted_quantifier_brace_model_rejects_at_the_grammar_layer_pcre2_faithfully`.
  - [x] **NO REGRESSION** — regex cert `total=228 witness=228 UNKNOWN=0 fully_certified=true spf=0` at seeds
    0/7/42 (224→228 = exactly the 4 new rules, all witnessed); `--lint-grammar` 0 errors (228 rules); lib
    suites **889/847/768** all 0-fail (dual/`generated_parsers`/no-features = 886/844/766 + the 2 validator
    pins + the 1 registry pin, cfg-gated); `parse_harness_equivalence_gate` ✅ (regex stays
    differential-CERTIFIED over the new grammar — `certified_grammars_are_byte_identical` green); combinator +
    semantic suites ✅; `duality_hunt_gate` ✅ post-re-baseline (same 2 adjudicated classes, owners unchanged,
    determinism tripwire green); `regex_pcre2_compile_oracle_gate` ✅ v11 (false_accept 292→286, false_reject
    46 EXACT); `regex_corpus_bundle_contract_gate` + `regex_pcre2_textsafe_corpus_gate` ✅; contract-JSON
    literal-brace/quantifier samples green (suite); clippy strict-source ok (generated debt unchanged at the
    known 178-eq_op).
  - [x] **LOCKSTEP** — ledger rows `REGEX-0092`/`0093`/`0094` + version consts (`embedding_api.rs`) + tracked
    contract JSON (`1.1.85`/`1.1.87`); contract MD Identity + "Release 1.1.85 / Contract 1.1.87 Highlights";
    regex book `rules-quantifier.md` (counted_quantifier/body rewritten to current truth + § `quant_bound_number`
    + the stale `digits`-appears-in list fixed) + `rules-atom.md` (literal split + § `literal_open_brace` +
    the stale pre-self-hosting `literal_char` body fixed) + `examples-quantifiers.md` (brace-model section +
    ws-set note) + `json-carrier.md` (+2 rows) + `changelog-index.md` (1.1.85 entry) + tracked HTML rebuilt
    (`regex_parser_book_gate` ✅); top book `parser-families.md` handoff line → `1.1.85`/`1.1.87` + the
    REGEX-0092/0093/0094 clause (`mdbook_docs_gate` ✅); AST-shape manifest `regex_v1.json` inventory 207→208
    (the `literal_open_brace` return_scalar entry at the alphabetical slot); duality contract re-baselined +
    `STIMULI-SIGNOFF.13.3` gate-lane note; corpus bundle: normalizer + regenerated canonical artifacts +
    baseline env v11; `MEMORY.md`/`CHANGES.md`/`DEVELOPMENT_NOTES.md`/`LIVE_ACHIEVEMENT_STATUS.md`/`docs/TASK_TREE.md`.
- ID: `.3.19`  Status: **`done`** (`PGEN-REGEX-PCRE2-0017`, session #68 2026-07-08; regex release
  `1.1.85`→**`1.1.86`**, contract `1.1.87`→**`1.1.88`**, AST-dump schema stays `1`, ledger **`REGEX-0095`**;
  STRAY-`\E` scope — empty-`\Q\E`-quantified spun out to `.3.23`. Full evidence in the Acceptance Checklist
  below) — LATENT accepts-invalid divergences, oracle-verified 2026-07-07
  during the `.3.13` implementation)  Goal: the ZERO-WIDTH-QUANTIFIED family beyond anchors — PGEN ACCEPTS
  `\E*` (stray `\E` is zero-width) and `\Q\E*` (empty quoted literal) which PCRE2 10.47 REJECTS (err 109);
  `\Qa\E*` correctly accepted by both. Both sides of PGEN (generator AND parser) agree today, so the
  duality hunter can NEVER surface these — only oracle-differential coverage can. Distinct from `.3.13`'s
  anchor-rule scope; spun out to keep that slice bounded.

  **🔎 IN-SLICE DESIGN ADJUDICATION (tools-first, session #68): the leaf's declared candidate encode —
  "the stray-`\E`/empty-`\Q\E` atoms joining the anchor treatment (non-quantifiable `!quantifier` piece
  forms)" — is REFUTED by the `pcre2test` 10.47 oracle.** Anchors and zero-widths behave DIFFERENTLY under
  a quantifier (44-cell oracle map, `scratchpad/zw_full_map.py`):
  - Anchors are OPAQUE non-repeatable items: `a^*` `a\A*` `a\b*` all REJECT (err 109) — the `*` binds to
    the anchor even with a repeatable `a` before it.
  - Stray `\E` / empty `\Q\E` are TRANSPARENT (elided): `a\E*` `\Ea*` `a\E\E*` `()\E*` `\Qa\E\E*` all
    ACCEPT — the `*` binds THROUGH the zero-width to the preceding repeatable atom.
  So the anchor `!quantifier` encode would NEWLY REJECT `a\E*` (a rejects-valid regression). The faithful
  rule is CONTEXTUAL: a quantifier on a stray-`\E`/empty-`\Q\E` is legal iff, skipping backward over
  transparent zero-widths ONLY (not anchors), the immediately-preceding item is a repeatable atom. The
  full divergence set (11 in the 44-cell map, all accepts-invalid): `\E*` `\Q\E*` `\E\E*` `\Q\E\Q\E*`
  (no repeatable predecessor); `^\E*` `\A\E*` `a^\E*` (predecessor after elision is an ANCHOR, non-repeatable);
  `(\E*)` `|\E*` `a|\E*` `(a|\E*)` (group/alternation edge resets the predecessor). Mirrors `.3.16`'s
  declared-design refutation (`@range` refuted atom-scoped).

  **🔎 SECOND IN-SLICE ADJUDICATION (interpreter-validated, session #68): the EMPTY `\Q\E`-quantified
  sub-family is SPUN OUT to a new leaf `.3.23`.** The first design attempt made empty `\Q\E`
  non-quantifiable too (`quoted_literal_char*`→`+` + a dedicated `empty_quoted_literal` in `zero_width`).
  The regex-CERTIFIED interpreter (`parse_harness_interpreter::interpret_parse` on the edited `.ebnf`,
  byte-identical to the generated parser) caught 8 residual accepts-invalid: `\Q\E*` `\Q\E+` `\Q\E?`
  `\Q\E{2}` `\Q\E{2,}` `\Q\E{2,3}` `\Q\E{,2}` `\Q\E\Q\E*` still ACCEPTED. Root cause (tools-first): `\Q` is
  ITSELF a valid `simple_escape` (char `Q`) — because PGEN models unterminated `\Q...` (PCRE2 quote-to-end:
  `\Qabc` `\Qa*b` ACCEPT in both, oracle-verified) via the `\Q`-as-escape path — so once empty `\Q\E` is
  split off `quoted_literal`, the longest_match tournament decomposes `\Q\E*` as `\Q`(escape) + `\E`(zero-
  width) + `*` (the ABSORPTION branch). Blocking that needs either a NEGATIVE lookahead on `simple_escape`
  (generation-BLIND → reintroduces the `.3.13` duality-break class) or removing `'Q'` from `simple_escape`
  (a rejects-valid regression on unterminated `\Qabc`, which the oracle corpus exercises — 41 `\Q` lines).
  Both entangle with the pre-existing `\Q`-as-`simple_escape` / unterminated-`\Q...\E` quoting model — a
  distinct, larger fidelity concern. So `.3.19` is scoped to the STRAY-`\E` family (the `.3.14` spin-out
  discipline); `.3.23` owns empty-`\Q\E`-quantified with its own tools-first `\Q`-model treatment.

  **FINAL ENCODE (Model A — transparent elision within a piece; STRAY-`\E` scope; interpreter-validated
  against all 60 curated cells + 27 AST byte-diffs):**
  (1) Re-home stray `\E` out of the quantifiable `atom` (the `.3.13` anchor-split precedent, a POSITIVE
  exclusion — generation-faithful): drop `'E'` from `simple_escape_letter_strict` (so a bare `\E` is no
  longer a general shorthand escape) and add `stray_end_quote_escape = "\\E" -> {type:"escape",
  kind:"shorthand", char:"E"}` preserving the exact pre-`.3.19` AST byte-shape. (2)
  `zero_width = stray_end_quote_escape` (empty `\Q\E` is NOT a member — deferred to `.3.23`). (3)
  Restructure `piece` (longest_match tournament), branch order:
  `piece_quoted_run_quantified | anchor !quantifier | zero_width !quantifier | atom quantifier? |
  atom zero_width+ quantifier`. The STANDALONE branch `zero_width !quantifier` (a stray `\E` never carries
  its own quantifier). The ABSORPTION branch `atom zero_width+ quantifier -> {type:"piece", atom:$1,
  quantifier:$3}` (a quantifier reaches through ≥1 transparent stray `\E` to bind the preceding atom; the
  `\E`s ELIDED, the PCRE2 delete-`\E` model; the `+` confines it to `<atom><\E>+<quant>` so plain `x*` is
  untouched) is placed LAST so its `\Q\E*` tie (5 bytes both ways) loses to the earlier `atom quantifier?`
  — leaving empty-`\Q\E`-quantified BYTE-IDENTICAL as the deferred divergence. `quoted_literal` is
  UNCHANGED (`*`). (4) NO validator migration — there is NO pattern-level zero-width-quantified check in
  `regex_compile_validation.rs` (the `ZeroWidth` references are all in `scan_char_class`, the `.3.15` class
  surface); this is a purely latent divergence, grammar-only. Both profiles tighten (the `.3.13` precedent
  — quantifier-target validity is not a relaxed-escape concern). AST semantic-correction (documented,
  `.3.13`/`.3.15` precedent, 8/27 curated patterns): `<atom><\E>+<quant>` patterns (`a\E*`, `ab\E*` binds
  `b`, `a\E\E*`, `\Qa\E\E*`, `()\E*`, `(a)\E*`, `(?:)\E*`, `(a|b)\E*`) change from `[piece(atom),
  piece(\E, quant)]` to `[piece(atom, quant)]` with the stray `\E`(s) elided — the quantifier now binds to
  the repeatable atom (PCRE2-faithful) instead of the zero-width; the other 19 curated patterns are
  byte-identical (incl. the deferred `\Q\E*`/`\Q\E{2}`).

  ### REGEX-PCRE2-FIDELITY.3.19 — Acceptance Checklist (enforced)
  - [x] **REPRODUCE / ISSUE** — `printf '%s' '\E*' | parseability_probe` → ACCEPT; `pcre2test` 10.47
    rejects err 109 ("quantifier does not follow a repeatable item"). A 44-cell oracle interaction map
    (scratchpad `zw_full_map.py`) pinned **11 accepts-invalid** stray-`\E`-quantified divergences: bare
    `\E*` `\E{2}` `\E\E*`; anchor-blocked `^\E*` `\A\E*` `a^\E*`; group/alternation-edge `(\E*)` `|\E*`
    `a|\E*` `(a|\E*)` — with the discriminating parity cells `a^*` REJECT (anchor opaque) vs `a\E*` ACCEPT
    (stray `\E` transparent), `()\E*`/`\Ea*` ACCEPT.
  - [x] **ROOT CAUSE (WHY + WHERE)** — `piece = … | atom quantifier?` (regex.ebnf) bound a quantifier to a
    stray `\E` because `\E` matched as a general `simple_escape` shorthand (char `"E"`, via
    `simple_escape_letter_strict`), making it a quantifiable `atom`. NO validator check exists — the
    `ZeroWidth` references in `regex_compile_validation.rs` are all class-scoped (`scan_char_class`, the
    `.3.15` surface), so this is a purely LATENT, hunter-invisible divergence (both PGEN sides agree). The
    leaf's original `anchor !quantifier` hypothesis was **REFUTED tools-first** by the oracle (anchors are
    OPAQUE — `a^*` rejects — but stray `\E` is TRANSPARENT — `a\E*` accepts; the anchor encode would have
    newly rejected `a\E*`, a rejects-valid regression). In-slice adjudication recorded above.
  - [x] **FIX (tier: GRAMMAR — no engine, no validator; never validator-owned)** — dropped `'E'` from
    `simple_escape_letter_strict` (positive exclusion, generation-faithful, `.3.13` precedent); added
    `stray_end_quote_escape` + `zero_width`; restructured `piece` with a STANDALONE (`zero_width
    !quantifier`) and a LAST-placed ABSORPTION (`atom zero_width+ quantifier`) branch that reaches a
    quantifier THROUGH the transparent `\E`(s), eliding them (PCRE2 delete-`\E` model). Empty-`\Q\E`-quantified
    spun out to `.3.23` (the absorption branch's `\Q\E*` tie loses to `atom quantifier?`, leaving it
    byte-identical). `quoted_literal` UNCHANGED.
  - [x] **ADDRESSED (verified)** — design validated ENTIRELY before the rebuild via the regex-CERTIFIED
    interpreter (all 60 curated cells + 27 AST byte-diffs); the **rebuilt release probe** then confirmed the
    44-cell map shows only the 2 empty-`\Q\E` deferred cells diverging (probe == interpreter == `pcre2test`).
    8 `<atom>\E<quant>` AST semantic-corrections (quantifier→atom, `\E` elided); 19/27 byte-identical. New
    pin `regex_stray_end_quote_quantifier_binds_through_pcre2_faithfully` green (10 rejects + 9 accepts +
    both-profile tightening + `.3.23`-deferred `\Q\E*` still-accepts + relaxed `\u` guard).
  - [x] **NO REGRESSION** — regex cert `total=230 witness=230 UNKNOWN=0 fully_certified=true
    sample_parse_failures=0` at seeds 0/7/42 (228→230 = the 2 new rules `zero_width`/`stray_end_quote_escape`,
    both witnessed — NO witness gap); `--lint-grammar` clean (230 rules); `parse_harness_equivalence_gate` ✅
    (regex stays differential-CERTIFIED — `certified_grammars_are_byte_identical`); `ast_shape_contract`
    regex ✅ (manifest inventory 208→211, re-derived from the regenerated `regex_return_annotations.json`);
    `regex_pcre2_compile_oracle_gate` ✅ **byte-identical baseline** (conformance-neutral — the stray-`\E`
    forms aren't in the 2189-cell corpus, exactly the `.3.13` anchor precedent); dual-feature lib suite
    **890 passed / 0 failed** (baseline 889 + the 1 new pin; the pin is `has_generated_regex_parser`-gated so
    no-features stays 768); `regex_parser_book_gate` + `mdbook_docs_gate` ✅ (tracked HTML regenerated);
    clippy source clean.
  - [x] **LOCKSTEP** — ledger **`REGEX-0095`** + `embedding_api.rs` version consts (release **1.1.86** /
    contract **1.1.88**, schema `1`) + tracked contract JSON + contract MD (Identity + "Release 1.1.86 /
    Contract 1.1.88 Highlights — REGEX-0095"); regex book `rules-piece` (§ `piece` 5 branches + § `zero_width`),
    `examples-anchors` ("Stray `\E` transparent quantifier binding"), `rules-escape` (38-letter strict set),
    `changelog-index`, tracked `regex_parser_book-html` regenerated; top book `parser-families` (version chain)
    + `stimuli-and-quality` (residual-class note), tracked `book-html` regenerated; ast_shape manifest 208→211;
    new leaf `.3.23` + live docs (CHANGES / DEVELOPMENT_NOTES / LIVE_ACHIEVEMENT_STATUS / MEMORY / TASK_TREE).

- ID: `.3.23`  Status: **`done`** (`PGEN-REGEX-PCRE2-0021`, session #71 2026-07-08; regex release
  `1.1.88`→**`1.1.89`**, contract `1.1.90`→**`1.1.91`**, AST-dump schema stays `1`, ledger
  **`REGEX-0099`**; spun out of `.3.19`, session #68) — the PCRE2 `\Q` QUOTING model made first-class,
  closing TWO sibling divergence classes with ONE root cause (`\Q`-as-`simple_escape`).
  Goal (as landed): (a) EMPTY `\Q\E`-QUANTIFIED accepts-invalid — `\Q\E*` `\Q\E+` `\Q\E?` `\Q\E{2}`
  `\Q\E{2,}` `\Q\E{2,3}` `\Q\E{,2}` `\Q\E\Q\E*` now REJECT (err 109 — empty `\Q\E` is zero-width); AND
  (b) 🔎 NEW FINDING (this session): UNTERMINATED `\Q…` metachar-tail REJECTS-VALID — `\Q)` `\Q(` `\Q[`
  `\Q(?:` `\Q**` `\Qa**` `\Qa)b` now ACCEPT (PCRE2 quotes `\Q…` to end-of-pattern as literal). Both flow
  from `\Q` being a bare `simple_escape` (char `Q`); the two are INSEPARABLE (see ROOT CAUSE), so one
  grammar restructure closes both.

  **🔎 IN-SLICE FINDING (surfaced to the director):** the `\Q`-as-`simple_escape` hack was not only the
  accepts-invalid empty-quantified blocker `.3.23` was scoped for — it ALSO carried a whole REJECTS-VALID
  class (unterminated `\Q…` with a structural metachar tail: `\Q^` mis-parsed `^` as an anchor, `\Q|` as
  alternation, `\Q)` as an unbalanced group). The leaf's own candidate design ("model `\Q…\E` AND
  unterminated `\Q…` as first-class quoting") already sanctioned the fix that closes both, so this slice
  delivers a strictly bigger fidelity win than the empty-quantified target alone.

  **🔎 SECOND IN-SLICE ROOT-CAUSE (tools-first, cert seed 42 spf=1):** the first encode made
  `unterminated_quoted_literal` a quantifiable `atom`. The cert diverse pass then flagged a
  generator-produced sample the (correct) parser rejects — `…\Q\E{0 ,0 }…` (dumped by the
  `SAMPLE-PARSE FAILURES` block). Root cause: the lookahead-blind generator used a BARE `\Q` (unterminated
  atom) as the absorption branch's atom (`atom zero_width+ quantifier`) + a stray `\E` + a quantifier; the
  parser re-fuses `\Q\E` into `empty_quoted_literal` (longest-match) and orphans the quantifier → correct
  REJECT, but a generator↔parser DUALITY break (the `.3.13`/`.3.19` lookahead-blindness anti-pattern, which
  a `!"\\E"` guard would have re-introduced). FIX: make `unterminated_quoted_literal` a NON-atom STANDALONE
  `piece` — it can then never be the absorption atom, and needs no lookahead (generation-faithful by
  construction). Re-verified spf=0 across seeds 0/1/2/3/5/13/42/100/314/500.

  **FINAL ENCODE (grammar tier, no engine change; the host validator already skips `\Q…` as
  quote-to-end):** (1) `'Q'` dropped from `simple_escape_letter_strict` (positive exclusion). (2)
  `quoted_literal` narrowed to nonempty terminated (`"\\Q" quoted_literal_char+ "\\E"`). (3) NEW
  `unterminated_quoted_literal = "\\Q" quoted_literal_char*` consumed by a NEW standalone `piece` branch
  (`-> {type:"piece", atom:$1, quantifier:[]}`, a NON-atom quote-to-end). (4) NEW `empty_quoted_literal =
  "\\Q" "\\E"` added to the non-quantifiable `zero_width` (joins stray `\E`). So `\Q\E*` rejects via the
  standalone `!quantifier` branch, `a\Q\E*` = `a*` via ABSORPTION, unterminated `\Q…` accepts as one
  literal run, and terminated `\Q…\E` / bare `\Q\E` / `\Qa\E*` stay byte-identical.

  ### REGEX-PCRE2-FIDELITY.3.23 — Acceptance Checklist (enforced)
  - [x] **REPRODUCE / ISSUE** — release probe `1.1.88`: `printf '\Q)' | parseability_probe --parse regex`
    → REJECT where `pcre2test` 10.47 ACCEPTs `/\Q)/`; `printf '\Q\E*'` → ACCEPT where `pcre2test` rejects
    err 109. An 85-cell `pcre2test` 10.47 oracle matrix pinned this session (14 REJECT / 71 ACCEPT).
  - [x] **ROOT CAUSE (WHY + WHERE)** — `'Q'` ∈ `simple_escape_letter_strict` (`grammars/regex.ebnf:882`
    pre-fix) ⇒ `\Q` = `{escape, shorthand, char:"Q"}`, so unterminated `\Q…` parsed as escape-`Q` + live
    regex (metachar tail breaks: `\Q)` unbalanced-group REJECT; `\Q^` anchor mis-parse); and empty `\Q\E`
    was a quantifiable `quoted_literal` atom (`char*` admits empty body), so `atom quantifier?` bound the
    `*`. ENTANGLED: fixing empty-quantified needs `Q` out of `simple_escape` (else longest-match
    re-decomposes `\Q\E*` = `\Q`(escape) + `\E`(zero_width) + `*` via `.3.19` absorption), which needs a
    first-class unterminated model to avoid regressing `\Qabc`. Second-order: cert seed-42 `SAMPLE-PARSE
    FAILURES` dump (`\Q\E{0,0}`) named the generator over-generation via the absorption branch.
  - [x] **FIX** — GRAMMAR tier (no engine, no validator), per FINAL ENCODE. `unterminated_quoted_literal`
    is a NON-atom `piece` (the generation-faithful duality-break fix, no lookahead). `lint` 0 errors /
    0 undefined-refs (236 rules).
  - [x] **ADDRESSED (verified)** — regex-CERTIFIED interpreter (`parse_harness_interpreter::interpret_parse`)
    == `pcre2test` on all 85 cells (**0 divergences**), with exactly 23 intended AST byte-changes
    (unterminated `\Q…` → one `{atom,quoted_literal,body}`; `a\Q\E*` → `a*`) and every must-stay-identical
    case (terminated `\Q…\E`, bare `\Q\E`, `\Qa\E*`, normal regex, `\E`/`\e`-families) UNCHANGED. Behavior
    flips: 7 unterminated-metachar REJECT→ACCEPT + 8 empty-quantified ACCEPT→REJECT, both profiles. New pin
    `parser_registry::tests::regex_quoted_literal_model_pcre2_faithfully` green; the `.3.19` pin's deferred
    `\Q\E*`→ACCEPT assertion flipped to REJECT.
  - [x] **NO REGRESSION** — regex cert `total=236 witness=236 UNKNOWN=0 fully_certified=true spf=0` at
    seeds 0/7/42 (234→236 = the 2 net-new rules, all witnessed, no gap; spf=0 also across seeds
    0/1/2/3/5/13/42/100/314/500 after the duality-break fix); `--lint-grammar` 0 errors (236 rules);
    `regex_pcre2_compile_oracle_gate` byte-identical baseline `2189/1858/285/46` (the `\Q`-model forms are
    not in the pcre2test corpus — the `.3.19`/`.3.21` precedent); `parse_harness_equivalence_gate` ✅ regex
    stays differential-CERTIFIED; `duality_hunt_gate` ✅ 9 lanes, NO new signature; `ast_shape_contract`
    regex ✅ inventory 214→217 (the 2 new rules + the new `piece` branch); dual lib suite **893/0**;
    version-consts-match-ledger ✅.
  - [x] **LOCKSTEP** — ledger `REGEX-0099` + embedding version consts (release **1.1.89** / contract
    **1.1.91**, schema `1`) + tracked contract JSON (`regex_parser_integration_contract_v1.json`) +
    contract MD ("Release 1.1.89 / Contract 1.1.91 Highlights — REGEX-0099" + Identity) + regex book
    (changelog + piece/atom/escape/quoted-literal chapters + regenerated HTML) + top book
    (`parser-families` version chain) + ast_shape manifest inventory 214→217; trees + `docs/TASK_TREE.md` +
    live docs (CHANGES / DEVELOPMENT_NOTES / LIVE_ACHIEVEMENT_STATUS / MEMORY / TASK_TREE).
- ID: `.3.20`  Status: **`done`** (`PGEN-REGEX-PCRE2-0018`, session #69 2026-07-08; regex release
  `1.1.86`→**`1.1.87`**, contract `1.1.88`→**`1.1.89`**, AST-dump schema stays `1`, ledger
  **`REGEX-0096`**; spun out of `.3.14`'s in-slice adjudication; oracle matrix re-pinned this session;
  the post-`.3.14` hunter had OBSERVED it at seed 42: `DUALITY-BREAK: signature="only ACCEPT verb may
  be quantified…" shrunk_reproducer="(*F)+"`)
  Goal: QUANTIFIED-VERB encoding — only `(*ACCEPT)` may take a quantifier (oracle
  `pcre2test` 10.47: `(*ACCEPT)+` AND `(*ACCEPT:x)+` ACCEPT; `(*:x)+` `(*PRUNE)+` `(*FAIL)*` `(*MARK:x)+`
  all err 109). Candidate mechanism = the `.3.13` piece-level split (non-ACCEPT `directive_verb` forms get
  a `!quantifier` piece branch; the ACCEPT-named form stays quantifiable in `atom`). Adjudicated OUT of
  `.3.14` to keep that slice bounded to arg shapes AND because it needs its own relaxed-semantics decision.
  GATE-PINNED (`STIMULI-SIGNOFF.13.3`, 2026-07-08): this class's signature (`only ACCEPT verb may be
  quantified…`) is pinned with THIS leaf as owner in
  `rust/test_data/grammar_quality/duality_hunt_gate_contract_v0.json` (regex canonical seed0 + scaled
  seed0/seed42) — the closing slice MUST re-baseline that contract same-commit (the vanished-signature
  discipline; `make -C rust duality_hunt_gate` fails until it does).

  **🔎 IN-SLICE OScope (tools-first, session #69): the oracle matrix widens the class beyond the leaf's
  stated `(*PRUNE)+`/`(*:x)+`/`(*MARK:x)+` set — EVERY `(*...)` directive except `(*ACCEPT)` rejects a
  following quantifier (err 109).** `pcre2test` 10.47 (this session): `(*ACCEPT)+` `(*ACCEPT:x)+`
  `(*ACCEPT)*` `(*ACCEPT)?` `(*ACCEPT){2,3}` ACCEPT; `(*:x)+` `(*PRUNE)+` `(*FAIL)*` `(*F)+` `(*SKIP)+`
  `(*COMMIT)+` `(*THEN)+` `(*MARK:x)+` **`(*UTF)+` `(*LIMIT_HEAP=5)+`** all err 109. The released
  parser (`1.1.86`) matches the oracle on verbs/shorthand/MARK (validator-rejected) but **ACCEPTS
  `(*UTF)+` / `(*LIMIT_HEAP=5)+`** — a NEW latent accepts-invalid: `find_invalid_verb_construct`'s
  start-option arm (`regex_compile_validation.rs:420-438`) checks POSITION but never a trailing
  quantifier, so a prefix-position start-option passes and the grammar's `atom quantifier?` binds the
  `+`. So `.3.20` both MIGRATES the existing verb/shorthand/MARK quantifier checks to the grammar AND
  CLOSES the start-option-quantified hole (ledger `REGEX-0096`).

  **🔎 RELAXED-SEMANTICS DECISION (within the ratified `project_regex_pcre2_faithful_by_default`
  principle — relaxed = a strict SUPERSET opt-out that only GAINS acceptance, never newly rejects):
  preserve relaxed acceptance of quantified UNKNOWN-name verbs `(*foo)+`.** Encode: the two quantifiable
  directive forms (`(*ACCEPT...)` and the `@profiles:["relaxed"]` catch-all `directive_relaxed_named`)
  stay in `atom` (quantifiable); every KNOWN non-ACCEPT directive moves to a non-quantifiable piece
  branch. So in DEFAULT (pcre2) `(*foo)+` still rejects (catch-all disabled ⇒ no directive matches ⇒
  parse fails); in RELAXED `(*foo)+` still ACCEPTS (catch-all quantifiable) while a KNOWN `(*PRUNE)+`
  rejects in BOTH profiles (the established relaxed-parity design — known names keep their exact PCRE2
  shape, unknown names are permissively accepted). No relaxed regression.

  **FINAL ENCODE (Model — piece-level split, the `.3.13`/`.3.19` idiom; validated pre-rebuild via the
  regex-CERTIFIED interpreter):**
  (1) Split `directive_verb` (the single quantifiable-`atom` rule) into two atom-producing rules with
  the SAME `{type:"atom", kind:"directive_verb", body:$2}` carrier: `directive_verb` (KEPT in `atom`,
  quantifiable) whose body is `directive_body_quantifiable = directive_accept_named | directive_relaxed_named`,
  and NEW `directive_verb_nonquant` whose body is `directive_body_nonquantifiable = directive_mark_named |
  directive_verb_named | directive_limit_named | directive_option_named | directive_mark_shorthand`.
  (2) `directive_accept_named = "ACCEPT" directive_payload_colon?` (the ACCEPT verb, byte-identical
  `{kind:"named",name,payload}` carrier); `directive_verb_named` re-points to `directive_verb_name_nonaccept
  = "FAIL" | "F" | "COMMIT" | "PRUNE" | "SKIP" | "THEN"` (ACCEPT removed). The relaxed guard swaps
  `directive_verb_name` → inline `"ACCEPT"` + `directive_verb_name_nonaccept` so it still excludes ALL 7
  recognized verb names at a name boundary (keeping the strict shapes authoritative in both profiles and
  avoiding an `(*ACCEPT)`-in-relaxed accept_named⟷catch-all tie). REMOVE the now-dead `directive_body`,
  `directive_named`, `directive_verb_name` (the H.10.1 proven-dead precedent). (3) `piece` gains a
  STANDALONE branch `directive_verb_nonquant !quantifier -> {type:"piece", atom:$1, quantifier:[]}` placed
  with the other non-quantifiable branches (after `zero_width !quantifier`, before `atom quantifier?`).
  Since `atom`'s directive coverage is now ACCEPT+catch-all only, a quantified non-ACCEPT directive:
  `directive_verb_nonquant !quantifier` matches the directive then the `!quantifier` lookahead FAILS on
  the trailing quantifier ⇒ backtrack, and `atom quantifier?` finds no matching atom ⇒ whole parse REJECTS
  (err-109-faithful). (4) Validator: `find_invalid_verb_construct` loses both quantified-verb arms (the
  empty-name shorthand check + the named-verb check) — it reduces to the start-option POSITION rule only
  (capstone `.4` scope); dead helper `quantifier_starts_at` removed (`is_pcre2_verb_name`/`find_star_verb_end`
  stay — still used by `star_directive_group_end_at`). AST byte-identical for every accepted directive
  (all carriers unchanged); the only behavior change is quantified non-ACCEPT directives flip ACCEPT→REJECT
  (verbs/shorthand/MARK move validator→grammar, no verdict change; start-options newly reject —
  `REGEX-0096`). Both profiles tighten identically for KNOWN names; relaxed `(*foo)+` preserved.

  ### REGEX-PCRE2-FIDELITY.3.20 — Acceptance Checklist (enforced)
  - [x] **REPRODUCE / ISSUE** — release probe `1.1.86`: `printf '(*UTF)+' | parseability_probe --parse
    regex` → ACCEPT and `printf '(*LIMIT_HEAP=5)+'` → ACCEPT, where `pcre2test` 10.47 rejects both err
    109; verbs/shorthand/MARK quantified already rejected. A 30-cell oracle matrix pinned this session
    (release-probe vs `pcre2test`): only `(*ACCEPT)`-family quantified ACCEPTs; all other directives
    quantified REJECT.
  - [x] **ROOT CAUSE (WHY + WHERE)** — quantified-verb checks were OUT-OF-BAND in
    `regex_compile_validation.rs::find_invalid_verb_construct` (empty-name arm `:403-418` + named-verb arm
    `:440-456`), and the start-option arm (`:420-438`) never checked a trailing quantifier at all ⇒
    `(*UTF)+`/`(*LIMIT_HEAP=5)+` latently accepted. `directive_verb` was a single quantifiable `atom`
    (`grammars/regex.ebnf:270,1378`), so the grammar imposed no per-name quantifiability rule.
  - [x] **FIX (tier: GRAMMAR — no engine; validator loses two arms)** — piece-level split per FINAL
    ENCODE: `directive_verb` (quantifiable, ACCEPT + relaxed catch-all) stays in `atom`; NEW
    `directive_verb_nonquant` (the rest) is a non-quantifiable `piece` branch (`directive_verb_nonquant
    !quantifier`). Validator's two quantified-verb arms + dead `quantifier_starts_at` DELETED. `lint` 0
    errors / 0 undefined-refs.
  - [x] **ADDRESSED (verified)** — 30-cell release-probe verdict matrix == `pcre2test` 10.47 (**0
    mismatches**): `(*UTF)+`/`(*LIMIT_HEAP=5)+` flip ACCEPT→REJECT, `(*ACCEPT)+`/`(*ACCEPT:x)+`/
    `(*ACCEPT){2,3}` accept, all bare directives accept. Relaxed matrix (`--profile relaxed`) confirms the
    RELAXED-SEMANTICS decision: `(*foo)+`/`(*bar)*`/`(*baz){2}` relaxed-ACCEPT / default-REJECT; KNOWN
    `(*PRUNE)+`/`(*UTF)+`/`(*LIMIT_HEAP=5)+` reject in BOTH. New pin
    `parser_registry::tests::regex_quantified_verb_rejects_at_the_grammar_layer_pcre2_faithfully` green.
  - [x] **NO REGRESSION** — regex cert `total=232 witness=232 UNKNOWN=0 fully_certified=true spf=0` at
    seeds 0/7/42 (230→232 = net +2 rules, all witnessed, NO gap); `--lint-grammar` 0 errors (232 rules);
    dual-feature lib suite **891 passed / 0 failed / 29 ignored** (890→891 = the new registry pin;
    `parse_harness_equivalence_gate` ✅ regex stays differential-CERTIFIED over the new grammar,
    `ast_shape_contract` regex ✅ manifest 211→214, version drift gate ✅, all 93 contract success samples
    parse); `duality_hunt_gate` ✅ re-baselined same-commit (the `only ACCEPT verb…` signature VANISHED
    from all lanes; the sole remaining regex class is the start-option-position class owned by `.4`, now
    in all 6 regex lanes — RNG-stream shift, NO novel class; tripwire green); `regex_pcre2_compile_oracle_gate`
    ✅ byte-identical baseline v11 (conformance-neutral — the start-option-quantified forms aren't in the
    2189-cell corpus, the `.3.19` precedent); `mdbook_docs_gate` + `regex_parser_book_gate` ✅.
  - [x] **LOCKSTEP** — ledger `REGEX-0096` + `embedding_api.rs` version consts (release **1.1.87** /
    contract **1.1.89**, schema `1`) + tracked contract JSON (`regex_parser_integration_contract_v1.json`
    version fields) + contract MD (Identity + "Release 1.1.87 / Contract 1.1.89 Highlights"); regex book
    (`rules-misc` § `directive_verb` rewritten, `rules-piece` § `piece` 6 branches, `changelog-index`) +
    tracked `regex_parser_book-html` regenerated; top book `parser-families` version chain +
    `stimuli-and-quality` residual-universe note (top book-html is gitignored); ast_shape manifest inventory
    211→214 (re-derived byte-exact from the regenerated inventory); duality contract re-baseline; live docs
    (CHANGES / DEVELOPMENT_NOTES / LIVE_ACHIEVEMENT_STATUS / MEMORY / TASK_TREE). Known pre-existing
    staleness recorded (NOT introduced here): the `#[ignore]`d contract fixture's `directive_named`/
    `directive_name` `required_rule_names` (see DEVELOPMENT_NOTES) — the non-ignored success test only
    checks parseability; all 93 samples parse.
- ID: `.3.21`  Status: **`done`** (`PGEN-REGEX-PCRE2-0019`, session #70 2026-07-08; regex release
  `1.1.87`→**`1.1.88`**, contract `1.1.89`→**`1.1.90`**, schema `1`, ledger `REGEX-0097`) — LATENT
  accepts-invalid divergence, oracle-verified 2026-07-08 during `.3.14`; UNBLOCKED like `.3.16` on
  2026-07-08 (`STIMULI-SIGNOFF.13.2` interpreter value-constraint mirror).
  Goal: LIMIT `=value` RANGE — PGEN accepts `(*LIMIT_HEAP=99999999999999999999)` (the `.3.14` grammar
  requires `digit+` but bounds no value) where PCRE2 10.47 REJECTS (err 160). This is a RELEASED
  accepts-invalid hole (parseability_probe release `1.1.87`: `(*LIMIT_HEAP=4294967290)abc`,
  `(*LIMIT_HEAP=4294967295)abc`, `(*LIMIT_HEAP=99999999999999999999)abc` all ACCEPT) — NOT a validator
  migration (`find_invalid_verb_construct` never value-checked LIMIT, `regex_compile_validation.rs:369-497`),
  so it takes a ledger row + release bump like `.3.18`/`.3.20`.
  **IN-SLICE ORACLE ADJUDICATION (tools-first — the leaf's own u32-max hypothesis was REFUTED):** the
  candidate boundary `4294967295`/`4294967296` is WRONG. `pcre2test` 10.47, one-pattern-per-run, binary
  search: last ACCEPT = **`4294967289`** (`0xFFFFFFF9`), first REJECT = **`4294967290`** (`0xFFFFFFFA`) —
  `4294967295` (u32 max) itself REJECTS. This is PCRE2's Horner-with-pre-check overflow guard
  (`n > UINT32_MAX/10 - 1` = `n > 429496728` before appending each digit ⇒ max accepted = `429496728*10+9
  = 4294967289`; the err-160 "here" marker sat right before the last digit). Uniform across all 4 names
  (LIMIT_HEAP/MATCH/DEPTH/RECURSION). Purely VALUE-based: unlimited leading zeros
  (`(*LIMIT_HEAP=0000000000000000004294967289)` ACCEPT, `…4294967290` REJECT; all-zeros = value 0 ACCEPT).
  **ENCODE (the `.3.16`/`.3.18` STRUCTURAL idiom scaled to ≤4294967289, NOT `@range` — `@range` proved
  atom-scoped/inert on native-body self-hosted rules by `.3.16`):** wrapper `directive_payload_digits =
  directive_limit_value_body -> $text` (keeps the released `value:"…"` STRING carrier byte-identically;
  the wrapper single-rule-ref body dodges the Or-root span hazard — `$text` spans the whole body);
  `directive_limit_value_body = "0"+ directive_limit_value_core? | directive_limit_value_core` (leading
  zeros + optional nonzero-led core, all-zeros ⇒ value 0); `directive_limit_value_core` = the nonzero-led
  1..4294967289 ladder (10-digit lexicographic bound `4294967289` most-specific-first + free 9..1-digit
  branches), so an out-of-range run has NO fully-consuming parse (the trailing `)` fails) and generation
  is in-range BY CONSTRUCTION. NOT `@profiles`-gated: value bound universal in both profiles (the `.3.18`
  quant-bound precedent; `.3.14`/`.3.20` known-name shapes stay authoritative in relaxed).
  See the Acceptance Checklist below for the executed evidence.

  ### REGEX-PCRE2-FIDELITY.3.21 — Acceptance Checklist (enforced)
  - [x] **REPRODUCE / ISSUE** — release probe `1.1.87`: `printf '(*LIMIT_HEAP=4294967290)a' |
    parseability_probe --parse regex` → ACCEPT, same for `(*LIMIT_HEAP=4294967295)a` (u32 max) and
    `(*LIMIT_HEAP=99999999999999999999)a`, where `pcre2test` 10.47 rejects all err 160. In-range values
    (`=0`/`=500`/`=65535`/`=4294967289`) correctly ACCEPT. Purely LATENT accepts-invalid — no validator
    check ever bounded the LIMIT value (`find_invalid_verb_construct` checks name/shape/POSITION only),
    invisible to the duality hunter (generator + parser agree; the `.3.19`/`.3.22` oracle-differential class).
  - [x] **ROOT CAUSE (WHY + WHERE)** — WHY: `directive_payload_digits = digit+` (`grammars/regex.ebnf:1477`
    pre-fix) bounds the LIMIT `=value` SHAPE but no VALUE, so any digit run parses; and no out-of-band
    check bounded it (`regex_compile_validation.rs::find_invalid_verb_construct:369-497`). WHERE:
    `directive_payload_digits` ← `directive_payload_equals` (`:1476`) ← `directive_limit_named` (`:1448`).
    Oracle-pinned FIRST (`pcre2test` 10.47, one-pattern-per-run, binary search): last ACCEPT =
    **`4294967289`** (`0xFFFFFFF9`), first REJECT = **`4294967290`** — PCRE2's Horner overflow guard
    `n > UINT32_MAX/10 - 1` (max = `429496728*10 + 9 = 4294967289`), NOT u32 max (which itself rejects).
    Uniform across all 4 LIMIT names, purely value-based (unlimited leading zeros; all-zeros = 0). The
    leaf's own candidate boundary (`4294967295`/`4294967296`) was REFUTED tools-first.
  - [x] **FIX (tier: GRAMMAR — no engine, no validator; NEW bound, not a migration)** — structural encode
    (`.3.16`/`.3.18` idiom, NOT `@range` — proved atom-scoped/inert by `.3.16`): wrapper
    `directive_payload_digits = directive_limit_value_body -> $text` (keeps the `{separator, value:"…"}`
    STRING carrier byte-identically; single-rule-ref body dodges the Or-root span hazard);
    `directive_limit_value_body = "0"+ directive_limit_value_core? | directive_limit_value_core` (leading
    zeros + optional nonzero-led core, all-zeros ⇒ 0); `directive_limit_value_core` = the nonzero-led
    1..4294967289 ladder (10 lexicographic-bound branches on `4294967289` + 9 free 9..1-digit branches).
    NOT `@profiles`-gated (value bound universal, the `.3.18` precedent). `lint` 0 errors / 0
    undefined-refs / 0 unreachable (234 rules).
  - [x] **ADDRESSED (verified)** — 21-cell release-probe verdict matrix == `pcre2test` 10.47 (**0
    mismatches**, both profiles): `4294967290`/`4294967295`/`4294967296`/`99999999999999999999` +
    leading-zero `…4294967290` flip ACCEPT→REJECT; boundary `4294967289` + `65536` + all-zeros + `500`
    (empty body) stay ACCEPT; all 4 LIMIT names uniform; relaxed == default (proving no fallback hole —
    out-of-range does NOT reparse as anything else). AST byte-identical for in-range (release
    `--parse-dump-ast`: `(*LIMIT_HEAP=00700)a` → `payload:{separator:"=", value:"00700"}`, raw digit
    string with leading zeros). Generation in-range BY CONSTRUCTION (debug `--generate-stimuli
    --entry-rule directive_limit_named --count 200`, seeds 0/7/42: **0/601 over-max**, incl.
    boundary-adjacent `4294967286`/`4294967236` + leading-zero forms). New pin
    `parser_registry::tests::regex_limit_value_range_rejects_at_the_grammar_layer_pcre2_faithfully` green.
  - [x] **NO REGRESSION** — regex cert `total=234 witness=234 UNKNOWN=0 fully_certified=true spf=0` at
    seeds 0/7/42 (232→234 = net +2 rules `directive_limit_value_body`/`directive_limit_value_core`, both
    witnessed, NO gap); `--lint-grammar` 0 errors (234 rules); dual-feature lib suite **892 passed / 0
    failed / 29 ignored** (891→892 = the new registry pin; `parse_harness_equivalence_gate` ✅ regex stays
    differential-CERTIFIED — `certified_grammars_are_byte_identical` + combinator/semantic byte-identity
    green); `duality_hunt_gate` ✅ UNCHANGED (9 lanes byte-identical to the pinned contract — the sole
    regex class stays the start-option-position signature owned by `.4`; NO new/vanished signature,
    because the generator now emits only in-range values so there is no new break, and the parser
    rejecting out-of-range creates none — oracle-differential, hunter-invisible; NO re-baseline);
    `regex_pcre2_compile_oracle_gate` ✅ EXACTLY byte-identical — **decisive stash-baseline** (grammar
    reverted + parser regen): OLD `1858/285/46` == NEW `1858/285/46` (match/false_accept/false_reject on
    2189 cases), zero corpus cells flipped (every corpus `LIMIT=value` is in-range: `=0`/`=123`/`=1`;
    change is grammatically isolated to `directive_payload_digits`). 🔎 the previously-recorded oracle
    baseline `1857/286` was STALE (a `.3.18` universe-shift bookkeeping artifact carried through the
    `.3.19`/`.3.20` "byte-identical" assertions without re-measurement) — corrected to the true measured
    **`1858/285/46`** this slice; gate ratchets (`MAX_FALSE_ACCEPT=299`, `MIN_MATCH=1845`) unaffected.
  - [x] **LOCKSTEP** — ledger `REGEX-0097` + `embedding_api.rs` version consts (release **1.1.88** /
    contract **1.1.90**, schema `1`) + tracked contract JSON version fields + contract MD (Identity +
    "Release 1.1.88 / Contract 1.1.90 Highlights — REGEX-0097"); regex book (`rules-misc` § `directive`
    LIMIT-value paragraph + `changelog-index` entry) + tracked `regex_parser_book-html` regenerated; top
    book `parser-families` version chain + `stimuli-and-quality` closure note (the last `@range`-class
    honest bound closed); ast_shape manifest UNCHANGED (no new return annotation — `$text` on
    `directive_payload_digits` preserved, the new rules are unannotated; inventory stays 214); oracle-gate
    env comment corrected (stale `1857/286`→true `1858/285`); live docs (CHANGES / DEVELOPMENT_NOTES /
    LIVE_ACHIEVEMENT_STATUS / MEMORY / TASK_TREE).
- ID: `.4`  Status: `pending` (RE-SCOPED 2026-07-09 `PGEN-REGEX-PCRE2-0022` — see the SCOPING LOG below)
  Goal: capstone — delete `validate_regex_compile_contract` + its module so the EBNF is the sole source
  of truth (`check_ebnf_source_of_truth.sh` green with no validator). **BLOCKED**: a tool-backed
  message-source probe (session #72) proved the validator is NOT residual — **8 check families / 54 of
  its 58 pinned reject inputs are still LOAD-BEARING** (the grammar ACCEPTS them; only the validator
  rejects — deleting it today = 54 accepts-invalid PCRE2 divergences). So `.4` is now a *deletion* leaf
  gated on the encode-in-EBNF child leaves `.4.1`..`.4.11` below; it runs LAST, once every family is
  grammar-owned. It still owns the `STIMULI-SIGNOFF.13.3` GATE-PINNED start-option POSITION duality
  re-baseline (`rust/test_data/grammar_quality/duality_hunt_gate_contract_v0.json`, regex canonical seed 0
  + scaled seed 0; `make -C rust duality_hunt_gate` fails until the encoding slice re-baselines it) — now
  carried by `.4.8`.
- ID: `.4.1` Status: `done` (`PGEN-REGEX-PCRE2-0023`, session #73) Goal: encode BARE property escape —
  `\p`/`\P` unbraced must be a 1-letter Unicode general category (`\pA`/`\P_`/`\p`@EOF reject). Owns
  `find_invalid_property_escape`. **STRUCTURAL** (the `property_escape` `p short_prop_letter` branch already
  exists; the leak is the fallback that lets `\p`+bad-letter parse — positive-exclusion in the escape rule,
  the `.3.13` idiom). Low risk. Released slice: release `1.1.89`→`1.1.90` / contract `1.1.91`→`1.1.92` /
  schema `1` (unchanged — no AST-shape change); ledger `REGEX-0100` (internal, behavior-NEUTRAL downstream).

#### `.4.1` Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — release `parseability_probe --parse regex --profile pcre2` on `\pA` / `\P_` / `\p`
  rejects with the VALIDATOR's message (`\p without braces must use a one-letter Unicode general category` /
  `malformed Unicode property escape`), which fires only AFTER a grammar-accept → the grammar ALONE accepts
  these 3 PCRE2-invalid patterns (message-source probe; the `.4` SCOPING LOG technique).
- [x] **ROOT CAUSE (WHY + WHERE)** — normal context: `simple_escape_letter_strict` (`regex.ebnf:934-935`)
  positively enumerates `'P'`/`'p'`, so `\pA` → `property_escape` fails (A ∉ short-category set) → `simple_escape`
  matches `\p` as a bare shorthand + `A` literal. Class context: `class_simple_escape_{strict,relaxed}`
  (`:745`/`:748`) use `any_char` with only `!"p{"`/`!"P{"` guards (braced form only) → `[\pA]` leaks the same
  way. `whitespace`/`special_char`/`unicode_char` (the other `simple_escape_tail` arms) are all `p`/`P`-free
  (`unicode_char = !builtin_ascii_char …`), confirmed. Validator: `regex_compile_validation.rs:25,127`.
- [x] **FIX** — grammar (fix-tier: DECLARATIVE, EBNF-only): drop `'P'`/`'p'` from `simple_escape_letter_strict`
  (positive-exclusion, generation-faithful half); broaden `!"p{"`/`!"P{"` → whole-letter `!"p"`/`!"P"` on
  `simple_escape` + both `class_simple_escape` variants (positional-safe: `$5`/`$13`/`$7` unchanged). Remove
  `find_invalid_property_escape` + its call site + its 2 unit tests from the validator (`is_short_unicode_property_letter`
  retained — `skip_regex_escape` still uses it for the other checks).
- [x] **ADDRESSED (verified)** — after regen + release-probe rebuild, the reject corpus REJECTS with the
  GRAMMAR message `Parser did not consume full input` (validator-free — proving grammar ownership): `\pA`
  `\P_` `\p` `\P` (pos 0), `[\pA]` `[\P_]` `[a\pA]` (class), `a\pAb`/`x\p` (pos 1, mid-pattern); relaxed also
  rejects `\pA`. ACCEPTS (unchanged): `\pL` `\PN` `[\pL\PN]` `\pl` `\Pn` `\pC` `\pZ` `\p{Lu}` `\P{Han}` `[\p{L}]`.
- [x] **NO REGRESSION** — regex cert-coverage `total=236 proof=0 witness=236 UNKNOWN=0 fully_certified=true
  spf=0` at seeds 0/7/42 (rule-count unchanged); `regex_pcre2_compile_oracle_gate` EXACTLY byte-identical
  baseline `2189/1858/285/46` (fresh recompile confirmed); `duality_hunt_gate` ✅ 9 lanes, 1/1 pinned per regex
  lane, byte-identical determinism (no new/vanished signature); `parse_harness_equivalence_gate` ✅
  `certified_grammars_are_byte_identical` (regex interpreter↔generated byte-identical); `regex_ast_shape_
  contract_gate` ✅ drift aligned; `regex_parser_integration_contract_gate` ✅; 51 focused lib tests pass (new
  pin + `regex_parser_pgen_rgx_0086_embedding_version_consts_match_ledger` at `1.1.90`/`1.1.92` + all
  `regex_compile_validation` tests); `--lint-grammar` 0 errors (236 rules); the other 5 fully-certified
  grammars untouched (only regex regenerated); clippy no-new-findings (178 eq_op generated debt pre-existing,
  none cite the 3 changed files).
- [x] **LOCKSTEP** — `grammars/regex.ebnf`; `regex_compile_validation.rs`; `parser_registry.rs` (pin
  `regex_bare_property_escape_pcre2_faithfully`); `embedding_api.rs` consts `1.1.90`/`1.1.92`;
  `regex_parser_integration_contract_v1.json`; ledger `REGEX-0100`; contract Identity + `1.1.90`/`1.1.92`
  Highlights; regex book (`rules-escape.md` § `simple_escape`/`property_escape` + `changelog-index.md` +
  tracked `-html`); top book `parser-families.md`; `CHANGES.md`; `DEVELOPMENT_NOTES.md`;
  `LIVE_ACHIEVEMENT_STATUS.md`; `MEMORY.md`; `docs/TASK_TREE.md`.
- ID: `.4.2` Status: **`done`** (`PGEN-REGEX-PCRE2-0027`, session #76; release `1.1.92`→`1.1.93`, contract `1.1.94`→`1.1.95`, schema `1`, ledger `REGEX-0103`) Goal: encode `\k`/group NAME
  validity — charset (letter/digit/`_`/Unicode, no leading digit) + length ≤ 128
  (`\k`/`\kabc`/`\k''`/`\k<>`/`\k{}` + a 129-char capture name reject). Owns
  `find_invalid_named_escape_or_group_name`. **DESIGN RESOLVED tools-first (session #76)** — the length half's
  byte-vs-code-unit question (SCOPING §`.4.2`) is settled: `pcre2test` 10.47 gives err 148 "subpattern name
  is too long (maximum 128 **code units**)" and the code-unit is width-dependent (8-bit → byte, 32-bit →
  code-point; measured: 64×'é'=128B OK/65×'é' REJECT @8-bit, 128×'é' OK/129 REJECT @32-bit). RGX is
  Unicode-only ([[feedback_rgx_unicode_only_8bit_test_divergence]]) ⇒ the faithful limit is **128 code
  points**, which PGEN's `name` rule counts natively ⇒ a pure-grammar `{0,127}` bound (BOUNDED-QUANT.1),
  **NO new primitive**. All three fixes are STRUCTURAL/DECLARATIVE (fix-hierarchy GRAMMAR tier): (1) bound the
  shared `name` rule `* → {0,127}` (PCRE2 enforces 128 UNIFORMLY on every name — defs AND `\g`/`(?&`/`(?P=`/
  `\k`/condition refs, all err 148, measured); (2) `\k` always-introducer: `!"k"` guard on `simple_escape` +
  drop `'k'` from `simple_escape_letter_strict` (the `.4.1` `\p`/`\P` precedent); (3) add the missing
  `\k'name'` quote branch to `backreference`. See the `.4.2` implementation section + Acceptance Checklist below.
- ID: `.4.3` Status: **`done`** (`PGEN-REGEX-PCRE2-0029`, session #77; release `1.1.93`→`1.1.94`, contract
  `1.1.95`→`1.1.96`, schema `1`, ledger `REGEX-0104`) Goal: encode counted-quantifier `{N,M}` min>max ORDER
  (err 104; `x{5,4}`/`a{\t5\t,\t2\t}` reject). Owns the residual of `find_invalid_counted_quantifier`.
  **VALUE-COMPARISON** (cross-number, leading-zero-hostile) — `.3.18` explicitly deferred it here; used the
  RULE-SPAN `value_compare` `@predicate` primitive (RULE-SPAN-VALUE-CONSTRAINT.2). Behavior-NEUTRAL
  validator→grammar migration; oracle byte-identical. See the `.4.3` implementation section + Acceptance
  Checklist below.
- ID: `.4.4` Status: **`done`** (`PGEN-REGEX-PCRE2-0025`, session #75; release `1.1.91`→`1.1.92`, contract
  `1.1.93`→`1.1.94`, schema `1`, ledger `REGEX-0102`) Goal: encode class shorthand/escape rejects —
  `\A \B \C \G \K \N`(unbraced)`\R \X \Z \z` inside `[...]` (`[\B]`/`[\K]`/`a[\NB]c` reject). Part of
  `find_invalid_char_class_construct`. **STRUCTURAL** (positive class-escape enumeration; separable from
  ranges). Behavior-NEUTRAL validator→grammar migration; oracle byte-identical. See the `.4.4`
  implementation section + Acceptance Checklist below.
- ID: `.4.5` Status: `active` (tools-first INVESTIGATION done 2026-07-09 session #77, `PGEN-REGEX-PCRE2-0030`, PURE-DOCS; BUILD pending) Goal: encode class RANGE validity — nonliteral endpoints (`[\d-x]`,
  `[a-\p{Lu}]`, …) + DESCENDING ranges (`[z-a]`, `[\x{100}-z]`, decoded octal/hex/control endpoints).
  Part of `find_invalid_char_class_construct`. nonliteral = **STRUCTURAL-ish** (a range endpoint must be a
  literal-class atom); descending = **VALUE-COMPARISON over DECODED codepoints** (hard — needs a
  value-constraint primitive that decodes `\x{}`/`\NNN`/`\cX`/`\a`/`\e` endpoints and compares).
  🔎 INVESTIGATION REFRAMED THE SCOPE (see the `.4.5` section below): three sub-classes, NOT two — (a) a
  NEW **accepts-invalid divergence** the SCOPING-LOG sample missed: a `-[…` right endpoint in a NORMAL
  class (`[x-[:alpha:]]` err 150, `[a-[b]]`/`[z-[a]]` err 108) is rejected by PCRE2 but the released
  parser+validator BOTH ACCEPT it (root: `dash_starts_alt_extended_class_operator` `regex_compile_validation.rs:716`
  wrongly suppresses range detection for `-[` inside `scan_char_class`, which only runs on normal classes);
  (b) non-`[` nonliteral migration (grammar ACCEPTS as split members today, validator rejects) — STRUCTURAL
  PEG-ordering; (c) non-`[` descending migration (grammar accepts, validator rejects via codepoint decode) —
  needs the `codepoint`-coercion `value_compare` widening. Frozen oracle matrix in the section below.
- ID: `.4.5.a` Status: **`done`** (`PGEN-REGEX-PCRE2-0031`, session #78; release `1.1.94`→`1.1.95`, contract
  `1.1.96`→`1.1.97`, schema `1`, ledger `REGEX-0105`) Goal: fix the `-[` (and `-||`) accepts-invalid class-RANGE
  divergence — a range whose right endpoint begins with `[` or `||` inside a NORMAL class. VALIDATOR-tier fix
  (corrects a mis-scoped check; grammar-migration of the whole class-range family stays the deferred `.4.5.b`/
  `.4.5.c` — those need the PEG-ordering + `value_compare` codepoint-widening primitives). BUILD-time refinement
  of the investigation's root cause (AST-dump-proven): the grammar ALREADY forms `class_range` correctly for
  `-[` (treats `[` as literal `0x5B`) — the divergence is ENTIRELY the validator's `dash_starts_alt_extended_class_operator`
  guard skipping range detection. See the `.4.5.a` implementation section + Acceptance Checklist below.
- ID: `.4.5.a.1` Status: **`done`** (`PGEN-REGEX-PCRE2-0032`, session #79; release `1.1.95`→`1.1.96`, contract
  `1.1.97`→`1.1.98`, schema `1`, ledger `REGEX-0106`) Goal: fix the `\v` / `\V` accepts-invalid class-RANGE
  divergence — the vertical-whitespace shorthands were treated as LITERAL range endpoints, so `[\v-x]`,
  `[\V-x]`, `[a-\v]` were ACCEPTED where PCRE2 10.47 rejects them err 150 "invalid range" (they are NONLITERAL,
  exactly like `\h` / `\H`). A SECOND class-range accepts-invalid correctness fix in the `.4.5.a` vein, surfaced
  by the `.4.5.b` tools-first investigation (session #79). GRAMMAR+VALIDATOR tier (unlike `.4.5.a`, which was
  validator-only): the two-halved `\v`/`\V` bug is (1) a GRAMMAR mis-classification — `v`/`V` were in
  `class_range_literal_escape_letter_strict` (regex.ebnf:878-879) so `\v`/`\V` formed a `class_range` endpoint
  and the store-aware generator could emit `\v`-ranges — and (2) a VALIDATOR omission —
  `is_nonliteral_class_escape` (regex_compile_validation.rs:652) enumerated `d D h H s S w W p P` but omitted
  `v`/`V`, so they fell through to `class_escape_literal_codepoint`'s `_ => next as u32` and decoded to literal
  `118`/`86`. FIX makes `\v`/`\V` byte-identical in structure to the already-correct `\h`/`\H`: remove `v`/`V`
  from the range-letter set (members via `class_simple_escape` unaffected; stops generation) + add `b'v'|b'V'`
  to `is_nonliteral_class_escape` (validator rejects the range). Behavior-CHANGING (3 accepts-invalid cells
  flip to reject) — oracle NET improvement. The class-range family stays validator-owned pending the deferred
  `.4.5.b`/`.4.5.c` grammar migration, which now migrates a CONSISTENT nonliteral family (`d D h H s S v V w W
  p P`). See the `.4.5.a.1` implementation section + Acceptance Checklist below.
- ID: `.4.5.b` Status: **`done`** (`PGEN-REGEX-PCRE2-0033`, session #80; release `1.1.96`→`1.1.97`, contract `1.1.98`→`1.1.99`, schema `1`, ledger `REGEX-0107`) Goal: migrate the non-`[` NONLITERAL class-range reject
  from the out-of-band validator INTO the grammar — a `class_range` whose LEFT or RIGHT endpoint is a
  NONLITERAL SHORTHAND (`\d \D \h \H \s \S \v \V \w \W`) or PROPERTY (`\p… \P…`) escape is PCRE2 err 150
  "invalid range" (oracle `pcre2test` 10.47), but the grammar today SPLITS it into separate members
  (`class_atom` excludes the shorthand/property escapes so `class_range` never forms → the mid-class `-`
  falls through to a literal member) so the GRAMMAR ACCEPTS it and only `validate_regex_compile_contract`
  rejects it. **STRUCTURAL** (a negative-lookahead `!invalid_class_range` guarding `class_item` /
  `class_item_visible` / `class_item_visible_nocaret`; two shapes — nonliteral-LEFT `<nl> - <atom>` and
  literal-LEFT/nonliteral-RIGHT `<atom> - <nl>`), duality-safe because the store-aware generator never
  emits these ranges (the oracle already rejects them). SCOPE (per the frontier design note + the
  resume-pointer migration set `d D h H s S v V w W p P` + `\p{}`): shorthand + property endpoints only;
  the POSIX-left (`[[:alpha:]-z]`) and the `-[`-right cases (`.4.5.a`, `[a-[:digit:]]`) stay
  validator-owned, migrated with `.4.5.c`. Because the validator STILL rejects these post-`.4.5.b` (its
  range-check deletion is deferred to after `.4.5.c`), the released `--parse` verdict is UNCHANGED — the
  migration's EFFECT is observable only at the GRAMMAR layer, so the before→after proof is the certified
  interpreter (grammar-only, no validator): grammar ACCEPTS the invalid ranges before → REJECTS after,
  carve-outs (`[a-]` `[-a]` `[\d-]` `[\d\-x]` `[\da-z]` `[-\d]` `[a-z]` `[--/]`) stay ACCEPT. See the
  `.4.5.b` implementation section + Acceptance Checklist below.
- ID: `.4.5.c` Status: **`done`** (`PGEN-REGEX-PCRE2-0034`, session #81; release `1.1.97`→`1.1.98`, contract
  `1.1.99`→`1.1.100`, schema `1`, ledger `REGEX-0108`) Goal: migrate the non-`[` DESCENDING class-range
  reject INTO the grammar — a `class_range` of two LITERAL endpoints whose left code point exceeds the
  right (`[z-a]`, `[9-0]`, `[\x39-\x30]`, `[\x{100}-a]`, PCRE2 err 108 "range out of order in character
  class", oracle `pcre2test` 10.47) is accepted by the GRAMMAR today (the `@validate: ord($1)<=ord($5)`
  is a non-parse-gating codegen annotation, proven `.4.5.a`) and rejected only by
  `validate_regex_compile_contract`. **VALUE-COMPARISON over DECODED code points** using the new
  `value_compare_codepoint` `@predicate` (`RULE-SPAN-VALUE-CONSTRAINT.3`, `PGEN-RSVC-0003`), consumed via
  a THIRD `invalid_class_range` alternative (the `.4.5.b` `!invalid_class_range` lookahead pattern
  extended to a value constraint), NOT a direct `@predicate` on `class_range`. Three tools-first
  root-causes drove the final design (each an interpreter/`--parse-dump-ast` finding): (1) a `le` gate ON
  `class_range` does not work — the failed range branch just BACKTRACKS and the class reparses `z`,`-`,`a`
  as members → still accepts; the descending range must be BLOCKED at the `class_item` level, so
  `descending_class_range` (the 5-element `class_range` shape gated `value_compare_codepoint gt`, matching
  iff descending) is added as an `invalid_class_range` alternative under the existing `!` guard. (2) The
  endpoints must reach the gate as RAW spellings — a bare ref to `class_atom` gives its typed RETURN
  value (`{type:escape,…}` for escapes, `scalar_text`→`None`→INAPPLICABLE→over-block ascending), so
  `class_range_endpoint = … -> $text` re-emits the raw matched text. (3) an UNDECODABLE endpoint
  (`\Q..\E`, `\u{...}`) → `None` → non-blocking → over-block (cert regression: `quoted_class_range_atom`
  UNKNOWN), so `class_range_endpoint` matches only the DECODABLE forms (bare literal + hex/octal/control/
  simple escapes), leaving `\Q..\E`/`\u{}` descending to the validator (join `.4.5.d`). (4) a BARE
  WHITESPACE endpoint (` `, `\t`, …) is the SAME non-blocking failure mode — caught by the
  `regex_pcre2_compile_oracle_gate` (false_reject `48→50`, cell `[ --]` + one `--`-operator cell): a
  `class_literal → whitespace` match reaches the `post` predicate with an EMPTY raw spelling
  (`value_compare_codepoint("" > "!") → None` for `[ -!]`, EITHER slot — proven by runtime trace; the
  whitespace-only span is not recovered through `$text` in the predicate-arg path, while every
  non-whitespace literal AND the escape spellings `\x20`/`\t`/`\040` resolve correctly), so an ASCENDING
  whitespace range (`[ -!]`, `[ --]`, `[\t-a]`) was wrongly matched and OVER-BLOCKED. FIX: `class_range_
  decodable_atom` spells out the NON-WHITESPACE `class_literal` alternatives directly (`class_range_
  decodable_escape | letter | digit | class_safe_special | unicode_char`), dropping the `whitespace`
  branch → `descending_class_range` does not form for a bare-whitespace endpoint → the range is left to
  the validator (joins the `.4.5.d` deferred set). The empty-`$text`-for-whitespace-in-a-post-predicate-
  arg is a GENERAL latent pipeline finding, recorded durably ([[project_dollar_text_whitespace_empty_in_predicate_arg]])
  for a future engine investigation. Post-fix oracle `2189/1867/274/48` byte-identical. Behavior-NEUTRAL
  validator→grammar migration (validator STILL rejects, so `--parse` verdict UNCHANGED; before→after
  proven at the GRAMMAR layer by the certified interpreter). Duality-safe by the `.4.3` empirical argument.
  SCOPE: non-`[` descending LITERAL (bare + decodable-escape) only; the `-[`-right (`.4.5.a`), POSIX-left,
  and `\Q..\E`/`\u{}`-endpoint cases stay validator-owned, migrated by the follow-up **`.4.5.d`** which
  then DELETES the `find_invalid_char_class_construct` range-check. Also surfaced a separate finding
  (G2/G3 bisect): a POST predicate on a TOP/ENTRY rule with a leading literal + no `->` mis-resolves
  positional args in the interpreter — recorded for a future investigation; does NOT affect `class_range`
  (a deep sub-rule). See the `.4.5.c` implementation section + Acceptance Checklist below.
- ID: `.4.5.d` Status: **`done`** (`PGEN-REGEX-PCRE2-0035`, session #82; release `1.1.98`→`1.1.99`, contract
  `1.1.100`→`1.1.101`, schema `1`, ledger `REGEX-0109`) Goal: migrate the still-validator-owned **POSIX-class
  (`[:name:]`) nonliteral class-range endpoint** reject INTO the grammar — a `class_range` whose LEFT or RIGHT
  endpoint is a valid POSIX class (`[[:alpha:]-z]`, `[!-[:alpha:]]`) is PCRE2 err 150 "invalid range in
  character class" (`pcre2test` 10.47). Tools-first (grammar-only interpreter, `zz_diag_class_range_residual`)
  pinned the EXACT residual: the DESCENDING posix-right cases (`[x-[:alpha:]]`, `[a-[:digit:]]`, left>`[`=0x5B)
  already reject via `.4.5.c`'s `descending_class_range` (the `[` reads as a `class_safe_special` literal 0x5B),
  so the genuine grammar-over-accept residual is the ASCENDING posix-right (`[!-[:alpha:]]`, `!`=33 < `[`=91) and
  the posix-LEFT (`[[:alpha:]-z]`). FIX (grammar tier, `feedback_no_workarounds_fix_hierarchy` tier 1 — the
  existing `.4.5.b` `!invalid_class_range` lookahead): two new alternatives referencing the EXISTING
  positively-reachable `posix_class` rule — `posix_class zw* "-" zw* class_atom` (posix LEFT) and `class_atom
  zw* "-" zw* posix_class` (posix RIGHT). No new rules (cert total stays 249), no engine, no `$text`, no
  blocker. The `class_atom`-after-`-` requirement preserves the trailing-dash carve-out (`[[:alpha:]-]` stays
  ACCEPT). **NOT fully behavior-neutral — a genuine PCRE2-CONVERGENT correctness fix for one subset (surfaced
  when the contract-manifest gate flipped, then root-caused).** For MOST posix-endpoint cases the validator
  ALREADY rejected them (released `--parse` unchanged), BUT the validator has an accepts-invalid HOLE:
  `dash_is_trailing_literal` skips whitespace/zero-width after the dash and treats it as a literal trailing dash
  even for a NonLiteral (posix) LEFT endpoint — so `[[:digit:]-   ]` (dash + only whitespace before `]`) was
  ACCEPTED by the released parser, where `pcre2test` 10.47 rejects err 150. The grammar migration flips that
  subset ACCEPT→REJECT at `--parse` (one contract success sample → failure; oracle byte-identical because the
  flip cells are not in the corpus). SCOPE: valid-POSIX-name
  endpoints only. The remaining class-range residuals stay validator-owned: the **collating/equivalence**
  bracket-tokens (`[!-[.a.]]`, `[!-[=a=]]` — no grammar recognizer yet; joins `.4.12`) and the **blocked
  descending** endpoints (`\Q..\E`/`\u{}` decode-`None`, bare-whitespace empty-`$text` —
  [[project_dollar_text_whitespace_empty_in_predicate_arg]]). The `find_invalid_char_class_construct`
  range-check DELETION waits until ALL of these land. See the `.4.5.d` implementation section + Acceptance
  Checklist below.
- ID: `.4.6` Status: **`done`** (`PGEN-REGEX-PCRE2-0024`, session #74; release `1.1.90`→`1.1.91`, contract
  `1.1.92`→`1.1.93`, schema `1`, ledger `REGEX-0101`) Goal: encode POSIX class NAME validity — unknown
  `[[:foo:]]` reject + the exact `[[:<:]]`/`[[:>:]]` word-boundary aliases (mixed `[a[:<:]]` reject). Part of
  `find_invalid_char_class_construct`. **STRUCTURAL** (an inline negative-lookahead guard on the class-member
  `[` literal for the `[:…:]` posix-token shape — the `literal_open_brace` `!(…) X -> $2` precedent).
  Behavior-NEUTRAL migration (scans to first `:]` like the deleted validator); oracle byte-identical. See the
  `.4.6` implementation section + Acceptance Checklist below.
- ID: `.4.7` Status: `in_progress` (decomposed into `.4.7.a`/`.4.7.b`/`.4.7.c`) Goal: encode scan-substring
  capture inventory — `(*scs:(N))`/`(*scs:(<name>))` must reference an AVAILABLE capture (`(*scs:(1)…)`@0-groups,
  `(*scs:(0)…)`, `(*scs:(<name>)…)`@unknown reject; forward refs LEGAL). Owns
  `find_invalid_scan_substring_capture_list`. **WHOLE-PATTERN two-pass** (capture count + name inventory) — the
  same store-aware class as `.3.22`; forward-ref legality forbids a single-pass `has_fact`. Consumes the
  `FINAL-PHASE-PREDICATE` primitive (tree COMPLETE). **DECOMPOSED tools-first** (`PGEN-REGEX-PCRE2-0046`,
  session #89) once the numeric sub-cases were PCRE2-oracled (`pcre2test` 10.47): the absolute + `-N` + named
  cases are cleanly expressible via `phase: final`/`phase: post`, but `+N` FORWARD-relative needs
  `prior_count + N ≤ full_count` — arithmetic the predicate vocabulary lacks (no "N groups defined at-or-after
  this position" fact). So:
  - ID: `.4.7.a` Status: **`done`** (`PGEN-REGEX-PCRE2-0046`, session #89; surface- & conformance-NEUTRAL, no
    version bump — the `.3.16`/`.3.17` precedent) Goal: NAMED scan-substring refs (`(*scs:(<name>))` /
    `(*scs:('name'))`) grammar-owned. The **second consumer** of `FINAL-PHASE-PREDICATE` after `.4.11`:
    `has_fact(regex_defined_capture_name, $name) phase: final` on `scs_capture_name`, reshaped `-> { name: $1 }`
    (from bare `-> $1`) so `$name` resolves — a parse `@predicate` arg resolves against the produced STRUCTURE,
    and ONLY a shaped object-key ref resolves (a bare carrier's `$text` and positional `$1` do NOT —
    interpreter-probed, see the `.4.7.a` implementation section). `scs_capture_name_ref` extracts `$2.name` to
    keep the embedded `captures` AST byte-identical. Validator's NAMED branch + `CaptureInventory.names` DELETED
    (behavior-neutral migration). Full evidence: the `.4.7.a` implementation section + Acceptance Checklist below.
  - ID: `.4.7.b` Status: **`done`** (`PGEN-REGEX-PCRE2-0047`, session #89; behavior-CHANGING accepts-invalid FIX,
    release `1.1.103`→`1.1.104` / contract `1.1.105`→`1.1.106` / schema `1` / ledger `REGEX-0113` per the `.4.11`
    precedent) Goal (REVISED): the RELATIVE-ZERO correctness fix ONLY — `(*scs:(+0))`/`(*scs:(-0))` (incl.
    leading-zero `+00`/`-00`) now REJECT (PCRE2 err 126 "a relative value of zero is not allowed"). The released
    validator ACCEPTED-invalid these (`+0`→prior_count, `-0`→prior_count+1, both in range). Fixed with a SINGLE
    validator guard (`is_relative_reference && reference == 0` in `validate_scan_substring_capture_refs`); absolute
    `0`/`00` already rejected (resolved index 0). **The grammar sign-split migration (absolute `phase:final` +
    backward `-N` `phase:post`) was BUILT + oracle-proven in-slice, then REVERTED and ABSORBED into `.4.7.c` — a
    WITNESSING WALL (below) makes it not-cleanly-achievable now, and it carries ZERO correctness value (the
    validator already handles absolute/backward correctly), so only the real bug is fixed.** See the `.4.7.b`
    implementation section + Acceptance Checklist below.
  - ID: `.4.7.c` Status: `pending` (BLOCKED on a new primitive) Goal: migrate the scan-substring NUMERIC refs
    (ABSOLUTE `(*scs:(N))`, BACKWARD `(*scs:(-N))`, FORWARD `(*scs:(+N))`) into the grammar → final
    `find_invalid_scan_substring_capture_list` deletion. **🧱 WITNESSING WALL (discovered `PGEN-REGEX-PCRE2-0047`,
    tools-first cert-coverage):** sound scs generation is ABSOLUTE-ONLY — the `@gen_predicate
    fact_count_at_least(regex_capture_group, $value)` compiles to a VALUE-DRAW `IndexUpTo` that replaces
    `scs_capture_number`'s whole render with an integer in `1..=prior_count` (empirically 0 signed scs refs in 60k
    stimuli). So a sign-split into `scs_capture_number_{absolute,backward,forward}` rules cannot keep BOTH
    `fully_certified` AND `spf=0`: with whole-render-replace the signed rules are never generated ⇒ `UNKNOWN=2`
    (cert regress, measured); with generation DESCENT the forward `+N` branch draws N≤prior_count and renders
    `+N` — UNSOUND (no group at-or-after) ⇒ the validator rejects it ⇒ `spf` spike. Backward `-N` alone IS
    soundly generatable (literal `-` + int≤prior) but cannot be isolated from forward without a branch-suppression
    mechanism that does not exist (no `@gen_weight`/`@gen_never`); and any scs-specific signed rule loses the
    shared-`signed_digits`-via-subroutines witness the current design relies on. Forward `+N` valid ⟺
    `prior_count + N ≤ full_count` ⟺ "≥ N capture groups defined AT-OR-AFTER this position" — a SUFFIX/forward
    count no predicate expresses AND the cert reach-driver cannot synthesize (its target-own-structure pass forces
    the rule but not a trailing group, so `+N` probes fail to re-parse ⇒ `SelectedButFailed`). **Needs a new
    parser-agnostic forward/suffix-count primitive** (which ALSO unblocks sound forward generation/witnessing).
    🔎 SURFACED for the director (see the Changelog callout + `docs/decisions/`). Until then ALL numeric scs refs
    stay validator-owned — correctness is PCRE2-faithful (the `.4.7.b` relative-zero fix included), full validator
    deletion deferred.
- ID: `.4.8` Status: `pending` Goal: encode start-option POSITION — a recognized `(*UTF)`… start option
  must appear only at the very start of the ENTIRE pattern, before any other construct AND not nested
  (`a(*CR)b`/`a(*LIMIT_HEAP=500)`/`(*FAIL)(*LIMIT_HEAP=5)a` FLAT-reject; `((*CRLF)a)`/`(?:(*CRLF)a)`/
  `(?=(*CRLF)a)`/`(*CRLF)((*LF)a)` NESTED-reject — all err 160). Owns the residual of
  `find_invalid_verb_construct` + the `STIMULI-SIGNOFF.13.3` duality pin (re-baseline
  `duality_hunt_gate_contract_v0.json` same-commit). **NO new primitive; grammar-only via the `.4.10`
  open-marker idiom.** ⚠️ SHARPENED `PGEN-REGEX-PCRE2-0041` (session #85, tool-backed, see the
  `.4.8 — NESTED-CASE DISCOVERY + DESIGN` section below): the earlier "LEFT-CONTEXT single-pass / CHEAPEST
  residual" framing was INCOMPLETE — it missed the NESTED-group rejects, which the released validator ALSO
  enforces (`is_start_option_position`), so behavior-neutrality needs the grammar to reject them too. Because
  the engine has NO pre-body emit, a monotonic `@emit_fact regex_body_started` must be set AT EVERY nesting
  OPENER (before the inner body) plus post-body on top-level flat pieces, gated by a single
  `@predicate lacks_fact(regex_body_started, body) phase:pre` on the extracted `start_option_piece`. This is
  a BROAD grammar migration (~8 new open-markers + ~11 existing-marker emits), NOT the cheapest residual. A
  future general pre-body-emit primitive (Option B) is SURFACED in that section but NOT taken this slice.
  ✅ **IMPLEMENTED via grammar design A″ (STRUCTURAL, not the fact-based A′), `PGEN-REGEX-PCRE2-0044`, session
  #86 — see the `.4.8 — IMPLEMENTATION LOG + A′→A″ PIVOT` section.** A′ (fact-based) was built + parse-derisked
  green, but its `@emit_fact`/`lacks_fact` gate is un-mirrorable on the GENERATION side (the stimuli generator
  has no `lacks_fact` branch-prune), so the generator emitted mis-positioned start options the A′ grammar then
  rejected (cert spf 0→65 — a duality break). A″ makes `start_option_piece` reachable ONLY off the whole-pattern
  `entry_concatenation` chain → position enforced STRUCTURALLY (no fact/predicate, zero hot-path cost, generation
  duality-clean by construction). Byte-identical (interpreter, 60+ patterns) + full oracle matrix (flat/nested/
  global-alternation) green; validator `find_invalid_verb_construct` + `is_start_option_position` DELETED; duality
  gate rebaselined (start-option class CLOSED). Migration test `regex_start_option_position_grammar_migration`.
- ID: `.4.8.1` Status: `pending` Goal: TIGHTEN the regex stimuli generator so it stops emitting PCRE2-INVALID
  patterns (be-alert tracking leaf, opened `PGEN-REGEX-PCRE2-0044`). **Not a `.4.8` regression** — a PRE-EXISTING
  generator over-approximation that A″ merely SURFACED: closing the dominant start-option duality-break class let
  the directed duality hunter re-target the next-easiest frontier, the generic `"Parser did not consume full
  input at position #"` signature. WHY+WHERE: the generator emits patterns PCRE2 10.47 ALSO rejects — a quantifier
  on a `(?#)` comment (`(?#)?+`, err 109), unbalanced parens (err 114), and conditional / nested `\Q\Q` forms —
  which the parser CORRECTLY rejects (verdict+furthest-pos identical on the pre-`.4.8` grammar, so pre-existing;
  NOT a parser rejects-valid bug). Currently PINNED in `duality_hunt_gate_contract_v0.json` (all 6 regex lanes,
  owner `.4.8.1`) so the gate stays honest while tracked. Cross-ref `STIMULI-SIGNOFF` (the generator-signoff
  vision + the 13.3 duality gate). Fix = constrain generation of quantifier-after-zero-width / conditional /
  nested-quoted-run so the generator never emits an invalid pattern; MEASURE the pinned-signature set shrinks.
- ID: `.4.9` Status: `pending` Goal: encode unbounded-lookbehind — a variable-length lookbehind body must
  be bounded (`(?<=a+)b`/`(?<=a*)b`/`(?<=a{2,})b`/`(?<=…(c+)…)` reject; fixed `(?<=a{2})b` accept). Owns
  `find_unbounded_quantified_lookbehind`. **LOOKBEHIND-LENGTH ANALYSIS** (hard — the body's max match
  length must be finite; needs a new parser-agnostic primitive).
- ID: `.4.10` Status: **`done`** (`PGEN-REGEX-PCRE2-0038`, session #84; release `1.1.100`→`1.1.101`, contract
  `1.1.102`→`1.1.103`, schema `1`, ledger `REGEX-0111`; see the `.4.10` implementation section + Acceptance
  Checklist below) Goal: encode `\K`-in-lookaround — `\K` inside any lookaround body rejects
  (`(?=a\Kb)`/`(?<=\K.)`/`(*pla:a\Kb)` reject; `\Kword` outside accepts). Owns
  `find_invalid_keep_out_escape_in_lookaround` (MIGRATED to the grammar + DELETED). **CONTEXTUAL** — the
  original `.1` table row 10 "hard one" (depends on the enclosing construct); the first grammar consumer of the
  scope-ancestry `@predicate` primitive `not_in_scope_kind` (`SCOPE-CONTEXT-PREDICATE.1`, landed `PGEN-SCP-0001`).
- ID: `.4.11` Status: **`done`** (`PGEN-REGEX-PCRE2-0045`, session #87; release `1.1.101`→`1.1.102`, contract
  `1.1.103`→`1.1.104`, schema `1`, ledger `REGEX-0098` Deferred→Fixed; see the `.4.11` implementation section +
  Acceptance Checklist below) Goal: encode the named-reference UNKNOWN-name inventory scoped by `.3.22`
  (`\k<zzz>`/`(?P=zzz)`/`(?&zzz)`/`\g{zzz}`… @undefined reject; forward/subroutine refs accept). **NOT** a
  current `validate_regex_compile_contract` check (validator is shape-only here) — a genuine
  released-parser accepts-invalid divergence (ledger `REGEX-0098`). **WHOLE-PATTERN two-pass**, same class
  as `.4.7`. The `.3.22` oracle matrix (17 cells) is its frozen acceptance spec — **17/17 PASS**.
  **FIRST CONSUMER of the `FINAL-PHASE-PREDICATE` primitive** (`.1` `PGEN-FPP-0001`, `.2` `PGEN-FPP-0002`).
  **As-built (differs from the pre-build sketch):** (1) a two-kind fact split — `regex_defined_capture_name`
  (parse INVENTORY, `@emit_fact name=$name` on the shaped `named_group`/`python_named_group` rules) vs
  `regex_capture_name` (gen-draw SOURCE, `@gen_emit_fact` on the bare `capture_name`) — the split stops the
  stimuli generator (which keys fact-draws on the producer rule's WHOLE RENDER) from splicing a whole
  `(?<name>…)` group into a reference slot; (2) named forms PEELED into HOMOGENEOUS rules so the gate binds
  to every named branch and never the numeric siblings — `named_backreference` (6 br) out of `backreference`,
  `named_subroutine_target` (2 br) out of `subroutine_target`, `reference_name` carrier added, `braced_name_ref`
  removed; (3) `has_fact(regex_defined_capture_name, $ref|$name) phase: final, **view: shaped**` on the three
  named-reference gates. **Root cause of the multi-branch reject-even-when-defined bug (PROVEN tools-first,
  `--trace-rules named_backreference`):** a `@predicate` defaults to `view: raw`; a multi-branch tournament
  captures the winner's raw `Sequence` for the final-predicate view, so a shaped-key `$ref` cannot resolve
  ("could not resolve attribute reference 'ref'") and the rule REJECTS a defined name. Single-branch rules only
  worked by the `semantic_raw_content=None` shaped-fallback accident (why the FPP.2 single-branch isolation
  proof missed it — see [[project_final_phase_deferred_predicate_primitive]] FIRST-CONSUMER FINDING). **Fix
  (hierarchy level 1, existing annotation option):** `view: shaped`. A second bug — `\g<zzz>` accepting undefined
  — SELF-RESOLVED: once `named_backreference` no longer fails structurally, longest-match routes `\g<zzz>`
  (7 chars) to it over `simple_escape` (2 chars); no `!"g"` guard needed.

  **`.4.11` Acceptance Checklist** (root cause + addressed + no regression):
  - **Root cause:** proven tools-first via `--trace-rules` (multi-branch `phase:final` shaped-key `$ref`
    resolved against raw `Sequence` under the `view:raw` default) — NOT eyeballed. WHY+WHERE both surfaced
    (resolver error + rule stack `…backreference > named_backreference`).
  - **Addressed:** `view: shaped` on the three named-reference `phase:final` gates in `grammars/regex.ebnf`
    (the lowest fix-hierarchy level that cleanly solves it — an existing annotation option). Zero engine/Rust
    change; `generated/*` are DERIVED (regenerated).
  - **No regression (tool-backed):** `.3.22` oracle matrix **17/17**; regex cert-coverage seeds 0/7/42
    **UNKNOWN 2→0, `fully_certified=true`**, witness 256→258; `ast_shape_contract` regex **aligned=4 drift=0
    regression_lock_failures=0** (inventory 236→238 re-synced; accepted-AST shapes byte-identical — `view` is a
    predicate, not a return-shape); `certified_grammars_are_byte_identical` regex byte-identical (interpreter
    mirror honors `view: shaped`); `duality_hunt` pass; `regex_pcre2_compile_oracle_gate` [result recorded in
    the implementation section]. **spf independence PROVEN by decisive clean-main baseline** (git-restored
    `regex.ebnf`, full regen+rebuild): clean-main spf **2/2/0** vs `.4.11` **0/1/1** — a NET IMPROVEMENT, so the
    residual seed-7/42 spf=1 (`\Q\E`/conditional/verb generator sample) is pre-existing and separately tracked,
    NOT a `.4.11` regression ([[feedback_prove_independence_with_decisive_baseline]]).
- ID: `.4.12` Status: **`done`** (`PGEN-REGEX-PCRE2-0037`, session #83; release `1.1.99`→`1.1.100`, contract
  `1.1.101`→`1.1.102`, schema `1`, ledger `REGEX-0110`; see the `.4.12` implementation section + Acceptance
  Checklist below) Goal: encode STANDALONE collating-element / equivalence-class
  reject — a `[.coll.]` / `[=equiv=]` token used as a plain class MEMBER (not a range endpoint). `pcre2test`
  10.47: `[[.a.]]` / `[[=a=]]` → err 113 "POSIX collating elements are not supported"; the released parser
  ACCEPTS both (grammar reads `[` as a literal member; the validator has no standalone collating/equivalence
  recognition). A genuine released-parser accepts-invalid gap, ledger `REGEX-00xx` candidate. DISTINCT from
  `.4.5.a` (which is the `-[` RANGE endpoint, err 150/108): the standalone-member reject is err 113 and needs
  its own recognizer (validator or grammar). `.4.5.a` deliberately did NOT fix this (surgical one-defect scope).
  Sequence after the remaining `.4.5.b`/`.4.5.c` class-range family, alongside the other `.4.7`..`.4.11` residuals.
  **🔎 ORACLE-SCOPED 2026-07-09 session #82 (post-`.4.5.d`; `pcre2test` 10.47, verified — NOT annotated):** PCRE2
  rejects a collating `[.x.]` / equivalence `[=x=]` bracket-token in a class **UNCONDITIONALLY** — `[[.a.]]`
  `[[=a=]]` (standalone) / `[a[.a.]b]` `[a[=a=]b]` (member) / `[[.a.]-z]` `[[=a=]-z]` (range LEFT) all err **113**
  "POSIX collating elements are not supported"; ONLY as a range RIGHT endpoint (`[a-[.a.]]` `[!-[.a.]]`
  `[!-[=a=]]`) does the `-` trigger err **150** first. This UNIFIES the standalone `.4.12` gap with the residual
  range-endpoint cases (the `.4.5.d` deferred collating/equivalence family): a SINGLE grammar recognizer that
  matches the `[.` … `.]` / `[=` … `=]` shape and REJECTS it (default/pcre2 profile) subsumes all of them — the
  `-` never even needs a range interpretation. Design: recognize the shape at the class-member position and make
  it non-parseable (or profile-gate to `relaxed`), mirroring the `.4.6` `class_member_literal` inline-lookahead
  idiom for the `[:...:]` posix shape. Careful boundary: `[.]`/`[=]` (single literal `.`/`=` member) and
  `[a.b]`/`[a=b]` (literals, no `[.`/`[=` opener) must stay ACCEPT; the opener is specifically `[.`/`[=`. This is
  the next slice after `.4.5.d` (a NEW recognizer, its own full lockstep + ledger `REGEX-00xx`).
- ID: `.5`  Status: `pending`  Goal: verification — full `regex_pcre2_compile_oracle_gate` parity
  (default), relaxed-mode test suite, cert-coverage at floor, RGX conformance ratchet, lockstep.

### REGEX-PCRE2-FIDELITY.4 — SCOPING LOG (`PGEN-REGEX-PCRE2-0022`, 2026-07-09, session #72, tool-backed, PURE-DOCS)

**Trigger.** The `MEMORY.md` resume pointer framed the next action as "delete the residual validator —
**only the start-option POSITION rule remains validator-owned**". A tools-first read of the ACTUAL code
found `validate_regex_compile_contract` (`rust/src/regex_compile_validation.rs`) still dispatches EIGHT
live checks (`parser_registry.rs:393-394` runs `parse_full_regex()` THEN the validator) — so the claim
needed verification, not trust ([[feedback_be_alert_root_cause_fishy_immediately]]).

**Method (definitive, no rebuild of logic).** The parse path runs the GRAMMAR first, then the validator
ONLY if the grammar accepted. So the rejection MESSAGE names the source:
`"Parser did not consume full input"` = grammar rejected (validator shadowed/dead for that input); a
validator `"…compile contract"` string = grammar ACCEPTED, validator rejected (**load-bearing** — deleting
it makes that input ACCEPT). Ran the validator's OWN pinned reject corpus (its `#[test]` `expect_err`
inputs, 58 cases) through the current release `parseability_probe --parse regex --profile pcre2` (rebuilt
this session to embed the `.3.23` parser) and classified each by message source. (First-pass classifier
naively read exit code and mis-reported ALL as "shadowed" — caught immediately as too-good-to-be-true;
the message-source correction gave the true map. Deterministic on re-run.)

**Result: 8/8 checks LOAD-BEARING; 54/58 reject inputs validator-owned; 4 grammar-shadowed**
(the empty-`\Q\E`-region / orphan-`\E` char-class cases already closed by `.3.15`/`.3.23`). Per-family
map + encodability = the `.4.1`..`.4.11` leaves above and the decision record
[[project_regex_validator_deletion_blocked_load_bearing]] (full table). Load-bearing sample reject
inputs the grammar ACCEPTS today: `\pA` `\k<>` `x{5,4}` `[\B]` `[z-a]` `[\d-x]` `[[:foo:]]` `(*scs:(1)a)`
`a(*CR)b` `(?<=a+)b` `(?=a\Kb)ab`.

**Why the belief was wrong (the key insight).** Two DIFFERENT axes were conflated:
(1) generator↔parser DUALITY (hunter-visible) IS clean except start-option-position — because the
store-aware generator never GENERATES these invalid forms; (2) validator LOAD-BEARING-ness is whether the
GRAMMAR ALONE accepts a HAND-WRITTEN invalid pattern — 8 families still do. "Duality-clean" ≠
"validator-deletable"; the capstone is about deletion, so it needs the grammar to reject all 54.

**Decision.** `.4` re-scoped into `.4.1`..`.4.11` (encode-per-family, released slices) THEN a final `.4`
deletion. Recommended order (structural/cheap first, so the validator shrinks monotonically with released
proof): `.4.1` → `.4.6` → `.4.4` → `.4.2` → then the hard families (`.4.3` order, `.4.5` descending
ranges, `.4.7`/`.4.11` two-pass inventories, `.4.8` start-option position, `.4.9` lookbehind length,
`.4.10` `\K`-in-lookaround) as their own design+build slices — several likely need a NEW parser-agnostic
primitive (value-comparison over decoded ranges / whole-pattern two-pass / contextual gate), the same
"hard" class flagged in the `.1` table (rows 9/10) and the `.3.18`/`.3.22` deferrals. No engine/grammar
change this slice (PURE-DOCS scope + re-scope).

### REGEX-PCRE2-FIDELITY.4 residual family — PRIMITIVE-GAP SCOPING (`PGEN-REGEX-PCRE2-0039`, 2026-07-10, session #84, tool-backed, PURE-DOCS)

**Trigger.** With `.4.10` landed (`\K`-in-lookaround grammar-owned), the residual `.4` family is
`.4.7`/`.4.8`/`.4.9`/`.4.11` + the blocked-descending class-range endpoints. The `.4` SCOPING LOG (above)
lumped `.4.7`/`.4.8`/`.4.11` as "two-pass / contextual — several likely need a NEW primitive." A tools-first
read of the ACTUAL engine + validators SHARPENS which primitive each needs — they do NOT all share one, and
`.4.8` needs none.

**Method (toolbox-first, per [[feedback_systematically_use_debug_toolbox]]).** Read the engine's
`@predicate` phase enum (`semantic_runtime.rs`) and the residual validators
(`find_invalid_scan_substring_capture_list`:783 + its `capture_inventory_before`:840,
`find_invalid_verb_construct`:170 + `is_start_option_position`; confirmed `.4.11` has NO validator — only
4 checks remain dispatched: verb / char-class / scs / lookbehind). Grepped the shared engine for any
whole-pattern / deferred / end-of-parse / two-pass / pre-scan hook.

**Decisive engine fact (WHY the primitive boundary is where it is).** `SemanticPredicatePhase` is EXACTLY
`Pre | Branch | Post` (`semantic_runtime.rs`; `@predicate.phase` parsing rejects anything else, `:4202`).
ALL THREE fire at the rule's LEFT-TO-RIGHT parse position — `Post` fires after the rule BODY but STILL at
that position, seeing only what has parsed SO FAR. There is NO whole-pattern pre-scan, no deferred /
end-of-parse resolution phase, no two-pass hook anywhere in the shared engine (grep: zero hits). So NO
existing phase can consult a capture that is defined LATER in the pattern (a legal forward reference). That
is the exact primitive boundary: a left-context/prior-state question is answerable today; a
whole-pattern/forward question is not.

**Per-leaf primitive gap (the sharpened map).**
- `.4.8` start-option POSITION — **LEFT-CONTEXT, single-pass; NO new primitive.** The validator's whole test
  is `is_start_option_position(bytes, index)` — "is every group before this position itself a start option",
  examining ONLY bytes BEFORE the position. Expressible today with the existing store: a monotonic
  `@emit_fact` "start-option prefix ended" set by the first non-start-option construct + a
  `@predicate lacks_fact(prefix_ended) phase:pre` gate on the start-option rule (the `.4.10` gate shape, but
  a monotonic left-context fact rather than a scope-ancestry walk — a monotonic fact is CORRECT here because
  once the prefix ends it never re-opens). ⇒ the CHEAPEST residual; owns the `STIMULI-SIGNOFF.13.3` duality
  pin (re-baseline `duality_hunt_gate` same-commit). The `.4` SCOPING LOG's "may need a new primitive" was
  imprecise for `.4.8`.
- `.4.7` scan-substring `(*scs:(N))`/`(*scs:(<name>))` inventory — **FORWARD-LOOKING; needs the new
  primitive.** Forward refs are LEGAL, so a ref must validate against the WHOLE-PATTERN capture inventory
  (captures defined AFTER the ref, not yet parsed). The validator is the reference two-pass:
  `capture_inventory_before(bytes, bytes.len())` (full, for forward refs) + `capture_inventory_before(bytes,
  index)` (prior). No left-to-right phase can produce `full` at the ref's position.
- `.4.11` named-reference UNKNOWN-name inventory — **FORWARD-LOOKING; needs the new primitive; the ONLY real
  correctness FIX.** Forward/subroutine refs accept; backward-defined names must exist. `.4.11` has NO
  validator at all → the released parser genuinely ACCEPTS-INVALID undefined `\k<zzz>`/`(?P=zzz)`/`(?&zzz)`/
  `\g{zzz}` (ledger `REGEX-0098`). `.4.7`/`.4.8` are behavior-NEUTRAL migrations; `.4.11` closes an ACTUAL
  released-parser divergence ([[feedback_correctness_before_speed]]). Frozen acceptance spec = the `.3.22`
  oracle matrix.
- `.4.9` unbounded-lookbehind — **LOOKBEHIND-LENGTH ANALYSIS; a DIFFERENT new primitive.** The body's max
  match length must be finite (`(?<=a+)b`/`(?<=a*)b`/`(?<=a{2,})b` reject; `(?<=a{2})b` accept). Not an
  inventory question — a bounded-length property computed over the body's sub-AST. Owns
  `find_unbounded_quantified_lookbehind`.

**🔎 SURFACED — the one genuinely-novel architectural direction (level-5 general engine primitive).**
`.4.7` + `.4.11` share ONE missing parser-agnostic capability: a WHOLE-PATTERN semantic inventory available
to a forward reference. SOTA anchor: PCRE2 itself runs a compile-time PRE-PASS to count captures / resolve
forward names before the main compile — the standard way to admit legal forward refs. Two candidate forms:
  (A) a true TWO-PASS parse — a pre-pass collects all capture definitions into the store, the main pass gates
      refs against the now-complete store; or
  (B) a DEFERRED-OBLIGATION terminal phase — during the single pass a ref registers a "must resolve against
      the FINAL inventory" obligation; at parse-completion all obligations are checked against the completed
      store (a new terminal phase, far smaller than a full re-parse, reusing the existing single-pass store +
      scope tree, and mirroring the existing `Post`-effect model).
(B) looks minimal, general, and idiomatic to the current engine; a design + SOTA-citation slice will settle
A-vs-B BEFORE any engine code ([[feedback_research_grounded_sota_no_trial_and_revert]],
[[feedback_no_workarounds_fix_hierarchy]] — nothing lower in the hierarchy works: the Pre/Branch/Post
enumeration is the proof that no existing annotation/store/phase can see a forward capture). Building this ONE
primitive unlocks BOTH `.4.7` and `.4.11`. This is the parser-agnostic, all-parsers primitive the whole `.4`
capstone has been converging on — flagged here for the director per [[feedback_surface_insights_prominently]].

**Recommended sequence (structural/cheap first, correctness-weighted).** `.4.8` (single-pass, no primitive,
in the `.4.10` groove, shrinks the validator by one check) → design + build the whole-pattern-inventory
primitive (SOTA-cited A-vs-B decision slice, then the engine slice) → apply to `.4.11` FIRST (the real
accepts-invalid fix, `REGEX-0098`) then `.4.7` → `.4.9` (its own lookbehind-length primitive) →
blocked-descending class-range endpoints → DELETE the `find_invalid_char_class_construct` range-check →
final `.4` deletion → `.5`. No engine/grammar change this slice (PURE-DOCS scope + sharpen).

**✅ DIRECTOR ENDORSEMENT (2026-07-10, session #84 close).** On seeing this SURFACED primitive, the director
replied verbatim: *"Please do what you see fit to achieve sota, signoff level quality."* — i.e. the
whole-pattern-inventory primitive is AUTHORIZED to be built at signoff grade. Binding constraints carried
forward: (1) survey the literature FIRST and act on a citation + worked mapping, not trial-and-revert
([[feedback_research_grounded_sota_no_trial_and_revert]]); (2) it is a parser-AGNOSTIC, all-parsers primitive
([[feedback_ast_pipeline_parser_agnostic]], [[feedback_features_parser_agnostic_enable_all_parsers]]) —
never regex-name-gated; (3) correctness is the floor ([[feedback_correctness_before_speed]]); (4) the A-vs-B
choice is settled in a SOTA-cited DESIGN slice BEFORE any engine code, and the commit states which
fix-hierarchy level it lands at + why nothing lower works ([[feedback_no_workarounds_fix_hierarchy]]). The
director opened a fresh session immediately after (deliberate `/clear`); the `MEMORY.md` resume pointer +
this section are the handoff.

### REGEX-PCRE2-FIDELITY — WHOLE-PATTERN-INVENTORY PRIMITIVE: A-vs-B DECIDED + tree spun out (`PGEN-FPP-0001`, 2026-07-10 session #86, PURE-DOCS)

**What this slice did.** Executed the director-authorized SOTA-cited DESIGN slice for the SURFACED
whole-pattern-inventory primitive (the shared `.4.7`+`.4.11` unlock). Two tool-backed research streams
(engine ground-truth by `file:line`; SOTA literature survey with primary-source citations), then synthesis.
The primitive is spun out to its own GENERAL, parser-agnostic tree — **`FINAL-PHASE-PREDICATE`** (the RSVC/SCP
model: a general engine primitive gets its own tree + decision record, consumed by the regex leaves) — with
`.1` DESIGN DONE and `.2` BUILD pending. Full record: decision [[project_final_phase_deferred_predicate_primitive]].

**The primitive (chosen shape).** A fourth `@predicate` phase, **`phase: final`**: checked ONCE at
whole-input parse completion (`parse_full` success, `ast_based_generator.rs:1649-1663`) against the
now-complete store — so it sees facts emitted ANYWHERE, including LATER than the carrying rule (a legal
forward reference). Realized via DEFERRED OBLIGATIONS (resolve args at rule-commit → enqueue a transactional
obligation → terminal discharge at completion) = backpatching generalized to a semantic check.

**A-vs-B → B (single pass + deferred obligations, terminal discharge), not A (two-pass pre-scan).** The
SOTA discriminator (survey, primary sources): a pre-pass (PCRE2 `parse_regex`, .NET `CountCaptures`) is
warranted ONLY when an *irreversible pre-emission global aggregate* forces the count to be known before the
main pass (PCRE2 memory-sizing/opcode selection — `HACKING`: "full knowledge of group names and numbers
throughout"). A reference **validator** has no such dependency, so it defers — the LLVM `ForwardRefVals`→
`validateEndOfModule` / Dragon §6.7 backpatching / Rust-Roslyn late-binding pattern. B reuses PGEN's one
authoritative parse + the store that already survives to a clean whole-pattern state at completion, so the
check cannot diverge from the parse; A would force re-running the grammar's structural recognition. Theory
backbone: a forward reference is a non-L-attributed right-to-left dependency ⇒ provably not
single-left-to-right-evaluable (Dragon §5.2.3–5.2.4; Knuth 1968; Bochmann 1976, Algol-60 scope example) ⇒
defer-or-pre-scan is REQUIRED (fix-hierarchy tier-5 proof nothing lower works).

**Duality-clean (no `.4.8`-style break).** `phase: final` gates no branch during the single pass, so the
stimuli generator ignores it safely; each consumer keeps its existing conservative `@gen_predicate` draw
(references only already-emitted names ⇒ sound). NO new generation primitive — the trap that killed `.4.8`'s
fact-based A′ ([[project_gen_side_no_lacks_fact_branch_prune]]) does not recur here.

**Frozen BUILD SPEC + consumer mapping** live in the decision record and `docs/tasks/FINAL-PHASE-PREDICATE.md`.
Sequencing (updated): **`FINAL-PHASE-PREDICATE.2` (engine BUILD)** → apply to **`.4.11`** FIRST (the real
accepts-invalid fix, `REGEX-0098`) → **`.4.7`** → `.4.9` (its own lookbehind-length primitive) →
blocked-descending class-range endpoints → DELETE `find_invalid_char_class_construct` range-check → final
`.4` deletion → `.5`.

### REGEX-PCRE2-FIDELITY.4.8 — NESTED-CASE DISCOVERY + DESIGN (`PGEN-REGEX-PCRE2-0041`, 2026-07-10, session #85, tool-backed, PURE-DOCS)

**Trigger / the scope-changing finding.** The resume pointer + the `.4` PRIMITIVE-GAP SCOPING framed `.4.8`
as "the CHEAPEST residual — a single monotonic `@emit_fact` prefix-ended fact + a `@predicate lacks_fact`
gate, LEFT-CONTEXT single-pass, NO new primitive, in the `.4.10` groove." A tools-first read BEFORE any code
found that framing **INCOMPLETE**: it modelled only the FLAT top-level ordering and MISSED the NESTED-group
cases — which the released validator ALSO rejects, so behavior-neutrality REQUIRES the grammar to reject them
too. `.4.8` is therefore NOT cheap; it is a broad (grammar-only) migration touching every pattern-nesting
opener. Recorded here per [[feedback_be_alert_root_cause_fishy_immediately]] +
[[feedback_surface_insights_prominently]] (I should never let the director be the one to notice a plan/reality
gap).

**Method (toolbox-first, per [[feedback_systematically_use_debug_toolbox]]).** (1) Froze the authoritative
accept/reject spec with `pcre2test` 10.47 (one pattern per invocation, blank-line-separated). (2) Read the
engine's effect-application timing in `rust/src/ast_pipeline/ast_based_generator.rs` (branch-start inline
effects + rule-level `@emit_fact`). (3) Enumerated the residual validator (`find_invalid_verb_construct` +
`is_start_option_position`, `regex_compile_validation.rs:170`/`:310`) and every grammar rule with an inner
`pattern` (the nesting constructs).

**Frozen oracle matrix (the acceptance spec for the BUILD; `pcre2test` 10.47, verified string-by-string).**
- ACCEPT (start-option prefix at byte 0, contiguous, then the body): `(*CRLF)abc`, `(*UTF)abc`,
  `(*CRLF)(*LIMIT_MATCH=123)abc`, `(*CRLF)(*UTF)(?:x)`, `(*CRLF)(a)`, `(*CRLF)a|b`, `((*ACCEPT))` (ACCEPT is a
  VERB, not a start option — verbs are valid ANYWHERE incl. nested).
- REJECT err 160 — FLAT (a top-level non-start-option precedes it): `a(*CR)b`, `a(*LIMIT_HEAP=500)`,
  `(*FAIL)(*LIMIT_HEAP=5)a` (a verb ends the prefix), `(a)(*CRLF)` (after a group).
- REJECT err 160 — **NESTED (start option inside ANY group; the cases the scoping MISSED):** `((*CRLF)a)`
  (capture), `(?:(*CRLF)a)` (non-capture), `(?=(*CRLF)a)` (lookaround), `(*CRLF)((*LF)a)` (group after a start
  option, still in prefix), `(*sr:(*CRLF)a)` (script-run), `(*scs:(1)(*CRLF)a)` (scan-substring),
  `(?i:(*CRLF)a)` (scoped modifiers), `(?|(*CRLF)a)` (branch-reset). PCRE2 requires a start option at the very
  start of the ENTIRE pattern, before any other construct AND not nested at any depth.

**Decisive engine fact (WHY the nested case is hard).** `SemanticRuntimeDirective::EmitFact` is an EFFECT that
fires POST-body: rule-level `@emit_fact` after the whole rule, and "branch-start" inline effects (INLINE-ACTIONS.2)
ALSO fire after the winning branch's body (`ast_based_generator.rs:3885` — "fire here — after the winner's body
delta is replayed"). There is NO pre-body emit anywhere in the engine. So a fact CANNOT be set before descending
into a group's inner `pattern` — a naive post-body monotonic fact leaves a nested start option (e.g. the FIRST
inner piece of a first-position group `((*CRLF)a)`) seeing the fact still UNSET ⇒ wrongly ACCEPTED. The
released validator rejects it (`is_start_option_position` walks from byte 0 and requires every prior `(*…)` to
be a start option — a `(` group-opener at byte 0 fails immediately), so a post-body-only migration would be an
accepts-invalid REGRESSION vs the released parser. The fix must set the fact AT THE OPENER, before the inner body.

**Design (Option A — grammar-only, the `.4.10` `capture_open` open-marker idiom; NO new primitive; CHOSEN).**
A single monotonic fact (kind `regex_body_started`, name `body`) means "a non-start-option construct has begun /
the start-option prefix has ended." It is CORRECT to be monotonic: once ANY top-level non-SO piece OR any group
has been entered, NO start option is ever valid again (a group is itself a non-SO construct, and a start option
is never valid inside one) — so the fact never needs retraction (unlike `.4.10`'s scope, which must auto-unwind).
  1. **Gate.** Split the start options (`directive_limit_named | directive_option_named`) out of the shared
     `directive_verb_nonquant` into a dedicated `start_option_piece` gated
     `@predicate { name: lacks_fact, args: [regex_body_started, body], phase: pre }` + `!quantifier` (preserving
     the err-109 non-quantifiable shape), emitting NOTHING. The verbs/MARK stay in `directive_verb_nonquant`.
  2. **Top-level ordering.** Wrap the non-SO `piece` branches so they carry `@emit_fact {regex_body_started, body}`
     (post-body — fires after each top-level flat piece / group, blocking a LATER top-level start option:
     `a(*CR)b`, `(*FAIL)(*LIMIT_HEAP=5)a`, `(a)(*CRLF)`).
  3. **Nesting.** Add `@emit_fact {regex_body_started, body}` to EVERY pattern-nesting OPENER MARKER (fires
     right after the opener token, BEFORE the inner body — the proven `capture_open`/lookaround-open idiom), so a
     start option nested at the start of a first-position group is blocked. Openers: reuse the EXISTING markers
     `capture_open`, `named_group_open_angle`/`_quote`, `python_named_group_open`, and the 7 lookaround `*_open`
     + `alpha_lookaround_open` (add the emit alongside their current `@emit_fact regex_capture_group` /
     `@open_scope lookaround`); CREATE byte-identity-preserving open-markers (the `.4.10` split) for the ~8
     currently-INLINE nesting constructs: `noncapturing_group` `(?:`, `atomic_group` `(?>` and `(*atomic:`,
     `scan_substring_group` `(*scs:…`, `script_run_group` `(*sr:…`, `scoped_inline_modifiers` `(?flags:`,
     `branch_reset_group` `(?|`, and `conditional` `(?(…)` (nests via `conditional_branch = piece*`).
  4. **Deletion.** With the grammar fully owning the position rule, DELETE `find_invalid_verb_construct` +
     `is_start_option_position` + the now-exclusive helpers (`find_star_verb_end`, `is_pcre2_start_option_name`,
     `is_pcre2_verb_name`, `is_non_verb_star_group_name` — keeping any still shared), the `.3.1`/`.3.2`/`.4.10`
     standalone-deletion precedent. Re-baseline `duality_hunt_gate_contract_v0.json` same-commit (the
     `STIMULI-SIGNOFF.13.3` start-option-class duality pin this leaf owns).

**🔎 SURFACED — Option B (a future general elegance; NOT taken this slice).** A general **pre-body `@emit_fact`
phase** (an "on-enter" effect symmetric to the existing post-body effect) would let ONE `@emit_fact
{regex_body_started, body} phase:pre` on the `atom` rule cover ALL nesting in a single place (every group is an
`atom`; entering it emits before its inner body), collapsing Option A's ~19 emit sites + ~8 new markers to ~5
sites AND auto-covering any FUTURE nesting construct (Option A's residual maintainability footgun: a new nesting
construct added without the emit would silently accept an invalid nested start option). Option B is an ENGINE
change — it needs the SOTA-cited design slice + parse-harness combinator/semantic-suite coverage + the
interpreter mirror, and it MAY compose with the already-endorsed whole-pattern-inventory primitive
(`.4.7`/`.4.11`). **Recommendation:** land `.4.8` via Option A (correctness-first, behavior-neutral,
grammar-only, leaves the stable engine alone per [[feedback_prefer_grammar_leave_engine_alone]], stays within
the endorsed "no primitive for `.4.8`" plan), and mitigate the footgun with a coverage guard test (assert every
nesting construct sets `regex_body_started`). Option B is flagged for the director as a candidate to FOLD into
the endorsed engine-primitive workstream later, NOT to block `.4.8`. Per [[feedback_pinpoint_real_blocker_not_menu]]
this is a recommendation with a noted alternative, not a menu — the tools prove Option A is correct and
within-principle, so `.4.8` proceeds on Option A unless the director redirects.

**⟳ DECISION REOPENED — director challenge, session #85 (2026-07-10): the A-vs-B choice reconsidered on SPEED
+ long-term-elegance grounds.** The director pushed back on the reflexive Option-A recommendation ("why is a
3rd engine primitive bad? Option B feels the cleaner, most elegant long-term route — and which impacts runtime
speed?"). Three corrections came out of it, recorded so they survive `/clear`:
- **"3rd primitive" is NOT a real cost — the framing is RETRACTED.** A GENERAL, parser-agnostic engine
  capability that lets the EBNF cleanly express the language is an asset, not debt
  ([[feedback_features_parser_agnostic_enable_all_parsers]], [[feedback_prefer_grammar_leave_engine_alone]]'s
  "engine features that let the EBNF cleanly express the language ARE on the table"). What matters is
  generality + net-complexity reduction + parse-harness-suite verification — the count of primitives is
  irrelevant. A pre-body/on-entry effect (the missing symmetry: the engine has POST-body effects but NO
  pre-body ones) passes all three.
- **Option A above was UNDER-DESIGNED (a strawman at ~19 sites). The refined grammar design A′ is ~2 sites,
  no engine change, and future-proof.** A′: a single ZERO-WIDTH boundary marker in `concatenation`
  (`concatenation = start_option_piece* body_boundary regular_piece*`) that emits `regex_body_started` ONCE —
  after the leading start-option run, BEFORE the body. Because every group is a `regular_piece` entered AFTER
  that boundary, `regex_body_started` is ALREADY set when we descend into ANY group, so a nested start option
  is blocked with NO per-opener markers — and a FUTURE nesting construct is auto-covered (no footgun). Removes
  the per-opener emits (design step 3 above) AND the per-piece emits (step 2). ⚠️ VERIFY before landing: the
  two-spread AST shape `[$1**, $3**]` byte-identity (mixed spread caveat, [[feedback_annotation_no_mixed_spread]]),
  the empty-concatenation interaction (`piece+`→boundary must not make `concatenation` match empty and change
  `alternative = concatenation?`), and boundary rollback on backtrack.
- **SPEED is the decisive axis (RGX-0078 imminent), and it cuts ACROSS A/B, not between them — driven by EMIT
  + GATE FREQUENCY on the parse hot path (inside the backtracking loop), NOT grammar-vs-engine.** Naive-A emits
  per-piece O(chars); naive-B (on-entry emit on `atom`) emits per ATOM-ATTEMPT incl. backtracked ones + rollback
  (potentially WORSE); A′ emits O(concatenations) (naturally minimal, no engine change). The GATE must be
  `phase:post` (evaluate `lacks_fact` only AFTER a real `(*NAME)` start-option token matched — near-zero for
  normal patterns), NOT `phase:pre` (a store lookup on EVERY piece attempt — the design steps above wrongly say
  `phase:pre`; A′/B both should use `phase:post`). So a well-designed grammar approach (A′) is at least as fast
  as B and faster than naive-B; B is only speed-competitive if deliberately designed for zero-cost-when-unused +
  IDEMPOTENT (emit-if-absent) semantics. The actual delta MUST be MEASURED vs the RGX-0078 baseline, not asserted
  ([[project_uvm_memory_not_the_memo]] "profile, don't infer"; correctness AND speed are BOTH floors now,
  [[feedback_correctness_before_speed]] ⛔ hardening).
- **Refined recommendation (mine): land `.4.8` on A′** (correct + speed-safe + no engine risk, `phase:post`
  gate + single boundary emit), and pursue the general on-entry-effect primitive (B) as a DELIBERATELY
  SOTA-designed platform capability inside the engine-lifecycle workstream (with the whole-pattern-inventory
  primitive) — designed for zero-cost-when-unused + idempotent emit + measured — then OPTIONALLY migrate `.4.8`
  onto it. **Alternative (if the director commits to B as the direction):** build B first, land `.4.8` directly
  on it (no rework), under two non-negotiables — zero hot-path cost when unused, and a measured before/after
  parse-speed delta. **DECISION PENDING director direction (A′ vs commit-to-B-first).** No code touched.

**✅ DECISION (director, 2026-07-10, session #85): "let's go for A′ now + B later."** Land `.4.8` on the
refined grammar design A′ NOW (correct + speed-safe + no engine risk); pursue the general on-entry-effect
primitive (B) LATER as a deliberately SOTA-designed, zero-cost-when-unused + idempotent platform capability in
the engine-lifecycle workstream, then optionally migrate `.4.8` onto it. Additional oracle facts gathered this
session (`pcre2test` 10.47, for the A′ BUILD spec — the grammar restructure must honor ALL of these):
- `(*CRLF)` ALONE -> **ACCEPT** (a start-option-only pattern, no body, is valid) — so `concatenation` must admit
  a leading-start-option run with ZERO following body pieces.
- `(*CRLF)|a` -> ACCEPT, `(*CRLF)a|(*LF)b` -> REJECT, `a(*CRLF)|b` -> REJECT — start options bind to the very
  start of the WHOLE pattern, NEVER per-alternative (so the `regex_body_started` fact, set by the FIRST
  alternative's first non-SO piece, correctly blocks a start option in a later alternative — the monotonic
  global fact is exactly right here; a per-alternative scope would be WRONG).
- empty pattern -> ACCEPT (current parser) — so `alternative = concatenation?` must stay able to match empty;
  the restructured `concatenation` must NOT itself become empty-matchable (else the `?` shape drifts).
- **⚠️ AST-shape RISK (verify FIRST, toolbox): the two-spread `[$1**, $3**]` is UNPROVEN** — no grammar uses it;
  `return_annotation.ebnf:63` + `regex.ebnf:1512` note mixed-spread "doesn't flatten in the mixed position."
  So A′'s flat-piece-list output MUST be verified byte-identical in the scratch slot (1.3) BEFORE relying on it,
  OR structured to avoid two-spread (e.g. a helper rule that returns a single flat Sequence which the parent
  flattens with the PROVEN single `[$1**]`).
- **A′ concatenation structure (candidate, honors the above):**
  `concatenation = start_option_piece+ body_boundary regular_piece* -> <flat> | body_boundary regular_piece+ -> <flat>`
  — branch 1 = a leading start-option run (incl. start-option-only, zero following regular pieces) then the
  boundary emit then the rest; branch 2 = no start options, boundary emit first then >=1 regular piece. Both emit
  `regex_body_started` exactly once via the zero-width `body_boundary`; NEITHER matches empty (so
  `concatenation?` empty semantics are preserved); every group is a `regular_piece` entered AFTER the boundary,
  so nested start options are blocked with NO per-opener markers. Gate the extracted `start_option_piece` with
  `@predicate lacks_fact(regex_body_started, body) phase:post` (NOT `pre` — post evals only after a real
  `(*NAME)` start-option token matched, near-zero hot-path cost) + `!quantifier`. `<flat>` = the two-spread to
  verify/replace per the RISK note above.
- IMPLEMENTATION SEQUENCE (fresh session): (1) scratch-verify the flat-list annotation; (2) split start options
  out of `directive_verb_nonquant` -> `start_option_piece` (gated `phase:post`); (3) restructure `concatenation`
  + `body_boundary` emit; (4) regen regex parser, run the FROZEN oracle matrix (ACCEPT/FLAT-reject/NESTED-reject
  + these edge cases) via released `--parse` byte-identical; (5) cert seeds 0/7/42, `duality_hunt_gate`
  rebaseline (`STIMULI-SIGNOFF.13.3` pin), `regex_pcre2_compile_oracle_gate`, differential-equivalence +
  ast_shape gates; (6) DELETE `find_invalid_verb_construct` + `is_start_option_position` + now-exclusive helpers;
  (7) lockstep book/contract/ledger/schema + release bump. Behavior-NEUTRAL (released `--parse` verdict
  byte-identical before/after).

### REGEX-PCRE2-FIDELITY.4.8 — IMPLEMENTATION LOG + A′→A″ PIVOT (`PGEN-REGEX-PCRE2-0044`, 2026-07-10, session #86, tool-backed)

**Built A′ (fact-based) first, exactly per the frozen spec; interpreter-derisked it GREEN (parse), then the
cert surfaced a generation defect that forced a within-principle pivot to A″ (structural). Recorded here so the
pivot survives `/clear`.** Method was toolbox-first throughout ([[feedback_systematically_use_debug_toolbox]],
[[feedback_be_alert_root_cause_fishy_immediately]]).

- **STEP 1 (scratch slot).** The two-spread `[$1**, $3**]` is HARD-BROKEN at codegen: two `**` peel-helpers
  (`__pgen_peel_alternative`) land in one lexical scope → **E0428 duplicate-definition** (not the "silent nest"
  the spec feared — a compile error). Tool-proven in the scratch slot. FIX: a single mixed run keeps `**` in the
  one array position it flattens cleanly — `[$1*, $2**]` (shallow-spread the leading start-option run, which is
  flat objects; deep-spread the body `piece*`). Verified byte-identical.
- **STEPS 2–4 (A′ built + interpreter-derisked).** Built A′: epsilon `body_boundary = "" @emit_fact
  regex_body_started` in a 2-branch `concatenation`, `start_option_piece` gated `@predicate
  lacks_fact(regex_body_started, body) phase:post`. The certified interpreter (`interpret_parse`, byte-identical
  to the generated parser) proved, WITHOUT regen: (a) A′ is byte-identical to pristine on 60+ valid patterns
  (shape-neutral); (b) A′'s grammar-only verdict matches the FULL `pcre2test` 10.47 oracle — flat rejects,
  NESTED rejects inside capture/noncapture/lookaround/script-run/scan-substring/scoped-mod/branch-reset, and the
  GLOBAL-alternation cases (`(*CRLF)a|(*LF)b` reject, `(*CRLF)|a` accept); (c) pristine ACCEPTED all 14
  violations (the single-source hole). `lacks_fact` queries the GLOBAL `fact_index` (`any_with_name`,
  `semantic_runtime.rs:2987`), so one boundary emit is visible inside every nested scope — the nesting works.
- **THE DEFECT (cert, post-regen).** Cert `fully_certified=true UNKNOWN=0` BUT **`sample_parse_failures=65`**
  (was **0** pre-`.4.8`). The failing samples are the GENERATOR emitting start options in invalid positions
  (`||(*CR)…` in later alternatives, mid-pattern `…(*UTF)…`) which the A′ grammar now REJECTS — a
  generator↔parser DUALITY BREAK ([[feedback_be_alert_root_cause_fishy_immediately]]). ROOT CAUSE (WHY+WHERE):
  the stimuli generator's store-aware machinery (`stimuli_generator.rs`) supports `fact_count_at_least` (a
  count-PRUNE: backtrack if too FEW facts) and `has_fact` (a NAME-DRAW), but there is **NO `lacks_fact`
  branch-prune** (backtrack if a fact IS present). So `@gen_predicate lacks_fact` is inert for gating a branch —
  the generator freely picks `concatenation` branch 1 (`start_option_piece+`) in later/nested concatenations.
  Making A′'s generation clean would need a NEW gen-engine primitive (a `lacks_fact` prune) — an ENGINE change.
- **PIVOT → A″ (structural, grammar-only, CHOSEN).** Per the fix hierarchy (grammar before engine,
  [[feedback_no_workarounds_fix_hierarchy]], [[feedback_prefer_grammar_leave_engine_alone]]) and "one clean fix"
  ([[feedback_pinpoint_real_blocker_not_menu]]): make `start_option_piece` reachable at EXACTLY ONE place — the
  whole-pattern entry — via a distinguished entry chain (`regex = entry_alternation`; `entry_alternation =
  entry_alternative ("|" alternative)*`; `entry_alternative = entry_concatenation?`; `entry_concatenation =
  start_option_piece+ piece* -> [$1*, $2**] | piece+ -> [$1**]`). Every LATER alternative and every NESTED
  pattern routes through the SHARED `alternative`/`concatenation` (no start-option branch), so a start option
  has NO derivation off the entry prefix. The position rule is now PURELY STRUCTURAL: NO fact, NO predicate, NO
  `body_boundary`, NO gen-dual, and ZERO fact-queries on the parse hot path (so it is also the FASTEST option —
  aligns with the RGX-0078 speed priority). Generation is duality-clean BY CONSTRUCTION (no grammar derivation
  reaches a mis-positioned start option). A″ re-derisked GREEN on the same interpreter suite (byte-identical +
  full oracle). 265 rules (pristine 259 +6: entry_alternation/entry_alternative/entry_concatenation +
  start_option_piece/start_option_verb/start_option_body); lint 0/0.
- **🔎 SURFACED to the director (this is a mechanism change from the explicitly-named A′).** A″ keeps the
  DECISION's essence (grammar-only, byte-neutral, no engine change, correct+fast, land now) and is strictly
  cleaner (no semantic-runtime interaction), so it is a within-principle technical decision — but because the
  director named "A′ (fact-based)" specifically, the pivot is flagged in the resume pointer + the reply callout
  for explicit feedback ([[feedback_surface_insights_prominently]]). Option B (the general on-entry-effect
  primitive) is UNAFFECTED — still the future platform capability; note it would ALSO need the dual gen-side
  `lacks_fact`-prune primitive to make a fact-based approach generation-clean, which is itself a candidate
  general stimuli-generator capability for the engine-lifecycle workstream.

**ORACLE-VERIFICATION INVESTIGATION (session #86, tool-backed — a `274 vs 270` discrepancy caught before commit).**
While consuming the compile-oracle gate, the corpus probe reported `false_accept_total=270`, but the draft
`.4.8` docs (copied from the REGEX-0111 sibling entry) claimed "oracle byte-identical `2189/1867/274/48`".
Toolbox-first ([[feedback_systematically_use_debug_toolbox]], [[feedback_be_alert_root_cause_fishy_immediately]]),
built a throwaway grammar-verdict flip-diff (`interpret_parse` on the pristine grammar `git show HEAD:…` vs A″,
over all **283 corpus `(*`-containing patterns**, no regen). Findings:
- **Exactly ONE grammar-verdict diff:** `a(*CR)b` (pcre2 rejects err 160) — pristine grammar ACCEPTED (the
  single-source hole), A″ grammar REJECTS. Every other star pattern (and, by the entry-chain falling through to
  the shared `piece+`, every non-star pattern) is **grammar-identical** to pristine. So the A″ grammar change is
  minimal and exactly the intended target.
- **Released verdict for `a(*CR)b` is UNCHANGED** (⇒ `.4.8` is genuinely behavior-neutral / corpus-invisible):
  pristine-released = grammar-accept AND validator-`is_start_option_position`-REJECT (the `a` before `(*CR)`
  fails the byte-0 walk) = REJECT; A″-released = grammar-REJECT = REJECT. Both MATCH pcre2. The reject LAYER
  moved validator→grammar, the verdict did not.
- **The gate asserts BOUNDS, not the tuple.** `regex_pcre2_compile_oracle_lightweight_v0.env` pins
  `MIN_MATCH_TOTAL=1845`, `MAX_MISMATCH_TOTAL=344`, `MAX_FALSE_ACCEPT_TOTAL=299`, `MAX_FALSE_REJECT_TOTAL=48`.
  A″ (subset 2188, line-1340 excluded): `match=1870, mismatch=318, fa=270, fr=48` — all within bounds; `fr=48`
  is EXACTLY at the ceiling, confirming A″ adds **zero** new false-rejects (the one change is fa→match, the safe
  direction). Full 2189 adds line-1340 (a non-changed pcre2-accept match) ⇒ `~2189/1871/270/48`.
- **🔎 SURFACED (pre-existing doc-integrity drift, NOT caused by `.4.8`):** the exact tuple `2189/1867/274/48` is
  STALE — it appears verbatim in **17 doc locations** (every REGEX-0105…0111 release entry) but the live
  false-accept count is `270` (drifted 274→270 at an earlier release — a corpus cell the "byte-identical" copy
  never re-measured, likely REGEX-0110's collating fix). `.4.8`'s OWN entries are corrected to the accurate
  framing (the `a(*CR)b`-only neutrality + the gate's real bounds + the measured tuple); the 17 historical
  occurrences are logged as a follow-up doc-drift sweep (a separate leaf, not rewritten mid-`.4.8`).

**ACCEPTANCE CHECKLIST (`.4.8` / `PGEN-REGEX-PCRE2-0044`).**
- [x] **ROOT CAUSE (WHY + WHERE)** — WHY: a recognized `(*UTF)`/`(*CRLF)`/`(*LIMIT_HEAP=…)`-class start option was an UNGATED member of `directive_body_nonquantifiable` (matched by the `directive_verb_nonquant` piece branch, reachable in ANY piece position at ANY nesting depth), while the PCRE2 err-160 POSITION rule (a start option is valid only as a contiguous prefix at the very start of the WHOLE pattern) lived OUT-OF-BAND in `regex_compile_validation.rs::find_invalid_verb_construct` → `is_start_option_position` (a byte-0 walk) — invisible to generation ([[project_ebnf_is_single_source_of_truth]]). WHERE (tool signatures): the certified interpreter (`interpret_parse`, grammar-only) shows pristine ACCEPTS all 14 mis-positioned forms; a grammar-verdict flip-diff over ALL 283 corpus `(*` patterns isolates the change to EXACTLY ONE cell (`a(*CR)b`: pristine grammar `accepted=true` → A″ `accepted=false`); the released `--parse` 27-case oracle matrix matches `pcre2test` 10.47.
- [x] **ADDRESSED (verified)** — verdict matrix pre→post: EXACTLY the 14 mis-positioned forms (flat `a(*CR)b` `a(*LIMIT_HEAP=500)` `(*FAIL)(*LIMIT_HEAP=5)a` `(a)(*CRLF)`; nested `((*CRLF)a)` `(?:(*CRLF)a)` `(?=(*CRLF)a)` `(*CRLF)((*LF)a)` `(*sr:…` `(*scs:…` `(?i:…` `(?|…`; post-`|` `(*CRLF)a|(*LF)b` `a(*CRLF)|b`) flip grammar ACCEPT→REJECT; leading runs / `(*CRLF)` alone / empty / `(*CRLF)|a` / verbs-anywhere stay ACCEPT — permanent test `rust/tests/regex_start_option_position_grammar_migration.rs` 2/2, plus released `--parse` 27/27 all matching `pcre2test` 10.47 (behavior-neutral: the deleted `is_start_option_position` rejected the SAME set from byte 0).
- [x] **NO REGRESSION** — regex cert-coverage `total=265 UNKNOWN=0 fully_certified=true` at seeds 0/7/42 (259→265: +6 structural rules; `sample_parse_failures` 4/4/6 = the PRE-EXISTING generic "did not consume full input" generator over-approximation, tracked `.4.8.1`, NOT a `.4.8` regression — verdict+furthest-pos identical on the pre-`.4.8` grammar); `--lint-grammar` 0 errors (265 rules); `regex_pcre2_compile_oracle_gate` bounds SATISFIED — A″ = `2189/1871/270/48` (match 1871 ≥ 1845, false-accept 270 ≤ 299, false-reject 48 ≤ 48 at the ceiling ⇒ ZERO new false-rejects), **byte-identical to the pristine-HEAD gate result**. HOW MEASURED (the full DEBUG gate could NOT complete this session — the RGX-0078 line-1340 80-deep-nested-paren pattern super-exponentially backtracks, killed at 5h45m; ORTHOGONAL to `.4.8`, which adds only O(1) top-level): (a) a fresh captured probe on the 2188 NON-pathological patterns (line-1340 excluded) = `1870/270/48`; (b) line-1340 is a pcre2-accept, non-changed match (`a(*CR)b`-only flip-diff ⇒ A″ = pristine on it) whose verdict is captured in the pristine-HEAD 00:19 full-gate summary `2189/1871/270/48`; (a)+(b) ⇒ A″ full = `2189/1871/270/48`. The flip-diff proves the sole grammar change `a(*CR)b` is released-neutral. ⚠️ NB the historical `2189/1867/274/48` tuple is stale (pre-existing drift, `fa` 274→270); `parse_harness_equivalence_gate` regex byte-identical; `ast_shape_contract` regex aligned (inventory 232→236, accepted-AST byte-identical); `duality_hunt_gate` regex lanes rebaselined (start-option-prefix signature VANISHED; residual = the `.4.8.1` over-approximation); 36 `regex_compile_validation` unit tests green; `metadata_is_stable` + version-drift gate `1.1.102`/`1.1.104` match the `REGEX-0112` ledger row.

### REGEX-PCRE2-FIDELITY.4.7.a — NAMED scan-substring refs are GRAMMAR-owned (`PGEN-REGEX-PCRE2-0046`, session #89)

**What landed.** The NAMED scan-substring capture reference (`(*scs:(<name>))` / `(*scs:('name'))`) is now
grammar-owned — the **second consumer** of the `FINAL-PHASE-PREDICATE` whole-input deferred-obligation
primitive after `.4.11`. A `phase: final` `has_fact(regex_defined_capture_name, $name)` gate on
`scs_capture_name` checks the referenced name against the whole-pattern capture-name inventory (emitted by
`named_group` / `python_named_group`, `.4.11`'s parse-side `regex_defined_capture_name` fact); a name defined
NOWHERE rejects at parse completion (PCRE2 err 115), a legal FORWARD reference accepts. The validator's NAMED
branch (`validate_scan_substring_capture_refs`'s `<name>`/`'name'` handling) + `CaptureInventory.names` are
DELETED. Numeric refs stay validator-owned (`.4.7.b`/`.4.7.c`).

**The reshape + the tools-first reference-resolution finding (the non-obvious part).** `scs_capture_name`
was `name -> $1` (a BARE STRING). A parse `@predicate` arg resolves against the rule's PRODUCED STRUCTURE,
and — proven by an in-process interpreter probe on three candidate designs (`interpret_parse`, no regen) —
ONLY a shaped OBJECT-KEY reference resolves on a bare-string carrier at `phase: final`:
| design | gate arg | carrier | forward-ref `ua;da;` | undefined `ua;db;` |
|---|---|---|---|---|
| A | positional `$1` | `-> $1` | **REJECT (wrong)** | reject |
| B | `$text` | `-> $1` | **REJECT (wrong)** | reject |
| C | `$name` | `-> { name: $1 }` | **ACCEPT (correct)** | reject |
Positional `$N` and `$text` do NOT resolve as parse-`@predicate` args (`$text` routes to an object-key
"text" lookup that misses a bare string; positional `$N` is unresolved in a directive payload). So
`scs_capture_name` is reshaped `-> { name: $1 }` and gated with `$name` (the proven `.4.11` `$ref`/`$name`
shaped-field idiom). `scs_capture_name_ref`'s two branches extract `-> $2.name` (was `-> $2`) so the embedded
`captures` AST is byte-identical. Single-branch, so no `view: shaped` needed (the `.3` other-view fallback
covers the default `raw` view). The pre-existing `@gen_predicate has_fact(regex_capture_name, $text)` stays —
`$text` (matched name) is unchanged by the `->` reshape, so generation still draws only defined names
(duality-clean, the `.4.11` gen-source/parse-inventory two-fact split).

**Behavior-neutral migration ⇒ NO version bump** (surface- & conformance-neutral, the `.3.16`/`.3.17`
precedent): the validator already rejected undefined named scs refs; the grammar now does so PCRE2-faithfully
(the check moved, the accept/reject language is unchanged), and all accepted-scs ASTs are byte-identical.

**🔎 SURFACED in-slice (be-alert):** the `.4.7` regen brought the local `generated/regex_parser.rs` current
with the committed `.4.11` grammar and surfaced TWO STALE assertions in
`parser_registry::tests::regex_parseability_adapter_accepts_valid_regex_and_rejects_garbage`: standalone
`(?P=name)` / `(?&name)` (UNDEFINED name) were asserted `Some(true)` (pre-`.4.11` accepts-invalid) but
`.4.11` (REGEX-0098) correctly rejects them (`pcre2test` 10.47: err 115). Fixed to `Some(false)` — a stale
test `.4.11` left behind, NOT a `.4.7.a` regression (the rules `python_named_backreference` /
`named_subroutine_target` are untouched by this slice). Root-caused per [[feedback_be_alert_root_cause_fishy_immediately]].

**`.4.7.a` Acceptance Checklist** (root cause + addressed + no regression):
- [x] **REPRODUCE / ISSUE** — `parseability_probe --parse regex '(*scs:(<zzz>)a)' --profile pcre2` rejected
  via the VALIDATOR (`scan_substring capture list references an unknown named capture`) — load-bearing; the
  grammar ALONE accepted it. Goal: move the named check into the grammar (validator shrink toward `.4` capstone).
- [x] **ROOT CAUSE (WHY + WHERE)** — tools-first, NOT eyeballed: (1) rejection-LAYER probe (grammar
  vs validator message source) confirmed named scs = validator-owned; (2) the interpreter reference-resolution
  probe (table above) proved WHY a bare-string carrier needs the `-> { name }` reshape (WHERE: the parse-arg
  resolver `resolve_semantic_reference` → `resolve_named_semantic_reference`, an object-key lookup); (3) the
  post-fix rejection is the self-explaining engine line `whole-input predicate 'has_fact' not satisfied at
  parse completion (phase:final obligation unresolved: args [regex_defined_capture_name, zzz])`.
- [x] **FIX** — fix-hierarchy **level 1** (existing declarative mechanism): a `phase: final` `@predicate` +
  a return-annotation reshape in `grammars/regex.ebnf`; ZERO engine/Rust-runtime change. Validator's named
  branch + `names` field deleted (the migrated code). `generated/*` DERIVED (regenerated).
- [x] **ADDRESSED (verified)** — named scs oracle 8/8 vs `pcre2test` 10.47: undefined `(*scs:(<zzz>)a)` /
  `(*scs:('zzz')a)` / `(*scs:(<name>)a|b)` / `()()(*scs:(1,2,'XYZ'))` REJECT (at the grammar `phase:final`
  gate); defined forward+backward across all three definition delimiters (`(?<>`,`(?'`,`(?P<`) + mixed
  numeric+named ACCEPT. New grammar-layer test
  `parser_registry::tests::regex_scan_substring_named_refs_reject_at_the_grammar_layer_pcre2_faithfully`.
- [x] **NO REGRESSION** — AST byte-identical 4/4 named-scs accept cases (`--parse-dump-ast-pretty` before/after
  regen); regex cert-coverage `total=267 witness=258 UNKNOWN=0 fully_certified=true`, `sample_parse_failures`
  **0/1/1** at seeds 0/7/42 = the `.4.11` baseline (unchanged; the seed-7/42 residual is the tracked pre-existing
  `\Q\E`/verb generator sample); `--lint-grammar` 0 errors (267 rules); `ast_shape_contract` regex **aligned=4
  drift=0 regression_lock_failures=0** (inventory re-synced: 3 changed return-annotation entries, accepted-AST
  byte-identical); `parse_harness_equivalence` regex **CLEAN 57/57** (interpreter mirrors the new `phase:final`
  gate) + full gate `certified_grammars_are_byte_identical`; `parse_harness_semantic_gate` 36/36; 36
  `regex_compile_validation` unit tests + all `parser_registry` regex grammar-layer tests green (incl. the 2
  fixed stale `.4.11` assertions); clippy source clean.
- [x] **LOCKSTEP** — NO version bump (surface- & conformance-neutral); `ast_shape_contract` manifest
  `regex_v1.json` re-synced (3 entries); regex book (`scan-substring` / scs reference doc) + top-book + this
  tree + `MEMORY.md` + `CHANGES.md`. No ledger row (behavior-neutral migration; the accepts-invalid FIX is
  `.4.7.b`'s relative-zero).

### REGEX-PCRE2-FIDELITY.4.7.b — relative-zero scan-substring accepts-invalid FIX + the numeric-migration WITNESSING WALL (`PGEN-REGEX-PCRE2-0047`, session #89)

**What landed.** A behavior-CHANGING accepts-invalid FIX: a RELATIVE scan-substring capture reference of value
zero (`(*scs:(+0))` / `(*scs:(-0))`, incl. leading-zero `+00`/`-00`) now REJECTS (PCRE2 err 126). One validator
guard in `validate_scan_substring_capture_refs` (`is_relative_reference && reference == 0`). Release
`1.1.103`→`1.1.104`, contract `1.1.105`→`1.1.106`, schema `1`, ledger `REGEX-0113` (per the `.4.11` precedent).

**🧱 WHY it is ONLY the validator fix (the numeric grammar migration was built, oracle-proven, then REVERTED).**
The original `.4.7.b` plan (split `scs_capture_number` by sign → absolute `phase:final` + backward `-N`
`phase:post`, shrink the validator to `+N`) was fully implemented AND verified correct at the grammar layer
(interpreter de-risk 14/14; full-pipeline oracle matrix green). But cert-coverage then exposed a WITNESSING WALL
(tools-first — the cert `UNKNOWN` list named the exact rules):
- Sound scs generation is ABSOLUTE-ONLY. The `@gen_predicate fact_count_at_least(regex_capture_group, $value)`
  compiles to a VALUE-DRAW `IndexUpTo` that replaces `scs_capture_number`'s WHOLE render with an integer in
  `1..=prior_count` (`stimuli_generator.rs:1190`), never descending into the sign branches — empirically **0
  signed scs refs across 60k stimuli** (3 seeds × 20k).
- With whole-render-replace kept, the split's `scs_capture_number_backward`/`_forward` rules are never generated
  ⇒ cert `UNKNOWN=2 fully_certified=false` (**measured**, seed 0) — a `fully_certified` regress.
- With generation DESCENT + a leaf value-draw, backward `-N` becomes sound+witnessed, but the forward `+N` branch
  draws N≤prior_count and renders `+N` (unsound — no group at-or-after) ⇒ the validator rejects it ⇒ **spf spike**
  (the [[project_gen_side_no_lacks_fact_branch_prune]] class, the `.4.8` A′ trap). There is no branch-suppression
  mechanism (`@gen_weight`/`@gen_never` do not exist), and forward `+N` is the exact blocked class of `.4.7.c`
  (its validity is a SUFFIX/forward count the reach-driver cannot synthesize — `SelectedButFailed`).
- The migration also carries ZERO correctness value (the validator already handles absolute/backward correctly;
  only `+0`/`-0` is a real bug). So the split was reverted and the numeric migration ABSORBED into `.4.7.c` (to
  land as a UNIT when the forward/suffix-count primitive exists, which also unblocks sound forward witnessing).

**`.4.7.b` Acceptance Checklist** (root cause + addressed + no regression):
- [x] **REPRODUCE / ISSUE** — `parseability_probe --parse regex '()(*scs:(+0)a)'` → ACCEPT (full pipeline,
  parse+validator), where `pcre2test` 10.47 `/()(*scs:(+0)a)/` = err 126. Confirmed on the LIVE released pipeline
  (the probe applies the validator — proven by the `(0)`-rejects/`(+0)`-accepts asymmetry). Accepts-invalid set:
  `()(*scs:(+0)a)` `()(*scs:(-0)a)()` `()(*scs:(+00)a)` `()(*scs:(-00)a)()` `()(*scs:(1,+0)a)`.
- [x] **ROOT CAUSE (WHY + WHERE)** — tools-first (`pcre2test` oracle + validator source read + probe): WHERE =
  `rust/src/regex_compile_validation.rs::validate_scan_substring_capture_refs`; WHY = it resolves a relative ref
  by arithmetic (`+N`→prior_count+N, `-N`→prior_count+1-N) and rejects only a 0/out-of-range RESOLVED index, with
  NO guard on the relative reference VALUE being 0 — so `+0`→prior_count (≥1) and `-0`→prior_count+1 pass. PCRE2
  rejects a relative value of ZERO structurally (err 126) before resolution. The oracle also fixed the design:
  leading zeros are LEGAL and value-decoded (`(01)`=1 ACCEPT) but all-zero rejects (`(00)`=0), so the reject is
  VALUE-based, not digit-prefix-based.
- [x] **FIX** — fix-hierarchy **level 2** (the existing owner: the scs numeric refs are validator-owned and stay
  so until `.4.7.c`). One guard: `if is_relative_reference && reference == 0 { reject }`. NO grammar/generated
  change (grammar byte-unchanged ⇒ generated parser byte-unchanged). The grammar migration (fix-hierarchy level 1)
  was attempted first and is DEFERRED for the witnessing-wall reason above (documented, not a shortcut).
- [x] **ADDRESSED (verified)** — `pcre2test` 10.47 oracle (err 126); validator unit test
  `rejects_scan_substring_relative_zero_capture_refs`; full-pipeline registry pin
  `regex_scan_substring_relative_zero_refs_reject_pcre2_faithfully` (5 rejects incl. mixed list + leading-zeros,
  4 nonzero-accept regression guards, both profiles reject the zero forms). `+0`/`-0` flip ACCEPT→REJECT;
  absolute `0`/`00` unchanged (already rejected); every nonzero ref unchanged.
- [x] **NO REGRESSION** — grammar byte-unchanged ⇒ regex cert-coverage `total=267 witness=258 UNKNOWN=0
  fully_certified=true spf=0` at seeds 0/7/42 (fully_certified PRESERVED — the wall avoided); the `.4.7.a` named
  test + all scs validator/registry tests green; `regex_parser_pgen_rgx_0086_embedding_version_consts_match_ledger`
  + `regex_parser_integration_contract_metadata_is_stable` green (consts↔ledger↔manifest = 1.1.104/1.1.106);
  clippy source-strict exit 0. Equivalence/semantic/shape-contract gates structurally unaffected (grammar
  byte-identical).
- [x] **LOCKSTEP** — release+contract bump (embedding_api consts + JSON manifest `regex_parser_integration_contract_v1.json`
  + contract doc identity + a new "Release 1.1.104 / Contract 1.1.106 Highlights"); ledger `REGEX-0113`; regex
  book `changelog-index.md` + `compile-contract-validator.md` (scs numeric row: relative-zero fix, still
  validator-owned) + HTML regen; top-book `parser-families.md` handoff changelog; `.4.7.c` updated (absorbs the
  numeric migration + the witnessing wall); a durable decision note in `docs/decisions/`; this tree + `TASK_TREE.md`
  + `LIVE_ACHIEVEMENT_STATUS.md` + `CHANGES.md` + `DEVELOPMENT_NOTES.md` + `MEMORY.md`.

### REGEX-PCRE2-FIDELITY.4.5 — TOOLS-FIRST INVESTIGATION (`PGEN-REGEX-PCRE2-0030`, 2026-07-09, session #77, PURE-DOCS)

**Method (toolbox-first, per [[feedback_systematically_use_debug_toolbox]]).** Built the authoritative
accept/reject spec with `pcre2test` 10.47 (`/PATTERN/utf`, one pattern per invocation — pcre2test
consumes subsequent `/…/` lines as SUBJECT data unless blank-separated), then ran the SAME matrix through
the released `parseability_probe --parse regex` (grammar + `validate_regex_compile_contract` post-check,
the shipped behavior) and diffed. Then read the validator (`regex_compile_validation.rs`
`find_invalid_char_class_construct`/`scan_char_class`/`read_class_atom`/`class_escape_literal_codepoint`/
`read_substantive_class_atom`) + the grammar (`grammars/regex.ebnf` `class_range`:705 / `class_atom`:844).

**Frozen oracle matrix (the acceptance spec for the BUILD).**
- err 150 "invalid range" — a NONLITERAL range endpoint (either side): `[\d-x]` `[\s-x]` `[\w-x]` `[\D-x]`
  `[\p{Lu}-x]` `[[:alpha:]-z]` `[a-\d]` `[a-\p{Lu}]` `[x-[:alpha:]]` `[a-[:digit:]]` `[a-[.-.]]`.
- err 108 "range out of order" — both endpoints literal, left>right after DECODE: `[z-a]` `[b-a]`
  `[\x{100}-z]` `[\x41-\x30]` `[\x{7a}-\x{61}]` `[\132-\101]` `[\cz-\ca]` `[\e-\a]` `[a-[b]]` `[z-[a]]` `[a-[]`.
- ACCEPT — literal-dash carve-outs `[a-]` `[-a]` `[a\-z]` `[\d-]` `[\d\-x]` `[--/]` `[!--]`; ascending
  decoded ranges `[a-z]` `[\x30-\x41]` `[\x{61}-\x{7a}]` `[\101-\132]` `[\ca-\cz]` `[\a-\e]`; equal `[a-a]`;
  and `[!-[]` (`!`=33 .. `[`=91 ascending — a `-[` range that is VALID).

**Parser-vs-oracle diff — ONE divergence class, everything else already correct.**
- non-`[` NONLITERAL (`[\d-x]`, `[a-\d]`, `[[:alpha:]-z]`, …): grammar ACCEPTS (splits into separate
  members — `class_atom`:844 excludes `\d`/`\p`/posix so no `class_range` forms; the bare mid-class `-`
  parses as a literal member), validator REJECTS (`ClassAtomKind::NonLiteral` endpoint). Probe=R = oracle. ✓
- non-`[` DESCENDING (`[z-a]`, `[\x41-\x30]`, `[\cz-\ca]`, …): grammar ACCEPTS (structural range),
  validator REJECTS via `class_escape_literal_codepoint` decode + `left>right`. Probe=R = oracle. ✓
- `-[…` RIGHT endpoint in a NORMAL class (`[x-[:alpha:]]` `[a-[:digit:]]` `[a-[.-.]]` `[a-[b]]` `[z-[a]]`
  `[a-[]`): **grammar ACCEPTS *and* validator ACCEPTS → probe=A, oracle=R → ACCEPTS-INVALID DIVERGENCE.**

**🔎 ROOT CAUSE of the divergence (WHY + WHERE, per [[feedback_why_and_where_before_solution]]).**
`dash_starts_alt_extended_class_operator` (`regex_compile_validation.rs:716`) returns true whenever
`bytes[dash+1]=='['`, and both range branches (`:436` and `:488-489`) use it to SKIP range detection —
treating `-` as a literal and the following `[…` as a fresh member. That guard exists for the
extended-class set-difference operator `-[…]`, but `scan_char_class` is dispatched ONLY for NORMAL classes
(`find_invalid_char_class_construct:345` gates on `!is_extended_class_start`), where there is NO
set-difference operator — so `atom-[…` is ALWAYS a range whose right endpoint begins at `[` (a posix/
collating token ⇒ nonliteral ⇒ err 150; else a literal `[`=91 ⇒ ascending accept `[!-[]` / descending
err 108 `[a-[b]]`). The grammar mirrors the miss (bare mid-class `-` before `[` parses as a literal
member). `[!-[]` masks the bug (both accept, same verdict). This is a genuine RELEASED-parser
accepts-invalid gap (a `.4.11`-style divergence, ledger `REGEX-00xx` candidate — formalize in the BUILD).

**Reframed BUILD decomposition (three sub-slices; fix hierarchy [[feedback_no_workarounds_fix_hierarchy]]).**
1. `.4.5.a` — the `-[` accepts-invalid fix (NEW divergence; fixes BOTH the grammar range rule AND retires
   the mis-applied validator guard). Structural. Land first (a real correctness gap, and cheapest).
2. `.4.5.b` — migrate non-`[` NONLITERAL-endpoint reject into the grammar (grammar must REJECT `\d-x`
   which it currently splits into members — the hard PEG-ordering: a mid-class `-` after a nonliteral atom
   before a rangeable atom must fail, without breaking the `[a-]`/`[-a]`/`[\d-]` literal-dash carve-outs).
3. `.4.5.c` — migrate non-`[` DESCENDING reject into the grammar. Needs the pre-authorized `codepoint`-
   coercion WIDENING of the `value_compare` primitive (decode `\x{}`/`\NNN`/`\cX`/`\a`/`\e`/char to a
   Unicode scalar, then `le`) — the engine tier, done with fullest care ([[feedback_correctness_before_speed]]).
   Only after `.4.5.b`+`.4.5.c` land can the `find_invalid_char_class_construct` range checks be deleted.

Unicode-only note ([[feedback_rgx_unicode_only_8bit_test_divergence]]): `[\x{100}-z]` etc. tested with
`utf` for the RGX code-point-faithful reading; the local `pcre2test` is 8-bit — width-only negatives stay
consumer-owned. No code/grammar change this slice (investigation + reframe only).

### REGEX-PCRE2-FIDELITY.4.5.a — the `-[` / `-||` accepts-invalid class-range fix (`PGEN-REGEX-PCRE2-0031`, session #78)

**BUILD-time refinement of the investigation's root cause (tools-first, AST-dump-proven).** The `.4.5`
INVESTIGATION note said the grammar "mirrors the miss (bare mid-class `-` before `[` parses as a literal
member)". `--parse-dump-ast-pretty` DISPROVED that: the grammar ALREADY forms a `class_range` for every
`-[` case — `[a-[b]]` → `{class_range, start:"a", end:"["}`, `[!-[]` → `start:"!", end:"["`,
`[x-[:alpha:]]` → `start:"x", end:"["` (the `[` is a `class_safe_special` literal `0x5B`; the trailing
`:alpha:]` are separate literal members). So the divergence is **ENTIRELY the out-of-band validator**, not
the grammar. The grammar's `@validate: ord($1) <= ord($5)` on `class_range` is not parse-gating (proven:
`[a-[b]]` = `a`(97) > `[`(91) accepts), so the class-range descending/nonliteral reject is validator-owned
for ALL cases — `.4.5.a` corrects that validator, keeping the family uniformly validator-owned until the
wholesale grammar migration in `.4.5.b`/`.4.5.c`.

**🔎 ROOT CAUSE (WHY + WHERE).** `scan_char_class` (`regex_compile_validation.rs`) has two range-detection
branches (the `previous_atom` branch + the after-`left_atom` branch). Both gated range detection on
`!dash_starts_alt_extended_class_operator(bytes, dash)`, which returns true when the char after `-` is `[`
or `||` — a guard for PCRE2's ALTERNATE extended-class syntax `(?[...])`. But `scan_char_class` is
dispatched ONLY for NORMAL classes (`find_invalid_char_class_construct` gates `b'[' if !is_extended_class_start`,
and `is_extended_class_start` = preceded by `(?`), where `-[` / `-||` is ALWAYS a range. So the guard was
UNCONDITIONALLY mis-applied: it made the validator treat `-` as a literal and skip the range check, so
every `-[` / `-||` range was accepts-invalid. Additionally `read_class_atom` classifies any bare `[` as
`Literal(0x5B)` — it did not recognize a `[:..:]` / `[...]` / `[=..=]` bracket token as a NON-LITERAL
range endpoint.

**Extended oracle matrix (`pcre2test` 10.47, the acceptance spec — the frozen `.4.5` matrix PLUS the
ascending-left nonliteral + `-||` + `\d-[` variants the BUILD surfaced).**
- err 150 (nonliteral endpoint, REGARDLESS of order): `[x-[:alpha:]]` `[a-[:digit:]]` `[a-[.-.]]`
  `[!-[:alpha:]]` `[!-[.a.]]` `[!-[=a=]]` `[a-[=a=]]` `[\d-[z]]` `[\d-||z]` `[\w-[a]]` (a nonliteral
  LEFT or a bracket-token RIGHT ⇒ invalid range).
- err 108 (literal descending): `[a-[b]]` `[z-[a]]` `[a-[]` `[a-[` `[~-||]` `[}-||]` (`[`=0x5B / `|`=0x7C
  literal endpoint, left > right).
- ACCEPT (ascending literal `-[`/`-||` + literal-dash carve-outs): `[!-[]` (the MASKING case: `!`(33) <
  `[`(91)) `[+-[]` `[Z-[]` `[[-a]` `[--[]` `[a-||b]` `[|-||]` `[a-]` `[-a]` `[--/]` `[!--/]`.

**FIX (validator tier — corrects an existing mis-scoped check; fix-hierarchy justification).** The whole
class-range validity family (`.4.5.b` non-`[` nonliteral, `.4.5.c` descending) is validator-owned, its
grammar migration deliberately deferred because it needs the hard PEG-ordering (`.4.5.b`) + the
`value_compare` codepoint-coercion WIDENING (`.4.5.c`) primitives. `.4.5.a` is the cheap structural
correctness fix sequenced first (per the investigation): (1) remove `!dash_starts_alt_extended_class_operator(...)`
from BOTH range branches and delete the now-unused function; (2) add `scan_class_bracket_token` (recognizes
`[:..:]`/`[...]`/`[=..=]` by scanning to the first `:]`/`.]`/`=]`) and check it FIRST in
`read_substantive_class_atom` (the range right-endpoint reader, used only at the 2 range branches) so a
bracket-token right endpoint classifies `NonLiteral` → err 150. A bare `[` still reads `Literal(0x5B)` and
orders normally. No grammar / codegen / generated-parser change ⇒ the released parser is byte-identical
except for the validator; cert rule-count, ast_shape inventory, and duality signatures are unchanged.

**Behavior change (NOT a neutral migration — unlike `.4.4`/`.4.6`).** This flips real accepts-invalid inputs
to reject (the released parser+validator BOTH accepted them before). `[!-[]` and the ascending/carve-out
controls stay ACCEPT. A stale test `allows_alt_extended_class_dash_operators_after_shorthand_escape`
(`[\d-[z]]`/`[\d-||z]` "should not be treated as a range", added way back at `8ed45af9` regex `1.1.27`) had
LOCKED IN the bug — `pcre2test` 10.47 rejects both err 150 — so it was DELETED and the cases re-pinned as
rejects.

**🔎 NEW divergence surfaced (out of `.4.5.a` scope, tracked as `.4.12`).** STANDALONE collating `[[.a.]]`
and equivalence `[[=a=]]` tokens (used as plain class MEMBERS, not range endpoints) → `pcre2test` 10.47
err 113 "POSIX collating elements are not supported"; the released parser ACCEPTS both. Distinct class
(err 113 vs the range err 150/108); needs its own recognizer. `.4.5.a` deliberately did not fix it
(surgical one-defect scope).

#### `.4.5.a` Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — released `parseability_probe --parse regex --profile pcre2` (11:22 `1.1.94`
  build): `[a-[b]]` `[z-[a]]` `[x-[:alpha:]]` `[a-[:digit:]]` `[a-[.-.]]` `[!-[:alpha:]]` `[!-[.a.]]`
  `[!-[=a=]]` `[\d-[z]]` `[\d-||z]` `[~-||]` `[}-||]` all "parse_full passed" (grammar AND validator ACCEPT)
  where `pcre2test` 10.47 `/PATTERN/utf` rejects them (err 150 nonliteral / err 108 descending) ⇒ a genuine
  released-parser accepts-invalid. `[a-[` already rejected (unterminated-class, same verdict). `[!-[]`
  (masking case) accepts on both.
- [x] **ROOT CAUSE (WHY + WHERE)** — `regex_compile_validation.rs::scan_char_class` two range branches gated
  on `!dash_starts_alt_extended_class_operator(bytes, dash)` (returns true on `-[`/`-||`), a guard for the
  ALTERNATE extended-class syntax `(?[...])`; but `scan_char_class` runs ONLY on NORMAL classes
  (`find_invalid_char_class_construct` gates `!is_extended_class_start`), so it was unconditionally mis-scoped.
  `read_class_atom` also classified any bare `[` as `Literal(0x5B)`, never recognizing a `[:..:]`/`[...]`/`[=..=]`
  token as a NON-LITERAL endpoint. `--parse-dump-ast-pretty` DISPROVED the investigation's "grammar splits `-`
  into a literal member" note — the grammar forms `class_range` correctly; the divergence is ENTIRELY the validator.
- [x] **FIX** — VALIDATOR tier (fix-hierarchy: corrects an existing mis-scoped check; the class-range validity
  FAMILY stays validator-owned pending the deferred `.4.5.b`/`.4.5.c` grammar migration, which need
  PEG-ordering + `value_compare` codepoint-widening — `.4.5.a` is the cheap structural fix sequenced first).
  Removed `!dash_starts_alt_extended_class_operator(...)` from both range branches + deleted the function;
  added `scan_class_bracket_token` (recognizes `[:..:]`/`[...]`/`[=..=]` by scanning to the first
  `:]`/`.]`/`=]`) checked first in `read_substantive_class_atom` → a bracket-token right endpoint classifies
  NON-LITERAL. A bare `[` still reads `Literal(0x5B)`. NO grammar/codegen/generated-parser change.
- [x] **ADDRESSED (verified)** — every frozen+extended-matrix cell flips to the `pcre2test` 10.47 verdict:
  the 15 accepts-invalid now REJECT (`E_PARSE_FAILURE`), the 11 controls (`[!-[]` `[+-[]` `[Z-[]` `[[-a]`
  `[--[]` `[a-||b]` `[|-||]` `[a-]` `[-a]` `[--/]` `[a-z]`) stay ACCEPT. New pins green:
  `regex_class_range_bracket_endpoint_rejects_pcre2_faithfully` (13 reject + 10 accept + 3 relaxed) +
  validator unit test `rejects_bracket_token_class_range_endpoints`. The stale
  `allows_alt_extended_class_dash_operators_after_shorthand_escape` (from `1.1.27`) DELETED — it had locked
  in the bug.
- [x] **NO REGRESSION** — `regex_pcre2_compile_oracle_gate` NET IMPROVEMENT, true-measured `2189/1867/274/48`
  (was `2189/1858/285/46`): ~11 DEFAULT-mode `-[`/`-[.`/`-[=` accepts-invalid corpus cells now correctly
  reject (false_accept 285→274), false-reject 46→48 for the 2 `alt_extended_class`-modifier cells
  `[\d-[z]]`/`[\d-||z]` (PCRE2_ALT_EXTENDED_CLASS — a NON-default mode PGEN does not model; same documented
  divergence class as `[A--B]`); env baseline v11→v12. regex cert-coverage `239/239 UNKNOWN=0
  fully_certified=true spf=0` at seeds 0/7/42 (UNCHANGED); `--lint-grammar` 0 errors (239 rules);
  `duality_hunt_gate` 9 lanes NO new/vanished signature (validator-only); `parse_harness_equivalence_gate`
  regex byte-identical; `ast_shape_contract` inventory 221 UNCHANGED; dual `--lib` suite green (my 2 pins
  pass; only the 2 version-drift gates flipped, satisfied by the `REGEX-0105` ledger row). cert rule-count,
  ast_shape, and duality all unchanged because there is NO grammar/generated change.
- [x] **LOCKSTEP** — `regex_compile_validation.rs` (fix + test); `parser_registry.rs` pin; `embedding_api.rs`
  consts `1.1.95`/`1.1.97`; `regex_parser_integration_contract_v1.json`; `regex_pcre2_compile_oracle_lightweight_v0.env`
  (v12, ratchet 48); ledger `REGEX-0105`; contract `1.1.95`/`1.1.97` Highlights + Identity; regex book
  `compile-contract-validator.md` + `changelog-index.md` + tracked HTML; top book `parser-families.md`;
  `CHANGES.md`; `DEVELOPMENT_NOTES.md`; `LIVE_ACHIEVEMENT_STATUS.md`; `MEMORY.md`; `docs/TASK_TREE.md`;
  the new `.4.12` leaf (standalone collating/equivalence divergence).

### REGEX-PCRE2-FIDELITY.4.5.a.1 — the `\v` / `\V` accepts-invalid class-range fix (`PGEN-REGEX-PCRE2-0032`, session #79)

**Surfaced by the `.4.5.b` tools-first investigation (per [[feedback_be_alert_root_cause_fishy_immediately]]).** While mapping
the COMPLETE nonliteral-endpoint set for the planned `.4.5.b` grammar migration, the oracle sweep (`pcre2test`
10.47, `[\L-~]` for every candidate escape letter) found the nonliteral class-escape set is
`\d \D \h \H \s \S \v \V \w \W` + `\p{}` `\P{}` — but the released parser handled only `\d \D \h \H \s \S \w \W \p \P`
correctly. `\v`/`\V` (the VERTICAL-whitespace shorthands — the analogs of `\h`/`\H`) were treated as LITERAL
range endpoints, so `[\v-x]`, `[\V-x]`, `[a-\v]` were ACCEPTED where PCRE2 rejects them err 150. A genuine
released-parser accepts-invalid gap (not a documented divergence — ledger `REGEX-0106`).

**🔎 ROOT CAUSE (WHY + WHERE) — a two-halved bug, tool-backed.**
- **Validator omission** (`regex_compile_validation.rs:648-654`). `is_nonliteral_class_escape` enumerates
  `matches!(next, b'd'|b'D'|b'h'|b'H'|b's'|b'S'|b'w'|b'W') || matches!(next, b'p'|b'P') && …` — it MISSES `v`/`V`.
  So a `\v`/`\V` range endpoint falls through to `class_escape_literal_codepoint` (`:635` `_ => next as u32`),
  decoding `\v`→`118`, `\V`→`86` (literal). `[\v-x]` = `118..120` ascending → ACCEPT; `[\V-x]` = `86..120` →
  ACCEPT; `[a-\v]` = `97..118` → ACCEPT. (`[a-\V]` = `97..86` descending → err 108 reject — the released reject
  was RIGHT verdict, WRONG reason.)
- **Grammar mis-classification** (`grammars/regex.ebnf:878-879`). `v`/`V` were in
  `class_range_literal_escape_letter_strict`, so `\v`/`\V` form a `class_range` endpoint (via
  `class_range_escape` → `class_range_simple_escape`). This both feeds the validator's mis-read AND lets the
  store-aware generator EMIT `\v`-ranges. (The already-correct `\h`/`\H` are NOT in this set — they are members
  only, via `class_simple_escape` — which is why they are handled right.) `--parse-dump-ast` on `[\v-x]` was
  blocked by the validator; the classification was read directly from the two rule sets + the oracle.

**Frozen oracle matrix (`pcre2test` 10.47, `/PATTERN/utf`, the acceptance spec).**
- err 150 (nonliteral `\v`/`\V` endpoint): `[\v-x]` `[\V-x]` `[a-\v]` `[a-\V]` `[\v-\v]` `[\V-\v]` `[\d-\v]`
  `[\v-\d]` `[\v-\x7f]` (a `\v`/`\V` on EITHER side ⇒ invalid range).
- ACCEPT (must stay): `[\v]` `[\V]` `[a\vb]` (valid MEMBERS — vertical-whitespace class); `[\x0b-\x0c]`
  `[\013-\014]` (codepoint 11-12 via HEX/OCTAL = LITERAL endpoints ⇒ valid range — the fix must NOT touch
  the hex/octal path, only the range-LETTER path); `[\h-x]` `[\d-x]` stay REJECT (unchanged).

**FIX (grammar + validator tier — makes `\v`/`\V` byte-identical in structure to the already-correct `\h`/`\H`;
fix-hierarchy justification).** The class-range validity FAMILY stays validator-owned pending the deferred
`.4.5.b`/`.4.5.c` grammar migration; `\v`/`\V` are corrected to MATCH the family's already-correct members:
(1) GRAMMAR — drop `'V'` (`:878`) and `'v'` (`:879`) from `class_range_literal_escape_letter_strict` so `\v`/`\V`
are no longer range endpoints (identical to `\h`/`\H`); the member path (`class_simple_escape`, `v`/`V` unguarded)
is untouched so `[\v]`/`[\V]` stay valid, and the generator can no longer emit `\v`-ranges (duality-safe — see
NO REGRESSION); (2) VALIDATOR — add `b'v'|b'V'` to `is_nonliteral_class_escape` so a `\v`/`\V` range endpoint
classifies `NonLiteral` → err 150. No change to hex/octal/control (`\x0b`/`\013` stay literal, valid). The
relaxed profile is unaffected (`v`/`V` were never in `class_range_literal_escape_letter_relaxed`; the validator
rejects nonliteral ranges in BOTH profiles, as it already does for `\h`/`\d`).

**Behavior change (NOT a neutral migration).** Flips `[\v-x]` `[\V-x]` `[a-\v]` from accepts-invalid to reject.
`[a-\V]` stays reject (verdict unchanged; now err 150 nonliteral not err 108 descending). Members + hex/octal
ranges + all other cells stay byte-identical.

**Why this is `.4.5.a.1`, not folded into `.4.5.b` (scope decision).** `.4.5.b` is the (harder) validator→grammar
MIGRATION of the whole nonliteral family via a negative-lookahead PEG. `\v`/`\V` is a CORRECTNESS bug (accepts-invalid)
in the SAME family. Per correctness-before-migration ([[feedback_correctness_before_speed]]) and one-defect-per-commit,
it lands FIRST as a surgical `.4.5.a`-style fix (making an inconsistent case consistent with `\h`/`\H`), which also
DE-RISKS `.4.5.b` — the migration now covers a uniformly-correct nonliteral set. A validator-only fix was rejected:
it would leave the generator emitting `\v`-ranges (grammar still forms them) that the corrected validator rejects →
a NEW duality break; the grammar half (stop generation) is required for duality-safety.

#### `.4.5.a.1` Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — released `parseability_probe --parse regex --profile pcre2` (11:22 `1.1.95` build):
  `[\v-x]` `[\V-x]` `[a-\v]` all "parse_full passed" (grammar AND validator ACCEPT) where `pcre2test` 10.47
  `/PATTERN/utf` rejects them err 150 "invalid range in character class" ⇒ a genuine released-parser
  accepts-invalid. `[\h-x]`/`[\d-x]` already REJECT (the correct sibling behavior); `[\v]`/`[\V]` members
  ACCEPT (must stay). Confirmed on `--profile relaxed` too (`[\v-x]`/`[\V-x]` ACCEPT, where `[\h-x]`/`[\d-x]`
  already REJECT — the validator's nonliteral check is not a relaxed opt-out).
- [x] **ROOT CAUSE (WHY + WHERE)** — two-halved: (validator) `regex_compile_validation.rs:652`
  `is_nonliteral_class_escape` omits `v`/`V` from the nonliteral shorthand set, so `\v`/`\V` decode to literal
  `118`/`86` in `class_escape_literal_codepoint` (`:635`); (grammar) `regex.ebnf:878-879`
  `class_range_literal_escape_letter_strict` includes `V`/`v`, so `\v`/`\V` form a `class_range` endpoint AND
  the generator can emit `\v`-ranges. Oracle sweep `[\L-~]` over the range-letter set isolated `\v`/`\V` (err 150)
  from the distinct `.3.11`-owned invalid-escape letters (`\I \J \M \O \T \Y …`, err 107, invalid EVERYWHERE).
- [x] **FIX** — GRAMMAR + VALIDATOR tier (make `\v`/`\V` consistent with `\h`/`\H`): drop `'V'`/`'v'` from
  `class_range_literal_escape_letter_strict`; add `b'v'|b'V'` to `is_nonliteral_class_escape`. Member path and
  hex/octal/control decode untouched. No new rule / no new primitive.
- [x] **ADDRESSED (verified)** — every matrix cell flips to the `pcre2test` 10.47 verdict: the 3 accepts-invalid
  `[\v-x]` `[\V-x]` `[a-\v]` now REJECT (`E_PARSE_FAILURE` from the validator's err-150 path); `[a-\V]` stays
  REJECT; the members `[\v]` `[\V]` `[a\vb]` and the hex/octal ranges `[\x0b-\x0c]` `[\013-\014]` stay ACCEPT;
  `[\h-x]`/`[\d-x]` unchanged. New validator unit pin `rejects_vertical_whitespace_shorthand_class_range_endpoints`.
- [x] **NO REGRESSION** — `regex_pcre2_compile_oracle_gate` NET IMPROVEMENT (false_accept drops by the
  default-mode `\v`/`\V` accepts-invalid corpus cells; env baseline re-pinned). regex cert-coverage
  `239/239 UNKNOWN=0 fully_certified=true spf=0` at seeds 0/7/42 (UNCHANGED — the rule count and reachability
  are unaffected; `class_range_literal_escape_letter_strict` still witnessed via its remaining terminals);
  `--lint-grammar` 0 errors (239 rules); `duality_hunt_gate` 9 lanes NO new/vanished signature (the generator
  no longer emits `\v`-ranges — the range endpoint was removed — so it now shares `\h`/`\H`'s generation profile,
  and the oracle verdict on `[\v-x]` was already reject before this change surfaced it); `parse_harness_equivalence_gate`
  regex byte-identical (interpreter tracks the regenerated grammar); `ast_shape_contract` inventory 221 UNCHANGED
  (no `\v`/`\V` range samples pinned); dual `--lib` suite green (new pin passes; the 2 version-drift gates
  satisfied by the `REGEX-0106` ledger row).
- [x] **LOCKSTEP** — `grammars/regex.ebnf` (range-letter set); `regex_compile_validation.rs` (fix + test);
  regenerated `generated/regex_parser.rs`; `parser_registry.rs` version pin; `embedding_api.rs` consts
  `1.1.96`/`1.1.98`; `regex_parser_integration_contract_v1.json`; `regex_pcre2_compile_oracle_lightweight_v0.env`
  (re-baseline); ledger `REGEX-0106`; contract `1.1.96`/`1.1.98` Highlights + Identity; regex book
  `compile-contract-validator.md` + `changelog-index.md` + tracked HTML; top book `parser-families.md`;
  `CHANGES.md`; `DEVELOPMENT_NOTES.md`; `LIVE_ACHIEVEMENT_STATUS.md`; `MEMORY.md`; `docs/TASK_TREE.md`.

### REGEX-PCRE2-FIDELITY.4.5.b — non-`[` NONLITERAL class-range reject is GRAMMAR-owned (`PGEN-REGEX-PCRE2-0033`, session #80)

**Tools-first root cause (grammar-only via the certified interpreter).** The `.4.5` investigation established
that a range with a nonliteral shorthand/property endpoint (`[\d-x]`, `[a-\d]`, …) is grammar-ACCEPT /
validator-REJECT. This slice PROVED it decisively with the interpreter (`interpret_parse`, which runs the
grammar WITHOUT `validate_regex_compile_contract`): all 23 nonliteral-endpoint cells `accepted=true` (the
grammar splits `[\d-x]` into 3 members `\d`,`-`,`x`, because `class_atom = quoted_class_range_atom |
class_range_escape | class_literal` excludes the shorthand escapes — `\d` isn't in `class_range_escape`'s
letter set — and the property escapes — `property_escape` isn't in `class_range_escape_unit` — so
`class_range` never forms and the mid-class `-` falls through to a literal member). `pcre2test` 10.47
`/…/utf` = err 150 for all 23 (frozen oracle). The reject lived only in `find_invalid_char_class_construct`.

**Fix (grammar tier — no engine, `feedback_no_workarounds_fix_hierarchy` tier 1).** A zero-width negative
lookahead `!invalid_class_range` on the three class-item positions (`class_item`, `class_item_visible`,
`class_item_visible_nocaret`), each delegating to a NAMED `*_core` passthrough (`-> $2`). `invalid_class_range`
= two shapes: `class_range_nonliteral_atom zw* "-" zw* ( class_range_nonliteral_atom | class_atom )` (nonliteral
LEFT) and `class_atom zw* "-" zw* class_range_nonliteral_atom` (literal-LEFT/nonliteral-RIGHT), where
`class_range_nonliteral_atom = "\\" (class_range_nonliteral_shorthand | property_escape)` and
`class_range_nonliteral_shorthand = d|D|h|H|s|S|v|V|w|W`. Shape 1's right side must be a real endpoint atom
(so `[\d-]` — dash then `]` — stays ACCEPT); the `-` is a BARE terminal (so `[\d\-x]`'s escaped dash stays a
member).

**Two engineering findings.** (1) The initial inline wiring `class_item = !invalid_class_range ( <alt> ) -> $2`
produced `<invalid_sequence_access>` for the `posix_class` branch (byte-parity pre-check caught it) — the
"inline alternation corrupts positions" hazard (`feedback_quantified_group_extraction`); fixed by the NAMED
`*_core` rule so `$2` is a single rule ref (the `.4.6` precedent). (2) `invalid_class_range` + its sub-rules
are referenced ONLY inside `!(…)` ⇒ lookahead-only / positively-unreachable ⇒ certified by PROOF
(`gather_verified_proof_covered_rules`), so cert stays `UNKNOWN=0` while total grows 239→245.

**Scope + why the validator stays.** Shorthand + property endpoints only; POSIX-left (`[[:alpha:]-z]`) and the
`-[`-right cases (`.4.5.a`, `[a-[:digit:]]`) stay validator-owned. The validator range-check is NOT deleted —
its deletion waits until the WHOLE class-range family (nonliteral + descending `.4.5.c` + the `.4.5.a`/`.4.5.a.1`
cases) is grammar-owned. So this slice is behavior-NEUTRAL at the released `--parse` surface (validator still
rejects); the effect is observable only at the grammar layer.

#### `.4.5.b` Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — certified interpreter grammar-only (`interpret_parse("grammars/regex.ebnf",
  "[\\d-x]", pcre2)`) → `accepted=true` for all 23 nonliteral-endpoint cells (shorthand+property, both sides),
  where `pcre2test` 10.47 `/[\d-x]/utf` = err 150 AND the released `--parse` (grammar+validator) already
  REJECTS — a grammar-accepts-invalid single-source-of-truth hole. Pin: `rust/tests/regex_class_range_nonliteral_grammar_migration.rs` (FAILS on all 23 REJECT cases before the edit).
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/regex.ebnf::class_atom` (the range-endpoint reader) excludes the
  shorthand escapes (`\d` etc. not in `class_range_escape`'s letter set) and property escapes (`property_escape`
  not in `class_range_escape_unit`), so `class_range` never forms for a nonliteral endpoint and the item
  alternation falls through to `class_escape` (member) + a literal `-`. The err-150 reject lived out-of-band in
  `regex_compile_validation.rs::find_invalid_char_class_construct` (`ClassAtomKind::NonLiteral` endpoint).
- [x] **FIX** — grammar tier (fix-hierarchy 1, existing declarative construct — negative lookahead): new
  `!invalid_class_range` guard + `class_range_nonliteral_atom` / `class_range_nonliteral_shorthand` +
  `class_item*_core` passthroughs (`grammars/regex.ebnf`). No engine/codegen change.
- [x] **ADDRESSED (verified)** — the interpreter grammar-only test flips all 23 nonliteral-endpoint cells
  ACCEPT→REJECT and keeps all 16 carve-outs (`[a-]` `[-a]` `[\d-]` `[\d\-x]` `[\da-z]` `[-\d]` `[--/]` `[!--]`
  `[a-z]` `[\n-\r]` `[\x30-\x39]` + members) ACCEPT; 22 valid-class ASTs byte-identical interp==generated
  (`accepted_class_inputs_keep_byte_identical_ast_through_the_guard`). Released `--parse` matrix 28/28 unchanged.
- [x] **NO REGRESSION** — regex cert `total=245 proof=3 witness=242 UNKNOWN=0 fully_certified=true
  sample_parse_failures=0` at seeds 0/7/42 (239→245: +3 witnessed `*_core`, +3 lookahead-only PROOF rules);
  `--lint-grammar` 0 errors (245 rules); `regex_pcre2_compile_oracle_gate` byte-identical `2189/1867/274/48`;
  `duality_hunt_gate` 9 lanes no new/vanished; `parse_harness_equivalence_gate` regex byte-identical (certified
  11/11); `ast_shape_contract` regex inventory 221→224 aligned; `parse_harness_combinator_gate` 27/27 +
  `parse_harness_semantic_gate` 32/32 (shared engine untouched); embedding version-drift + metadata gates green
  (`1.1.97`/`1.1.99`); the other fully-certified grammars' generated parsers untouched (only regex regenerated).
- [x] **LOCKSTEP** — `grammars/regex.ebnf` (+6 rules); regenerated `generated/regex_parser.rs`; new test
  `rust/tests/regex_class_range_nonliteral_grammar_migration.rs`; `embedding_api.rs` consts `1.1.97`/`1.1.99`;
  `regex_parser_integration_contract_v1.json`; `ast_shape_contract/regex_v1.json` (221→224 re-baseline); ledger
  `REGEX-0107`; contract `1.1.97`/`1.1.99` Highlights + Identity; regex book `rules-char-class.md` +
  `changelog-index.md` + `compile-contract-validator.md` + tracked HTML; `CHANGES.md`; `DEVELOPMENT_NOTES.md`;
  `LIVE_ACHIEVEMENT_STATUS.md`; `MEMORY.md`; `docs/TASK_TREE.md`.

### REGEX-PCRE2-FIDELITY.4.5.c — non-`[` DESCENDING class-range reject is GRAMMAR-owned (`PGEN-REGEX-PCRE2-0034`, session #81)

**Tools-first root cause (grammar-only via the certified interpreter).** A `class_range` of two LITERAL
endpoints whose left code point exceeds the right (`[z-a]`, `[9-0]`, `[\x39-\x30]`, `[\x{100}-a]`) is PCRE2 err
108 "range out of order in character class" (`pcre2test` 10.47). The GRAMMAR accepted these — `class_range`
forms for any two literals and its `@validate: ord($1)<=ord($5)` is a NON-parse-gating codegen annotation
(proven `.4.5.a`), so the reject lived only in `validate_regex_compile_contract`. This is the descending-literal
sibling of the nonliteral-endpoint hole `.4.5.b`/`REGEX-0107`.

**Fix (grammar tier — consumes the RSVC.3 engine primitive, `feedback_no_workarounds_fix_hierarchy` tier 3
"new annotation" only because the general `value_compare` could not widen char-escapes to code points).** A
THIRD `invalid_class_range` alternative `descending_class_range = class_range_endpoint zw* "-" zw*
class_range_endpoint`, gated by `@predicate { name: value_compare_codepoint, args: [$1, gt, $5], phase: post }`
(`RULE-SPAN-VALUE-CONSTRAINT.3`, `PGEN-RSVC-0003`): the predicate DECODES each raw endpoint spelling to its
Unicode code point (bare char + `\xHH`/`\x{}`/`\NNN`/`\o{}`/`\cX`/named escapes) and matches iff left > right.
Because it is an alternative of the zero-width `!invalid_class_range` guard (the `.4.5.b` infrastructure), a
descending range is BLOCKED at the `class_item` level and cannot fall back to being reparsed as separate
members. `class_range_endpoint = class_range_decodable_atom -> $text` re-emits the RAW matched text so the
predicate sees uniform spellings.

**Four tools-first findings drove the final endpoint set (each an interpreter/`--parse-dump-ast`/runtime-trace
result — never eyeballed).** (1) A `le` gate directly ON `class_range` does NOT reject: the failed range branch
just backtracks and the class reparses `z`,`-`,`a` as members → the descending gate must live on the
`class_item` guard, not on `class_range`. (2) A bare positional ref to `class_atom` yields its TYPED return
(`{type:escape,…}` for escapes → `scalar_text` `None` → non-blocking → over-block ascending), so
`class_range_endpoint -> $text` returns the raw spelling. (3) an UNDECODABLE endpoint (`\Q..\E`, `\u{...}`) →
decode `None` → non-blocking → over-block (cert regression: `quoted_class_range_atom` went UNKNOWN), so the
endpoint is restricted to DECODABLE forms. (4) **a BARE WHITESPACE endpoint is the same non-blocking failure**
— caught by `regex_pcre2_compile_oracle_gate` (false_reject `48→50`): a `class_literal → whitespace` match
reaches the `post` predicate with an EMPTY raw spelling (`value_compare_codepoint("" > "!") → None` for
`[ -!]`, proven by runtime trace in EITHER endpoint slot; every non-whitespace literal and the ESCAPE spellings
`\x20`/`\t`/`\040` resolve correctly), so an ASCENDING whitespace range (`[ -!]`, `[ --]`, `[\t-a]`) was wrongly
OVER-BLOCKED. FIX: `class_range_decodable_atom` spells out the NON-WHITESPACE `class_literal` alternatives
directly (`class_range_decodable_escape | letter | digit | class_safe_special | unicode_char`), dropping the
`whitespace` branch — `descending_class_range` simply does not form for a bare-whitespace endpoint; the range is
left to the validator. The empty-`$text`-for-a-whitespace-match-in-a-`post`-predicate-arg is a GENERAL latent
pipeline finding, recorded durably (`project_dollar_text_whitespace_empty_in_predicate_arg`) for a future engine
investigation; a whitespace ESCAPE spelling stays gated.

**Scope + why the validator stays.** Non-`[` descending LITERAL ranges with a DECODABLE (bare non-whitespace +
hex/octal/control/simple-escape) endpoint only. The `-[`-right (`.4.5.a`), POSIX-left, `\Q..\E`/`\u{}`-endpoint,
and bare-whitespace-endpoint descending cases stay validator-owned, migrated by the follow-up **`.4.5.d`**,
which then DELETES the `find_invalid_char_class_construct` range-check. Behavior-NEUTRAL at the released
`--parse` surface (validator still rejects; the before→after ACCEPT→REJECT is observable only at the GRAMMAR
layer via the certified interpreter). Also surfaced (NOT fixed): a POST predicate on a TOP/ENTRY rule with a
leading literal + no `->` mis-resolves positional args (the direct-on-`class_range` attempt) — the named
guard-rule indirection sidesteps it.

#### `.4.5.c` Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — certified interpreter grammar-only (`interpret_parse("grammars/regex.ebnf",
  "[z-a]", pcre2)`) → `accepted=true` for the 10 descending-literal cells (bare/hex/octal/braced), where
  `pcre2test` 10.47 `/[z-a]/utf` = err 108 AND the released `--parse` (grammar+validator) already REJECTS.
  The braced `[\x{100}-a]` is the code-point discriminator (textual `"\x{100}" < "a"` would accept). Pin:
  `rust/tests/regex_class_range_descending_grammar_migration.rs`.
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/regex.ebnf::class_range` forms for two literals regardless of
  order; its `@validate: ord($1)<=ord($5)` is a non-parse-gating codegen annotation (`.4.5.a`). The err-108
  reject lived out-of-band in `regex_compile_validation.rs::find_invalid_char_class_construct` (range ordering).
- [x] **FIX** — grammar tier consuming `value_compare_codepoint` (`RULE-SPAN-VALUE-CONSTRAINT.3`): third
  `invalid_class_range` alternative `descending_class_range` + `class_range_endpoint`/`class_range_decodable_atom`/
  `class_range_decodable_escape` (`grammars/regex.ebnf`). No new engine change (the primitive landed in RSVC.3,
  commit `20bc0462`). Bare-whitespace endpoints excluded (finding 4).
- [x] **ADDRESSED (verified)** — the interpreter grammar-only test flips all 10 descending cells ACCEPT→REJECT
  and keeps the ascending/equal/carve-out controls (`[a-z]` `[a-a]` `[0-9]` `[\x30-\x39]` `[a-\x{100}]` `[a-]`
  `[-a]` `[\d-]` `[\d\-x]` `[--/]` + members) ACCEPT; ascending WHITESPACE ranges (`[ -!]` `[ --]` `[\t-a]`)
  stay ACCEPT (finding 4). Released `--parse` verdict UNCHANGED (behavior-neutral).
- [x] **NO REGRESSION** — regex cert `total=249 proof=7 witness=242 UNKNOWN=0 fully_certified=true
  sample_parse_failures=0` at seeds 0/7/42 (245→249: +4 lookahead-only PROOF rules); `--lint-grammar` 0 errors
  (249 rules); `regex_pcre2_compile_oracle_gate` byte-identical `2189/1867/274/48` (the false_reject `48→50`
  whitespace regression ROOT-CAUSED and fixed before commit); `duality_hunt_gate` 9 lanes no new/vanished;
  `parse_harness_equivalence_gate` regex byte-identical (certified 4/4 gate tests); `ast_shape_contract` regex
  inventory 224→225 aligned (+1 `class_range_endpoint` `-> $text`); `parse_harness_combinator_gate` 27/27 +
  `parse_harness_semantic_gate` 33/33 (shared engine untouched); embedding version-drift + metadata gates green
  (`1.1.98`/`1.1.100`); other fully-certified grammars' generated parsers untouched (only regex regenerated).
- [x] **LOCKSTEP** — `grammars/regex.ebnf` (+4 rules, whitespace-excluded endpoint); regenerated
  `generated/regex_parser.rs`; new test `rust/tests/regex_class_range_descending_grammar_migration.rs`;
  `embedding_api.rs` consts `1.1.98`/`1.1.100`; `regex_parser_integration_contract_v1.json`;
  `ast_shape_contract/regex_v1.json` (224→225 re-baseline); ledger `REGEX-0108`; contract `1.1.98`/`1.1.100`
  Highlights + Identity; regex book `rules-char-class.md` + `changelog-index.md` + `compile-contract-validator.md`
  + tracked HTML; top book `parser-families.md`; `CHANGES.md`; `DEVELOPMENT_NOTES.md`; `LIVE_ACHIEVEMENT_STATUS.md`;
  `MEMORY.md`; `docs/TASK_TREE.md`; durable decision `project_dollar_text_whitespace_empty_in_predicate_arg`.

### REGEX-PCRE2-FIDELITY.4.5.d — POSIX-class nonliteral class-range endpoint is GRAMMAR-owned (`PGEN-REGEX-PCRE2-0035`, session #82)

**Tools-first root cause (grammar-only via the certified interpreter — `zz_diag_class_range_residual`, run then
deleted).** A `class_range` whose LEFT or RIGHT endpoint is a valid POSIX class (`[[:alpha:]-z]`,
`[!-[:alpha:]]`) is PCRE2 err 150 "invalid range in character class" (`pcre2test` 10.47). A grammar-only
residual sweep over the whole `.4.5` matrix pinned the EXACT over-accept set: the DESCENDING posix-right cases
(`[x-[:alpha:]]`, `[a-[:digit:]]`, left code point > `[`=0x5B) ALREADY reject via `.4.5.c`'s
`descending_class_range` (the bracket's `[` reads as a `class_safe_special` literal 0x5B, so `x-[` is a
descending literal range), so the genuine grammar-over-accept residual is only the ASCENDING posix-right
(`[!-[:alpha:]]`, `!`=33 < `[`=91 → not descending) and the posix-LEFT (`[[:alpha:]-z]`). The grammar accepted
these by forming an ascending `!-[` `class_range` and reading the trailing `:alpha:]` as separate literal
members, so the err-150 reject lived only in `validate_regex_compile_contract`.

**Fix (grammar tier — the existing `.4.5.b` `!invalid_class_range` lookahead, `feedback_no_workarounds_fix_hierarchy`
tier 1).** Two new `invalid_class_range` alternatives referencing the EXISTING positively-reachable `posix_class`
rule: `posix_class zw* "-" zw* class_atom` (posix LEFT) and `class_atom zw* "-" zw* posix_class` (posix RIGHT).
Because `posix_class` is already positively entered (the first `class_item_core` alternative), no new
lookahead-only rule is created — cert `total` stays 249 (unlike `.4.5.b`/`.4.5.c`, which added rules). The
`class_atom`-after-`-` requirement (the `.4.5.b` shape-design) preserves the trailing-dash carve-out:
`[[:alpha:]-]` (posix then end-of-class dash) has no atom after `-`, so the alternative does not match → ACCEPT,
exactly like PCRE2. One EBNF hazard re-confirmed the hard way: a comment line inserted BETWEEN `|` alternatives
TERMINATES the rule in the frontend (the two new branches were silently dropped — grammar-only verdict
unchanged until the comment was moved ABOVE the rule and the alternatives made contiguous). Recorded on the rule
and in [[feedback_ebnf_meta_grammar_lockstep]]-adjacent notes.

**🔎 NOT fully behavior-neutral — a validator accepts-invalid HOLE surfaced + closed (tools-first, second
diagnostic `zz_diag_posix_validator`).** The `regex_parser_integration_contract` success-sample gate FLIPPED on
`[[:digit:]-   ]` (a declared-success sample). Root cause (a validator/grammar/pcre2 three-way diagnostic, run
then deleted): the released parser (grammar + validator) had ACCEPTED `[[:digit:]-   ]`, because the validator's
`dash_is_trailing_literal` (`regex_compile_validation.rs:707`) skips SPACE/TAB (and `\Q\E`/`\E` zero-width) after
the dash and returns "trailing literal dash" if it then hits `]` — but it applies that skip even when the range's
LEFT endpoint is a NonLiteral POSIX class, so `[:digit:]-<whitespace>]` skipped the NonLiteral-range reject →
accepts-invalid. `pcre2test` 10.47 rejects `[[:digit:]-   ]`/`[[:alpha:]- ]`/`[[:digit:]-\t]` err 150. So for
this **flip subset** (posix-LEFT range, dash followed only by whitespace/zero-width before `]`) the grammar
migration flips the released `--parse` verdict **ACCEPT→REJECT**, PCRE2-convergently — a genuine correctness fix,
NOT a neutral migration. It is PERFECTLY PCRE2-convergent (verified string-by-string against `pcre2test`): the
`\Q\E`-only-trailing carve-out `[[:alpha:]-\Q\E]` stays ACCEPT in both PCRE2 and the grammar (no over-reject).
The oracle gate stayed byte-identical `2189/1867/274/48` because the flip cells are NOT in the corpus — the
CONTRACT MANIFEST (a different surface) caught it. One contract success sample (`[[:digit:]-   ]`) moved to
failure. For every OTHER posix-endpoint case the validator ALREADY rejected, so those stay behavior-neutral.

**Scope + why the validator stays.** VALID-POSIX-name endpoints only. The remaining class-range residuals stay
validator-owned: the **collating/equivalence** bracket-tokens (`[!-[.a.]]`, `[!-[=a=]]` — no grammar recognizer
yet; joins the `.4.12` collating/equivalence family) and the **blocked descending** endpoints (`\Q..\E`/`\u{}`
whose `$text` decodes `None`, and bare-whitespace whose `$text` is empty —
[[project_dollar_text_whitespace_empty_in_predicate_arg]]). The `find_invalid_char_class_construct` range-check
DELETION waits until ALL of these land (and until the `dash_is_trailing_literal` whitespace-skip hole above is
also resolved on the validator side, or the whole range-check is deleted with the grammar owning it).

#### `.4.5.d` Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — grammar-only interpreter (`interpret_parse("grammars/regex.ebnf", "[!-[:alpha:]]",
  pcre2)`) → `accepted=true` for the ascending posix-right (`[!-[:alpha:]]`) and posix-left (`[[:alpha:]-z]`)
  cells, where `pcre2test` 10.47 = err 150 AND the released `--parse` (grammar+validator) already REJECTS — a
  grammar-accepts-invalid single-source-of-truth hole. Pin: `rust/tests/regex_class_range_posix_grammar_migration.rs`.
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/regex.ebnf::class_atom` reads the bracket's `[` as a
  `class_safe_special` literal 0x5B, so an ASCENDING `!-[` `class_range` forms and `:alpha:]` fall through to
  literal members; `class_range`'s `@validate: ord($1)<=ord($5)` is a non-parse-gating codegen annotation
  (`.4.5.a`). The err-150 reject lived out-of-band in `regex_compile_validation.rs::find_invalid_char_class_construct`
  (`ClassAtomKind::NonLiteral` posix endpoint). Descending posix-right was already owned by `.4.5.c`.
- [x] **FIX** — grammar tier (fix-hierarchy 1, existing negative-lookahead construct): two `invalid_class_range`
  alternatives `posix_class zw* "-" zw* class_atom` / `class_atom zw* "-" zw* posix_class` referencing the
  existing `posix_class` rule. No new rule, no engine/codegen change.
- [x] **ADDRESSED (verified)** — the interpreter grammar-only test flips the posix-endpoint cells
  (`[!-[:alpha:]]` `[[:alpha:]-z]` `[[:alpha:]-a]` `[[:digit:]-9]` `[[:^alpha:]-z]` `[[:alpha:]-[:digit:]]`
  `[a-[:alpha:]]` + the whitespace flip subset `[[:digit:]-   ]` `[[:alpha:]- ]` `[[:digit:]-\t]`) grammar
  ACCEPT→REJECT, keeps the `.4.5.c` descending posix-right (`[x-[:alpha:]]` `[a-[:digit:]]`) REJECT (regression
  pin), and keeps every carve-out ACCEPT (`[[:alpha:]-]` `[[:alpha:]-\Q\E]` `[[:alpha:]a]` `[a[:alpha:]]`
  `[a-z[:digit:]]` `[[:alpha:][:digit:]]` `[!-[]` `[a-z]` `[ -!]`) — every string verified against `pcre2test`
  10.47. Released `--parse` verdict is UNCHANGED for the neutral majority (validator already rejected) but FLIPS
  ACCEPT→REJECT for the whitespace subset (`[[:digit:]-   ]` family) — PCRE2-convergent (the validator's
  `dash_is_trailing_literal` accepts-invalid hole); one contract success sample moved to failure.
- [x] **NO REGRESSION** — regex cert `total=249 proof=7 witness=242 UNKNOWN=0 fully_certified=true
  sample_parse_failures=0` at seeds 0/7/42 (UNCHANGED — no new rules); `--lint-grammar` 0 errors (249 rules);
  `regex_pcre2_compile_oracle_gate` byte-identical `2189/1867/274/48` (gate probe recompiled fresh with the new
  grammar); `duality_hunt_gate` 9 lanes no new/vanished; `parse_harness_equivalence_gate` regex byte-identical;
  `ast_shape_contract` regex inventory 225 aligned (no new `->` shape — `invalid_class_range` is lookahead-only);
  `parse_harness_combinator_gate` 2/2 + `parse_harness_semantic_gate` 33/33 (shared engine untouched);
  embedding version-drift + metadata gates green (`1.1.99`/`1.1.101`; success/failure sample counts re-pinned
  93/25 → **92/26** for the moved sample); the 18 regex integration-contract lib tests green; other
  fully-certified grammars' generated parsers untouched (only regex regenerated). NOTE the released-verdict FLIP
  for the `[[:digit:]-   ]` whitespace subset is a PCRE2-CONVERGENT correctness improvement (not a regression),
  corpus-invisible, caught by the contract manifest.
- [x] **LOCKSTEP** — `grammars/regex.ebnf` (+2 alternatives on `invalid_class_range`, no new rule); regenerated
  `generated/regex_parser.rs`; new test `rust/tests/regex_class_range_posix_grammar_migration.rs` (incl. the flip
  subset + `\Q\E` carve-out, all pcre2test-verified); `embedding_api.rs` consts `1.1.99`/`1.1.101` + the metadata
  test (sample counts 92/26, moved-sample name assertion); `regex_parser_integration_contract_v1.json` (sample
  `[[:digit:]-   ]` moved success→failure as
  `posix_class_range_left_endpoint_dash_whitespace_rejects_pcre2_faithfully`, version fields
  `1.1.99`/`1.1.101`); ledger `REGEX-0109`; contract `1.1.99`/`1.1.101` Highlights + Identity; regex book
  `rules-char-class.md` + `changelog-index.md` + `compile-contract-validator.md` + tracked HTML; `CHANGES.md`;
  `DEVELOPMENT_NOTES.md`; `LIVE_ACHIEVEMENT_STATUS.md`; `MEMORY.md`; `docs/TASK_TREE.md`. (ast_shape manifest
  UNCHANGED — no new shape.)

### REGEX-PCRE2-FIDELITY.4.12 — collating-element / equivalence-class bracket-token is GRAMMAR-owned (`PGEN-REGEX-PCRE2-0037`, session #83)

**Tools-first root cause (grammar-only via the certified interpreter — reproduced by the permanent test
`rust/tests/regex_class_bracket_token_grammar_migration.rs`, run against the pre-fix grammar).** A POSIX
collating-element `[.….]` or equivalence-class `[=…=]` bracket-token in a character class is REJECTED by
`pcre2test` 10.47 — err 113 "POSIX collating elements are not supported" as a standalone / member / range-LEFT
token, and err 150 "invalid range in character class" as a range-RIGHT endpoint (the `-` triggers the range
error first). The released parser ACCEPTED all of these: `grammars/regex.ebnf`'s `class_literal` reads a class
`[` as a `class_safe_special` literal 0x5B, so the surrounding `.`/`a`/`=` fell through to separate literal
members and neither the grammar nor the out-of-band `validate_regex_compile_contract` ever recognised the token
(a genuine released-parser accepts-invalid gap — NOT a validator→grammar migration; ledger `REGEX-0110`). The
pre-fix interpreter run flipped 21 tokens grammar-ACCEPT; the 3 DESCENDING range-right cells
(`[a-[.a.]]` `[z-[.a.]]` `[a-[=a=]]`) already rejected via `.4.5.c`'s `descending_class_range` (regression pins).

**Tokenization is PCRE2-EXACT — every case oracle-verified against `pcre2test` 10.47 directly (NOT task-note
annotations; the [[feedback_report_expected_verify_against_oracle]] discipline).** The `0036` scoping had only
captured the range-endpoint err codes; the live oracle surfaced three subtleties it missed: (1) the opener `[`
can be the CLASS-OPENING bracket itself, not only a member `[` — `[.a.]` (single leading bracket) is err 113;
(2) an opener is formed ONLY when `.`/`=` IMMEDIATELY follows `[` — a `^` negation or an invisible `\E`/`\Q\E`
between breaks it (`[^.a.]`, `[\E.a.]` ACCEPT); (3) the content scan is escape-aware and stops at an UNESCAPED
`]` (the class close) — `[.].]` ACCEPT (class `[.]` closes at the first `]`, then `.]` are body literals),
`[.\].]` REJECT (the escaped `\]` is crossed as content, the `.]` terminator found), and `\` escapes ONLY `]`,
not the terminator dot (`[.a\.]` REJECT — the `.]` still terminates). A lone `.`/`=` is a literal (`[.]`/`[=]`
ACCEPT); an empty token still rejects (`[..]` err 113).

**Fix (grammar tier — a new `class_bracket_token` recognizer + three lookahead guards, `feedback_no_workarounds_fix_hierarchy`
tier 1).** `class_bracket_token = "[" class_bracket_token_tail`; `class_bracket_token_tail = "." ( "\\" "]" |
!".]" !"]" builtin_any_char )* ".]" | "=" ( "\\" "]" | !"=]" !"]" builtin_any_char )* "=]"`. Referenced ONLY
inside `!(…)` lookaheads (and inside the already-lookahead-only `invalid_class_range`), so both rules are
lookahead-only / positively-unreachable → certified by PROOF (cert `total` 249→251, proof 7→9). Three guard
sites block the shape wherever a class `[` reads as a literal: (1) the MEMBER guard `!class_bracket_token` on
`class_member_literal` / `class_member_literal_nocaret` (`-> $2`→`-> $3` for the inserted lookahead; mirrors the
`.4.6` `[:name:]` posix idiom); (2) the class-OPEN guard `!class_bracket_token_tail` in `char_class`'s no-caret
alternative (`body: $2`→`body: $3`) — placed BEFORE the `class_zero_width*` prefix so it fires ONLY when `.`/`=`
is literally the first char after `[` (an invisible/`^` structurally routes the parse away, matching PCRE2);
(3) the range-RIGHT `class_atom … "-" … class_bracket_token` alternative on `invalid_class_range` (an ASCENDING
`[!-[.a.]]` (33 < 91) not caught by `.4.5.c`'s descending guard). The class-open guard sits only on the no-caret
alternative because the initial-`]` alternative (needs `]` after `[`) and the caret alternative (needs `^`) both
structurally exclude a `.`/`=`-immediate opener.

**Scope + the validator range-check deletion.** This closes the collating/equivalence half of the `.4.5.d`
deferred residual. The remaining class-range residual is the **blocked descending** endpoints (`\Q..\E`/`\u{}`
whose `$text` decodes `None`, and bare-whitespace whose `$text` is empty —
[[project_dollar_text_whitespace_empty_in_predicate_arg]]). The `find_invalid_char_class_construct` range-check
DELETION still waits until those land (and the `dash_is_trailing_literal` whitespace-skip hole is resolved), so
the validator stays wired for now — behavior-changing here only because the grammar now REJECTS the
collating/equivalence tokens the validator never recognised.

#### `.4.12` Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — the permanent test `rust/tests/regex_class_bracket_token_grammar_migration.rs` run
  against the pre-fix grammar via `interpret_parse("grammars/regex.ebnf", <pat>, pcre2)`: 21 collating/equivalence
  tokens (`[.a.]` `[=a=]` `[..]` `[[.a.]]` `[a[.a.]b]` `[[.a.]-z]` `[!-[.a.]]` `[.\].]` `[.a\.]` …) `accepted=true`
  where `pcre2test` 10.47 = err 113/150; the 3 descending range-right cells already rejected via `.4.5.c`.
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/regex.ebnf::class_literal` (via `class_safe_special`) reads a class
  `[` as literal 0x5B, so a `[.`/`[=` token decomposes into separate literal members and no recognizer fires;
  the err-113/150 reject existed in NEITHER the grammar NOR `validate_regex_compile_contract` (a genuine
  accepts-invalid gap, not a masked migration). Tokenization boundaries (class-open opener, `^`/invisible break,
  escape-aware `]`-stopping terminator) established string-by-string against `pcre2test` 10.47.
- [x] **FIX** — grammar tier (fix-hierarchy 1): new lookahead-only `class_bracket_token` / `class_bracket_token_tail`
  recognizer + `!class_bracket_token` member guards + `!class_bracket_token_tail` class-open guard + a
  `class_atom … "-" … class_bracket_token` `invalid_class_range` alternative. No engine/codegen change.
- [x] **ADDRESSED (verified)** — `regex_class_bracket_token_grammar_migration.rs` GREEN: all 21 tokens flip grammar
  ACCEPT→REJECT, the 3 descending controls stay REJECT (regression pins), and all 23 carve-outs stay ACCEPT
  (`[.]` `[=]` `[a.b]` `[[]` `[[.]` `[.].]` `[.a].]` `[^.a.]` `[\E.a.]` `[].a.]` `[\.a.]` `[\[.a.]` `[[:alpha:]]`
  `[a-z]` …) — every string verified against `pcre2test` 10.47. Behavior-CHANGING at the released `--parse`
  surface (accepts-invalid → PCRE2-convergent reject).
- [x] **NO REGRESSION** — regex cert `total=251 proof=9 witness=242 UNKNOWN=0 fully_certified=true
  sample_parse_failures=0` at seeds 0/7/42 (249→251: +2 lookahead-only PROOF rules, witness/UNKNOWN unchanged);
  `--lint-grammar` 0 errors (251 rules, 0 unreachable/undefined); `regex_pcre2_compile_oracle_gate` byte-identical
  `2189/1867/274/48` (collating cells corpus-invisible; probe recompiled fresh); `duality_hunt_gate` 9 lanes no
  new/vanished (generator never emits the tokens); `parse_harness_equivalence_gate` regex byte-identical (11
  certified grammars); `ast_shape_contract` 18/18 (3 inventory entries re-baselined `$2`→`$3`, count 225 —
  accepted-AST shape byte-identical); `parse_harness_combinator_gate` + `parse_harness_semantic_gate` green
  (shared engine untouched); regex integration-contract lib tests green (`metadata_is_stable` at `1.1.100`/`1.1.102`,
  sample counts UNCHANGED 92/26 — contract-sample-invisible); only regex regenerated.
- [x] **LOCKSTEP** — `grammars/regex.ebnf` (+2 rules, 3 guard sites); regenerated `generated/regex_parser.rs`;
  new test `rust/tests/regex_class_bracket_token_grammar_migration.rs`; `embedding_api.rs` consts
  `1.1.100`/`1.1.102`; `regex_v1.json` ast_shape inventory (`char_class` branch 2 + `class_member_literal`(_nocaret)
  `$2`→`$3`); ledger `REGEX-0110`; contract `1.1.100`/`1.1.102` Highlights + Identity; regex book
  `rules-char-class.md` + `changelog-index.md` + `compile-contract-validator.md` + tracked HTML; top book
  `parser-families.md`; `CHANGES.md`; `DEVELOPMENT_NOTES.md`; `LIVE_ACHIEVEMENT_STATUS.md`; `MEMORY.md`;
  `docs/TASK_TREE.md`.

### REGEX-PCRE2-FIDELITY.4.10 — `\K`-in-lookaround is GRAMMAR-owned (`PGEN-REGEX-PCRE2-0038`, session #84)

**Tools-first root cause (oracle-frozen, grammar-only via the certified interpreter — reproduced by the
permanent test `rust/tests/regex_keep_out_in_lookaround_grammar_migration.rs`).** `pcre2test` 10.47 rejects
`\K` inside ANY lookaround body with err 199 "\K is not allowed in lookarounds" — every form (`(?=`/`(?!`/
`(?<=`/`(?<!`/`(?*`/`(?<*` + the alpha `(*pla:`…`(*naplb:`), INCLUDING when the `\K` sits inside a group
nested in the lookaround (`(?=a(b\Kc))`, `((?=x\Ky))`, `(?=a(?:b\Kc))`, `(?:(?=a\Kb))`, `(?=a|b\Kc)`,
`(?=(?<=x\Ky))`) — and ACCEPTS it everywhere else (`a\Kb`, `\Kword`, `(?>a\Kb)`, `(a\Kb)`, `(?:a\Kb)`,
`(?<name>a\Kb)`, and AFTER a lookaround closes: `(?=ab)\K`, `(?<=ab)\Kc`, `(*pla:ab)\K`, `a(?=b)\Kc`). Every
one of the 19 reject + 16 accept cells was verified string-by-string against `pcre2test` 10.47 (the
[[feedback_report_expected_verify_against_oracle]] discipline). The released parser already REJECTED all the
`\K`-in-lookaround forms — but via the **out-of-band validator** `find_invalid_keep_out_escape_in_lookaround`
(`regex_compile_validation.rs`), while the GRAMMAR ALONE ACCEPTED them (`keep_out`/`\K` was an ungated `anchor`
branch). A single-source-of-truth hole, so this is a **behavior-NEUTRAL validator→grammar migration** (NOT a
released-parser accepts-invalid gap): the released `--parse` accept/reject verdict is byte-identical before and
after; only the owning layer moves.

**Why the whole-active-chain scope walk (and not the innermost-only `current_scope_is` or the global
`has_fact`).** `\K`-in-lookaround is inherently CONTEXTUAL — it depends on an ENCLOSING construct that can be
several levels up (`(?=a(b\Kc))` — the `\K` is inside a capture group inside the lookaround). It is the
original `.1` table row 10 "hard one" flagged as needing a new parser-agnostic primitive. That primitive is
`not_in_scope_kind(lookaround)` (`SCOPE-CONTEXT-PREDICATE.1`, landed source-only in `PGEN-SCP-0001`): TRUE iff
NO currently-open scope — the innermost frame OR any ancestor up to root — is a `lookaround`. The innermost-only
`current_scope_is` misses the nested case; the global+monotonic `has_fact` is wrong because `close_scope` never
retracts it, so it would leak past the lookaround close and wrongly reject the valid `(?=ab)\K`. The
active-chain walk auto-unwinds the instant the scope closes — the honest, robust, general expression (fix
hierarchy tier 5 justified: no lower tier can express a whole-ancestor contextual gate). **This is the first
GRAMMAR consumer of the scope-tree `@open_scope`/`@close_scope` directives.**

**Fix (grammar tier + validator deletion, `feedback_no_workarounds_fix_hierarchy`).** (1) Each lookaround OPENS
a `lookaround` scope at its opener and CLOSES it after its body. `@open_scope` fires as an EFFECT *after* its
rule body, so it MUST bind to a small OPEN-MARKER rule matching just the opener token (the proven
`capture_open`/`named_group_open` idiom) — binding it to the whole `"(?=" pattern ")"` rule would open the scope
only after the body already parsed, too late to gate an inner `\K`. Seven markers: `lookahead_pos_open`,
`lookahead_neg_open`, `lookbehind_pos_open`, `lookbehind_neg_open`, `non_atomic_lookahead_pos_open`,
`non_atomic_lookbehind_pos_open` (`-> {type:atom, kind:lookaround_open}`) and `alpha_lookaround_open` (`-> $2`,
surfacing the alpha name so the parent's `name:$1` stays shape-preserving). `@close_scope` binds to each outer
rule (fires post-body); opens/closes are LIFO so arbitrary lookaround nesting balances, and a FAILED lookaround
rolls the scope back via the checkpoint snapshot. The marker's output ($1) is discarded by the parent's
`-> {…, body: $2}`, so every accepted lookaround AST is BYTE-IDENTICAL to the pre-marker form. (2) `\K` is
extracted from the `anchor` alternation into its own `keep_out` rule gated by
`@predicate { name: not_in_scope_kind, args: [lookaround], phase: pre }` — `\K` has a single parse
(`K` ∉ `simple_escape_letter_strict`, no literal fallback), so gating this one rule is complete; `phase: pre`
short-circuits (the predicate reads no capture). (3) Because the grammar now fully owns the reject (proven
grammar-only by the migration test), the STANDALONE validator check `find_invalid_keep_out_escape_in_lookaround`
+ its 4 exclusive helpers (`lookaround_body_start_at`, `alpha_lookaround_body_start_at`,
`is_alpha_lookaround_name`, `find_keep_out_escape`) were DELETED (the `.3.1`/`.3.2`/`.3.16` standalone-deletion
precedent; the shared `find_matching_group_end` is kept — 7 other refs). The deleted validator's
`find_matching_group_end` is depth-tracked and `find_keep_out_escape` descends into nested groups, and its
`is_alpha_lookaround_name` set equals the grammar's `alpha_lookaround_name` — so the grammar rejects EXACTLY the
validator's set (deletion is behavior-neutral).

**Duality (empirical, the `.4.5.c` argument).** The generator honors only `@gen_predicate`, never parse-time
`@predicate`, and does NOT maintain a scope chain during generation — so a gen-side scope dual is unavailable.
The migration is duality-safe by measurement: the store-aware generator provably never emits `\K` inside a
lookaround — `duality_hunt_gate` shows no new/vanished signature across 9 lanes (canonical + scaled 2000-sample,
seeds 0/7/42), and cert `sample_parse_failures=0` over 600 samples. (Root-caused a red-herring during
verification: the oracle gate's DEBUG probe grinds ~20 min on ONE pre-existing pathological corpus pattern
`((((…(x))))` — deeply-nested balanced capture groups, catastrophic `capturing_group` backtracking, RGX-0078.
Proven ORTHOGONAL to this slice by construction: that pattern's charset is `( ) 0 8 \ x` — no `(?`/`(*`/`\K`, so
none of the 7 markers or the `keep_out` gate can engage; the lookbehind-heavy `(?<=(()()…` corpus pattern, which
DOES open scopes, parses in 0.44 s. Not a regression — `.4.12`'s gate ran the same corpus.)

#### `.4.10` Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — the permanent test `rust/tests/regex_keep_out_in_lookaround_grammar_migration.rs`
  run against the pre-fix grammar via `interpret_parse("grammars/regex.ebnf", <pat>, pcre2)`: all 19
  `\K`-in-lookaround forms (`(?=a\Kb)` … `(?=(?<=x\Ky))`) `accepted=true` where `pcre2test` 10.47 = err 199,
  while the released `--parse` already REJECTED them (via the out-of-band validator) — the single-source-of-truth
  hole. Every one of the 19 reject + 16 accept cells oracle-verified against `pcre2test` 10.47.
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/regex.ebnf::keep_out`/`\K` was an ungated `anchor` branch (grammar
  ACCEPTED `\K` anywhere); the err-199 reject lived ONLY in `regex_compile_validation.rs::find_invalid_keep_out_escape_in_lookaround`.
  The reject is inherently CONTEXTUAL (an enclosing lookaround, possibly several groups up), so no lower fix
  tier can express it — it needs the whole-active-chain scope-ancestry gate `not_in_scope_kind`.
- [x] **FIX** — grammar tier (fix-hierarchy grammar + engine-primitive-consumer): 7 lookaround open-markers with
  `@open_scope {kind: lookaround}` + `@close_scope {kind: lookaround}` on the outer rules; `keep_out` extracted
  from `anchor` and gated `@predicate not_in_scope_kind(lookaround) phase:pre`; the standalone validator check
  `find_invalid_keep_out_escape_in_lookaround` + 4 exclusive helpers DELETED. First grammar consumer of the
  `SCOPE-CONTEXT-PREDICATE.1` primitive (landed `PGEN-SCP-0001`). No engine change this slice.
- [x] **ADDRESSED (verified)** — `regex_keep_out_in_lookaround_grammar_migration.rs` GREEN: all 19
  `\K`-in-lookaround forms REJECT and all 16 controls ACCEPT at the GRAMMAR layer (certified interpreter, no
  validator). Released `--parse` (grammar + remaining validator, post-deletion probe): the 21-case oracle matrix
  21/21 (11 reject + 10 accept), byte-identical to the pre-change verdict — behavior-NEUTRAL migration confirmed.
  37 remaining `regex_compile_validation` unit tests pass (deletion clean).
- [x] **NO REGRESSION** — regex cert `total=259 proof=9 witness=250 UNKNOWN=0 fully_certified=true
  sample_parse_failures=0` at seeds 0/7/42 (251→259: +8 WITNESSED rules — 7 lookaround open-markers + `keep_out`;
  proof/UNKNOWN unchanged); `--lint-grammar` 0 errors (259 rules, 0 unreachable/undefined/shadowing);
  `regex_pcre2_compile_oracle_gate` byte-identical `2189/1867/274/48` (`\K`-in-lookaround cells corpus-invisible;
  probe recompiled fresh with the validator deletion); `duality_hunt_gate` 9 lanes no new/vanished signature (the
  store-aware generator never emits `\K`-in-lookaround); `parse_harness_equivalence_gate` regex byte-identical (11
  certified grammars — the interpreter reproduces the new `@open_scope`/`@close_scope`/`not_in_scope_kind` parse);
  `ast_shape_contract` 4 aligned / drift=0, inventory 225→232 (+7 markers +`keep_out`, −`anchor` branch 8,
  `alpha_lookaround` `name:$2,body:$4`→`name:$1,body:$2`; accepted-AST shape byte-identical);
  `parse_harness_combinator_gate` 2/2 + `parse_harness_semantic_gate` 2/2 (shared engine untouched);
  `metadata_is_stable` at `1.1.101`/`1.1.103`; AST-dump schema stays `1`. Only regex regenerated; the pathological
  corpus-pattern slowness is pre-existing RGX-0078, proven orthogonal.
- [x] **LOCKSTEP** — `grammars/regex.ebnf` (+7 markers, +`keep_out`, `anchor` branch → rule-ref); regenerated
  `generated/regex_parser.rs`; `rust/src/regex_compile_validation.rs` (check + 4 helpers + 2 tests DELETED, note
  added); new test `rust/tests/regex_keep_out_in_lookaround_grammar_migration.rs`; `embedding_api.rs` consts +
  `regex_parser_integration_contract_v1.json` manifest `1.1.101`/`1.1.103`; `regex_v1.json` ast_shape inventory
  (225→232); ledger `REGEX-0111`; contract `1.1.101`/`1.1.103` Highlights + Identity; regex book
  `examples-anchors.md` + `rules-groups.md` + `compile-contract-validator.md` + `changelog-index.md` + tracked
  HTML; `CHANGES.md`; `DEVELOPMENT_NOTES.md`; `LIVE_ACHIEVEMENT_STATUS.md`; `MEMORY.md`; `docs/TASK_TREE.md`.

### REGEX-PCRE2-FIDELITY.4.6 — POSIX class NAME validity is GRAMMAR-owned (`PGEN-REGEX-PCRE2-0024`, session #74)

**Design (tool-backed, `pcre2test` 10.47 oracle + message-source probe).** PCRE2 treats a `[:name:]` token
inside a class as a POSIX-class ATTEMPT: the name (after an optional `^`) must be one of the 14 valid names,
else err 130 "unknown POSIX class name"; with no `:]` terminator before the class ends, `[:` is ordinary
literals (`[[:foo]` ACCEPT). The `[[:<:]]`/`[[:>:]]` word-boundary aliases are anchor atoms (already
`posix_word_boundary_alias`, matched before `char_class`); inside a larger class `[a[:<:]]` the `[:<:]` is an
invalid-name attempt → reject.

This slice is a BEHAVIOR-NEUTRAL migration (the `.4.1` precedent): the grammar guard scans to the FIRST `:]`
EXACTLY like the deleted validator `scan_posix_class`, so the accept/reject SET is byte-identical (oracle gate
stays `2189/1858/285/46`) — only the reject source/message moves validator→grammar. Honest bound (pre-existing,
out of `.4.6` scope): the validator (and thus this guard) scan to the first `:]` across BOTH escaped and
unescaped `]`, whereas PCRE2's posix-name boundary stops at an UNESCAPED `]` (`[x[:foo]bar:]y]` ACCEPT in PCRE2,
REJECT in PGEN both before and after this slice). A first PCRE2-exact-boundary attempt (`!"]"` in the run)
introduced a +1 oracle false-accept on the corpus's escaped-`]` case `[abc[:x\]pqr:]]` — reverted for
zero-regression; the PCRE2-exact `]`-boundary (a name-`]`-boundary primitive) is a dedicated follow-up.

The grammar's `posix_class` (`"[:" posix_negation? posix_name ":]"`) already accepts the 14 valid names and
wins the `class_item` tournament by longest-match; the DEFECT was the LITERAL FALLBACK — when `posix_class`
fails on a bad name, the `[`/`:`/letters matched as `class_literal`s, so the grammar ACCEPTED `[[:foo:]]`
(only the out-of-band `find_invalid_char_class_construct` rejected it — the single-source-of-truth hole).

FIX (grammar tier, DECLARATIVE): guard the class-member `[` literal with an inline negative lookahead for
the `[:…:]` posix-token shape, at the 3 member positions (`class_item`, `class_item_visible`,
`class_item_visible_nocaret`) via two wrapper rules `class_member_literal` / `class_member_literal_nocaret`
(`!( "[:" "^"? ( !":]" !"]" builtin_any_char )* ":]" ) class_literal[_nocaret] -> $2`). Name-agnostic on
purpose: for a VALID name `posix_class` wins so blocking the literal is inert; for an INVALID name the literal
is blocked and the class cannot close → reject. `scan_posix_class` recognition STAYS in the validator (range
analysis `[[:alpha:]-z]` err 150, owned by `.4.5`); only its name-reject + `is_valid_posix_class_name` are
deleted. Zero-width guard → generation unchanged (duality-neutral); `-> $2` preserves the bare-string
class-member value (the `literal_open_brace` regex.ebnf:327 precedent).

Released slice: release `1.1.90`→`1.1.91` / contract `1.1.92`→`1.1.93` / schema `1` (unchanged — no
AST-shape change on accepted inputs); ledger `REGEX-0101` (internal, behavior-neutral downstream; reject
CODE `E_PARSE_FAILURE` unchanged, only the reject MESSAGE moves validator→grammar; accept/reject SET
byte-identical to `1.1.90`, oracle gate `2189/1858/285/46` unchanged).

#### `.4.6` Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — message-source probe (release `parseability_probe --parse regex --profile pcre2`,
  the `.4` SCOPING-LOG technique): `[[:foo:]]` `[[:foo:]` `[a[:<:]]` `[a[:>:]]` `[[::]]` `[[:al pha:]]`
  `[x[:foo:]y]` `[[:foo:]x]` `[^[:foo:]]` `[[:^foo:]]` `[[:foo:bar:]]` `[[:al:num:]]` all REJECT with the
  VALIDATOR message `unknown POSIX character class name` (fires only AFTER a grammar-accept) ⇒ the GRAMMAR
  ALONE accepts these 13 PCRE2-invalid patterns (`pcre2test` 10.47 rejects all 13, err 130).
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/regex.ebnf`: `posix_class` fails on an invalid name, so the
  `[`/`:`/letters fall back to `class_literal` (via `class_safe_special`, incl. `[` and `:`) →
  `char_class` closes as a class of literals. The PCRE2 name-validity rule lived OUT-OF-BAND in
  `find_invalid_char_class_construct` → `scan_posix_class` → `is_valid_posix_class_name`
  (`rust/src/regex_compile_validation.rs`), invisible to the single-source-of-truth EBNF
  ([[project_ebnf_is_single_source_of_truth]]).
- [x] **FIX** — fix-hierarchy GRAMMAR tier (no engine change): the `class_member_literal` /
  `class_member_literal_nocaret` inline-lookahead wrappers (`!( "[:" "^"? ( !":]" builtin_any_char )* ":]" )
  class_literal[_nocaret] -> $2`) block the `[` literal for the `[:…:]` shape at the 3 member positions
  (`class_item`, `class_item_visible`, `class_item_visible_nocaret`); `is_valid_posix_class_name` + its call
  + 2 validator unit tests deleted same-slice; `scan_posix_class` recognition retained for range analysis.
- [x] **ADDRESSED (verified)** — 33-cell before→after matrix: the 13 REJECT now via the GRAMMAR message
  (`Parser did not consume full input at position 0`, both profiles), the ≥20 controls stay ACCEPT; EVERY
  verdict matches `pcre2test` 10.47 (incl. the escaped-`]` corpus case `[abc[:x\]pqr:]]` REJECT). The 4
  contract-pinned POSIX AST shapes (`[[:space:]]+` `[[:blank:]]+` `^[:a[:digit:]]+` `^[:a[:digit:]:b]+`)
  byte-identical + 5 baseline class ASTs (`[abc]`/`[a-z]`/`[[:alpha:]]`/`[^xy]`/`[]x]`) byte-identical.
- [x] **NO REGRESSION** — regex cert-coverage `total=238 witness=238 UNKNOWN=0 fully_certified=true spf=0` at
  seeds 0/7/42 (236→238, the 2 net-new wrapper rules witnessed); `regex_pcre2_compile_oracle_gate` EXACTLY
  byte-identical baseline `2189/1858/285/46` (a first `!"]"` PCRE2-exact attempt regressed +1 false-accept on
  `[abc[:x\]pqr:]]` and was reverted); `duality_hunt_gate` 9 lanes NO new signature; `parse_harness_equivalence_gate`
  regex byte-identical (differential-CERTIFIED); `regex_ast_shape_contract_gate` aligned (inventory 217→219);
  dual `--lib` suite **891/0/29** (892 − 2 deleted validator tests + 1 new pin); `--lint-grammar` 0 errors
  (238 rules); the other 5 fully-certified grammars untouched (only `regex_parser.rs` regenerated); clippy
  no-new-findings (pre-existing codegen `ToTokens`/`Span`/`Range` debt only).
- [x] **LOCKSTEP** — `grammars/regex.ebnf`; `regex_compile_validation.rs`; `parser_registry.rs` pin
  `regex_posix_class_names_reject_at_the_grammar_layer_pcre2_faithfully`; `embedding_api.rs` consts
  `1.1.91`/`1.1.93`; `regex_parser_integration_contract_v1.json`; contract Identity + `1.1.91`/`1.1.93`
  Highlights; ledger `REGEX-0101`; `ast_shape_contract/regex_v1.json` (+2 inventory entries); regex book
  (`rules-char-class` § POSIX-name-validity + `changelog-index` + tracked HTML); top book `parser-families.md`;
  `CHANGES.md`; `DEVELOPMENT_NOTES.md`; `LIVE_ACHIEVEMENT_STATUS.md`; `MEMORY.md`; `docs/TASK_TREE.md`.

### REGEX-PCRE2-FIDELITY.4.4 — escape-in-class rejects are GRAMMAR-owned (`PGEN-REGEX-PCRE2-0025`, session #75)

**Design (tool-backed, message-source probe + `pcre2test` 10.47 oracle).** Inside a character class PCRE2
forbids the anchor / assertion / special escapes `\A \B \C \G \K \R \X \Z \z` outright, and `\N` unless it
is the braced named-codepoint form `\N{…}`. These were the escape-in-class half of the out-of-band
`find_invalid_char_class_construct` (`read_class_atom`'s `\N`-unbraced check + the `A|B|C|G|K|Q|R|X|Z|z`
`matches!`). The message-source probe (both profiles) proved every reject form is **VALIDATOR-reject =
load-bearing** — the GRAMMAR alone accepted `[\B]`/`[\K]`/`a[\NB]c`/`[\C]`/`[\A-x]`/`[a-\B]`/`[\z-\x{FFFF}]`
— and every control (`[\b]`/`[\d]`/`[\N{U+00E9}]`/`[\g]`/`[\j]`/`[\I-x]` + all pattern-body `\C`/`\B`/`\K`/`\N`)
stays ACCEPT.

Three grammar reach paths admitted these escapes and all three are closed this slice (STRUCTURAL, DECLARATIVE):
1. **Class-member catch-all** — `class_simple_escape_{strict,relaxed}` matched `\<letter>` via `any_char`.
   Guarded with `!"A" !"B" !"C" !"G" !"K" !"R" !"X" !"Z" !"z" !( "N" !"{" )` (positions shift → `char:$23`
   strict / `char:$17` relaxed). `!( "N" !"{" )` is the only conditional guard — it rejects a bare `\N`
   while a following `{` keeps `\N{…}` a valid member.
2. **`\C` via `single_byte_escape`** — the FIRST alt of `class_escape_unit` unconditionally matched `\C`.
   `single_byte_escape` is DROPPED from `class_escape_unit` (it stays in `escape_unit` for the pattern body,
   where `\C` is valid); the `!"C"` guard then also blocks the catch-all.
3. **Range endpoint** — `A`/`G` (and lowercase `z`) were in `class_range_literal_escape_letter_strict`, so
   `[\A-x]`/`[\G-x]`/`[\z-\x{FFFF}]` parsed via the RANGE path (which the member guards do not cover). `A`,
   `G`, `z` are dropped from that rule; the other range letters (`I J M O T V Y` + lowercase) are untouched
   (out of the `.4.4` reject set — `\I` etc. are owned by `.3.11`/`.4.5`).

Name-agnostic and duality-neutral: the guards are zero-width lookaheads, and the removed productions were
never part of a NET-accepted input (each was validator-rejected), so generation is unchanged and no NET
accept/reject verdict flips — only the reject SOURCE/message moves validator→grammar. The validator's two
`read_class_atom` reject blocks are deleted (dead post-migration); `scan_char_class` range analysis
(`.4.5`) is untouched. NO new rule (`single_byte_escape` still exists), so cert rule-count stays 238 and
ast_shape inventory stays 219.

Released slice: release `1.1.91`→`1.1.92` / contract `1.1.93`→`1.1.94` / schema `1` (unchanged — no
AST-shape change on accepted inputs); ledger `REGEX-0102` (internal, behavior-neutral downstream; reject
CODE `E_PARSE_FAILURE` unchanged, only the reject MESSAGE moves validator→grammar; accept/reject SET
byte-identical to `1.1.91`, oracle gate `2189/1858/285/46` unchanged).

#### `.4.4` Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — message-source probe (release `parseability_probe --parse regex --profile
  {pcre2,relaxed}`, the `.4` SCOPING-LOG technique): `[\A]` `[\B]` `[\C]` `[\G]` `[\K]` `[\N]` `[\R]` `[\X]`
  `[\Z]` `[\z]` `a[\NB]c` `[a\Kb]` and the range forms `[\B-x]` `[a-\B]` `[\A-x]` `[\G-x]` `[\z-\x{FFFF}]`
  all REJECT with the VALIDATOR message (`… character class …` / `\N is not accepted …`), firing only AFTER
  a grammar-accept ⇒ the GRAMMAR ALONE accepts these PCRE2-invalid patterns (`pcre2test` 10.47 rejects all).
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/regex.ebnf`: (1) `class_simple_escape_{strict,relaxed}`
  (`:778`/`:781`) matched `\<reject-letter>` via `any_char`; (2) `class_escape_unit` (`:744`) tried
  `single_byte_escape = "C"` first, so `[\C]` accepted; (3) `class_range_literal_escape_letter_strict`
  (`:827-828`) listed `A`/`G`/`z`, so `[\A-x]`/`[\z-\x{FFFF}]` parsed via the RANGE path. The reject lived
  OUT-OF-BAND in `regex_compile_validation.rs::read_class_atom` (`:778-792`: `\N`-unbraced + the
  `A|B|C|G|K|Q|R|X|Z|z` `matches!`) — a single-source-of-truth hole ([[project_ebnf_is_single_source_of_truth]]).
- [x] **FIX** — fix-hierarchy GRAMMAR tier (no engine change): the 10 member guards on both
  `class_simple_escape` variants + drop `single_byte_escape` from `class_escape_unit` + drop `A`/`G`/`z`
  from `class_range_literal_escape_letter_strict`. Validator: delete the two `read_class_atom` reject blocks
  + the 3 unit tests (`rejects_invalid_class_escape` / `rejects_keep_out_escape_in_character_class` /
  `rejects_not_newline_escape_in_character_class`); ADD the `parser_registry.rs` pin
  `regex_class_escapes_reject_at_the_grammar_layer_pcre2_faithfully`.
- [x] **ADDRESSED (verified)** — after regen + both-binary rebuild, the message-source probe shows the 18
  reject forms (member/mid/range) flip to the GRAMMAR message (`Parser did not consume full input`, both
  profiles) and the 28 controls (20 class members incl. braced `\N{…}` + 8 pattern-body) stay ACCEPT. The
  pin `regex_class_escapes_reject_at_the_grammar_layer_pcre2_faithfully` (19 reject + 20 accept + 8 body + 6
  relaxed cells) is GREEN. Every verdict matches `pcre2test` 10.47.
- [x] **NO REGRESSION** — regex cert-coverage `total=238 witness=238 UNKNOWN=0 fully_certified=true spf=0` at
  seeds 0/7/42 (rule-count unchanged); `regex_pcre2_compile_oracle_gate` EXACTLY byte-identical baseline
  `2189/1858/285/46`; `duality_hunt_gate` 9 lanes NO new/vanished signature; `parse_harness_equivalence_gate`
  regex byte-identical (differential-CERTIFIED); `regex_ast_shape_contract_gate` aligned (inventory 219,
  unchanged); dual `--lib` suite (−3 deleted validator tests + 1 new pin); `--lint-grammar` 0 errors (238
  rules); the other 5 fully-certified grammars untouched (only `regex_parser.rs` regenerated); clippy
  no-new-findings.
- [x] **LOCKSTEP** — `grammars/regex.ebnf`; `regex_compile_validation.rs`; `parser_registry.rs` pin;
  `embedding_api.rs` consts `1.1.92`/`1.1.94`; `regex_parser_integration_contract_v1.json`; contract Identity
  + `1.1.92`/`1.1.94` Highlights; ledger `REGEX-0102`; regex book (`rules-char-class` § escape-in-class +
  `rules-escape` + `compile-contract-validator` + `changelog-index` + tracked HTML); top book
  `parser-families.md`; `CHANGES.md`; `DEVELOPMENT_NOTES.md`; `LIVE_ACHIEVEMENT_STATUS.md`; `MEMORY.md`;
  `docs/TASK_TREE.md`.

### REGEX-PCRE2-FIDELITY.4.2 — SCOPING (tools-first, session #75, PURE-DOCS)

Message-source probe (release `parseability_probe`, both profiles) + grammar read of the name rules
(`grammars/regex.ebnf`) **before** any edit, to split `.4.2` into its load-bearing vs already-owned parts:

- **Named-group CHARSET is ALREADY grammar-owned — nothing to migrate.** `name = ( letter | '_' |
  unicode_char ) ( letter | digit | '_' | unicode_char )*` (`regex.ebnf:528`) already enforces first-char ≠
  digit + the charset, so `(?<>x)` `(?<1bad>x)` `(?'1bad'x)` `(?P<1bad>x)` `(?<a b>x)` all **GRAMMAR-reject**
  today (probe: GRAMMAR-reject, not VALIDATOR-reject). The `capture_name`/`named_group_open_*` rules
  (`:1163`–`:1180`) delegate to `name`, so the named-group charset half is a no-op.
- **`\k` backreference shape + non-empty name is LOAD-BEARING.** `\k` `\kabc` `\k''` `\k<>` `\k{}` are
  **VALIDATOR-reject** (grammar accepts) — the `\k` rule does not require a delimiter + non-empty `name` the
  way `read_delimited_name_at` does. STRUCTURAL: route `\k` through a delimited non-empty `name` (the
  `named_group_open_*` idiom).
- **Length ≤ 128 is LOAD-BEARING for BOTH named groups and `\k`, and carries a real design question.** The
  129-char `(?<a…a>x)` / `\k<a…a>` are **VALIDATOR-reject** (grammar's `name` is unbounded `*`). A bounded
  quantifier `( first )( rest ){0,127}` is now expressible (BOUNDED-QUANT.1 landed `{N,M}` first-class), so
  the ASCII case is STRUCTURAL. **BUT** PCRE2's `PCRE2_MAX_NAME_SIZE` is 128 **code units** and the deleted
  validator uses **byte** `name.len()`, whereas a `{0,127}` char-quantifier counts **characters** — they
  DIVERGE for a multi-byte Unicode name (`unicode_char` in `name`). So a byte-exact migration needs either a
  Unicode-aware code-unit bound or a rule-span byte-length primitive; the fix-hierarchy escalation (new
  primitive) must be justified tools-first against PCRE2's exact code-unit rule before it is chosen.

**Decision.** `.4.2` is NOT a one-shot structural slice like `.4.4`/`.4.6`: the charset half is already
owned, the `\k` shape half is clean STRUCTURAL, but the length half has a byte-vs-char-count subtlety that
wants a deliberate design pass (measure PCRE2's exact code-unit limit across widths; decide bounded-quant
vs a length primitive per the fix hierarchy). Recommended: take `.4.2` up in a FRESH design-focused session
with this scoping as the starting point. `find_invalid_named_escape_or_group_name` stays validator-owned
until then.

### REGEX-PCRE2-FIDELITY.4.2 — `\k`/group NAME validity is GRAMMAR-owned (`PGEN-REGEX-PCRE2-0027`, session #76)

**Design (tool-backed, `pcre2test` 10.47 oracle + message-source probe).** The fresh design-focused session
the SCOPING called for. Three tool-backed facts settle it:
- **Name length = 128 code units, UNIFORM across ALL name positions.** `pcre2test` gives err 148 "subpattern
  name is too long (maximum 128 code units)" for a 129-char name in EVERY position — def `(?<…>x)` AND refs
  `\g<…>` / `(?&…)` / `(?P=…)` / `\k<…>` / condition `(?(…)a)` (all measured). ⇒ bounding the SHARED `name`
  rule is the faithful, simplest, duality-safe (gen+parse share the rule) encoding, orthogonal to `.4.11`
  (existence, not length).
- **The code-unit is width-dependent; RGX is Unicode-only ⇒ 128 code POINTS.** 8-bit: 64×'é' (128 bytes) OK /
  65×'é' (130 bytes) REJECT (code-unit = byte). 32-bit: 128×'é' OK / 129×'é' REJECT (code-unit = code-point).
  Per [[feedback_rgx_unicode_only_8bit_test_divergence]] RGX matches the Unicode (code-point) semantics, so
  the faithful limit is 128 code-points. PGEN's `name` counts Unicode scalars natively ⇒ `( first )( rest ){0,127}`
  = max 128 code-points = EXACT. **NO new primitive** (BOUNDED-QUANT.1 made `{N,M}` first-class). The deleted
  validator used BYTE `name.len()` (8-bit semantics); the grammar's code-point count is MORE correct for RGX —
  the only behavior change is multi-byte names of 65–128 code-points, now ACCEPT (was validator-REJECT), which
  is the ratified Unicode-only posture, not a fidelity regression.
- **`\k` shape.** `pcre2test`: bare `\k` / `\kabc` → err 169 "\k is not followed by a braced, angle-bracketed,
  or quoted name"; empty `\k''` / `\k<>` / `\k{}` → err 162 "subpattern name expected"; `\k<n>` / `\k'n'` /
  `\k{n}` → COMPILE-OK. So `\k` is ALWAYS a named-backref introducer (never a bare shorthand), needing a
  non-empty delimited name. The GRAMMAR alone accepted `\k`/`\kabc`/`\k<>`/…: lowercase `k` was in
  `simple_escape_letter_strict`, so `\k` matched the `simple_escape` catch-all (`{kind:"shorthand",char:"k"}`)
  and the trailing chars parsed as literals — the `.4.1` `\p`/`\P` reach-path exactly. (Bonus latent fix:
  `\k'n'` was silently mis-parsed as shorthand-`k` + `'`/`n`/`'` literals — NO `\k'…'` branch existed — so its
  AST shape was WRONG though ACCEPTED; this slice makes it a proper `{type:"backreference",kind:"named"}`.)

Three grammar edits (STRUCTURAL/DECLARATIVE, fix-hierarchy GRAMMAR tier, no engine change):
1. **Length** — `name = ( letter | '_' | unicode_char ) ( letter | digit | '_' | unicode_char ){0,127} -> $text`
   (was `*`). Bounds every name to ≤128 code-points, matching PCRE2 err 148 uniformly.
2. **`\k` always-introducer** — `!"k"` guard added to `simple_escape` (`char:$5`→`$6`) + `'k'` dropped from
   `simple_escape_letter_strict`. `\k` never matches the shorthand catch-all in EITHER profile (relaxed does
   not re-admit `k`), so a malformed `\k` has no rule ⇒ hard REJECT.
3. **`\k'name'` quote branch** — `| "\\k" "'" name "'" -> {type:"backreference", kind:"named", ref:$3}` added
   to `backreference` (the angle/braced forms already exist via `name_ref`/`braced_name_ref`).

Validator: `find_invalid_named_escape_or_group_name` + its 5 EXCLUSIVE helpers (`read_delimited_name_at`,
`read_named_group_name_at`, `is_pcre2_capture_name`, `is_pcre2_name_char`, `is_pcre2_name_digit`) + the
`PCRE2_MAX_NAME_SIZE` const are DELETED (charset was already grammar-owned per SCOPING; shape+length now are
too). Shared helpers (`skip_char_class_for_group` / `is_extended_class_start` / `skip_quoted_literal_escape`
/ `is_short_unicode_property_letter`) STAY. 3 validator tests deleted
(`allows_unicode_capture_names_and_named_backreferences` / `rejects_malformed_named_backreference_escapes` /
`rejects_capture_names_beyond_pcre2_limit`); ADD `parser_registry.rs` pin
`regex_named_names_reject_at_the_grammar_layer_pcre2_faithfully`. This is the 4th `.4` deletion-prep child +
the 11th compile-contract check migrated; `find_invalid_named_escape_or_group_name` fully retired.

Released slice: release `1.1.92`→`1.1.93` / contract `1.1.94`→`1.1.95` / schema `1` (the backreference typed
shape already exists; `\k'n'`'s shape change is a fix within the existing carrier, no schema bump); ledger
`REGEX-0103`.

#### `.4.2` Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — message-source probe (release `parseability_probe --parse regex --profile
  {pcre2,relaxed}`) + `pcre2test` 10.47: `\k` `\kabc` `\k''` `\k<>` `\k{}` REJECT with the VALIDATOR message
  ("malformed/invalid named backreference escape") firing only AFTER a grammar-accept; a 129-char capture
  name / `\k` name REJECT with the VALIDATOR "capture group name" message ⇒ the GRAMMAR ALONE accepts these
  PCRE2-invalid patterns.
- [x] **ROOT CAUSE (WHY + WHERE)** — `grammars/regex.ebnf`: (1) `name` (`:528`) was unbounded `*` (no length
  gate); (2) `simple_escape_letter_strict` (`:1000`) listed lowercase `k`, so `\k` matched the `simple_escape`
  (`:979`) catch-all as a shorthand and trailing chars parsed as literals; (3) no `\k'…'` branch existed in
  `backreference` (`:409`). The shape+length rejects lived OUT-OF-BAND in
  `regex_compile_validation.rs::find_invalid_named_escape_or_group_name` (`:138`) — a single-source-of-truth
  hole ([[project_ebnf_is_single_source_of_truth]]).
- [x] **FIX** — fix-hierarchy GRAMMAR tier (no engine change): (1) `name` `*`→`{0,127}`; (2) `!"k"` guard on
  `simple_escape` + drop `'k'` from `simple_escape_letter_strict`; (3) add `"\\k" "'" name "'"` branch.
  Validator: delete `find_invalid_named_escape_or_group_name` + its 5 exclusive helpers + `PCRE2_MAX_NAME_SIZE`
  + 3 tests; ADD the `parser_registry.rs` pin.
- [x] **ADDRESSED (verified)** — after regen + both-binary rebuild, the message-source probe shows the malformed
  `\k` forms + the 129-char names flip to the GRAMMAR message (`Parser did not consume full input`, both
  profiles) and the controls (`\k<n>`/`\k'n'`/`\k{n}`, valid + 128-char + Unicode names) stay ACCEPT; `\k'n'`
  now dumps a `backreference` AST. Every verdict matches `pcre2test` 10.47.
- [x] **NO REGRESSION** — regex cert-coverage `total=238 witness=238 UNKNOWN=0 fully_certified=true spf=0` at
  seeds 0/7/42; `regex_pcre2_compile_oracle_gate` byte-identical baseline `2189/1858/285/46`; `duality_hunt_gate`
  9 lanes NO new/vanished signature; `parse_harness_equivalence_gate` regex byte-identical; `regex_ast_shape_contract_gate`
  aligned; dual `--lib` suite (−3 deleted validator tests + 1 new pin); `--lint-grammar` 0 errors; the other 5
  fully-certified grammars untouched; clippy no-new-findings.
- [x] **LOCKSTEP** — `grammars/regex.ebnf`; `regex_compile_validation.rs`; `parser_registry.rs` pin;
  `embedding_api.rs` consts `1.1.93`/`1.1.95`; `regex_parser_integration_contract_v1.json`; contract Identity +
  Highlights; ledger `REGEX-0103`; regex book (`rules-escape` + `rules-groups` + `compile-contract-validator` +
  `changelog-index` + tracked HTML); top book `parser-families.md`; `CHANGES.md`; `DEVELOPMENT_NOTES.md`;
  `LIVE_ACHIEVEMENT_STATUS.md`; `MEMORY.md`; `docs/TASK_TREE.md`.

### REGEX-PCRE2-FIDELITY.4.3 — counted-quantifier `{N,M}` min>max ORDER is GRAMMAR-owned (`PGEN-REGEX-PCRE2-0029`, session #77)

**Design (tool-backed, `pcre2test` 10.47 oracle + empirical duality analysis).** PCRE2 rejects a counted
quantifier whose minimum exceeds its maximum: `pcre2test` 10.47 gives err 104 "numbers out of order in {}
quantifier" for `x{5,4}` / `a{\t5\t,\t2\t}`, while `{4,5}` / `{5,5}` / `{5,}` / `{,5}` / `{5}` all compile
clean. This min>max ORDER rule was the LAST residual of the out-of-band validator
`find_invalid_counted_quantifier` (`regex_compile_validation.rs:162`) → `validate_counted_quantifier_body`
(`:400`) — a RULE-SPAN VALUE COMPARISON across the two bound captures that no CONTEXT-FREE structural
encoding can express (min ≤ max over two independent [0,65535] numbers is not context-free; `.3.18`
explicitly DEFERRED it here for exactly this reason). It is the first consumer of the general
`value_compare` `@predicate` primitive (RULE-SPAN-VALUE-CONSTRAINT.2, [[project_rule_span_value_compare_primitive]]).

The GRAMMAR change (STRUCTURAL/DECLARATIVE, fix-hierarchy tier-1 existing annotation, no engine change):
extract the `{n,m}` range form of `counted_quantifier_body` into a dedicated rule and gate it with the
value comparison:
```ebnf
@predicate: { name: value_compare, args: [$1, le, $5], phase: post }
counted_quantifier_range = quant_bound_number brace_ws? "," brace_ws? quant_bound_number brace_ws?  -> {min: $1, max: $5}
```
and reference it as the first body branch (`counted_quantifier_range -> $1`). `$1`/`$5` are the two
`quant_bound_number` bound captures; `compare_values` is decimal-integer numeric when both parse as i64, so
`{05,4}` = 5>4 REJECTS and `{05,5}` = 5≤5 ACCEPTS (the leading-zero-hostile case the validator's own comment
named). Scoping to a dedicated single-sequence rule is the PROVEN idiom (`sem_value_compare`) and avoids the
`{n,}` null-max branch entirely (where a cross-capture compare would be ill-defined).

**WHY the whole-pattern REJECT is automatic (tool-traced, no guard change).** A `value_compare` `post`
rejection is BACKTRACKABLE (proven by `sem_value_compare_backtrack`): on `{5,4}` the range branch LOSES the
`counted_quantifier_body` tournament; the other three body branches each fully-consume only a PREFIX
(`{n,}`→`5,`, `{n}`→`5`) so the outer `counted_quantifier` `"}"` never follows → `counted_quantifier` fails
as a whole → `quantifier?` matches empty → the `literal_open_brace` guard's negative lookahead
(`grammars/regex.ebnf:327`, raw `digit+`) still recognizes `{5,4}` as quantifier-shaped and BLOCKS the
literal fallback ⇒ whole-pattern REJECT, err-104-faithful. This is the identical mechanism `.3.18` already
uses for the err-105 out-of-range VALUE bound; the `literal_open_brace` guard needs NO change.

**WHY it is duality-neutral (empirical, tools-first — the `.4.3`-owned generation-satisfaction question).**
The duality oracle is grammar-parse + `validate_regex_compile_contract`, which ALREADY rejects `{5,4}` today,
so if the generator emitted min>max it would ALREADY be a pinned duality break — it is NOT (the only pinned
regex signature is the start-option class, `.4.8`). Confirmed empirically: (a) the 2000-sample directed
duality hunt at seeds 0/7/42 surfaces ONLY the start-option signature; (b) a 3000-sample plain generation
(seeds 0/7/42) emits 11 `{n,m}` range forms, ALL `{0,0}` variants — 0 min>max (the generator favors the
`"0"+` expansion of `quant_bound_number`). Parse-time `@predicate` is not honored generation-side (the
generator draws only off `@gen_predicate`, never parse-time `@predicate`), so adding the gate leaves
generation byte-identical and the emitted values already satisfy min≤max ⇒ NO `@gen_predicate` companion
needed. The `duality_hunt_gate` remains the standing guard if a future generator distribution ever emits
min>max.

Validator: `find_invalid_counted_quantifier` + `validate_counted_quantifier_body` (both min>max-only) are
DELETED, plus the `find_invalid_counted_quantifier(input)` call at `validate_regex_compile_contract:46`;
the entry `.3.18` narrowing comment is updated to record the migration. The min>max validator tests are
deleted. This ADDS a rule (`counted_quantifier_range`) → cert rule-count 238→239 and ast_shape inventory
+1 (the range rule's `{min,max}` carrier).

Released slice: release `1.1.93`→`1.1.94` / contract `1.1.95`→`1.1.96` / schema `1` (unchanged — the
`{min,max}` typed shape is preserved through the extracted rule); ledger `REGEX-0104` (internal,
behavior-neutral downstream; reject CODE `E_PARSE_FAILURE` unchanged, only the reject MESSAGE moves
validator→grammar; accept/reject SET byte-identical to `1.1.93`, oracle gate `2189/1858/285/46` unchanged).

🔎 **SURFACED FINDING (latent codegen gap — routed for a future engine leaf).** The natural gate
`@predicate value_compare [$1, le, $5]` (raw-view positional) HARD-ERRORED at parse time —
`--trace-rules counted_quantifier_range` at `PGEN_TRACE_VERBOSITY=debug` on `A{0,0}` showed
`❌ Exiting rule 'counted_quantifier_range' with error: "Semantic runtime could not resolve attribute
reference '$1'"` (cert `A{0,0}` `parsed=false`, UNKNOWN=1). ROOT CAUSE (code-read, `ast_based_generator.rs`):
a Raw-view post-predicate resolves its `$N` refs against `semantic_raw_content`, which is captured only when
`semantic_capture_raw_for_post` is true, via `if semantic_capture_raw_for_post { semantic_raw_content =
Some(result.clone()); }` emitted BEFORE the return transform — but on the **sequence-with-return-transform**
path (a rule with a `->` object annotation over a sequence body) that raw-capture line is NOT emitted, so
`semantic_raw_content` stays `None` and falls back to `node.content` = the shaped JSON, where positional
`$N` resolution returns `None` (JSON has no positional slots) ⇒ hard error. **Any raw-view positional
`@predicate` on a rule that also has a `->` sequence transform will similarly fail.** Worked around here with
`view: shaped` + NAMED refs (the proven SV idiom, robust to both the JSON shape and the unmatched
`brace_ws?` optionals). The codegen fix (emit the raw capture on the transform path too) is a separate,
parser-agnostic engine leaf — deferred, not needed for `.4.3`. **NOW TASK-TREE OWNED** by
[`RAWCAP-TRANSFORM-PATH`](RAWCAP-TRANSFORM-PATH.md) (created 2026-07-09 session #77, director
directive): root cause PINNED to `ast_based_generator.rs:2917-2928` — the non-`Or` `rule_body_inner`
inits `semantic_raw_content = None` and `#post_parse_transform_tokens` shadows `result` with the
shaped Json without ever capturing the raw content (the `Or` path does, at `:3358`/`:3377`/`:3731`);
`.1` will audit the `SEMREF-SHAPED` blast radius before the codegen fix lands in `.2`.

#### `.4.3` Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — `pcre2test` 10.47: `x{5,4}` / `a{\t5\t,\t2\t}` / `a{5,2}` REJECT err 104
  "numbers out of order in {} quantifier"; `{4,5}` `{5,5}` `{5,}` `{,5}` `{5}` compile-clean. Message-source
  probe (release `parseability_probe --parse regex`, before the fix): `x{5,4}` REJECTs with the VALIDATOR
  message ("counted quantifier minimum cannot exceed counted quantifier maximum") firing only AFTER a
  grammar-accept ⇒ the GRAMMAR ALONE accepts these PCRE2-invalid patterns.
- [x] **ROOT CAUSE (WHY + WHERE)** — the min>max ORDER reject lived OUT-OF-BAND in
  `regex_compile_validation.rs::validate_counted_quantifier_body` (`:433-440`, the `minimum > maximum` check),
  reached from `find_invalid_counted_quantifier` (`:162`), invoked as a post-parse contract at
  `validate_regex_compile_contract:46` — a single-source-of-truth hole ([[project_ebnf_is_single_source_of_truth]]).
  The grammar's `counted_quantifier_body` range branch (`grammars/regex.ebnf:230`) accepted min>max structurally.
  (The `value_compare`-resolution mechanism is the SURFACED FINDING above, WHY+WHERE tool-traced.)
- [x] **FIX** — fix-hierarchy tier-1 (existing declarative `@predicate` primitive, no engine change): extract
  `counted_quantifier_range` gated by `@predicate value_compare [$min, le, $max] phase:post view:shaped`
  (named refs to the shaped `{min,max}` — see the SURFACED FINDING for why raw positional fails); reference it
  as body branch 0 (`-> $1`). Validator: delete `find_invalid_counted_quantifier` +
  `validate_counted_quantifier_body` + the call + the 3 min>max tests; ADD the `parser_registry.rs` pin.
- [x] **ADDRESSED (verified)** — after regen + both-binary rebuild, the message-source probe shows `x{5,4}` /
  `a{05,4}` / `a{\t5\t,\t2\t}` / `a{ 5 , 2 }` / `a{10,2}` flip to the GRAMMAR message (`Parser did not consume
  full input`) while the controls `{4,5}` `{5,5}` `{05,5}` `{0,65535}` `{5,}` `{,5}` `{5}` `{\n5,2\n}` stay
  ACCEPT. Every verdict matches `pcre2test` 10.47. The pin
  `regex_counted_quantifier_order_rejects_at_the_grammar_layer_pcre2_faithfully` (6 reject + 8 accept + 3
  relaxed) is GREEN.
- [x] **NO REGRESSION** — regex cert-coverage `total=239 witness=239 UNKNOWN=0 fully_certified=true spf=0` at
  seeds 0/7/42; `regex_pcre2_compile_oracle_gate` byte-identical baseline `2189/1858/285/46`;
  `duality_hunt_gate` regex 3 seeds NO new/vanished signature (only the start-option class);
  `parse_harness_equivalence_gate` regex byte-identical (4 gate tests pass); `regex_ast_shape_contract_gate`
  aligned (inventory 220→221); dual `--lib` suite (883 pass with generated_parsers after the version lockstep
  — −3 deleted validator tests + 1 pin); `--lint-grammar` 0 errors (239 rules); the other 5 fully-certified
  grammars untouched (only `regex_parser.rs` regenerated).
- [x] **LOCKSTEP** — `grammars/regex.ebnf`; `regex_compile_validation.rs`; `parser_registry.rs` pin;
  `embedding_api.rs` consts `1.1.94`/`1.1.96`; `regex_parser_integration_contract_v1.json`; `regex_v1.json`
  manifest inventory 220→221; contract Identity + `1.1.94`/`1.1.96` Highlights; ledger `REGEX-0104`; regex book
  (`rules-quantifier` § `counted_quantifier_range` + `compile-contract-validator` + `changelog-index` + tracked
  HTML); top book `parser-families.md`; `CHANGES.md`; `DEVELOPMENT_NOTES.md`; `LIVE_ACHIEVEMENT_STATUS.md`;
  `MEMORY.md`; `docs/TASK_TREE.md`.

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

- **(2026-07-09, session #80)** `.4.5.b` LANDED (`PGEN-REGEX-PCRE2-0033`, RELEASED regex slice — release
  `1.1.96`→`1.1.97`, contract `1.1.98`→`1.1.99`, schema `1`, ledger `REGEX-0107`) — the non-`[` NONLITERAL
  class-range reject (shorthand `\d \D \h \H \s \S \v \V \w \W` + property `\p…`/`\P…` endpoints, err 150) is now
  GRAMMAR-owned via a zero-width `!invalid_class_range` negative lookahead on the three class-item positions
  (`*_core` passthroughs `-> $2` — a NAMED core rule, since inline `(…)` corrupts positional refs). Behavior-NEUTRAL
  at the released `--parse` surface (the validator still rejects — its range-check deletion waits until the whole
  family is grammar-owned), so the before→after is proven at the GRAMMAR layer by the certified interpreter
  (`rust/tests/regex_class_range_nonliteral_grammar_migration.rs`: 23 invalid ranges flip ACCEPT→REJECT, 16
  carve-outs + 22 valid ASTs stay identical). cert **245/245 UNKNOWN=0** ×0/7/42 (239→245: +3 witnessed `*_core`,
  +3 lookahead-only PROOF rules); oracle byte-identical `2189/1867/274/48`; duality 9 lanes; equivalence + combinator
  27/27 + semantic 32/32; ast_shape 221→224. SCOPE: shorthand + property only; POSIX-left + the `-[`-right (`.4.5.a`)
  stay validator-owned. Frontier → **`.4.5.c`** (non-`[` DESCENDING reject to grammar; engine-tier `value_compare`
  codepoint-coercion WIDENING decoding `\x{}`/`\NNN`/`\cX`/`\a`/`\e` — MUST also migrate the `.4.5.a` (`-[`) +
  `.4.5.a.1` (`\v`/`\V`) + POSIX-left cases so the whole class-range family is grammar-owned, THEN delete the
  `find_invalid_char_class_construct` range-check) → `.4.7`..`.4.12` → final `.4` deletion → `.5`.
- **(2026-07-09, session #79)** `.4.5.a.1` LANDED (`PGEN-REGEX-PCRE2-0032`, RELEASED regex slice — release
  `1.1.95`→`1.1.96`, contract `1.1.97`→`1.1.98`, schema `1`, ledger `REGEX-0106`) — the `\v` / `\V` class-range
  accepts-invalid FIX (a genuine correctness fix, sibling of `REGEX-0105`), surfaced by the `.4.5.b` tools-first
  investigation. The vertical-whitespace shorthands `\v` / `\V` were treated as LITERAL range endpoints, so
  `[\v-x]` / `[\V-x]` / `[a-\v]` were accepted where PCRE2 10.47 rejects err 150; two-halved root cause —
  `is_nonliteral_class_escape` omitted `v`/`V` (validator) and `class_range_literal_escape_letter_strict`
  listed `v`/`V` (grammar). FIX makes `\v`/`\V` byte-identical to `\h`/`\H`: dropped `'V'`/`'v'` from the
  range-letter set (members via `class_simple_escape` untouched ⇒ `[\v]`/`[\V]` valid; stops `\v`-range
  generation ⇒ duality-safe) + added `b'v'|b'V'` to `is_nonliteral_class_escape` (err-150 reject). Behavior-CHANGING
  (3 cells flip ACCEPT→REJECT); regenerated `generated/regex_parser.rs`; cert 239 / ast_shape 221 UNCHANGED;
  oracle byte-identical `2189/1867/274/48` (corpus-invisible). Full release-probe matrix == oracle (0
  divergences); duality 9 lanes no new/vanished; equivalence byte-identical; new validator pin
  `rejects_vertical_whitespace_shorthand_class_range_endpoints`. The class-range family stays validator-owned
  pending `.4.5.b`/`.4.5.c`, now over a UNIFORMLY-correct nonliteral family (`d D h H s S v V w W p P`).
  Frontier → **`.4.5.b`** (grammar-migrate the nonliteral class-range family; a negative-lookahead
  `!invalid_class_range` at `class_item`, nonliteral atom set = shorthand `\d\D\h\H\s\S\v\V\w\W` + property
  `\p\P`, NOT single-char escapes) → `.4.5.c` (descending, engine-tier `value_compare` codepoint-widening) →
  `.4.7`..`.4.12` → final `.4` deletion → `.5`.
- **(2026-07-09, session #78)** `.4.5.a` LANDED (`PGEN-REGEX-PCRE2-0031`, RELEASED regex slice — release
  `1.1.94`→`1.1.95`, contract `1.1.96`→`1.1.97`, schema `1`, ledger `REGEX-0105`) — the `-[` / `-||`
  class-range accepts-invalid FIX. A range whose right endpoint began with `[` or `||` inside a NORMAL class
  (`[a-[b]]` descending, `[x-[:alpha:]]` nonliteral, `[~-||]` descending, `[\d-[z]]`) was accepted by the
  released parser (grammar AND validator) but PCRE2 10.47 rejects it — the validator's `scan_char_class`
  gated range detection on the mis-scoped `dash_starts_alt_extended_class_operator` guard (an ALTERNATE
  extended-class `(?[...])` operator, but `scan_char_class` runs only on NORMAL classes). VALIDATOR-tier fix
  (the class-range family stays validator-owned pending `.4.5.b`/`.4.5.c`): guard removed from both branches
  + deleted; new `scan_class_bracket_token` makes `read_substantive_class_atom` classify a `[:..:]`/`[...]`/`[=..=]`
  right endpoint NON-LITERAL. AST-dump PROVED the grammar already forms the range correctly (the investigation's
  grammar-side note was imprecise) ⇒ NO grammar/codegen/generated change (cert 239, ast_shape 221, duality
  UNCHANGED). Oracle NET IMPROVEMENT `2189/1867/274/48` (was `2189/1858/285/46`) — ~11 default-mode accepts-invalid
  fixed; the 2 new false-rejects `[\d-[z]]`/`[\d-||z]` are `alt_extended_class`-modifier corpus cells (a
  non-default mode PGEN does not model), ratchet 46→48 (env v12). Stale `1.1.27` test that locked in the bug
  DELETED. 🔎 NEW `.4.12` opened (standalone collating `[[.a.]]`/equivalence `[[=a=]]` = err 113, still accepts-invalid,
  distinct class). Frontier → `.4.5.b`/`.4.5.c` (grammar-migrate the whole class-range family, then delete the
  validator range checks) → the other hard/two-pass families (`.4.7`,`.4.8`,`.4.9`,`.4.10`,`.4.11`,`.4.12`) →
  final `.4` deletion → `.5`.
- **(2026-07-09, session #76)** `.4.2` LANDED (`PGEN-REGEX-PCRE2-0027`, RELEASED regex slice — release
  `1.1.92`→`1.1.93`, contract `1.1.94`→`1.1.95`, schema `1`, ledger `REGEX-0103`): `\k`/group NAME validity
  is now GRAMMAR-owned — the design-focused session the `.4.2` SCOPING called for. Tool-backed (`pcre2test`
  10.47): name ≤ 128 CODE UNITS (err 148) UNIFORM across every position; code-unit = byte@8-bit /
  code-point@32-bit; RGX Unicode-only ⇒ 128 code points ⇒ pure-grammar `{0,127}` (BOUNDED-QUANT.1), NO new
  primitive. 3 grammar edits: `name` `*`→`{0,127}`; `!"k"` on `simple_escape` + `'k'` dropped from
  `simple_escape_letter_strict` (`.4.1` `\p`/`\P` precedent); `\k'name'` quote branch added to `backreference`.
  `find_invalid_named_escape_or_group_name` + 5 exclusive helpers + `PCRE2_MAX_NAME_SIZE` + 3 tests DELETED
  (11th check migrated, 4th `.4` deletion-prep child; function FULLY retired). NOT byte-neutral — 2 fidelity
  refinements: `\k'name'` shape corrected (shorthand+literals→`backreference`) + multi-byte 65–128 code-point
  names now ACCEPT (was 8-bit-BYTE validator-REJECT), Unicode-only-faithful. VERIFIED tools-first: message-source
  probe 10 malformed `\k` flip VALIDATOR→GRAMMAR-reject both profiles; `\k'n'`→proper backref AST; `\d`→
  `char:d` (`$5`→`$6` correct); 128 ACCEPT/129 REJECT (ASCII); 65×é ACCEPT/129×é REJECT (code-point, =32-bit
  PCRE2). cert 238/238/0 ×seeds 0/7/42; oracle byte-identical `2189/1858/285/46`; duality 9 lanes; differential
  CERTIFIED; ast_shape 219→220; dual lib `887/0`; drift gate green. Frontier → the hard/primitive families
  (`.4.3`,`.4.5`,`.4.7`,`.4.8`,`.4.9`,`.4.10`,`.4.11`) → final `.4` deletion → `.5`.
- **(2026-07-09, session #75)** `.4.4` LANDED (`PGEN-REGEX-PCRE2-0025`, RELEASED regex slice — release
  `1.1.91`→`1.1.92`, contract `1.1.93`→`1.1.94`, schema `1`, ledger `REGEX-0102`) — the escape-in-class
  rejects (`\A \B \C \G \K \N`-unbraced `\R \X \Z \z`) are now GRAMMAR-owned (behavior-NEUTRAL
  validator→grammar migration, the 3rd `.4` deletion-prep child, the 10th compile-contract check migrated).
  Three reach paths closed: 10 guards `!"A"…!"z" !( "N" !"{" )` on both `class_simple_escape` variants
  (`char:$23`/`$17`); `single_byte_escape` dropped from `class_escape_unit` (kept in `escape_unit` for the
  pattern body); `A`/`G`/`z` dropped from `class_range_literal_escape_letter_strict` (range-endpoint
  reachable). `!( "N" !"{" )` keeps `\N{…}` valid. The two `read_class_atom` reject blocks + 3 validator unit
  tests deleted; `parser_registry.rs` pin `regex_class_escapes_reject_at_the_grammar_layer_pcre2_faithfully`
  added. VERIFIED: message-source probe 18 reject forms flip VALIDATOR→GRAMMAR-reject (both profiles), 28
  controls stay ACCEPT (`pcre2test` 10.47 agrees); cert 238/238/0 spf=0 ×seeds 0/7/42; oracle byte-identical
  `2189/1858/285/46`; duality 9 lanes no new signature; equivalence byte-identical; ast_shape aligned
  (manifest `$7`→`$17` / `$13`→`$23`, text-only positional bump; inventory stays 219). NO new rule → cert
  stays 238. Frontier per the standing PNT order → `.4.2` (`\k`/group NAME charset+len) → the
  hard/primitive-needing families (`.4.3`,`.4.5`,`.4.7`,`.4.8`,`.4.9`,`.4.10`,`.4.11`) → final `.4`
  deletion → `.5`.
- **(2026-07-09, session #74)** `.4.6` LANDED (`PGEN-REGEX-PCRE2-0024`, RELEASED regex slice — release
  `1.1.90`→`1.1.91`, contract `1.1.92`→`1.1.93`, schema `1`, ledger `REGEX-0101`) — POSIX character-class NAME
  validity is now GRAMMAR-owned (behavior-NEUTRAL validator→grammar migration, the 2nd `.4` deletion-prep
  child). The class-member `[` literal is guarded by an inline `[:…:]`-posix-shape negative lookahead
  (`class_member_literal`/`class_member_literal_nocaret`), so an unknown name has no fully-consuming parse →
  grammar-REJECT; `is_valid_posix_class_name` + 2 validator tests deleted (`scan_posix_class` recognition
  retained for range analysis). VERIFIED: 33-cell matrix 100% == `pcre2test` 10.47; cert 238/238/0 spf=0
  ×seeds 0/7/42; oracle byte-identical `2189/1858/285/46` (a first `!"]"` PCRE2-exact attempt regressed +1
  false-accept on the escaped-`]` case `[abc[:x\]pqr:]]` — reverted); duality 9 lanes no new signature;
  equivalence + ast_shape (217→219) green; dual lib 891/0. Honest bound (pre-existing): the scan crosses
  escaped/unescaped `]` (like the validator) vs PCRE2's unescaped-only boundary (`[x[:foo]bar:]y]`) — a
  PCRE2-exact `]`-boundary follow-up. Frontier per the standing PNT order → `.4.4` (`\B`/`\K`/`\N`-in-class,
  STRUCTURAL) → `.4.2` (name charset+len) → the hard/primitive-needing families (`.4.3`,`.4.5`,`.4.7`,`.4.8`,
  `.4.9`,`.4.10`,`.4.11`) → final `.4` deletion → `.5`.
- **(2026-07-09, session #72)** `.4` capstone RE-SCOPED (`PGEN-REGEX-PCRE2-0022`, PURE-DOCS — no
  release/contract/schema bump; stays `1.1.89`/`1.1.91`/`1`). 🔎 Tool-backed finding: the `MEMORY`
  "only the start-option POSITION rule remains validator-owned" was WRONG. A message-source probe (the
  parse path rejects with a GRAMMAR message vs a VALIDATOR "…compile contract" message —
  `parser_registry.rs:393-394`) of `validate_regex_compile_contract`'s 58-input pinned reject corpus
  proved **8 check families / 54 inputs are still LOAD-BEARING** (grammar accepts, only the validator
  rejects); only 4 (empty-`\Q\E`-region) are grammar-shadowed. Deleting the validator today = 54
  accepts-invalid PCRE2 divergences. Root of the wrong belief: DUALITY-clean (the generator never EMITS
  these) ≠ VALIDATOR-deletable (a user can HAND-WRITE them). `.4` re-scoped into encode-per-family child
  leaves `.4.1`..`.4.11` (see the SCOPING LOG + [[project_regex_validator_deletion_blocked_load_bearing]])
  THEN a final deletion; several need a NEW parser-agnostic primitive (value-comparison / whole-pattern
  two-pass / contextual). Recommended next actionable slice = `.4.1` (bare property escape, STRUCTURAL,
  low-risk) — a released slice, best in fresh context. Frontier per the standing PNT order → `.4.1` →
  `.4.6`/`.4.4`/`.4.2` (structural) → the hard/primitive-needing families → final `.4` deletion → `.5`.
- **(2026-07-08, session #71)** `.3.23` LANDED (`PGEN-REGEX-PCRE2-0021`, regex release `1.1.88`→`1.1.89`,
  contract `1.1.90`→`1.1.91`, schema `1`, ledger `REGEX-0099`) — the PCRE2 `\Q` QUOTING model made
  first-class (`Q` dropped from `simple_escape_letter_strict`), closing TWO sibling divergence classes with
  ONE root cause: (a) empty-`\Q\E`-quantified accepts-invalid (`\Q\E*` `\Q\E{2}` now REJECT err-109) — the
  `.3.19`-deferred target; AND (b) 🔎 NEW unterminated-`\Q…`-metachar-tail REJECTS-VALID (`\Q)` `\Q(` `\Q[`
  `\Q^` `\Q|` `\Q(?:` now ACCEPT — PCRE2 quotes to end-of-pattern). NEW rules: `unterminated_quoted_literal`
  (a NON-atom quote-to-end `piece`) + `empty_quoted_literal` (a non-quantifiable `zero_width`). Two in-slice
  root-causes tools-first: the leaf's stated empty-quantified blocker (the `\Q`-as-`simple_escape`
  entanglement) AND a cert seed-42 duality break (the generator over-generated `\Q\E{0,0}` when unterminated
  was a quantifiable atom — fixed by the non-atom piece, generation-faithful, no lookahead). Validated
  pre-rebuild by the regex-CERTIFIED interpreter (85/85 vs oracle, 23 intended AST changes). Cert 236/236/0
  spf=0 seeds 0/7/42 (+ 8 more seeds); oracle gate byte-identical `2189/1858/285/46`; equivalence gate ✅;
  duality gate 9 lanes NO new signature; ast_shape 214→217; dual suite 893/0. Full evidence: the `.3.23`
  leaf checklist. Frontier per the standing PNT order → capstone `.4` (delete the validator; owns the
  start-option POSITION duality re-baseline AND the `.3.22` named-reference two-pass check) → `.5`
  (verification).
- **(2026-07-08, session #70)** `.3.22` SCOPED + CLOSED (`PGEN-REGEX-PCRE2-0020`, PURE-DOCS — no release/contract
  bump; release stays `1.1.88`, contract `1.1.90`, schema `1`; ledger `REGEX-0098` **Deferred**). The
  named-reference UNKNOWN-name family — PGEN `1.1.88` ACCEPTS 9 spellings (`\k<zzz>`/`\k'zzz'`/`\k{zzz}` +
  `(?P=zzz)` + `\g{zzz}` + `(?&zzz)`/`(?P>zzz)`/`\g<zzz>`/`\g'zzz'`) that `pcre2test` 10.47 rejects err 115;
  controls (define-then-ref + FORWARD-ref + subroutine) parity-ACCEPT. Tools-first re-verified this session
  (oracle + released probe). Root cause: NO parse-side named-reference inventory (grammar refs ungated; validator
  shape-only; `regex_capture_name` is `@gen_emit_fact`-only). The fix is inherently WHOLE-PATTERN two-pass
  (forward refs LEGAL ⇒ a single-pass `has_fact` would reject `\k<aa>(?'aa'x)`), so it is DEFERRED to capstone
  `.4` (same two-pass class as start-option POSITION); the oracle matrix + encode design are the frozen `.4`
  acceptance spec (leaf SCOPING LOG). NOTE: `\g1`@0-groups (numeric single-digit N<10 Non-Goal, REGEX-0083/0086)
  is a related-but-DISTINCT pre-existing divergence, not this named family. Frontier per the standing PNT order →
  `.3.23` (empty-`\Q\E`-quantified, blocked on the `\Q`-model) → capstone `.4` (delete the validator; owns the
  start-option POSITION duality re-baseline AND now the `.3.22` named-reference two-pass check) → `.5`.
- **(2026-07-08, session #68)** `.3.19` LANDED (`PGEN-REGEX-PCRE2-0017`, regex release `1.1.85`→`1.1.86`,
  contract `1.1.87`→`1.1.88`, schema `1`, ledger `REGEX-0095`) — a quantifier on a **stray `\E`**
  (PCRE2 zero-width but TRANSPARENT, unlike an opaque anchor) now rejects err-109-faithfully, grammar-encoded:
  stray `\E` re-homed out of quantifiable `atom` into a non-quantifiable `zero_width` piece (`'E'` dropped
  from `simple_escape_letter_strict`, positive-exclusion), plus a `piece` ABSORPTION branch
  (`atom zero_width+ quantifier`) that lets a quantifier reach THROUGH the transparent `\E`(s), eliding them
  (PCRE2 delete-`\E` model). 11 accepts-invalid spellings (bare `\E*`/`\E{2}`/`\E\E*`, anchor-blocked
  `^\E*`/`a^\E*`, group/alt-edge `(\E*)`/`|\E*`) fixed; 8 `<atom>\E<quant>` AST semantic-corrections; all
  else byte-identical. Two in-slice adjudications: the leaf's `anchor !quantifier` hypothesis REFUTED
  tools-first by the oracle (anchors opaque, stray `\E` transparent); the full-family design REFUTED by the
  interpreter (empty-`\Q\E` unmasks the `\Q`-as-`simple_escape` model) → empty-`\Q\E`-quantified SPUN OUT to
  `.3.23`. Cert 230/230/0 ×3 seeds (2 new rules witnessed, no gap); dual suite 890/0; oracle gate
  byte-identical (conformance-neutral, `.3.13` precedent). Full evidence: the `.3.19` leaf checklist.
  Frontier per the standing PNT order → `.3.20`–`.3.23` (`.3.20` quantified-verb, needs its relaxed-semantics
  decision + gate-contract re-baseline; `.3.21` LIMIT value range, needs the structural-shape decision;
  `.3.22` named-reference inventory; `.3.23` empty-`\Q\E`-quantified, blocked on the `\Q`-model) → capstone
  `.4` → `.5`.
- **(2026-07-08, session #67)** `.3.18` LANDED (`PGEN-REGEX-PCRE2-0016`, regex release `1.1.84`→`1.1.85`,
  contract `1.1.86`→`1.1.87`, schema `1`, ledger `REGEX-0092`/`0093`/`0094`) — the full PCRE2 brace
  tokenization model grammar-encoded per the `-0015` frozen scoping: value-structural `quant_bound_number`
  (≤65535 by construction; the u32-overflow hole closed), the inline `literal_open_brace` negative-lookahead
  guard (the err-109 position class closed), `brace_ws` (space+tab, oracle-frozen via hex cells) end-to-end,
  the validator narrowed to tab-aware min>max ORDER only (6th migration; two MORE ws-set divergences found +
  fixed in-slice: tab-order accepts-invalid, newline-brace rejects-valid). Duality-gate contract re-baselined
  same-commit (same 2 adjudicated classes, owners `.3.20`/`.4` unchanged — lane/seed shifts only, NO novel
  class); the oracle-corpus normalizer now skips pcre2test `expand`-modifier lines (an ingestion soundness
  defect the ratchet surfaced; baseline env v11, false-reject ratchet EXACT at 46, false-accepts 292→286).
  Cert 228/228/0 ×3 seeds; suites 889/847/768. Full evidence: the `.3.18` leaf checklist. Frontier per the
  standing PNT order → `.3.19`–`.3.22` (`.3.19` zero-width-quantified family; `.3.20` quantified-verb, needs
  its relaxed-semantics decision + the gate-contract re-baseline; `.3.21` LIMIT value range, needs the
  structural-shape decision; `.3.22` named-reference inventory, sequenced after) → capstone `.4` → `.5`.
- **(2026-07-08, session #66)** `STIMULI-SIGNOFF.13.3` LANDED (`PGEN-STIMULI-SIGNOFF-0018`) — the
  duality-hunt gate lane + scaled enumeration that CLOSES the `STIMULI-SIGNOFF.13` node. Regex
  consequence: the hunter-visible residual universe is now PINNED in
  `rust/test_data/grammar_quality/duality_hunt_gate_contract_v0.json` — exactly 2 classes, both owned
  here (quantified-verb → `.3.20`; start-option-position → capstone `.4`); a 22,000-sample enumeration
  (16k directed 8 seeds + 6k diverse replay) found NO third class, and `.3.18`'s counted-quantifier
  class stayed latent even at scale (its leaf note stands: the generator CAN emit `a{5,2}`, just rarely).
  The `.3.20`/`.4` closing slices must re-baseline the gate contract same-commit (notes added to both
  leaves). Frontier per the standing PNT order → `.3.18`–`.3.22` below. SAME SESSION, `.3.18` was then
  SCOPED tools-first (`PGEN-REGEX-PCRE2-0015`, PURE-DOCS): the full PCRE2 10.47 brace-tokenization model
  oracle-pinned and 🔎 FOUR NEW released-parser accepts-invalid divergences CONFIRMED (`a{4294967296}`
  validator-u32-overflow hole + the err-109 position family `{2,5}`-at-start / `x|{2,5}` / `a{2}{3}`) —
  full matrix + encode design in the `.3.18` leaf; implementation = the next fresh-session slice.
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
