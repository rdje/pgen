# GRAMMAR-WELLFORMED.H.12.8.3.1 — close the 2 `…scoped_call…` cousin reach-gaps (branch-1 longest-match shadowing)

Tool-proven WHY+WHERE + grammar-gate fix for **2 of the 3** SV canonical reach-gaps (the `.8.3`
cohort): `known_unscoped_class_scoped_call_interface_class_identifier` and
`known_unscoped_class_scoped_call_type_parameter_identifier`. Both are shadowed in
`class_scoped_call_prefix` by branch 1 (`scoped_class_scoped_call_prefix_identifier`, gated only
`lacks_class`) under the engine's longest-match alternation selection. The fix extends branch 1's gate
to also exclude `interface_class` and `type_parameter` heads — the exact tightening predicted by
[[project-sv-full-certification-via-multi-entry]] ("grammar tightening that excludes
type-parameter/interface-class heads from the generic scoped-call alternative; the deferral is now
lifted").

> Slice `PGEN-GRAMMAR-WELLFORMED-0146` (**CODE / released-SV** — `grammars/systemverilog.ebnf` +
> regen). Parent: the SV `UNKNOWN`→0 lane `GRAMMAR-WELLFORMED.H.12.8`, cohort `.8.3` (the 3 canonical
> reach-gaps). Sibling `.8.3.2` = `context_member_method_call` (the harder store-gate, deferred).
> The director-reaffirmed goal is literal `UNKNOWN=0`; closing these 2 drives canonical `22 → 20`,
> union `3 → 1`.
> Reads with [[feedback_systematically_use_debug_toolbox]], [[feedback_pinpoint_real_blocker_not_menu]],
> [[feedback_grammar_rules_must_consult_store]], [[project_grammar_wellformedness_contract]]
> (a dead/shadowed branch is a DEFECT to fix), [[project_cert_coverage_tournament_loser_leak]]
> (commit only improvements), [[feedback_no_codebase_change_without_tool_backed_facts]].

## REPRODUCE (tool output, this session)

Canonical SV cert `(systemverilog_file, sv_2017)` count 40 seed 0 (DEBUG `ast_pipeline`):
```
CERTIFICATE-COVERAGE: grammar='systemverilog' entry='systemverilog_file' samples=40
  total=1304 proof=1 witness=1281 UNKNOWN=22 fully_certified=false (sample_parse_failures=0, proof_reverify_failures=0)
```
`PGEN_CERT_COVERAGE_DEBUG_PROBES=1` — both cousins, every pass (plannable / store-free / carrier-div):
```
[plannable-probe] rule='known_unscoped_class_scoped_call_interface_class_identifier' parsed=true witnessed_target=false sample="typedef interface class\foo ;(*\foo_0 =\foo ::\foo_0 ()*);"
[plannable-probe] rule='known_unscoped_class_scoped_call_type_parameter_identifier'  parsed=true witnessed_target=false sample="localparam type\foo ;(*\foo_0 =\foo ::\foo_0 ()*);"
```
`parsed=true witnessed_target=false` ⇒ a **reach/routing gap** (the sample parses, but the target
branch is never the one that fires), NOT a store-gate.

## ROOT CAUSE (WHY + WHERE — scoped trace, `--trace-rules class_scoped_call_prefix,…`)

Witness `module m;typedef interface class \foo ;initial \foo ::\foo_0 ();endmodule` (statement
context). The forward typedef DOES emit the fact (`declared_forward_interface_class_identifier`
`:5174` → `@emit_fact declaration_family=interface_class`; trace: `has_fact(type_name,\foo)→true`).
At the call `\foo::\foo_0()`, `class_scoped_call_prefix` (`grammars/systemverilog.ebnf:6263`)
evaluates all four head branches:
```
🚪 Entering branch 1/4 …  ✅ non_typedef_package_scope parsed 46→54 (' \foo ::')
                          ✅ class_identifier parsed 54→60 ('\foo_0')
                          ✅ scoped_class_scoped_call_prefix_identifier parsed 46→60 (14 bytes ' \foo ::\foo_0')
✅ Leaving branch 1/4 … at position 60 (success)
🚪 Entering branch 2/4 …  🚫 known_unscoped_class_scoped_call_class_identifier rejected (fact …declaration_family,class)
🚪 Entering branch 3/4 …  ✅ known_unscoped_class_scoped_call_interface_class_identifier parsed 46→51 (5 bytes ' \foo')
✅ Leaving branch 3/4 … at position 51 (success)
🚪 Entering branch 4/4 …  🚫 known_unscoped_class_scoped_call_type_parameter_identifier rejected (fact …type_parameter)
```
- **WHERE:** branch 1 = `scoped_class_scoped_call_prefix_identifier` (`:6261`), gated ONLY by
  `lacks_fact_attribute_equals(type_name, $scope.body.name.body, declaration_family, class)` (`:6260`).
- **WHY:** for an `interface_class`/`type_parameter` head `\foo`, `lacks_class` **passes** (the head
  is not a class), so branch 1 matches the **longer** `\foo::\foo_0` (14 bytes, the `pkg::class`
  reading — `package_identifier := declaration_identifier` `:3520` is ungated). Under the engine's
  **longest-match** alternation selection (documented at `:6247-6249`), branch 1's 14-byte match beats
  the correct branch 3's 5-byte (`\foo`) match. Branch 1 then fails the mandatory trailing
  `scope_resolution` (only one `::`), so `class_scoped_call_prefix` → `class_scoped_tf_call` fail, and
  `subroutine_call`/expression falls through to the ungated `package_scope` `tf_call` (AST: `kind:
  package_scope` + `tf`). The cousins are never the selected branch ⇒ `witnessed_target=false`.
- **Asymmetry confirmed:** the `class` head (branch 2) IS witnessed because `lacks_class` correctly
  rejects branch 1 for it, letting branch 2 fire. The two non-class families have no such exclusion.

## FIX (declarative grammar gate — fix-hierarchy tier: grammar/store-predicate, level 1)

Extend branch 1's gate to exclude **all three** non-package type families. Stacked `@predicate`
directives compose with AND (template: `known_unscoped_block_class_type` `:1642-1644`). Add two lines
beside the existing `lacks_class` on `scoped_class_scoped_call_prefix_identifier` (`:6260-6261`):
```
@predicate: { name: lacks_fact_attribute_equals, args: [type_name, $scope.body.name.body, declaration_family, class],          phase: post, view: shaped }
@predicate: { name: lacks_fact_attribute_equals, args: [type_name, $scope.body.name.body, declaration_family, interface_class], phase: post, view: shaped }
@predicate: { name: lacks_fact_attribute_equals, args: [type_name, $scope.body.name.body, declaration_family, type_parameter],  phase: post, view: shaped }
scoped_class_scoped_call_prefix_identifier := non_typedef_package_scope class_identifier -> {scope: $1, name: $2}
```
Effect: for an interface_class/type_parameter head, branch 1 is rejected ⇒ branch 3/4 (the only
remaining matches) win ⇒ the cousins witness. A genuine package scope (`pkg::Class::method()`) lacks
all three type families ⇒ branch 1 still fires (no regression). The class head is unchanged.

`scoped_class_scoped_call_prefix_identifier` is used ONLY in `class_scoped_call_prefix` (`:6263`) —
surgical, parser-agnostic, inert for every other grammar.

**Behavior note (ceremony):** the FIX is acceptance-preserving for the witness strings
(`interface_class::m()` / `type_parameter::m()` still parse — via `class_scoped_tf_call` now instead of
the `package_scope` fallback) but it CHANGES the AST shape for those constructs (`package_scope`/`tf`
→ `class_scoped_tf`), which is a route CORRECTION. Released-SV lockstep applies: ledger entry +
release bump; schema unchanged (no new node kinds); SV external corpus must stay 14/14; ast_shape
contract reviewed.

## Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — canonical cert `UNKNOWN=22`; both cousins `parsed=true witnessed_target=false` on every pass (pasted above).
- [x] **ROOT CAUSE (WHY + WHERE)** — scoped `--trace-rules` proves branch 1 (`scoped_class_scoped_call_prefix_identifier` `:6261`, gated only `lacks_class` `:6260`) longest-matches `\foo::\foo_0` and shadows cousins #3/#4; falls through to ungated `package_scope` `tf_call`. Asymmetry vs the witnessed `class` head explained.
- [x] **FIX** — added two AND-stacked `@predicate` directives (`lacks_fact_attribute_equals … interface_class` + `… type_parameter`) on `scoped_class_scoped_call_prefix_identifier` (`grammars/systemverilog.ebnf:6261`), template `known_unscoped_block_class_type` `:1642-1644`. Minimal, declarative, parser-agnostic; rule used only in `class_scoped_call_prefix`.
- [x] **ADDRESSED (verified)** — route re-check: the 2 witness samples now parse via `class_scoped_tf` (zero `package_scope`). Canonical cert `UNKNOWN 22 → 20` (`witness 1281 → 1283`), both cousins `parsed=true witnessed_target=true`, `spf=0`, byte-identical seeds 0/7/42. Union (4-config) `UNKNOWN 3 → 1` (`witness 1300 → 1302`), residual = EXACTLY `["context_member_method_call"]`, byte-identical seeds 0/7/42.
- [x] **NO REGRESSION** — only `generated/systemverilog_parser.rs` regenerated ⇒ the 6 fully-certified grammars byte-identical by construction; SV external corpus gate ✅ (`primary_blocked_corpus: <none>`, uvm_pkg 2017/2023 preprocess+parse_full `ok`); `cargo test --features generated_parsers --lib` **739 passed; 0 failed** (incl. `ast_shape_contract` + `certificate_coverage_union_is_over_positively_covered_sets` soundness lock); `--lint-grammar` 0 errors (1425 rules); `clippy_on_rust_change` source-clean (grammar-only change; generated debt allowed, confirmed at commit).
- [x] **LOCKSTEP** — SV integration contract `1.0.150 → 1.0.151` + schema-6 note; ledger `SV-0013`; `LIVE_ACHIEVEMENT_STATUS.md` / `CHANGES.md` / `DEVELOPMENT_NOTES.md`; book `grammar-wellformedness.md` (SV residual `3 → 1`); tree (`TASK_TREE.md` frontier + `GRAMMAR-WELLFORMED.md` leaf); `MEMORY.md`. Schema unchanged (no new node kinds — `class_scoped_tf` already in schema 6).

## Owns / next
- `GRAMMAR-WELLFORMED.H.12.8.3.2` — `context_member_method_call` (the remaining reach-gap; store-gated declaration-hosting carrier; needs the store-aware generator extended — deferred-harder).
