# `SV-CORPUS-GRAD.13c.2b` — the constant size cast, diagnosed

Three re-runnable instruments, each answering one question the leaf asked. All are read-only and
touch neither the grammar nor the generated parsers.

| script | question | verdict at the time of writing |
|---|---|---|
| `constant_primary_lrm_alternative_audit.py` | is an LRM alternative MISSING from the grammar? | **no** — 15/15, 16/16, 5/5, order-identical |
| `casting_type_edge_matrix.sh` | which edge is the discriminator? | `casting_type`'s `constant_primary` alternative, 3 ACCEPT / 2 REJECT |
| `corpus_row_cast_bisect.py` | does the fix unblock exactly the 2 attributed rows? | **yes** — both flip REJECT → ACCEPT with only the cast prefixes removed |

## 1. The alternative-list audit — the leaf's own worry, answered NO

The leaf opened suspecting a missing alternative: *"a constant-expression path that is missing ONE
`constant_primary` alternative is unlikely to be missing only that one."* It is missing **none**.

```text
=== constant_primary_sv_2017 — shipped 15 alternatives vs LRM-extracted 15   (all ==, in order)
=== constant_primary_sv_2023 — shipped 16 alternatives vs LRM-extracted 16   (all ==, in order)
=== casting_type            — shipped 5  alternatives vs LRM-extracted 5     (all ==, in order)
VERDICT: AGREE — no missing LRM alternative
```

Arm 2 is `grammars/systemverilog_lrm_profiled_generated.ebnf`, machine-extracted from IEEE 1800
Annex A — so this is a fidelity **proof**, not an assertion. `constant_cast` is alternative 12 of
`constant_primary_sv_2017` and has been all along; the derivation is declared and simply
unreachable.

## 2. The edge matrix — one grammar edge, not one construct class

```text
control_size_cast_in_statement.sv            ACCEPT   8'(1) in a STATEMENT expression
control_simple_type_cast_in_constant.sv      ACCEPT   int'(1) — casting_type = simple_type
control_param_name_cast_in_constant.sv       ACCEPT   W'(1)  — casting_type = ps_type_identifier
defect_constant_size_cast.sv                 REJECT   8'(1) in a CONSTANT expression
defect_constant_size_cast_corpus_shape.sv    REJECT   512'({…}) — the OpenTitan corpus shape
```

⭐ Rows 2 and 3 are what make the diagnosis precise, and both were **absent** from the leaf's
original framing. A size cast in a constant expression is fine when its casting type is a keyword
(`int`) or a name (`W`) — those resolve on `casting_type` branch 1/5 (`simple_type`). Only a
**numeric** size leaves `constant_primary` as the sole viable alternative, and that is the one the
indirect left-recursive cycle makes unreachable at the seed position.

## 3. The corpus bisect — "unblocks 2 rows" measured, not attributed

```text
top_darjeeling_rnd_cnst_pkg.sv   22 size casts removed   REJECT furthest_position=5899 → ACCEPT
top_earlgrey_rnd_cnst_pkg.sv     11 size casts removed   REJECT furthest_position=5906 → ACCEPT
VERDICT: the numeric size cast is the SOLE blocker of both rows
```

The transformation deletes only the `N'` prefix, so `512'({ … })` becomes the legal parenthesised
concatenation `({ … })` and every other byte of both files is still parsed at full strength. That is
why the flip is evidence about *this* construct rather than about a mutilated input.

## Re-run

```bash
python3 docs/tasks/artifacts/sv_corpus_grad/constant_size_cast/constant_primary_lrm_alternative_audit.py
bash    docs/tasks/artifacts/sv_corpus_grad/constant_size_cast/casting_type_edge_matrix.sh
python3 docs/tasks/artifacts/sv_corpus_grad/constant_size_cast/corpus_row_cast_bisect.py
```

Each exits non-zero if its claim stops holding. When `ENGINE-UNIVERSAL-SERVICES.13` lands, the
matrix and the bisect are exactly the two that must be re-baselined — the matrix's REJECT arms
become ACCEPT, and the bisect's "before" stops being a rejection.
