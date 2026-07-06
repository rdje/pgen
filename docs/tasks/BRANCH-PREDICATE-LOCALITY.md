# BRANCH-PREDICATE-LOCALITY — make inline `phase: branch` predicates branch-LOCAL (F3), and fix the suspected live SV:4605 consequence

- Tree ID: `BRANCH-PREDICATE-LOCALITY`
- Status: `active` (created 2026-07-06, session #47, spawned by `SEM-FINDINGS` — director directive
  2026-07-06 to elegantly address the `PARSE-HARNESS.6.2` findings; this tree owns finding **F3**)
- Priority: **HIGHEST of the SEM-FINDINGS fixes** — a suspected LIVE SV parser defect rides on it.

## 1. The finding (tool-established, PARSE-HARNESS.6.2 session #47)

An INLINE `@predicate: { …, phase: branch }` placed at the start of ONE alternative is applied to
**every** branch of the rule's tournament, not just its own:

- WHERE: `CompiledSemanticRuntimeAnnotations::branch_predicates_for_rule`
  (`rust/src/ast_pipeline/semantic_runtime.rs:735-747`) — it returns the RULE-level directives
  **chained with a flat-map over ALL branch buckets** (`branch_directives_for_rule(rule).iter().flat_map(…)`).
  The generated tournament (and the interpreter, which calls the SAME function) evaluates
  `branch_predicates_for_rule(rule) ∪ branch_predicates_for_rule_branch(rule, idx)` per candidate —
  so an inline branch predicate gates every branch AND double-evaluates on its own branch.
- EVIDENCE: scratch-slot trace on the `sem_branch_gate` isolating grammar —
  `🛡️ predicate 'has_fact' REJECTED branch 1/2` **and** `… REJECTED branch 2/2` from ONE inline
  predicate on branch 1; pinned differentially by `parse_harness_semantic_suite::sem_branch_gate`
  (the current anchors encode the flattening).
- The SV grammar KNOWS about this: the `net_declaration` comment (`grammars/systemverilog.ebnf:3452`)
  explicitly works AROUND flattening ("an inline `phase: branch` predicate is flattened rule-wide …
  the gate lives on a helper with `phase: post` so it stays BRANCH-LOCAL").

## 2. The suspected LIVE SV defect (VERIFY FIRST — leaf `.1`)

`grammars/systemverilog.ebnf:4605` (`scoped_or_hierarchical_tf_identifier`, landed `.b.6.2.2`) puts an
inline branch predicate on the `package_scope` alternative:

```
@branch_policy: priority_first
scoped_or_hierarchical_tf_identifier := class_scope tf_identifier          -> {kind: "class_scope", …}
    | @predicate: { name: lacks_fact_attribute_equals, args: [type_name, $scope.name.body, …], phase: branch, view: shaped }
      package_scope tf_identifier                                          -> {kind: "package_scope", scope: $1, name: $2}
    | hierarchical_tf_identifier                                           -> {kind: "hierarchical", body: $1}
```

Under flattening, that predicate ALSO evaluates on the `hierarchical` branch, whose shaped content
`{kind: "hierarchical", body: …}` has **no `scope` key** → `try_resolve` fails → **the branch is
silently BLOCKED** (the tournament's unresolvable-arg rule). If confirmed, hierarchical tf-names are
unparseable THROUGH THIS RULE (any accept of such input must be coming through a different path —
or is genuinely rejected). 4605 appears to predate/contradict the 3452 workaround knowledge.

## 3. The elegant fix (design; confirm in `.1`, implement in `.2`)

**Fix the registry function** (engine tier — justified because the defect IS the registry semantics;
no grammar-level tier can repair an engine flat-map):

- `branch_predicates_for_rule` returns ONLY rule-level branch-phase predicates (from
  `directives_by_rule`) — it stops flat-mapping the branch buckets. Inline (per-branch) predicates
  reach the tournament exclusively via `branch_predicates_for_rule_branch(rule, idx)`, which is
  ALREADY chained at every call site (generated template @`ast_based_generator.rs:3264-3272` and the
  interpreter's mirrored tournament).
- **Interpreter parity is automatic** — it calls the same registry function (zero interpreter change).
- Blast-radius audit (part of `.1`): grep every grammar for inline `phase: branch` users; confirm
  nothing RELIES on flattening (the SV:3452 site deliberately avoids it via `phase: post` helpers, so
  it is untouched). Regenerating parsers changes behavior ONLY where an inline branch predicate
  exists — expected: SV:4605 (the fix's beneficiary) and the suite's isolating grammars.
- Suite re-anchor (deliberate, documented): `sem_branch_gate`'s flattening pin flips to the
  branch-local pin (`"mode:other;x;"` REJECT → ACCEPT-as-`normal_pick`); the old and new semantics
  both recorded. `sem_branch_select` (the helper-rule idiom) must stay CLEAN — it is
  flattening-independent.

## 4. Verification battery (leaf `.2`)

Full engine-change battery: `parse_harness_semantic_gate` (re-anchored) + `parse_harness_equivalence_gate`
(11 CERTIFIED byte-identical or explicitly re-baselined — SV WILL change if `.1` confirms; that is the
fix working, handled as a re-baseline with the reproducer as evidence) + `parse_harness_combinator_gate`
+ SV cert union gate seeds 0/7/42 + external corpus 14/14 + `ast_shape_contract` + v2005 conformance +
clippy + `mdbook_docs_gate`. SV ledger entry + release bump + conformance re-lock if SV-observable
(expected YES on hierarchical tf inputs). Book/normative-spec lockstep: the inline-branch-predicate
semantics section states BRANCH-LOCAL as the contract.

## 5. Leaves

- `.1` — **EVIDENCE: the SV:4605 reproducer + blast-radius audit — `not-started` (frontier).**
  (a) Author a minimal SV input whose parse must route a hierarchical tf-name through
  `scoped_or_hierarchical_tf_identifier`; run `parseability_probe --parse-dump-ast-pretty` +
  `PGEN_TRACE_VERBOSITY=debug --trace-rules scoped_or_hierarchical_tf_identifier`; look for
  `🛡️ … REJECTED branch 3/3 … unresolved args` (the silent block) — record ACCEPT/REJECT + which
  branch wins TODAY. (b) Grep all `grammars/*.ebnf` for inline `phase: branch`; classify each user.
  (c) `git log -S 'branch_predicates_for_rule'` — was flattening ever a deliberate decision (a
  decision record trumps my bug reading; then the fix needs a director call). NO code.
- `.2` — **FIX: registry-fn locality + SV:4605 verification + re-anchors — `not-started`.** Blocked on
  `.1`. The one-function fix + regen + the §4 battery + the enforced acceptance checklist + SV
  ledger/release handling + book/spec lockstep.

## 6. Current Frontier

| # | Leaf | Status | Notes |
| --- | --- | --- | --- |
| 1 | `.1` (SV:4605 reproducer + blast-radius audit) | `not-started` (**frontier**) | Tools-first; NO code. |
| 2 | `.2` (registry-fn fix + verification) | `not-started` | Blocked on `.1`. Engine tier, full battery, SV release handling. |

## 7. Relationships

- Spawned by [`SEM-FINDINGS`](SEM-FINDINGS.md) (F3). Evidence base: `PARSE-HARNESS.6.2`
  (`PGEN-PARSE-HARNESS-0015`) — `sem_branch_gate`/`sem_branch_select` pins + the scratch-slot trace.
- Touches the SV family (ledger `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`, release policy,
  v2005 conformance locks) and the semantic-annotation normative spec.
