# POSITIONAL-PAYLOAD-REFS — make positional `$N` references resolvable in semantic-directive payloads (F4)

- Tree ID: `POSITIONAL-PAYLOAD-REFS`
- Status: `active` (created 2026-07-06, session #47, spawned by `SEM-FINDINGS` — director directive
  2026-07-06; this tree owns finding **F4**)

## 1. The finding (tool-established, PARSE-HARNESS.6.2 session #47)

Positional `$N[.path]` references in a directive payload (`@emit_fact: { name: $2, … }`,
`args: [pf, $2.word]`) can **never resolve**:

- WHERE: the annotation compiler **strips the `$` sigil** when parsing payload values — the frozen
  compiled literal for `name: $2` is `SemanticRuntimeValue::RuleReference("2")` (read directly from
  the emitted `sem_ref_raw_positional` parser's `CompiledSemanticRuntimeAnnotations` literal).
- The emitted resolver dispatches positionally ONLY on `starts_with('$')`
  (`resolve_semantic_reference`, template @`ast_based_generator.rs:5534`); a stripped `"2"` routes to
  `resolve_named_semantic_reference`, whose lexer REJECTS a digit head → resolution always fails →
  `@emit_fact` hard-errors → the rule fails. Pinned by
  `parse_harness_semantic_suite::sem_ref_positional_unresolvable` (both implementations reject
  identically — the hard-error parity).
- The positional machinery itself (`resolve_positional_semantic_reference` +
  `parse_semantic_reference_segments`, incl. `[N]` indexed access) was built DELIBERATELY
  (SV-EXH-PROOF.3.3.4.a.2, `PGEN-SV-EXH-PROOF-0027`) — it is dead code from compiled directives, a
  half-wired surface exactly like `.6.1`'s bounded quantifiers.

## 2. The elegant fix (design; confirm locus in `.1`, implement in `.2`)

**Preserve the sigil for digit-headed references in the payload compiler** (engine tier, but the
narrowest possible cut — ONE parsing site in `semantic_runtime.rs`, no codegen-template change):

- When the payload parser converts a `$…` reference into `RuleReference`, keep the leading `$` iff
  the reference body is digit-headed (positional); named references stay stripped (today's working
  behavior, untouched — `$body` → `RuleReference("body")` continues to resolve via the named path).
- The ALREADY-EMITTED resolver templates then just work: `RuleReference("$2.word")` hits the
  existing `starts_with('$')` → digit → positional dispatch. **No template change ⇒ no forced regen**
  of shipped parsers (their frozen literals contain no positional refs — nothing to change), and the
  **interpreter compiles in-process with the same function ⇒ parity automatic**.
- Blast radius: ZERO existing grammars can be using positional payload refs (they would be failing
  hard today) — verify by grep in `.1`. New capability is additive.
- Suite re-anchor (deliberate, documented): `sem_ref_positional_unresolvable` becomes the WORKING
  `sem_ref_positional` case (`"(a)[a]"` REJECT → ACCEPT; construct renamed; the old hard-error
  parity note preserved in the leaf log). Also extend with an `[N]` indexed-access input so the
  a.2 bracket machinery finally gets live coverage.
- REJECTED alternative: digit-head dispatch in the resolver template — a codegen-template change
  forcing a full regen for behavior reachable more surgically in the compiler.

## 3. Verification battery (leaf `.2`)

`parse_harness_semantic_gate` (re-anchored + the new indexed-access input) +
`parse_harness_equivalence_gate` (11 CERTIFIED byte-identical — expected untouched; no shipped grammar
uses positional payload refs) + `parse_harness_combinator_gate` + SV cert union gate seeds 0/7/42 +
clippy + `mdbook_docs_gate`. Normative-spec + semantic_annotation book lockstep: positional payload
references become DOCUMENTED syntax (with the `[N]` form), replacing the current undocumented
hard-fail.

## 4. Leaves

- `.1` — **EVIDENCE: strip locus + zero-usage audit — `done` (2026-07-06, session #50,
  `PGEN-POSITIONAL-PAYLOAD-REFS-0001`).** NO code. All three parts evidence-backed:

  **(a) Strip locus CONFIRMED — ONE function, ONE call site (locus refinement: the function lives in
  `unified_semantic_ast.rs`, not `semantic_runtime.rs` as §2 guessed):**
  - `StructuredSemanticValueParser::parse_rule_reference`
    (`rust/src/ast_pipeline/unified_semantic_ast.rs:532–617`): `expect_char('$')` consumes the sigil
    (`:533`), `let start = self.position;` captures from AFTER it (`:534`), returns
    `self.input[start..self.position]` (`:616`) → `$2.word` → `"2.word"`, `$body` → `"body"`.
  - Single dispatch site: `parse_value` `'$' =>` arm (`unified_semantic_ast.rs:419–421`). BOTH
    annotation entries converge on it: bootstrap `parse_bootstrap` → `parse_structured_payload`
    (`:305→:331`) and generated-parser `parse_generated_semantic_annotation_entry` →
    `from_named_payload` → `parse_structured_payload` (`:229/:272→:76`). Object/array payloads
    recurse through the same `parse_value`; `parse_predicate`'s object-form `args:` array values
    included (`semantic_runtime.rs:3270–3277`) — so emit_fact names/attributes, predicate args,
    scope names, export/import fields ALL funnel through the one function.
  - Conversion is verbatim: `SemanticRuntimeValue::from_semantic_value`
    (`rust/src/ast_pipeline/semantic_runtime.rs:72–82`) clones text — the compiled/frozen literal is
    exactly what `parse_rule_reference` returned. (The `RuleReference("$1")` unit tests at
    `semantic_runtime.rs:3457+` hand-construct the unified value, bypassing the parser — that is why
    the sigil survives there.)
  - Resolver dispatch (emitted template `ast_based_generator.rs:5541`): positional REQUIRES
    `starts_with('$')` (`:5557`); the positional path receives the FULL `$`-carrying ref (`:5565`)
    and `parse_semantic_reference_segments` itself REQUIRES the leading `$` (`:5701,:5706`); the
    named lexer `lex_semantic_reference_segments_named` REJECTS a digit head (`:5941,:5948`) → a
    stripped `"2.word"` routes named → `None` → hard error. Interpreter mirror identical
    (`parse_harness_interpreter.rs:1210,:1224–1237`, segments `:1328`).
  - Parity-automatic claim VERIFIED: `compile_semantic_runtime_annotations`
    (`semantic_runtime.rs:2980`) is the single shared compile function — codegen freezes its output
    (`ast_based_generator.rs:6683`), the interpreter calls the SAME function in-process
    (`parse_harness_interpreter.rs:235–241`). A `parse_rule_reference` fix reaches both
    implementations with no template change.

  **(b) Zero-usage audit CONFIRMED — blast radius nil:**
  - Grammars: `grep -nE '@(emit_fact|predicate|open_scope|close_scope|export_to_library|
    import_from_library|fact_kind|predicate_def|scope)[^_a-zA-Z]' grammars/*.ebnf
    grammars/scratch/*.ebnf | grep -E '\$[0-9]'` → ONE hit, a comment (`grammars/regex.ebnf:787`
    “(was `body:$4`)” historical note). The broader `\$[0-9]` sweep hits only `regex.ebnf`
    `@generate`/`@optimize`/`@validate`/`@semantic_value` lines — NON-runtime kinds: the directive
    compiler returns `Ok(None)` for them (`semantic_runtime.rs:2609`) and a repo-wide grep finds NO
    consumer dispatching on those names (doc-comment mentions only).
  - Generated frozen literals: `grep -hoE 'RuleReference\("[0-9$][^"]*"' generated/*.rs` → **0
    matches** across all 11 generated parsers; the 61 existing literals are all alpha-headed named
    refs (`body`×46, `scope.body.name.body`×4, `head.body`×3, `package.body`/`name.body`×2,
    `scope.name.body`/`index`/`head`/`body.name.body`×1).

  **(c) Named path untouched + collateral survey:**
  - The §2 fix shape (preserve `$` iff digit-headed) leaves every alpha-headed ref byte-identical:
    still stripped, still routed through the non-`$` branch (`:5570`) →
    `resolve_named_semantic_reference` — today's working path.
  - `library.rs` fact-export/artifact paths (`:195–200`, `:238–242`) are defensive verbatim
    markers (RuleReference "shouldn't survive resolution") — no sigil assumption. `predicate_expr`
    `$refs` are a SEPARATE surface (`PredicateValue::ArgRef`, bindings-based resolution at
    `semantic_runtime.rs:2399`) — architecturally unaffected. `stimuli_generator.rs:11476`'s
    `fact_count_at_least` threshold arm matches `RuleReference(_)` wildcard — unaffected.
  - ⚠️ **ONE flagged `.2` collateral:** `emit_name_is_whole_render`
    (`stimuli_generator.rs:3194–3196`) treats ANY undotted `RuleReference` name as "the whole
    render"; post-fix a `name: $2` literal (`"$2"`, undotted) would wrongly qualify ($2 is position
    2's SUB-render). Unreachable today (zero usage) but `.2` makes it reachable — `.2` MUST exclude
    `$`-headed refs from the whole-render heuristic (or resolve the position) and cover it.

- `.2` — **FIX: sigil preservation + suite re-anchor + docs — `not-started` (frontier).** Blocked on
  `.1` → now UNBLOCKED. The §2 fix (in `parse_rule_reference`, `unified_semantic_ast.rs` — the `.1`-
  confirmed locus) + the §3 battery + the enforced acceptance checklist + spec/book lockstep + the
  `.1`-flagged `emit_name_is_whole_render` exclusion.

## 5. Current Frontier

| # | Leaf | Status | Notes |
| --- | --- | --- | --- |
| 1 | `.1` (strip locus + zero-usage audit) | `done` (2026-07-06 #50, `PGEN-POSITIONAL-PAYLOAD-REFS-0001`) | Locus = `parse_rule_reference` (`unified_semantic_ast.rs:532–617`), ONE site; zero usage in grammars + generated; 1 collateral flagged for `.2`. |
| 2 | `.2` (sigil preservation + re-anchor + docs) | `not-started` (**frontier**) | Narrow engine cut in the `.1`-confirmed locus; parity automatic; MUST handle `emit_name_is_whole_render`. |

## 6. Relationships

- Spawned by [`SEM-FINDINGS`](SEM-FINDINGS.md) (F4). Evidence base: `PARSE-HARNESS.6.2` —
  `sem_ref_positional_unresolvable` pin + the frozen-literal read; the SV-EXH-PROOF.3.3.4.a.2
  positional-resolver machinery this tree brings to life.
- Sibling half-wired-surface precedent: `.6.1`'s bounded quantifiers (PARSE-HARNESS backlog).
