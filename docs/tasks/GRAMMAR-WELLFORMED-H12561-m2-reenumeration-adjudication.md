# GRAMMAR-WELLFORMED.H.12.5.6.1 — M2 re-enumeration + per-rule over-gen-vs-parser-bug adjudication (at `UNKNOWN=84`)

The opening slice of the M2 leaf (`H.12.5.6`, M2 over-generation / store-unfaithful) — the
director's BINDING 2026-06-17 priority lane 1 (SV `UNKNOWN`→0). The `-0083` M2 classification was
made at `UNKNOWN=121`; nine intervening slices have since driven the residual to **84**, so the M2
set had to be **re-enumerated tools-first** against the current baseline before any fix. This slice
does exactly that, splits the 36-rule M2 set into its actual mechanisms, and carries the **first**
adjudication (the constraint cluster) through a decisive WHY+WHERE.

> Slice `PGEN-GRAMMAR-WELLFORMED-0112` (**PURE-DOCS INVESTIGATION** — NO code/grammar/generated/
> release/schema/ledger change; clippy not invoked). Status: `done`. Splits `H.12.5.6` →
> `.6.1` (this), `.6.2` (M2a constraint reach-honesty fix — next frontier), `.6.3` (M2b store-gated
> identifiers — parked-adjacent). Reads with [[feedback_tools_first_no_guessing]],
> [[feedback_why_and_where_before_solution]], [[feedback_no_codebase_change_without_tool_backed_facts]],
> [[feedback_be_alert_root_cause_fishy_immediately]], [[feedback_grammar_rules_must_consult_store]].

## Baseline reproduced (deterministic ⇒ signal)

`./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage
--grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0` (with
`PGEN_CERT_COVERAGE_DUMP_ALL=1` / `PGEN_CERT_COVERAGE_DEBUG_PROBES=1`):

`total=1289 proof=1 witness=1204 UNKNOWN=84 fully_certified=false (sample_parse_failures=0,
proof_reverify_failures=0)` — matches the layer-A pointer exactly.

## The 84 partition (tool-backed, from the per-rule plannable-probe outcomes)

The cert report's own probe diagnostics partition the 84 cleanly (sums exactly: 19+36+14+15 = 84):

| Bucket | Count | Signal | Owner |
|---|---|---|---|
| `no_path` (alternate-entry A1 / sv_2023-profile A2 / decomposition artifacts) | **19** | "NO reach path from the entry" warning line | adjudicated NON-defects (`H.12.6`) — STAY |
| **M2** — parser REJECTS the witness (`parsed=false`) | **36** | every probe attempt `parsed=false` | **`H.12.5.6`** (this leaf) |
| **M1-residual** — `parsed=true`, routed-elsewhere | **14** | probe parses but bytes route through a sibling | `H.12.5.5` (M1 reach lane) |
| **M3** — no plannable reach probe at all | **15** | rule never appears in a `[plannable-probe]` line | `H.12.5.7` (M3 property/sequence temporal) |

- **M1-residual (14, → `H.12.5.5`):** `known_unscoped_class_scope_type_parameter_identifier`,
  `known_unscoped_base_class_type_parameter_identifier`, `known_unscoped_let_identifier`,
  `context_member_method_call` (the PARKED one), `named_checker_port_connection(_sv_2017)`,
  `property_actual_arg`, `known_unscoped_parameter_identifier`, `repeat_range`,
  `with_covergroup_expression`, `known_unscoped_class_scoped_call_{class,interface_class,type_parameter}_identifier`,
  `class_scoped_tf_call`.
- **M3 (15, → `H.12.5.7`):** `property_case_item` + the SVA temporal `kw_*` cluster
  (`kw_accept_on`, `kw_eventually`, `kw_nexttime`, `kw_reject_on`, `kw_s_always`, `kw_s_eventually`,
  `kw_s_nexttime`, `kw_s_until`, `kw_s_until_with`, `kw_sync_accept_on`, `kw_sync_reject_on`,
  `kw_until`, `kw_until_with`, `kw_constant`).

## M2 (36) is THREE distinct mechanisms — not one "over-gen" class

Re-classified by reading the actual generator samples the parser rejected (`PGEN_CERT_COVERAGE_DEBUG_PROBES=1`)
and probing each through `parseability_probe --parse systemverilog … --profile sv_2017`:

