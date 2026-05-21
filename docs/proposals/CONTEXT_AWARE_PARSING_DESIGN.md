# Context-Aware Parsing via Semantic Annotations

**Status:** DRAFT — for review and amendment. No code change attached.
**Owner:** PGEN engine + SystemVerilog grammar.
**Authors:** discussion 2026-05-21, after `.3.3.4.b.4` diagnosis (uvm_pkg `if(a.b.c(x))` failure).
**Restore tag for the pre-design baseline:** `checkpoint/post-3-3-4-b-3-layer-0-clean-pre-next-lrm-defect` @ `4195ee22`.

> This proposal is intentionally *amendable*. Anywhere you see **`[TBC]`** or **`Alternative:`**, that section is up for revision. The architectural principle in §1 is the part we want to land first; the mechanism in §3–§6 follows from it.

---

## 1. Principle

The parser is **a process with memory**. Semantic annotations are how that memory is built, organised, scoped, and queried. Their original role — *steering the parser generator at codegen time* — and their newer role — *steering the parser at parse time* — are converging into a single role: **"things we've seen and decided are with us wherever we go, and they help us decide what to do next."**

Consequence: when an EBNF rule has two or more syntactically-overlapping alternatives that the LRM cannot disambiguate by tokens alone (because the LRM grammar is intentionally permissive), the disambiguation comes from **prior context** — not from a syntactic patch, not from a lookahead trick, not from a "best-guess" priority order.

We therefore reject any "fallback path" that triggers when context is missing. If we have the right semantic at the right time, we always know which branch to take. Missing context = a bug to fix at the producer/scope side, not a fallback to take at the consumer side.

## 2. The concrete problem the design must solve

The immediate driver (and an excellent stress test) is the residual uvm/uvm_compat blocker.

**Minimal failing repro:**
```sv
package p;
function int f();
  if (seed_map.seed_table.exists(type_id)) begin
  end
endfunction
endpackage
```

The parser has no idea what `seed_map` is. Depending on the answer, the correct route for `seed_table.exists(type_id)` is one of:

| If `seed_map` is | then `seed_map.seed_table` is | then `.exists(type_id)` is | route |
|---|---|---|---|
| a class instance with associative-array field `seed_table` | array reference | array-method call | `built_in_method_call` |
| a class instance with method `seed_table` returning an object | method-call returning object | regular method call | `method_identifier (args)` |
| a hierarchical name path | a sub-hierarchical name | a free function call | `tf_call_with_args` or `system_tf_call` |

The LRM-extracted grammar has all three alternatives. Today the parser commits to whichever syntactically wins first — that's the drunk-stumbling failure mode.

**Goal:** the parser knows, at the moment it must choose a branch, what `seed_map` is, what `seed_map.seed_table` is, and therefore which alternative is correct.

## 3. Information the parser must carry

At every commit point in the parse, the parser needs persistent access to:

