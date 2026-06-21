# GRAMMAR-WELLFORMED.H.12.5.8.2 — SVA infix binary-operator parse bug: FIX DESIGN + DECISION

Design/decision leaf for the `H.12.5.8` SVA infix property/sequence binary-operator parse bug whose
root cause `.8.1` (`-0118`) established: `sequence_expr`/`property_expr` are never left-recursion-eliminated
(PGEN's eliminator handles only the indirect bare-reference wrapper-chain, not direct inline LR), so the
runtime cycle-breaker blocks every infix branch and `a OP b` rejects at the operator. This leaf **decides
the fix direction** and **designs the grammar restructure**; the implementation is `.8.3`.

> Slice `PGEN-GRAMMAR-WELLFORMED-0119` (**PURE-DOCS DESIGN** — no grammar/Rust/generated/release/schema/
> ledger change; clippy not invoked; no gate run). Status: `done`.
> Reads with [[feedback_why_and_where_before_solution]], [[feedback_tools_first_no_guessing]],
> [[feedback_research_grounded_sota_no_trial_and_revert]], [[feedback_no_workarounds_fix_hierarchy]],
> [[project_vision_and_discipline]], [[project_ebnf_is_single_source_of_truth]],
> [[feedback_ebnf_meta_grammar_lockstep]], [[feedback_uvm_is_valid_sv]], [[feedback_correctness_before_speed]],
> [[feedback_regex_book_live]], [[feedback_quantified_group_extraction]].
> Parent: `GRAMMAR-WELLFORMED.H.12.5.8` (split by `.8.1`).

## The decision: direction A (grammar restructure to the non-left-recursive precedence cascade)

Two candidate directions were characterized in `.8.1`:

- **(A)** rewrite `sequence_expr`/`property_expr` into the proven non-left-recursive `next (OP next)*`
  iterative idiom already used across the SV expression grammar ([[feedback_quantified_group_extraction]]
  `binary_operator` idiom).
- **(B)** implement genuine **direct** left-recursion elimination in the engine
  (`detect_left_recursive_chain_plan`, `rust/src/ast_pipeline/mod.rs:1692`).

**Decision: A.** The deciding fact (tools-grounded, from IEEE 1800-2017 Table 16-3 below): the current
grammar is a **flat, precedence-free** list of binary-operator alternatives
(`sequence_expr := … | sequence_expr and sequence_expr | sequence_expr or sequence_expr | …`), but SVA
operators have a **real precedence and associativity** (`intersect` ≻ `and` ≻ `or`; `until`/`iff`/
implication are right-associative; etc.). To parse SVA *correctly* the grammar **must** become a
precedence cascade **regardless of A or B** — a naive direct-LR elimination `A := β (op β)*` would flatten
all operators to one precedence and left-associativity, which is *wrong*. Given the cascade restructure
is required either way:

- A writes the cascade in the iterative idiom → **no engine change**.
- B writes the *same* cascade left-recursively → the **same grammar work plus** a new, high-blast-radius
  engine feature, for identical correctness.

So **B is dominated by A for this fix**. A is the level-1 declarative fix ([[feedback_no_workarounds_fix_hierarchy]]),
uses the proven idiom, has the smallest blast radius (one grammar file + SV regen; the eliminator and every
other grammar are untouched), and is correctness-first ([[feedback_correctness_before_speed]]). This also
matches the project discipline "use what we have well, don't drift to new mechanisms unless every existing
combination is insufficient" ([[project_vision_and_discipline]]) — the existing idiom *is* sufficient.

### Recorded as separate future work (NOT this lane)

- **Complete the engine's direct-LR eliminator** (the dominated option B) is a legitimate *general*
  capability gap — it would also fix `block_event_expression` (`grammars/systemverilog.ebnf:681`, a second
  un-eliminated direct-LR rule found in `.8.1`) and make the book's claim true. It is recorded as a future
  general-engine backlog idea, sequenced after the locked program; it is **not** required for `H.12.5.8`.
- **Book reconciliation** (`docs/book/src/developer-architecture.md` overclaims that
  `expr := expr "+" term | term` is auto-eliminated): the honest fix in the A wave is to **narrow** that
  claim to "PGEN eliminates left recursion expressed as an indirect rule-reference chain; write direct
  binary-operator recursion with the `next (OP next)*` precedence-cascade idiom." Do it in `.8.3`'s wave
  ([[feedback_regex_book_live]]).

## The authority: IEEE 1800-2017 Table 16-3 (§16.12) — sequence & property operator precedence

Extracted tools-first from the vendored LRM (`docs/systemverilog/2017/`). **Highest (tightest) → lowest
(loosest):**

| # | Sequence ops | Property ops | Associativity |
|---|---|---|---|
| 1 | `[*]` `[=]` `[->]` (repetition) | — | unary (postfix) |
| 2 | `##` (cycle delay) | — | **left** |
| 3 | — | `throughout` | **right** |
| 4 | `within` | — | **left** |
| 5 | `intersect` | — | **left** |
| 6 | — | `not`, `nexttime`, `s_nexttime` | unary (prefix) |
| 7 | `and` | `and` | **left** |
| 8 | `or` | `or` | **left** |
| 9 | — | `iff` | **right** |
| 10 | — | `until` `s_until` `until_with` `s_until_with` `implies` | **right** |
| 11 | — | `\|->` `\|=>` `#-#` `#=#` | **right** |
| 12 | — | `always` `s_always` `eventually` `s_eventually` | unary (prefix) |
| 13 | — | `if`-`else`, `case`, `accept_on`, `reject_on`, `sync_accept_on`, `sync_reject_on` | unary (prefix) |

LRM notes: *"The precedence for the strong and weak sequence operators is not defined because these
operators require parentheses."* and *"The operators described in Table 11-2 [ordinary expression
operators] have higher precedence than the sequence and property operators."* `and`/`or` are
**bifunctional** (same precedence/associativity at sequence and property level). Sequence operators
(1–5) all bind tighter than the property-only operators.

> ⚠️ `.8.3` must re-confirm the exact *production operands* per **Annex A.2.10** before encoding (Table 16-3
> gives precedence/associativity; A.2.10 gives which operand type — `expression_or_dist` vs `sequence_expr`
> vs `property_expr` — sits on each side; e.g. `throughout` is `expression_or_dist throughout sequence_expr`,
> and the implication ops are `sequence_expr |-> property_expr`). Verify against the LRM, the authority,
> per [[project_ebnf_is_single_source_of_truth]].

## The cascade blueprint (one rule per precedence level; left → `next (op next)*`, right → `next (op level)?`)

**Sequence layer** (replaces the flat `sequence_expr`, loosest entry first; each level references the next
tighter; preserve the existing non-binary forms — `first_match(...)`, `clocking_event sequence_expr`, the
paren/instance/`expression_or_dist (boolean_abbrev)?` operands — at the unary base):

```
sequence_expr        := seq_or_expr
seq_or_expr          := seq_and_expr        ( or        seq_and_expr )*          # 8 left
seq_and_expr         := seq_intersect_expr  ( and       seq_intersect_expr )*    # 7 left
seq_intersect_expr   := seq_within_expr     ( intersect seq_within_expr )*       # 5 left
seq_within_expr      := seq_throughout_expr ( within    seq_throughout_expr )*   # 4 left
seq_throughout_expr  := expression_or_dist throughout seq_throughout_expr        # 3 right (A.2.10 LHS = expression_or_dist)
                      | seq_delay_expr
seq_delay_expr       := ( cycle_delay_range )? seq_unary ( cycle_delay_range seq_unary )*   # 2 ## left (+ leading-## head form)
seq_unary            := expression_or_dist ( boolean_abbrev )?                    # 1 repetition (postfix, existing)
                      | sequence_instance ( sequence_abbrev )?
                      | lparen sequence_expr ( comma sequence_match_item )* rparen ( sequence_abbrev )?
                      | kw_first_match lparen sequence_expr ( comma sequence_match_item )* rparen
                      | clocking_event sequence_expr
```

**Property layer** (replaces the flat `property_expr_sv_2017`/`_sv_2023`, loosest entry first):

```
property_expr_sv_2017 := prop_guard
prop_guard   := accept_on(...) prop_guard | reject_on(...) prop_guard | sync_accept_on(...) prop_guard
              | sync_reject_on(...) prop_guard | if (...) prop_guard ( else prop_guard )? | case(...)…endcase   # 13 unary
              | prop_temporal
prop_temporal:= ( always | s_always (range)? | eventually (range)? | s_eventually ) prop_temporal            # 12 unary prefix
              | prop_impl
prop_impl    := prop_until ( ( seq_implication_op ) prop_impl )?                                              # 11 right; LHS seq forms (sequence_expr |-> property_expr) per A.2.10
              | sequence_expr ( |-> | |=> | #-# | #=# ) prop_impl
prop_until   := prop_iff ( ( until | s_until | until_with | s_until_with | implies ) prop_until )?            # 10 right
prop_iff     := prop_or  ( iff prop_iff )?                                                                    # 9 right
prop_or      := prop_and ( or prop_and )*                                                                     # 8 left
prop_and     := prop_not ( and prop_not )*                                                                    # 7 left
prop_not     := not prop_not | nexttime (range)? prop_not | s_nexttime (range)? prop_not | prop_primary       # 6 unary prefix
prop_primary := sequence_expr | strong( sequence_expr ) | weak( sequence_expr ) | ( property_expr )
              | property_instance | clocking_event property_expr                                              # base (sequences bind tightest)
```

(Right-assoc levels use `next ( op level )?` so they fold right; left-assoc use `next ( op next )*`. The
named op-rule idiom from [[feedback_quantified_group_extraction]] applies if an inline alternation lead is
needed for the iteration positional model.)

## Implementation sub-questions for `.8.3` (resolve carefully — quality over speed)

1. **AST-shape / return annotations.** The current flat branches carry `{kind:"and", lhs, rhs}` etc. The
   cascade emits `next (op next)*` lists. Decide: preserve the binary `{kind, lhs, rhs}` shape via a
   left/right-fold return annotation (the `binary_operator` idiom — keep schema stable if achievable), or
   accept a shape change (schema bump + lockstep ceremony + ledger row, the `SV-0002`/`SV-0004` precedent).
   Prefer shape-preserving if the fold annotation supports it; this is the dominant `.8.3` decision.
2. **Operand types per Annex A.2.10** (re-confirm against the LRM): `throughout` LHS = `expression_or_dist`;
   implication/`#-#`/`#=#` LHS = `sequence_expr`, RHS = `property_expr`; `or_assign`/`sequence_or_assign`.
3. **Both profiles.** `property_expr_sv_2017` AND `property_expr_sv_2023` (`:4055`/`:4126`) — the 2023
   variant differs only in `( constant_expression )?` vs `( cycle_delay_const_range_expression )?` on a few
   nexttime/eventually forms; the cascade must mirror both.
4. **`boolean_abbrev` `[ ]` interaction.** The `-0105` bracket-restoration fix lives in the repetition
   (level 1) operand — keep it intact; the cascade only re-homes it under `seq_unary`.
5. **EBNF meta-grammar lockstep** ([[feedback_ebnf_meta_grammar_lockstep]]): if the restructure introduces
   no new EBNF *construct* (only new rules using existing forms), `ebnf.ebnf` needs no change — verify.
6. **Sibling `block_event_expression`** (`:681`, also un-eliminated direct-LR): decide whether to fix in
   the same wave (same idiom) or a sibling leaf; it is out of strict `H.12.5.8` scope but is the same class.
7. **Verification (mandatory):** regen `make focus_systemverilog` + rebuild; decisive A/B; the full infix
   matrix now PARSES (`a ##1 b`/`and`/`or`/`intersect`/`within`/`throughout`/`until`-family/implication) with
   correct precedence/associativity (spot-check `a or b and c` ⇒ `a or (b and c)`); SV external corpus 14/14;
   **global cert UNKNOWN** (expect the infix M3 rules to now WITNESS — `56` should DROP) deterministic at
   seeds 0/7/42 `spf=0`; the 6 fully-certified grammars byte-identical; `clippy_on_rust_change` clean;
   release/schema/ledger ceremony per the outcome of sub-question 1; book reconciliation in-wave.

## VERIFICATION (this slice)

- Design/decision only; no code/grammar/generated change ⇒ no clippy, no regen, no gate run.
- The precedence authority is the LRM (Table 16-3, tools-extracted from `docs/systemverilog/2017/`);
  `.8.3` re-confirms operand types against Annex A.2.10.
- No live-status row changes: SV remains `Mostly Done`, cert `UNKNOWN=56`.

## OUTCOME

`H.12.5.8.2` done (decision A + Table 16-3 + cascade blueprint). **`H.12.5.8.3`** (`pending`, new frontier):
implement the cascade restructure in `grammars/systemverilog.ebnf` per this blueprint + sub-questions,
regen, verify, and run the full lockstep ceremony. Given its size and signoff-criticality, `.8.3` is a
focused implementation leaf best taken with fresh context.