### M2a — constraint/sequence/property bodies via a store-gated out-of-class context (≈18) → `.6.2`

Rules: `constraint_block_item`, `constraint_expression`, `constraint_primary(_sv_2017)`,
`constraint_set`, `extern_constraint_declaration(_sv_2017)`, `solve_before_list`,
`uniqueness_constraint(_sv_2017)`, `constant_cast`, `property_qualifier`, `hierarchical_array_identifier`,
`ps_or_hierarchical_array_identifier`, `index_variable_identifier`, `loop_variables`, and the
constraint keyword leaves `kw_before` / `kw_foreach` / `kw_soft` / `kw_solve`.

**Decisive WHY+WHERE.** Every M2a witness sample shares ONE context — the out-of-class constraint
definition `constraint \foo ::\foo { … }` — and ALL reject at `furthest_position=22` (the parser
consumes `constraint \foo ::\foo` and chokes on `{`). The discriminating A/B:

| input | verdict |
|---|---|
| `constraint C::c { x < 5; }` (C **undeclared**) | **REJECT** (furthest 15) |
| `class C; endclass`⏎`constraint C::c { x < 5; }` (C declared) | **PASS** |
| `class C; rand int x; constraint c { x < 5; } endclass` (in-class) | **PASS** |
| `class C; rand int x; rand int y; constraint c { unique {x, y}; } endclass` | **PASS** |
| `class C; rand int x; rand int y; constraint c { solve x before y; } endclass` | **PASS** |

`--trace-rules` on `constraint \foo ::\foo {}` shows it mis-routed through
`data_declaration → … → class_scope → scope_resolution` (seeking `::` after `constraint`); the
out-of-class `constraint_declaration` never fires because **`\foo` is not a declared class**.

**ADJUDICATION:** NOT a parser bug (sound store-gating per [[feedback_grammar_rules_must_consult_store]]
— an out-of-class constraint with an unknown class is genuinely not valid) and NOT structural
over-generation (the syntax shape is fine). It is a **reach-path-selection into a store-gated
context**: the reach BFS witnesses these rules through the *shortest* top-level path — the
out-of-class `constraint <class>::<name> { … }` — whose class scope the minimal `\foo` witness never
declares, when a **non-gated in-class path** (`class C; constraint c { … } endclass`, proven to
parse + witness the same rules) exists. By the attribution rule a witness EXISTS ⇒ generator-reach
deficiency, fixable by a **reach-honesty preference** that deprioritizes store-gated reference edges
in favour of a non-gated path — the same family as the M1 reach fixes (`H.12.5.5.2`/`.3`,
`RTL-FE-CLOSURE.5.x` lineage), and crucially **NOT** the parked store-prelude synthesis. → child `.6.2`.

### M2-sequence-operators (3: `kw_intersect` / `kw_or` / `kw_within`) → routed to `H.12.5.8` (lane 2)

🚨 **A REAL RELEASED-PARSER BUG surfaced (the stimuli generator as bug-finding oracle).** The
sequence keyword leaves never witness because the SV parser rejects the entire SVA sequence/property
**binary-operator** layer. Isolation (hand-written, minimal):

| input | verdict |
|---|---|
| `module m; sequence s; a; endsequence endmodule` (bare operand) | **PASS** |
| `module m; sequence s; a and b; endsequence endmodule` | **REJECT** (furthest 23, at the operator) |
| `module m; sequence s; a or b; endsequence endmodule` | **REJECT** |
| `module m; sequence s; a intersect b; endsequence endmodule` | **REJECT** |
| `module m; sequence s; a within b; endsequence endmodule` | **REJECT** |
| `module m; sequence s; a ##1 b; endsequence endmodule` | **REJECT** (the `H.12.5.8` `##` symptom) |
| `module m; property p; a or b; endproperty endmodule` | **REJECT** |

