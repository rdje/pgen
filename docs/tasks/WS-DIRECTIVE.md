# WS-DIRECTIVE — `@whitespace_sensitive`: replace the codegen grammar-NAME layout gate with a declarative grammar-level directive

- Status: `active` (created 2026-07-07, session #54). FRONTIER = `.2`.
- Roadmap lane: cross-cutting engine correctness / EBNF-single-source-of-truth — whether the
  generated parser auto-skips layout (whitespace/comments) is an ACCEPTANCE-RELEVANT behavior,
  and today it is decided by the grammar's FILE NAME, not by anything declared in the grammar.
  This violates two standing director doctrines:
  [[feedback_features_parser_agnostic_enable_all_parsers]] (2026-06-08: *"capability-gated,
  never grammar-name-gated"*) and [[project_ebnf_is_single_source_of_truth]] (the EBNF + its
  annotations are the single source of truth for what a parser accepts).
- Origin: the latent tension recorded 2026-07-05 during `PARSE-HARNESS.5.1`
  (decision memory `project_codegen_whitespace_sensitivity_grammar_name_gated`): the interpreter
  had to MIRROR the name-gate expression-for-expression to reach byte-identity; the principled
  fix named there is a `@whitespace_sensitive` directive. Picked up by PNT (session #54) as the
  only live, unblocked layer-A frontier candidate (design-heavy, fresh-session item).

## 1. Problem statement (tool-backed, verified against HEAD `a161939c`, 2026-07-07)

Parser codegen decides the three layout-policy booleans by comparing the grammar name to
string literals:

- `rust/src/ast_pipeline/ast_based_generator.rs:1240` —
  `let allow_trailing_layout = !self.grammar_name.eq_ignore_ascii_case("regex");`
  (RAW-name comparison; consumed at `:1438`/`:1462` — whether `parse_full` consumes trailing
  layout after the entry rule).
- `rust/src/ast_pipeline/ast_based_generator.rs:4621` —
  `let allow_layout_skip_for_terminals = normalized_grammar_name != "regex";`
  (NORMALIZED-name comparison — ASCII-alphanumerics-only, lowercased, `:4615-4620`; consumed at
  `:6258` — whether `match_string` skips leading layout before a terminal).
- `rust/src/ast_pipeline/ast_based_generator.rs:4622-4625` —
  `let allow_layout_skip_for_regexes = !matches!(normalized_grammar_name.as_str(),
  "regex" | "systemverilogpreprocessor");` (consumed at `:4944`/`:4954` — whether
  `match_regex` skips leading layout before a regex token).

The interpreter reproduces the same name-gate to stay byte-identical
(`rust/src/parse_harness_interpreter.rs:370-384`, `grammar_layout_policy(grammar_name)` —
its comment even cites stale codegen line numbers, a drift symptom of the duplication).

Consequences (why this is a defect, not a style nit):

1. **The `.ebnf` lies by omission.** Nothing in `grammars/regex.ebnf` or
   `grammars/systemverilog_preprocessor.ebnf` says they are whitespace-sensitive; the single
   most acceptance-relevant global policy lives in engine string literals. A reader of the
   grammar (or the stimuli generator, or any future tool that consumes the gen-AST) cannot see
   it — the exact "invisible out-of-band acceptance constraint" class
   `EBNF-SOURCE-OF-TRUTH` exists to eliminate.
2. **The capability is closed to every other grammar.** A synthetic whitespace-sensitive
   grammar CANNOT be probed through the parse harness: the scratch slot (TOOLBOX 1.3), the
   compile-and-run oracle (1.4), and the interpreter (1.5) all force
   whitespace-INSENSITIVE layout for any grammar not literally named `regex` — the harness
   whose whole purpose is "parse an ARBITRARY grammar" cannot express an entire class of
   grammars. Likewise any future whitespace-sensitive deliverable grammar would need an ENGINE
   edit (the anti-pattern the 2026-06-08 doctrine forbids).
3. **Two implementations must stay in expression-for-expression lockstep by hand** (codegen +
   interpreter `LayoutPolicy`), including the RAW-vs-NORMALIZED name asymmetry between `:1240`
   and `:4621` — pure drift risk, already visibly drifting in comments.

Fact base (established via grep/read this session): the ONLY grammar-name-gated layout sites in
`rust/src` are the three codegen booleans + the interpreter mirror above. The stimuli generator
has NO regex name-gate (its many `"regex"` hits are token-TYPE dispatch). Two OTHER name-gate
classes exist and are explicitly OUT OF SCOPE here (routed in §6): the regex→`pcre2`
default-profile gates (`rust/src/parser_registry.rs:138`, `rust/src/main.rs:2256`) and the svpp
stimuli quantified-separator heuristic (`rust/src/ast_pipeline/stimuli_generator.rs:11642`).

## 2. Design (leaf `.1`, this document)

### D1 — Surface

A **grammar-level** semantic directive, following the `@fact_kind:` / `@predicate_def:`
precedent (free-standing `@name: payload` lines; mechanically each binds to the following rule,
and a compile-time sweep collects them grammar-wide):

```ebnf
@whitespace_sensitive: true
```

declares the grammar FULLY whitespace-sensitive (all three facets), and

```ebnf
@whitespace_sensitive: { terminals: true, regex_tokens: true, trailing: true }
```

is the granular form; an absent field means insensitive (the default). Facet semantics map
1:1 onto the codegen booleans:

| payload field | true means | codegen boolean it drives |
|---|---|---|
| `terminals` | `match_string` must NOT skip leading layout | `allow_layout_skip_for_terminals = false` |
| `regex_tokens` | `match_regex` must NOT skip leading layout | `allow_layout_skip_for_regexes = false` |
| `trailing` | `parse_full` must NOT consume trailing layout | `allow_trailing_layout = false` |

`@whitespace_sensitive: false` is accepted and identical to absence (explicitness allowed).
The payload parses through the EXISTING generic annotation surface
(`grammars/semantic_annotation.ebnf:16` + `custom_annotation` fallback; scalar-`true`
precedent: `@stop_at_rule_boundary: true` in `grammars/ebnf.ebnf:112`) — **zero meta-grammar
change**, so the EBNF-meta-grammar-lockstep doctrine is satisfied by verification (dual-run
gate), not by an edit.

### D2 — Semantics and defaults

- Absent directive ⇒ whitespace-INSENSITIVE (all skips allowed) — today's behavior for every
  grammar except regex/svpp, so all other grammars are untouched by construction.
- More than one `@whitespace_sensitive` declaration in one grammar ⇒ hard compile error
  (contract-grade; no last-wins ambiguity).
- Malformed payload (not `true`/`false`/object-of-known-boolean-fields) ⇒ hard compile error
  naming the offending field.

### D3 — Declarations added to the shipped grammars

- `grammars/regex.ebnf` → `@whitespace_sensitive: true`
  (terminals + regex_tokens + trailing — exactly the current `"regex"` gate).
- `grammars/systemverilog_preprocessor.ebnf` → `@whitespace_sensitive: { regex_tokens: true }`
  (exactly the current svpp membership in the `:4622` matches! — terminals and trailing keep
  skipping).

### D4 — The name-gate is DELETED, not kept as fallback

Codegen reads the compiled directive; the `"regex"` / `"systemverilogpreprocessor"` layout
literals are removed. Single source of truth — no shadow default that could silently diverge.
Equivalence is PROVEN by emit-neutrality: with D3 in place, the three booleans are identical
for every shipped grammar, so every regenerated `generated/*_parser.rs` must be
**byte-identical** (`cmp`) — the decisive no-regression oracle.

### D5 — The interpreter derives from the SAME compiled directive

`parse_harness_interpreter.rs::grammar_layout_policy` stops matching names and reads the
compiled annotations already available to the interpreter load path — one decision point,
byte-identity preserved by the differential gates instead of by hand-mirroring.

### D6 — Wiring points (implementation map for `.2`)

1. `rust/src/ast_pipeline/semantic_runtime.rs` — `parse_semantic_runtime_directive`
   (`:2636-2653`) gains a `"whitespace_sensitive"` arm → new
   `SemanticRuntimeDirective::WhitespaceSensitive(LayoutSensitivity)` (a
   `{terminals, regex_tokens, trailing}` bool struct) + payload validation per D2;
   `compile_semantic_runtime_annotations` records it grammar-wide (duplicate ⇒ error) and
   exposes `layout_sensitivity()` on `CompiledSemanticRuntimeAnnotations`.
2. `rust/src/ast_pipeline/semantic_directive_registry.rs` + the annotation validator —
   register the name (capability: parser-steering) so contract-grade validation knows it.
3. `rust/src/ast_pipeline/ast_based_generator.rs` — the three sites read the compiled
   sensitivity (threaded to `generate_helper_methods` / the `parse_full` emitter as needed);
   name literals deleted.
4. `rust/src/parse_harness_interpreter.rs` — `grammar_layout_policy` takes the compiled
   annotations; name literals deleted.
5. Grammars per D3; regen ALL shipped parsers; `cmp` byte-identity per D4.
6. Lockstep (same commit): ebnf parser book (grammar-author surface — the directive, payload
   grammar, examples, defaults, errors), top book `annotation-system.md` (+ a parse-harness
   chapter note that whitespace-sensitive SYNTHETIC grammars are now probe-able),
   `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`,
   `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`, TOOLBOX 1.3 note if warranted,
   continuity docs (CHANGES / DEVELOPMENT_NOTES / MEMORY / TASK_TREE row).

### D7 — Explicitly out of scope (routed, §6)

Generation-side spacing policy (the stimuli generator does not consult the parse-side layout
booleans today; introducing that coupling is a separate, evidence-first question), the
default-profile name-gates, and the svpp stimuli separator heuristic.

## 3. Tree

- `WS-DIRECTIVE.1` — **DESIGN** (this file; §1 fact base + §2 design). Status: `done`
  (2026-07-07, session #54, `PGEN-WS-DIRECTIVE-0001`, docs-only).
- `WS-DIRECTIVE.2` — **IMPLEMENT + lockstep** (one code slice per D6, enforced checklist §5).
  Status: `pending`.

## 4. Tree-level acceptance criteria

1. ZERO grammar-name-gated layout decisions remain in `rust/src` (grep-provable: the
   `"regex"` / `"systemverilogpreprocessor"` layout literals at the four §1 sites are gone).
2. `@whitespace_sensitive` is a parsed, validated, documented grammar-level directive;
   regex + svpp declare their true layout policy IN THE GRAMMAR.
3. Emit-neutrality: every shipped `generated/*_parser.rs` regenerates byte-identical (`cmp`)
   with the directive-driven codegen — plus zero cert-coverage movement on the touched
   grammars (regex/svpp seeds 0/7/42 byte-identical headline).
4. Gates green: `parse_harness_equivalence_gate` 4/4 (11 CERTIFIED byte-identical),
   `parse_harness_combinator_gate` 20/20, `parse_harness_semantic_gate` 24/24, features-on lib
   suite, `ebnf_frontend_dual_run_gate`, annotation contract surface, clippy strict-source,
   `mdbook_docs_gate` + `ebnf_parser_book_gate`.
5. Capability demonstrated: a SYNTHETIC directive-bearing whitespace-sensitive grammar parses
   whitespace-sensitively through the harness (interpreter + compile-and-run oracle agree,
   byte-identical) — the probe that was impossible pre-tree.
6. Full same-commit lockstep per D6.

## 5. WS-DIRECTIVE.2 — acceptance checklist (enforced)

## Acceptance Checklist (enforced)
- [ ] **REPRODUCE / ISSUE** — <the §1 name-gate sites at HEAD + a live probe: a synthetic
  ws-sensitive grammar is forced insensitive through the harness>
- [ ] **ROOT CAUSE (WHY + WHERE)** — <§1: layout policy keyed on grammar-name literals at
  `ast_based_generator.rs:1240/:4621/:4622-4625` + interpreter mirror
  `parse_harness_interpreter.rs:370-384`; invisible in the `.ebnf`>
- [ ] **FIX** — <D1-D6: new grammar-level directive (fix-hierarchy tier 3, new annotation —
  tiers 1/2 cannot express a codegen-time layout policy; recorded justification in §2)>
- [ ] **ADDRESSED (verified)** — <before→after: the §5-capability probe INSENSITIVE→SENSITIVE;
  regex/svpp policies now declared in-grammar; name literals deleted>
- [ ] **NO REGRESSION** — <emit-neutrality `cmp` across ALL shipped generated parsers;
  equivalence 4/4 / combinator 20/20 / semantic 24/24; features-on lib; dual-run gate;
  cert headlines byte-identical for regex+svpp seeds 0/7/42; clippy strict-source>
- [ ] **LOCKSTEP** — <D6 item 6, same commit>

## 6. Findings routed elsewhere (surfacing directive, 2026-07-07)

Two more grammar-name-gate instances of the same doctrine-violation CLASS were established
tools-first during `.1` scoping; they are different capabilities and are NOT owned here:

- **F1 — regex→`pcre2` default-profile name-gates.**
  `rust/src/parser_registry.rs:138` (parse side) + `rust/src/main.rs:2256` (generation side)
  hard-code "an unspecified profile for the grammar named `regex` defaults to `pcre2`". The
  principled fix is a grammar-level `@default_profile: pcre2` directive (same shape as this
  tree). Candidate follow-up tree — needs director visibility, listed in the session
  surfacing callout. Until then the behavior is correct, just name-gated.
- **F2 — svpp stimuli quantified-separator heuristic.**
  `rust/src/ast_pipeline/stimuli_generator.rs:11642`
  (`should_insert_quantified_separator`) is gated on BOTH the grammar name
  (`systemverilog_preprocessor`) AND four hard-coded rule names — generation-side, doubly
  name-coupled. Candidate for the `STIMULI-SIGNOFF` capability-gap backlog (a declarative
  separator/lexical-cohesion construct is the LEXICAL-ANNOTATIONS lane's territory).

## 7. Verification log

- 2026-07-07 (session #54, `.1`): §1 fact base established by direct reads/greps at HEAD
  `a161939c` (all four layout name-gate sites + consumers; interpreter mirror + stale-comment
  drift; stimuli generator negative result; the two out-of-scope name-gate classes F1/F2;
  the generic `custom_annotation` parse surface + `@stop_at_rule_boundary: true` scalar
  precedent + `@fact_kind:` grammar-level-sweep precedent at `semantic_runtime.rs:2647`).

## 8. Decisions

- Directive-with-payload over bare flag: the existing annotation surface REQUIRES `: value`
  (`semantic_annotation.ebnf:16`), and svpp needs granularity (`regex_tokens` only) — so
  `true` | `{facets}` covers both cleanly with zero meta-grammar change.
- Delete-not-fallback (D4): a retained name-gate fallback would be a second, shadow source of
  truth — the defect class this tree removes. Emit-neutrality is the safety net.
- Out-of-scope routing (D7/§6) keeps this tree one clean capability; F1/F2 are surfaced, not
  silently dropped.
