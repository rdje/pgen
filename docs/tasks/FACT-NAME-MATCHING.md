# FACT-NAME-MATCHING — quoted predicate args vs `$ref`-emitted fact names: unify or lint (F2)

- Tree ID: `FACT-NAME-MATCHING`
- Status: `active` (created 2026-07-06, session #47, spawned by `SEM-FINDINGS` — director directive
  2026-07-06; this tree owns finding **F2**)

## 1. The finding (tool-established, PARSE-HARNESS.6.2 session #47)

Fact-NAME matching in the store index is **variant-sensitive**, and the two sides of a typical
gate arrive as DIFFERENT variants:

- An emitted name resolved from a `$ref` is coerced by `coerce_semantic_runtime_scalar`
  (bool → number → **Identifier** → String), so `name: $body` over text `special` stores
  `Identifier("special")`.
- A QUOTED literal in predicate args stays `String("special")`
  (`UnifiedSemanticValue::String` → `SemanticRuntimeValue::String` via `from_semantic_value`).
- MEASURED (scratch-slot trace, session #47): `🔍 has_fact(kind=mode, name=String("special")) →
  false` **with the fact present** — the gate silently never passes. A grammar author writing the
  natural `args: [mode, "special"]` gets a dead gate with zero diagnostics.
- The working convention (SV-wide, and now the `.6.2` suite): UNQUOTED identifiers in `args:`.
  Pinned by `sem_branch_gate`'s in-grammar comment.

## 2. Adjudication to make FIRST (leaf `.1` — the tier decision)

The elegant fix depends on one evidence read: **is the engine internally consistent?**

- `fact_attribute_equals` compares attribute VALUES via `semantic_values_match(…)`
  (`semantic_runtime.rs:2119-2155`) — read whether that helper is variant-LENIENT (textual) or
  strict. Also read `fact_index.any_with_name` / `positions_for_name` — how names hash/compare.
- If ATTRIBUTES match leniently while NAMES match strictly → an engine **inconsistency**, and the
  elegant fix is (a): unify NAME matching to textual equality (engine tier, small — the index's
  name key normalizes to the scalar text; `Identifier("x")`, `String("x")` unify; `Number` stays
  numeric-textual). Verified by the whole battery; the suite gains a quoted-arg case that then
  PASSES.
- If BOTH are strict (consistent, plausibly deliberate typing) → the safe tier is (b): a
  **validator V-rule** (annotation-validator warning: "a quoted-String name arg to
  has_fact/lacks_fact/fact_attribute_equals/lacks_fact_attribute_equals will never match a
  `$ref`-emitted Identifier name — use an unquoted identifier"), plus normative-spec wording.
  Check `git log`/decision records for any deliberate-typing rationale before choosing (a).

## 3. Verification battery (leaf `.2`)

If (a) engine unification: full battery — `parse_harness_semantic_gate` (+ a new quoted-arg
isolating case) + `parse_harness_equivalence_gate` (11 CERTIFIED byte-identical — any change means a
shipped grammar was RELYING on the mismatch: stop, investigate, surface) +
`parse_harness_combinator_gate` + SV cert union gate seeds 0/7/42 + external corpus +
`ast_shape_contract` + clippy + `mdbook_docs_gate`; interpreter parity automatic (shared runtime
function). If (b) validator rule: validator suite + `annotation_contract_gate` + a
warning-fires/warning-silent test pair + spec/book lockstep; no parser behavior change.

## 4. Leaves

- `.1` — **EVIDENCE: the consistency read + tier decision — `not-started` (frontier).**
  Read `semantic_values_match` + the fact-index name comparison; grep decision records/git for
  deliberate-typing rationale; RECORD the (a)/(b) decision here with the evidence. NO code.
- `.2` — **FIX at the decided tier + suite case + docs — `not-started`.** Blocked on `.1`. The
  enforced acceptance checklist + the §3 battery + normative-spec/book lockstep (the matching
  contract becomes documented either way).

## 5. Current Frontier

| # | Leaf | Status | Notes |
| --- | --- | --- | --- |
| 1 | `.1` (consistency read → tier decision) | `not-started` (**frontier**) | Tools-first; NO code. |
| 2 | `.2` (fix + case + docs) | `not-started` | Blocked on `.1`. |

## 6. Relationships

- Spawned by [`SEM-FINDINGS`](SEM-FINDINGS.md) (F2). Evidence base: `PARSE-HARNESS.6.2` — the
  scratch-slot `has_fact` trace + `sem_branch_gate`'s unquoted-identifier convention comment.
- Touches: the annotation validator (`rust/src/ast_pipeline/annotation_validator.rs`) if (b);
  `semantic_runtime.rs` fact index if (a); `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` either way.
