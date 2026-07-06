# FACT-NAME-MATCHING — quoted predicate args vs `$ref`-emitted fact names: unify or lint (F2)

- Tree ID: `FACT-NAME-MATCHING`
- Status: `complete` (created 2026-07-06, session #47, spawned by `SEM-FINDINGS` — director directive
  2026-07-06; this tree owns finding **F2**. `.1` adjudication + `.2` fix both landed session #50 —
  **F2 CLOSED**: fact/scope NAME matching is textual, quoted and unquoted name args are equivalent,
  on both implementations)

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

- `.1` — **EVIDENCE: the consistency read + tier decision — `done` (2026-07-06, session #50,
  `PGEN-FACT-NAME-MATCHING-0001`). DECISION: tier (a) — engine inconsistency CONFIRMED; unify
  NAME matching to textual equality.** The full comparison inventory (all
  `rust/src/ast_pipeline/semantic_runtime.rs`):

  | Comparison | Mechanism | Semantics |
  | --- | --- | --- |
  | fact KIND | `eq_ignore_ascii_case` / lowercased index key (`:1059,:1096`) | textual, case-insensitive |
  | attribute KEY | `eq_ignore_ascii_case` (`:2196,:2213,:2231`) | textual |
  | attribute VALUE | `semantic_values_match` (`:3398–3403`) | **textual** when both sides text-bearing (String/Identifier/RuleReference/Number unify) |
  | `resolve_path` fact-by-name / scope-by-name | `fact_name_matches` (`:3350`, via `:1584,:1596`) | **textual** (doc-commented "textual `name` matches") |
  | fact-index NAME (has_fact / lacks_fact / has_fact_in_current_scope / has_fact_attribute / fact_attribute_equals / lacks_fact_attribute_equals) | enum-keyed `HashMap<(usize, SemanticRuntimeValue), _>` + derived `Hash`/`PartialEq` (`:1050`; `n == name` `:1103,:1140`; hash-key lookup `:1119`) | **variant-STRICT** — `Identifier("x") ≠ String("x")` |
  | `current_scope_is` NAME (arg 2) | `current_scope.name.as_ref() == Some(&expected_name)` (`:2125`) | **variant-STRICT** — same trap for scope names |

  - **The strictness is ACCIDENTAL, not deliberate typing:** the strict `fact.name ==
    expected_name` came from the ORIGINAL predicate-evaluators commit (`a529c2d2`, no typing
    rationale); the index commit (`be3c5754`) preserved it purely for performance ("+`Hash` on
    `SemanticRuntimeValue` … Required because the index uses `(scope_depth, SemanticRuntimeValue)`
    as a HashMap key" — rationale is the ≤200ns p99 store-performance contract, zero typing
    deliberation). The LENIENT attribute matcher was a deliberate design choice at introduction
    (`d4dc2284`, CHANGES entry: "added scalar-friendly semantic value matching for attribute
    comparisons"). No decision record defends strict name equality (grep of `docs/decisions/`
    empty; the `.6.2` book fact documents the unquoted-args WORKAROUND convention, not a
    rationale).
  - **A single `fact_attribute_equals` call mixes THREE textual comparisons (kind, key, attribute
    value) with ONE variant-strict comparison (name)** — and scope-NAME matching is itself split
    (textual in `resolve_path`'s `find_scope_by_name`, strict in `current_scope_is`). The variant
    on each side is incidental: the emit side assigns it by coercion heuristic
    (`coerce_semantic_runtime_scalar`: bool → number → Identifier → String,
    `ast_based_generator.rs:2340`, interpreter mirror `parse_harness_interpreter.rs:1173`) while
    the query side assigns it by surface syntax (quoted = String, unquoted = Identifier) — strict
    equality therefore compares two accidents.
  - **`.2` scope (the (a) fix):** (i) normalize the fact-index name key to a `NameKey`
    (Text(scalar text) for text-bearing variants; Boolean/Null stay distinct — EXACTLY
    `semantic_values_match` semantics) at insert/remove/query; (ii) `current_scope_is`'s name
    comparison goes textual via the same helper; (iii) new suite case: quoted `args: [kind,
    "name"]` vs `$ref`-emitted fact PASSES; (iv) interpreter parity automatic (the index lives in
    the SHARED `SemanticRuntimeState`); (v) §3 battery — any equivalence-gate change means a
    shipped grammar RELIED on the mismatch → stop and surface.

- `.2` — **FIX at tier (a) + suite case + docs — `done` (2026-07-06, session #50,
  `PGEN-FACT-NAME-MATCHING-0002`).** The engine unification landed in the SHARED runtime
  (`rust/src/ast_pipeline/semantic_runtime.rs`), so interpreter parity is automatic and NO
  codegen-template change / NO shipped-parser regen was needed:
  - **`FactNameKey`** (Text(scalar text) for String/Identifier/RuleReference/Number;
    Boolean/Null distinct — EXACTLY `semantic_values_match` semantics) normalizes the fact-index
    name key at `insert` / `remove` / `any_with_name` / `any_with_name_at_scope` /
    `positions_for_name`; the index stays hash-based (the ≤200ns contract untouched).
  - **`semantic_runtime_values_match`** (the `SemanticRuntimeValue` mirror of
    `semantic_values_match`) replaces the strict `==` in `current_scope_is` — the second strict
    site the `.1` inventory found.
  - **Suite**: NEW `sem_quoted_name_args` (construct `QuotedNameArgTextualMatch`) covers BOTH
    unified sites in one grammar — a `$2`-emitted fact (Identifier via coercion) queried with a
    QUOTED `"special"` (has_fact) and a `$2`-named scope queried with a QUOTED `"sc"`
    (current_scope_is); ACCEPT + 2 targeted REJECTs. Suite 23 → 24. (The case composes with the
    just-landed F4 positional refs — `name: $2` payloads.)

## Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — the F2 finding (scratch-slot trace, session #47): `🔍 has_fact(kind=mode,
  name=String("special")) → false` WITH the fact present — the natural quoted arg is a silently
  dead gate; grammars carry the unquoted-identifier workaround convention.
- [x] **ROOT CAUSE (WHY + WHERE)** — `.1` (commit `cf262c9d`): the scratch-slot `--trace-rules`
  self-explaining query trace (session #47) showed the miss WITH the fact present
  (`🔍 has_fact(kind=mode, name=String("special")) → false`); the source read pinned WHERE:
  fact-index name key = the raw `SemanticRuntimeValue` enum (`:1050`, `n == name` `:1103/:1140`,
  hash lookup `:1119`) + `current_scope_is` strict `==` (`:2125`) — variant-STRICT by ACCIDENT
  (original `==` a529c2d2 no rationale; be3c5754 preserved it for perf) while
  kind/key/attribute-value/`resolve_path` names are ALL textual; one `fact_attribute_equals`
  call = 3 textual + 1 strict comparison.
- [x] **FIX** — engine tier per the `.1` adjudication (the inconsistency lives in the engine;
  no grammar/annotation tier can reach an index key): `FactNameKey` normalization + the
  `current_scope_is` textual comparison, both in the shared runtime.
- [x] **ADDRESSED (verified)** — `sem_quoted_name_args` `"(special)!{sc}end"` ACCEPTs with quoted
  args on BOTH sites (pre-fix: dead gate → REJECT); `parse_harness_semantic_gate` **24/24 CLEAN
  2/2 tests** — the quoted-arg case live and byte-identical on both implementations.
- [x] **NO REGRESSION** — `parse_harness_equivalence_gate` 4/4: **11 CERTIFIED byte-identical**
  (the §3 tripwire is SILENT — no shipped grammar relied on the strict mismatch);
  `sv_cert_recognized_union_gate` GREEN deterministic seeds 0/7/42 (canonical `1343/10/1321/12`,
  union 1, residual `context_member_method_call`); `ast_shape_contract_gate` 18/18;
  `sv_external_corpus_triage_gate` GREEN (`primary_parse_failure_corpus: <none>` — 14/14);
  `parse_harness_combinator_gate` 16/16; semantic_runtime units 98/98 (no unit pinned the strict
  behavior); clippy strict-source GREEN (generated stage = pre-existing `eq_op` debt only).
- [x] **LOCKSTEP** — normative spec NEW section *Fact & Scope Name Matching (Normative)*; top book
  `semantic-store.md` §6 "Name matching is textual" + `parse-harness.md` (24 cases, new table row,
  the variant-sensitivity author fact REWRITTEN to the unified reality); semantic_annotation book
  `semantic-store.md` + rendered HTML (gate GREEN); `TOOLBOX.md` §1.8 (24/24); `mdbook_docs_gate`
  GREEN; tree + TASK_TREE.md + live docs this commit.

## 5. Current Frontier

| # | Leaf | Status | Notes |
| --- | --- | --- | --- |
| 1 | `.1` (consistency read → tier decision) | `done` (2026-07-06 #50, `PGEN-FACT-NAME-MATCHING-0001`) | **Tier (a)**: names strict by ACCIDENT (index perf commit), attributes textual BY DESIGN, scope-name matching itself split — unify names to textual. |
| 2 | `.2` (fix + case + docs) | `done` (2026-07-06 #50, `PGEN-FACT-NAME-MATCHING-0002`) | F2 CLOSED: names match textually on both sites; suite 24/24; 11 CERTIFIED byte-identical; spec/book lockstep. **TREE COMPLETE.** |

## 6. Relationships

- Spawned by [`SEM-FINDINGS`](SEM-FINDINGS.md) (F2). Evidence base: `PARSE-HARNESS.6.2` — the
  scratch-slot `has_fact` trace + `sem_branch_gate`'s unquoted-identifier convention comment.
- Touches: the annotation validator (`rust/src/ast_pipeline/annotation_validator.rs`) if (b);
  `semantic_runtime.rs` fact index if (a); `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` either way.