So only a *single bare* sequence operand parses; `and`/`or`/`intersect`/`within`/`##` are all
rejected. The WHERE: `sequence_declaration → sequence_expr → expression_or_dist` matches the bare
operand, then `sequence_declaration` expects `endsequence` and finds the operator — i.e.
`sequence_expr`'s binary-operator branches never fire. `grammars/systemverilog.ebnf:4573`
`sequence_expr` carries **five left-recursive branches** (`delay_binary` :4575, `and` :4583,
`intersect` :4585, `or` :4587, `within` :4593) interleaved with non-LR branches — the classic
signature of an **LR-elimination defect on a multi-branch left-recursion** (the same family as the
`-0111` `module_path_conditional_expression` fix). This is bigger than the `##`-only lane-2 ticket
(`H.12.5.8`); **`H.12.5.8` is broadened** to "SV parser rejects the entire `sequence_expr` /
`property_expr` binary-operator layer." Pre-existing (not a regression — nothing changed this
session). Full WHY+WHERE + targeted fix + release/ledger/book lockstep owned by `H.12.5.8`.

### M2b — store-gated identifiers whose witness never establishes the fact (≈15) → `.6.3`

Rules: `known_unscoped_{class_scope_class,class_scope_interface_class,block_type,data_type,block_class_type,
covergroup_type,block_covergroup,interface_class_type}_identifier`, `provisional_unscoped_block_class_type`,
`checked_type_identifier`, `checked_nettype_identifier`, `wildcard_escape_nettype_identifier`,
`declared_class_alias_identifier`.

Samples: `localparam \foo \foo ;` (`<type> <name> ;` — `\foo` must be a KNOWN type),
`typedef \foo \foo ;` (the aliased name must be a declared class). The minimal witness never
establishes the consulted store fact, so the gate fails and the rule never positively witnesses —
the SAME family as the PARKED `context_member_method_call` (blocked on `STORE-AWARE-GEN.4b` gen-time
name/value-coupled prelude synthesis, [[project_memory_architecture_adoption]] not relevant; the
director's STANDING "do NOT chase with a fragile name-coupling hack" applies). → child `.6.3`
(parked-adjacent; resolves only when `STORE-AWARE-GEN.4b` lands, unless a non-gated reach path like
M2a's exists for a given rule). `checked_nettype_identifier`'s store gate was already tightened to
`fact_attribute_equals(... declaration_family, nettype)` by `SV-0003` (`-0100`); its residual here is
the witness-reach gap, not the gate.

## Decisions

- `2026-06-17` (`.6.1`): the M2 set at `UNKNOWN=84` is **36 rules in THREE mechanisms**, not the one
  "over-generation/store" class the `-0083` label implied. The split routes work correctly: M2a is a
  *tractable reach-honesty* fix (`.6.2`), M2b is *parked-adjacent* store-prelude (`.6.3`), and the
  sequence-operator finding is a *real parser bug* that belongs to lane 2 (`H.12.5.8`), now broadened.
- `2026-06-17` (`.6.1`): the M2a constraint cluster is adjudicated **parser-correct** — the reach
  BFS, not the parser, is at fault (it prefers a store-gated out-of-class path over a non-gated
  in-class one). This is the attribution rule's *generator-reach* branch, decided tools-first
  (in-class + declared-class forms both PASS), NOT guessed from the rule names.
- `2026-06-17` (`.6.1`): **fix-parser-bugs-ASAP** — the sequence/property binary-operator rejection
  is the highest-severity finding here, but it is already the director's sequenced lane 2
  (`H.12.5.8`); this slice documents + broadens its scope and routes it there rather than re-ordering
  the binding priority. No code touched.

## Verification (read-only — PURE-DOCS)

- Baseline cert reproduced deterministically (`UNKNOWN=84`, `spf=0`); the 19 `no_path` are the
  cert's own warning line; the 36/14/15 split is computed from the per-rule `[plannable-probe]`
  `parsed=`/`witnessed_target=` outcomes (cross-referenced against the `DUMP_ALL` 84-name list).
- Every adjudication A/B above was run through `parseability_probe --parse systemverilog … --profile
  sv_2017` (the released SV parser is the judge), not inferred.
- NO code/grammar/generated/release/schema/inventory/ledger change. Parser-family rows UNCHANGED — SV
  stays the only non-fully-certified shipped grammar (`UNKNOWN=84`).

## Artifacts (scratch — not tracked)

`/tmp/h1256/` — `cert_seed0.log` (baseline + `DUMP_ALL` 84-name + 19-`no_path` lists),
`cert_seed0_probes.log` (per-rule `[plannable-probe]` outcomes), the `g_*`/`v_*`/`t_*`/`s_*` probe
samples (generator-exact constraint witnesses, hand-written valid comparisons, and the SVA
sequence-operator isolation set).
