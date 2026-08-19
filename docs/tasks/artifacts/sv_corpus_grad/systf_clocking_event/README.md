# `SV-CORPUS-GRAD.13c.2s` — a greedy `( X )*` in front of an **optional** `X`-shaped tail

`.13c.2r` fixed a starving star where the starved element was **mandatory**, so the rule matched
nothing and **no verdict moved**. This leaf fixes the same mechanism where the starved element is
**optional** — so each rule matched everything *except* the form the tail exists for, and the
verdict did move. Four sites, one device.

| file | what it is |
|---|---|
| `probe_matrix.sh` | the 12-case matrix that chose the fix; runs on the shipped grammar or on any arm |
| `sweep_arms.py` | every pinned reproducer × every profile it declares × each arm, on **both** axes (verdict and typed AST) |

## The four sites

| production | what could not be reached | demonstrated at source level? |
|---|---|---|
| `system_tf_call` alt 3 | the **clocking-event** argument of A.8.2 | ✅ `$rose(a, @(posedge clk))` REJECT → ACCEPT |
| `sequence_list_of_arguments` | a **named** argument after a positional one | ✅ `sq(a ##1 b, .q(d))` REJECT → ACCEPT |
| `property_list_of_arguments` | same | ✅ `pr(a ##1 b, .q(d))` REJECT → ACCEPT |
| `let_list_of_arguments` | same | ⛔ **no** — see *the masked three* below |

## The masked three, and why measuring beat reading

`.13c.2r`'s census flagged the three `*_list_of_arguments` rules as *unconfirmed* candidates: they
carry the shape, but a source-level probe showed the construct **parsing**. Entering each rule
directly (`--interpret-entry-rule`) settles it:

| rule | `a, .b(2)` | `.b(2)` | `a, c` |
|---|---|---|---|
| `let_list_of_arguments` | **REJECT** | ACCEPT | ACCEPT |
| `property_list_of_arguments` | **REJECT** | ACCEPT | ACCEPT |
| `sequence_list_of_arguments` | **REJECT** | ACCEPT | ACCEPT |
| `list_of_arguments` (the general one) | ACCEPT | — | — |

⇒ all three starve, and the **general** production rescues the call site. That rescue holds only
while every positional argument is *also* a plain expression. The moment one is not — `a ##1 b` is a
`sequence_actual_arg` and not an expression — both routes fail and the construct is unparseable.
That is the probe that turned three "unconfirmed candidates" into a shipped fix, and it is why the
`let` site is repaired with the honest note that it has **no** demonstrated flip: `let_actual_arg :=
expression`, so the general route always covers it.

⭐ Reachability was checked before the rules were touched: on `sq(a ##1 b)` the tree carries
`sequence_list_of_arguments`' own `ordered_tail`/`named_tail` fields; on `sq(a, .q(b))` it carries
the general `list_of_arguments` shape. The rule is live — just not for the mixed form.

## Three spellings, measured identical

| arm | spelling |
|---|---|
| A | `( comma !( clocking_event ) ( expression )? )*` — semantically exact |
| **B (shipped)** | `( comma !at_sign ( expression )? )*` — one-token peek, refuses *before* the expression |
| C | `( comma ( expression )? !at_sign )*` — one-token peek at the end (the `.13c.2r` placement) |

All three: identical on **155** pinned reproducer checks (`widen=2 narrow=0 shape=0`) and on 12
targeted probes. B ships because `at_sign` is the unique first token of **both** `clocking_event`
alternatives — a sound necessary condition — and it fails earliest. The extra texts B refuses and A
does not (a comma followed by an `@` that is not a clocking event) were rejected either way, which
`invalid_systf_bare_at.sv` now pins.

## ⛔ The trap this fix fell into, recorded because it cost a regeneration

The fix was first written with its explanatory comment **between two alternatives, at column 0**.
The EBNF frontend ends a rule there and silently discards every alternative after it
([[a-column-0-comment-inside-a-rule-body-deletes-the-following-alternatives]]). `--lint-grammar`
reported `1610 rules` and every counter zero — **byte-identical to the healthy grammar** — while the
reproducer went back to REJECT. Only the behavioural probe saw it. Every comment in this fix now
sits **above** its rule, and the grammar says so at the site.

## Re-run

```bash
bash    docs/tasks/artifacts/sv_corpus_grad/systf_clocking_event/probe_matrix.sh          # 12/12
bash    docs/tasks/artifacts/sv_corpus_grad/systf_clocking_event/probe_matrix.sh OLD.ebnf # goes RED 4/12
python3 docs/tasks/artifacts/sv_corpus_grad/systf_clocking_event/sweep_arms.py OLD.ebnf grammars/systemverilog.ebnf
```

`OLD.ebnf` is any pre-fix revision, e.g. `git show <sha>:grammars/systemverilog.ebnf > OLD.ebnf`.