1. **Variable/identifier bindings**, scoped:
   - name → declared type
   - declared type → kind (`class | array | scalar | enum | interface | virtual_interface | unresolved`)
   - if `class`: → member scope (containing this class's fields + methods)
   - if `array`: → element type + array kind (queue / dynamic / associative / fixed)

2. **Type-name registry**, scoped:
   - name → kind (`class | typedef | enum | …`)
   - if `class`: extends/implements + member scope

3. **Scope chain**, organised hierarchically:
   - file → package → class → method → block
   - each scope queryable for "does name N resolve here, and to what?"

4. **Cross-file imports**:
   - import of a package transitively brings in all its exports
   - same primitive as `.3.3.4.a` (`@export_to_library` / `@import_from_library`), extended to richer fact kinds.

This is the "memory" §1 refers to.

## 4. Producers — facts to emit (decl-side semantic annotations)

For every grammar rule that BINDS a name, emit a typed fact. Concrete required additions (non-exhaustive — first cut, **`[TBC]`** during review):

| Decl rule | Fact to emit |
|---|---|
| `data_declaration` / `variable_decl_assignment` | `@emit_fact { kind: variable_binding, name: $name, type_kind: $resolved_type_kind, type_ref: $type_ref }` |
| `class_declaration` | `@emit_fact { kind: type_binding, name: $name, type_kind: "class" }` + `@open_scope` for members |
| `class_property` (= field) | `@emit_fact { kind: class_member, container: $enclosing_class, name: $name, member_kind: "field", type_kind: $type_kind, type_ref: $type_ref }` |
| `class_method` / `method_prototype` | `@emit_fact { kind: class_member, container: $enclosing_class, name: $name, member_kind: "method" }` |
| `parameter_port_declaration` / `tf_port_item` | `@emit_fact { kind: variable_binding, name: $name, type_kind: $type_kind, scope: $enclosing }` |
| `typedef` | `@emit_fact { kind: type_binding, name: $name, type_kind: $aliased_type_kind, body: $type_descriptor }` |
| `package_declaration` | `@open_scope kind:package, name:$name` (already done) |

The `type_kind` value is one of: `class | array | queue | dynamic_array | assoc_array | enum | scalar | interface | unresolved`.

**Open question (review):**
- **`[TBC]`** What exact shape do we want for `type_ref`? Bare string, or a structured object capturing element type + dimensionality? Tradeoff is fact-payload size vs. query precision.

## 5. Consumers — predicates to query (use-site semantic annotations)

At the disambiguation points in the grammar, the predicates that decide which alternative to take. Concretely for the method-call family:

```ebnf
@branch_policy: context_first      # NEW policy: take the predicate-true branch
method_call_body :=
        @predicate receiver_is_array args:[$enclosing_receiver]
        built_in_method_call                                                 -> {kind: "built_in", body: $1}
    |   @predicate receiver_is_class args:[$enclosing_receiver]
        method_identifier attribute_instance* lparen list_of_arguments rparen -> {kind: "method_call_with_args", ...}
    |   @predicate receiver_is_class args:[$enclosing_receiver]
        method_identifier attribute_instance*                                -> {kind: "method_bare_property", ...}
```

Where `$enclosing_receiver` is the receiver positional capture from the **parent** rule (`direct_method_call := method_call_root dot method_call_body` → method_call_body's parent gives us $1 = receiver).

**Predicates needed:**

- `receiver_is_array($r)` — true iff the resolved type of `$r` is `array | queue | dynamic_array | assoc_array`.
- `receiver_is_class($r)` — true iff the resolved type of `$r` is `class` (or class instance).
- `name_resolves($n)` — true iff `$n` resolves to a known binding in any active scope (for the cases where we want positive context but don't care about the specific kind).

The resolver underlying these predicates must do **multi-segment lookup**: given `seed_map.seed_table`, walk through `seed_map`'s class scope to find `seed_table`'s binding.

## 6. Engine primitives — gap analysis

| Primitive | Today | After this proposal |
|---|---|---|
| `@emit_fact` | ✓ | ✓ (no change; new fact-kinds emerge as conventions) |
| `@predicate has_fact` (gate commit) | ✓ | ✓ (kept for fact-existence checks) |
| `@predicate <user_predicate>` with named predicates | partial (a few baked-in: `has_fact`, `lacks_fact_attribute_equals`, …) | **extend with `receiver_is_array`, `receiver_is_class`, `name_resolves`** — same shape as existing predicates, but resolver-backed |
| Predicate references parent-rule positional capture (`$1` of parent) | ✗ | **NEW: `$enclosing_<role>`** or similar mechanism for child rules to reference parent siblings. **Alternative:** lift the predicate to the parent rule level (no engine change needed) — see §7. |
| `@open_scope` / `@close_scope` (flat scopes) | ✓ | ✓, but need **scope kind** (`package | class | function | block`) and the member-scope-of-class concept |
| Multi-segment name resolution (dotted path through scopes) | ✗ | **NEW: scope-resolver primitive** — walk a dotted name through the scope tree, returning the bound type-kind or `unresolved` |
| `@branch_policy: context_first` | ✗ | **NEW: branch-by-predicate selection** at the rule level. **Alternative:** factor each branch into a separate rule whose `@predicate has_fact phase: branch` gates it; existing `priority_first` policy may suffice if predicates fail cleanly. |
| `@export_to_library` / `@import_from_library` | ✓ (kind=package only, MVP-0) | **extend** to export `type_binding`, `class_member`, `variable_binding` |

So three real engine extensions, plus a number of convention/grammar additions:

1. **Scope-kind awareness** in `@open_scope` (package vs. class vs. function), with member-scope-of-class as a queryable nested scope.
2. **Multi-segment resolver** (dotted path through nested scopes).
3. **Branch-by-predicate** at the rule level — *probably* expressible with existing `phase: branch` predicates if we lift the disambiguation up one rule. **`[TBC]` — verify before adding a new primitive.**

## 7. Open questions for review

- **§4 Producers** — is the proposed fact-kind set complete enough for uvm? What about for ports / interface modports / `localparam` / `generate`-emitted names?
- **§5 Consumers** — should the predicates be named (`receiver_is_array`) or expressed via composable boolean operators on facts (`has_fact(variable_binding, $r) AND fact_attribute(variable_binding, $r, type_kind, array)`)? Tradeoff: named predicates are easier to read, composable ones are more general.
- **§6 Engine primitives** — do we add `$enclosing_receiver` (capture-from-parent), or lift the predicate to the parent rule level so existing `$1` works? The latter is grammar-only (no engine change) but requires restructuring; the former is engine-level (one new primitive) but keeps the grammar shape.
- **Failure mode when context is unknown** — per §1 there is no fallback. So what does the parser do when it hits `obj.method(args)` and `obj` is genuinely unresolved (e.g., parsed in isolation, no library)? Options:
  - **(a)** the parse is a hard error — "unresolved receiver, cannot disambiguate".
  - **(b)** a deliberate "unresolved" type kind that has its own queryable predicate, and the grammar has an explicit alternative for the unresolved case (still context-aware, just one of the contexts is "I don't know").
  - **`[TBC]`** — preference?

## 8. Phased implementation plan (proposal — for amendment)

- **Phase 0 (this doc)** — design proposal review + amendment until landed.
- **Phase 1 — Engine: scope-kind awareness + multi-segment resolver.** Parser-agnostic. Verified by a tiny synthetic grammar that exercises class + member-scope + dotted lookup. **Leaf:** `.3.3.4.b.5.1`.
- **Phase 2 — Engine: branch-by-predicate (or grammar-restructure equivalent).** Decide between the two paths during Phase 1 (verify whether existing `phase: branch` predicates can reference parent-rule captures). **Leaf:** `.3.3.4.b.5.2`.
- **Phase 3 — Engine: extend `@export_to_library` / `@import_from_library` to richer fact kinds.** **Leaf:** `.3.3.4.b.5.3`.
- **Phase 4 — SV grammar producer pass.** Add `@emit_fact` to every decl-site rule per §4. No other changes. Verify facts emit correctly via a probe. Re-run uvm corpus — parse still fails (consumers not yet wired), but no regression elsewhere. **Leaf:** `.3.3.4.b.6.1`.
- **Phase 5 — SV grammar consumer pass.** Wire predicates from §5 into the method-call disambiguation rules. Drop the symptomatic `( lparen list_of_arguments rparen )?` workarounds. Re-run uvm — expect parse to succeed on the diagnosed construct. **Leaf:** `.3.3.4.b.6.2`.
- **Phase 6 — verification + lockstep.** Full triage gate; lib tests; RGX 44/0; SV shape-contract; book lockstep; release bump 1.0.126 → 1.0.127. **Leaf:** `.3.3.4.b.6.3`.

Each leaf commits independently with its own contract bump and lockstep. No leaf depends on a future leaf to be valid.

## 9. Non-goals (deliberately out of scope)

- Full SV type checking. We need *enough* type information to disambiguate the grammar — not enough to type-check expressions.
- LRM-strict adherence in cases where LRM permissiveness is the bug we're fixing. The LRM's BNF is the *spec for what is syntactically valid*; our PGEN grammar is the *spec for what we parse, which is a deterministic subset chosen via context*.
- Parser-specific features. Every engine extension here must be applicable to VHDL, RTL, and future grammars — `feedback_ast_pipeline_parser_agnostic`.

## 10. Restore points

- Pre-design baseline (current HEAD): `checkpoint/post-3-3-4-b-3-layer-0-clean-pre-next-lrm-defect` @ `4195ee22`.
- Pre-Layer-0 baseline (kept for reference): `checkpoint/post-3-3-4-b-1-clean-pre-layer-0` @ `f758b878`.

## 11. References

- `feedback_ast_pipeline_parser_agnostic` (memory) — every pipeline change must be a general primitive.
- `feedback_prefer_grammar_leave_engine_alone` (memory, refined) — engine changes ARE on the table when they're parser-AGNOSTIC features that make the EBNF cleanly express what the language needs.
- `feedback_verify_rule_correctness_before_runtime_hypotheses` (memory) — when a parse failure is tied to a rule, READ THE RULE AGAINST THE SPEC FIRST. (This proposal extends the discipline: also, ASK WHETHER THE LRM HAS GIVEN US ENOUGH TO DISAMBIGUATE — and if not, that's the disambiguation we owe via context.)
- `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` §1.0.121 — the `.3.3.3` IIFE exception-safety fix that landed `@open_scope`/`@close_scope`/`@emit_fact`/`has_fact` reliably. This proposal builds on that.
- `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` §1.0.122 — the `.3.3.4.a` MVP-0 library mechanism. This proposal extends its fact-kind set.

---

**Amend freely.** When the principle in §1 and the open questions in §7 are settled, we can start scoping Phase 1.
