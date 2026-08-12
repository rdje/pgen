# `ENGINE-UNIVERSAL-SERVICES.13` acceptance (a) — the per-cycle adjudication

Two re-runnable instruments plus their probe inputs. Read-only: nothing here changes a grammar, the
codegen, or a generated parser.

| script | question | answer at the time of writing |
|---|---|---|
| `canonicalize_lint_cycles.py` | how many DISTINCT cycles is `left_recursion_unhandled=30`? | SV **30 rows → 7 cycles**, `ebnf` **5 → 3**; **10** total, not 35 |
| `adjudicate.py` | which of them cost real text? | **8 of 10** REJECT a spec-grounded input; 2 are DEAD-BUT-COVERED |

## 1. The denominator was 3.5× too big

`detect_left_recursion` starts a DFS from every rule, so one 12-rule cycle is reported 12 times —
once per rule that can start it. `left_recursion_unhandled=30` counts **rule rows**. Canonicalising
each reported path by rotation gives the real worklist:

```text
grammars/systemverilog.ebnf: 30 reported rule rows -> 7 DISTINCT cycles
grammars/ebnf.ebnf:           5 reported rule rows -> 3 DISTINCT cycles
```

⛔ The canonicaliser refuses rather than guesses: every row must be the shape the lint guarantees
(`rule -> … -> rule`, first == last == the reporting rule), and the headline count must equal the
number of rows actually parsed — so a truncated dump or a changed message aborts before a number is
printed.

## 2. The adjudication — 8 of 10 cost real text

```text
SystemVerilog — 7 distinct cycles
  SV-1  cast/call    sv_2017  REJECT  size cast whose casting_type is a constant_function_call
  SV-5  cast/call    sv_2023  REJECT  same, through the sv_2023 method-call receiver
  SV-2  cast/const   sv_2017  REJECT  numeric size cast in a constant expression
  SV-3  cast/const   sv_2023  REJECT  same, through constant_primary_sv_2023
  SV-6  property     sv_2017  REJECT  property_expr implies property_expr, non-expression left operand
  SV-7  property     sv_2023  REJECT  same, through the sv_2023 property cascade
  SV-4  class-scope  sv_2023  ACCEPT  DEAD-BUT-COVERED — data_type (branch 1/2) served the text

ebnf — 3 distinct cycles (arm 1 = hand-written frontend, arm 2 = generated meta-parser)
  EBNF-1  rc=0 / rc=1   -> $1 + $2
  EBNF-2  rc=0 / rc=1   -> $1 ? $2 : $3
  EBNF-3  rc=0 / rc=0   -> $1.name — DEAD-BUT-COVERED (no member_access node in the arm-2 AST)
```

⭐ **The leaf's own caution is refuted by its own measurement.** `.13` opened saying *"the 30 are not
30 defects — for many the LRM may never put a legal string on that path."* Measured: 8 of the 10
distinct cycles reject text the standard licenses.

⭐ **And the fix surface is 3 knots, not 30 cycles.** SV-1/2/3/5 all pass through the single edge
`casting_type → constant_primary`; SV-6/7 through the single alternative `property_expr implies
property_expr`; EBNF-1/2/3 through `return_expression`. Whatever `.13` lands has to break three
knots, and the count of *reported rules* was never the size of the job.

## 3. Two verdicts that are not "pass" — and why they are EARNED, not asserted

`DEAD-BUT-COVERED` means the probe **accepts** while the cycle's own alternative stays unreachable,
the text being served by a sibling rule. An accepting probe proves nothing on its own — that is the
`SV-CORPUS-GRAD.13c.2b` trap in the other direction — so both rows carry mechanical evidence that
`adjudicate.py` re-checks on every run:

- **SV-4** — the traced run must show BOTH `Infinite recursion detected in rule
  'incomplete_class_scoped_type'` and `Rule 'data_type_or_incomplete_class_scoped_type_sv_2023'
  selected branch 1/2`. Guard rejected the recursive alternative; `data_type` served the text.
- **EBNF-3** — the arm-2 AST must contain `property_access_suffix` and **no** `member_access` node,
  i.e. `$1.name` is a positional reference with a property suffix, never the left-recursive
  `member_access_return`.

If either stops reproducing the script exits non-zero rather than printing a green row.

## 4. Traps this measurement walked into (both are recorded because they nearly published)

1. **`a -> b` in a property looks like a passing probe and exercises nothing.** `->` is also an
   ordinary binary expression operator (A.8.6), so `sequence_expr` consumes it and
   `prop_primary_sv_2017 selected branch 1/30` — the left-recursive alternative is branch 26 and
   never fires. The probe had to force a non-expression left operand (`(a |=> b) -> c`).
2. **`ebnf_dual_run_diff` exits 0 on an arm-2 rejection unless `--emit-ast-json` is passed.**
   Without it only arm 1 is reported. A first draft of `adjudicate.py` omitted the flag and read
   three green rows where two are rejections — the same *"an under-featured invocation yields a
   clean-looking result"* shape `CI-PARITY-GATE-ROT.24` documents.

## Re-run

```bash
python3 docs/tasks/artifacts/engine_universal_services/lr_cycle_adjudication/canonicalize_lint_cycles.py
python3 docs/tasks/artifacts/engine_universal_services/lr_cycle_adjudication/adjudicate.py
```

Both exit non-zero if their claim stops holding. When `.13`'s fix lands, `adjudicate.py` is the
before→after oracle: the six REJECT rows must become ACCEPT, the two arm-2 `rc=1` rows must become
`rc=0`, and the two DEAD-BUT-COVERED rows must then be served by their own declared derivations
rather than by a sibling.
