# GRAMMAR-WELLFORMED.H.12.5.8.1 — SVA infix property/sequence binary-operator parse bug: WHY+WHERE

Tools-first WHY+WHERE for the `H.12.5.8` lane-2 leaf — the indirectly/directly left-recursive **infix**
property/sequence binary-operator forms the SV parser rejects (`a ##1 b`, `a and b`, `a or b`,
`a intersect b`, `a within b`, `a until b`, `a s_until b`, …). The leaf mandated *"open the leaf,
tools-first WHY+WHERE first"*; this slice is that investigation and **splits** `H.12.5.8` into the
WHY+WHERE (this, `.8.1`, done) and a fix-design/decision child (`.8.2`, the new frontier).

> Slice `PGEN-GRAMMAR-WELLFORMED-0118` (**PURE-DOCS INVESTIGATION** — no grammar/Rust/generated/
> release/schema/ledger change; clippy not invoked; no gate run). Status: `done`.
> Reads with [[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]],
> [[feedback_no_codebase_change_without_tool_backed_facts]], [[feedback_be_alert_root_cause_fishy_immediately]],
> [[project_grammar_wellformedness_contract]], [[feedback_uvm_is_valid_sv]], [[feedback_always_signoff_decisions]],
> [[feedback_regex_book_live]].
> Parent: `GRAMMAR-WELLFORMED.H.12.5.8` (split by this slice). Broadened by `H.12.5.7.1` (`-0116`) to the
> whole infix property/sequence class; found during `-0105`.

## Reproduction (tools-first — the real SV parser)

`parseability_probe --parse systemverilog <file> --profile 2017` (DEBUG binary `rust/target/debug/parseability_probe`,
Jun 17 19:29 — post-dates the generated `systemverilog_parser.rs` Jun 17 18:44; `-0117` was generator-only,
no parser regen, so the binary is parse-accurate). Sample shell:
`module m; logic a, b; assert property (<op>); endmodule`.

| Construct | verdict | furthest_position |
|---|---|---|
| `a` (bare) | **PASS** | — |
| `a ##1 b` (cycle-delay) | **FAIL** | 42 |
| `a and b` | **FAIL** | 40 |
| `a or b` | **FAIL** | 40 |
| `a intersect b` | **FAIL** | 40 |
| `a within b` | **FAIL** | 40 |
| `a until b` (property) | **FAIL** | 40 |
| `a s_until b` (property) | **FAIL** | 40 |

Bare `a` parses; **every** infix binary operator rejects right at the operator (the same locus the
`-0116` adjudication recorded as `furthest≈47`, here `40/42` in the minimal `module` shell). The whole
infix sequence/property binary-operator layer is unparseable — valid IEEE 1800 SystemVerilog rejected, so
by the attribution rule and [[feedback_uvm_is_valid_sv]] this is a genuine parser defect, not generator
over-generation.

## Scope (the affected grammar branches)

- `sequence_expr` (`grammars/systemverilog.ebnf:4573`) — **direct** left recursion, self-binary
  (`A := A op A`): `sequence_expr cycle_delay_range sequence_expr (…)*` (b1), `sequence_expr kw_and
  sequence_expr` (b5), `… kw_intersect …` (b6), `… kw_or …` (b7), `… kw_within …` (b10).
- `property_expr_sv_2017` (`:4055`) / `property_expr_sv_2023` (`:4126`) — **indirect** left recursion via
  the union wrapper `property_expr := property_expr_sv_2017 | property_expr_sv_2023` (`:4198`): the branches
  `property_expr kw_or/kw_and/kw_until/kw_s_until/kw_until_with/kw_s_until_with/kw_iff property_expr`,
  `property_expr implies property_expr`, plus the `sequence_expr hash …# property_expr` overlap/imply forms.

(The **prefix** temporal operators — `accept_on`/`eventually`/`nexttime`/… — are a *different* problem,
already resolved as a generator-reach gap in `H.12.5.7.2` / `-0117`. This leaf is the **infix** class only.)

## WHERE (two layers, code-grounded)

1. **Generated parser (symptom site).** `parse_sequence_expr` and `parse_property_expr_sv_2017` are emitted
   **directly from the left-recursive grammar** — `generated/systemverilog_parser.rs` contains **zero**
   `_lr_base`/`_lr_suffix` functions (whole-file `grep -c` = 0). So the rules rely on the runtime
   cycle-breaker, not on an eliminated base/suffix.
2. **Codegen pass (root site).** `RustASTPipeline::eliminate_left_recursive_patterns`
   (`rust/src/ast_pipeline/mod.rs:1627`, default-on via `eliminate_left_recursion: true` `:1364/1588`) →
   `detect_left_recursive_chain_plan` (`:1692`) collects suffixes **only** from alternatives for which
   `extract_rule_reference_name(alternative)` (`:2127`) succeeds — i.e. an alternative that is a **bare
   rule reference** (a single `rule_reference` Atom, or a 1-element Sequence wrapping one). It then requires
   `extract_wrapper_suffix` (`:2170`) to find that the referenced wrapper rule is **entirely** `base suffix`
   (for an `Or`, **every** alternative must be base-prefixed via `sequence_suffix_if_prefixed_with_rule`
   `:2154`). If no such wrapper alternative exists, `wrapper_rules` is empty → `detect_…` returns `None`
   (`:1718`) → the rule is **skipped**.

## WHY (root mechanism)

- The SV infix branches are **multi-element inline Sequences** (`sequence_expr kw_and sequence_expr`), so
  `extract_rule_reference_name` returns `None` for them (length ≥ 2) → they are never collected as wrapper
  suffixes. `sequence_expr` therefore matches no chain plan and is left **un-eliminated**.
