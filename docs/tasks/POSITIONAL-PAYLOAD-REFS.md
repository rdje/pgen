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

- `.1` — **EVIDENCE: strip locus + zero-usage audit — `not-started` (frontier).** (a) Pinpoint the
  exact payload-parsing function(s) in `semantic_runtime.rs` where the sigil is dropped (parse_emit_fact
  / parse_predicate / parse_open_scope / parse_export_* value paths — one shared value-parser
  expected). (b) Grep ALL `grammars/*.ebnf` directive payloads + `generated/*` frozen literals for
  positional refs (expected: none — confirming zero blast radius). (c) Confirm the named-path
  behavior is untouched by the change shape (the `$`-strip stays for alpha-headed refs). NO code.
- `.2` — **FIX: sigil preservation + suite re-anchor + docs — `not-started`.** Blocked on `.1`.
  The §2 fix + the §3 battery + the enforced acceptance checklist + spec/book lockstep.

## 5. Current Frontier

| # | Leaf | Status | Notes |
| --- | --- | --- | --- |
| 1 | `.1` (strip locus + zero-usage audit) | `not-started` (**frontier**) | Tools-first; NO code. |
| 2 | `.2` (sigil preservation + re-anchor + docs) | `not-started` | Blocked on `.1`. Narrow engine cut; parity automatic. |

## 6. Relationships

- Spawned by [`SEM-FINDINGS`](SEM-FINDINGS.md) (F4). Evidence base: `PARSE-HARNESS.6.2` —
  `sem_ref_positional_unresolvable` pin + the frozen-literal read; the SV-EXH-PROOF.3.3.4.a.2
  positional-resolver machinery this tree brings to life.
- Sibling half-wired-surface precedent: `.6.1`'s bounded quantifiers (PARSE-HARNESS backlog).
