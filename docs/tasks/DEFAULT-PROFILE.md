# DEFAULT-PROFILE — `@default_profile`: replace the regex→`pcre2` default-profile name-gates with a declarative grammar-level directive

- Status: `active` (created 2026-07-07, session #55). `.1` DESIGN = this document.
  FRONTIER = `.2` implementation.
- Roadmap lane: cross-cutting engine correctness / EBNF-single-source-of-truth — WHICH dialect
  profile a parser runs under when the caller specifies none is an ACCEPTANCE-RELEVANT behavior
  (it decides whether `@profiles:["relaxed"]`-gated constructs are accepted), and today it is
  decided by comparing the grammar's NAME to the string literal `"regex"` in three engine
  surfaces. This violates the same two standing director doctrines the completed sibling tree
  `WS-DIRECTIVE` just enforced for layout policy:
  [[feedback_features_parser_agnostic_enable_all_parsers]] (2026-06-08: *"capability-gated,
  never grammar-name-gated"*) and [[project_ebnf_is_single_source_of_truth]] (the EBNF + its
  annotations are the single source of truth for what a parser accepts).
- Origin: routed finding **F1** of `WS-DIRECTIVE` §6 (established tools-first 2026-07-07,
  session #54; surfaced to the director in that session's callout). Picked up by PNT
  (session #55) as the top routed layer-A frontier candidate — the same doctrine-violation
  class as the tree that just completed, with the directive shape already director-endorsed
  by the WS-DIRECTIVE precedent.

## 1. Problem statement (tool-backed, verified against HEAD `f9d5885a`, 2026-07-07)

The knowledge "an unspecified profile for the grammar named `regex` means strict `pcre2`;
`relaxed` is the opt-out" (REGEX-PCRE2-FIDELITY.2/.3.1) lives in engine string literals at
three boundaries, NOT in `grammars/regex.ebnf`:

- **Parse side** — `rust/src/parser_registry.rs:138-143`
  (`normalize_generated_grammar_profile`): `if grammar_name == "regex"` → unspecified/empty
  requested profile ⇒ `pcre2`; `relaxed` (case-insensitive) passes; **any other explicit
  value is silently coerced to `pcre2` too**. Exposed as
  `active_grammar_profile` (`:126-128`), the profile oracle the parse-harness interpreter
  (`parse_harness_interpreter.rs:126`, `:209`) and the differential-equivalence gate
  (`parse_harness_equivalence.rs:331`, `:1002`) normalize through so they match `parse_sample`
  byte-for-byte. The registry's own regex wrappers re-invoke it with the literal name at
  `parser_registry.rs:362/:374/:398`.
- **Generation side** — `rust/src/main.rs:2254-2258` (`apply_grammar_profile_filter`):
  `None if grammar.grammar_name == "regex" => Some("pcre2")` — the generation twin, so
  default-mode stimuli exclude the `relaxed`-only constructs the default-mode parser rejects.
- **Embedding side** — `rust/src/embedding_api.rs:1531` + `:1559`
  (`parse_generated_regex` / `parse_generated_regex_ast_json`): explicit
  `parser.set_grammar_profile(Some("pcre2"))` calls, each with a comment warning that the
  codegen profile guard treats `None` as permissive — i.e. every embedder must REMEMBER the
  default because the artifact itself doesn't carry it.

Mechanics that make the gates load-bearing (read this session): the generated parser's
`grammar_profile` field initializes to `None` (codegen `ast_based_generator.rs:582`), is set
via `set_grammar_profile` (codegen `:983-987`, emitted per parser at `:1512`), and the guard
`rule_profile_is_enabled` (codegen `:5046-5057`; e.g. `generated/regex_parser.rs:252123`)
treats `None` as **permissive** (all rules active, `eq_ignore_ascii_case` membership
otherwise). `@profiles` branch guards are emitted at codegen `:2830`.
`grammars/regex.ebnf` gates 5 constructs `@profiles: ["relaxed"]` (lines 489/538/648/679/1132).

Consequences (why this is a defect, not a style nit):

1. **The `.ebnf` lies by omission.** Nothing in `grammars/regex.ebnf` says its default
   dialect is strict `pcre2`; the acceptance-relevant default lives in engine literals across
   three files. The stimuli generator, the interpreter, and any future consumer of the
   gen-AST cannot see it — the exact invisible out-of-band acceptance-constraint class
   `EBNF-SOURCE-OF-TRUTH` exists to eliminate.
2. **The capability is closed to every other grammar.** No other grammar — shipped or
   synthetic — can declare "my unspecified-profile default is P". A synthetic profiled
   grammar in the parse harness is permissive-by-default with no way to say otherwise; a
   future profiled deliverable grammar would need an ENGINE edit (the anti-pattern the
   2026-06-08 doctrine forbids).
3. **The default is caller-owned instead of artifact-owned.** Every boundary
   (registry wrapper, embedding function, any future direct constructor) must re-apply
   `Some("pcre2")` by hand — `embedding_api.rs` carries two such call sites with warning
   comments; forgetting one silently flips regex to permissive. That is a standing footgun,
   not a hypothetical (the comments exist precisely because it was once forgotten).

NOT in this class (verified): `parse_harness_interpreter.rs:2158`,
`annotation_validator.rs:757/:2223`, `ast_based_generator.rs:3843/:4417/:4612/:7436`,
`stimuli_generator.rs` `"regex"` hits — all token-TYPE tag dispatch (`regex` the token kind),
not grammar-name gates. `parser_registry.rs:1198/:1269/:1326/:1399` are the blessed
data-driven per-grammar registry table dispatch, not default-profile knowledge.

## 2. Design (leaf `.1`, this document)

### D1 — Surface

A **grammar-level** semantic directive, following the `@whitespace_sensitive` /
`@fact_kind:` precedent (free-standing `@name: payload` line; mechanically binds to the
following rule, compiled by a grammar-wide sweep):

```ebnf
@default_profile: pcre2
```

Payload = ONE profile name (identifier-shaped scalar; trimmed; must be non-empty). It names
the profile an **unspecified or empty** requested profile resolves to — on the parse side,
the generation side, and every harness surface. Parser-agnostic: ANY grammar may declare it.
Parses through the existing generic annotation surface (`custom_annotation` +
scalar-payload precedent) — **zero meta-grammar change**; the dual-run gate verifies.

### D2 — Semantics and defaults

- Absent directive ⇒ unspecified profile stays `None` (the permissive guard) — today's
  behavior for every grammar except regex, so all other grammars are untouched by
  construction.
- Duplicate declarations with IDENTICAL payloads are tolerated; CONFLICTING payloads are a
  hard compile error (mirrors `compile_layout_sensitivity` / `@fact_kind` V-DECL-1).
- Malformed payload (empty, non-scalar, or non-identifier-shaped) ⇒ hard compile error
  naming the directive and the accepted form.
- An EXPLICIT requested profile always wins over the declared default. The current
  regex-only quirk "any explicit non-`relaxed` value coerces to `pcre2`"
  (`parser_registry.rs:141`) is REPLACED by generic pass-through — acceptance-equivalent for
  the shipped regex grammar because the emitted guard tests case-insensitive membership in
  the `@profiles` lists and the only listed value is `relaxed`: an unknown explicit profile
  disables exactly the same rule set as `pcre2`. (`.2` proves this with a probe; the only
  observable delta is the profile STRING an observer reports for a bogus request, not
  acceptance.)

### D3 — The artifact carries its own default (parse side)

Codegen, for a directive-bearing grammar ONLY, emits into the generated parser:

1. `pub const DEFAULT_GRAMMAR_PROFILE: &str = "<payload>";`
2. constructor initialization `grammar_profile: Some(DEFAULT_GRAMMAR_PROFILE...)` instead of
   `None`,
3. `set_grammar_profile(None)` RESTORES the declared default (it does not fall back to
   permissive-`None`) — "unspecified means the declared default" holds at every entry point,
   including direct constructors, with no caller cooperation.

For grammars WITHOUT the directive, the emitted code is byte-for-byte today's — so the 10
non-regex parsers regenerate **byte-identical** (`cmp`), the decisive no-regression oracle
for them. The regex artifact intentionally changes (it now embodies its declared default) —
its oracles are the behavior gates in §4.

### D4 — The name-gates are DELETED, not kept as fallback

- `parser_registry.rs:138-143` regex arm deleted. `active_grammar_profile` resolves an
  unspecified profile through a per-grammar datum in the EXISTING registry table
  (populated from the generated parser's `DEFAULT_GRAMMAR_PROFILE` constant — single source
  of truth: the grammar, carried by codegen, read by the registry; same pattern as the
  table's `parse_and_cover` generated-symbol references). Generic trim/empty→`None`
  normalization stays engine-generic.
- `main.rs:2256` generation arm replaced by consulting the compiled directive on the loaded
  grammar (one shared compile fn, D6).
- `embedding_api.rs:1531`/`:1559` explicit sets deleted — the D3 constructor default owns it.

### D5 — The interpreter derives from the SAME compiled directive

The parse-harness interpreter resolves its effective profile as
requested-or-compiled-default from the loaded grammar's annotations (general — works for
arbitrary/synthetic grammars, not just registered ones), keeping byte-parity with the
generated parser via the single compile fn instead of via the registry name-oracle.
`active_grammar_profile` remains correct for registered grammars through D4.

### D6 — Compile shape (mirrors `compile_layout_sensitivity`)

`rust/src/ast_pipeline/semantic_runtime.rs` gains
`DEFAULT_PROFILE_DIRECTIVE_NAME: &str = "default_profile"` +
`compile_default_profile(&Annotations) -> Result<Option<String>, String>` (grammar-wide
sweep over rule- and branch-level annotations; D2 duplicate/conflict/malformed semantics)
with a `pub(crate)` payload parser shared with the annotation validator (no second dialect).
**NO new `SemanticRuntimeDirective` enum variant** (the WS-DIRECTIVE.2 refinement: the
directive is compile-time-only and must never enter `directives_by_rule`, or every generated
parser's serialized directive tokens would change and break the D3 byte-identity oracle for
the 10 non-bearing parsers).

### D7 — Wiring points (implementation map for `.2`)

1. `semantic_runtime.rs` — constant + `compile_default_profile` + payload parser (D6).
2. `semantic_directive_registry.rs` — catalog entry (`ParserSteering`), placed with the
   `whitespace_sensitive` entry (`:374-377`).
3. `annotation_validator.rs` — a `"default_profile"` arm beside `whitespace_sensitive`
   (`:544`), linting through the SAME payload parser.
4. `ast_based_generator.rs` — compile the directive once (beside the `:1264` layout compile);
   thread to the constructor/`set_grammar_profile` emitters (`:582`, `:983-987`) per D3.
5. `parser_registry.rs` — delete the regex arm; add the table datum + resolve through it (D4).
6. `main.rs::apply_grammar_profile_filter` — consult the compiled directive (D4).
7. `embedding_api.rs` — delete the two explicit sets (D4).
8. `parse_harness_interpreter.rs` — resolve requested-or-compiled-default (D5).
9. `grammars/regex.ebnf` — `@default_profile: pcre2` beside `@whitespace_sensitive: true`
   (line 18). Regen ALL shipped parsers; `cmp` per D3.
10. Consider a combinator/semantic-suite case (a synthetic `@profiles`-gated grammar with
    `@default_profile`) proving interpreter==oracle byte-identity for the directive; decide
    in `.2` with the suite's determinism constraints in view.
11. Lockstep (same commit): ebnf parser book (grammar-author surface: directive, payload,
    defaults, errors, examples), top book `annotation-system.md`,
    `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`,
    `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`,
    `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` + regex parser book (the
    pcre2-by-default statement becomes grammar-declared), TOOLBOX §1.7 if the suite grows,
    continuity docs (CHANGES / DEVELOPMENT_NOTES / MEMORY / TASK_TREE row).

### D8 — Explicitly out of scope (routed, §6)

The SystemVerilog profile-ALIAS map (`parser_registry.rs:148-158`: `"2017"`→`"sv_2017"`,
`"1364-2005"`→`"verilog_2005"`, …) — a name-gated alias table, a SIBLING sub-class
(request-spelling normalization, not defaulting) with its own design questions (routed §6
F-A). The svpp stimuli separator heuristic stays routed to `STIMULI-SIGNOFF` (WS-DIRECTIVE
§6 F2). Generation-side spacing policy remains out (WS-DIRECTIVE D7).

## 3. Tree

- `DEFAULT-PROFILE.1` — DESIGN (this document; docs-only commit). **DONE** when committed.
- `DEFAULT-PROFILE.2` — IMPLEMENT end-to-end per D3–D7 + full lockstep + verification (§4/§5).
  **Status: open (the frontier).**

## 4. Tree-level acceptance criteria

1. `@default_profile` compiles from the EBNF through ONE shared compile fn consumed by the
   validator, codegen, the generation-side filter, and the interpreter (no second dialect,
   no shadow default).
2. `grammars/regex.ebnf` declares `@default_profile: pcre2`; the `== "regex"`
   default-profile literals in `parser_registry.rs`, `main.rs`, and the two
   `embedding_api.rs` explicit sets are DELETED (delete-not-fallback).
3. Behavior preserved on every existing lane: regex cert `fully_certified` seeds 0/7/42; RGX
   conformance + broader corpus; differential-equivalence 11 CERTIFIED; embedding suite;
   features-on lib green.
4. The 10 non-directive-bearing parsers regenerate BYTE-IDENTICAL (`cmp`); only the regex
   artifact changes, and only by the D3 carrier/constructor/setter delta.
5. The capability is OPEN to any grammar: a synthetic `@profiles`-gated grammar with
   `@default_profile` rejects the gated construct by default and accepts it under the
   opt-in profile, interpreter and compile-and-run oracle agreeing byte-for-byte.
6. Books / normative spec / steering matrix / regex contract lockstep in the same commit.

## 5. Acceptance Checklist (enforced; leaf `.2` copies + earns this)

```markdown
## Acceptance Checklist (enforced)
- [ ] **REPRODUCE / ISSUE** — grep/read evidence of the three name-gate boundaries at HEAD +
  probe: a synthetic profiled grammar has NO way to declare a default (permissive by
  default through interpreter AND oracle), while regex gets pcre2 by name.
- [ ] **ROOT CAUSE (WHY + WHERE)** — the default-profile knowledge lives in engine literals
  (`parser_registry.rs:138`, `main.rs:2256`, `embedding_api.rs:1531/:1559`), invisible to
  the EBNF and closed to other grammars (§1 fact base, file:line-pinned).
- [ ] **FIX** — grammar-level `@default_profile` directive, tier: declarative/grammar +
  parser-agnostic engine wiring (fix-hierarchy: new annotation — levels 1/2 impossible, the
  capability does not exist declaratively).
- [ ] **ADDRESSED (verified)** — synthetic-grammar probe flips REJECT/ACCEPT correctly by
  profile with the directive; regex lanes byte-equivalent with the name-gates deleted.
- [ ] **NO REGRESSION** — 10 non-regex parsers `cmp` byte-identical; regex + svpp certs
  fully_certified seeds 0/7/42; RGX conformance/broader corpus; equivalence 4/4 (11
  CERTIFIED); combinator + semantic suites CLEAN; features-on lib green; dual-run ✅;
  clippy source-strict ok.
- [ ] **LOCKSTEP** — ebnf book + top book + normative spec + steering matrix + regex
  contract/book + continuity docs same-commit; both book gates ✅.
```

## 6. Findings routed elsewhere (surfacing directive)

- **F-A — SV profile-ALIAS name-gate.** `parser_registry.rs:148-158` maps request spellings
  (`"2017"`, `"ieee1800-2017"`, …) to canonical profile names for the grammar named
  `systemverilog`. Same doctrine tension, different capability (alias normalization). A
  future `@profile_alias` (or `@profiles`-declared alias list) directive is the principled
  shape. NOT owned here; listed for director visibility with this tree's session callout.

## 7. Verification log

- 2026-07-07 (session #55, `.1`): §1 fact base established by direct reads/greps at HEAD
  `f9d5885a` — the three name-gate boundaries + their comments; the generated-parser profile
  mechanics (constructor `None`, permissive guard, `@profiles` guard emission); the
  interpreter/equivalence observers of `active_grammar_profile`; the 5 `@profiles` sites in
  regex.ebnf; the not-in-class token-type-tag hits; the `@whitespace_sensitive` precedent
  chain (compile fn + validator arm + registry catalog entry).

## 8. Decisions

- D3 artifact-owned default over registry-only defaulting: eliminates the standing
  "every embedder must remember `Some(\"pcre2\")`" footgun class, not just today's instances.
- Delete-not-fallback (per WS-DIRECTIVE D4): a retained name-gate fallback would be a second,
  shadow source of truth — the defect class this tree removes.
- NO new runtime enum variant (per WS-DIRECTIVE.2 refinement): compile-time-only directive,
  never enters `directives_by_rule` — protects the 10-parser byte-identity oracle.
- The D2 explicit-value pass-through (dropping the regex-only coerce-to-pcre2 quirk) is a
  deliberate generalization, argued acceptance-equivalent in D2 and probe-verified in `.2`.