- `property_expr` *is* a union of bare references, but `extract_wrapper_suffix("property_expr",
  "property_expr_sv_2017", …)` bails: `property_expr_sv_2017`'s **first** alternative is `sequence_expr`
  (not `property_expr`-prefixed), and the `Or` arm of `extract_wrapper_suffix` requires **all** alternatives
  base-prefixed → returns `None`. So the property layer is left **un-eliminated** too.
- At parse time the un-eliminated left-recursive branch re-enters the same rule at the same position and the
  runtime cycle-breaker (`mutual_recursion_handler.rs:119` `CycleType::LeftRecursive => false`) aborts it.
  Trace of `a and b` scoped to `sequence_expr` (`--trace-rules sequence_expr`) shows exactly this:

  ```
  🚪 Entering branch 2/12 for rule 'sequence_expr' at position 39   # the direct-LR `sequence_expr cycle_delay_range …`
  💥 Infinite recursion detected in rule 'sequence_expr' at position 39
  ❌ Branch 2/12 for rule 'sequence_expr' failed at position 39
  🚪 Entering branch 3/12 …                                          # `expression_or_dist (boolean_abbrev)?` → consumes only `a`
  ```

  The non-left-recursive branch consumes only the first operand `a`; the operator branches (`kw_and`, …)
  are all left-recursive and are likewise cut by the cycle-breaker, so the position never advances past `a`
  → `and b` is unparseable → `furthest_position` parks at the operator.

### Decisive root-cause experiment (the eliminator does NOT handle direct LR at all)

A minimal grammar `expr := expr plus term -> {…} | term -> {…}` (a textbook direct-LR binary) through
`ast_pipeline … --generate-parser`:

- `PGEN_TRACE_VERBOSITY=debug` shows `eliminate_left_recursive_patterns` runs but reports
  **`Completed left-recursion elimination pass (0 transformations)`**.
- the generated parser has **zero** `_lr_base`/`_lr_suffix` and a single `parse_expr`.
- the pipeline's own post-gen self-verification **fails** (`Parser did not consume full input`) — the tiny
  parser cannot parse `1 + 2` either, the identical class of failure.

So this is **not** SV-specific and **not** a broken suffix: PGEN's LR-eliminator only ever transforms the
**indirect bare-reference wrapper-chain** shape (the `module_path_expression` / `-0111` case). **Direct
inline left recursion (`A := A α | β`) is unhandled by construction**, and self-binary `A := A op A` plus
the union-mediated indirect form fall outside the recognized shape.

## Book-accuracy finding (for the fix leaf to reconcile)

`docs/book/src/developer-architecture.md` states *"You write the natural left-recursive EBNF (e.g.
`expr := expr "+" term | term`); the AST pipeline eliminates it … handling direct and indirect/chained
cases."* The decisive experiment above shows the **direct** case is **not** handled (0 transformations,
parser rejects `1+2`). This is a book↔codebase drift ([[feedback_regex_book_live]]); its correction is
coupled to the fix decision (`.8.2`) — if the fix adds genuine direct-LR elimination the claim becomes
true; if the fix restructures the grammar instead, the claim must be narrowed to the chain-only shape.
Recorded here; **not** edited in this pure-docs investigation slice to avoid documenting a non-decision.

## OUTCOME — `H.12.5.8` split

- **`H.12.5.8.1`** (this slice, `done`): WHY+WHERE. PURE-DOCS, no behaviour change — SV stays
  `UNKNOWN=56`, the only non-fully-certified shipped grammar.
- **`H.12.5.8.2`** (`pending`, the new lane-2 frontier leaf): **fix design + decision** between two
  characterized directions (decide tools-first/research-grounded; do NOT implement before the design leaf):
  - **(A) Grammar restructure** — rewrite `sequence_expr`/`property_expr` into the proven non-left-recursive
    `next (OP next)*` iterative idiom already used across the SV expression grammar
    ([[feedback_quantified_group_extraction]] `binary_operator` idiom; the `-0111` non-LR conditional-suffix
    precedent). Grammar-local, lower engine blast radius, LRM-faithful, but must encode IEEE 1800 §16
    operator precedence/associativity and likely changes AST shape/schema.
  - **(B) Engine fix** — implement genuine **direct** left-recursion elimination in
    `detect_left_recursive_chain_plan` (the classic `A := Aα|β ⇒ A := β α*`). General/parser-agnostic, makes
    the book claim true, fixes every grammar; but high blast radius, must be strictly additive / byte-identical
    for all currently-shipped grammars, and the self-binary `A := A op A` operand-on-both-ends + precedence/
    associativity need care (the `-0117` interaction subtlety is the cautionary precedent).
  - implement leaf (`.8.3`) follows the design decision.

## VERIFICATION (this slice)

- Tools-only investigation; **no** code/grammar/generated change ⇒ no clippy, no regen, no gate run.
- Reproduction is parser-deterministic (`parseability_probe` verdicts above); the direct-LR experiment is
  pipeline-deterministic (`0 transformations` at debug verbosity).
- No live-status row changes: SV remains `Mostly Done`, cert `UNKNOWN=56`. `LIVE_ACHIEVEMENT_STATUS.md`
  unchanged.

## Artifacts (scratch — not tracked, under `/tmp`)

- `/tmp/h1258/probes/*.sv` — the minimal per-operator `parseability_probe` adjudication samples.
- `/tmp/h1258/seq_and_trace.log` — `--trace-rules sequence_expr` trace of `a and b` (the cycle-breaker abort).
- `/tmp/h1258/tinylr.ebnf` + `tinylr_parser.rs` — the minimal direct-LR experiment (0 transformations).
