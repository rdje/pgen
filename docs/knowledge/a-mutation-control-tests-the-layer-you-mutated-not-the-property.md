---
id: a-mutation-control-tests-the-layer-you-mutated-not-the-property
title: A mutation-based RED control tests the layer you mutated, not the property you care about — once a fix has two layers it exits 0 and reports "the defect is unreachable" about a defect reached every time
answers:
  - "my RED control passes the mutant instead of failing it — is my fixture wrong"
  - "how do I prove a regression fixture actually reaches the bug it claims to reach"
  - "should a red control mutate the fix or replay the old code"
  - "my probe says the defect is not reachable but I reproduced it an hour ago"
  - "how do I keep a red control working as the fix grows more guards"
  - "why did my mutant exit 0 when I disabled the guard"
  - "a control that never fails - how do I tell it apart from one that cannot fail"
tags: [controls, evidence, regression, probes, red-arms, instruments, git]
date: 2026-08-17
status: current
evidence: |
  ENGINE-UNIVERSAL-SERVICES.30. Two `ZeroDivisionError` crashes in
  `stimuli/sv/corpus_parse_cost.py` were fixed with a shared `pct()` guard PLUS an `if lr_total:`
  branch at the call site. The RED control mutated only the guard (`if not denominator:` ->
  `if False:`) and the mutant exited **0**, so the arm reported "these fixtures do not reach the
  defect, so arms 1-3 prove nothing" — about fixtures that reach it every single time. Replaced by
  replaying the newest historical blob still containing the unguarded expression
  (`git rev-list HEAD -- <file>`, first blob matching -> `54deff5d`), which dies with
  ZeroDivisionError rc=1 as required. A second defect in the same arm: the replay must live two
  directories below the repo root, because the instrument computes
  `ROOT = dirname(__file__)/../..` — a copy under `rust/target/` resolved ROOT to `rust/` and died
  at exit 2, which is indistinguishable from "not reachable".
reverify: "bash docs/tasks/artifacts/engine_universal_services/es30_zero_denominator/probe.sh   # 8/8, arm 5 replays the pre-fix blob and must report ZeroDivisionError"
---

A RED control exists to answer one question: **does my fixture actually reach the defect?** Without
it, a suite of green arms is consistent with "the bug is fixed" *and* with "my fixture never
exercised the bug." The usual way to build one is to break the fix on purpose and demand a failure.

⛔ **That method silently stops working the moment a fix has more than one layer.**

## The measured instance

The fix for a `ZeroDivisionError` had two parts, both deliberate:

```python
def pct(numerator, denominator, digits=1):
    if not denominator:                 # layer 1 — the shared guard
        return "n/a"
    return f"{100.0 * numerator / denominator:.{digits}f} %"
...
if lr_total:                            # layer 2 — the call site
    A(f"… **{pct(lr_committed, lr_total, 3)}**. The elimination machinery is …")
else:
    A("⚠️ This sample contains NO left-recursion-family entries …")
```

The control mutated layer 1 and expected a crash:

```bash
sed 's|^    if not denominator:$|    if False:|' "$INSTR" > "$MUT"   # disable the guard
python3 "$MUT" --manifest one_trivial_file.tsv --outdir out          # expect ZeroDivisionError
```

It exited **0**. Layer 2 still short-circuits, so the division is never reached. The arm then
printed:

> ✗ the pre-fix mutant exited 0 without ZeroDivisionError — these fixtures do not reach the defect,
> so arms 1-3 prove nothing

which is exactly backwards: the fixture (`module m; endmodule`) reaches the defect every time.

## Why this is worse than an ordinary broken test

- It fails **toward disarmament**. The natural next move is to doubt the fixture and go hunting for
  a "better" one — i.e. to spend the effort on the half that was already correct.
- Its message is **specific and confident**. A vague failure invites investigation; this one hands
  you a wrong diagnosis in the vocabulary of the thing you were checking.
- It **degrades with maintenance**. The control is written when the fix has one layer and quietly
  stops discriminating the day someone adds a guard — which is the day the code got safer, so
  nothing else looks suspicious either.

⇒ same family as [[a-check-whose-inputs-all-pass-has-not-been-tested]], but the mechanism is
different: there the inputs could not fail; here the *subject* was never executed.

## What to do instead — replay the real pre-fix code

The defect existed in a committed blob. Use that blob; it cannot drift out of step with the fix,
however many layers the fix grows:

```bash
NEEDLE='100.0 \* lr_committed / lr_total'          # the unguarded expression itself
for c in $(git rev-list HEAD -- "$INSTR"); do
  if git show "$c:$INSTR" | grep -qE "$NEEDLE"; then PREFIX_COMMIT="$c"; break; fi
done
git show "$PREFIX_COMMIT:$INSTR" > "$MUT"          # the REAL pre-fix instrument
```

Search for the **newest blob still containing the defect** rather than pinning a hash or taking
`HEAD~1`: it works before the fix is committed (HEAD still has it) and after (HEAD~n does), and it
survives unrelated commits to the same file.

⚠️ **If no such blob exists — a shallow clone — report NOT EVALUATED and FAIL.** An unreachable
history is not evidence that the fix works, and arm 5 is the only thing standing between "the
fixtures pass" and "the fixtures prove something."

## ⛔ And put the replay where the code expects to live

A script that resolves its own root from `__file__` will not survive being copied elsewhere:

```python
ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
```

The first replay was written to `rust/target/…`, so `ROOT` resolved to `rust/`, every corpus path
missed, and the instrument died at **exit 2** — which the arm read as "did not crash the way I
expected", i.e. as *not reachable* again. Write the replay two directories below the repo root
(here, beside the original) and delete it in a `trap`.

⭐ The general form: **a control's own environment is part of the control.** Both failures in this
arm produced the same wrong conclusion by different routes, and neither looked like a bug in the
control.
