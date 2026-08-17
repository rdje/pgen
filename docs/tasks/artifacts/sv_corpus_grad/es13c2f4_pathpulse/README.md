# `SV-CORPUS-GRAD.13c.2f` slice 4 — the PATHPULSE pair (`PGEN-SV-CORPUS-GRAD-0221`, 2026-08-17)

What IEEE 1800 A.7.5's two `PATHPULSE$` tokens cost, and what proves the repair.

| file | what it is |
|---|---|
| `corpus_transitions.tsv` | every row whose `(observed, adjudication)` pair moved, **joined over the MANIFEST** |
| `arm_red_control.py` / `.txt` | the RED control for the four new `arm` claims — runs the repro runner's OWN checker |

## Why the join is over the manifest

⛔ **A corpus delta joined on the pass/fail verdict measures only the rows that CROSSED it.**
Adjudication classes are decided positionally as well, so a row can leave the defect bar without its
verdict moving. `.13h` wrote that rule down one slice earlier; this slice reproduces it exactly —
the pass/fail join finds **2** transitions and the manifest join finds **3**. The third is
`verilator/test_regress/t/t_specparam.v`, which still FAILS and reclassified
`unexplained_rejects_valid → explained_svpp_conditional` because the parse now runs past every
`PATHPULSE$` specparam and dies on an `` `ifdef `` at byte 1170.

## The measured effect

- corpus `pass 9774 → 9776` over 16 336 files, `timeout=0 crash=0`, **3 transitions, all
  improvements, 0 regressions**;
- axis-2 bar **302 → 300** with ADJUDICATED/ROUTED/NO-VERDICT/DARK unmoved, and
  **accepts-invalid byte-identical at 21**;
- `verilog_2005` lane `pass 2181 → 2182`, unexplained `68 → 67`, accepts-invalid byte-identical at
  **14**;
- parse-cost binding counters `+0 / +0 / +0`, family share `2.741 %` re-derived exactly.

## Why an `arm` and not a verdict

`specparam PATHPULSE$a$y = 3;` ACCEPTED before the fix *and* after it — before, because the
production was unreachable and the text fell through to the ordinary `specparam_assignment`; after,
because that fall-through is still legal for a single limit value (`.13c.2h`). So the verdict is not
the evidence: the declared AST `kind` is. `arm_red_control.py` runs each claim against an AST it must
refuse as well as one it must accept — **8/8 as declared, 4 of them RED** — because a control never
seen RED is not known to work (`docs/CLAIM_VERIFICATION.md` §3 leg 2).

## Reproducing

```bash
python3 docs/tasks/artifacts/sv_corpus_grad/es13c2f4_pathpulse/arm_red_control.py
python3 stimuli/sv/run_adjudication_repros.py            # 39 rows, 12 armed, failures=0
python3 docs/tasks/artifacts/sv_corpus_grad/nonterminal_as_literal_sweep.py
```

⚠️ The corpus baselines `results*.tsv` / `positions*.tsv` are **untracked**, so the before-side of
`corpus_transitions.tsv` cannot be re-derived after promotion — which is why the transitions are
banked here rather than left to a re-run. One casualty of that, stated rather than hidden: the
**before** furthest-position of `t_specparam.v` was not captured before `positions.tsv` was promoted,
so only its after-position (1170) is measured; what the row's before-basis records is that the parse
stopped where the preprocessor could not have altered the text.
