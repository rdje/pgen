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

## 1b. Leaf `.1` VERDICT (2026-07-06, session #48, tools-first — all three sub-items evidence-backed)

**CONFIRMED — and BROADER than suspected: branches 1 AND 3 of `scoped_or_hierarchical_tf_identifier`
are BOTH 100% dead today; only branch 2 (package_scope) can ever win. The flattening is a day-one
implementation bug that CONTRADICTS its own introducing commit's documented semantics. No input is
observed to REJECT outright (sibling lanes absorb everything), so the live damage is (i) dead
branches, (ii) double predicate evaluation, (iii) three grammar workaround sites. A NEW finding:
the naive fix would flip statement-position dotted-call AST shapes corpus-wide — `.2` must pair the
engine fix with routing preservation (see §3b).**

### (a) Reproducers + traces (probe = shipped `parseability_probe`, SV profile `sv_2017`)

| Input (statement in `module top; initial begin … end endmodule`) | Verdict today | Winning lane today | `scoped_or_hierarchical_tf_identifier` trace |
|---|---|---|---|
| `foo.bar(1);` (anchored hierarchical) | ACCEPT | `method_call` (`kind:"method"`) | `✅ Leaving branch 3/3 … (success)` → `🛡️ 'lacks_fact_attribute_equals' REJECTED branch 3/3 … unresolved args […, RuleReference("scope.name.body"), …]` → `🚫 Branch 3/3 … rejected by branch predicate` |
| `$root.foo.bar(1);` (rooted) | ACCEPT | `rooted_tf_call_sv_only` (`kind:"rooted_tf"` — the SV-0029 firewall lane) | same branch-3 kill at position 45 |
| `C::m(1);` (class scope, `C` a declared class) | ACCEPT | `class_scoped_tf_call` (subroutine_call lane #1) | `✅ Leaving branch 1/3 … (success)` → `🛡️ … REJECTED branch 1/3 … unresolved args` (branch 1's `scope` = `class_scope = {body:…}` has NO `name` key) |
| `p::f(1);` (package scope, `p` a declared package) | ACCEPT | `tf_call` via **branch 2** (`kind:"package_scope"`) | `🛡️ … PASSED branch 2/3` printed **TWICE back-to-back** — the flattening double-evaluation, live |

So: the rule's `{kind:"class_scope"}` and `{kind:"hierarchical"}` shapes are UNPRODUCIBLE today;
every structural success of branches 1/3 is predicate-killed by the broadcast `$scope.name.body`
unresolvable-ref rule. Acceptance is preserved everywhere by sibling lanes.

### (b) Blast-radius audit (grep `phase: branch` over ALL `grammars/*.ebnf` + scratch)

Exactly **ONE live inline `phase: branch` user in the repository: SV:4605 itself.** Two comment
sites document the flattening and work around it (SV:3452 `wildcard_escape_nettype_identifier`
post-phase helper; SV:3582 same idiom), and a THIRD site (SV:5234-5243, `SV-0029`
`SV-DOLLAR-LRM-FIDELITY.4`) both documents it ("that rule's branch-2 typedef-exclusion predicate is
BROADCAST to every branch by the runtime … trace-proven") **and RELIES on it as a routing
firewall**: `rooted_tf_call_sv_only := !( identifier ) hierarchical_tf_identifier …` exists
precisely because branch 3 is dead, with the `!( identifier )` guard added "so every
identifier-headed TF call keeps its existing route byte-identically".

### (c) History adjudication (`git log -S`)

- `9fc91261` (2026-03-20, "Add semantic branch predicate seam"): `branch_predicates_for_rule`
  returned ONLY rule-level directives — the exact semantics the fix restores.
- `43bbc43c` (2026-03-21, "Compile branch-local semantic annotations"): added the flat-map — while
  its own CHANGES.md entry documents the intended semantics as *"generated branch selection now
  evaluates: rule-wide branch predicates **plus branch-local predicates for the candidate branch
  only**"*. The implementation contradicts the intent recorded in the same commit.
- VERDICT: **flattening was never a deliberate decision — it is an implementation bug**, later
  discovered and worked around three times in the SV grammar (3452 / 3582 / 5234), then pinned
  differentially by `.6.2`'s `sem_branch_gate`. No decision record blesses it. The fix proceeds.

### NEW finding (routing consequence — surfaced to director 2026-07-06)

Reviving branches 1/3 does NOT merely "make hierarchical tf-names parseable through this rule" —
under `priority_first` it would FLIP the winning lane for dotted calls that today fall through to
later lanes:

- **statement position** (`subroutine_call`, SV:5248-5255): `tf_call` is lane #3, `method_call`
  lane #5 → EVERY `x.y(...)`-shaped statement call (`obj.m(1);` — ubiquitous in UVM) would flip
  `{kind:"method"}` → `{kind:"tf", body:{…,kind:"hierarchical"}}`.
- **chain-initial** (`chainable_call_initial`, SV:3151-3155): `tf_call_with_args` #3 precedes
  `direct_callable_method_call` #4 → `a.b(1).c()` chain roots flip similarly.
- **expression position** is safe: in `call_primary` (SV:3174-3185)
  `split_direct_callable_method_call` (#5) wins before `tf_call_with_args` (#9).

A corpus-wide statement-call shape migration has NO correctness gain (`obj.m(1);` is genuinely
parse-time-ambiguous between method call and hierarchical tf call; both shapes are faithful) and
would break the released SV contract downstream. Project precedent (SV-0029's firewall, release
policy, shape-contract discipline) says PRESERVE routing. Therefore `.2` = the engine fix **+ an SV
companion that keeps today's winners byte-identical** (mechanism decided by `.2` measurement-first:
candidate = explicitly retiring the dead branches 1/3 at SV:4603-4607 so the revived engine has
nothing to re-route, with cert/witness impact measured; naive lane reordering is NOT
byte-identical — e.g. `p::f(1);` wins via branch 2 today and method-lane machinery also probes
package receivers).

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

- `.1` — **EVIDENCE: the SV:4605 reproducer + blast-radius audit — `done`**
  (`PGEN-BRANCH-PREDICATE-LOCALITY-0001`, 2026-07-06 session #48; full evidence in §1b).
  (a) Four reproducers traced — branch 3 AND branch 1 confirmed structurally-successful-then-
  predicate-killed (`🚫 Branch 3/3 … rejected by branch predicate` / `🚫 Branch 1/3 …`); branch 2
  double-evaluates; every reproducer still ACCEPTS via sibling lanes (method / rooted_tf /
  class_scoped_tf). (b) Blast radius: ONE live inline `phase: branch` in all grammars = SV:4605;
  three SV comment sites document/workaround the flattening (3452, 3582, 5234 — the last RELIES on
  it as a routing firewall). (c) History: the flat-map contradicts its own introducing commit's
  (`43bbc43c`) documented "candidate branch only" semantics — an implementation bug, never a
  decision. PLUS the new routing-consequence finding (§1b): the fix must preserve statement/chain
  lane winners. NO code (verified: docs-only commit).
- `.2` — **FIX: registry-fn locality + SV routing preservation + re-anchors — `not-started`
  (frontier).** The one-function engine fix, PLUS the §1b-mandated SV companion (measurement-first:
  regen in a sandbox, diff shape-contract + external corpus winners, then retire/guard the dead
  branches so today's routing stays byte-identical), + regen + the §4 battery + the enforced
  acceptance checklist + SV ledger/release handling + book/spec lockstep.

## 6. Current Frontier

| # | Leaf | Status | Notes |
| --- | --- | --- | --- |
| 1 | `.1` (SV:4605 reproducer + blast-radius audit) | `done` (2026-07-06 #48) | CONFIRMED broader: branches 1+3 both dead; flattening = day-one bug; routing-flip risk found. |
| 2 | `.2` (registry-fn fix + SV routing preservation + verification) | `not-started` (**frontier**) | Engine tier + SV companion, full battery, SV release handling. |

## 7. Relationships

- Spawned by [`SEM-FINDINGS`](SEM-FINDINGS.md) (F3). Evidence base: `PARSE-HARNESS.6.2`
  (`PGEN-PARSE-HARNESS-0015`) — `sem_branch_gate`/`sem_branch_select` pins + the scratch-slot trace.
- Touches the SV family (ledger `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`, release policy,
  v2005 conformance locks) and the semantic-annotation normative spec.
